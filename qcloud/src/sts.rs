use crate::{
    error::QCloudError,
    http_client::CLIENT_INSTANCE,
    secrets::Secrets,
    signature::{generate_sign_content, object_to_str, SignAlgorithm},
};
use chrono::Utc;
use hyper::{
    body::{aggregate, Buf},
    Body, Method, Request,
};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::{self, json};
use urlencoding;

const STS_DOMAIN: &str = "sts.tencentcloudapi.com";

pub struct PolicyScope<'a> {
    pub action: &'a str,
    pub bucket: &'a str,
    pub region: &'a str,
    pub prefix: &'a str,
}

impl<'a> From<(&'a str, &'a str, &'a str, &'a str)> for PolicyScope<'a> {
    fn from(tuple: (&'a str, &'a str, &'a str, &'a str)) -> Self {
        Self {
            action: tuple.0,
            bucket: tuple.1,
            region: tuple.2,
            prefix: tuple.3,
        }
    }
}

#[derive(Serialize, Debug)]
struct PolicyPrincipal<'a> {
    qcs: &'a str,
}

#[derive(Serialize, Debug)]
pub struct PolicyStatement<'a> {
    action: &'a str,
    effect: &'a str,
    principal: PolicyPrincipal<'a>,
    resource: String,
}

#[derive(Serialize, Debug)]
pub struct PolicyDescription<'a> {
    version: &'a str,
    statement: Vec<PolicyStatement<'a>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all(deserialize = "PascalCase"))]
#[allow(dead_code)]
pub struct StsCredentialResponse {
    pub tmp_secret_id: String,
    pub tmp_secret_key: String,
    pub token: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all(deserialize = "PascalCase"))]
#[allow(dead_code)]
pub struct StsSuccessResponse {
    pub credentials: StsCredentialResponse,
    pub expiration: String,
    pub expired_time: u64,
    pub request_id: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all(deserialize = "PascalCase"))]
#[allow(dead_code)]
pub struct StsErrorResponseInner {
    pub code: String,
    pub message: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all(deserialize = "PascalCase"))]
#[allow(dead_code)]
pub struct StsErrorResponse {
    pub error: StsErrorResponseInner,
    pub request_id: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum StsResponse {
    Success(StsSuccessResponse),
    Error(StsErrorResponse),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all(deserialize = "PascalCase"))]
#[allow(dead_code)]
pub struct StsResponseWrapper {
    pub response: StsResponse,
}

pub async fn get_credential<'a>(
    secrets: &'a Secrets<'a>,
    policy: &'a PolicyDescription<'a>,
    region: &str,
    duration_seconds: usize,
) -> Result<StsResponseWrapper, QCloudError> {
    let policy_str = serde_json::to_string(policy).unwrap();
    let action = "GetFederationToken";
    let nonce: i32 = (OsRng.next_u32() % 10000 + 10000) as i32;
    let timestamp = Utc::now().timestamp();
    let encoded_policy = urlencoding::encode(&policy_str).into_owned();
    let mut params_map = json!({
        "SecretId": secrets.secret_id,
        "Timestamp": timestamp,
        "Nonce": nonce,
        "Action": action,
        "DurationSeconds": duration_seconds,
        "Name": "cos-sts-nodejs",
        "Version": "2018-08-13",
        "Region": region,
        "Policy": encoded_policy,
    });

    let params_map = params_map.as_object_mut().unwrap();

    let obj_str = object_to_str(&params_map);
    let obj_str = format!("POST{}/?{}", STS_DOMAIN, obj_str);

    let signed = generate_sign_content(secrets.secret_key, &obj_str, SignAlgorithm::Base64);

    params_map.insert("Signature".to_string(), signed.into());

    let request = Request::builder()
        .method(Method::POST)
        .uri(format!("https://{}/", STS_DOMAIN))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Host", STS_DOMAIN)
        .body(Body::from(serde_urlencoded::to_string(params_map).unwrap()))
        .unwrap();

    let res = CLIENT_INSTANCE.request(request).await?;

    let body = aggregate(res).await?;

    let res = serde_json::from_reader(body.reader())?;

    Ok(res)
}

pub fn get_policy(scope: Vec<PolicyScope>) -> PolicyDescription {
    let statement = scope
        .iter()
        .map(|item| {
            let short_bucket_name = &item.bucket[0..item.bucket.rfind("-").unwrap()];
            let app_id = &item.bucket[item.bucket.rfind("-").unwrap() + 1..];
            let mut resource = format!(
                "qcs::cos:{}:uid/{}:prefix//{}/{}/{}",
                item.region, app_id, app_id, short_bucket_name, item.prefix,
            );
            if item.action.eq("name/cos:GetService") {
                resource = "*".to_string();
            }
            PolicyStatement {
                action: item.action,
                effect: "allow",
                principal: PolicyPrincipal { qcs: "*" },
                resource,
            }
        })
        .collect();
    PolicyDescription {
        version: "2.0",
        statement,
    }
}

#[cfg(test)]
mod tests {
    use super::{get_credential, get_policy};
    use std::env;

    #[tokio::test]
    async fn it_works() {
        dotenv::dotenv().ok();

        let secret_id = env::var("SECRET_ID").unwrap();
        let secret_key = env::var("SECRET_KEY").unwrap();
        let bucket_name = env::var("IMAGES_BUCKET_NAME").unwrap();
        let bucket_region = env::var("BUCKET_REGION").unwrap();

        let secrets = crate::secrets::Secrets::new(&secret_id, &secret_key);

        let policy = get_policy(vec![(
            "name/cos:PutObject",
            bucket_name.as_str(),
            bucket_region.as_str(),
            "images/*",
        )
            .into()]);

        let res = get_credential(&secrets, &policy, &env::var("BUCKET_REGION").unwrap(), 600)
            .await
            .unwrap();

        println!("{:?}", res);
    }
}

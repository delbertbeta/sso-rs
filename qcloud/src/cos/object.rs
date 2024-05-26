use hyper::body::Incoming;
use hyper::{Method, Response};

use crate::{error::QCloudError, http_client::CLIENT_INSTANCE, secrets::Secrets};

use super::util::generate_request;

pub async fn head<'a>(
    secrets: &'a Secrets<'a>,
    bucket: &'a str,
    region: &'a str,
    path: &'a str,
) -> Result<Response<Incoming>, QCloudError> {
    let request = generate_request(
        Method::HEAD,
        format!("/{}", &path),
        bucket,
        region,
        secrets,
        None,
        None,
        None,
    );

    let res = CLIENT_INSTANCE.request(request).await?;

    Ok(res)
}

#[cfg(test)]
mod tests {
    use crate::{cos::object::head, secrets};
    use std::env;

    #[tokio::test]
    async fn it_works() {
        dotenv::dotenv().ok();

        let secret_id = env::var("SECRET_ID").unwrap();
        let secret_key = env::var("SECRET_KEY").unwrap();
        let bucket_name = env::var("IMAGES_BUCKET_NAME").unwrap();
        let bucket_region = env::var("BUCKET_REGION").unwrap();

        let secrets = secrets::Secrets::new(&secret_id, &secret_key);

        let res = head(&secrets, &bucket_name, &bucket_region, "test.jpg")
            .await
            .unwrap();

        println!("{:?}", res);
    }
}

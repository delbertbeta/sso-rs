use std::{collections::HashSet, env};

use qcloud::secrets::Secrets;
use tldextract::TldOption;
use url::Url;

pub const SESSION_COOKIE_KEY: &str = "delbertbeta-s-sso";
pub const RSA_PRIVATE_KEY_REDIS_KEY: &str = "rsa_private_key";

pub struct Envs {
    pub database_url: String,
    pub prod: bool,
    pub redis_url: String,
    pub rust_log: String,
    pub bucket_region: String,
    pub bucket_name: String,
    pub bucket_secret_id: String,
    pub bucket_secret_key: String,
    pub cdn_base_url: String,
}

lazy_static! {
    pub static ref PARSED_FRONTEND_URL: Url = {
        let front_end_url = std::env::var("FRONT_END_URL").expect("FRONT_END_URL not defined");
        Url::parse(&front_end_url).expect("FRONT_END_URL is invalid")
    };
    pub static ref ROOT_DOMAIN: String = {
        let domain_extractor = TldOption::default().build();
        let extracted_domain = domain_extractor
            .extract(
                PARSED_FRONTEND_URL
                    .domain()
                    .expect("FRONT_END_URL is invalid"),
            )
            .expect("FRONT_END_URL parse error");
        format!(
            "{}.{}",
            extracted_domain
                .domain
                .expect("FRONT_END_URL has not main domain"),
            extracted_domain
                .suffix
                .expect("FRONT_END_URL has not suffix")
        )
    };
    pub static ref ENVS: Envs = Envs {
        database_url: env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file"),
        prod: env::var("PROD").map_or(false, |_| true),
        redis_url: env::var("REDIS_URL").expect("REDIS_URL is not set in .env file"),
        rust_log: std::env::var("RUST_LOG")
            .unwrap_or_else(|_| "sso_rs=debug,tower_http=trace".into()),
        bucket_region: env::var("BUCKET_REGION").expect("BUCKET_REGION is not set in .env file"),
        bucket_name: env::var("BUCKET_NAME").expect("BUCKET_NAME is not set in .env file"),
        bucket_secret_id: env::var("BUCKET_SECRET_ID")
            .expect("BUCKET_SECRET_ID is not set in .env file"),
        bucket_secret_key: env::var("BUCKET_SECRET_KEY")
            .expect("BUCKET_SECRET_KEY is not set in .env file"),
        cdn_base_url: env::var("CDN_BASE_URL").expect("CDN_BASE_URL is not set in .env file"),
    };
    pub static ref SECRETS: Secrets<'static> =
        Secrets::new(&ENVS.bucket_secret_id, &ENVS.bucket_secret_key);
    pub static ref SUPPORT_IMAGE_TYPE: HashSet<&'static str> =
        HashSet::from(["gif", "bmp", "jpg", "jpeg", "png", "webp"]);
}

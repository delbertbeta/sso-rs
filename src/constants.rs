use std::{collections::HashSet, env};

use qcloud::secrets::Secrets;
use url::Url;

pub const SESSION_COOKIE_KEY: &str = "delbertbeta-s-sso";
pub const RSA_PRIVATE_KEY_REDIS_KEY: &str = "rsa_private_key";

pub struct Envs {
    pub database_url: String,
    pub prod: bool,
    pub redis_url: String,
    pub rust_log: String,
    pub cos_bucket_region: String,
    pub cos_bucket_name: String,
    pub qcloud_secret_id: String,
    pub qcloud_secret_key: String,
}

lazy_static! {
    pub static ref PARSED_FRONTEND_URL: Url = {
        let front_end_url = std::env::var("FRONT_END_URL").expect("FRONT_END_URL not defined");
        Url::parse(&front_end_url).expect("FRONT_END_URL is invalid")
    };
    pub static ref ENVS: Envs = Envs {
        database_url: env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file"),
        prod: env::var("PROD").map_or(false, |_| true),
        redis_url: env::var("REDIS_URL").expect("REDIS_URL is not set in .env file"),
        rust_log: std::env::var("RUST_LOG")
            .unwrap_or_else(|_| "sso_rs=debug,tower_http=trace".into()),
        cos_bucket_region: env::var("COS_BUCKET_REGION")
            .expect("COS_BUCKET_REGION is not set in .env file"),
        cos_bucket_name: env::var("COS_BUCKET_NAME")
            .expect("COS_BUCKET_NAME is not set in .env file"),
        qcloud_secret_id: env::var("QCLOUD_SECRET_ID")
            .expect("QCLOUD_SECRET_ID is not set in .env file"),
        qcloud_secret_key: env::var("QCLOUD_SECRET_KEY")
            .expect("QCLOUD_SECRET_KEY is not set in .env file"),
    };
    pub static ref SECRETS: Secrets<'static> =
        Secrets::new(&ENVS.qcloud_secret_id, &ENVS.qcloud_secret_key);
    pub static ref SUPPORT_IMAGE_TYPE: HashSet<&'static str> =
        HashSet::from(["gif", "bmp", "jpg", "jpeg", "png", "webp"]);
}

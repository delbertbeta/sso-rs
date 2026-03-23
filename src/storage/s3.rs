use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{config::Credentials, Client};

use crate::constants::ENVS;

pub async fn get_s3_client() -> Client {
    let sdk_config = aws_config::defaults(BehaviorVersion::v2024_03_28())
        .endpoint_url(&ENVS.bucket_endpoint)
        .region(Region::new(&ENVS.bucket_region))
        .credentials_provider(Credentials::from_keys(
            &ENVS.bucket_secret_id,
            &ENVS.bucket_secret_key,
            None,
        ))
        .load()
        .await;

    let s3_config = aws_sdk_s3::config::Builder::from(&sdk_config)
        .force_path_style(true)
        .build();

    Client::from_conf(s3_config)
}

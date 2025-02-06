use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{config::Credentials, Client};

use crate::constants::ENVS;

pub async fn get_s3_client() -> Client {
    let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
        .endpoint_url(&ENVS.bucket_endpoint)
        .region(Region::new(&ENVS.bucket_region))
        .credentials_provider(Credentials::from_keys(
            &ENVS.bucket_secret_id,
            &ENVS.bucket_secret_key,
            None,
        ))
        .load()
        .await;

    let client = Client::new(&config);

    client
}

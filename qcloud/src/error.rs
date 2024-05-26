use thiserror::Error;

#[derive(Error, Debug)]
pub enum QCloudError {
    #[error(transparent)]
    ParseError(#[from] serde_json::Error),

    #[error(transparent)]
    RequestError(#[from] hyper::Error),

    #[error(transparent)]
    ClientError(#[from] hyper_util::client::legacy::Error),
}

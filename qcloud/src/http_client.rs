use hyper::{client::HttpConnector, Body, Client};
use hyper_tls::HttpsConnector;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CLIENT_INSTANCE: Client<HttpsConnector<HttpConnector>, Body> = {
        let https = HttpsConnector::new();
        Client::builder().build::<HttpsConnector<HttpConnector>, Body>(https)
    };
}

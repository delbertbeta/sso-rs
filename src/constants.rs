use url::Url;

pub const SESSION_COOKIE_KEY: &str = "delbertbeta-s-sso";
pub const RSA_PRIVATE_KEY_REDIS_KEY: &str = "rsa_private_key";
lazy_static! {
    pub static ref PARSED_FRONTEND_URL: Url = {
        let front_end_url = std::env::var("FRONT_END_URL").expect("FRONT_END_URL not defined");
        Url::parse(&front_end_url).expect("FRONT_END_URL is invalid")
    };
}

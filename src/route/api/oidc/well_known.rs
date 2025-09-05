use axum::extract::Extension;
use axum::Json;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use openssl::pkey::{PKey, Private};
use openssl::rsa::Rsa;
use serde_json::{json, Value};

use crate::constants::PARSED_FRONTEND_URL;

#[derive(Clone)]
pub struct OidcKeys {
    pub private_key: Rsa<Private>,
    #[allow(dead_code)]
    pub public_key_pem: String,
}

impl OidcKeys {
    pub fn new() -> Self {
        let rsa = Rsa::generate(2048).unwrap();
        let public_key_pem = String::from_utf8(rsa.public_key_to_pem().unwrap()).unwrap();
        Self {
            private_key: rsa,
            public_key_pem,
        }
    }
}

pub async fn jwks_handler(Extension(oidc_keys): Extension<OidcKeys>) -> Json<Value> {
    let pkey = PKey::from_rsa(oidc_keys.private_key.clone()).unwrap();
    let rsa = pkey.rsa().unwrap();
    let n = rsa.n();
    let e = rsa.e();

    let jwk = json!({
        "kty": "RSA",
        "use": "sig",
        "alg": "RS256",
        "kid": "1", // In a real scenario, this would be a dynamic key ID
        "n": URL_SAFE_NO_PAD.encode(n.to_vec()),
        "e": URL_SAFE_NO_PAD.encode(e.to_vec()),
    });

    Json(json!({
        "keys": [jwk]
    }))
}

pub async fn openid_configuration_handler() -> Json<Value> {
    let issuer = PARSED_FRONTEND_URL.to_string();
    Json(json!({
        "issuer": &issuer,
        "authorization_endpoint": format!("{}api/oidc/authorize", issuer),
        "token_endpoint": format!("{}api/oidc/token", issuer),
        "userinfo_endpoint": format!("{}api/oidc/userinfo", issuer),
        "jwks_uri": format!("{}.well-known/jwks.json", issuer),
        "response_types_supported": ["code"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["RS256"],
        "scopes_supported": ["openid", "profile", "email"],
    }))
}

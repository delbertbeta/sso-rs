use std::collections::HashMap;

use base64::prelude::*;
use chrono::Utc;
use hmac_sha1::hmac_sha1;
use itertools::Itertools;
use serde_json::{Map, Value};

pub enum SignAlgorithm {
    Base64,
    Hex,
}

pub fn generate_key_time(max_age: i64) -> String {
    let start_timestamp = Utc::now().timestamp();
    let end_timestamp = start_timestamp + max_age;
    format!("{};{}", start_timestamp, end_timestamp)
}

pub fn generate_sign_content(secret_key: &str, content: &str, alg: SignAlgorithm) -> String {
    let bin = hmac_sha1(secret_key.as_bytes(), content.as_bytes()).to_vec();
    match alg {
        SignAlgorithm::Base64 => BASE64_STANDARD.encode(bin),
        SignAlgorithm::Hex => hex::encode(bin),
    }
}

pub fn object_to_str(obj: &Map<String, Value>) -> String {
    obj.keys()
        .sorted()
        .map(|key| {
            let val = match obj[key] {
                Value::Number(ref v) => v.to_string(),
                Value::String(ref v) => v.to_string(),
                _ => "".to_string(),
            };
            format!("{}={}", key, val)
        })
        .join("&")
}

pub fn generate_key_list_and_key_val_pair(
    header_map: &HashMap<String, String>,
) -> (String, String) {
    let mut key_list: Vec<String> = vec![];
    let mut key_val_pair: Vec<String> = vec![];
    header_map.keys().sorted().for_each(|key| {
        let val = &header_map[key];
        let key = key.to_lowercase();
        key_val_pair.push(format!("{}={}", &key, val));
        key_list.push(key);
    });
    (key_list.join(";"), key_val_pair.join("&"))
}

pub fn generate_http_string(
    method: &str,
    path: &str,
    http_params: &str,
    http_headers: &str,
) -> String {
    format!("{}\n{}\n{}\n{}\n", method, path, http_params, http_headers)
}

pub fn generate_string_to_sign(key_time: &str, http_string: &str) -> String {
    let sha1_str = openssl::sha::sha1(http_string.as_bytes());
    format!("sha1\n{}\n{}\n", key_time, hex::encode(sha1_str))
}

pub fn generate_sign(
    secret_id: &str,
    key_time: &str,
    header_list: &str,
    url_param_list: &str,
    signature: &str,
) -> String {
    format!("q-sign-algorithm=sha1&q-ak={}&q-sign-time={}&q-key-time={}&q-header-list={}&q-url-param-list={}&q-signature={}",
secret_id, key_time, key_time, header_list, url_param_list, signature)
}

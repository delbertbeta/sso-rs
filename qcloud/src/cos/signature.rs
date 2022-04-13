use base64::encode;
use chrono::Utc;
use hmacsha1::hmac_sha1;
use itertools::Itertools;
use serde_json::{Map, Value};

pub fn generate_key_time(max_age: i64) -> String {
    let start_timestamp = Utc::now().timestamp();
    let end_timestamp = start_timestamp + max_age;
    format!("{};{}", start_timestamp, end_timestamp)
}

pub fn generate_sign_content(secret_key: &str, content: &str) -> String {
    encode(hmac_sha1(secret_key.as_bytes(), content.as_bytes()).to_vec())
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

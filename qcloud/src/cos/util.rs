use std::collections::HashMap;

use hyper::{Body, Method, Request};

use crate::{
    secrets::Secrets,
    signature::{
        generate_http_string, generate_key_list_and_key_val_pair, generate_key_time, generate_sign,
        generate_sign_content, generate_string_to_sign, SignAlgorithm,
    },
};

pub fn generate_request(
    method: Method,
    path: String,
    bucket: &str,
    region: &str,
    secrets: &Secrets,
    body: Option<Body>,
    header_map: Option<HashMap<String, String>>,
    query_params: Option<HashMap<String, String>>,
) -> Request<Body> {
    let key_time = generate_key_time(600);
    let sign_key = generate_sign_content(secrets.secret_key, &key_time, SignAlgorithm::Hex);

    let mut header_map: HashMap<String, String> = header_map.unwrap_or(HashMap::new());
    #[allow(unused_variables, unused_mut)]
    let mut query_params: HashMap<String, String> = query_params.unwrap_or(HashMap::new());

    let host = format!("{}.cos.{}.myqcloud.com", bucket, region);
    let uri = format!("https://{}{}", host, path);

    header_map.insert("Host".to_string(), host);

    let (url_param_list, http_parameters) = generate_key_list_and_key_val_pair(&query_params);
    let (header_list, http_headers) = generate_key_list_and_key_val_pair(&header_map);

    let http_string = generate_http_string(
        &method.as_str().to_lowercase(),
        &path,
        &http_parameters,
        &http_headers,
    );
    let string_to_sign = generate_string_to_sign(&key_time, &http_string);
    let signature = generate_sign_content(&sign_key, &string_to_sign, SignAlgorithm::Hex);
    let sign = generate_sign(
        secrets.secret_id,
        &key_time,
        &header_list,
        &url_param_list,
        &signature,
    );

    header_map.insert("Authorization".to_string(), sign);

    let mut request = Request::builder().method(method).uri(uri);

    for (key, value) in header_map.into_iter() {
        request = request.header(key, value);
    }

    request.body(body.unwrap_or(Body::empty())).unwrap()
}

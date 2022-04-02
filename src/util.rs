use openssl::{
    error::ErrorStack,
    rsa::{Padding, Rsa},
};
use validator::ValidationError;

pub fn validate_padding_string(val: &str) -> Result<(), ValidationError> {
    let str = String::from(val);
    match str.starts_with(" ") || str.ends_with(" ") {
        true => Err(ValidationError::new("Can't starts or ends with space")),
        false => Ok(()),
    }
}

pub fn decrypt_rsa_content(
    private_key: String,
    content: String,
) -> Result<Option<String>, ErrorStack> {
    let private_key = Rsa::private_key_from_pem(private_key.as_bytes())?;
    let mut buf: Vec<u8> = vec![0; private_key.size() as usize];

    let content = base64::decode(content).unwrap_or(b"".to_vec());

    private_key.private_decrypt(&content, &mut buf, Padding::PKCS1)?;

    let decrypted = String::from_utf8(buf);

    match decrypted {
        Err(_) => Ok(None),
        Ok(val) => Ok(Some(val.trim_matches(char::from(0)).to_string())),
    }
}

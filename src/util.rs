use validator::ValidationError;

pub fn validate_padding_string(val: &str) -> Result<(), ValidationError> {
    let str = String::from(val);
    match str.starts_with(" ") || str.ends_with(" ") {
        true => Err(ValidationError::new("Can't starts or ends with space")),
        false => Ok(()),
    }
}

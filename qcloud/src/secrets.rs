pub struct Secrets<'a> {
    pub secret_id: &'a str,
    pub secret_key: &'a str,
}

impl<'a> Secrets<'a> {
    pub fn new(secret_id: &'a str, secret_key: &'a str) -> Self {
        Self {
            secret_id,
            secret_key,
        }
    }
}

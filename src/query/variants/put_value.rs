use crate::Key;

pub struct PutValueQuery {
    value: String,
}

impl PutValueQuery {
    pub fn new(value: String) -> (PutValueQuery, Key) {
        let key = Key::from_sha256(value.as_bytes());
        let query = PutValueQuery { value };
        (query, key)
    }

    pub fn value(&self) -> String {
        self.value.clone()
    }
}

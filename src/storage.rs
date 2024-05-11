pub struct Record {
    pub value: String,
    pub expires_at: f64,
}

#[derive(Default)]
pub struct LocalDHTStorage {
    records: Vec<Record>,
}

impl LocalDHTStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn put(&mut self, value: String, expires_at: f64) {
        self.records.push(Record { value, expires_at });
    }
}

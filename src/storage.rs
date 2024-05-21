use crate::Key;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Record {
    pub value: String,
    pub expires_at: f64,
}

#[derive(Debug, Default)]
pub struct LocalDHTStorage {
    records: HashMap<Key, Record>,
}

impl LocalDHTStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &Key) -> Option<&Record> {
        self.records.get(key)
    }

    pub fn put(&mut self, key: Key, record: Record) {
        self.records.insert(key, record);
    }
}

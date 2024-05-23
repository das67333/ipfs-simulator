use crate::{Key, PeerId, CONFIG};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct Record {
    pub data: RecordData,
    pub expires_at: f64,
}

#[derive(Debug, Clone, Serialize)]
pub enum RecordData {
    ProviderRecord { key: Key, providers: Vec<PeerId> },
}

impl Record {
    pub fn new_provider_record(self_id: PeerId, key: Key, curr_time: f64) -> Self {
        Self {
            data: RecordData::ProviderRecord {
                key,
                providers: vec![self_id],
            },
            expires_at: curr_time + CONFIG.record_expiration_interval,
        }
    }

    pub fn key(&self) -> Key {
        match &self.data {
            RecordData::ProviderRecord { key, .. } => key.clone(),
        }
    }

    pub fn refreshed(&self, curr_time: f64) -> Self {
        Self {
            data: self.data.clone(),
            expires_at: curr_time + CONFIG.record_expiration_interval,
        }
    }
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

    pub fn remove(&mut self, key: &Key) -> bool {
        self.records.remove(key).is_some()
    }

    pub fn remove_expired(&mut self, curr_time: f64) {
        self.records.retain(|_, record| record.expires_at > curr_time);
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[derive(Debug, Default)]
pub struct LocalFileStorage {
    data: HashMap<Key, String>,
}

impl LocalFileStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &Key) -> Option<&String> {
        self.data.get(key)
    }

    pub fn put(&mut self, key: Key, data: String) {
        self.data.insert(key, data);
    }

    pub fn remove(&mut self, key: &Key) -> bool {
        self.data.remove(key).is_some()
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

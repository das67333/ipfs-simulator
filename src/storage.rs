use crate::{Key, PeerId, CONFIG};
use serde::Serialize;
use std::collections::HashMap;

/// Represents a record in the storage.
#[derive(Debug, Clone, Serialize)]
pub struct Record {
    /// The data associated with the record.
    pub data: RecordData,
    /// The expiration time of the record.
    pub expires_at: f64,
}

/// Represents the data associated with a record.
#[derive(Debug, Clone, Serialize)]
pub enum RecordData {
    /// Provider record containing a key and a list of providers.
    ProviderRecord { key: Key, providers: Vec<PeerId> },
}

impl Record {
    /// Creates a new provider record.
    ///
    /// # Arguments
    ///
    /// * `self_id` - The ID of the current peer.
    /// * `key` - The key associated with the record.
    /// * `curr_time` - The current simulation time.
    ///
    /// # Returns
    ///
    /// A new `Record` instance.
    pub fn new_provider_record(self_id: PeerId, key: Key, curr_time: f64) -> Self {
        Self {
            data: RecordData::ProviderRecord {
                key,
                providers: vec![self_id],
            },
            expires_at: curr_time + CONFIG.record_expiration_interval,
        }
    }

    /// Returns the key associated with the record.
    pub fn key(&self) -> Key {
        match &self.data {
            RecordData::ProviderRecord { key, .. } => key.clone(),
        }
    }

    /// Returns a refreshed copy of the record with an updated expiration time.
    ///
    /// # Arguments
    ///
    /// * `curr_time` - The current simulation time.
    ///
    /// # Returns
    ///
    /// A new `Record` instance with the same data but an updated expiration time.
    pub fn refreshed(&self, curr_time: f64) -> Self {
        Self {
            data: self.data.clone(),
            expires_at: curr_time + CONFIG.record_expiration_interval,
        }
    }
}

/// Represents the local storage for the DHT.
#[derive(Debug, Default)]
pub struct LocalDHTStorage {
    records: HashMap<Key, Record>,
}

impl LocalDHTStorage {
    /// Creates a new `LocalDHTStorage` instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieves a record from the storage.
    ///
    /// # Arguments
    ///
    /// * `key` - The key associated with the record.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the record if found, or `None` if not found.
    pub fn get(&self, key: &Key) -> Option<&Record> {
        self.records.get(key)
    }

    /// Inserts a record into the storage.
    ///
    /// # Arguments
    ///
    /// * `key` - The key associated with the record.
    /// * `record` - The record to be inserted.
    pub fn put(&mut self, key: Key, record: Record) {
        self.records.insert(key, record);
    }

    /// Removes a record from the storage.
    ///
    /// # Arguments
    ///
    /// * `key` - The key associated with the record.
    ///
    /// # Returns
    ///
    /// `true` if the record was removed, `false` otherwise.
    pub fn remove(&mut self, key: &Key) -> bool {
        self.records.remove(key).is_some()
    }

    /// Removes expired records from the storage.
    ///
    /// # Arguments
    ///
    /// * `curr_time` - The current simulation time.
    pub fn remove_expired(&mut self, curr_time: f64) {
        self.records
            .retain(|_, record| record.expires_at > curr_time);
    }

    /// Clears the storage, removing all records.
    pub fn clear(&mut self) {
        self.records.clear();
    }
}

/// Represents the local file storage.
#[derive(Debug, Default)]
pub struct LocalFileStorage {
    data: HashMap<Key, String>,
}

impl LocalFileStorage {
    /// Creates a new `LocalFileStorage` instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieves data from the storage.
    ///
    /// # Arguments
    ///
    /// * `key` - The key associated with the data.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the data if found, or `None` if not found.
    pub fn get(&self, key: &Key) -> Option<&String> {
        self.data.get(key)
    }

    /// Inserts data into the storage.
    ///
    /// # Arguments
    ///
    /// * `key` - The key associated with the data.
    /// * `data` - The data to be inserted.
    pub fn put(&mut self, key: Key, data: String) {
        self.data.insert(key, data);
    }

    /// Removes data from the storage.
    ///
    /// # Arguments
    ///
    /// * `key` - The key associated with the data.
    ///
    /// # Returns
    ///
    /// `true` if the data was removed, `false` otherwise.
    pub fn remove(&mut self, key: &Key) -> bool {
        self.data.remove(key).is_some()
    }

    /// Clears the storage, removing all data.
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

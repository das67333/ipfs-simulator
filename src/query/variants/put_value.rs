use crate::{storage::Record, Key};

#[derive(Debug)]
pub struct PutValueQuery {
    key: Key,
    record: Record,
}

impl PutValueQuery {
    /// Creates a new `PutValueQuery` instance.
    ///
    /// # Arguments
    ///
    /// * `record` - The record to store.
    ///
    /// # Returns
    ///
    /// A new `PutValueQuery` instance.
    pub fn new(record: Record) -> PutValueQuery {
        PutValueQuery {
            key: record.key(),
            record,
        }
    }

    /// Returns the key of the record to store.
    pub fn key(&self) -> Key {
        self.key.clone()
    }

    /// Returns the record to store.
    pub fn record(&self) -> Record {
        self.record.clone()
    }
}

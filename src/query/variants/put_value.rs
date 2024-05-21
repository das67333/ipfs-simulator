use crate::{storage::Record, Key};

#[derive(Debug)]
pub struct PutValueQuery {
    key: Key,
    record: Record,
}

impl PutValueQuery {
    pub fn new(record: Record) -> PutValueQuery {
        PutValueQuery {
            key: record.key(),
            record,
        }
    }

    pub fn key(&self) -> Key {
        self.key.clone()
    }

    pub fn record(&self) -> Record {
        self.record.clone()
    }
}

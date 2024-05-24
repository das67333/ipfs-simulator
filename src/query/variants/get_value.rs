use super::QueryState;
use crate::{message::PutValueRequest, storage::Record, Key, PeerId, CONFIG};

/// Query to get the value associated with a key from the DHT.
#[derive(Debug)]
pub struct GetValueQuery {
    key: Key,
    caching: Vec<PeerId>,
}

impl GetValueQuery {
    /// Creates a new `GetValueQuery` instance.
    /// 
    /// # Arguments
    /// 
    /// * `key` - The key to retrieve the value for.
    /// 
    /// # Returns
    /// 
    /// A new `GetValueQuery` instance.
    pub fn new(key: Key) -> Self {
        Self {
            key,
            caching: vec![],
        }
    }

    /// Returns the key to retrieve the value for.
    pub fn key(&self) -> Key {
        self.key.clone()
    }

    /// Handles a response to the query.
    /// 
    /// # Arguments
    /// 
    /// * `peer` - The peer that sent the response.
    /// * `record` - The record associated with the key, if it was found.
    /// 
    /// # Returns
    /// 
    /// If the query is completed, returns the record and a list of pairs
    /// of peers and requests to send to them.
    pub fn on_response(
        &mut self,
        peer: PeerId,
        record: Option<Record>,
    ) -> QueryState<(), (Record, Vec<(PeerId, PutValueRequest)>)> {
        if let Some(record) = record {
            let requests = self
                .caching
                .iter()
                .map(|&dst| {
                    (
                        dst,
                        PutValueRequest {
                            key: self.key.clone(),
                            record: record.clone(),
                        },
                    )
                })
                .collect();
            QueryState::Completed((record, requests))
        } else {
            if self.caching.len() < CONFIG.caching_max_peers {
                self.caching.push(peer);
            }
            QueryState::InProgress(())
        }
    }
}

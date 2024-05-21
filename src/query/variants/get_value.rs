use super::QueryState;
use crate::{message::PutValueRequest, storage::Record, Key, PeerId, CONFIG};

#[derive(Debug)]
pub struct GetValueQuery {
    key: Key,
    caching: Vec<PeerId>,
}

impl GetValueQuery {
    pub fn new(key: Key) -> Self {
        Self {
            key,
            caching: vec![],
        }
    }

    pub fn key(&self) -> Key {
        self.key.clone()
    }

    pub fn on_response(
        &mut self,
        peer: PeerId,
        record: Option<Record>,
    ) -> QueryState<(), (String, Vec<(PeerId, PutValueRequest)>)> {
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
            QueryState::Completed((record.value, requests))
        } else {
            if self.caching.len() < CONFIG.caching_max_peers {
                self.caching.push(peer);
            }
            QueryState::InProgress(())
        }
    }
}

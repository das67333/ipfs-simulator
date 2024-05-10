use crate::{query::QueryId, Key, PeerId};
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct FindNodeRequest {
    pub query_id: QueryId,
    pub key: Key,
}

#[derive(Clone, Serialize)]
pub struct FindNodeResponse {
    pub query_id: QueryId,
    pub closest_peers: Vec<PeerId>,
}

use crate::{query::QueryId, storage::Record, Key, PeerId};
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct FindNodeRequest {
    /// The ID of the query that originated the request.
    pub query_id: QueryId,
    /// The key to search for.
    pub key: Key,
}

/// Response to a FindNode request.
#[derive(Clone, Serialize)]
pub struct FindNodeResponse {
    /// The ID of the query that originated the request.
    pub query_id: QueryId,
    /// The list of locally closest peers.
    pub closest_peers: Vec<PeerId>,
}

#[derive(Clone, Serialize)]
pub struct PingRequest {}

#[derive(Clone, Serialize)]
pub struct PingResponse {}

#[derive(Clone, Serialize)]
pub struct GetValueRequest {
    /// The ID of the query that originated the request.
    pub query_id: QueryId,
    /// The key to search for.
    pub key: Key,
}

#[derive(Clone, Serialize)]
pub struct GetValueResponse {
    /// The ID of the query that originated the request.
    pub query_id: QueryId,
    /// The value associated with the key.
    pub record: Option<Record>,
}

#[derive(Clone, Serialize)]
pub struct PutValueRequest {
    /// The key of the value to store.
    pub key: Key,
    /// The value to store.
    pub record: Record,
}

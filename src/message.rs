use crate::{query::QueryId, Key, PeerId};
use serde::Serialize;

/// Represents a FindNode request.
#[derive(Clone, Serialize)]
pub struct FindNodeRequest {
    /// The ID of the query.
    pub query_id: QueryId,
    /// The key to search for.
    pub key: Key,
}

/// Represents a response to a FindNode query.
#[derive(Clone, Serialize)]
pub struct FindNodeResponse {
    /// The ID of the query.
    pub query_id: QueryId,
    /// The list of locally closest peers.
    pub closest_peers: Vec<PeerId>,
}

/// Represents a PutValue request.
#[derive(Clone, Serialize)]
pub struct PutValueRequest {
    /// The value to store.
    pub value: String,
    /// The expiration time of the value.
    pub expires_at: f64,
}

#[derive(Clone, Serialize)]
pub struct PingRequest {}

#[derive(Clone, Serialize)]
pub struct PingResponse {}

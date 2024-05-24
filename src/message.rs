use crate::{query::QueryId, storage::Record, Key, PeerId};
use serde::Serialize;

/// Request to find the closest peers to a key.
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

/// Timeout event for a FindNode query.
#[derive(Clone, Serialize)]
pub struct FindNodeQueryTimeout {
    pub query_id: QueryId,
}

/// Request to get the value associated with a key.
#[derive(Clone, Serialize)]
pub struct GetValueRequest {
    /// The ID of the query that originated the request.
    pub query_id: QueryId,
    /// The key to search for.
    pub key: Key,
}

/// Response to a GetValue request.
#[derive(Clone, Serialize)]
pub struct GetValueResponse {
    /// The ID of the query that originated the request.
    pub query_id: QueryId,
    /// The value associated with the key.
    pub record: Option<Record>,
}

/// Timeout event for a GetValue query.
#[derive(Clone, Serialize)]
pub struct GetValueQueryTimeout {
    pub query_id: QueryId,
}

/// Request to store a value associated with a key.
#[derive(Clone, Serialize)]
pub struct PutValueRequest {
    /// The key of the value to store.
    pub key: Key,
    /// The value to store.
    pub record: Record,
}

/// Response to a PutValue request.
#[derive(Clone, Serialize)]
pub struct PutValueQueryTimeout {
    pub query_id: QueryId,
}

/// Timeout event for a PutValue query.
#[derive(Clone, Serialize)]
pub struct RetrieveDataRequest {
    pub query_id: QueryId,
    pub key: Key,
}

/// Response to a RetrieveData request.
#[derive(Clone, Serialize)]
pub struct RetrieveDataResponse {
    pub query_id: QueryId,
    pub data: Option<String>,
}

/// Timeout event for a RetrieveData query.
#[derive(Clone, Serialize)]
pub struct RetrieveDataQueryTimeout {
    pub query_id: QueryId,
}

/// Request to check if a peer is still alive.
#[derive(Clone, Serialize)]
pub struct PingRequest {}

/// Response to a Ping request.
#[derive(Clone, Serialize)]
pub struct PingResponse {}

/// Timeout event for a Ping query.
#[derive(Clone, Serialize)]
pub struct PingTimeout {}

/// Timer for bootstrapping the network.
#[derive(Clone, Serialize)]
pub struct BootstrapTimer {}

/// Timer for republishing a DHT records.
#[derive(Clone, Serialize)]
pub struct RepublishTimer {
    pub key: Key,
}

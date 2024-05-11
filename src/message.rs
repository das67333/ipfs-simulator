use crate::{query::QueryId, Key, PeerId};
use serde::Serialize;

#[derive(Clone, Serialize)]
/// Represents a FindNode request.
pub struct FindNodeRequest {
    /// The ID of the query.
    pub query_id: QueryId,
    /// The key to search for.
    pub key: Key,
}

#[derive(Clone, Serialize)]
/// Represents a response to a FindNode query.
pub struct FindNodeResponse {
    /// The ID of the query.
    pub query_id: QueryId,
    /// The list of locally closest peers.
    pub closest_peers: Vec<PeerId>,
}

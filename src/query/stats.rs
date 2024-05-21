use super::variants::evaluate_closest_peers;
use crate::{Key, PeerId};

/// Struct to store statistics related to queries.
#[derive(Debug, Default, Clone)]
pub struct QueriesStats {
    pub find_node_queries_started: usize,
    pub find_node_queries_completed: usize,
    pub find_node_queries_failed: usize,
    pub closest_peers_total: usize,
    pub closest_peers_correct: usize,
    pub get_value_queries_started: usize,
    pub get_value_queries_completed: usize,
    pub put_value_queries_started: usize,
    pub put_value_queries_completed: usize,
    pub put_value_queries_failed: usize,
    pub ping_requests_cnt: usize,
    pub ping_responses_cnt: usize,
}

impl QueriesStats {
    /// Creates a empty instance of `QueriesStats`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Updating the statistics by evaluating the `FindNodeQuery`.
    ///
    /// # Arguments
    ///
    /// * `target_key` - The key used in the query.
    /// * `peers` - The list of peers returned by the query.
    pub fn evaluate(&mut self, target_key: Key, peers: &[PeerId]) {
        self.closest_peers_total += peers.len();
        self.closest_peers_correct += evaluate_closest_peers(target_key, peers);
    }
}

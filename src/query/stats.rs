use super::variants::evaluate_closest_peers;
use crate::{Key, PeerId};

/// Struct to store statistics related to queries.
#[derive(Clone)]
pub struct QueriesStats {
    pub find_node_queries_cnt: usize,
    pub closest_peers_total: usize,
    pub closest_peers_correct: usize,
}

impl QueriesStats {
    /// Creates a empty instance of `QueriesStats`.
    pub fn new() -> Self {
        Self {
            find_node_queries_cnt: 0,
            closest_peers_total: 0,
            closest_peers_correct: 0,
        }
    }

    /// Updating the statistics by evaluating the `FindNodeQuery`.
    ///
    /// # Arguments
    ///
    /// * `target_key` - The key used in the query.
    /// * `peers` - The list of peers returned by the query.
    pub fn evaluate(&mut self, target_key: Key, peers: &[PeerId]) {
        self.find_node_queries_cnt += 1;
        self.closest_peers_total += peers.len();
        self.closest_peers_correct += evaluate_closest_peers(target_key, peers);
    }
}

impl Default for QueriesStats {
    fn default() -> Self {
        Self::new()
    }
}

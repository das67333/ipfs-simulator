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
    pub get_value_queries_failed: usize,
    pub put_value_queries_started: usize,
    pub put_value_queries_completed: usize,
    pub put_value_queries_failed: usize,
    pub ping_requests_cnt: usize,
    pub ping_responses_cnt: usize,
    pub ping_requests_failed: usize,
    pub retrieve_data_queries_started: usize,
    pub retrieve_data_queries_completed: usize,
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

    pub fn merge(&mut self, other: &Self) {
        self.find_node_queries_started += other.find_node_queries_started;
        self.find_node_queries_completed += other.find_node_queries_completed;
        self.find_node_queries_failed += other.find_node_queries_failed;
        self.closest_peers_total += other.closest_peers_total;
        self.closest_peers_correct += other.closest_peers_correct;
        self.get_value_queries_started += other.get_value_queries_started;
        self.get_value_queries_completed += other.get_value_queries_completed;
        self.get_value_queries_failed += other.get_value_queries_failed;
        self.put_value_queries_started += other.put_value_queries_started;
        self.put_value_queries_completed += other.put_value_queries_completed;
        self.put_value_queries_failed += other.put_value_queries_failed;
        self.ping_requests_cnt += other.ping_requests_cnt;
        self.ping_responses_cnt += other.ping_responses_cnt;
        self.ping_requests_failed += other.ping_requests_failed;
        self.retrieve_data_queries_started += other.retrieve_data_queries_started;
        self.retrieve_data_queries_completed += other.retrieve_data_queries_completed;
    }
}

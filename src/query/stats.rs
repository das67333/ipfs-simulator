use super::variants::evaluate_closest_peers;
use crate::{Key, PeerId};

/// Struct to store statistics related to queries.
#[derive(Debug, Default, Clone)]
pub struct QueriesStats {
    pub closest_peers_total: u64,
    pub closest_peers_correct: u64,
    pub find_node_queries_started: u32,
    pub find_node_queries_completed: u32,
    pub find_node_queries_failed: u32,
    pub get_value_queries_started: u32,
    pub get_value_queries_completed: u32,
    pub get_value_queries_failed: u32,
    pub put_value_queries_started: u32,
    pub put_value_queries_completed: u32,
    pub put_value_queries_failed: u32,
    pub ping_requests_cnt: u32,
    pub ping_responses_cnt: u32,
    pub ping_requests_failed: u32,
    pub retrieve_data_queries_started: u32,
    pub retrieve_data_queries_completed: u32,
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
        self.closest_peers_total += peers.len() as u64;
        self.closest_peers_correct += evaluate_closest_peers(target_key, peers) as u64;
    }

    pub fn merge(&mut self, other: &Self) {
        self.closest_peers_total += other.closest_peers_total;
        self.closest_peers_correct += other.closest_peers_correct;
        self.find_node_queries_started += other.find_node_queries_started;
        self.find_node_queries_completed += other.find_node_queries_completed;
        self.find_node_queries_failed += other.find_node_queries_failed;
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

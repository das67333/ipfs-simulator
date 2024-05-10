use crate::K_VALUE;

use super::{variants::QueryState, FindNodeQuery, QueryConfig};
use std::collections::HashMap;

#[allow(dead_code)]
pub struct QueryPool {
    next_id: QueryId,
    config: QueryConfig,
    find_node_queries: HashMap<QueryId, FindNodeQuery>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub struct QueryId(u64);

impl QueryPool {
    pub fn new() -> Self {
        Self::default()
    }

    fn next_query_id(&mut self) -> QueryId {
        let query_id = self.next_id;
        self.next_id = QueryId(self.next_id.0.wrapping_add(1));
        query_id
    }

    /// Adds a query to the pool that iterates towards the closest peers to the target.
    pub fn add_find_node_query(&mut self, query: FindNodeQuery) -> QueryId {
        let query_id = self.next_query_id();
        self.find_node_queries.insert(query_id, query);
        query_id
    }

    pub fn get_mut_find_node_query(&mut self, query_id: QueryId) -> Option<&mut FindNodeQuery> {
        self.find_node_queries.get_mut(&query_id)
    }

    pub fn stats(&self) -> usize {
        self.find_node_queries
            .values()
            .map(|query| match &query.state {
                QueryState::InProgress => 0,
                QueryState::Completed(value) => (value.len() == K_VALUE) as usize,
            })
            .sum()
    }
}

impl Default for QueryPool {
    fn default() -> Self {
        Self {
            next_id: QueryId(0),
            config: QueryConfig::default(),
            find_node_queries: HashMap::new(),
        }
    }
}

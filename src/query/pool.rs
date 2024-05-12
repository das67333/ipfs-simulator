use super::{FindNodeQuery, PutValueQuery};
use std::collections::HashMap;

/// Represents a peer's pool of queries.
#[allow(dead_code)]
pub struct QueryPool {
    next_id: QueryId,
    find_node_queries: HashMap<QueryId, FindNodeQuery>,
    put_value_queries: HashMap<QueryId, PutValueQuery>,
}

/// Represents a unique identifier for a query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub struct QueryId(u64);

impl QueryPool {
    /// Creates an empty `QueryPool` instance.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next_query_id(&mut self) -> QueryId {
        let query_id = self.next_id;
        self.next_id = QueryId(self.next_id.0.wrapping_add(1));
        query_id
    }

    /// Adds a `FindNodeQuery` to the pool.
    ///
    /// Returns the unique identifier assigned to the query.
    pub fn add_find_node_query(&mut self, query_id: QueryId, query: FindNodeQuery) {
        self.find_node_queries.insert(query_id, query);
    }

    /// Removes a `FindNodeQuery` from the pool.
    pub fn remove_find_node_query(&mut self, query_id: QueryId) -> bool {
        self.find_node_queries.remove(&query_id).is_some()
    }

    /// Returns a mutable reference to the `FindNodeQuery` with the specified query ID, if it exists.
    pub fn get_mut_find_node_query(&mut self, query_id: QueryId) -> Option<&mut FindNodeQuery> {
        self.find_node_queries.get_mut(&query_id)
    }

    /// Adds a `PutValueQuery` to the pool.
    ///
    /// Returns the unique identifier assigned to the query.
    pub fn add_put_value_query(&mut self, query_id: QueryId, query: PutValueQuery) {
        self.put_value_queries.insert(query_id, query);
    }

    /// Removes a `PutValueQuery` from the pool.
    pub fn remove_put_value_query(&mut self, query_id: QueryId) -> Option<PutValueQuery> {
        self.put_value_queries.remove(&query_id)
    }
}

impl Default for QueryPool {
    fn default() -> Self {
        Self {
            next_id: QueryId(0),
            find_node_queries: HashMap::new(),
            put_value_queries: HashMap::new(),
        }
    }
}

use super::{FindNodeQuery, GetValueQuery, PutValueQuery};
use std::collections::{HashMap, HashSet};

/// Represents a peer's pool of queries.
#[derive(Debug, Default)]
pub struct QueriesPool {
    next_id: QueryId,
    find_node_queries: HashMap<QueryId, FindNodeQuery>,
    get_value_queries: HashMap<QueryId, GetValueQuery>,
    put_value_queries: HashMap<QueryId, PutValueQuery>,
    retrieve_data_queries: HashSet<QueryId>,
}

/// Represents a unique identifier for a query.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub struct QueryId(u64);

impl std::fmt::Display for QueryId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl QueriesPool {
    /// Creates an empty `QueryPool` instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the ID of the next query.
    pub fn next_query_id(&mut self) -> QueryId {
        let query_id = self.next_id;
        self.next_id = QueryId(self.next_id.0.wrapping_add(1));
        query_id
    }

    /// Adds a `FindNodeQuery` to the pool.
    ///
    /// # Arguments
    ///
    /// * `query_id` - The ID of the query.
    /// * `query` - The `FindNodeQuery` to add.
    pub fn add_find_node_query(&mut self, query_id: QueryId, query: FindNodeQuery) {
        self.find_node_queries.insert(query_id, query);
    }

    /// Removes a `FindNodeQuery` from the pool.
    ///
    /// # Arguments
    ///
    /// * `query_id` - The ID of the query to remove.
    ///
    /// # Returns
    ///
    /// The removed `FindNodeQuery`, if it existed.
    pub fn remove_find_node_query(&mut self, query_id: QueryId) -> Option<FindNodeQuery> {
        self.find_node_queries.remove(&query_id)
    }

    /// Returns a mutable reference to the `FindNodeQuery` with the specified query ID, if it exists.
    ///
    /// # Arguments
    ///
    /// * `query_id` - The ID of the query to retrieve.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `FindNodeQuery`, if it exists.
    pub fn get_mut_find_node_query(&mut self, query_id: QueryId) -> Option<&mut FindNodeQuery> {
        self.find_node_queries.get_mut(&query_id)
    }

    /// Adds a `GetValueQuery` to the pool.
    ///
    /// # Arguments
    ///
    /// * `query_id` - The ID of the query.
    /// * `query` - The `GetValueQuery` to add.
    pub fn add_get_value_query(&mut self, query_id: QueryId, query: GetValueQuery) {
        self.get_value_queries.insert(query_id, query);
    }

    /// Removes a `GetValueQuery` from the pool.
    ///
    /// # Arguments
    ///
    /// * `query_id` - The ID of the query to remove.
    ///
    /// # Returns
    ///
    /// The removed `GetValueQuery`, if it existed.
    pub fn remove_get_value_query(&mut self, query_id: QueryId) -> Option<GetValueQuery> {
        self.get_value_queries.remove(&query_id)
    }

    /// Returns a mutable reference to the `GetValueQuery` with the specified query ID, if it exists.
    ///
    /// # Arguments
    ///
    /// * `query_id` - The ID of the query to retrieve.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `GetValueQuery`, if it exists.
    pub fn get_mut_get_value_query(&mut self, query_id: QueryId) -> Option<&mut GetValueQuery> {
        self.get_value_queries.get_mut(&query_id)
    }

    /// Adds a `PutValueQuery` to the pool.
    ///
    /// # Arguments
    ///
    /// * `query_id` - The ID of the query.
    /// * `query` - The `PutValueQuery` to add.
    pub fn add_put_value_query(&mut self, query_id: QueryId, query: PutValueQuery) {
        self.put_value_queries.insert(query_id, query);
    }

    /// Removes a `PutValueQuery` from the pool.
    ///
    /// # Arguments
    ///
    /// * `query_id` - The ID of the query to remove.
    ///
    /// # Returns
    ///
    /// The removed `PutValueQuery`, if it existed.
    pub fn remove_put_value_query(&mut self, query_id: QueryId) -> Option<PutValueQuery> {
        self.put_value_queries.remove(&query_id)
    }

    /// Adds a `RetrieveDataQuery` to the pool.
    ///
    /// # Arguments
    ///
    /// * `query_id` - The ID of the query.
    pub fn add_retrieve_data_query(&mut self, query_id: QueryId) {
        self.retrieve_data_queries.insert(query_id);
    }

    /// Removes a `RetrieveDataQuery` from the pool.
    ///
    /// # Arguments
    ///
    /// * `query_id` - The ID of the query to remove.
    ///
    /// # Returns
    ///
    /// `true` if the query was removed, `false` if the query did not exist.
    pub fn remove_retrieve_data_query(&mut self, query_id: QueryId) -> bool {
        self.retrieve_data_queries.remove(&query_id)
    }
}

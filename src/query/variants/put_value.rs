use crate::{
    query::{QueryId, QueryPool},
    Key,
};

pub struct PutValueQuery {
    value: String,
}

impl PutValueQuery {
    pub fn new_query(queries: &mut QueryPool, value: String) -> (QueryId, Key) {
        let key = Key::from_sha256(value.as_bytes());
        let query = PutValueQuery { value };
        let query_id = queries.add_put_value_query(query);
        (query_id, key)
    }

    pub fn value(&self) -> String {
        self.value.clone()
    }
}

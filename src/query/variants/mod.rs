mod find_node;

pub use find_node::FindNodeQuery;

pub enum QueryState<T> {
    InProgress,
    Completed(T),
}

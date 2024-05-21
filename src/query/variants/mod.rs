mod find_node;
mod get_value;
mod put_value;

pub use find_node::{evaluate_closest_peers, FindNodeQuery};
pub use get_value::GetValueQuery;
pub use put_value::PutValueQuery;

pub enum QueryState<T, Y> {
    InProgress(T),
    Completed(Y),
}

#[derive(Debug, Clone)]
pub enum QueryTrigger {
    Manual,
    Bootstrap,
    GetValue(super::QueryId),
    PutValue(super::QueryId),
}

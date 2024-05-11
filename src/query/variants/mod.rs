mod find_node;
mod ping;
mod put_value;

pub use find_node::{evaluate_closest_peers, FindNodeQuery};
pub use ping::PingRequest;
pub use put_value::PutValueQuery;

pub enum QueryState<T, Y> {
    InProgress(T),
    Completed(Y),
}

#[derive(Clone)]
pub enum QueryTrigger {
    Manual,
    Bootstrap,
    PutValue(super::QueryId),
}

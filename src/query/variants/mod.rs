mod find_node;
mod ping;

pub use find_node::FindNodeQuery;
pub use ping::PingRequest;

pub enum QueryState<T> {
    InProgress,
    Completed(T),
}

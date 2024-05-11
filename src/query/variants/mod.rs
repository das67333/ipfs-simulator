mod find_node;
mod ping;

pub use find_node::{evaluate_closest_peers, FindNodeQuery};
pub use ping::PingRequest;

pub enum QueryState<T, Y> {
    InProgress(T),
    Completed(Y),
}

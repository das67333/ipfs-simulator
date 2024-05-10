pub mod app;
pub mod config;
pub mod kbucket;
pub mod message;
pub mod network;
pub mod peer;
pub mod query;

pub const K_VALUE: usize = 20;
pub const ALPHA_VALUE: usize = 3;

pub use dslab_core::Id as PeerId;
pub use kbucket::{Distance, Key};

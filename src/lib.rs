pub mod app;
pub mod config;
pub mod kbucket;
pub mod message;
pub mod network;
pub mod peer;
pub mod query;
pub mod storage;

pub use config::SimulationConfig;
pub use dslab_core::Id as PeerId;
pub use kbucket::{Distance, Key};

lazy_static::lazy_static! {
    static ref CONFIG: SimulationConfig = SimulationConfig::from_default_config_file();
    static ref K_VALUE: usize = CONFIG.k;
    static ref ALPHA_VALUE: usize = CONFIG.alpha;
    static ref KEYS_POOL: Vec<Key> = (0..CONFIG.num_peers)
        .map(|id: PeerId| Key::from_sha256(&id.to_le_bytes()))
        .collect();
    static ref KEYS_TREE: kbucket::KeysTree = kbucket::KeysTree::new(&KEYS_POOL);
    static ref PEER_ID_BY_KEY: std::collections::HashMap<Key, PeerId> = KEYS_POOL
        .iter().enumerate().map(|(id, key)| (key.clone(), id as PeerId)).collect();
}

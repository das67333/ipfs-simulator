use serde::Deserialize;
use std::path::Path;

/// Represents the structure to parse the configuration file into.
#[derive(Debug, Deserialize)]
pub struct ConfigTOML {
    pub log_level_filter: String,
    pub log_file_path: Option<String>,
    pub enable_user_load_generation: bool,
    pub user_load_block_size: Option<usize>,
    pub user_load_blocks_pool_size: Option<usize>,
    pub user_load_events_interval: Option<f64>,
    pub seed: u64,
    pub k: usize,
    pub alpha: usize,
    pub num_peers: u32,
    pub delay_distribution: String,
    pub delay_mean: Option<f64>,
    pub delay_std_dev: Option<f64>,
    pub delay_min: Option<f64>,
    pub delay_max: Option<f64>,
    pub topology: String,
    pub record_publication_interval: f64,
    pub record_expiration_interval: f64,
    pub kbuckets_refresh_interval: f64,
    pub query_timeout: f64,
    pub caching_max_peers: usize,
    pub enable_bootstrap: bool,
    pub enable_republishing: bool,
}

impl ConfigTOML {
    /// Parses the configuration from a TOML file.
    pub fn from_file(path: impl AsRef<Path>) -> Self {
        let data = std::fs::read_to_string(path).expect("Failed to read config file");
        toml::from_str(&data).expect("Failed to parse config")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_file() {
        let _config = ConfigTOML::from_file("config.toml");
    }
}
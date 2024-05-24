use super::toml_parser::ConfigTOML;
use crate::network::{DelayDistribution, Topology};

/// Represents the configuration of the IPFS simulator.
#[derive(Debug)]
pub struct SimulationConfig {
    pub log_level_filter: log::LevelFilter,
    pub log_file_path: Option<String>,
    pub enable_user_load_generation: bool,
    pub user_load_block_size: Option<usize>,
    pub user_load_blocks_pool_size: Option<usize>,
    pub user_load_events_interval: Option<f64>,
    pub seed: u64,
    pub k: usize,
    pub alpha: usize,
    pub num_peers: u32,
    pub delay_distribution: DelayDistribution,
    pub topology: Topology,
    pub record_publication_interval: f64,
    pub record_expiration_interval: f64,
    pub kbuckets_refresh_interval: f64,
    pub query_timeout: f64,
    pub caching_max_peers: usize,
    pub enable_bootstrap: bool,
    pub enable_republishing: bool,
}

impl SimulationConfig {
    /// Creates a new `SimulationConfig` instance from the default configuration file.
    pub fn from_default_config_file() -> Self {
        let toml = ConfigTOML::from_file("config.toml");
        Self::from_toml(toml)
    }

    /// Creates a new `SimulationConfig` instance from the specified TOML configuration file.
    fn from_toml(toml: ConfigTOML) -> Self {
        let log_level_filter = match toml.log_level_filter.as_str() {
            "off" => log::LevelFilter::Off,
            "error" => log::LevelFilter::Error,
            "warn" => log::LevelFilter::Warn,
            "info" => log::LevelFilter::Info,
            "debug" => log::LevelFilter::Debug,
            "trace" => log::LevelFilter::Trace,
            _ => panic!("invalid log_level_filter"),
        };
        if toml.enable_user_load_generation {
            assert!(
                toml.user_load_block_size.is_some(),
                "missing user_load_block_size"
            );
            assert!(
                toml.user_load_blocks_pool_size.is_some(),
                "missing user_load_blocks_pool_size"
            );
            assert!(
                toml.user_load_events_interval.is_some(),
                "missing user_load_events_interval"
            );
        }
        let delay_distribution = match toml.delay_distribution.as_str() {
            "constant" => {
                let mean = match toml.delay_mean {
                    Some(mean) => {
                        assert!(mean >= 0., "delay_mean must be non-negative");
                        mean
                    }
                    None => panic!("missing delay_mean"),
                };
                DelayDistribution::Constant(mean)
            }
            "uniform" => {
                let left = match toml.delay_min {
                    Some(min) => {
                        assert!(min >= 0., "delay_min must be non-negative");
                        min
                    }
                    None => panic!("missing delay_min"),
                };
                let right = match toml.delay_max {
                    Some(max) => {
                        assert!(max > left, "delay_max must be greater than delay_min");
                        max
                    }
                    None => panic!("missing delay_max"),
                };
                DelayDistribution::Uniform { left, right }
            }
            "positive_normal" => {
                let mean = match toml.delay_mean {
                    Some(mean) => {
                        assert!(mean >= 0., "delay_mean must be non-negative");
                        mean
                    }
                    None => panic!("missing delay_mean"),
                };
                let std_dev = match toml.delay_std_dev {
                    Some(std_dev) => {
                        assert!(std_dev >= 0., "delay_std_dev must be non-negative");
                        std_dev
                    }
                    None => panic!("missing delay_std_dev"),
                };
                DelayDistribution::PositiveNormal { mean, std_dev }
            }
            _ => panic!("invalid delay distribution"),
        };

        let topology = match toml.topology.as_str() {
            "full" => Topology::Full,
            "ring" => Topology::Ring {
                first_id: 0,
                last_id: toml.num_peers - 1,
            },
            "star" => Topology::Star { center_id: 0 },
            _ => panic!("invalid topology"),
        };

        Self {
            log_level_filter,
            log_file_path: toml.log_file_path,
            enable_user_load_generation: toml.enable_user_load_generation,
            user_load_block_size: toml.user_load_block_size,
            user_load_blocks_pool_size: toml.user_load_blocks_pool_size,
            user_load_events_interval: toml.user_load_events_interval,
            seed: toml.seed,
            k: toml.k,
            alpha: toml.alpha,
            num_peers: toml.num_peers,
            delay_distribution,
            topology,
            record_publication_interval: toml.record_publication_interval,
            record_expiration_interval: toml.record_expiration_interval,
            kbuckets_refresh_interval: toml.kbuckets_refresh_interval,
            query_timeout: toml.query_timeout,
            caching_max_peers: toml.caching_max_peers,
            enable_bootstrap: toml.enable_bootstrap,
            enable_republishing: toml.enable_republishing,
        }
    }
}

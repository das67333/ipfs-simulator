use super::toml_parser::ConfigTOML;
use crate::network::{DelayDistribution, Topology};

pub struct SimulationConfig {
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
    pub fn from_default_config_file() -> Self {
        let toml = ConfigTOML::from_file("config.toml");
        Self::from_toml(toml)
    }

    fn from_toml(toml: ConfigTOML) -> Self {
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

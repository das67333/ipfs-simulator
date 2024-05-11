use rand::distributions::{Distribution, Uniform};
use rand_distr::Normal;

/// Represents the distribution of delay values.
#[derive(Clone)]
pub enum DelayDistribution {
    /// Represents a constant delay value.
    Constant(f64),
    /// Represents a uniform distribution of delay values.
    Uniform { left: f64, right: f64 },
    /// Represents a normal distribution of delay values.
    /// If sampled value is negative, it is replaced with 0.
    PositiveNormal { mean: f64, std_dev: f64 },
}

impl Distribution<f64> for DelayDistribution {
    fn sample<R: rand::Rng + ?Sized>(&self, _rng: &mut R) -> f64 {
        match self {
            Self::Constant(delay) => *delay,
            Self::Uniform { left, right } => {
                let distr = Uniform::new_inclusive(*left, *right);
                distr.sample(_rng)
            }
            Self::PositiveNormal { mean, std_dev } => {
                let distr = Normal::new(*mean, *std_dev).unwrap();
                distr.sample(_rng).max(0.)
            }
        }
    }
}

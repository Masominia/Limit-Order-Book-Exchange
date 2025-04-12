//! Follow a fixed volume profile over the horizon.

use super::{share, Schedule};
use crate::types::Qty;

pub struct Vwap {
    cumulative: Vec<f64>,
}

impl Vwap {
    /// `weights[i]` is the expected share of volume in bucket i; any positive scale.
    pub fn new(weights: &[f64]) -> Self {
        let total: f64 = weights.iter().sum();
        let mut acc = 0.0;
        let cumulative = weights
            .iter()
            .map(|w| {
                acc += w / total;
                acc
            })
            .collect();
        Vwap { cumulative }
    }

    /// The usual intraday shape: busy at both ends, quiet in the middle.
    pub fn u_shape(buckets: usize, depth: f64) -> Self {
        let weights: Vec<f64> = (0..buckets)
            .map(|i| {
                let x = (i as f64 + 0.5) / buckets as f64;
                1.0 + depth * (2.0 * x - 1.0).powi(2)
            })
            .collect();
        Vwap::new(&weights)
    }

    pub fn cumulative(&self, tau: f64) -> f64 {
        let n = self.cumulative.len();
        let pos = tau.clamp(0.0, 1.0) * n as f64;
        let i = pos.floor() as usize;
        if i >= n {
            return 1.0;
        }
        let before = if i == 0 { 0.0 } else { self.cumulative[i - 1] };
        before + (pos - i as f64) * (self.cumulative[i] - before)
    }
}

impl Schedule for Vwap {
    fn name(&self) -> &'static str {
        "vwap"
    }

    fn target(&self, step: usize, horizon: usize, total: Qty, _market_volume: Qty) -> Qty {
        share(total, self.cumulative((step + 1) as f64 / horizon as f64))
    }
}

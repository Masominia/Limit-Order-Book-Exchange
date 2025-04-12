//! The optimal schedule of the linear-impact, mean-variance problem: remaining quantity
//! proportional to sinh(urgency (1 - t/T)). Urgency zero is equal slices, large urgency is immediate.

use super::{share, Schedule};
use crate::types::Qty;

pub struct FrontLoaded {
    /// Dimensionless: the product of the risk-aversion rate and the horizon.
    pub urgency: f64,
}

impl FrontLoaded {
    /// Fraction of the order still to do at time fraction `tau`.
    pub fn remaining(&self, tau: f64) -> f64 {
        if self.urgency < 1e-9 {
            1.0 - tau
        } else {
            (self.urgency * (1.0 - tau)).sinh() / self.urgency.sinh()
        }
    }
}

impl Schedule for FrontLoaded {
    fn name(&self) -> &'static str {
        "front-loaded"
    }

    fn target(&self, step: usize, horizon: usize, total: Qty, _market_volume: Qty) -> Qty {
        share(total, 1.0 - self.remaining((step + 1) as f64 / horizon as f64))
    }
}

//! Equal quantity in every step.

use super::{share, Schedule};
use crate::types::Qty;

pub struct Twap;

impl Schedule for Twap {
    fn name(&self) -> &'static str {
        "twap"
    }

    fn target(&self, step: usize, horizon: usize, total: Qty, _market_volume: Qty) -> Qty {
        share(total, (step + 1) as f64 / horizon as f64)
    }
}

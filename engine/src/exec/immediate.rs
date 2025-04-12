//! Everything in the first step.

use super::Schedule;
use crate::types::Qty;

pub struct Immediate;

impl Schedule for Immediate {
    fn name(&self) -> &'static str {
        "immediate"
    }

    fn target(&self, _step: usize, _horizon: usize, total: Qty, _market_volume: Qty) -> Qty {
        total
    }
}

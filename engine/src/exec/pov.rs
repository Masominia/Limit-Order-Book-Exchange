//! Percentage of volume: follow the market's traded volume at a fixed participation rate,
//! and complete whatever is left in the last step.

use super::Schedule;
use crate::types::Qty;

pub struct Pov {
    /// Own volume as a fraction of total market volume (own trades included).
    pub rate: f64,
}

impl Schedule for Pov {
    fn name(&self) -> &'static str {
        "pov"
    }

    fn target(&self, step: usize, horizon: usize, total: Qty, market_volume: Qty) -> Qty {
        if step + 1 >= horizon {
            total
        } else {
            ((self.rate * market_volume as f64) as Qty).min(total)
        }
    }
}

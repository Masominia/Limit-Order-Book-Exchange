//! Execution of a large parent order as a sequence of market orders.

pub mod frontloaded;
pub mod immediate;
pub mod pov;
pub mod runner;
pub mod twap;
pub mod vwap;

use crate::types::Qty;

/// A schedule answers one question: how much of the parent order should be done by the end of this step?
pub trait Schedule {
    fn name(&self) -> &'static str;

    /// Cumulative target after step `step` (0-based) of `horizon`, for a parent order of `total` lots.
    /// `market_volume` is the volume traded in the market since the start of the execution, own trades included.
    fn target(&self, step: usize, horizon: usize, total: Qty, market_volume: Qty) -> Qty;
}

pub(crate) fn share(total: Qty, fraction: f64) -> Qty {
    ((total as f64 * fraction.clamp(0.0, 1.0)).round() as Qty).min(total)
}

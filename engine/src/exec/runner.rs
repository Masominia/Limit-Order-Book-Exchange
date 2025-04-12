//! Runs a schedule against a live market and measures what it cost.

use super::Schedule;
use crate::agents::Action;
use crate::sim::market::Market;
use crate::types::{Qty, Side};

#[derive(Clone, Copy, Debug)]
pub struct Report {
    /// Mid price when the order arrived.
    pub arrival: f64,
    /// Average fill price.
    pub average: f64,
    pub filled: Qty,
    /// Implementation shortfall in basis points of the arrival price, positive is a cost.
    pub shortfall_bps: f64,
    /// Signed move of the mid price from arrival to the end of the horizon, in basis points, positive is adverse.
    pub drift_bps: f64,
    pub child_orders: usize,
}

pub fn execute(market: &mut Market, side: Side, total: Qty, horizon: usize, schedule: &dyn Schedule) -> Report {
    let arrival = market.reference;
    let (mut filled, mut cost, mut market_volume, mut children) = (0u64, 0.0, 0u64, 0usize);
    for step in 0..horizon {
        let before = market.tape.len();
        market.step();
        market_volume += market.tape[before..].iter().map(|t| t.qty).sum::<Qty>();
        let target = schedule.target(step, horizon, total, market_volume).min(total);
        if target > filled {
            let (_, trades) = market.submit(Action::Market { side, qty: target - filled });
            children += 1;
            for t in &trades {
                filled += t.qty;
                market_volume += t.qty;
                cost += t.qty as f64 * t.price as f64;
            }
        }
    }
    let average = if filled > 0 { cost / filled as f64 } else { f64::NAN };
    let sign = side.sign() as f64;
    let end = market.book.mid().unwrap_or(market.reference);
    Report {
        arrival,
        average,
        filled,
        shortfall_bps: sign * (average - arrival) / arrival * 1e4,
        drift_bps: sign * (end - arrival) / arrival * 1e4,
        child_orders: children,
    }
}

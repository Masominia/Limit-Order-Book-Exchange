//! Zero-intelligence order flow: random limit orders around the opposite best price,
//! random market orders and random cancellations.

use super::{Account, Action, Agent, View};
use crate::math::rng::Rng;
use crate::types::{OrderId, Price, Qty, Side};

pub struct Noise {
    /// Limit orders per step.
    pub limit_rate: f64,
    /// Market orders per step.
    pub market_rate: f64,
    /// Probability per step that any given resting order is cancelled.
    pub cancel_rate: f64,
    /// Mean distance, in ticks, between a new limit order and the opposite best price.
    pub depth_scale: f64,
    pub max_qty: Qty,
    live: Vec<OrderId>,
    account: Account,
}

impl Noise {
    pub fn new(limit_rate: f64, market_rate: f64, cancel_rate: f64, depth_scale: f64, max_qty: Qty) -> Self {
        Noise { limit_rate, market_rate, cancel_rate, depth_scale, max_qty, live: Vec::new(), account: Account::default() }
    }

    fn side(rng: &mut Rng) -> Side {
        if rng.chance(0.5) {
            Side::Buy
        } else {
            Side::Sell
        }
    }
}

impl Agent for Noise {
    fn name(&self) -> &'static str {
        "noise"
    }

    fn act(&mut self, view: &View, rng: &mut Rng, out: &mut Vec<Action>) {
        // forget orders that were filled, so that the cancellation rate applies to resting orders only
        if view.time % 64 == 0 {
            self.live.retain(|id| view.book.contains(*id));
        }
        for _ in 0..rng.count(self.cancel_rate * self.live.len() as f64) {
            if self.live.is_empty() {
                break;
            }
            let i = rng.below(self.live.len() as u64) as usize;
            out.push(Action::Cancel(self.live.swap_remove(i)));
        }
        let reference = view.reference.round() as Price;
        for _ in 0..rng.count(self.limit_rate) {
            let side = Self::side(rng);
            let offset = 1 + rng.exponential(self.depth_scale) as Price;
            let price = match side {
                Side::Buy => view.book.best_ask().unwrap_or(reference + 1) - offset,
                Side::Sell => view.book.best_bid().unwrap_or(reference - 1) + offset,
            };
            out.push(Action::Limit { side, price, qty: 1 + rng.below(self.max_qty) });
        }
        for _ in 0..rng.count(self.market_rate) {
            out.push(Action::Market { side: Self::side(rng), qty: 1 + rng.below(self.max_qty) });
        }
    }

    fn placed(&mut self, id: OrderId) {
        self.live.push(id);
    }

    fn filled(&mut self, side: Side, price: Price, qty: Qty) {
        self.account.fill(side, price, qty);
    }

    fn inventory(&self) -> i64 {
        self.account.inventory
    }

    fn cash(&self) -> i64 {
        self.account.cash
    }
}

//! A trend follower: buys after the price has risen over a window, sells after it has fallen.

use super::{Account, Action, Agent, View};
use crate::math::rng::Rng;
use crate::types::{Price, Qty, Side};

pub struct Momentum {
    pub window: usize,
    /// Move, in ticks, that triggers a trade.
    pub threshold: f64,
    /// Probability of acting on a signal in a given step.
    pub rate: f64,
    pub qty: Qty,
    account: Account,
}

impl Momentum {
    pub fn new(window: usize, threshold: f64, rate: f64, qty: Qty) -> Self {
        Momentum { window, threshold, rate, qty, account: Account::default() }
    }
}

impl Agent for Momentum {
    fn name(&self) -> &'static str {
        "momentum"
    }

    fn act(&mut self, view: &View, rng: &mut Rng, out: &mut Vec<Action>) {
        if view.mids.len() <= self.window || !rng.chance(self.rate) {
            return;
        }
        let change = view.reference - view.mids[view.mids.len() - 1 - self.window];
        if change > self.threshold {
            out.push(Action::Market { side: Side::Buy, qty: self.qty });
        } else if change < -self.threshold {
            out.push(Action::Market { side: Side::Sell, qty: self.qty });
        }
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

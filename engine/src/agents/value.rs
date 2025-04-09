//! A value trader: buys when the price is below the fundamental value by more than a band, sells above it.

use super::{Account, Action, Agent, View};
use crate::math::rng::Rng;
use crate::types::{Price, Qty, Side};

pub struct Value {
    /// Mispricing, in ticks, tolerated before trading.
    pub band: f64,
    /// Probability of acting per step, per tick of mispricing beyond the band (capped at one).
    pub rate: f64,
    pub qty: Qty,
    account: Account,
}

impl Value {
    pub fn new(band: f64, rate: f64, qty: Qty) -> Self {
        Value { band, rate, qty, account: Account::default() }
    }
}

impl Agent for Value {
    fn name(&self) -> &'static str {
        "value"
    }

    fn act(&mut self, view: &View, rng: &mut Rng, out: &mut Vec<Action>) {
        let gap = view.fundamental - view.reference;
        let excess = gap.abs() - self.band;
        if excess <= 0.0 || !rng.chance((self.rate * excess).min(1.0)) {
            return;
        }
        out.push(Action::Market { side: if gap > 0.0 { Side::Buy } else { Side::Sell }, qty: self.qty });
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

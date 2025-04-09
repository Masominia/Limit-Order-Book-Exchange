//! A market maker: a ladder of quotes on both sides, shifted against its inventory, refreshed periodically.

use super::{Account, Action, Agent, View};
use crate::math::rng::Rng;
use crate::types::{OrderId, Price, Qty, Side};

pub struct Maker {
    /// Distance in ticks from the quote centre to the first quote on each side.
    pub half_spread: Price,
    pub levels: usize,
    pub qty: Qty,
    /// Ticks by which the quote centre moves down per lot of inventory held.
    pub skew: f64,
    /// Steps between two refreshes of the quotes.
    pub refresh: u64,
    quotes: Vec<OrderId>,
    account: Account,
}

impl Maker {
    pub fn new(half_spread: Price, levels: usize, qty: Qty, skew: f64, refresh: u64) -> Self {
        Maker { half_spread, levels, qty, skew, refresh: refresh.max(1), quotes: Vec::new(), account: Account::default() }
    }
}

impl Agent for Maker {
    fn name(&self) -> &'static str {
        "maker"
    }

    fn act(&mut self, view: &View, _rng: &mut Rng, out: &mut Vec<Action>) {
        if view.time % self.refresh != 0 {
            return;
        }
        out.extend(self.quotes.drain(..).map(Action::Cancel));
        let centre = view.reference - self.skew * self.account.inventory as f64;
        let (bid, ask) = ((centre - 0.25).floor() as Price - self.half_spread + 1, (centre + 0.25).ceil() as Price + self.half_spread - 1);
        for level in 0..self.levels as Price {
            out.push(Action::Limit { side: Side::Buy, price: bid - level, qty: self.qty });
            out.push(Action::Limit { side: Side::Sell, price: ask.max(bid + 1) + level, qty: self.qty });
        }
    }

    fn placed(&mut self, id: OrderId) {
        self.quotes.push(id);
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

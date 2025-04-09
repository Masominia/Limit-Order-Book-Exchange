//! Trading agents. Each sees the book and a little history, and answers with a list of actions.

pub mod maker;
pub mod momentum;
pub mod noise;
pub mod value;

use crate::book::Book;
use crate::math::rng::Rng;
use crate::types::{OrderId, Price, Qty, Side};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Limit { side: Side, price: Price, qty: Qty },
    Market { side: Side, qty: Qty },
    Cancel(OrderId),
}

/// What an agent may look at when it decides.
pub struct View<'a> {
    pub book: &'a Book,
    pub time: u64,
    /// Last mid price at which the book had both sides.
    pub reference: f64,
    /// The fundamental value, a random walk that only some agents use.
    pub fundamental: f64,
    /// Reference price at the end of every past step.
    pub mids: &'a [f64],
}

pub trait Agent {
    fn name(&self) -> &'static str;

    fn act(&mut self, view: &View, rng: &mut Rng, out: &mut Vec<Action>);

    /// Called when one of the agent's limit orders is left resting in the book.
    fn placed(&mut self, _id: OrderId) {}

    /// Called for every fill, with the agent's own side of the trade.
    fn filled(&mut self, _side: Side, _price: Price, _qty: Qty) {}

    /// Signed position in lots.
    fn inventory(&self) -> i64 {
        0
    }

    /// Cash received minus cash paid, in ticks times lots.
    fn cash(&self) -> i64 {
        0
    }
}

/// Position and cash bookkeeping shared by the agents that track them.
#[derive(Default, Clone, Copy, Debug)]
pub struct Account {
    pub inventory: i64,
    pub cash: i64,
}

impl Account {
    pub fn fill(&mut self, side: Side, price: Price, qty: Qty) {
        self.inventory += side.sign() * qty as i64;
        self.cash -= side.sign() * price * qty as i64;
    }

    /// Value of the account with the position marked at `price`.
    pub fn wealth(&self, price: f64) -> f64 {
        self.cash as f64 + self.inventory as f64 * price
    }
}

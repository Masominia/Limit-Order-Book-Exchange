//! Price-time priority book. Levels live in ordered maps, an index maps each resting order to its level.

use std::collections::{BTreeMap, HashMap};

use super::level::{Level, Resting};
use crate::types::{OrderId, Price, Qty, Side, Trade};

#[derive(Default)]
pub struct Book {
    pub(crate) bids: BTreeMap<Price, Level>,
    pub(crate) asks: BTreeMap<Price, Level>,
    pub(crate) index: HashMap<OrderId, (Side, Price)>,
    pub(crate) next_id: OrderId,
}

impl Book {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn fresh_id(&mut self) -> OrderId {
        self.next_id += 1;
        self.next_id
    }

    pub fn best_bid(&self) -> Option<Price> {
        self.bids.keys().next_back().copied()
    }

    pub fn best_ask(&self) -> Option<Price> {
        self.asks.keys().next().copied()
    }

    pub fn mid(&self) -> Option<f64> {
        Some(0.5 * (self.best_bid()? + self.best_ask()?) as f64)
    }

    pub fn spread(&self) -> Option<Price> {
        Some(self.best_ask()? - self.best_bid()?)
    }

    pub fn resting_orders(&self) -> usize {
        self.index.len()
    }

    pub fn contains(&self, id: OrderId) -> bool {
        self.index.contains_key(&id)
    }

    pub fn volume(&self, side: Side) -> Qty {
        self.side(side).values().map(|l| l.total).sum()
    }

    pub fn volume_at(&self, side: Side, price: Price) -> Qty {
        self.side(side).get(&price).map_or(0, |l| l.total)
    }

    /// The first `levels` price levels from the best outwards, as (price, quantity).
    pub fn depth(&self, side: Side, levels: usize) -> Vec<(Price, Qty)> {
        match side {
            Side::Buy => self.bids.iter().rev().take(levels).map(|(p, l)| (*p, l.total)).collect(),
            Side::Sell => self.asks.iter().take(levels).map(|(p, l)| (*p, l.total)).collect(),
        }
    }

    pub(crate) fn side(&self, side: Side) -> &BTreeMap<Price, Level> {
        match side {
            Side::Buy => &self.bids,
            Side::Sell => &self.asks,
        }
    }

    /// Limit order. Whatever crosses the book trades at once, the remainder rests.
    /// Returns the order id and the quantity left resting (zero if fully filled).
    pub fn limit(&mut self, owner: u32, side: Side, price: Price, qty: Qty, trades: &mut Vec<Trade>) -> (OrderId, Qty) {
        let id = self.fresh_id();
        let rest = self.sweep(id, owner, side, Some(price), qty, trades);
        if rest > 0 {
            let levels = match side {
                Side::Buy => &mut self.bids,
                Side::Sell => &mut self.asks,
            };
            levels.entry(price).or_default().push(Resting { id, owner, qty: rest });
            self.index.insert(id, (side, price));
        }
        (id, rest)
    }

    /// Market order. Returns the filled quantity; what the book cannot fill is dropped.
    pub fn market(&mut self, owner: u32, side: Side, qty: Qty, trades: &mut Vec<Trade>) -> Qty {
        let id = self.fresh_id();
        qty - self.sweep(id, owner, side, None, qty, trades)
    }

    /// Cancels a resting order and returns the quantity removed.
    pub fn cancel(&mut self, id: OrderId) -> Option<Qty> {
        let (side, price) = self.index.remove(&id)?;
        let levels = match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };
        let level = levels.get_mut(&price)?;
        let order = level.remove(id)?;
        if level.is_empty() {
            levels.remove(&price);
        }
        Some(order.qty)
    }
}

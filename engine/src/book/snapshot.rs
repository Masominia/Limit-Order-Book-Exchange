//! Top-of-book summary and internal consistency checks.

use super::orderbook::Book;
use crate::types::{Price, Qty, Side};

#[derive(Clone, Copy, Debug)]
pub struct Snapshot {
    pub bid: Option<Price>,
    pub ask: Option<Price>,
    pub bid_qty: Qty,
    pub ask_qty: Qty,
    /// (bid quantity - ask quantity) / (bid quantity + ask quantity) at the best prices.
    pub imbalance: f64,
}

impl Book {
    pub fn snapshot(&self) -> Snapshot {
        let (bid, ask) = (self.best_bid(), self.best_ask());
        let bid_qty = bid.map_or(0, |p| self.volume_at(Side::Buy, p));
        let ask_qty = ask.map_or(0, |p| self.volume_at(Side::Sell, p));
        let total = (bid_qty + ask_qty) as f64;
        let imbalance = if total > 0.0 { (bid_qty as f64 - ask_qty as f64) / total } else { 0.0 };
        Snapshot { bid, ask, bid_qty, ask_qty, imbalance }
    }

    /// Volume within `ticks` of the best price on one side.
    pub fn volume_near(&self, side: Side, ticks: Price) -> Qty {
        match side {
            Side::Buy => self.best_bid().map_or(0, |b| self.bids.range(b - ticks..=b).map(|(_, l)| l.total).sum()),
            Side::Sell => self.best_ask().map_or(0, |a| self.asks.range(a..=a + ticks).map(|(_, l)| l.total).sum()),
        }
    }

    /// Average price paid by a market order of the given size, without sending it. None if the book is too thin.
    pub fn quote(&self, side: Side, qty: Qty) -> Option<f64> {
        let (mut left, mut cost) = (qty, 0.0);
        let mut walk = |price: Price, total: Qty| {
            let take = left.min(total);
            cost += take as f64 * price as f64;
            left -= take;
        };
        match side {
            Side::Buy => self.asks.iter().for_each(|(p, l)| walk(*p, l.total)),
            Side::Sell => self.bids.iter().rev().for_each(|(p, l)| walk(*p, l.total)),
        }
        if left == 0 && qty > 0 {
            Some(cost / qty as f64)
        } else {
            None
        }
    }

    /// Verifies that the book is not crossed, that no level is empty, that level totals add up
    /// and that the index describes exactly the resting orders.
    pub fn check(&self) -> Result<(), String> {
        if let (Some(b), Some(a)) = (self.best_bid(), self.best_ask()) {
            if b >= a {
                return Err(format!("crossed book: bid {b}, ask {a}"));
            }
        }
        let mut count = 0;
        for (side, levels) in [(Side::Buy, &self.bids), (Side::Sell, &self.asks)] {
            for (price, level) in levels {
                if level.is_empty() {
                    return Err(format!("empty level at {price}"));
                }
                let mut sum = 0;
                for r in &level.queue {
                    if r.qty == 0 {
                        return Err(format!("order {} has zero quantity", r.id));
                    }
                    if self.index.get(&r.id) != Some(&(side, *price)) {
                        return Err(format!("order {} is not indexed at its level", r.id));
                    }
                    sum += r.qty;
                    count += 1;
                }
                if sum != level.total {
                    return Err(format!("level {price}: total {} but orders sum to {sum}", level.total));
                }
            }
        }
        if count != self.index.len() {
            return Err(format!("{count} resting orders but {} index entries", self.index.len()));
        }
        Ok(())
    }
}

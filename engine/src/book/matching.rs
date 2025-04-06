//! The matching loop shared by limit and market orders.

use super::orderbook::Book;
use crate::types::{OrderId, Price, Qty, Side, Trade};

impl Book {
    /// Trades `qty` against the opposite side, best price first and oldest order first within a price,
    /// never beyond `limit`. Returns the quantity left over.
    pub(crate) fn sweep(&mut self, taker: OrderId, owner: u32, side: Side, limit: Option<Price>, mut qty: Qty, trades: &mut Vec<Trade>) -> Qty {
        while qty > 0 {
            let best = match side {
                Side::Buy => self.best_ask(),
                Side::Sell => self.best_bid(),
            };
            let Some(price) = best else { break };
            if let Some(l) = limit {
                if (side == Side::Buy && price > l) || (side == Side::Sell && price < l) {
                    break;
                }
            }
            let levels = match side {
                Side::Buy => &mut self.asks,
                Side::Sell => &mut self.bids,
            };
            let level = levels.get_mut(&price).expect("best price has a level");
            while qty > 0 {
                let Some(front) = level.queue.front_mut() else { break };
                let fill = qty.min(front.qty);
                front.qty -= fill;
                level.total -= fill;
                qty -= fill;
                trades.push(Trade { maker: front.id, taker, maker_owner: front.owner, taker_owner: owner, taker_side: side, price, qty: fill });
                if front.qty == 0 {
                    let done = level.queue.pop_front().expect("front exists");
                    self.index.remove(&done.id);
                }
            }
            if level.is_empty() {
                levels.remove(&price);
            }
        }
        qty
    }
}

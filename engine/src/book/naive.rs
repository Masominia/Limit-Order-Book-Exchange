//! A deliberately simple book: one unsorted list, a full scan for every fill.
//! It exists only as an independent reference for the fast book in tests and in the `fuzz` command.

use crate::types::{Order, OrderId, Price, Qty, Side, Trade};

#[derive(Default)]
pub struct NaiveBook {
    pub orders: Vec<Order>,
    next_id: OrderId,
}

impl NaiveBook {
    pub fn new() -> Self {
        Self::default()
    }

    fn best(&self, side: Side) -> Option<usize> {
        let mut best: Option<usize> = None;
        for (i, o) in self.orders.iter().enumerate() {
            if o.side != side {
                continue;
            }
            let better = match best {
                None => true,
                Some(b) => {
                    let cur = &self.orders[b];
                    let improves = if side == Side::Buy { o.price > cur.price } else { o.price < cur.price };
                    improves || (o.price == cur.price && o.id < cur.id)
                }
            };
            if better {
                best = Some(i);
            }
        }
        best
    }

    fn sweep(&mut self, taker: OrderId, owner: u32, side: Side, limit: Option<Price>, mut qty: Qty, trades: &mut Vec<Trade>) -> Qty {
        while qty > 0 {
            let Some(i) = self.best(side.opposite()) else { break };
            let resting = self.orders[i];
            if let Some(l) = limit {
                if (side == Side::Buy && resting.price > l) || (side == Side::Sell && resting.price < l) {
                    break;
                }
            }
            let fill = qty.min(resting.qty);
            qty -= fill;
            trades.push(Trade { maker: resting.id, taker, maker_owner: resting.owner, taker_owner: owner, taker_side: side, price: resting.price, qty: fill });
            if fill == resting.qty {
                self.orders.remove(i);
            } else {
                self.orders[i].qty -= fill;
            }
        }
        qty
    }

    pub fn limit(&mut self, owner: u32, side: Side, price: Price, qty: Qty, trades: &mut Vec<Trade>) -> (OrderId, Qty) {
        self.next_id += 1;
        let id = self.next_id;
        let rest = self.sweep(id, owner, side, Some(price), qty, trades);
        if rest > 0 {
            self.orders.push(Order { id, owner, side, price, qty: rest });
        }
        (id, rest)
    }

    pub fn market(&mut self, owner: u32, side: Side, qty: Qty, trades: &mut Vec<Trade>) -> Qty {
        self.next_id += 1;
        let id = self.next_id;
        qty - self.sweep(id, owner, side, None, qty, trades)
    }

    pub fn cancel(&mut self, id: OrderId) -> Option<Qty> {
        let i = self.orders.iter().position(|o| o.id == id)?;
        Some(self.orders.remove(i).qty)
    }

    pub fn volume(&self, side: Side) -> Qty {
        self.orders.iter().filter(|o| o.side == side).map(|o| o.qty).sum()
    }
}

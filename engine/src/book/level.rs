//! One price level: a first-in first-out queue of resting orders.

use std::collections::VecDeque;

use crate::types::{OrderId, Qty};

#[derive(Clone, Copy, Debug)]
pub struct Resting {
    pub id: OrderId,
    pub owner: u32,
    pub qty: Qty,
}

#[derive(Default, Debug)]
pub struct Level {
    pub queue: VecDeque<Resting>,
    pub total: Qty,
}

impl Level {
    pub fn push(&mut self, order: Resting) {
        self.total += order.qty;
        self.queue.push_back(order);
    }

    /// Removes an order wherever it sits in the queue; the others keep their priority.
    pub fn remove(&mut self, id: OrderId) -> Option<Resting> {
        let position = self.queue.iter().position(|r| r.id == id)?;
        let order = self.queue.remove(position)?;
        self.total -= order.qty;
        Some(order)
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

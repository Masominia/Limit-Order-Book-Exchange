//! A limit order book exchange, a population of trading agents and execution schedules.
//!
//! Prices are integer ticks, quantities are integer lots, time is a step counter.

pub mod agents;
pub mod book;
pub mod exec;
pub mod math;
pub mod sim;
pub mod types;

pub use book::Book;
pub use sim::market::Market;
pub use types::{Order, OrderId, Price, Qty, Side, Trade};

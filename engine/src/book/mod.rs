//! The order book: price levels, matching, snapshots and a slow reference implementation.

pub mod level;
pub mod matching;
pub mod naive;
pub mod orderbook;
pub mod snapshot;

pub use orderbook::Book;

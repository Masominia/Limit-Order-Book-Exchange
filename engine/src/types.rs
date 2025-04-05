//! Basic vocabulary of the exchange.

pub type Price = i64;
pub type Qty = u64;
pub type OrderId = u64;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn opposite(self) -> Side {
        match self {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        }
    }

    /// +1 for a buy, -1 for a sell.
    pub fn sign(self) -> i64 {
        match self {
            Side::Buy => 1,
            Side::Sell => -1,
        }
    }

    pub fn parse(text: &str) -> Option<Side> {
        match text {
            "buy" => Some(Side::Buy),
            "sell" => Some(Side::Sell),
            _ => None,
        }
    }
}

/// A resting or incoming limit order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Order {
    pub id: OrderId,
    pub owner: u32,
    pub side: Side,
    pub price: Price,
    pub qty: Qty,
}

/// One fill. The price is always the resting order's price.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Trade {
    pub maker: OrderId,
    pub taker: OrderId,
    pub maker_owner: u32,
    pub taker_owner: u32,
    pub taker_side: Side,
    pub price: Price,
    pub qty: Qty,
}

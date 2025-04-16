//! A few orders by hand, printing the book after each.
//!
//!     cargo run --release --example book_demo

use lobsim::{Book, Side};

fn show(book: &Book) {
    let (bids, asks) = (book.depth(Side::Buy, 3), book.depth(Side::Sell, 3));
    let text = |levels: &[(i64, u64)]| levels.iter().map(|(p, q)| format!("{q}@{p}")).collect::<Vec<_>>().join("  ");
    println!("    bids: {:<28} asks: {}", text(&bids), text(&asks));
}

fn main() {
    let mut book = Book::new();
    let mut trades = Vec::new();
    for (side, price, qty) in [(Side::Buy, 99, 10), (Side::Buy, 98, 20), (Side::Sell, 101, 10), (Side::Sell, 102, 20)] {
        book.limit(1, side, price, qty, &mut trades);
    }
    println!("four resting orders");
    show(&book);
    let filled = book.market(2, Side::Buy, 15, &mut trades);
    println!("market buy of 15: filled {filled} in {} trades at {:?}", trades.len(), trades.iter().map(|t| (t.qty, t.price)).collect::<Vec<_>>());
    show(&book);
    trades.clear();
    let (id, rest) = book.limit(2, Side::Sell, 98, 35, &mut trades);
    println!("limit sell of 35 at 98: {} lots traded, {rest} resting", trades.iter().map(|t| t.qty).sum::<u64>());
    show(&book);
    println!("cancel it: {:?} lots removed", book.cancel(id));
    show(&book);
}

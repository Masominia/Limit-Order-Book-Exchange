use lobsim::{Book, Side};

#[test]
fn price_priority_then_time_priority() {
    let mut b = Book::new();
    let mut t = Vec::new();
    let (first, _) = b.limit(1, Side::Sell, 101, 5, &mut t);
    let (second, _) = b.limit(2, Side::Sell, 101, 5, &mut t);
    let (better, _) = b.limit(3, Side::Sell, 100, 5, &mut t);
    b.limit(9, Side::Buy, 101, 12, &mut t);
    let makers: Vec<_> = t.iter().map(|x| (x.maker, x.price, x.qty)).collect();
    assert_eq!(makers, vec![(better, 100, 5), (first, 101, 5), (second, 101, 2)]);
    assert_eq!(b.volume_at(Side::Sell, 101), 3);
    assert!(b.best_bid().is_none());
}

#[test]
fn trades_happen_at_the_resting_price() {
    let mut b = Book::new();
    let mut t = Vec::new();
    b.limit(1, Side::Buy, 100, 5, &mut t);
    b.limit(2, Side::Sell, 95, 5, &mut t);
    assert_eq!((t[0].price, t[0].qty, t[0].taker_side), (100, 5, Side::Sell));
    assert_eq!((t[0].maker_owner, t[0].taker_owner), (1, 2));
}

#[test]
fn a_limit_order_stops_at_its_price_and_rests() {
    let mut b = Book::new();
    let mut t = Vec::new();
    b.limit(1, Side::Sell, 100, 5, &mut t);
    b.limit(1, Side::Sell, 102, 5, &mut t);
    let (id, rest) = b.limit(2, Side::Buy, 101, 8, &mut t);
    assert_eq!((rest, t.len()), (3, 1));
    assert!(b.contains(id));
    assert_eq!((b.best_bid(), b.best_ask()), (Some(101), Some(102)));
    assert!(b.check().is_ok());
}

#[test]
fn market_orders_walk_the_book_and_drop_the_rest() {
    let mut b = Book::new();
    let mut t = Vec::new();
    b.limit(1, Side::Buy, 99, 4, &mut t);
    b.limit(1, Side::Buy, 97, 4, &mut t);
    assert_eq!(b.market(2, Side::Sell, 6, &mut t), 6);
    assert_eq!(t.iter().map(|x| x.price).collect::<Vec<_>>(), vec![99, 97]);
    assert_eq!(b.market(2, Side::Sell, 10, &mut t), 2);
    assert_eq!(b.market(2, Side::Sell, 10, &mut t), 0);
    assert_eq!(b.resting_orders(), 0);
}

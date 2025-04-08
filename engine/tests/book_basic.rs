use lobsim::{Book, Side};

#[test]
fn empty_book() {
    let b = Book::new();
    assert!(b.best_bid().is_none() && b.best_ask().is_none() && b.mid().is_none() && b.spread().is_none());
    assert_eq!(b.resting_orders(), 0);
    assert!(b.check().is_ok());
}

#[test]
fn resting_orders_form_the_quotes() {
    let mut b = Book::new();
    let mut t = Vec::new();
    b.limit(1, Side::Buy, 99, 10, &mut t);
    b.limit(1, Side::Buy, 98, 5, &mut t);
    b.limit(2, Side::Sell, 101, 7, &mut t);
    b.limit(2, Side::Sell, 103, 4, &mut t);
    assert!(t.is_empty());
    assert_eq!((b.best_bid(), b.best_ask(), b.spread()), (Some(99), Some(101), Some(2)));
    assert_eq!(b.mid(), Some(100.0));
    assert_eq!(b.depth(Side::Buy, 5), vec![(99, 10), (98, 5)]);
    assert_eq!(b.depth(Side::Sell, 1), vec![(101, 7)]);
    assert_eq!((b.volume(Side::Buy), b.volume(Side::Sell), b.volume_at(Side::Sell, 103)), (15, 11, 4));
    assert!(b.check().is_ok());
}

#[test]
fn snapshot_and_quote() {
    let mut b = Book::new();
    let mut t = Vec::new();
    b.limit(1, Side::Buy, 99, 30, &mut t);
    b.limit(2, Side::Sell, 101, 10, &mut t);
    b.limit(2, Side::Sell, 102, 10, &mut t);
    let s = b.snapshot();
    assert_eq!((s.bid, s.ask, s.bid_qty, s.ask_qty), (Some(99), Some(101), 30, 10));
    assert!((s.imbalance - 0.5).abs() < 1e-12);
    assert_eq!(b.quote(Side::Buy, 15), Some((10.0 * 101.0 + 5.0 * 102.0) / 15.0));
    assert_eq!(b.quote(Side::Buy, 21), None);
    assert_eq!(b.quote(Side::Sell, 30), Some(99.0));
    assert_eq!((b.volume_near(Side::Sell, 0), b.volume_near(Side::Sell, 1)), (10, 20));
}

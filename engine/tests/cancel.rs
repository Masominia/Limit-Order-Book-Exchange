use lobsim::{Book, Side};

#[test]
fn cancel_removes_only_that_order() {
    let mut b = Book::new();
    let mut t = Vec::new();
    let (a, _) = b.limit(1, Side::Buy, 99, 5, &mut t);
    let (c, _) = b.limit(1, Side::Buy, 99, 7, &mut t);
    let (d, _) = b.limit(1, Side::Buy, 99, 9, &mut t);
    assert_eq!(b.cancel(c), Some(7));
    assert_eq!(b.cancel(c), None);
    assert_eq!(b.volume_at(Side::Buy, 99), 14);
    b.market(2, Side::Sell, 6, &mut t);
    assert_eq!(t.iter().map(|x| (x.maker, x.qty)).collect::<Vec<_>>(), vec![(a, 5), (d, 1)]);
    assert!(b.check().is_ok());
}

#[test]
fn cancelling_the_last_order_removes_the_level() {
    let mut b = Book::new();
    let mut t = Vec::new();
    let (a, _) = b.limit(1, Side::Sell, 101, 5, &mut t);
    b.limit(1, Side::Sell, 104, 5, &mut t);
    b.cancel(a);
    assert_eq!(b.best_ask(), Some(104));
    assert!(b.check().is_ok());
}

#[test]
fn filled_orders_cannot_be_cancelled() {
    let mut b = Book::new();
    let mut t = Vec::new();
    let (a, _) = b.limit(1, Side::Sell, 101, 5, &mut t);
    b.market(2, Side::Buy, 5, &mut t);
    assert_eq!(b.cancel(a), None);
    assert_eq!(b.cancel(12345), None);
}

use lobsim::agents::maker::Maker;
use lobsim::agents::momentum::Momentum;
use lobsim::agents::value::Value;
use lobsim::agents::{Account, Action, Agent, View};
use lobsim::math::rng::Rng;
use lobsim::{Book, Side};

fn view<'a>(book: &'a Book, reference: f64, fundamental: f64, mids: &'a [f64], time: u64) -> View<'a> {
    View { book, time, reference, fundamental, mids }
}

#[test]
fn account_bookkeeping() {
    let mut a = Account::default();
    a.fill(Side::Buy, 100, 10);
    a.fill(Side::Sell, 103, 4);
    assert_eq!((a.inventory, a.cash), (6, -1000 + 412));
    assert_eq!(a.wealth(102.0), -588.0 + 612.0);
}

#[test]
fn maker_quotes_both_sides_and_leans_against_inventory() {
    let book = Book::new();
    let mut rng = Rng::new(1);
    let mut maker = Maker::new(2, 3, 5, 0.5, 1);
    let mut out = Vec::new();
    maker.act(&view(&book, 1000.0, 1000.0, &[], 1), &mut rng, &mut out);
    let bids: Vec<i64> = out.iter().filter_map(|a| if let Action::Limit { side: Side::Buy, price, .. } = a { Some(*price) } else { None }).collect();
    let asks: Vec<i64> = out.iter().filter_map(|a| if let Action::Limit { side: Side::Sell, price, .. } = a { Some(*price) } else { None }).collect();
    assert_eq!((bids.len(), asks.len()), (3, 3));
    let centre_flat = (bids[0] + asks[0]) as f64 / 2.0;
    assert!(bids[0] < asks[0] && (centre_flat - 1000.0).abs() <= 0.5);

    maker.filled(Side::Buy, 1000, 10);
    assert_eq!(maker.inventory(), 10);
    out.clear();
    maker.act(&view(&book, 1000.0, 1000.0, &[], 2), &mut rng, &mut out);
    let bid = out.iter().find_map(|a| if let Action::Limit { side: Side::Buy, price, .. } = a { Some(*price) } else { None }).unwrap();
    assert!(bid <= bids[0] - 4, "a long maker lowers its quotes: {bid} against {}", bids[0]);
}

#[test]
fn momentum_follows_and_value_opposes() {
    let book = Book::new();
    let mut rng = Rng::new(1);
    let rising: Vec<f64> = (0..20).map(|i| 1000.0 + i as f64).collect();
    let mut out = Vec::new();
    Momentum::new(10, 5.0, 1.0, 3).act(&view(&book, 1019.0, 1000.0, &rising, 20), &mut rng, &mut out);
    assert_eq!(out, vec![Action::Market { side: Side::Buy, qty: 3 }]);
    out.clear();
    Value::new(3.0, 1.0, 3).act(&view(&book, 1019.0, 1000.0, &rising, 20), &mut rng, &mut out);
    assert_eq!(out, vec![Action::Market { side: Side::Sell, qty: 3 }]);
    out.clear();
    Value::new(3.0, 1.0, 3).act(&view(&book, 1001.0, 1000.0, &rising, 20), &mut rng, &mut out);
    assert!(out.is_empty());
}

use lobsim::agents::Action;
use lobsim::sim::market::Market;
use lobsim::sim::metrics::summarise;
use lobsim::sim::params::Params;
use lobsim::Side;

fn quick() -> Params {
    Params { warmup: 500, ..Params::default() }
}

#[test]
fn same_seed_same_tape() {
    let (mut a, mut b, mut c) = (Market::standard(&quick(), 5), Market::standard(&quick(), 5), Market::standard(&quick(), 6));
    a.run(2000);
    b.run(2000);
    c.run(2000);
    assert_eq!(a.tape, b.tape);
    assert_ne!(a.tape, c.tape);
}

#[test]
fn the_book_stays_consistent_and_positions_net_to_zero() {
    let mut m = Market::standard(&quick(), 2);
    for _ in 0..50 {
        m.run(100);
        m.book.check().unwrap();
    }
    let (inventory, cash) = m.agents().iter().fold((0, 0), |acc, a| (acc.0 + a.inventory(), acc.1 + a.cash()));
    assert_eq!((inventory, cash), (0, 0));
    assert_eq!(m.records.len(), 5000);
}

#[test]
fn external_orders_are_filled_and_reported() {
    let mut m = Market::standard(&quick(), 3);
    let ask = m.book.best_ask().unwrap();
    let (resting, trades) = m.submit(Action::Market { side: Side::Buy, qty: 3 });
    assert!(resting.is_none());
    assert_eq!(trades.iter().map(|t| t.qty).sum::<u64>(), 3);
    assert!(trades.iter().all(|t| t.price >= ask && t.taker_side == Side::Buy));
    let (resting, trades) = m.submit(Action::Limit { side: Side::Buy, price: ask - 500, qty: 1 });
    assert!(resting.is_some() && trades.is_empty());
}

#[test]
fn value_traders_anchor_the_price() {
    let free = Params { value: 0, ..quick() };
    let (mut with, mut without) = (Market::standard(&quick(), 9), Market::standard(&free, 9));
    let (mut g1, mut g2) = (Vec::new(), Vec::new());
    for _ in 0..20_000 {
        with.step();
        without.step();
        g1.push(with.reference - with.fundamental);
        g2.push(without.reference - without.fundamental);
    }
    let (s1, s2) = (summarise(&with, 50, &g1), summarise(&without, 50, &g2));
    assert!(s1.mispricing < 0.6 * s2.mispricing, "{} against {}", s1.mispricing, s2.mispricing);
    assert!(s1.mean_spread >= 1.0 && s1.mean_spread < 5.0 && s1.one_sided < 0.01);
}

#[test]
fn parameters_are_read_from_text() {
    let map = [("makers", "0"), ("depth_scale", "12.5"), ("ignored", "x")].iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    let p = Params::from_map(&map).unwrap();
    assert_eq!((p.makers, p.depth_scale, p.momentum), (0, 12.5, 1));
    let bad = [("makers", "many")].iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    assert!(Params::from_map(&bad).is_err());
}

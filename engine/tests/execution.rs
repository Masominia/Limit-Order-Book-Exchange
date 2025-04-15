use lobsim::exec::immediate::Immediate;
use lobsim::exec::runner::execute;
use lobsim::exec::twap::Twap;
use lobsim::exec::Schedule;
use lobsim::math::stats;
use lobsim::sim::market::Market;
use lobsim::sim::params::Params;
use lobsim::Side;

fn costs(schedule: &dyn Schedule, runs: u64) -> Vec<f64> {
    let params = Params { warmup: 1500, ..Params::default() };
    (0..runs)
        .map(|i| {
            let mut m = Market::standard(&params, 100 + i);
            let side = if i % 2 == 0 { Side::Buy } else { Side::Sell };
            let r = execute(&mut m, side, 150, 100, schedule);
            assert_eq!(r.filled, 150);
            r.shortfall_bps
        })
        .collect()
}

#[test]
fn crossing_the_spread_costs_money() {
    let c = costs(&Immediate, 120);
    assert!(stats::mean(&c) > 5.0 * stats::std_error(&c) && stats::mean(&c) > 1.0);
}

#[test]
fn slicing_trades_impact_for_risk() {
    let (now, sliced) = (costs(&Immediate, 200), costs(&Twap, 200));
    assert!(stats::mean(&sliced) < stats::mean(&now), "{} against {}", stats::mean(&sliced), stats::mean(&now));
    assert!(stats::std_dev(&sliced) > stats::std_dev(&now));
}

#[test]
fn report_is_consistent() {
    let mut m = Market::standard(&Params { warmup: 1500, ..Params::default() }, 4);
    let r = execute(&mut m, Side::Sell, 60, 30, &Twap);
    assert!(r.child_orders <= 30 && r.child_orders >= 25);
    assert!((r.shortfall_bps - (r.arrival - r.average) / r.arrival * 1e4).abs() < 1e-9);
}

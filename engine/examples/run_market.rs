//! Run the standard population and print what the market looks like.
//!
//!     cargo run --release --example run_market

use lobsim::sim::market::Market;
use lobsim::sim::metrics::summarise;
use lobsim::sim::params::Params;

fn main() {
    let mut market = Market::standard(&Params::default(), 1);
    let mut gap = Vec::new();
    for _ in 0..50_000 {
        market.step();
        gap.push(market.reference - market.fundamental);
    }
    let s = summarise(&market, 50, &gap);
    println!("{} steps, {} trades, {:.2} lots per step", s.steps, s.trades, s.volume_per_step);
    println!("spread {:.2} ticks, depth near the touch {:.0} lots", s.mean_spread, s.mean_depth);
    println!("volatility {:.3} ticks per step, variance ratio {:.2}, excess kurtosis {:.2}", s.volatility, s.variance_ratio, s.kurtosis);
    println!("price minus fundamental value: {:.2} ticks (standard deviation)", s.mispricing);
    for agent in market.agents() {
        println!("{:<9} inventory {:>6}  wealth {:>10.0}", agent.name(), agent.inventory(), agent.cash() as f64 + agent.inventory() as f64 * market.reference);
    }
}

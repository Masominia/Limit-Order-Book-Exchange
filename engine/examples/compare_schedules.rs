//! The same parent order under four schedules.
//!
//!     cargo run --release --example compare_schedules

use lobsim::exec::frontloaded::FrontLoaded;
use lobsim::exec::immediate::Immediate;
use lobsim::exec::pov::Pov;
use lobsim::exec::runner::execute;
use lobsim::exec::twap::Twap;
use lobsim::exec::Schedule;
use lobsim::math::stats;
use lobsim::sim::market::Market;
use lobsim::sim::params::Params;
use lobsim::Side;

fn main() {
    let schedules: Vec<Box<dyn Schedule>> = vec![Box::new(Immediate), Box::new(FrontLoaded { urgency: 4.0 }), Box::new(Twap), Box::new(Pov { rate: 0.2 })];
    println!("{:<14} {:>10} {:>10}", "schedule", "cost, bp", "risk, bp");
    for s in &schedules {
        let costs: Vec<f64> = (0..300)
            .map(|i| {
                let mut m = Market::standard(&Params::default(), i);
                execute(&mut m, if i % 2 == 0 { Side::Buy } else { Side::Sell }, 200, 200, s.as_ref()).shortfall_bps
            })
            .collect();
        println!("{:<14} {:>10.2} {:>10.2}", s.name(), stats::mean(&costs), stats::std_dev(&costs));
    }
}

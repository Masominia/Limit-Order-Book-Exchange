//! Command line front end: `lobsim <command> key=value ...`, output as `key value` lines and `row ...` lines.
//!
//! Commands: fuzz, bench, simulate, profile, execute, impact. Any field of `sim::params::Params` can be overridden.

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::process;
use std::time::Instant;

use lobsim::agents::Action;
use lobsim::book::naive::NaiveBook;
use lobsim::exec::frontloaded::FrontLoaded;
use lobsim::exec::immediate::Immediate;
use lobsim::exec::pov::Pov;
use lobsim::exec::runner::execute;
use lobsim::exec::twap::Twap;
use lobsim::exec::vwap::Vwap;
use lobsim::exec::Schedule;
use lobsim::math::rng::Rng;
use lobsim::math::stats;
use lobsim::sim::metrics::summarise;
use lobsim::sim::params::Params;
use lobsim::{Book, Market, Side, Trade};

fn fail(msg: &str) -> ! {
    eprintln!("lobsim: {msg}");
    process::exit(1)
}

struct Args(HashMap<String, String>);

impl Args {
    fn num(&self, key: &str, default: f64) -> f64 {
        self.0.get(key).map_or(default, |v| v.parse().unwrap_or_else(|_| fail(&format!("{key} is not a number"))))
    }
    fn int(&self, key: &str, default: u64) -> u64 {
        self.num(key, default as f64) as u64
    }
    fn text(&self, key: &str, default: &str) -> String {
        self.0.get(key).cloned().unwrap_or_else(|| default.to_string())
    }
    fn params(&self) -> Params {
        Params::from_map(&self.0).unwrap_or_else(|e| fail(&e))
    }
    /// The side for run number `i`: fixed if `side=buy|sell` was given, alternating otherwise.
    fn side(&self, i: u64) -> Side {
        match self.0.get("side") {
            Some(s) => Side::parse(s).unwrap_or_else(|| fail("side must be buy or sell")),
            None => {
                if i % 2 == 0 {
                    Side::Buy
                } else {
                    Side::Sell
                }
            }
        }
    }
}

/// One random order-flow event applied to both books; returns false on the first disagreement.
fn fuzz(orders: u64, seed: u64) -> (u64, u64, u64) {
    let mut rng = Rng::new(seed);
    let (mut fast, mut slow) = (Book::new(), NaiveBook::new());
    let (mut a, mut b): (Vec<Trade>, Vec<Trade>) = (Vec::new(), Vec::new());
    let (mut mismatches, mut invariant_failures, mut trades) = (0u64, 0u64, 0u64);
    let mut ids = Vec::new();
    for _ in 0..orders {
        a.clear();
        b.clear();
        let side = if rng.chance(0.5) { Side::Buy } else { Side::Sell };
        let qty = 1 + rng.below(20);
        let kind = rng.uniform();
        let same = if kind < 0.65 {
            let price = 1000 + rng.below(40) as i64 - 20 - 6 * side.sign();
            let (x, y) = (fast.limit(0, side, price, qty, &mut a), slow.limit(0, side, price, qty, &mut b));
            if x.1 > 0 {
                ids.push(x.0);
            }
            x == y
        } else if kind < 0.8 {
            fast.market(0, side, qty, &mut a) == slow.market(0, side, qty, &mut b)
        } else if !ids.is_empty() {
            let id = ids.swap_remove(rng.below(ids.len() as u64) as usize);
            fast.cancel(id) == slow.cancel(id)
        } else {
            true
        };
        trades += a.len() as u64;
        let volumes = fast.volume(Side::Buy) == slow.volume(Side::Buy) && fast.volume(Side::Sell) == slow.volume(Side::Sell);
        if !(same && a == b && volumes) {
            mismatches += 1;
        }
        if fast.check().is_err() {
            invariant_failures += 1;
        }
    }
    (trades, mismatches, invariant_failures)
}

fn bench(orders: u64, seed: u64) -> (f64, usize, usize) {
    let mut rng = Rng::new(seed);
    let mut book = Book::new();
    let mut trades = Vec::with_capacity(1024);
    let mut ids: Vec<u64> = Vec::new();
    let mut count = 0usize;
    let t0 = Instant::now();
    for _ in 0..orders {
        trades.clear();
        let side = if rng.chance(0.5) { Side::Buy } else { Side::Sell };
        let kind = rng.uniform();
        if kind < 0.6 {
            let price = 10_000 - side.sign() * (1 + rng.exponential(8.0) as i64);
            let (id, rest) = book.limit(0, side, price, 1 + rng.below(10), &mut trades);
            if rest > 0 {
                ids.push(id);
            }
        } else if kind < 0.7 {
            book.market(0, side, 1 + rng.below(10), &mut trades);
        } else if !ids.is_empty() {
            let id = ids.swap_remove(rng.below(ids.len() as u64) as usize);
            book.cancel(id);
        }
        count += trades.len();
    }
    (orders as f64 / t0.elapsed().as_secs_f64(), count, book.resting_orders())
}

fn schedule(a: &Args, horizon: usize) -> Box<dyn Schedule> {
    match a.text("schedule", "twap").as_str() {
        "twap" => Box::new(Twap),
        "immediate" => Box::new(Immediate),
        "frontloaded" => Box::new(FrontLoaded { urgency: a.num("urgency", 2.0) }),
        "pov" => Box::new(Pov { rate: a.num("rate", 0.2) }),
        "vwap" => Box::new(Vwap::u_shape(horizon.clamp(1, 20), a.num("depth", 2.0))),
        _ => fail("schedule must be twap, immediate, frontloaded, pov or vwap"),
    }
}

fn main() {
    let mut argv = env::args().skip(1);
    let command = argv.next().unwrap_or_else(|| fail("usage: lobsim <command> key=value ..."));
    let a = Args(argv.filter_map(|s| s.split_once('=').map(|(k, v)| (k.to_string(), v.to_string()))).collect());
    let seed = a.int("seed", 1);
    let t0 = Instant::now();
    match command.as_str() {
        "fuzz" => {
            let orders = a.int("orders", 20_000);
            let (trades, mismatches, invariant_failures) = fuzz(orders, seed);
            println!("orders {orders}\ntrades {trades}\nmismatches {mismatches}\ninvariant_failures {invariant_failures}");
        }
        "bench" => {
            let (rate, trades, resting) = bench(a.int("orders", 2_000_000), seed);
            println!("orders_per_second {rate:.0}\ntrades {trades}\nresting {resting}");
        }
        "simulate" => {
            let (steps, lag) = (a.int("steps", 100_000), a.int("lag", 50) as usize);
            let mut m = Market::standard(&a.params(), seed);
            let mut gap = Vec::with_capacity(steps as usize);
            let mut check_failures = 0;
            for i in 0..steps {
                m.step();
                gap.push(m.reference - m.fundamental);
                if i % 1000 == 0 && m.book.check().is_err() {
                    check_failures += 1;
                }
            }
            let s = summarise(&m, lag, &gap);
            println!("steps {}\ntrades {}\nvolume_per_step {:.6}\nmean_spread {:.6}\nmean_depth {:.6}", s.steps, s.trades, s.volume_per_step, s.mean_spread, s.mean_depth);
            println!("one_sided {:.6}\nvolatility {:.6}\nvariance_ratio {:.6}\nkurtosis {:.6}", s.one_sided, s.volatility, s.variance_ratio, s.kurtosis);
            println!("return_autocorrelation {:.6}\nflow_autocorrelation {:.6}\nmispricing {:.6}", s.return_autocorrelation, s.flow_autocorrelation, s.mispricing);
            println!("resting_orders {}\ncheck_failures {check_failures}", m.book.resting_orders());
            let (inventory, cash): (i64, i64) = m.agents().iter().fold((0, 0), |acc, ag| (acc.0 + ag.inventory(), acc.1 + ag.cash()));
            println!("net_inventory {inventory}\nnet_cash {cash}");
            for (i, ag) in m.agents().iter().enumerate() {
                let wealth = ag.cash() as f64 + ag.inventory() as f64 * m.reference;
                println!("row agent {i} {} {} {:.1}", ag.name(), ag.inventory(), wealth / steps as f64);
            }
            if let Some(path) = a.0.get("out") {
                let every = a.int("every", 10) as usize;
                let mut w = BufWriter::new(File::create(path).unwrap_or_else(|_| fail("cannot write the output file")));
                writeln!(w, "step,mid,spread,depth,volume").unwrap();
                for (i, r) in m.records.iter().enumerate().step_by(every.max(1)) {
                    writeln!(w, "{i},{},{},{},{}", r.mid, r.spread, r.depth, r.volume).unwrap();
                }
            }
        }
        "profile" => {
            // average resting volume at each distance from the mid price
            let (steps, width) = (a.int("steps", 50_000), a.int("width", 30) as usize);
            let mut m = Market::standard(&a.params(), seed);
            let mut sum = vec![0.0; width];
            let mut samples = 0.0;
            for i in 0..steps {
                m.step();
                if i % 10 != 0 {
                    continue;
                }
                let Some(mid) = m.book.mid() else { continue };
                samples += 1.0;
                for side in [Side::Buy, Side::Sell] {
                    for (price, qty) in m.book.depth(side, width) {
                        let d = ((price as f64 - mid).abs() - 0.5).ceil() as usize;
                        if d >= 1 && d <= width {
                            sum[d - 1] += 0.5 * qty as f64;
                        }
                    }
                }
            }
            for (d, s) in sum.iter().enumerate() {
                println!("row {} {:.6}", d + 1, s / samples);
            }
        }
        "execute" => {
            let (size, horizon, seeds) = (a.int("size", 200), a.int("horizon", 200) as usize, a.int("seeds", 500));
            let params = a.params();
            let plan = schedule(&a, horizon);
            let (mut cost, mut drift, mut filled, mut children) = (Vec::new(), Vec::new(), 0u64, 0usize);
            for i in 0..seeds {
                let mut m = Market::standard(&params, seed + i);
                let r = execute(&mut m, a.side(i), size, horizon, plan.as_ref());
                filled += r.filled;
                children += r.child_orders;
                if r.filled > 0 {
                    cost.push(r.shortfall_bps);
                    drift.push(r.drift_bps);
                }
            }
            println!("schedule {}", plan.name().replace('-', "_"));
            println!("mean_shortfall {:.6}\nstd_shortfall {:.6}\nstd_error {:.6}", stats::mean(&cost), stats::std_dev(&cost), stats::std_error(&cost));
            println!("mean_drift {:.6}\ndrift_std_error {:.6}", stats::mean(&drift), stats::std_error(&drift));
            println!("fill_rate {:.6}\nchild_orders {:.3}", filled as f64 / (size * seeds) as f64, children as f64 / seeds as f64);
        }
        "impact" => {
            let (seeds, relax) = (a.int("seeds", 500), a.int("relax", 200));
            let params = a.params();
            let sizes: Vec<u64> = a.text("sizes", "5,10,20,40,80,160").split(',').map(|s| s.parse().unwrap_or_else(|_| fail("bad sizes"))).collect();
            for &size in &sizes {
                let (mut cost, mut moved, mut later, mut unfilled) = (Vec::new(), Vec::new(), Vec::new(), 0u64);
                for i in 0..seeds {
                    let mut m = Market::standard(&params, seed + i);
                    let side = a.side(i);
                    let sign = side.sign() as f64;
                    let arrival = m.reference;
                    let (_, trades) = m.submit(Action::Market { side, qty: size });
                    let filled: u64 = trades.iter().map(|t| t.qty).sum();
                    if filled < size {
                        unfilled += 1;
                        continue;
                    }
                    let paid: f64 = trades.iter().map(|t| t.qty as f64 * t.price as f64).sum::<f64>() / filled as f64;
                    cost.push(sign * (paid - arrival) / arrival * 1e4);
                    moved.push(sign * (m.book.mid().unwrap_or(arrival) - arrival) / arrival * 1e4);
                    m.run(relax);
                    later.push(sign * (m.reference - arrival) / arrival * 1e4);
                }
                println!(
                    "row {size} {:.6} {:.6} {:.6} {:.6} {:.6} {:.6} {unfilled}",
                    stats::mean(&cost),
                    stats::std_error(&cost),
                    stats::mean(&moved),
                    stats::std_error(&moved),
                    stats::mean(&later),
                    stats::std_error(&later)
                );
            }
        }
        _ => fail("unknown command"),
    }
    println!("seconds {:.6}", t0.elapsed().as_secs_f64());
}

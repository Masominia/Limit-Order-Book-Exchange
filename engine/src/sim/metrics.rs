//! Summary statistics of a recorded run.

use crate::math::stats;
use crate::sim::market::Market;

#[derive(Clone, Copy, Debug)]
pub struct Summary {
    pub steps: usize,
    pub trades: usize,
    pub volume_per_step: f64,
    pub mean_spread: f64,
    pub mean_depth: f64,
    /// Share of steps in which one side of the book was empty.
    pub one_sided: f64,
    /// Standard deviation of the one-step change of the mid price, in ticks.
    pub volatility: f64,
    /// Variance of `lag`-step changes divided by `lag` times the one-step variance. One for a random walk.
    pub variance_ratio: f64,
    /// Excess kurtosis of `lag`-step changes.
    pub kurtosis: f64,
    /// Lag-one autocorrelation of one-step mid changes.
    pub return_autocorrelation: f64,
    /// Lag-one autocorrelation of the sign of the net traded volume, over steps that traded.
    pub flow_autocorrelation: f64,
    /// Standard deviation of (mid - fundamental value), in ticks.
    pub mispricing: f64,
}

pub fn summarise(market: &Market, lag: usize, mispricing: &[f64]) -> Summary {
    let r = &market.records;
    let mids: Vec<f64> = r.iter().map(|x| x.mid).collect();
    let one = stats::differences(&mids, 1);
    let many = stats::differences(&mids, lag);
    let spreads: Vec<f64> = r.iter().map(|x| x.spread).filter(|s| !s.is_nan()).collect();
    let depth: Vec<f64> = r.iter().map(|x| x.depth).collect();
    let signs: Vec<f64> = r.iter().filter(|x| x.signed_volume != 0).map(|x| x.signed_volume.signum() as f64).collect();
    Summary {
        steps: r.len(),
        trades: market.tape.len(),
        volume_per_step: r.iter().map(|x| x.volume as f64).sum::<f64>() / r.len().max(1) as f64,
        mean_spread: stats::mean(&spreads),
        mean_depth: stats::mean(&depth),
        one_sided: 1.0 - spreads.len() as f64 / r.len().max(1) as f64,
        volatility: stats::std_dev(&one),
        variance_ratio: stats::variance(&many) / (lag as f64 * stats::variance(&one)),
        kurtosis: stats::excess_kurtosis(&many),
        return_autocorrelation: stats::autocorrelation(&one, 1),
        flow_autocorrelation: stats::autocorrelation(&signs, 1),
        mispricing: stats::std_dev(mispricing),
    }
}

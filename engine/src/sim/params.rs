//! Parameters of the standard population, with defaults, readable from `key=value` pairs.

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Params {
    /// Starting price in ticks. With 10,000, one tick is one basis point.
    pub start: f64,
    /// Standard deviation of the fundamental value per step, in ticks.
    pub fundamental_vol: f64,
    pub limit_rate: f64,
    pub market_rate: f64,
    pub cancel_rate: f64,
    pub depth_scale: f64,
    pub max_qty: u64,
    pub makers: usize,
    pub maker_half_spread: i64,
    pub maker_levels: usize,
    pub maker_qty: u64,
    pub maker_skew: f64,
    pub maker_refresh: u64,
    pub momentum: usize,
    pub momentum_window: usize,
    pub momentum_threshold: f64,
    pub momentum_rate: f64,
    pub value: usize,
    pub value_band: f64,
    pub value_rate: f64,
    /// Steps run before anything is recorded.
    pub warmup: u64,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            start: 10_000.0,
            fundamental_vol: 0.5,
            limit_rate: 1.0,
            market_rate: 0.15,
            cancel_rate: 0.005,
            depth_scale: 8.0,
            max_qty: 10,
            makers: 1,
            maker_half_spread: 2,
            maker_levels: 3,
            maker_qty: 8,
            maker_skew: 0.02,
            maker_refresh: 5,
            momentum: 1,
            momentum_window: 50,
            momentum_threshold: 6.0,
            momentum_rate: 0.05,
            value: 1,
            value_band: 3.0,
            value_rate: 0.03,
            warmup: 3000,
        }
    }
}

impl Params {
    /// Overrides the defaults with any matching keys; unknown keys are ignored, bad numbers are reported.
    pub fn from_map(map: &HashMap<String, String>) -> Result<Params, String> {
        let mut p = Params::default();
        macro_rules! read {
            ($($field:ident),*) => {$(
                if let Some(text) = map.get(stringify!($field)) {
                    p.$field = text.parse().map_err(|_| format!("{} is not a valid number", stringify!($field)))?;
                }
            )*};
        }
        read!(
            start, fundamental_vol, limit_rate, market_rate, cancel_rate, depth_scale, max_qty, makers, maker_half_spread, maker_levels, maker_qty,
            maker_skew, maker_refresh, momentum, momentum_window, momentum_threshold, momentum_rate, value, value_band, value_rate, warmup
        );
        Ok(p)
    }
}

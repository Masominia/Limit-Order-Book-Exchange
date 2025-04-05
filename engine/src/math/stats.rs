//! Descriptive statistics on slices.

pub fn mean(x: &[f64]) -> f64 {
    if x.is_empty() {
        return f64::NAN;
    }
    x.iter().sum::<f64>() / x.len() as f64
}

pub fn variance(x: &[f64]) -> f64 {
    if x.len() < 2 {
        return f64::NAN;
    }
    let m = mean(x);
    x.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / (x.len() - 1) as f64
}

pub fn std_dev(x: &[f64]) -> f64 {
    variance(x).sqrt()
}

pub fn std_error(x: &[f64]) -> f64 {
    std_dev(x) / (x.len() as f64).sqrt()
}

/// Kurtosis minus three: zero for a normal distribution.
pub fn excess_kurtosis(x: &[f64]) -> f64 {
    let m = mean(x);
    let (mut m2, mut m4) = (0.0, 0.0);
    for v in x {
        let d = (v - m) * (v - m);
        m2 += d;
        m4 += d * d;
    }
    let n = x.len() as f64;
    (m4 / n) / ((m2 / n) * (m2 / n)) - 3.0
}

pub fn autocorrelation(x: &[f64], lag: usize) -> f64 {
    if x.len() <= lag + 1 {
        return f64::NAN;
    }
    let m = mean(x);
    let num: f64 = (lag..x.len()).map(|i| (x[i] - m) * (x[i - lag] - m)).sum();
    let den: f64 = x.iter().map(|v| (v - m) * (v - m)).sum();
    num / den
}

/// Differences x[i + lag] - x[i] taken every `lag` points (no overlap).
pub fn differences(x: &[f64], lag: usize) -> Vec<f64> {
    (lag..x.len()).step_by(lag).map(|i| x[i] - x[i - lag]).collect()
}

/// Least-squares slope and intercept of y on x.
pub fn line_fit(x: &[f64], y: &[f64]) -> (f64, f64) {
    let (mx, my) = (mean(x), mean(y));
    let sxy: f64 = x.iter().zip(y).map(|(a, b)| (a - mx) * (b - my)).sum();
    let sxx: f64 = x.iter().map(|a| (a - mx) * (a - mx)).sum();
    (sxy / sxx, my - sxy / sxx * mx)
}

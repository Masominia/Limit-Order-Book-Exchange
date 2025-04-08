use lobsim::math::rng::Rng;
use lobsim::math::stats::{autocorrelation, differences, excess_kurtosis, line_fit, mean, std_dev, std_error, variance};

#[test]
fn generator_moments() {
    let mut rng = Rng::new(11);
    let u: Vec<f64> = (0..100_000).map(|_| rng.uniform()).collect();
    let z: Vec<f64> = (0..100_000).map(|_| rng.normal()).collect();
    let e: Vec<f64> = (0..100_000).map(|_| rng.exponential(3.0)).collect();
    assert!((mean(&u) - 0.5).abs() < 0.005 && (variance(&u) - 1.0 / 12.0).abs() < 0.002);
    assert!(mean(&z).abs() < 0.015 && (std_dev(&z) - 1.0).abs() < 0.01 && excess_kurtosis(&z).abs() < 0.08);
    assert!((mean(&e) - 3.0).abs() < 0.05 && (excess_kurtosis(&e) - 6.0).abs() < 0.8);
    assert!(autocorrelation(&z, 1).abs() < 0.015);
}

#[test]
fn integer_draws() {
    let mut rng = Rng::new(2);
    let mut seen = [0u32; 6];
    for _ in 0..60_000 {
        seen[rng.below(6) as usize] += 1;
    }
    assert!(seen.iter().all(|&c| (c as f64 - 10_000.0).abs() < 400.0));
    let events: u64 = (0..20_000).map(|_| rng.count(2.3)).sum();
    assert!((events as f64 / 20_000.0 - 2.3).abs() < 0.02);
    let (mut a, mut b) = (Rng::new(5), Rng::new(5));
    assert_eq!(a.next_u64(), b.next_u64());
}

#[test]
fn statistics_on_known_data() {
    let x = [1.0, 2.0, 3.0, 4.0, 5.0];
    assert_eq!((mean(&x), variance(&x)), (3.0, 2.5));
    assert!((std_error(&x) - (2.5f64 / 5.0).sqrt()).abs() < 1e-15);
    assert_eq!(differences(&[0.0, 1.0, 3.0, 6.0, 10.0], 2), vec![3.0, 7.0]);
    let y: Vec<f64> = x.iter().map(|v| 2.0 * v - 1.0).collect();
    let (slope, intercept) = line_fit(&x, &y);
    assert!((slope - 2.0).abs() < 1e-12 && (intercept + 1.0).abs() < 1e-12);
    let alternating: Vec<f64> = (0..100).map(|i| if i % 2 == 0 { 1.0 } else { -1.0 }).collect();
    assert!(autocorrelation(&alternating, 1) < -0.98);
}

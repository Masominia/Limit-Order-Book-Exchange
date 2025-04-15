use lobsim::exec::frontloaded::FrontLoaded;
use lobsim::exec::immediate::Immediate;
use lobsim::exec::pov::Pov;
use lobsim::exec::twap::Twap;
use lobsim::exec::vwap::Vwap;
use lobsim::exec::Schedule;

fn path(s: &dyn Schedule, horizon: usize, total: u64) -> Vec<u64> {
    (0..horizon).map(|i| s.target(i, horizon, total, 10 * (i as u64 + 1))).collect()
}

#[test]
fn every_schedule_is_monotone_and_completes() {
    let schedules: Vec<Box<dyn Schedule>> =
        vec![Box::new(Twap), Box::new(Immediate), Box::new(FrontLoaded { urgency: 3.0 }), Box::new(Pov { rate: 0.1 }), Box::new(Vwap::u_shape(10, 2.0))];
    for s in &schedules {
        let p = path(s.as_ref(), 50, 333);
        assert!(p.windows(2).all(|w| w[0] <= w[1]), "{} is not monotone", s.name());
        assert_eq!(*p.last().unwrap(), 333, "{} does not complete", s.name());
    }
}

#[test]
fn twap_is_linear_and_urgency_interpolates() {
    assert_eq!(path(&Twap, 4, 100), vec![25, 50, 75, 100]);
    assert_eq!(path(&FrontLoaded { urgency: 0.0 }, 4, 100), vec![25, 50, 75, 100]);
    let (slow, fast) = (path(&FrontLoaded { urgency: 1.0 }, 10, 1000), path(&FrontLoaded { urgency: 5.0 }, 10, 1000));
    assert!(slow[0] > 100 && fast[0] > slow[0] && fast[2] > 700);
    assert!(path(&FrontLoaded { urgency: 50.0 }, 10, 1000)[0] > 990);
}

#[test]
fn pov_tracks_volume_and_vwap_follows_its_profile() {
    let p = Pov { rate: 0.25 };
    assert_eq!((p.target(0, 10, 100, 40), p.target(3, 10, 100, 1000), p.target(9, 10, 100, 0)), (10, 100, 100));
    let v = Vwap::new(&[3.0, 1.0]);
    assert!((v.cumulative(0.5) - 0.75).abs() < 1e-12 && (v.cumulative(0.25) - 0.375).abs() < 1e-12 && v.cumulative(1.0) == 1.0);
    let u = Vwap::u_shape(20, 3.0);
    assert!(u.cumulative(0.1) > 0.1 && (u.cumulative(0.5) - 0.5).abs() < 1e-9);
}

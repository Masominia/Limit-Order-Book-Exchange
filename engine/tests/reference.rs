use lobsim::book::naive::NaiveBook;
use lobsim::math::rng::Rng;
use lobsim::{Book, Side};

/// The fast book and the list-scanning book must produce the same fills for any order flow.
#[test]
fn random_flow_matches_the_reference_book() {
    for seed in 0..20 {
        let mut rng = Rng::new(seed);
        let (mut fast, mut slow) = (Book::new(), NaiveBook::new());
        let mut ids = Vec::new();
        for _ in 0..3000 {
            let (mut a, mut b) = (Vec::new(), Vec::new());
            let side = if rng.chance(0.5) { Side::Buy } else { Side::Sell };
            let qty = 1 + rng.below(15);
            match rng.below(10) {
                0..=5 => {
                    let price = 500 + rng.below(30) as i64 - 15 - 4 * side.sign();
                    let x = fast.limit(1, side, price, qty, &mut a);
                    assert_eq!(x, slow.limit(1, side, price, qty, &mut b));
                    if x.1 > 0 {
                        ids.push(x.0);
                    }
                }
                6..=7 => assert_eq!(fast.market(2, side, qty, &mut a), slow.market(2, side, qty, &mut b)),
                _ => {
                    if !ids.is_empty() {
                        let id = ids.swap_remove(rng.below(ids.len() as u64) as usize);
                        assert_eq!(fast.cancel(id), slow.cancel(id));
                    }
                }
            }
            assert_eq!(a, b);
            assert_eq!(fast.volume(Side::Buy), slow.volume(Side::Buy));
            fast.check().unwrap();
        }
    }
}

#[test]
fn quantity_is_conserved() {
    let mut rng = Rng::new(3);
    let mut book = Book::new();
    let (mut submitted, mut traded, mut cancelled, mut dropped) = (0u64, 0u64, 0u64, 0u64);
    let mut ids = Vec::new();
    for _ in 0..5000 {
        let mut t = Vec::new();
        let side = if rng.chance(0.5) { Side::Buy } else { Side::Sell };
        let qty = 1 + rng.below(9);
        if rng.chance(0.7) {
            submitted += qty;
            let (id, rest) = book.limit(0, side, 100 + rng.below(10) as i64 - 5, qty, &mut t);
            if rest > 0 {
                ids.push(id);
            }
        } else if rng.chance(0.5) {
            submitted += qty;
            dropped += qty - book.market(0, side, qty, &mut t);
        } else if let Some(id) = ids.pop() {
            cancelled += book.cancel(id).unwrap_or(0);
        }
        traded += t.iter().map(|x| x.qty).sum::<u64>();
    }
    // every traded lot removes one lot from the taker and one from the maker
    assert_eq!(submitted, 2 * traded + cancelled + dropped + book.volume(Side::Buy) + book.volume(Side::Sell));
}

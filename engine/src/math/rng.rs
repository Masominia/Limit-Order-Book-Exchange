//! xoshiro256++ seeded through splitmix64.

pub struct Rng {
    s: [u64; 4],
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        let mut z = seed;
        let mut s = [0u64; 4];
        for v in &mut s {
            z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut x = z;
            x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            *v = x ^ (x >> 31);
        }
        Rng { s }
    }

    pub fn next_u64(&mut self) -> u64 {
        let result = self.s[0].wrapping_add(self.s[3]).rotate_left(23).wrapping_add(self.s[0]);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }

    /// Uniform on the open interval (0, 1).
    pub fn uniform(&mut self) -> f64 {
        ((self.next_u64() >> 11) as f64 + 0.5) / 9_007_199_254_740_992.0
    }

    /// Uniform integer in 0..n.
    pub fn below(&mut self, n: u64) -> u64 {
        ((self.uniform() * n as f64) as u64).min(n.saturating_sub(1))
    }

    pub fn chance(&mut self, p: f64) -> bool {
        self.uniform() < p
    }

    /// Number of events in one step for a rate that may exceed one: the integer part plus a coin for the rest.
    pub fn count(&mut self, rate: f64) -> u64 {
        let whole = rate.floor();
        whole as u64 + u64::from(self.chance(rate - whole))
    }

    pub fn normal(&mut self) -> f64 {
        let (u, v) = (self.uniform(), self.uniform());
        (-2.0 * u.ln()).sqrt() * (2.0 * std::f64::consts::PI * v).cos()
    }

    pub fn exponential(&mut self, mean: f64) -> f64 {
        -mean * self.uniform().ln()
    }
}

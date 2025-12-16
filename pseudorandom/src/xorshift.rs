use crate::PRNG;

pub struct XorShift32(u32);

impl PRNG for XorShift32 {
    type Item = u32;

    /// # Panics
    /// if `seed == 0`
    fn new(seed: Self::Item) -> Self {
        if seed == 0 {
            panic!("Seed cannot be zero");
        }
        Self(seed & 0xFFFFFFFF)
    }

    fn next(&mut self) -> Self::Item {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x & 0xFFFFFFFF;
        self.0
    }
}

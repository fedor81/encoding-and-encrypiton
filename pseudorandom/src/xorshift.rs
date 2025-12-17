use anyhow::Result;

use crate::PRNG;

/// Xorshift — класс генераторов псевдослучайных чисел, открытых Джорджем Марсалья.
/// Генераторы такого типа представляют собой подмножество регистров сдвига с линейной
/// обратной связью, что позволяет эффективно реализовать их без чрезмерного использования
/// разреженных многочленов.
pub struct XorShift32(u32);

impl XorShift32 {
    /// # Panics
    /// if `seed == 0`
    pub fn build(seed: u32) -> Result<Self> {
        anyhow::ensure!(
            seed != 0,
            "Seed cannot be zero. Please provide a non-zero seed for the XorShift32 generator."
        );
        Ok(Self(seed & 0xFFFFFFFF))
    }
}

impl PRNG for XorShift32 {
    type Item = u32;

    fn next(&mut self) -> Self::Item {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x & 0xFFFFFFFF;
        self.0
    }
}

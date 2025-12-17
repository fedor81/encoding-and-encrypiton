use anyhow::Result;

use crate::PRNG;

/// LCG (Linear Congruential Generator) — один из самых простых в реализации генераторов
/// псевдослучайных чисел.
///
/// # Основная формула LCG
///
/// `X = (a * X + c) mod m`
///
/// `X0`​ — начальное значение (seed),
/// `a` — множитель,
/// `c` — приращение,
/// `m` — модуль (обычно степень двойки, например 232232).
pub struct LCG {
    m: u32,
    a: u32,
    c: u32,
    x: u32,
}

impl PRNG for LCG {
    type Item = u32;

    fn next(&mut self) -> Self::Item {
        self.x = (self.a.wrapping_mul(self.x).wrapping_add(self.c)) % self.m;
        self.x
    }
}

impl LCG {
    pub fn build(seed: u32) -> Result<Self> {
        let m = 2u32.pow(31);
        let a = 1103515245;
        let c = 12345;
        Self::custom(m, a, c, seed)
    }

    pub fn custom(m: u32, a: u32, c: u32, seed: u32) -> Result<Self> {
        anyhow::ensure!(
            seed != 0,
            "Seed cannot be zero. Please provide a non-zero seed for the LCG generator."
        );
        Ok(Self { m, a, c, x: seed })
    }
}

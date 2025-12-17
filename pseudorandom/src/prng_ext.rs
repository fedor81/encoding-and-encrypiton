use crate::PRNG;

pub trait PRNGExt {
    fn next_f64(&mut self) -> f64;
}

impl<T> PRNGExt for T
where
    T: PRNG<Item = u32>,
{
    /// Генерирует u32 и преобразует в [0, 1)
    fn next_f64(&mut self) -> f64 {
        self.next() as f64 / (u32::MAX as f64 + 1.0)
    }
}

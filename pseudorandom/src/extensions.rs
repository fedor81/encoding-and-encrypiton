use crate::PRNG;

pub trait F64Ext {
    fn next_f64(&mut self) -> f64;
}

impl<T> F64Ext for T
where
    T: PRNG<Item = u32>,
{
    /// Генерирует u32 и преобразует в [0, 1)
    fn next_f64(&mut self) -> f64 {
        self.next() as f64 / (u32::MAX as f64 + 1.0)
    }
}

impl F64Ext for Box<dyn PRNG<Item = u32>> {
    fn next_f64(&mut self) -> f64 {
        self.next() as f64 / (u32::MAX as f64 + 1.0)
    }
}

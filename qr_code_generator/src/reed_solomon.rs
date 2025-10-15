use anyhow::{Context, Result};

use crate::{Coder, Poly, RefPoly, gf::GF256};

/// k – число информационных символов, подлежащих кодированию,
/// n – число кодовых символов в кодируемом блоке.
/// t – количество ошибочных символов, которые может исправить код.
/// n–k = 2t – число контрольных символов.
/// Минимальное расстояние определяется следующим образом: dmin = n–k+1.
pub struct ReedSolomon<T>
where
    T: GF256,
{
    control_count: usize,
    gf: T,
    gen_poly: Poly,
}

impl<T> ReedSolomon<T>
where
    T: GF256,
{
    pub fn new(control_count: usize, gf: T) -> Self {
        Self {
            control_count,
            gen_poly: Self::build_gen_poly(&gf, control_count),
            gf,
        }
    }

    /// Конструирует порождающий многочлен следующим образом:
    ///
    /// `g(x) = (x + a^1)(x + a^2)...(x + a^(d-1))`
    ///
    /// где `a` - примитивный элемент, `d = n - k + 1` - расстояние Хэмминга.
    fn build_gen_poly(gf: &T, control_count: usize) -> Poly {
        let mut gen_poly = vec![1];

        // Умножаем на (x + α^i)
        // По правилу a * (b + c) = a * b + a * c
        for i in 0..control_count {
            // Сперва умножаем на x, сдвигая коэффициенты
            let shifted_poly = gf.shift_poly(&gen_poly, 1);

            // Затем умножаем на α^i
            let alpha_i = gf.pow(2, i as u8);
            gen_poly = gf.mul_poly(&gen_poly, &vec![alpha_i]);

            // Складываем с результатом умножения
            gen_poly = gf.add_poly(&gen_poly, &shifted_poly);
        }

        gen_poly
    }
}

impl<T> Coder for ReedSolomon<T>
where
    T: GF256,
{
    fn encode(&self, data: RefPoly) -> Result<Poly> {
        if data.len() + self.control_count > 255 {
            anyhow::bail!("Message too long and cannot be encoded with GF256");
        }
        todo!()
    }

    fn decode(&self, data: RefPoly) -> Result<Poly> {
        if data.len() > 255 {
            anyhow::bail!("Message too long and cannot be decoded with GF256");
        }
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod build_gen_poly;
}

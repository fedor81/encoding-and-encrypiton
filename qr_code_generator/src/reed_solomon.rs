use anyhow::{Context, Result};

use crate::{
    Coder, Poly, RefPoly,
    gf::{GF256, GF256Poly},
};

/// k – число информационных символов, подлежащих кодированию,
/// n – число кодовых символов в кодируемом блоке.
/// t – количество ошибочных символов, которые может исправить код.
/// n–k = 2t – число контрольных символов.
/// Минимальное расстояние определяется следующим образом: dmin = n–k+1.
pub struct ReedSolomon<T>
where
    T: GF256Poly,
{
    control_count: usize,
    gf: T,
    gen_poly: Poly,
}

impl<T> ReedSolomon<T>
where
    T: GF256Poly,
{
    /// # Panics
    /// Panics if `control_count` is greater than 255.
    pub fn new(control_count: usize, gf: T) -> Self {
        if control_count > 255 {
            panic!(
                "The number of control characters cannot exceed 255, actual: {}",
                control_count
            );
        }

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
    T: GF256Poly,
{
    fn encode(&self, data: RefPoly) -> Result<Poly> {
        if data.len() + self.control_count > 255 {
            anyhow::bail!("Message too long and cannot be encoded with GF256");
        }

        // Полином сдвигается на n-k позиций для контрольных символов
        let mut message = vec![0; self.control_count];
        message.extend_from_slice(data);

        for (i, &n) in self
            .gf
            .mod_poly(&message, &self.gen_poly) // Вычисляем остаток от деления
            .iter()
            .enumerate()
        {
            message[i] = n;
        }

        Ok(message)
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
    mod encode;
}

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

    /// Вычисляет синдромы для многочлена `data`.
    /// Коэффициенты синдрома ошибки получаются подстановкой степеней примитивного члена в остаток
    /// многочлен `e(x) = C(x) mod g(x)`, или в сам многочлен сообщения `C(x)`.
    ///
    /// Нетрудно убедиться, что если бы сообщение не было искажено, то все коэффициенты Si оказались
    /// бы равны нулю: ведь неискажённое сообщение `C(x)` кратно порождающему многочлену `g(x)`,
    /// для которого числа `a1 , a2, ..., aN-K` являются корнями.
    fn calculate_syndromes(&self, data: RefPoly) -> Poly {
        let mut syndromes = vec![0u8; self.control_count];

        for i in 0..self.control_count {
            syndromes[i] = self.gf.eval_poly(data, self.gf.pow(2, i as u8))
        }

        syndromes
    }

    /// Локаторы ошибок – это элементы поля Галуа, степень которых совпадает с позицией
    /// ошибки. Так, если искажён коэффициент при x4, то локатор этой ошибки равен a4, если
    /// искажён коэффициент при x7 то локатор ошибки будет равен a7 и т.п. (а – примитивный член,
    /// т.е. в нашем случае a=2).
    ///
    /// Многочлен локаторов L(x) – это многочлен, корни которого обратны локаторам ошибок.
    /// Таким образом, многочлен L(x) должен иметь вид `L(x) = (1+xX1)(1+xX2)…(1+xXi)`,
    /// где `X1, X2, Xi` – локаторы ошибок. (`1+xXi` обращается в ноль при `x=Xi-1 : XiXi-1 = 1, 1+1 =0`)
    fn find_error_locator(&self, syndromes: RefPoly) -> Result<Poly> {
        todo!()
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

    /// Декодирование основано на построении многочлена синдрома ошибки S(x) и отыскании
    /// соответствующего ему многочлена локаторов L(x).
    fn decode(&self, data: RefPoly) -> Result<Poly> {
        if data.len() > 255 {
            anyhow::bail!("Message too long and cannot be decoded with GF256");
        }

        // Если все синдромы равны нулю, то сообщение не повреждено
        let syndromes = self.calculate_syndromes(data);

        if syndromes.iter().all(|&s| s == 0) {
            return Ok(data[self.control_count..].to_vec());
        }

        let error_locator = self.find_error_locator(&syndromes)?;

        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::gf::FastGF256;

    use super::*;

    mod build_gen_poly;
    mod encode;
    mod syndromes;

    // Вспомогательная функция для создания кодера с заданным количеством контрольных символов
    fn create_encoder(control_count: usize) -> ReedSolomon<FastGF256> {
        ReedSolomon::new(control_count, FastGF256::new())
    }
}

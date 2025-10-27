use anyhow::{Context, Result, anyhow};

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
    /// `g(x) = (x + a^0)(x + a^1)...(x + a^(control_count-1))`
    ///
    /// где `a` - примитивный элемент.
    fn build_gen_poly(gf: &T, control_count: usize) -> Poly {
        let mut gen_poly = vec![1];

        // Умножаем на (x + α^i)
        // По правилу a * (b + c) = a * b + a * c
        for i in 0..control_count {
            // Сперва умножаем на x, сдвигая коэффициенты
            let shifted_poly = gf.shift_poly(&gen_poly, 1);

            // Затем умножаем на α^i
            let alpha_i = gf.alpha_pow(i as u8);
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
            let point = self.gf.alpha_pow(i as u8); // Используем α^i
            syndromes[i] = self.gf.eval_poly(data, point);
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
    /// где `X1, X2, Xi` – локаторы ошибок. (`1+xXi` обращается в ноль при `x=Xi^(-1) : Xi * Xi^(-1) = 1, 1+1 =0`)
    ///
    /// # Алгоритм Берлекэмпа-Месси
    ///
    fn find_error_locator(&self, syndromes: RefPoly) -> Result<Poly> {
        // C(x) - текущий полином локатора ошибок
        let mut locator = vec![1u8]; // C(x) = 1
        let mut old_locator = vec![1u8]; // B(x) — копия последнего C(x) на момент обновления L или предыдущий C(x)
        let mut locator_degree = 0; // текущая степень C(x)
        let mut m = 1; // номер итерации, прошедших с обновления L

        for n in 0..self.control_count {
            // Вычисляем расхождение d
            let mut discrepancy = syndromes[n];

            // d = Sn + C₁ * S{n-1} + C₂ * S{n-2} + ... + CL * S{n-L}
            for i in 1..=locator_degree {
                let product = self.gf.mul(locator[i], syndromes[n - i]);
                discrepancy = self.gf.add(discrepancy, product);
            }

            // Если d равно нулю, это значит C(x) и L на данный момент верны, достаточно инкрементировать m и продолжить итерации.
            if discrepancy == 0 {
                m += 1;
                continue;
            }

            // Если d ненулевое, алгоритм поправляет C(x) так, чтобы его обнулить d:
            // C(x) = C(x) - (d/b)·B(x)·x^m, где B(x) – предыдущий C(x), b - копия последнего расхождения d

            // Умножение на x^m — это, по сути, сдвиг коэффициентов B(x) на m

            // Если 2L ≤ n: кардинально меняем локатор
            if 2 * locator_degree <= n {
                // Сохраняем текущий локатор
                let temp = locator.clone();

                // Вычисляем новый локатор
                let scale = self.gf.div(discrepancy, old_locator[0]);
                let scaled_old = self.gf.scale_poly(&old_locator, scale);
                let shifted_scaled_old = self.gf.shift_poly(&scaled_old, m);

                locator = self.gf.add_poly(&locator, &shifted_scaled_old);

                // Обновляем степень и предыдущий локатор
                locator_degree = n + 1 - locator_degree;
                old_locator = temp;
                m = 1;

            // Если 2L > n: только корректируем текущий локатор
            } else {
                let scale = self.gf.div(discrepancy, old_locator[0]);
                let scaled_old = self.gf.scale_poly(&old_locator, scale);
                let shifted_scaled_old = self.gf.shift_poly(&scaled_old, m);

                locator = self.gf.add_poly(&locator, &shifted_scaled_old);
                m += 1;
            }
        }

        // Проверяем, что можем исправить найденное количество ошибок
        // Можем исправить максимум control_count / 2 ошибок
        if locator_degree > self.control_count / 2 {
            anyhow::bail!(
                "Failed to find error locator: too many errors to correct \
                (locator degree: {locator_degree}, control count: {}, max correctable: {}). \n\
                Syndromes:\t{syndromes:?} \n\
                Locator:\t{locator:?} \n\
                Old Locator:\t{old_locator:?}",
                self.control_count,
                self.control_count / 2
            );
        }

        Ok(locator)
    }

    /// Находит корни полинома локатора L(x) – они будут обратны к локаторам ошибок.
    ///
    /// `L(x) = (1+xX1)(1+xX2)…(1+xXi)`, где `X1, X2, Xi` – локаторы ошибок.
    /// (`1+xXi` обращается в ноль при `x=Xi^(-1) : Xi * Xi^(-1) = 1, 1+1 =0`)
    ///
    /// Для поиска корней L(х) на множестве локаторов позиций кодовых символов используется метод
    /// проб и ошибок, получивший название метод Ченя. Для всех ненулевых элементов a GF(2m),
    /// которые генерируются в порядке `1, a, а2,... a14` проверяется условие `L(a^(-1))=0`.
    fn find_error_positions(&self, error_locator: RefPoly, data_len: usize) -> Result<Vec<usize>> {
        let mut positions = Vec::new();
        let expected_errors = error_locator.len() - 1;

        // Проверяем все возможные позиции ошибок методом Чена
        // L(x) имеет корни в обратных значениях локаторов ошибок
        // Если L(α^(-i)) = 0, то ошибка в позиции i
        for i in 0..data_len {
            let alpha_i = self.gf.alpha_pow(i as u8); // α^i
            let alpha_inv = self.gf.inverse(alpha_i); // α^(-i)
            let value = self.gf.eval_poly(error_locator, alpha_inv);

            if value == 0 {
                positions.push(i);

                // Ранний выход если нашли все ожидаемые ошибки
                if positions.len() == expected_errors {
                    break;
                }
            }
        }

        if positions.len() > expected_errors {
            anyhow::bail!(
                "Found more roots ({}) than expected errors ({}). \n\
                Locator: {:?}, \n\
                Data length: {}, Found positions: {:?}",
                positions.len(),
                expected_errors,
                error_locator,
                data_len,
                positions
            );
        }

        Ok(positions)
    }

    /// Далее вычисляется `W(x) = L(x)*S(x)`, коэффициенты старшие чем N-k должны быть обнулены.
    ///
    /// Далее вычисляются значения ошибок по формуле `Yi = W( Xi^(-1) )/L'( Xi^(-1) )`.
    /// Таким образом, составляется полином ошибки. Его коэффициентами являются значения ошибок Yi
    /// стоящие в позициях, определяемых локаторами ошибок.
    fn find_error_magnitudes(
        &self,
        syndromes: RefPoly,
        locator: RefPoly,
        error_positions: &[usize],
        data_len: usize,
    ) -> Vec<u8> {
        // W(x) = L(x)*S(x), обрезаем до control_count
        let omega_full = self.gf.mul_poly(locator, syndromes);
        let omega = if omega_full.len() > self.control_count {
            omega_full[..self.control_count].to_vec()
        } else {
            omega_full
        };

        // Вычисляем производную локатора ошибок
        let locator_derivative = self.find_locator_derivative(&locator);

        let mut magnitudes = Vec::new();

        for &err_pos in error_positions.iter() {
            let x_inv = self.gf.inverse(self.gf.alpha_pow(err_pos as u8));

            let numerator = self.gf.eval_poly(&omega, x_inv);
            let denominator = self.gf.eval_poly(&locator_derivative, x_inv);

            let magnitude = self.gf.div(numerator, denominator);
            magnitudes.push(magnitude);
        }

        magnitudes
    }

    /// Вычисляет производную L'(x) следующим образом – для чётных степеней производная равна нулю,
    /// для нечётных - степени уменьшенной на 1: `(x^2)' = 0, (x^3)' = x^2`
    fn find_locator_derivative(&self, locator: RefPoly) -> Poly {
        let mut locator_derivative = vec![0; locator.len()];

        // Производная для x^0 = 0, поэтому начинаем с 1
        for i in 1..locator.len() {
            // Нечетная степень
            if i % 2 == 1 {
                locator_derivative[i - 1] = locator[i];
            }
        }

        // Убираем нулевые коэффициенты
        for i in (1..locator_derivative.len()).rev() {
            if locator_derivative[i] == 0 {
                locator_derivative.pop();
            } else {
                break;
            }
        }

        locator_derivative
    }

    /// Исправляет ошибки в сообщении. Ошибка на позиции err_pos[i] с magnitude[i] вычитается из сообщения.
    ///
    /// # Panics
    /// Паникует, если err_pos[i] >= message.len()
    fn correct_errors(
        &self,
        data: RefPoly,
        error_positions: &[usize],
        error_magnitudes: &[u8],
    ) -> Poly {
        let mut corrected = data.to_vec();

        for (&pos, &magnitude) in error_positions.iter().zip(error_magnitudes.iter()) {
            if pos < corrected.len() {
                corrected[pos] = self.gf.sub(corrected[pos], magnitude);
            } else {
                panic!(
                    "Error position out of bounds (pos: {}, len: {}) \n\
                    message: {:?} \n\
                    positions: {:?} \n\
                    magnitudes: {:?}",
                    pos,
                    corrected.len(),
                    data,
                    error_positions,
                    error_magnitudes
                );
            }
        }
        corrected
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
        let mut encoded = vec![0; self.control_count];
        encoded.extend_from_slice(data);

        // Вычисляем остаток от деления message на gen_poly
        let remainder = self.gf.mod_poly(&encoded, &self.gen_poly);

        for (i, &n) in remainder.iter().enumerate().take(self.control_count) {
            encoded[i] = n;
        }

        Ok(encoded)
    }

    /// # Шаги декодирования
    /// 1. Вычислить e(x) = C(x) mod g(x).
    /// 2. Если e(x) = 0 то выделить p(x) из C(x).
    /// 3. Иначе, вычислить полином синдрома Si = e(ai+1)
    /// 4. Вычислить L(x) с помощью Берлекэмпа-Месси
    /// 5. Получить корни L(x) – локаторы ошибок
    /// 6. Вычислить L’(x). L’i = Li+1 для чётных i и 0 для нечётных.
    /// 7. Вычислить W(x) = S(x)*L(x)
    /// 8. Получить значения ошибок Yi = W(Xi-1 )/L’(Xi-1 )
    /// 9. Сформировать многочлен ошибок E(X) на основе локаторов и значений ошибок и
    /// скорректировать C(x) = C(x) + E(x).
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
        let error_positions = self.find_error_positions(&error_locator, data.len())?;
        let error_magnitudes =
            self.find_error_magnitudes(&syndromes, &error_locator, &error_positions, data.len());

        // Исправляем ошибки
        let corrected = self.correct_errors(data, &error_positions, &error_magnitudes);

        // Проверяем синдромы после исправления
        let syndromes_after = self.calculate_syndromes(&corrected);
        if syndromes_after.iter().any(|&s| s != 0) {
            anyhow::bail!(
                "Could not correct all errors. \n\
                Original data:\t {data:?}, \n\
                Corrected data:\t {corrected:?}, \n\
                Error locator: {error_locator:?}, \n\
                Error positions: {error_positions:?}, \n\
                Error magnitudes: {error_magnitudes:?}, \n\
                Syndromes before:\t {syndromes:?}, \n\
                Syndromes after:\t {syndromes_after:?}",
            );
        }

        Ok(corrected[self.control_count..].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use crate::gf::FastGF256;

    use super::*;

    mod build_gen_poly;
    mod decode;
    mod encode;
    mod locator;
    mod syndromes;
    mod utils;

    pub use utils::{StressTestConfig, create_encoder, stress_test_common};
}

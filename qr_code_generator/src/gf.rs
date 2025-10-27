//! Примитивный элемент поля GF(256) — это элемент, который порождает мультипликативную группу поля
//! GF(256), то есть при возведении его в степени от 1 до 255 (исключая 0) можно получить все ненулевые
//! элементы этого поля.

use crate::{Poly, RefPoly};

mod fast_gf256;
mod simple_gf256;

pub use fast_gf256::FastGF256;
pub use simple_gf256::SimpleGF256;

/// Примитивный полином: x⁸ + x⁴ + x³ + x² + 1 = 0x11D или 285 в десятичной.
pub const PRIMITIVE_POLY_FULL: u16 = 0x11D;

/// Возьмем младшие степени примитивного полинома: x⁴ + x³ + x² + 1 = 0x1D или 29 в десятичной.
pub const PRIMITIVE_POLY: u8 = 0x1D;

/// Определяет арифметические операции над элементами поля GF(256).
pub trait GF256 {
    fn _div(&self, a: u8, b: u8) -> u8;
    fn _mul(&self, a: u8, b: u8) -> u8;
    fn _pow(&self, a: u8, n: u8) -> u8;
    fn _inverse(&self, a: u8) -> u8;

    /// Примерный элемент поля.
    fn alpha() -> u8 {
        2
    }

    /// Возвести в степень примитивный элемент.
    fn alpha_pow(&self, n: u8) -> u8 {
        self.pow(Self::alpha(), n)
    }

    /// Увеличить число на 1.
    fn inc(&self, a: u8) -> u8 {
        self.add(a, 1)
    }

    /// Уменьшить число на 1.
    fn dec(&self, a: u8) -> u8 {
        self.sub(a, 1)
    }

    /// Возвести число a в степень n.
    fn pow(&self, a: u8, n: u8) -> u8 {
        if n == 0 {
            return 1;
        }
        if a == 0 {
            return 0;
        }
        Self::_pow(&self, a, n)
    }

    /// Найти a^(-1) в поле GF(256).
    fn inverse(&self, a: u8) -> u8 {
        if a == 0 {
            panic!("Zero has no inverse");
        }
        Self::_inverse(&self, a)
    }

    fn mul(&self, a: u8, b: u8) -> u8 {
        if a == 0 || b == 0 {
            return 0;
        }
        Self::_mul(&self, a, b)
    }

    fn div(&self, a: u8, b: u8) -> u8 {
        if b == 0 {
            panic!("Division by zero");
        }
        if a == 0 {
            return 0;
        }
        Self::_div(&self, a, b)
    }

    fn add(&self, a: u8, b: u8) -> u8 {
        a ^ b
    }

    fn sub(&self, a: u8, b: u8) -> u8 {
        a ^ b
    }
}

impl<T> GF256Poly for T where T: GF256 {}

/// Определяет операции над полиномами в поле GF(256).
pub trait GF256Poly: GF256 {
    /// Складывает многочлены с учетом правил сложения GF256
    fn add_poly(&self, a: RefPoly, b: RefPoly) -> Poly {
        let len = a.len().max(b.len());
        let mut result = vec![0u8; len];

        for i in 0..len {
            let a_val = a.get(i).copied().unwrap_or_default();
            let b_val = b.get(i).copied().unwrap_or_default();
            result[i] = self.add(a_val, b_val);
        }

        result
    }

    /// Умножает многочлены с учетом правил GF256
    fn mul_poly(&self, a: RefPoly, b: RefPoly) -> Poly {
        let mut result = vec![0u8; a.len() + b.len() - 1];

        for (i, &coef_a) in a.iter().enumerate() {
            for (j, &coef_b) in b.iter().enumerate() {
                let product = self.mul(coef_a, coef_b);
                result[i + j] = self.add(result[i + j], product);
            }
        }

        result
    }

    /// Функция для вычисления значения полинома в точке
    fn eval_poly(&self, poly: RefPoly, x: u8) -> u8 {
        let mut result = 0u8;

        for (i, &coef) in poly.iter().enumerate() {
            let term = self.mul(coef, self.pow(x, i as u8));
            result = self.add(result, term);
        }

        result
    }

    /// Умножает коэффициенты многочлена на скаляр
    fn scale_poly(&self, poly: RefPoly, scalar: u8) -> Poly {
        poly.iter().map(|&coef| self.mul(coef, scalar)).collect()
    }

    /// Сдвигает многочлен на n
    fn shift_poly(&self, poly: RefPoly, shift: usize) -> Poly {
        let mut result = vec![0u8; shift];
        result.extend_from_slice(poly);
        result
    }

    /// Вычисляет остаток от деления многочлена a на многочлен b
    fn mod_poly(&self, dividend: RefPoly, divisor: RefPoly) -> Poly {
        let (_quotient, remainder) = self._div_poly(dividend, divisor);
        remainder
    }

    /// Вычисляет частное от деления многочлена a на многочлен b
    fn div_poly(&self, dividend: RefPoly, divisor: RefPoly) -> Poly {
        let (quotient, _remainder) = self._div_poly(dividend, divisor);
        quotient
    }

    /// Вычисляет частное и остаток от деления
    fn _div_poly(&self, dividend: RefPoly, divisor: RefPoly) -> (Poly, Poly) {
        if divisor.is_empty() {
            panic!("Division by zero: {:?}", divisor);
        }

        let leader = divisor[divisor.len() - 1];
        if leader == 0 {
            panic!("Divisor has zero leading coefficient");
        }

        // Деление меньшего на больший
        if dividend.len() < divisor.len()
            || (dividend.len() == divisor.len()
                && dividend[dividend.len() - 1] < divisor[divisor.len() - 1])
        {
            return (vec![0], dividend.to_vec());
        }

        let mut quotient = vec![0u8; dividend.len() - divisor.len() + 1];
        let mut remainder = dividend.to_vec();
        let quotient_len = quotient.len();

        for i in 0..quotient.len() {
            if remainder[remainder.len() - i - 1] != 0 {
                // Вычисляем коэффициент частного
                let coef = self.div(remainder[remainder.len() - i - 1], leader);
                quotient[quotient_len - i - 1] = coef;

                // Вычитаем (коэффициент * делитель) из остатка
                for j in 0..divisor.len() {
                    if divisor[divisor.len() - j - 1] != 0 {
                        let curr_idx = remainder.len() - i - j - 1;

                        let product = self.mul(coef, divisor[divisor.len() - j - 1]);
                        remainder[curr_idx] = self.sub(remainder[curr_idx], product)
                    }
                }
            }
        }

        // Возвращаем реальный остаток
        let mut actual_remainder = remainder[..divisor.len() - 1].to_vec();
        
        // Если остаток пустой или все коэффициенты нулевые, возвращаем [0]
        if actual_remainder.is_empty() || actual_remainder.iter().all(|&x| x == 0) {
            actual_remainder = vec![0];
        }

        (quotient, actual_remainder)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub mod arithmetic_operations;
    pub mod poly_operations;
}

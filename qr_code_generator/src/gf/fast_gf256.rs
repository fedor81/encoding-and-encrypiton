use super::{GF256, PRIMITIVE_POLY, PRIMITIVE_POLY_FULL};

pub struct FastGF256 {
    exp_table: [u8; 256],
    log_table: [u8; 256],
}

impl FastGF256 {
    pub fn new() -> Self {
        let mut exp_table = [0u8; 256];
        let mut log_table = [0u8; 256];

        let mut x = 1;

        for i in 0..256 {
            exp_table[i] = x as u8;
            log_table[x as usize] = i as u8;

            x <<= 1;

            // 0x100 или 0b1_0000_0000 или 256
            if x >= 256 {
                x ^= PRIMITIVE_POLY_FULL;
            }
        }

        FastGF256 {
            exp_table,
            log_table,
        }
    }

    #[cfg(test)]
    pub fn pow_primitive_poly(&self, n: u8) -> u8 {
        self.exp_table[n as usize]
    }
}

impl GF256 for FastGF256 {
    fn _div(&self, a: u8, b: u8) -> u8 {
        let log_a = self.log_table[a as usize] as i16;
        let log_b = self.log_table[b as usize] as i16;
        let result = (log_a - log_b + 255) % 255;
        self.exp_table[result as usize]
    }

    /// Так как любой элемент представим в виде степени примитивного многочлена, если
    /// `a=x^n`, `b=x^m`, то `a*b=x^(n+m)` - произведение элементов можно представить в виде степени примитивного.
    /// - Подсчитаем таблицу степеней x т.е. `exp[i] = x^i`
    /// - Подсчитаем таблицу логарифмов x т.е. `x^log[a] = a`
    /// - Получаем `a * b = exp[log[a] + log[b] % 255]`
    fn _mul(&self, a: u8, b: u8) -> u8 {
        let log_a = self.log_table[a as usize] as usize;
        let log_b = self.log_table[b as usize] as usize;
        self.exp_table[(log_a + log_b) % 255]
    }

    fn _pow(&self, a: u8, n: u8) -> u8 {
        let log_a = self.log_table[a as usize] as usize;
        let result = (log_a * n as usize) % 255;
        self.exp_table[result]
    }

    fn _inverse(&self, a: u8) -> u8 {
        let log_a = self.log_table[a as usize] as usize;
        self.exp_table[255 - log_a]
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests as gf_tests;
    use super::FastGF256;

    #[test]
    fn test_gf256() {
        gf_tests::arithmetic_operations::test_gf256(FastGF256::new());
    }

    #[test]
    fn test_gf256_performance() {
        gf_tests::arithmetic_operations::test_gf256_performance(
            FastGF256::new(),
            std::time::Duration::from_secs(10),
        );
    }

    #[test]
    fn test_gf256_exceptions() {
        gf_tests::arithmetic_operations::test_gf256_exceptions(FastGF256::new());
    }

    #[test]
    fn test_poly_operations() {
        gf_tests::poly_operations::test_poly_operations(FastGF256::new());
    }
}

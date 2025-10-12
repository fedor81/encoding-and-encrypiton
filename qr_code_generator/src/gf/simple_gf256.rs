use super::{GF256, PRIMITIVE_POLY};

pub struct SimpleGF256 {}

impl GF256 for SimpleGF256 {
    fn _div(&self, a: u8, b: u8) -> u8 {
        let b_inverse = self.inverse(b);
        self.mul(a, b_inverse)
    }

    fn _mul(&self, a: u8, b: u8) -> u8 {
        let mut a = a;
        let mut b = b;
        let mut result = 0;

        while a > 0 {
            result ^= b * (a & 1);
            a >>= 1;
            // b << 1 соответствует домножению на многочлена на x
            // (b >> 7) соответветствует проверки, что b
            // многочлен степени 7.
            // ^ (irreducible_poly * (b >> 7)) соответствует
            // взятию по модулю в этом случае. В противном случаем
            // для взятия по модулю ничего не нужно делать
            b = (b << 1) ^ (PRIMITIVE_POLY * (b >> 7));
        }
        result
    }

    fn _pow(&self, a: u8, n: u8) -> u8 {
        let mut a = a;
        let mut result = 1;
        let mut pow = n as u8;

        while pow > 0 {
            if pow & 1 > 0 {
                result = self.mul(result, a);
            }
            a = self.mul(a, a);
            pow >>= 1;
        }
        result
    }

    /// Найти a^(-1) в поле GF(256) можно как a^(254).
    fn _inverse(&self, a: u8) -> u8 {
        self.pow(a, 254)
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests as gf_tests;
    use super::SimpleGF256;

    #[test]
    fn test_gf256() {
        gf_tests::test_gf256(SimpleGF256 {});
    }

    #[test]
    fn test_gf256_performance() {
        gf_tests::test_gf256_performance(SimpleGF256 {}, std::time::Duration::from_secs(70));
    }

    #[test]
    fn test_gf256_exceptions() {
        gf_tests::test_gf256_exceptions(SimpleGF256 {});
    }
}

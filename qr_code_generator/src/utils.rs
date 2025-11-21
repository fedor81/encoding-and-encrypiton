/// Преобразует последовательность байт в число, указанного типа.
pub fn convert_to_number<T>(bits: &[bool]) -> T
where
    T: Default + std::ops::ShlAssign + std::ops::AddAssign + From<u8>,
{
    let mut result = T::default();
    for &c in bits {
        result <<= 1.into();
        if c {
            result += T::from(1);
        }
    }
    result
}

/// Преобразует байты в двоичную строку.
pub fn convert_to_bits(bytes: &[u8]) -> Vec<bool> {
    let mut result = Vec::with_capacity(bytes.len() * 8);
    for byte in bytes {
        for bit in (0..8).rev() {
            if byte & (1 << bit) != 0 {
                result.push(true);
            } else {
                result.push(false);
            }
        }
    }
    result
}

/// Добавляет нули в конец последовательности бит, чтобы длина стала кратной 8-ми.
pub fn add_zeros(bits: &mut Vec<bool>) {
    match bits.len() % 8 {
        0 => {}
        n => {
            bits.extend(std::iter::repeat(false).take(8 - n));
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("0", 0)]
    #[case("1", 1)]
    #[case("101", 5)]
    #[case("11111111", 255)]
    #[case("10000000", 128)]
    fn test_convert_to_number(#[case] input: &str, #[case] expected: u8) {
        let bits = input
            .chars()
            .into_iter()
            .map(|ch| ch == '1')
            .collect::<Vec<_>>();
        assert_eq!(
            convert_to_number::<u8>(&bits),
            expected,
            "Failed for input: {:?}",
            bits
        );
    }

    #[test]
    fn test_convert_to_bits() {
        for i in 0..=255 {
            let expected = format!("{:08b}", i)
                .chars()
                .into_iter()
                .map(|c| c == '1')
                .collect::<Vec<_>>();
            let actual = convert_to_bits(&[i]);
            assert_eq!(actual, expected, "Failed for input: {}", i);
        }
    }

    #[rstest]
    #[case("111", "11100000")]
    #[case("11010000", "11010000")]
    #[case("11010000", "11010000")]
    #[case("1101001", "11010010")]
    fn test_add_zeros(#[case] input: &str, #[case] expected: &str) {
        let mut input = input.chars().map(|c| c == '1').collect::<Vec<_>>();
        let expected = expected.chars().map(|c| c == '1').collect::<Vec<_>>();
        add_zeros(&mut input);
        assert_eq!(input, expected);
    }
}

use anyhow::Result;

/// Преобразует последовательность байт в число, указанного типа.
pub fn bits_to_number<T>(bits: &[bool]) -> T
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

pub fn bits_to_bytes(bits: &[bool]) -> Result<Vec<u8>> {
    if bits.len() % 8 != 0 {
        anyhow::bail!("Invalid length, must be a multiple of 8, current: {}", bits.len())
    }
    Ok(bits
        .chunks(8)
        .map(|chunk| chunk.iter().fold(0u8, |acc, &bit| (acc << 1) | bit as u8))
        .collect())
}

/// Преобразует байты в двоичную строку.
pub fn bytes_to_bits(bytes: &[u8]) -> Vec<bool> {
    let mut result = Vec::with_capacity(bytes.len() * 8);
    for &byte in bytes {
        result.extend_from_slice(&byte_to_bits(byte));
    }
    result
}

pub fn byte_to_bits(byte: u8) -> Vec<bool> {
    let mut result = Vec::with_capacity(8);
    for bit in (0..8).rev() {
        if byte & (1 << bit) != 0 {
            result.push(true);
        } else {
            result.push(false);
        }
    }
    result
}

/// Переводит число в вектор бит, указанного размера. Первые если в числе бит меньше чем длина вектора,
/// то они принимаются нулями.
pub fn to_bit_array(value: u32, size: usize) -> Vec<bool> {
    let mut bits = vec![false; size];
    for i in 0..size {
        bits[size - i - 1] = (value >> i) & 1 == 1;
    }
    bits
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
    // use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("0", 0)]
    #[case("1", 1)]
    #[case("101", 5)]
    #[case("11111111", 255)]
    #[case("10000000", 128)]
    fn test_bits_to_number(#[case] input: &str, #[case] expected: u8) {
        let bits = input.chars().into_iter().map(|ch| ch == '1').collect::<Vec<_>>();
        assert_eq!(bits_to_number::<u8>(&bits), expected, "Failed for input: {:?}", bits);
    }

    #[test]
    fn test_bytes_to_bits() {
        for i in 0..=255 {
            let expected = format!("{:08b}", i)
                .chars()
                .into_iter()
                .map(|c| c == '1')
                .collect::<Vec<_>>();
            let actual = bytes_to_bits(&[i]);
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

    #[rstest]
    #[case("10000000_00000001", vec![128, 1])]
    #[case("00000000_00000000_00000000", vec![0, 0, 0])]
    #[case("00000001_00000000_00000001", vec![1, 0, 1])]
    fn test_bits_to_bytes(#[case] mut input: String, #[case] expected: Vec<u8>) {
        input = input.replace('_', "");
        let bits = input.chars().map(|c| c == '1').collect::<Vec<_>>();
        assert!(bits.len() % 8 == 0, "The input sequence must be a multiple of 8.");
        let actual = bits_to_bytes(&bits).unwrap();
        assert_eq!(actual, expected, "Input: {input}")
    }

    #[test]
    fn test_bytes_to_bits_and_back() {
        for i in 0..100 {
            let length = rand::random_range(10..100);
            let input = rand::random_iter().take(length).collect::<Vec<_>>();

            let bits = bytes_to_bits(&input);
            let bytes = bits_to_bytes(&bits).unwrap();

            assert_eq!(bytes, input, "Iteration: {i}");
        }
    }
}

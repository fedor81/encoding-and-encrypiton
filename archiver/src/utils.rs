use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

use super::FrequencyMap;

pub fn sort_words_and_probabilities(
    words: Vec<u8>,
    probabilities: Vec<f64>,
) -> (Vec<u8>, Vec<f64>) {
    let mut word_probability = words
        .into_iter()
        .zip(probabilities.into_iter())
        .collect::<Vec<_>>();

    word_probability.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap().reverse());
    word_probability.into_iter().unzip()
}

pub fn create_probabilities_map(path: &PathBuf) -> Result<HashMap<u8, f64>> {
    // Проверяем существование файла перед открытием
    if !path.exists() {
        anyhow::bail!("File does not exist: {}", path.display());
    }

    let file =
        File::open(&path).with_context(|| format!("❌ Failed to open file: {}", path.display()))?;
    let mut reader = BufReader::new(file);

    // 1MB buffer
    let size = 1024 * 1024;
    let mut buf = vec![0u8; size];

    let mut freq_map = FrequencyMap::new();

    while let Ok(n) = reader.read(buf.as_mut_slice()) {
        if n == 0 {
            break;
        }
        freq_map.consume(&buf[..n]);
    }

    Ok(freq_map.build())
}

/// Преобразует двоичную строку в число типа T.
///
/// # Параметры
/// - `s`: Двоичная строка (только символы '0' и '1').
///
/// # Возвращает
/// - Число типа `T`, соответствующее двоичной строке.
pub fn convert_to_bytes<T>(s: &str) -> T
where
    T: Default + std::ops::ShlAssign + std::ops::AddAssign + From<u8>,
{
    let mut result = T::default();
    for c in s.chars() {
        result <<= 1.into();
        if c == '1' {
            result += T::from(1);
        }
    }
    result
}

pub fn convert_to_string(bytes: &[u8]) -> String {
    let mut result = String::new();
    for byte in bytes {
        for bit in (0..8).rev() {
            if byte & (1 << bit) != 0 {
                result.push('1');
            } else {
                result.push('0');
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_to_bytes() {
        let tests = [
            ("0", 0u8),
            ("1", 1u8),
            ("101", 5u8),
            ("11111111", 255u8),
            ("10000000", 128u8),
        ];

        for (input, expected) in tests.iter() {
            assert_eq!(
                convert_to_bytes::<u8>(input),
                *expected,
                "Failed for input: {}",
                input
            );
        }
    }

    #[test]
    fn test_convert_to_string() {
        for i in 0..=255 {
            let expected = format!("{:08b}", i);
            let actual = convert_to_string(&[i]);
            assert_eq!(actual, expected, "Failed for input: {}", i);
        }
    }
}

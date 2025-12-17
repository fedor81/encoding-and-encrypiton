use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
};

use super::FrequencyMap;

pub fn sort_words_and_probabilities(words: Vec<u8>, probabilities: Vec<f64>) -> (Vec<u8>, Vec<f64>) {
    let mut word_probability = words.into_iter().zip(probabilities.into_iter()).collect::<Vec<_>>();

    word_probability.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap().reverse());
    word_probability.into_iter().unzip()
}

pub fn create_probabilities_map(path: &PathBuf) -> Result<HashMap<u8, f64>> {
    // Проверяем существование файла перед открытием
    if !path.exists() {
        anyhow::bail!("File does not exist: {}", path.display());
    }

    let file = File::open(&path).with_context(|| format!("❌ Failed to open file: {}", path.display()))?;
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

pub fn cmp_files_text<P: AsRef<Path>>(actual_path: P, expected_path: P) {
    let actual_path = actual_path.as_ref();
    let expected_path = expected_path.as_ref();

    // Открытие файлов
    let actual_file =
        File::open(actual_path).unwrap_or_else(|_| panic!("Failed to open actual file: {}", actual_path.display()));
    let expected_file = File::open(expected_path)
        .unwrap_or_else(|_| panic!("Failed to open expected file: {}", expected_path.display()));

    let actual_reader = BufReader::new(actual_file);
    let expected_reader = BufReader::new(expected_file);

    for (line_num, (actual_line, expected_line)) in actual_reader.lines().zip(expected_reader.lines()).enumerate() {
        let line_num = line_num + 1; // Нумерация строк с 1

        let actual = actual_line.unwrap_or_else(|err| {
            panic!(
                "Failed to read line {}, error: {}, from actual file: {}\n",
                line_num,
                err,
                actual_path.display()
            )
        });
        let expected = expected_line.unwrap_or_else(|err| {
            panic!(
                "- Failed to read line {}, error: {}, from expected file: {}\n",
                line_num,
                err,
                expected_path.display(),
            )
        });

        if actual != expected {
            panic!(
                "Files differ at line {}:\n\
                 File: {}\n\
                 Expected: {:?}\n\
                 Actual:   {:?}\n",
                line_num,
                actual_path.display(),
                expected,
                actual
            );
        }
    }

    // Проверяем что в одном файле не осталось лишних строк
    let actual_file = File::open(actual_path).expect("Failed to reopen actual file");
    let expected_file = File::open(expected_path).expect("Failed to reopen expected file");

    let actual_lines: Vec<_> = BufReader::new(actual_file).lines().collect();
    let expected_lines: Vec<_> = BufReader::new(expected_file).lines().collect();

    assert_eq!(
        actual_lines.len(),
        expected_lines.len(),
        "Files have different number of lines:\n  actual: {} lines\n  expected: {} lines",
        actual_lines.len(),
        expected_lines.len(),
    );

    println!(
        "✅ Text files are identical: {} == {}",
        actual_path.display(),
        expected_path.display()
    );
}

pub fn cmp_files<P: AsRef<Path>>(actual_path: P, expected_path: P) {
    let actual_path = actual_path.as_ref();
    let expected_path = expected_path.as_ref();

    // Проверка существования файлов
    if !actual_path.exists() {
        panic!("Actual file does not exist: {}", actual_path.display());
    }
    if !expected_path.exists() {
        panic!("Expected file does not exist: {}", expected_path.display());
    }

    // Сравнение метаданных
    let actual_metadata = fs::metadata(actual_path)
        .unwrap_or_else(|_| panic!("Failed to get metadata for actual file: {}", actual_path.display()));
    let expected_metadata = fs::metadata(expected_path)
        .unwrap_or_else(|_| panic!("Failed to get metadata for expected file: {}", expected_path.display()));

    assert_eq!(
        actual_metadata.len(),
        expected_metadata.len(),
        "File sizes differ:\n  actual: {} bytes ({})\n  expected: {} bytes ({})",
        actual_metadata.len(),
        actual_path.display(),
        expected_metadata.len(),
        expected_path.display()
    );

    // Открытие файлов
    let mut actual_file =
        File::open(actual_path).unwrap_or_else(|_| panic!("Failed to open actual file: {}", actual_path.display()));
    let mut expected_file = File::open(expected_path)
        .unwrap_or_else(|_| panic!("Failed to open expected file: {}", expected_path.display()));

    // Сравнение побайтово (более надежно чем построчно)
    let mut actual_buf = Vec::new();
    let mut expected_buf = Vec::new();

    actual_file
        .read_to_end(&mut actual_buf)
        .unwrap_or_else(|_| panic!("Failed to read actual file: {}", actual_path.display()));
    expected_file
        .read_to_end(&mut expected_buf)
        .unwrap_or_else(|_| panic!("Failed to read expected file: {}", expected_path.display()));

    // Поиск первого отличающегося байта
    for (i, (actual_byte, expected_byte)) in actual_buf.iter().zip(expected_buf.iter()).enumerate() {
        if actual_byte != expected_byte {
            panic!(
                "Files differ at byte {}:\n  actual: 0x{:02x} ({})\n  expected: 0x{:02x} ({})\nFile: {}",
                i,
                actual_byte,
                if *actual_byte >= 32 && *actual_byte <= 126 {
                    format!("'{}'", *actual_byte as char)
                } else {
                    "non-printable".to_string()
                },
                expected_byte,
                if *expected_byte >= 32 && *expected_byte <= 126 {
                    format!("'{}'", *expected_byte as char)
                } else {
                    "non-printable".to_string()
                },
                actual_path.display()
            );
        }
    }

    // Если дошли до конца - файлы идентичны
    println!(
        "✅ Files are identical: {} == {}",
        actual_path.display(),
        expected_path.display()
    );
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
            assert_eq!(convert_to_bytes::<u8>(input), *expected, "Failed for input: {}", input);
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

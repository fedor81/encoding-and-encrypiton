use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

use reed_solomon::{BlockCoder, Coder, new_reed_solomon};

const MAX_BLOCK_SIZE: usize = 255;

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.controls == 0 || cli.controls >= MAX_BLOCK_SIZE {
        anyhow::bail!(
            "Количество контрольных символов должно быть от 1 до {}",
            MAX_BLOCK_SIZE - 1
        );
    }

    let rs = new_reed_solomon(cli.controls);

    match cli.command {
        Command::Encode {
            input,
            input_format,
            output_format,
        } => {
            let data = input_format.parse_input_data(&input)?;
            let block_size = MAX_BLOCK_SIZE - cli.controls;

            let encoded = rs.encode_blocks_to_vec(&data, block_size)?;

            println!("{}", output_format.parse(&encoded)?);
        }
        Command::Decode {
            input,
            input_format: r#type,
            output_format,
        } => {
            let data = r#type.parse_input_data(&input)?;
            let block_size = MAX_BLOCK_SIZE; // полный блок: данные + контрольные

            check_len(&data, block_size);

            let decoded = rs.decode_blocks_to_vec(&data, block_size)?;
            let output = output_format.parse(&decoded)?;

            println!("{}", output);
        }
    };
    Ok(())
}

#[derive(Parser)]
#[command(version, about = "Кодер/декодер Рида-Соломона", long_about = None, author = "laroxyss")]
struct Cli {
    /// Исполняемая команда
    #[command(subcommand)]
    command: Command,

    /// Количество контрольных символов
    #[arg(short, long, default_value = "10")]
    controls: usize,
}

#[derive(Subcommand)]
enum Command {
    /// Закодировать данные
    Encode {
        /// Входные данные
        input: String,

        /// Тип входных данных
        #[arg(long, default_value = "auto")]
        input_format: DataFormat,

        /// Тип выходных данных
        #[arg(long, default_value = "hex")]
        output_format: OutputFormat,
    },

    /// Декодировать данные
    Decode {
        /// Входные данные
        input: String,

        /// Тип входных данных
        #[arg(long, default_value = "auto")]
        input_format: DataFormat,

        /// Тип выходных данных
        #[arg(long, default_value = "auto")]
        output_format: DataFormat,
    },
}

/// Выходной формат — только для бинарных данных
#[derive(Debug, Clone, clap::ValueEnum)]
enum OutputFormat {
    /// Шестнадцатеричная строка без 0x (например: a1b2c3)
    Hex,

    /// Байты через пробел (например: 10 20 255)
    Bytes,
}

impl OutputFormat {
    fn parse(&self, data: &[u8]) -> Result<String> {
        match self {
            OutputFormat::Hex => Ok(DataFormat::bytes_to_hex(data)),
            OutputFormat::Bytes => Ok(DataFormat::bytes_to_string(data)),
        }
    }
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum DataFormat {
    /// Автоматическое определение формата
    Auto,

    /// Строка в формате UTF-8 (кодируется в байты)
    Text,

    /// Строка в формате hex (например: a1b2c3 или 0xa1b2c3)
    Hex,

    /// Строка байтов через пробел (например: 10 20 30)
    Bytes,
}

impl DataFormat {
    fn parse(&self, data: &[u8]) -> Result<String> {
        match self {
            DataFormat::Text => Self::bytes_to_text(data),
            DataFormat::Hex => Ok(Self::bytes_to_hex(&data)),
            DataFormat::Auto => {
                let text = Self::bytes_to_text(data);
                if text.is_ok() {
                    text
                } else {
                    Ok(Self::bytes_to_hex(data))
                }
            }
            DataFormat::Bytes => Ok(Self::bytes_to_string(data)),
        }
    }

    fn bytes_to_string(data: &[u8]) -> String {
        format!("{}", data.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" "))
    }

    /// Попытка интерпретировать как UTF-8
    fn bytes_to_text(data: &[u8]) -> Result<String> {
        if let Ok(utf8_str) = std::str::from_utf8(data) {
            // Проверим, что строка "печатающаяся" и не содержит мусора
            // (опционально: можно пропустить проверку и выводить всё)
            if utf8_str
                .chars()
                .all(|c| c.is_ascii_graphic() || c.is_whitespace() || c.is_ascii_control())
            {
                return Ok(utf8_str.to_string());
            }
        }
        anyhow::bail!("Не удалось декодировать как UTF-8: {:#?}", data)
    }

    fn bytes_to_hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join("")
    }

    fn parse_input_data(&self, input: &str) -> Result<Vec<u8>> {
        match self {
            DataFormat::Auto => Self::detect_and_parse(input),
            _ => Self::parse_with_format(input, self),
        }
    }

    fn from_hex(input: &str) -> Result<Vec<u8>> {
        let clean = if input.starts_with("0x") || input.starts_with("0X") {
            &input[2..]
        } else {
            input
        };
        if clean.is_empty() {
            return Ok(vec![]);
        }
        if clean.chars().any(|c| !c.is_ascii_hexdigit()) {
            anyhow::bail!("Некорректные символы в hex-строке");
        }
        if clean.len() % 2 != 0 {
            anyhow::bail!("Нечётная длина hex-строки");
        }
        hex::decode(clean).map_err(|e| anyhow::anyhow!("Ошибка парсинга hex: {}", e))
    }

    fn from_bytes(input: &str) -> Result<Vec<u8>> {
        if input.trim().is_empty() {
            return Ok(vec![]);
        }
        input
            .split_whitespace() // ← split по любому whitespace
            .map(|s| {
                s.parse::<u8>()
                    .map_err(|_| anyhow::anyhow!("Некорректное число: '{}'", s))
            })
            .collect()
    }

    fn parse_with_format(input: &str, format: &DataFormat) -> Result<Vec<u8>> {
        match format {
            DataFormat::Text => Ok(input.as_bytes().to_vec()),
            DataFormat::Hex => Self::from_hex(input),
            DataFormat::Bytes => Self::from_bytes(input),
            DataFormat::Auto => unreachable!(), // handled separately
        }
    }

    fn detect_and_parse(input: &str) -> Result<Vec<u8>> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Ok(vec![]);
        }

        // 1. Проверяем, похоже ли на hex:
        //    - только hex-символы (возможно с 0x)
        //    - чётная длина (после 0x)
        let clean_hex = if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
            &trimmed[2..]
        } else {
            trimmed
        };

        if !clean_hex.is_empty() && clean_hex.chars().all(|c| c.is_ascii_hexdigit()) && clean_hex.len() % 2 == 0 {
            return hex::decode(clean_hex).map_err(|e| anyhow::anyhow!("Ошибка парсинга как hex: {}", e));
        }

        // 2. Проверяем, похоже ли на список байтов (через пробелы)
        if trimmed.split_whitespace().count() > 1 {
            let all_numbers = trimmed.split_whitespace().all(|s| s.parse::<u8>().is_ok());
            if all_numbers {
                return Ok(trimmed
                    .split_whitespace()
                    .map(|s| s.parse::<u8>().unwrap()) // safe due to check above
                    .collect::<Vec<_>>());
            }
        }

        // 3. По умолчанию — текст (UTF-8)
        Ok(trimmed.as_bytes().to_vec())
    }
}

fn check_len(data: &[u8], block_size: usize) {
    if data.len() > block_size && data.len() % block_size != 0 {
        eprint!(
            "Длина входных данных ({}) не кратна размеру блока ({})",
            data.len(),
            block_size
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex() {
        for n in 0..100 {
            let rs = new_reed_solomon(n);
            let message = rand::random_iter().take(10).collect::<Vec<_>>();

            let encoded = rs.encode(&message).unwrap();
            let hex_encoded = DataFormat::bytes_to_hex(&encoded);

            let dehexed = DataFormat::Hex.parse_input_data(&hex_encoded).unwrap();
            let decoded = rs.decode(&dehexed).unwrap();

            assert_eq!(
                message, decoded,
                "Ошибка декодирования для n={n}:\n
                    hex_encoded:\t{hex_encoded:?}\n
                    dehexed:\t{dehexed:?}\n
                    decoded:\t{decoded:?}\n"
            );
        }
    }
}

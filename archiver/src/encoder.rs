use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use anyhow::{Context, Result};

use crate::StateSaver;

/// Интерфейс для кодирования последовательности байтов в строку. Не потоковый!
pub trait Encoder {
    /// Преобразует целевую последовательность байтов в закодированную строку.
    fn convert_to_string(&self, bytes: &[u8]) -> String;

    /// Кодирует последовательность байтов.
    fn encode_bytes(&self, bytes: &[u8]) -> Vec<u8> {
        let bit_string = self.convert_to_string(bytes);
        Self::convert_to_bytes(bit_string)
    }

    fn convert_to_bytes(mut bit_string: String) -> Vec<u8> {
        // Дополняем до полного байта
        let padding = (8 - (bit_string.len() % 8)) % 8;
        for _ in 0..padding {
            bit_string.push('0');
        }

        // Преобразуем битовую строку в байты
        let mut encoded = Vec::new();
        for chunk in bit_string.as_bytes().chunks(8) {
            let mut byte = 0u8;
            for &b in chunk.into_iter() {
                byte <<= 1;
                if b == b'1' {
                    byte |= 1u8;
                }
            }
            encoded.push(byte);
        }

        encoded
    }

    fn encode_file(&self, path: &PathBuf) -> Result<Vec<u8>> {
        let mut file = File::open(path).expect("Failed to open file");
        let size = file.metadata().context("Failed to extract file metadata")?.len();
        let mut buf = Vec::with_capacity(size as usize);
        file.read_to_end(&mut buf).expect("Failed to read file");

        Ok(self.encode_bytes(&buf))
    }
}

impl<T> FileEncoder for T where T: Encoder + StateSaver {}

pub trait FileEncoder
where
    Self: Encoder + StateSaver + Sized,
{
    fn encode_file(self, target: &PathBuf, destination: &PathBuf) -> Result<()> {
        // Размер исходного файла (в байтах) для корректного удаления паддинга при декодировании
        let original_size = std::fs::File::open(target)
            .and_then(|f| f.metadata())
            .map(|m| m.len() as usize)
            .context("Failed to extract file metadata")?;

        let encoded_file = Encoder::encode_file(&self, target);
        let state = self.save_state()?;
        let mut file = File::create(destination).context("Failed to create file")?;

        // Записываем размер состояния, состояние и сжатый файл
        Self::write_state(&state, &mut file)?;
        // Записываем размер исходного файла (usize)
        file.write_all(&original_size.to_le_bytes())
            .context("Failed to write original size")?;
        file.write_all(&encoded_file?).context("Failed to write encoded file")?;
        Ok(())
    }
}

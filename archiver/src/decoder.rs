use anyhow::{Context, Result};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use super::{StateSaver, utils::convert_to_string};

pub trait Decoder {
    /// Декодирует строку битов.
    fn decode_string(&self, bit_string: &str) -> Result<Vec<u8>>;

    fn decode_bytes(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        let bit_string = convert_to_string(bytes);
        self.decode_string(&bit_string)
    }
}

impl<T> FileDecoder for T where T: Decoder + StateSaver {}

pub trait FileDecoder
where
    Self: Decoder + StateSaver + Sized,
{
    fn decode_file<P: AsRef<Path>>(target: P, destination: P) -> Result<()> {
        let mut file = File::open(target).context("Failed to create file")?;

        // Восстанавливаем состояние кодека
        let state = Self::read_state(&mut file)?;

        // Читаем оригинальный размер файла (в байтах), записанный при кодировании
        let mut original_size_buf = [0u8; std::mem::size_of::<usize>()];
        file.read_exact(&mut original_size_buf)
            .context("Failed to read original size")?;
        let original_size = usize::from_le_bytes(original_size_buf);

        // Читаем оставшуюся закодированную часть до конца файла
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .context("Failed to read encoded part")?;

        // Декодируем файл
        let decoder = Self::load_state(state)?;
        decoder.decode_and_write(&bytes, destination, original_size)
    }

    fn decode_and_write<P: AsRef<Path>>(
        &self,
        bytes: &[u8],
        destination: P,
        original_size: usize,
    ) -> Result<()> {
        let mut decoded = self.decode_bytes(&bytes).context("Failed to decode")?;

        // Удаляем возможные лишние байты, появившиеся из-за паддинга при кодировании
        if decoded.len() > original_size {
            decoded.truncate(original_size);
        }

        // Записываем результат
        let mut target_file = File::create(destination).context("Failed to create file")?;
        target_file
            .write_all(&mut decoded)
            .context("Failed to write to file")?;

        Ok(())
    }
}

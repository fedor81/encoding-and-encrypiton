use anyhow::{Context, Result};
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
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

pub trait FileDecoder
where
    Self: Decoder + StateSaver + Sized,
{
    fn decode_file(target: &PathBuf, destination: &PathBuf) -> Result<()> {
        let mut file = File::open(target).context("Failed to create file")?;
        let file_size = match file.metadata() {
            Ok(metadata) => metadata.len() as usize,
            Err(_) => 0,
        };

        // Читаем размер состояния (usize)
        let mut state_size = [0; std::mem::size_of::<usize>()];
        file.read_exact(&mut state_size)
            .context("Failed to read state size")?;
        let state_size = usize::from_le_bytes(state_size);

        assert!(state_size > 0);

        // Читаем состояние
        let mut state = vec![0; state_size];
        file.read_exact(&mut state)
            .context("Failed to read state")?;

        // Читаем закодированную часть
        let mut bytes = Vec::with_capacity(file_size - state_size - std::mem::size_of::<usize>());
        file.read_exact(&mut bytes)
            .context("Failed to read encoded part")?;

        // Декодируем файл
        let decoder = Self::load_state(state);
        decoder.decode_and_write(&bytes, destination)
    }

    fn decode_and_write(&self, bytes: &[u8], destination: &PathBuf) -> Result<()> {
        let mut decoded = self.decode_bytes(&bytes).context("Failed to decode")?;

        // Записываем результат
        let mut target_file = File::create(destination).context("Failed to create file")?;
        target_file
            .write_all(&mut decoded)
            .context("Failed to write to file")?;

        Ok(())
    }
}

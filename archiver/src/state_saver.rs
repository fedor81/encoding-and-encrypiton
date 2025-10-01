use std::{
    fs::File,
    io::{Read, Write},
};

use anyhow::{Context, Result};

pub trait StateSaver
where
    Self: Sized,
{
    /// Сохраняет состояние объекта в вектор байтов.
    fn save_state(self) -> Result<Vec<u8>>;

    /// Загружает состояние объекта из вектора байтов.
    fn load_state(state: Vec<u8>) -> Result<Self>;

    fn write_state(state: &[u8], file: &mut File) -> Result<()> {
        file.write_all(state.len().to_le_bytes().as_slice())
            .context("Failed to write state length to file")?;
        file.write_all(state).context("Failed to write state")?;
        Ok(())
    }

    fn read_state(file: &mut File) -> Result<Vec<u8>> {
        // Читаем размер состояния (usize)
        let mut state_size = [0; std::mem::size_of::<usize>()];
        file.read_exact(&mut state_size)
            .context("Failed to read state size")?;
        let state_size = usize::from_le_bytes(state_size);

        if state_size == 0 {
            return Err(anyhow::anyhow!("Invalid state size: 0"));
        }

        // Читаем состояние
        let mut state = vec![0; state_size];
        file.read_exact(&mut state)
            .context("Failed to read state")?;
        Ok(state)
    }
}

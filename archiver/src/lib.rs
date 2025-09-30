use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    fmt::Debug,
    path::{self, Path},
};

mod codes;
mod decoder;
mod encoder;
mod freq_map;
mod huffman;
pub mod io;
mod shannon_fano;
mod state_saver;
pub mod utils;

pub use codes::Codes;
pub use decoder::{Decoder, FileDecoder};
pub use encoder::{Encoder, FileEncoder};
pub(crate) use freq_map::FrequencyMap;
pub use huffman::HuffmanArchiver;
pub use shannon_fano::ShannonFanoEncoder;
pub(crate) use state_saver::StateSaver;
pub use utils::create_probabilities_map;

pub trait CodesBuilder {
    /// Строит оптимальный код на основе вероятностей вхождений символов.
    /// Сумма `probabilities` должна быть равна `1`.
    /// Возвращает вектор строк, где каждый элемент - код символа.
    fn build_optimal_codes(words: Vec<u8>, probabilities: Vec<f64>) -> Codes;

    fn build_optimal_codes_from_hashmap(words_probabilities: HashMap<u8, f64>) -> Codes {
        let (keys, values): (Vec<_>, Vec<_>) = words_probabilities.into_iter().unzip();
        Self::build_optimal_codes(keys, values)
    }
}

pub trait FileArchiver
where
    Self: FileEncoder + FileDecoder,
{
}

impl<T> FileArchiver for T where T: FileEncoder + FileDecoder {}

/// Archives the file in the specified location.
pub fn archive_by_haffman<P>(target: P, destination: P) -> Result<()>
where
    P: AsRef<Path> + Debug,
{
    let target = &target.as_ref().to_path_buf();
    let destination = &destination.as_ref().to_path_buf();

    let probabilities =
        create_probabilities_map(target).context("Failed to create probabilities map")?;
    let encoder = HuffmanArchiver::new(probabilities);

    encoder.encode_file(target, destination)?;
    Ok(())
}

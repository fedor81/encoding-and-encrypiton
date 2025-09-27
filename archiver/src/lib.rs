use std::collections::HashMap;

mod codes;
mod decoder;
mod encoder;
mod freq_map;
mod huffman;
pub mod io;
mod shannon_fano;
mod utils;

pub use codes::Codes;
pub use decoder::{Decoder, FileDecoder};
pub use encoder::{Encoder, FileEncoder};
pub(crate) use freq_map::FrequencyMap;
pub use huffman::HuffmanArchiver;
pub use shannon_fano::ShannonFanoEncoder;
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

pub trait StateSaver {
    /// Сохраняет состояние объекта в вектор байтов.
    fn save_state(self) -> Vec<u8>;

    /// Загружает состояние объекта из вектора байтов.
    fn load_state(state: Vec<u8>) -> Self;
}

pub trait FileArchiver
where
    Self: FileEncoder + FileDecoder,
{
}

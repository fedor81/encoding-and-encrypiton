use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

use freq_map::FrequencyMap;

mod codes;
mod encoder;
mod freq_map;
mod huffman;
pub mod io;
mod shannon_fano;

pub use codes::Codes;
pub use encoder::Encoder;
pub use huffman::HuffmanEncoder;
pub use shannon_fano::ShannonFanoEncoder;

pub trait CodesBuilder {
    /// Строит оптимальный код на основе вероятностей вхождений символов.
    /// Сумма `probabilities` должна быть равна `1`.
    /// Возвращает вектор строк, где каждый элемент - код символа.
    fn build_optimal_codes(words: Vec<u8>, probabilities: Vec<f64>) -> Codes;

    fn build_optimal_codes_from_hashmap(words_probabilities: HashMap<u8, f64>) -> Codes {
        Self::build_optimal_codes(
            words_probabilities.keys().copied().collect(),
            words_probabilities.values().copied().collect(),
        )
    }
}

fn sort_words_and_probabilities(words: Vec<u8>, probabilities: Vec<f64>) -> (Vec<u8>, Vec<f64>) {
    let mut word_probability = words
        .into_iter()
        .zip(probabilities.into_iter())
        .collect::<Vec<_>>();

    word_probability.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap().reverse());
    word_probability.into_iter().unzip()
}

pub fn create_probabilities_map(path: &PathBuf) -> HashMap<u8, f64> {
    let file = File::open(&path).expect("Failed to open file");
    let mut reader = BufReader::new(file);

    // 1MB buffer
    let size = 1024 * 1024;
    let mut buf = vec![0u8; size];

    let mut freq_map = FrequencyMap::new();

    while let Ok(n) = reader.read(buf.as_mut_slice()) {
        freq_map.consume(&buf[..n]);
    }

    freq_map.build()
}

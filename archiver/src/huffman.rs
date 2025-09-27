use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};

use crate::{Decoder, huffman::huffman_tree::HuffmanTree};

use super::{
    Codes, CodesBuilder, Encoder, StateSaver,
    utils::{convert_to_bytes, sort_words_and_probabilities},
};
use decoder::HuffmanDecoder;

mod decoder;
mod huffman_tree;

#[derive(Debug)]
pub struct HuffmanArchiver {
    word_code: HashMap<u8, String>,
    mean_code_length: u16,
    decoder: Option<HuffmanDecoder>,
}

impl HuffmanArchiver {
    pub fn new(words_probabilities: HashMap<u8, f64>) -> Self {
        let codes = Self::build_optimal_codes_from_hashmap(words_probabilities);

        Self {
            mean_code_length: codes.mean_code_length().ceil() as u16,
            word_code: codes.into(),
            decoder: None,
        }
    }

    /// Build word_code from the remaining state bytes
    fn build_word_code(state: &[u8]) -> HashMap<u8, String> {
        let mut word_code = HashMap::new();
        let mut i = 0;

        while i < state.len() {
            let word = state[i];
            let code_len = state[i + 1] as usize;
            i += 2;

            // Читаем значение кода
            let code_value = u16::from_le_bytes([state[i], state[i + 1]]);
            i += 2;

            // Преобразуем число обратно в строковый код
            let code = format!("{:0width$b}", code_value, width = code_len);
            word_code.insert(word, code);
        }
        word_code
    }
}

impl Encoder for HuffmanArchiver {
    fn convert_to_string(&self, bytes: &[u8]) -> String {
        let capacity = bytes.len() * self.mean_code_length as usize;
        let mut bit_string = String::with_capacity(capacity);

        for byte in bytes {
            let code = self
                .word_code
                .get(byte)
                .expect(format!("Unknown byte: {}", byte).as_str());
            bit_string.push_str(code);
        }
        bit_string
    }
}

impl CodesBuilder for HuffmanArchiver {
    fn build_optimal_codes(words: Vec<u8>, probabilities: Vec<f64>) -> Codes {
        let (words, probabilities) = sort_words_and_probabilities(words, probabilities);
        let tree = HuffmanTree::build(&probabilities);
        let codes = tree.build_codes();

        Codes::new(words, probabilities, codes)
    }
}

impl StateSaver for HuffmanArchiver {
    fn save_state(self) -> Vec<u8> {
        let mut result = Vec::new();

        #[allow(unused_mut)]
        let mut items = self.word_code.into_iter().collect::<Vec<_>>();

        // При тестах сортируем
        #[cfg(test)]
        items.sort_by_key(|(word, _)| *word);

        for (word, code) in items {
            // Сохраняем слово (1 байт)
            result.push(word);

            // Сохраняем длину кода (1 байт)
            result.push(code.len() as u8);

            // Сохраняем сам код, преобразованный в байты
            let code_value = convert_to_bytes::<u16>(&code);
            result.extend_from_slice(&code_value.to_le_bytes());
        }

        result.extend_from_slice(&self.mean_code_length.to_le_bytes());
        result
    }

    fn load_state(mut state: Vec<u8>) -> Self {
        // Ensure there are at least 2 bytes to extract the mean_code_length
        assert!(
            state.len() >= 2,
            "State must contain at least 2 bytes for mean_code_length"
        );

        // Extract the last two bytes as a fixed-size array
        let mcl_bytes = [state[state.len() - 2], state[state.len() - 1]];
        let mean_code_length = u16::from_le_bytes(mcl_bytes);

        // Truncate the state to remove the mean_code_length bytes
        state.truncate(state.len() - 2);

        let mut entity = Self {
            word_code: Self::build_word_code(&state),
            mean_code_length,
            decoder: None,
        };

        entity.decoder = Some((&entity).into());
        entity
    }
}

impl Into<HuffmanDecoder> for &HuffmanArchiver {
    /// Превращает Хаффман кодировщик в декодировщик!
    fn into(self) -> HuffmanDecoder {
        // TODO: Построить дерево
        todo!()
    }
}

impl Decoder for HuffmanArchiver {
    fn decode_string(&self, bit_string: &str) -> Result<Vec<u8>> {
        self.decoder
            .as_ref()
            .context("Failed to decode string: Huffman decoder does not set.")?
            .decode_string(bit_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_optimal_codes() {
        assert_eq!(
            vec!["0", "11", "10"],
            HuffmanArchiver::build_optimal_codes(vec![1, 2, 3], vec![0.5, 0.25, 0.25]).codes()
        );
        assert_eq!(
            vec!["0", "10", "110", "111"],
            HuffmanArchiver::build_optimal_codes(vec![1, 2, 3, 4], vec![0.5, 0.25, 0.125, 0.125])
                .codes()
        );
        assert_eq!(
            vec!["00", "111", "110", "101", "011", "010", "1001", "1000"],
            HuffmanArchiver::build_optimal_codes(
                vec![1, 2, 3, 4, 5, 6, 7, 8],
                vec![0.170, 0.168, 0.166, 0.140, 0.118, 0.110, 0.083, 0.045]
            )
            .codes()
        );
    }

    #[test]
    fn test_save_and_load_huffman_archiver() {
        let archiver = HuffmanArchiver {
            word_code: HashMap::from([(1, "1".into()), (2, "11".into()), (3, "1110".into())]),
            mean_code_length: 100,
            decoder: None,
        };

        let state = archiver.save_state();
        assert_eq!(
            state,
            vec![
                // code, len, value
                1, 1, 1, 0, // (1, "1")
                2, 2, 3, 0, // (2, "11")
                3, 4, 14, 0, // (3, "1110")
                100, 0 // mean_code_length
            ]
        );

        let archiver = HuffmanArchiver::load_state(state);

        assert_eq!(archiver.mean_code_length, 100);
        assert_eq!(
            archiver.word_code,
            HashMap::from([(1, "1".into()), (2, "11".into()), (3, "1110".into())]),
        );
    }
}

use anyhow::{Context, Result};
use std::{cell::RefCell, collections::HashMap, fmt, path::Path};

use super::{
    Codes, CodesBuilder, Encoder, StateSaver,
    utils::{convert_to_bytes, sort_words_and_probabilities},
};
use crate::{Decoder, FileEncoder, create_probabilities_map, huffman::huffman_tree::HuffmanTree};
use decoder::HuffmanDecoder;

mod decoder;
mod huffman_tree;

#[derive(Debug)]
pub struct HuffmanArchiver {
    word_code: HashMap<u8, String>,
    mean_code_length: u16,
    decoder: RefCell<Option<HuffmanDecoder>>,
}

impl Clone for HuffmanArchiver {
    fn clone(&self) -> Self {
        Self {
            word_code: self.word_code.clone(),
            mean_code_length: self.mean_code_length.clone(),
            decoder: RefCell::new(None),
        }
    }
}

impl HuffmanArchiver {
    pub fn new(words_probabilities: HashMap<u8, f64>) -> Self {
        let codes = Self::build_optimal_codes_from_hashmap(words_probabilities);
        Self::_new(codes.mean_code_length().ceil() as u16, codes.into())
    }

    fn _new(mean_code_length: u16, word_code: HashMap<u8, String>) -> Self {
        Self {
            mean_code_length,
            word_code,
            decoder: RefCell::new(None),
        }
    }

    /// Archives the file in the specified location.
    pub fn archive<P>(target: P, destination: P) -> Result<()>
    where
        P: AsRef<Path> + fmt::Debug,
    {
        let target = &target.as_ref().to_path_buf();
        let destination = &destination.as_ref().to_path_buf();

        let probabilities =
            create_probabilities_map(target).context("Failed to create probabilities map")?;
        let encoder = Self::new(probabilities);

        <Self as FileEncoder>::encode_file(encoder, target, destination)?;
        Ok(())
    }

    /// If decoder is not initialized, initialize it and return
    fn decoder(&self) -> Result<std::cell::Ref<HuffmanDecoder>> {
        // Проверяем, инициализирован ли уже декодер
        if self.decoder.borrow().is_none() {
            let decoder = HuffmanDecoder::try_from(&self.word_code)?;
            *self.decoder.borrow_mut() = Some(decoder);
        }

        Ok(std::cell::Ref::map(self.decoder.borrow(), |opt| {
            opt.as_ref().unwrap()
        }))
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
        let tree = HuffmanTree::build(&probabilities, &words);
        let codes = tree.build_codes();

        Codes::new(words, probabilities, codes)
    }
}

impl StateSaver for HuffmanArchiver {
    fn save_state(self) -> Result<Vec<u8>> {
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
        Ok(result)
    }

    fn load_state(mut state: Vec<u8>) -> Result<Self> {
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

        Ok(Self::_new(mean_code_length, Self::build_word_code(&state)))
    }
}

impl TryInto<HuffmanDecoder> for &HuffmanArchiver {
    /// Превращает Хаффман кодировщик в декодировщик!
    fn try_into(self) -> Result<HuffmanDecoder, Self::Error> {
        HuffmanDecoder::try_from(&self.word_code)
    }

    type Error = anyhow::Error;
}

impl Decoder for HuffmanArchiver {
    fn decode_string(&self, bit_string: &str) -> Result<Vec<u8>> {
        self.decoder()?.decode_string(bit_string)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;

    fn new_simple_archiver() -> HuffmanArchiver {
        let word_code = HashMap::from([(1, "1".into()), (2, "11".into()), (3, "1110".into())]);
        HuffmanArchiver {
            word_code: word_code,
            mean_code_length: 100,
            decoder: RefCell::new(None),
        }
    }

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
        let archiver = new_simple_archiver();
        let word_code = archiver.word_code.clone();

        let state = archiver.save_state().unwrap();
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

        let archiver = HuffmanArchiver::load_state(state).unwrap();

        assert_eq!(archiver.mean_code_length, 100);
        assert_eq!(archiver.word_code, word_code);
    }

    #[test]
    fn test_save_and_load_huffman_archiver_to_file() {
        let expected_archiver = new_simple_archiver();
        let expected_state = expected_archiver
            .clone()
            .save_state()
            .expect("Failed to save state");

        let actual_state;
        let filename = "test_save_and_load_huffman_archiver_to_file.huff";
        std::fs::remove_file(filename).ok();

        {
            let mut file = File::create(filename).expect("Failed to create file");
            HuffmanArchiver::write_state(&expected_state, &mut file).unwrap();
        }

        {
            let mut file = File::open(filename).expect("Failed to open file");
            actual_state = HuffmanArchiver::read_state(&mut file).unwrap();
        }

        assert_eq!(expected_state, actual_state);

        let actual_archiver = HuffmanArchiver::load_state(actual_state).unwrap();

        assert_eq!(actual_archiver.word_code, expected_archiver.word_code);
        assert_eq!(
            actual_archiver.mean_code_length,
            expected_archiver.mean_code_length
        );

        std::fs::remove_file(filename).ok();
    }
}

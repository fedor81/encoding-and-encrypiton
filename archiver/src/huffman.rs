use std::collections::HashMap;

use super::{Codes, CodesBuilder, Encoder, sort_words_and_probabilities};

mod huffman_tree;
use huffman_tree::build_tree;

#[derive(Debug)]
pub struct HuffmanEncoder {
    word_code: HashMap<u8, String>,
    mean_code_length: usize,
}

impl HuffmanEncoder {
    pub fn new(words_probabilities: HashMap<u8, f64>) -> Self {
        let codes = Self::build_optimal_codes_from_hashmap(words_probabilities);
        Self {
            mean_code_length: codes.mean_code_length().ceil() as usize,
            word_code: codes.into(),
        }
    }

    pub fn serialize_codes(&self) -> String {
        todo!()
    }
}

impl Encoder for HuffmanEncoder {
    fn convert_to_string(&self, bytes: &[u8]) -> String {
        let capacity = bytes.len() * self.mean_code_length;
        let mut bit_string = String::with_capacity(capacity);

        for &byte in bytes {
            let code = self.word_code.get(&byte).expect("Unknown byte");
            bit_string.push_str(code);
        }
        bit_string
    }
}

impl CodesBuilder for HuffmanEncoder {
    fn build_optimal_codes(words: Vec<u8>, probabilities: Vec<f64>) -> Codes {
        let (words, probabilities) = sort_words_and_probabilities(words, probabilities);
        let tree = build_tree(&probabilities);
        let codes = tree.build_codes();

        Codes::new(words, probabilities, codes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_optimal_codes() {
        assert_eq!(
            vec!["0", "11", "10"],
            HuffmanEncoder::build_optimal_codes(vec![1, 2, 3], vec![0.5, 0.25, 0.25]).codes()
        );
    }
}

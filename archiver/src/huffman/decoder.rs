use std::collections::HashMap;

use anyhow::{Context, Ok, Result};

use crate::{Decoder, huffman::huffman_tree::HuffmanTree};

#[derive(Debug)]
pub struct HuffmanDecoder {
    tree: HuffmanTree,
}

impl HuffmanDecoder {
    pub fn new(tree: HuffmanTree) -> Self {
        Self { tree }
    }
}

impl Decoder for HuffmanDecoder {
    fn decode_string(&self, bit_string: &str) -> Result<Vec<u8>> {
        let mut decoded = Vec::new();
        let bit_string = bit_string.as_bytes();

        let mut node = &self.tree;
        let mut i = 0;

        while i < bit_string.len() {
            match bit_string[i] {
                b'0' => {
                    node = node.left().ok_or(anyhow::anyhow!("Invalid bit string"))?;
                }
                b'1' => {
                    node = node.right().ok_or(anyhow::anyhow!("Invalid bit string"))?;
                }
                _ => {
                    anyhow::bail!("Invalid bit string")
                }
            };

            // Если мы дошли до листа, то добавляем слово в результат
            if let HuffmanTree::Leaf { word, .. } = node {
                decoded.push(*word);
                node = &self.tree;
            }
            i += 1;
        }

        Ok(decoded)
    }
}

impl TryFrom<&HashMap<u8, String>> for HuffmanDecoder {
    type Error = anyhow::Error;

    fn try_from(value: &HashMap<u8, String>) -> std::result::Result<Self, Self::Error> {
        let tree = HuffmanTree::restore_from_word_code(value)
            .context("Failed to convert HuffmanArchiver into HuffmanDecoder")?;
        Ok(HuffmanDecoder::new(tree))
    }
}

#[cfg(test)]
mod tests {
    use super::super::huffman_tree::tests::{new_test_codes, new_test_tree};
    use super::*;

    #[test]
    fn test_simple_decode_string() {
        let decoder = HuffmanDecoder::new(HuffmanTree::Node {
            probability: 1.0,
            left: HuffmanTree::new_leaf(0.5, 0, 5).into(),
            right: HuffmanTree::new_leaf(0.5, 1, 8).into(),
            count_codes: 2,
        });
        assert_eq!(
            decoder.decode_string("0101011").unwrap(),
            vec![5, 8, 5, 8, 5, 8, 8],
        );
    }

    #[test]
    fn test_decode_string() {
        let mut input = String::new();
        let codes = new_test_codes();

        let expected = vec![1, 0, 3, 2, 4, 0];

        for word in &expected {
            input.push_str(
                codes
                    .get(&word)
                    .expect(&format!("Word: {} does not contains in codes", word).as_str()),
            );
        }

        let decoder = HuffmanDecoder::new(new_test_tree());
        assert_eq!(decoder.decode_string(&input).unwrap(), expected);
    }
}

use std::collections::HashMap;

use anyhow::{Context, Result};

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
        todo!()
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

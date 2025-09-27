use anyhow::{Context, Result};

use crate::Decoder;

#[derive(Debug)]
pub struct HuffmanDecoder {}

impl Decoder for HuffmanDecoder {
    fn decode_string(&self, bit_string: &str) -> Result<Vec<u8>> {
        todo!()
    }
}

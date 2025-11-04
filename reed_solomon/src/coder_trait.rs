use super::*;

pub trait Coder {
    fn encode(&self, data: &[u8]) -> Result<Vec<u8>>;
    fn decode(&self, data: &[u8]) -> Result<Vec<u8>>;
}

impl<T> BlockCoder for T where T: Coder {}

/// Для работы с блоками данных
pub trait BlockCoder: Coder {
    fn encode_blocks(&self, data: &[u8], block_size: usize) -> Result<Vec<Vec<u8>>> {
        let mut blocks = Vec::new();

        for chunk in data.chunks(block_size) {
            let encoded_block = self.encode(chunk)?;
            blocks.push(encoded_block);
        }

        Ok(blocks)
    }

    fn decode_blocks(&self, data: &[u8], block_size: usize) -> Result<Vec<Vec<u8>>> {
        let mut blocks = Vec::new();

        for chunk in data.chunks(block_size) {
            let decoded_block = self.decode(chunk)?;
            blocks.push(decoded_block);
        }

        Ok(blocks)
    }

    fn decode_blocks_to_vec(&self, data: &[u8], block_size: usize) -> Result<Vec<u8>> {
        match self.decode_blocks(data, block_size) {
            Ok(blocks) => Ok(blocks.into_iter().flatten().collect()),
            Err(e) => Err(e),
        }
    }

    fn encode_blocks_to_vec(&self, data: &[u8], block_size: usize) -> Result<Vec<u8>> {
        match self.encode_blocks(data, block_size) {
            Ok(blocks) => Ok(blocks.into_iter().flatten().collect()),
            Err(e) => Err(e),
        }
    }
}

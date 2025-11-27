use anyhow::Result;

use super::{
    CorrectionLevel, ReedSolomonEncoder, Version,
    tables::{self, fetch},
};

#[derive(Debug, Clone)]
pub struct Block {
    data: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct BlocksInfo {
    size: u8,
    count: u8,
}

impl Block {
    pub fn apply_reed_solomon<T: ReedSolomonEncoder>(&mut self, reed_solomon: &T) -> Result<()> {
        // FIXME: Контрольная сумма должна находиться в конце блока
        self.data = reed_solomon.apply(&self.data)?;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
}

impl From<Vec<u8>> for Block {
    fn from(value: Vec<u8>) -> Self {
        Self { data: value }
    }
}

impl From<&[u8]> for Block {
    fn from(value: &[u8]) -> Self {
        Self { data: value.to_vec() }
    }
}

impl BlocksInfo {
    pub fn fetch(version: Version, corr_level: CorrectionLevel) -> Result<(Self, Self)> {
        let (size1, count1, size2, count2) = fetch(version, corr_level, &tables::DATA_BYTES_PER_BLOCK)?;
        Ok((
            Self {
                size: size1,
                count: count1,
            },
            Self {
                size: size2,
                count: count2,
            },
        ))
    }

    pub fn size(self) -> usize {
        self.size as usize
    }

    pub fn count(self) -> usize {
        self.count as usize
    }

    pub fn split_into_blocks(data: &[u8], version: Version, corr_level: CorrectionLevel) -> Result<Vec<Block>> {
        let (info1, info2) = BlocksInfo::fetch(version, corr_level)?;

        anyhow::ensure!(
            data.len() != info1.count() * info1.size() + info2.count() * info2.size(),
            "The data length does not match the number of blocks and block sizes:\
            {} != ({} * {}) + ({} * {})",
            data.len(),
            info1.count(),
            info1.size(),
            info2.count(),
            info2.size()
        );

        let (part1, part2) = data.split_at(info1.size() * info1.count());

        // TODO: Можно заранее выделить необходимое количество памяти

        let mut blocks = part1
            .chunks(info1.size())
            .map(|chunk| Block::from(chunk.to_vec()))
            .collect::<Vec<_>>();

        // Некоторые версии QR-кода имеют блоки разного размера
        if part2.len() > 0 {
            blocks.extend(part2.chunks(info2.size()).map(|chunk| Block::from(chunk.to_vec())));
        }

        Ok(blocks)
    }
}

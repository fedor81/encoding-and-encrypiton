use anyhow::Result;
use reed_solomon::{self, Coder, ReedSolomon, gf::FastGF256, new_reed_solomon};

use super::{Block, CorrectionLevel, Version, tables};

pub trait ReedSolomonEncoder
where
    Self: Sized,
{
    fn apply(&self, data: &[u8]) -> Result<Vec<u8>>;
    fn new(version: Version, corr_level: CorrectionLevel) -> Result<Self>;

    fn apply_for_blocks(&self, blocks: &mut [Block]) -> Result<()> {
        for block in blocks {
            block.apply_reed_solomon(self)?;
        }
        Ok(())
    }
}

impl ReedSolomonEncoder for ReedSolomon<FastGF256> {
    fn apply(&self, data: &[u8]) -> Result<Vec<u8>> {
        self.encode(data)
    }

    fn new(version: Version, corr_level: CorrectionLevel) -> Result<Self> {
        let control_count = tables::EC_BYTES_PER_BLOCK
            .get(version.num() as usize - 1)
            .and_then(|bytes_per_level| bytes_per_level.get(corr_level.index()).copied())
            .ok_or(anyhow::format_err!(
                "Index out of range. Invalid QR version: {} or correction level index: {}",
                version.num(),
                corr_level.index()
            ))? as usize;
        Ok(new_reed_solomon(control_count))
    }
}

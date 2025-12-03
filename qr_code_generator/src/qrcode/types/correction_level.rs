use super::{CorrectionLevel, Version, tables};

impl CorrectionLevel {
    pub fn max_data_len(self, version: Version) -> usize {
        tables::fetch(version, self, &tables::DATA_LENGTHS).unwrap() as usize
    }

    /// Returns the index of the current level in the table.
    ///
    /// # Returns
    /// L – 0, M – 1, Q – 2, H – 3
    pub fn index(self) -> usize {
        match self {
            Self::L => 0,
            Self::M => 1,
            Self::Q => 2,
            Self::H => 3,
        }
    }
}

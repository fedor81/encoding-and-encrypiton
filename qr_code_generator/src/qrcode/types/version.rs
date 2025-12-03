use super::{CorrectionLevel, Version, tables};
use crate::utils;

impl Version {
    /// # Panics
    /// if not `1 <= version <= 40`
    pub fn new(version: u8) -> Self {
        assert!(1 <= version && version <= 40);
        Self(version)
    }

    /// Количество модулей QR-кода
    pub const fn size(self) -> usize {
        self.0 as usize * 4 + 17
    }

    /// # Panics
    pub fn build(bytes_count: usize, corr_level: CorrectionLevel) -> Self {
        for version in 1..=40 {
            if bytes_count <= Self::new(version).max_bytes_count(corr_level) {
                return Self::new(version);
            }
        }
        panic!("Version cannot be selected for level: {corr_level:?}, too much data: {bytes_count}")
    }

    /// Возвращает максимально допустимое количество бит
    pub fn max_bytes_count(self, corr_level: CorrectionLevel) -> usize {
        tables::fetch(self, corr_level, &tables::DATA_LENGTHS).unwrap() as usize / 8
    }

    /// Возвращает номер версии
    pub fn num(self) -> usize {
        self.0 as usize
    }

    pub fn get_alignment_positions(self) -> &'static [u8] {
        match self.0 {
            1..=40 => tables::ALIGNMENT_PATTERN_POSITIONS[self.0 as usize - 1],
            _ => unreachable!(),
        }
    }

    pub fn get_version_info_bits(self) -> Vec<bool> {
        utils::byte_to_bits(self.0)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(3550, CorrectionLevel::Q, Version(19))]
    #[case(2725, CorrectionLevel::H, Version(19))]
    #[case(18671, CorrectionLevel::M, Version(40))]
    #[case(16, CorrectionLevel::L, Version(1))]
    fn test_build(#[case] bits_count: usize, #[case] corr_level: CorrectionLevel, #[case] expected: Version) {
        assert_eq!(Version::build(bits_count / 8, corr_level), expected);
    }
}

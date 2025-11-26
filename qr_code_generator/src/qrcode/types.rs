use std::ops::Not;

use super::tables::{DATA_LENGTHS, fetch};

/// QR коды разных уровней коррекции
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CorrectionLevel {
    /// Low - 7%
    L,
    /// Medium - 15%
    M,
    /// Quartile - 25%
    Q,
    /// High - 30%
    H,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Module {
    Dark,
    Light,
    #[default]
    Unused,
}

impl Not for Module {
    type Output = Self;
    fn not(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
            Self::Unused => panic!("Unused module cannot be inverted"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Version(u8);

impl Version {
    /// # Panics
    /// if not `1 <= version <= 40`
    pub fn new(version: u8) -> Self {
        assert!(1 <= version && version <= 40);
        Self(version)
    }

    /// Количество модулей QR-кода
    pub const fn size(self) -> u8 {
        self.0 * 4 + 17
    }

    /// # Panics
    pub fn build(bits_count: usize, corr_level: CorrectionLevel) -> Self {
        for version in 1..=40 {
            if bits_count <= Self::new(version).max_data_len(corr_level) {
                return Self::new(version);
            }
        }
        panic!("The version cannot be selected, there is too much data.")
    }

    pub fn max_data_len(self, corr_level: CorrectionLevel) -> usize {
        fetch(self, corr_level, &DATA_LENGTHS).unwrap() as usize
    }

    /// Возвращает номер версии
    pub fn num(self) -> usize {
        self.0 as usize
    }
}

impl CorrectionLevel {
    pub fn max_data_len(self, version: Version) -> usize {
        fetch(version, self, &DATA_LENGTHS).unwrap() as usize
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

use anyhow::{Context, Result};
use std::ops::Not;

use crate::utils::{add_zeros, bits_to_bytes, bytes_to_bits};
use tables::DATA_LENGTHS;

mod tables;

pub struct QRCode {
    data: Vec<u8>,
    version: Version,
    corr_level: CorrectionLevel,
    modules: Vec<Vec<Module>>,
}

impl QRCode {
    /// Кодирование происходит побайтовым способом, что позволяет кодировать любую последовательность
    /// байт, например UTF-8, но уменьшает плотность данных.
    pub fn build(data: &[u8], corr_level: CorrectionLevel) -> Result<Self> {
        let mut data = Self::add_service_information(data);
        let version = Version::build(data.len() * 8, corr_level);
        Self::expand_to_max_size(&mut data, version, corr_level);

        Ok(Self {
            data,
            version,
            corr_level,
            modules: vec![vec![Module::default()]; version.max_data_len(corr_level)],
        })
    }

    /// Способ кодирования — поле длиной 4 бита, которое имеет следующие значения:
    /// - 0001 для цифрового кодирования
    /// - 0010 для буквенно-цифрового
    /// - 0100 для побайтового
    const BYTES_ENCODING: &[bool] = &[false, true, false, false];

    fn add_service_information(data: &[u8]) -> Vec<u8> {
        let payload_len = data.len();
        let mut result = Vec::new();

        result.extend_from_slice(Self::BYTES_ENCODING);
        result.extend_from_slice(&bytes_to_bits(&payload_len.to_le_bytes()));
        result.extend_from_slice(&bytes_to_bits(data));

        add_zeros(&mut result); // Дописываем нули в конец до кратности 8
        bits_to_bytes(&result).expect("The sequence must be a multiple of 8 after add zeros")
    }

    /// Дополняет данные до максимально возможной длины в версии чередующимися байтами EC и 11
    fn expand_to_max_size(data: &mut Vec<u8>, version: Version, corr_level: CorrectionLevel) {
        let mut push_ec = true;

        while data.len() < version.max_data_len(corr_level) {
            if push_ec {
                data.push(0b11101100); // EC
            } else {
                data.push(0b00010001); // 11
            }
            push_ec = !push_ec;
        }
    }
}

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
enum Module {
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
    fn build(bits_count: usize, corr_level: CorrectionLevel) -> Self {
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
}

impl CorrectionLevel {
    pub fn max_data_len(self, version: Version) -> usize {
        fetch(version, self, &DATA_LENGTHS).unwrap() as usize
    }
}

/// Obtains an object from a hard-coded table.
///
/// The table must be a 40×4 array. The outer array represents the content for each version.
/// The inner array represents the content in each error correction level, in the order [L, M, Q, H].
fn fetch<T>(version: Version, corr_level: CorrectionLevel, table: &[[T; 4]; 40]) -> Result<T>
where
    T: PartialEq + Default + Copy,
{
    if 1 <= version.0 && version.0 <= 40 {
        return Ok(table[(version.0 - 1) as usize][corr_level as usize]);
    }
    anyhow::bail!("Invalid version: {}. Version must be in range [1, 40]", version.0)
}

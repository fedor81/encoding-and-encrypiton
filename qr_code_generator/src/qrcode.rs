use anyhow::{Context, Result};
use std::ops::Not;

use crate::utils::convert_to_bits;

pub struct QRCode {
    data: Vec<bool>,
    version: Version,
    corr_level: CorrectionLevel,
    modules: Vec<Vec<Module>>,
}

impl QRCode {
    /// Кодирование происходит побайтовым способом, что позволяет кодировать любую последовательность
    /// байт, например UTF-8, но уменьшает плотность данных.
    pub fn build(data: &[u8], corr_level: CorrectionLevel) -> Result<Self> {
        let payload_len = data.len();
        let mut data = convert_to_bits(data);

        /// Способ кодирования — поле длиной 4 бита, которое имеет следующие значения:
        /// - 0001 для цифрового кодирования
        /// - 0010 для буквенно-цифрового
        /// - 0100 для побайтового
        const BYTES_ENCODING: &[bool] = &[false, true, false, false];

        let mut data = Vec::new();
        data.extend_from_slice(BYTES_ENCODING);
        data.extend_from_slice(&convert_to_bits(&payload_len.to_le_bytes()));

        let version = Version::build(data.len(), corr_level)?;
        Ok(Self {
            data,
            version,
            corr_level,
            modules: vec![vec![Module::default()]; version.max_len(corr_level)? as usize],
        })
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
struct Version(u8);

impl Version {
    /// # Panics
    /// if not `1 <= version <= 40`
    pub fn new(version: u8) -> Self {
        assert!(1 <= version && version <= 40);
        Self(version)
    }

    /// Количество моделей QR-кода
    pub const fn size(self) -> u8 {
        self.0 * 4 + 17
    }

    pub fn max_len(self, corr_level: CorrectionLevel) -> Result<u16> {
        if 1 <= self.0 && self.0 <= 40 {
            Ok(DATA_LENGTHS[(self.0 - 1) as usize][corr_level as usize])
        } else {
            anyhow::bail!(
                "Invalid version: {}. Version must be in range [1, 40]",
                self.0
            )
        }
    }

    pub fn build(data_len: usize, corr_level: CorrectionLevel) -> Result<Self> {
        todo!()
    }
}

// This table is copied from https://github.com/kennytm/qrcode-rust
const DATA_LENGTHS: [[u16; 4]; 40] = [
    [152, 128, 104, 72],
    [272, 224, 176, 128],
    [440, 352, 272, 208],
    [640, 512, 384, 288],
    [864, 688, 496, 368],
    [1088, 864, 608, 480],
    [1248, 992, 704, 528],
    [1552, 1232, 880, 688],
    [1856, 1456, 1056, 800],
    [2192, 1728, 1232, 976],
    [2592, 2032, 1440, 1120],
    [2960, 2320, 1648, 1264],
    [3424, 2672, 1952, 1440],
    [3688, 2920, 2088, 1576],
    [4184, 3320, 2360, 1784],
    [4712, 3624, 2600, 2024],
    [5176, 4056, 2936, 2264],
    [5768, 4504, 3176, 2504],
    [6360, 5016, 3560, 2728],
    [6888, 5352, 3880, 3080],
    [7456, 5712, 4096, 3248],
    [8048, 6256, 4544, 3536],
    [8752, 6880, 4912, 3712],
    [9392, 7312, 5312, 4112],
    [10208, 8000, 5744, 4304],
    [10960, 8496, 6032, 4768],
    [11744, 9024, 6464, 5024],
    [12248, 9544, 6968, 5288],
    [13048, 10136, 7288, 5608],
    [13880, 10984, 7880, 5960],
    [14744, 11640, 8264, 6344],
    [15640, 12328, 8920, 6760],
    [16568, 13048, 9368, 7208],
    [17528, 13800, 9848, 7688],
    [18448, 14496, 10288, 7888],
    [19472, 15312, 10832, 8432],
    [20528, 15936, 11408, 8768],
    [21616, 16816, 12016, 9136],
    [22496, 17728, 12656, 9776],
    [23648, 18672, 13328, 10208],
];

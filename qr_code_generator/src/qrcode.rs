use anyhow::{Context, Result};
use std::ops::Not;

use crate::utils::{add_zeros, bits_to_bytes, bytes_to_bits};
use tables::{DATA_BYTES_PER_BLOCK, DATA_LENGTHS};

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
        let blocks = BlocksInfo::split_into_blocks(&data, version, corr_level)?;

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

    /// Добавляет способ кдирования и длину данных
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

#[derive(Debug, Clone)]
struct Block {
    data: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
struct BlocksInfo {
    size: u8,
    count: u8,
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
        let (size1, count1, size2, count2) = fetch(version, corr_level, &DATA_BYTES_PER_BLOCK)?;
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

        if data.len() != info1.count() * info1.size() + info2.count() * info2.size() {
            anyhow::bail!(
                "The data length does not match the number of blocks and block sizes:\
                {} != ({} * {}) + ({} * {})",
                data.len(),
                info1.count(),
                info1.size(),
                info2.count(),
                info2.size()
            );
        }

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

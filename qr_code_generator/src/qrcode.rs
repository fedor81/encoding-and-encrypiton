use anyhow::{Context, Ok, Result};

use crate::utils::{add_zeros, bits_to_bytes, bytes_to_bits};

mod blocks;
mod draw;
mod rs_encoder;
mod tables;
mod types;

pub(self) use blocks::{Block, BlocksInfo};
pub(self) use rs_encoder::ReedSolomonEncoder;
pub(self) use types::{Canvas, MaskPattern};
pub use types::{CorrectionLevel, Module, Version};

pub struct QRCode {
    modules: Canvas,
}

impl QRCode {
    /// Кодирование происходит побайтовым способом, что позволяет кодировать любую последовательность
    /// байт, например UTF-8, но уменьшает плотность данных.
    pub fn build<T: ReedSolomonEncoder>(data: &[u8], corr_level: CorrectionLevel) -> Result<Self> {
        let mut data = Self::add_service_information(data);
        let version = Version::build(data.len() * 8, corr_level);

        Self::expand_to_max_size(&mut data, version, corr_level);

        // Разбиваем данные на блоки
        let mut blocks = BlocksInfo::split_into_blocks(&data, version, corr_level).context("split data into blocks")?;

        // Применяем кодирование
        Self::apply_reed_solomon::<T>(&mut blocks, version, corr_level)
            .context("apply Reed-Solomon error correction")?;

        // Объединяем блоки
        data = Self::combine_blocks(&blocks);

        let modules = Self::build_modules(&data, corr_level, version).context("build QR code modules (canvas)")?;

        Ok(Self { modules })
    }

    pub fn build_with_default_encoder(data: &[u8], corr_level: CorrectionLevel) -> Result<Self> {
        Self::build::<reed_solomon::ReedSolomon<reed_solomon::gf::FastGF256>>(data, corr_level)
            .with_context(|| "Failed to build QR code with default encoder")
    }

    /// У нас имеется несколько блоков данных и столько же блоков байтов коррекции,
    /// их надо объединить в один поток байт. Делается это следующим образом:
    /// из каждого блока данных по очереди берётся один байт информации, когда очередь
    /// доходит до последнего блока, из него берётся байт и очередь переходит к первому блоку.
    /// Так продолжается до тех пор, пока в каждом блоке не кончатся байты.
    /// Если в текущем блоке уже нет байт, то он пропускается.
    fn combine_blocks(blocks: &[Block]) -> Vec<u8> {
        let mut result = Vec::with_capacity(blocks.len() * blocks.get(0).expect("blocks slice is empty").len());

        // TODO: Можно попробовать избежать лишнего копирования
        for i in 0..blocks.len() {
            for block in blocks {
                if let Some(&byte) = block.as_slice().get(i) {
                    result.push(byte);
                }
            }
        }

        result
    }

    fn apply_reed_solomon<T: ReedSolomonEncoder>(
        blocks: &mut [Block],
        version: Version,
        corr_level: CorrectionLevel,
    ) -> Result<()> {
        let reed_solomon = T::new(version, corr_level)?;
        reed_solomon.apply_for_blocks(blocks)
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

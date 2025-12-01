use anyhow::{Context, Ok, Result};
use image::{ImageBuffer, Luma};

use crate::{
    Drawable,
    utils::{add_zeros, bits_to_bytes, bytes_to_bits},
};

mod blocks;
mod rs_encoder;
mod tables;
mod types;

pub(self) use blocks::{Block, BlocksInfo};
pub(self) use rs_encoder::ReedSolomonEncoder;
pub(self) use types::Canvas;
pub use types::{CorrectionLevel, Module, Version};

pub struct QRCode {
    data: Vec<u8>,
    version: Version,
    corr_level: CorrectionLevel,
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

        Ok(Self {
            data,
            version,
            corr_level,
            modules,
        })
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

    fn build_modules(data: &[u8], corr_level: CorrectionLevel, version: Version) -> Result<Canvas> {
        let size = version.size();
        let mut modules = vec![vec![Module::default(); size]; size];

        Self::add_patterns(&mut modules, version)?;

        Ok(modules)
    }

    fn add_patterns(modules: &mut Canvas, version: Version) -> Result<()> {
        Self::add_finder_patterns(modules).context("add finder patterns")?;
        Self::add_separators(modules).context("add separators")?;
        Self::add_timing_patterns(modules).context("add timing patterns")?;
        Self::add_alignment_patterns(modules, version).context("add alignment patterns")?;
        Self::add_version_info(modules, version).context("add version information")?;
        Ok(())
    }

    fn add_finder_patterns(modules: &mut Canvas) -> Result<()> {
        let size = modules.len();

        if size < 7 {
            anyhow::bail!("QR code size is too small to fit finder patterns (size < 7)");
        }

        // Верхний левый
        Self::add_finder_pattern(modules, 0, 0).context("failed to add top-left finder pattern")?;

        // Верхний правый
        Self::add_finder_pattern(modules, size - 7, 0).context("failed to add top-right finder pattern")?;

        // Нижний левый
        Self::add_finder_pattern(modules, 0, size - 7).context("failed to add bottom-left finder pattern")?;

        Ok(())
    }

    /// Добавляет чёрный квадрат размером 3 на 3 модуля, который окружён рамкой из белых модулей,
    /// которая окружена рамкой из чёрных модулей, которая окружена рамкой из белых модулей
    /// только с тех сторон, где нет отступа.
    fn add_finder_pattern(modules: &mut Canvas, x: usize, y: usize) -> Result<()> {
        for i in 0..7 {
            for j in 0..7 {
                modules[y + j][x + i].try_set(
                    if i == 0 || i == 6 || j == 0 || j == 6 || (i >= 2 && i <= 4 && j >= 2 && j <= 4) {
                        Module::Dark
                    } else {
                        Module::Light
                    },
                )?;
            }
        }
        Ok(())
    }

    fn add_separators(modules: &mut Canvas) -> Result<()> {
        let size = modules.len();

        for i in 0..8 {
            if i < 7 {
                modules[7][i].try_set(Module::Light)?; // справа
                modules[i][7].try_set(Module::Light)?; // снизу
            }
            modules[7][size - 8].try_set(Module::Light)?; // слева от верхнего правого
            modules[size - 8][7].try_set(Module::Light)?; // сверху от нижнего левого
        }
        Ok(())
    }

    /// Добавляет полосы синхронизации. Полосы начинаются от самого нижнего правого чёрного
    /// модуля верхнего левого поискового узора и идут, чередуя чёрные и белые модули,
    /// вниз и вправо до противоположных поисковых узоров.
    fn add_timing_patterns(modules: &mut Canvas) -> Result<()> {
        let size = modules.len();

        // Горизонтальный тайминг
        for i in 8..size - 8 {
            modules[6][i]
                .try_set(if i % 2 == 0 { Module::Dark } else { Module::Light })
                .context("set horizontal timing")?;
        }

        // Вертикальный тайминг
        for i in 8..size - 8 {
            modules[i][6]
                .try_set(if i % 2 == 0 { Module::Dark } else { Module::Light })
                .context("set vertical timing")?;
        }
        Ok(())
    }

    /// Добавляет выравнивающие узоры (alignment patterns).
    /// Пропускает позиции, где находятся finder patterns.
    fn add_alignment_patterns(modules: &mut Canvas, version: Version) -> Result<()> {
        let size = modules.len();
        let positions = version.get_alignment_positions();

        for x in positions.iter().map(|n| *n as usize) {
            for y in positions.iter().map(|n| *n as usize) {
                // Пропускаем позиции с finder patterns
                if !((x < 9 && y < 9) || (x > size - 10 && y < 9) || (x < 9 && y > size - 10)) {
                    Self::add_single_alignment_pattern(modules, x, y)
                        .with_context(|| format!("failed to add alignment pattern at ({}, {})", x, y))?;
                }
            }
        }
        Ok(())
    }

    /// Добавляет один выравнивающий узор 5×5 с центром в (x, y).
    /// Требует, чтобы координаты (x, y) были ≥ 2 и ≤ size - 3, чтобы шаблон вмещался.
    fn add_single_alignment_pattern(modules: &mut Canvas, x: usize, y: usize) -> Result<()> {
        // Проверим, что шаблон поместится
        let size = modules.len();
        if x < 2 || x >= size - 2 || y < 2 || y >= size - 2 {
            anyhow::bail!(
                "alignment pattern at ({}, {}) is too close to the edge and would go out of bounds",
                x,
                y
            );
        }

        for i in 0..5 {
            for j in 0..5 {
                modules[y - 2 + j][x - 2 + i]
                    .try_set(if i == 0 || i == 4 || j == 0 || j == 4 || (i == 2 && j == 2) {
                        Module::Dark
                    } else {
                        Module::Light
                    })
                    .with_context(|| {
                        format!(
                            "failed to set module at ({}, {}) while adding alignment pattern centered at ({x}, {y})",
                            x - 2 + i,
                            y - 2 + j,
                        )
                    })?;
            }
        }
        Ok(())
    }

    fn add_version_info(modules: &mut Canvas, version: Version) -> Result<()> {
        if version.num() >= 7 {
            let size = modules.len();
            let version_info = version.get_version_info_bits();

            // Размещаем биты версии
            for i in 0..18 {
                let row = i / 3;
                let col = i % 3;

                let module = if version_info[i] { Module::Dark } else { Module::Light };

                // Нижний левый
                modules[size - 11 + row][col].try_set(module).with_context(|| {
                    format!(
                        "failed to set version info bit {i} at ({}, {col}) in bottom-left area",
                        size - 11 + row,
                    )
                })?;

                // Верхний правый
                modules[row][size - 11 + col].try_set(module).with_context(|| {
                    format!(
                        "failed to set version info bit {i} at ({row}, {}) in top-right area",
                        size - 11 + col
                    )
                })?;
            }
        }
        Ok(())
    }
}

impl Drawable for QRCode {
    fn draw<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let size = self.modules.len();
        let mut img = ImageBuffer::<Luma<u8>, Vec<u8>>::new(size as u32, size as u32);

        for x in 0..size {
            for y in 0..size {
                let color = if self.modules[x][y].is_dark() { 0 } else { 255 };
                img.put_pixel(x as u32, y as u32, Luma([color]));
            }
        }

        img.save(path)?;
        Ok(())
    }
}

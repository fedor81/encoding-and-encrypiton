//! Определение построения Canvas<Module> QR-кода
use image::{ImageBuffer, Luma};

use super::*;
use crate::Drawable;

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

impl QRCode {
    pub fn build_modules(data: &[u8], corr_level: CorrectionLevel, version: Version) -> Result<Canvas> {
        let size = version.size();
        let mut modules = Canvas::new(size);

        Self::add_patterns(&mut modules, version)?;

        Ok(modules)
    }

    fn add_patterns(modules: &mut Canvas, version: Version) -> Result<()> {
        Self::add_finder_patterns(modules).context("add finder patterns")?;
        Self::add_separators(modules).context("add separators for finders")?;
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
                modules.try_set(
                    x + i,
                    y + j,
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

        for y in 0..8 {
            // Вертикально справа от левого верхнего finder (x=7)
            modules.try_set(7, y, Module::Light).context("vertical top-right")?;

            // Слева от верхнего правого finder (x = size - 8)
            modules
                .try_set(size - 8, y, Module::Light)
                .context("vertical top-right")?;

            // Справа от нижнего левого finder (x = 7)
            modules
                .try_set(7, size - 8 + y, Module::Light)
                .context("vertical bottom-left")?;
        }

        for x in 0..7 {
            // Горизонтально под левым верхним finder (y=7)
            modules.try_set(x, 7, Module::Light).context("horizontal top-left")?;

            // Под правым верхним finder (y=7)
            modules
                .try_set(size - 7 + x, 7, Module::Light)
                .context("horizontal top-right")?;

            // Над нижним левым finder (y = size - 8)
            modules
                .try_set(x, size - 8, Module::Light)
                .context("horizontal bottom-left")?;
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
            modules
                .try_set(6, i, if i % 2 == 0 { Module::Dark } else { Module::Light })
                .context("set horizontal timing")?;
        }

        // Вертикальный тайминг
        for i in 8..size - 8 {
            modules
                .try_set(i, 6, if i % 2 == 0 { Module::Dark } else { Module::Light })
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
                modules
                    .try_set(
                        x - 2 + i,
                        y - 2 + j,
                        if i == 0 || i == 4 || j == 0 || j == 4 || (i == 2 && j == 2) {
                            Module::Dark
                        } else {
                            Module::Light
                        },
                    )
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
                modules.try_set(col, size - 11 + row, module).with_context(|| {
                    format!(
                        "failed to set version info bit {i} at ({}, {col}) in bottom-left area",
                        size - 11 + row,
                    )
                })?;

                // Верхний правый
                modules.try_set(size - 11 + col, row, module).with_context(|| {
                    format!(
                        "failed to set version info bit {i} at ({row}, {}) in top-right area",
                        size - 11 + col
                    )
                })?;
            }
        }
        Ok(())
    }

    fn add_mask_info(modules: &mut Canvas, mask: MaskPattern) -> Result<()> {
        todo!()
    }
}

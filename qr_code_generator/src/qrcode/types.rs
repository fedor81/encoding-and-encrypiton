use anyhow::Result;
use std::ops::Not;

use crate::{qrcode::tables, utils};

use super::tables::{DATA_LENGTHS, fetch};

/// QR код в виде матрицы модулей
pub type Canvas = Vec<Vec<Module>>;

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

impl Module {
    pub fn is_dark(self) -> bool {
        self == Self::Dark
    }

    pub fn is_light(self) -> bool {
        self == Self::Light
    }

    pub fn is_unused(self) -> bool {
        self == Self::Unused
    }

    /// Устанавливает значение модуля, если он Unused, иначе ошибка
    pub fn try_set(&mut self, module: Module) -> Result<()> {
        self.try_set_with(|| module)
    }

    /// Устанавливает значение модуля, возвращаемой функцией. НО если модуль не Unused - ошибка
    pub fn try_set_with<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce() -> Module,
    {
        if !self.is_unused() {
            anyhow::bail!("Cannot replace a non-unused module with another module");
        }
        *self = f();
        Ok(())
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
    pub const fn size(self) -> usize {
        self.0 as usize * 4 + 17
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

    pub fn get_alignment_positions(self) -> &'static [u8] {
        match self.0 {
            1..=40 => tables::ALIGNMENT_PATTERN_POSITIONS[self.0 as usize],
            _ => unreachable!(),
        }
    }

    pub fn get_version_info_bits(self) -> Vec<bool> {
        utils::byte_to_bits(self.0)
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

// Mask patterns and functions was copied from https://github.com/kennytm/qrcode-rust,
// but i16 was replaced with u16.

/// The mask patterns. Since QR code and Micro QR code do not use the same
/// pattern number, we name them according to their shape instead of the number.
#[derive(Debug, Copy, Clone, Default)]
pub enum MaskPattern {
    /// QR code pattern 000: `(x + y) % 2 == 0`.
    #[default]
    Checkerboard = 0b000,

    /// QR code pattern 001: `y % 2 == 0`.
    HorizontalLines = 0b001,

    /// QR code pattern 010: `x % 3 == 0`.
    VerticalLines = 0b010,

    /// QR code pattern 011: `(x + y) % 3 == 0`.
    DiagonalLines = 0b011,

    /// QR code pattern 100: `((x/3) + (y/2)) % 2 == 0`.
    LargeCheckerboard = 0b100,

    /// QR code pattern 101: `(x*y)%2 + (x*y)%3 == 0`.
    Fields = 0b101,

    /// QR code pattern 110: `((x*y)%2 + (x*y)%3) % 2 == 0`.
    Diamonds = 0b110,

    /// QR code pattern 111: `((x+y)%2 + (x*y)%3) % 2 == 0`.
    Meadow = 0b111,
}

mod mask_functions {
    pub const fn checkerboard(x: u16, y: u16) -> bool {
        (x + y) % 2 == 0
    }
    pub const fn horizontal_lines(_: u16, y: u16) -> bool {
        y % 2 == 0
    }
    pub const fn vertical_lines(x: u16, _: u16) -> bool {
        x % 3 == 0
    }
    pub const fn diagonal_lines(x: u16, y: u16) -> bool {
        (x + y) % 3 == 0
    }
    pub const fn large_checkerboard(x: u16, y: u16) -> bool {
        ((y / 2) + (x / 3)) % 2 == 0
    }
    pub const fn fields(x: u16, y: u16) -> bool {
        (x * y) % 2 + (x * y) % 3 == 0
    }
    pub const fn diamonds(x: u16, y: u16) -> bool {
        ((x * y) % 2 + (x * y) % 3) % 2 == 0
    }
    pub const fn meadow(x: u16, y: u16) -> bool {
        ((x + y) % 2 + (x * y) % 3) % 2 == 0
    }
}

impl MaskPattern {
    pub fn get_function(self) -> fn(u16, u16) -> bool {
        match self {
            MaskPattern::Checkerboard => mask_functions::checkerboard,
            MaskPattern::HorizontalLines => mask_functions::horizontal_lines,
            MaskPattern::VerticalLines => mask_functions::vertical_lines,
            MaskPattern::DiagonalLines => mask_functions::diagonal_lines,
            MaskPattern::LargeCheckerboard => mask_functions::large_checkerboard,
            MaskPattern::Fields => mask_functions::fields,
            MaskPattern::Diamonds => mask_functions::diamonds,
            MaskPattern::Meadow => mask_functions::meadow,
        }
    }

    pub fn apply(self, x: u16, y: u16) -> bool {
        Self::get_function(self)(x, y)
    }
}

use super::tables;

mod canvas;
mod correction_level;
mod mask;
mod module;
mod version;

/// QR код в виде матрицы модулей
pub struct Canvas {
    modules: Vec<Vec<Module>>,
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
pub enum Module {
    Dark,
    Light,
    #[default]
    Unused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version(u8);

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

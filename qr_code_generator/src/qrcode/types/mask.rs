//! Mask patterns and functions was copied from https://github.com/kennytm/qrcode-rust,
//! but i16 was replaced with u16.

use super::MaskPattern;

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

use anyhow::Result;

mod coder;
mod gf;
mod reed_solomon;

pub use coder::{BlockCoder, Coder};
pub use reed_solomon::ReedSolomon;

/// Представление полинома в поле GF(256). Старший индекс - старший коэффициент.
type Poly = Vec<u8>;

/// Ссылочное представление полинома поля GF(256).
type RefPoly<'a> = &'a [u8];

pub fn new_reed_solomon_fast_gf256(control_count: usize) -> ReedSolomon<gf::FastGF256> {
    ReedSolomon::new(control_count, gf::FastGF256::new())
}

use anyhow::Result;

mod gf;
mod reed_solomon;

pub use reed_solomon::ReedSolomon;

trait Coder {
    fn encode(&self, data: &[u8]) -> Result<Vec<u8>>;
    fn decode(&self, data: &[u8]) -> Result<Vec<u8>>;
}

/// Представление полинома в поле GF(256). Старший индекс - старший коэффициент.
type Poly = Vec<u8>;

/// Ссылочное представление полинома поля GF(256).
type RefPoly<'a> = &'a [u8];

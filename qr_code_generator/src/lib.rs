use anyhow::Result;

mod gf;
mod reed_solomon;

pub use reed_solomon::ReedSolomon;

trait Coder {
    fn encode(&self, data: &[u8]) -> Result<Vec<u8>>;
    fn decode(&self, data: &[u8]) -> Result<Vec<u8>>;
}

type Poly = Vec<u8>;
type RefPoly<'a> = &'a [u8];

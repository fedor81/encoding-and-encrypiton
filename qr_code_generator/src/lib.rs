use anyhow::Result;
use std::path::Path;

pub mod barcode;
pub mod qrcode;
mod utils;

pub trait Drawable {
    /// Draws the code to the specified path.
    fn draw<P: AsRef<Path>>(&self, path: P) -> Result<()>;
}

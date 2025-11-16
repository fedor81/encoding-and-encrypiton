use anyhow::Result;
use image::{ImageBuffer, Luma};
use std::path::Path;

pub mod barcode;

fn draw_barcode<P: AsRef<Path>>(code: &[u8], path: P) -> Result<()> {
    let width = code.len() as u32;
    let height = (width / 3) as u32;

    let mut img = ImageBuffer::<Luma<u8>, Vec<u8>>::new(width, height);

    for i in 0..code.len() {
        // 0 - black, 1 - white
        let color = if code[i] == 0 { 255 } else { 0 };
        for y in 0..height {
            img.put_pixel(i as u32, y, Luma([color]));
        }
    }

    img.save(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::barcode::Code128;

    use super::*;

    #[test]
    fn test_draw_works() {
        let code = vec![1, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1, 1, 1];
        let path = "test_draw_works.png";
        assert!(draw_barcode(&code, path).is_ok());
        assert!(Path::new(path).exists());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_draw_barcode_works() {
        let code = Code128::encode_with_codeset("Hello World", barcode::CodeSet::B)
            .expect("Code128 doesn't work");
        let path = "test_draw_barcode_works.png";
        assert!(draw_barcode(&code, path).is_ok());
        assert!(Path::new(path).exists());
        std::fs::remove_file(path).unwrap();
    }
}

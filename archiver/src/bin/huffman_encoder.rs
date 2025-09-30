use anyhow::{Context, Result};

use archiver::{archive_by_haffman, io::read_filepath};

fn main() -> Result<()> {
    let target = read_filepath(&"Please enter the path to the file you want to archive:")?;
    let destination = read_filepath(
        &"Please enter the path where you would like the result of the operation to be saved:",
    )?;

    archive_by_haffman(target, destination)
}

#[cfg(test)]
mod tests {
    use super::*;

    use archiver::io::path_to_absolute;
    use std::path::PathBuf;

    #[test]
    fn test_huffman_encoder() {
        let target = path_to_absolute(PathBuf::from("./src/lib.rs")).unwrap();
        let destination = path_to_absolute(PathBuf::from("encoder_test.huff")).unwrap();

        std::fs::remove_file(&destination).ok();
        archive_by_haffman(target, destination.clone()).unwrap();

        std::fs::remove_file(destination).unwrap();
    }
}

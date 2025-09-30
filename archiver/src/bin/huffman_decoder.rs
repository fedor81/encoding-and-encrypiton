use std::path::PathBuf;

use archiver::{HuffmanArchiver, io::read_filepath};

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let target = read_filepath(&"Please provide the location of the file you wish to extract:")?;
    let destination = read_filepath(
        &"Please specify the location where you want to save the outcome of the operation:",
    )?;

    extract(target, destination)
}

/// Extracts the file from the archive
fn extract(target: PathBuf, destination: PathBuf) -> Result<()> {
    println!("Finish!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use archiver::{FileEncoder, create_probabilities_map, io::path_to_absolute};
    use std::path::PathBuf;

    #[test]
    fn test_main() {
        let target = path_to_absolute(PathBuf::from("../README.md")).unwrap();
        let destination = path_to_absolute(PathBuf::from("../README.huff")).unwrap();

        // Архивация файла
        let probabilities = create_probabilities_map(&target).unwrap();
        let encoder = HuffmanArchiver::new(probabilities);
        encoder.encode_file(&target, &destination).unwrap();

        // TODO: Распаковка файла

        // TODO: Сравнение
    }
}

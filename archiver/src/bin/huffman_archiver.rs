use std::path::PathBuf;

use archiver::{FileEncoder, HuffmanArchiver, create_probabilities_map, io::read_filepath};

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let target = read_filepath(&"Please enter the path to the file you want to archive:")?;
    let destination = read_filepath(
        &"Please enter the path where you would like the result of the operation to be saved:",
    )?;

    archive(target, destination)
}

/// Archives the file
fn archive(target: PathBuf, destination: PathBuf) -> Result<()> {
    println!("\nCreate probabilities map for file: {:?}", target);
    let probabilities =
        create_probabilities_map(&target).context("Failed to create probabilities map")?;

    let encoder = HuffmanArchiver::new(probabilities);

    println!("Encoding...");
    encoder.encode_file(&target, &destination)?;

    println!("Finish!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use archiver::io::path_to_absolute;
    use std::path::PathBuf;

    #[test]
    fn test_main() {
        let target = path_to_absolute(PathBuf::from("../README.md")).unwrap();
        let destination = path_to_absolute(PathBuf::from("../README.huff")).unwrap();

        archive(target, destination.clone()).unwrap();
        std::fs::remove_file(destination).unwrap();
    }
}

use archiver::{FileDecoder, HuffmanArchiver, io::read_filepath};

use anyhow::Result;

fn main() -> Result<()> {
    let target = read_filepath(&"Please provide the location of the file you wish to extract:")?;
    let destination =
        read_filepath(&"Please specify the location where you want to save the outcome of the operation:")?;

    HuffmanArchiver::decode_file(&target, &destination)
}

#[cfg(test)]
mod tests {
    use archiver::{FileDecoder, HuffmanArchiver, io::path_to_absolute, utils::cmp_files};
    use std::{fs, path::PathBuf};

    #[test]
    fn test_huffman_decoder() {
        let original = path_to_absolute(PathBuf::from("./src/lib.rs")).unwrap();
        let archived = path_to_absolute(PathBuf::from("decoder_test.huff")).unwrap();
        let extracted = archived.with_extension("extract");

        // Сборка мусора, если файлы существуют
        if fs::remove_file(&archived).is_ok() {
            println!("- Removed archived file: {}\n", archived.to_str().unwrap());
        };
        if fs::remove_file(&extracted).is_ok() {
            println!("- Removed extracted file: {}\n", extracted.to_str().unwrap());
        }

        // Архивация файла
        HuffmanArchiver::archive(&original, &archived).expect("Failed to archive file");

        // Распаковка файла
        HuffmanArchiver::decode_file(&archived, &extracted).expect("Failed to extract file");

        // Проверка (побайтово, без артефактов перевода строк)
        cmp_files(&original, &extracted);

        // Сборка мусора
        fs::remove_file(&archived).expect("Failed to remove archived file");
        fs::remove_file(&extracted).expect("Failed to remove extracted file");
    }
}

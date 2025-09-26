use std::{fs::File, io::Read, path::PathBuf};

/// Интерфейс для кодирования последовательности байтов в строку. Не потоковый!
pub trait Encoder {
    /// Преобразует целевую последовательность байтов в закодированную строку.
    fn convert_to_string(&self, bytes: &[u8]) -> String;

    /// Кодирует последовательность байтов.
    fn encode_bytes(&self, bytes: &[u8]) -> Vec<u8> {
        let bit_string = self.convert_to_string(bytes);
        Self::convert_to_bytes(bit_string)
    }

    fn convert_to_bytes(mut bit_string: String) -> Vec<u8> {
        // Дополняем до полного байта
        let padding = (8 - (bit_string.len() % 8)) % 8;
        for _ in 0..padding {
            bit_string.push('0');
        }

        // Преобразуем битовую строку в байты
        let mut encoded = Vec::new();
        for chunk in bit_string.as_bytes().chunks(8) {
            let mut byte = 0u8;
            for &b in chunk.into_iter() {
                if b == b'1' {
                    byte += 1u8;
                }
                byte <<= 1;
            }
            encoded.push(byte);
        }

        encoded
    }

    fn encode_file(&self, path: &PathBuf) -> Vec<u8> {
        let mut file = File::open(path).expect("Failed to open file");
        let size = file
            .metadata()
            .expect("Failed to extract file metadata")
            .len();
        let mut buf = vec![0u8; size as usize];
        file.read_to_end(&mut buf).expect("Failed to read file");

        self.encode_bytes(&buf)
    }
}

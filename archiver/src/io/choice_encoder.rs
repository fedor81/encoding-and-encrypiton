use crate::{Codes, CodesBuilder, HuffmanEncoder, ShannonFanoEncoder};

#[derive(Debug)]
pub enum EncoderChoice {
    ShannonFano(ShannonFanoEncoder),
    Huffman(HuffmanEncoder),
}

impl EncoderChoice {
    pub fn build_optimal_codes(&self, probabilities: Vec<f64>) -> Codes {
        let words = vec![0u8; probabilities.len()];
        match self {
            EncoderChoice::ShannonFano(_) => {
                ShannonFanoEncoder::build_optimal_codes(words, probabilities)
            }
            EncoderChoice::Huffman(_) => HuffmanEncoder::build_optimal_codes(words, probabilities),
        }
    }

    pub fn read_from_stdin() -> EncoderChoice {
        let stdin = std::io::stdin();
        let mut buf = String::new();

        println!("0 - shannon_fano");
        println!("1 - huffman");

        loop {
            print!("enter a number: ");
            buf.clear();
            stdin.read_line(&mut buf).expect("Failed to read line");

            if let Ok(algorithm) = buf.trim().parse() {
                match algorithm {
                    0 => return EncoderChoice::ShannonFano(ShannonFanoEncoder::new()),
                    1 => return EncoderChoice::Huffman(HuffmanEncoder::new(todo!())),
                    _ => {
                        println!("Entered unavailable number!");
                        continue;
                    }
                }
            } else {
                println!("Failed to parse number!");
                continue;
            }
        }
    }
}

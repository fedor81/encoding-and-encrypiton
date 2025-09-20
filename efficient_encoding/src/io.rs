use std::str::FromStr;

use crate::{Codes, CodesBuilder, HuffmanEncoder, ShannonFanoEncoder};

pub fn print_codes(name: &str, codes: &Codes) {
    println!("{}:", name);
    println!("Probabilities: {:?}", codes.probabilities());
    println!("Codes: {:?}", codes.codes());
    println!("Mean length: {}", codes.mean_code_length());
    println!(
        "Relative efficiency ratio: {}",
        codes.relative_efficiency_ratio()
    );
    println!(
        "Statistical compression ratio: {}",
        codes.statistical_compression_ratio()
    );
    print!("\n");
}

pub fn choice_encoder() -> Encoder {
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
                0 => return Encoder::ShannonFano(ShannonFanoEncoder::new()),
                1 => return Encoder::Huffman(HuffmanEncoder::new()),
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

#[derive(Debug)]
pub enum Encoder {
    ShannonFano(ShannonFanoEncoder),
    Huffman(HuffmanEncoder),
}

impl Encoder {
    pub fn build_optimal_codes(&self, probabilities: Vec<f64>) -> Codes {
        match self {
            Encoder::ShannonFano(_) => ShannonFanoEncoder::build_optimal_codes(probabilities),
            Encoder::Huffman(_) => HuffmanEncoder::build_optimal_codes(probabilities),
        }
    }
}

pub fn read_vec_numbers<T>() -> Vec<T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    let stdin = std::io::stdin();
    let mut buf = String::new();

    println!("Enter a vector of probabilities of characters appearing separated by a whitespace:");

    stdin.read_line(&mut buf).unwrap();
    return buf
        .trim()
        .split_whitespace()
        .map(|s| s.parse::<_>().unwrap())
        .collect::<Vec<_>>();
}

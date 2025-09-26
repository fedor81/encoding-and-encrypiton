use std::{path::PathBuf, str::FromStr};

use crate::Codes;

mod choice_encoder;
pub use choice_encoder::EncoderChoice;

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

pub fn read_vec_numbers<T>(output: &str) -> Vec<T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    let stdin = std::io::stdin();
    let mut buf = String::new();

    println!("\n{}", output);

    stdin.read_line(&mut buf).unwrap();
    return buf
        .trim()
        .split_whitespace()
        .map(|s| s.parse::<_>().unwrap())
        .collect::<Vec<_>>();
}

pub fn read_filepath(output: &str) -> PathBuf {
    let stdin = std::io::stdin();
    let mut buf = String::new();

    println!("\n{}", output);
    stdin.read_line(&mut buf).expect("Failed to read line");
    buf.into()
}

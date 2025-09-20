use std::{fmt::Debug, str::FromStr};

use efficient_encoding::{CodesBuilder, HuffmanEncoder, ShannonFanoEncoder};

fn main() {
    let mut encoder = choice_encoder();
    let numbers = read_vec_numbers();
    let codes = encoder.build_optimal_codes(numbers);
    // TODO: Определить среднюю длину кода, коэффициенты относительной эффективности и статистического сжатия.
    // TODO: Сравнить и обосновать различия в качественных характеристиках полученных кодов.
}

fn choice_encoder() -> Box<dyn CodesBuilder> {
    let stdin = std::io::stdin();
    let mut buf = String::new();

    println!("0 - shannon_fano");
    println!("1 - huffman");

    loop {
        print!("enter a number: ");
        stdin.read_line(&mut buf).expect("Failed to read line");

        if let Ok(algorithm) = buf.trim().parse() {
            match algorithm {
                0 => return Box::new(ShannonFanoEncoder::new()),
                1 => return Box::new(HuffmanEncoder::new()),
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

fn read_vec_numbers<T>() -> Vec<T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    let stdin = std::io::stdin();
    let mut buf = String::new();

    stdin.read_line(&mut buf).unwrap();
    return buf
        .trim()
        .split_whitespace()
        .map(|s| s.parse::<_>().unwrap())
        .collect::<Vec<_>>();
}

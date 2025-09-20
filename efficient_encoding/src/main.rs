use efficient_encoding::io::{print_codes, read_vec_numbers};
use efficient_encoding::{CodesBuilder, HuffmanEncoder, ShannonFanoEncoder};

fn main() {
    let probabilities = read_vec_numbers();
    print!("\n");

    let shannon_fano_codes = ShannonFanoEncoder::build_optimal_codes(probabilities.clone());
    print_codes("Shannon-Fano codes", &shannon_fano_codes);

    let huffman_codes = HuffmanEncoder::build_optimal_codes(probabilities);
    print_codes("Huffman codes", &huffman_codes);
}

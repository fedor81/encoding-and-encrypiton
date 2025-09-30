use archiver::io::{print_codes, read_vec_numbers};
use archiver::{CodesBuilder, HuffmanArchiver, ShannonFanoEncoder};

fn main() {
    let probabilities = read_vec_numbers(
        "Enter a vector of probabilities for characters appearing in a sequence separated by whitespace:",
    );
    print!("\n");

    let words = vec![0u8; probabilities.len()];

    let shannon_fano_codes = ShannonFanoEncoder::build_optimal_codes(
        words.iter().copied().collect(),
        probabilities.clone(),
    );
    print_codes("Shannon-Fano codes", &shannon_fano_codes);

    let huffman_codes = HuffmanArchiver::build_optimal_codes(words, probabilities);
    print_codes("Huffman codes", &huffman_codes);
}

mod huffman_tree;

use super::Codes;
use huffman_tree::build_tree;

#[derive(Debug, Default)]
pub struct HuffmanEncoder {}

impl HuffmanEncoder {
    pub fn new() -> Self {
        HuffmanEncoder {}
    }
}

impl HuffmanEncoder {
    pub fn build_optimal_codes(mut probabilities: Vec<f64>) -> Codes {
        probabilities.sort_by(|a, b| a.partial_cmp(b).unwrap().reverse());

        let tree = build_tree(&probabilities);
        let codes = tree.build_codes();

        Codes::new(probabilities, codes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_optimal_codes() {
        assert_eq!(
            vec!["0", "11", "10"],
            HuffmanEncoder::build_optimal_codes(vec![0.5, 0.25, 0.25]).codes
        );
    }
}

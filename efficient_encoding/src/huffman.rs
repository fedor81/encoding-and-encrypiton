use super::{Codes, CodesBuilder};

pub struct HuffmanEncoder {}

impl HuffmanEncoder {
    pub fn new() -> Self {
        HuffmanEncoder {}
    }
}

impl CodesBuilder for HuffmanEncoder {
    fn build_optimal_codes(&mut self, mut probabilities: Vec<f64>) -> Codes {
        probabilities.sort_by(|a, b| a.partial_cmp(b).unwrap().reverse());
        todo!()
    }
}

use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};

#[derive(Debug)]
pub enum HuffmanTree {
    Leaf {
        probability: f64,
        index: usize,
    },
    Node {
        probability: f64,
        left: Box<HuffmanTree>,
        right: Box<HuffmanTree>,
        count_codes: usize,
    },
}

impl HuffmanTree {
    pub fn build(probabilities: &[f64]) -> HuffmanTree {
        match probabilities.len() {
            0 => panic!("No probabilities provided"),
            1 => return HuffmanTree::new_leaf(probabilities[0], 0),
            _ => {}
        }

        let mut heap = probabilities
            .iter()
            .copied()
            .enumerate()
            .map(|(i, p)| HuffmanTree::new_leaf(p, i))
            .collect::<BinaryHeap<_>>();

        while heap.len() >= 2 {
            let item1 = heap.pop().unwrap();
            let item2 = heap.pop().unwrap();
            heap.push(HuffmanTree::unite(item1, item2));
        }
        heap.pop().unwrap()
    }

    pub fn new_leaf(probability: f64, index: usize) -> Self {
        HuffmanTree::Leaf { probability, index }
    }

    pub fn unite(left: HuffmanTree, right: HuffmanTree) -> Self {
        let probability = left.probability() + right.probability();
        let count_codes = left.count_codes() + right.count_codes();

        HuffmanTree::Node {
            probability,
            left: Box::new(left),
            right: Box::new(right),
            count_codes,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, HuffmanTree::Leaf { .. })
    }

    pub fn probability(&self) -> f64 {
        match self {
            HuffmanTree::Leaf { probability, .. } => *probability,
            HuffmanTree::Node { probability, .. } => *probability,
        }
    }

    pub fn index(&self) -> Option<usize> {
        match self {
            HuffmanTree::Leaf { index, .. } => Some(*index),
            HuffmanTree::Node { .. } => None,
        }
    }

    pub fn left(&self) -> Option<&Self> {
        match self {
            HuffmanTree::Node { left, .. } => Some(left),
            HuffmanTree::Leaf { .. } => None,
        }
    }

    pub fn right(&self) -> Option<&Self> {
        match self {
            HuffmanTree::Node { right, .. } => Some(right),
            HuffmanTree::Leaf { .. } => None,
        }
    }

    pub fn count_codes(&self) -> usize {
        match self {
            HuffmanTree::Leaf { .. } => 1,
            HuffmanTree::Node { count_codes, .. } => *count_codes,
        }
    }

    pub fn build_codes(&self) -> Vec<String> {
        let mut queue = VecDeque::new();
        queue.push_back((self, String::new()));

        let mut codes = vec![String::new(); self.count_codes()];

        while let Some((node, code)) = queue.pop_front() {
            match node {
                HuffmanTree::Leaf { index, .. } => {
                    codes[*index] = code;
                }
                HuffmanTree::Node { left, right, .. } => {
                    // Прямой доступ к полям без использования left()/right()
                    queue.push_back((left, format!("{}0", code)));
                    queue.push_back((right, format!("{}1", code)));
                }
            }
        }
        codes
    }

    // Альтернативная реализация build_codes с рекурсией
    pub fn build_codes_recursive(&self) -> Vec<String> {
        let mut codes = vec![String::new(); self.count_codes()];
        self.build_codes_helper(String::new(), &mut codes);
        codes
    }

    fn build_codes_helper(&self, current_code: String, codes: &mut Vec<String>) {
        match self {
            HuffmanTree::Leaf { index, .. } => {
                codes[*index] = current_code;
            }
            HuffmanTree::Node { left, right, .. } => {
                // Прямой доступ к полям
                left.build_codes_helper(format!("{}0", current_code), codes);
                right.build_codes_helper(format!("{}1", current_code), codes);
            }
        }
    }
}

impl PartialEq for HuffmanTree {
    fn eq(&self, other: &Self) -> bool {
        self.probability() == other.probability()
    }
}

impl Eq for HuffmanTree {}

impl PartialOrd for HuffmanTree {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.probability().partial_cmp(&self.probability())
    }
}

impl Ord for HuffmanTree {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

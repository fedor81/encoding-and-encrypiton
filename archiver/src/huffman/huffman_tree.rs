use std::collections::{BinaryHeap, VecDeque};

pub fn build_tree(probabilities: &[f64]) -> HuffmanTree {
    match probabilities.len() {
        0 => panic!("No probabilities"),
        1 => return HuffmanTree::new(probabilities[0], 0).unite(None),
        _ => {}
    };

    let mut heap = probabilities
        .iter()
        .copied()
        .enumerate()
        .map(|(i, p)| HuffmanTree::new(p, i))
        .collect::<BinaryHeap<_>>();

    while heap.len() >= 2 {
        let item1 = heap.pop().unwrap();
        let item2 = heap.pop().unwrap();
        heap.push(HuffmanTree::unite(item1, Some(item2)));
    }
    heap.pop().unwrap()
}

#[derive(Debug)]
pub struct HuffmanTree {
    left: Option<Box<HuffmanTree>>,
    right: Option<Box<HuffmanTree>>,
    probability: f64,
    index: Option<usize>,
    count_codes: usize,
}

impl HuffmanTree {
    pub fn new(probability: f64, index: usize) -> Self {
        Self {
            left: None,
            right: None,
            probability,
            index: Some(index),
            count_codes: 1,
        }
    }

    pub fn is_terminal(&self) -> bool {
        match (&self.left, &self.right) {
            (None, None) => true,
            _ => false,
        }
    }

    pub fn unite(self, other: Option<Self>) -> Self {
        if let Some(other) = other {
            Self {
                count_codes: self.count_codes + other.count_codes,
                probability: self.probability + other.probability,
                left: Some(Box::new(self)),
                right: Some(Box::new(other)),
                index: None,
            }
        } else {
            Self {
                count_codes: self.count_codes,
                probability: self.probability,
                left: Some(Box::new(self)),
                right: None,
                index: None,
            }
        }
    }

    pub fn probability(&self) -> f64 {
        self.probability
    }

    pub fn index(&self) -> Option<usize> {
        self.index
    }

    pub fn left(&self) -> Option<&Self> {
        match &self.left {
            Some(node) => Some(node),
            None => None,
        }
    }

    pub fn right(&self) -> Option<&Self> {
        match &self.right {
            Some(node) => Some(node),
            None => None,
        }
    }

    /// Показывает количество кодовых слов или листьев в дереве.
    pub fn count_codes(&self) -> usize {
        self.count_codes
    }

    pub fn build_codes(&self) -> Vec<String> {
        let mut queue = VecDeque::new();
        queue.push_back((self, String::new()));

        let mut codes = vec![String::new(); self.count_codes()];

        while let Some((node, code)) = queue.pop_front() {
            if node.is_terminal() {
                codes[node.index().expect("The terminal node must have an index")] = code;
            } else {
                if let Some(left) = node.left() {
                    queue.push_back((left, format!("{}0", code)));
                }
                if let Some(right) = node.right() {
                    queue.push_back((right, format!("{}1", code)));
                }
            }
        }
        codes
    }
}

impl PartialEq for HuffmanTree {
    fn eq(&self, other: &Self) -> bool {
        self.probability == other.probability
    }
}

impl Eq for HuffmanTree {}

impl PartialOrd for HuffmanTree {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.probability
            .partial_cmp(&other.probability)
            .map(|order| order.reverse())
    }
}

impl Ord for HuffmanTree {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

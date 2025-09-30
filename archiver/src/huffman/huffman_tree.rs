use anyhow::{Context, Result};
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, VecDeque},
    fmt,
};

pub enum HuffmanTree {
    Leaf {
        probability: f64,
        index: usize,
        word: u8,
    },
    Node {
        probability: f64,
        left: Box<HuffmanTree>,
        right: Box<HuffmanTree>,
        count_codes: usize,
    },
}

impl HuffmanTree {
    pub fn restore_from_word_code(codes: &HashMap<u8, String>) -> Result<Self> {
        if codes.is_empty() {
            anyhow::bail!("Cannot restore HuffmanTree: no codes provided");
        }

        let mut root = HuffmanTree::Node {
            probability: 0.0,
            left: Box::new(Self::Leaf {
                probability: 0.0,
                index: usize::MAX,
                word: 0,
            }),
            right: Box::new(HuffmanTree::Leaf {
                probability: 0.0,
                index: usize::MAX,
                word: 0,
            }),
            count_codes: codes.len(),
        };

        for (word, code) in codes {
            let mut current = &mut root;

            for bit in code.chars() {
                // Проверяем валидность бита
                if bit != '0' && bit != '1' {
                    anyhow::bail!(
                        "Invalid code for word {}: code '{}' contains invalid character '{}'",
                        word,
                        code,
                        bit
                    );
                }

                match current {
                    HuffmanTree::Node { left, right, .. } => {
                        let next_node = if bit == '0' { left } else { right };

                        // Если достигли конца кода, создаем лист
                        if next_node.is_empty_leaf() {
                            *next_node = Box::new(HuffmanTree::new_empty_node());
                        }

                        // Безопасно переходим к следующему узлу
                        current = next_node.as_mut();
                    }
                    HuffmanTree::Leaf { .. } => {
                        anyhow::bail!("Invalid code structure: prefix conflict for word {}", word);
                    }
                }
            }

            // Заменяем конечный узел на лист с данными
            *current = Self::Leaf {
                probability: 0.0,
                index: *word as usize,
                word: *word,
            };
        }

        Ok(root)
    }

    fn new_empty_node() -> Self {
        HuffmanTree::Node {
            probability: 0.0,
            left: Box::new(Self::Leaf {
                probability: 0.0,
                index: usize::MAX,
                word: 0,
            }),
            right: Box::new(Self::Leaf {
                probability: 0.0,
                index: usize::MAX,
                word: 0,
            }),
            count_codes: 0,
        }
    }

    fn is_empty_leaf(&self) -> bool {
        matches!(self, HuffmanTree::Leaf { index, .. } if *index == usize::MAX)
    }

    pub fn build(probabilities: &[f64], words: &[u8]) -> HuffmanTree {
        match probabilities.len() {
            0 => panic!("No probabilities provided"),
            1 => return HuffmanTree::new_leaf(probabilities[0], 0, words[0]),
            _ => {}
        }

        let mut heap = words
            .iter()
            .copied()
            .zip(probabilities.iter().copied().enumerate())
            .map(|(word, (idx, prob))| HuffmanTree::new_leaf(prob, idx, word))
            .collect::<BinaryHeap<_>>();

        while heap.len() >= 2 {
            let item1 = heap.pop().unwrap();
            let item2 = heap.pop().unwrap();
            heap.push(HuffmanTree::unite(item1, item2));
        }
        heap.pop().unwrap()
    }

    pub fn new_leaf(probability: f64, index: usize, word: u8) -> Self {
        HuffmanTree::Leaf {
            probability,
            index,
            word,
        }
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

    pub fn is_leaf(&self) -> bool {
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

    // Общий метод, который возвращает итератор пар (ключ, код)
    fn build_code_pairs(&self) -> Vec<(usize, String, u8)> {
        let mut pairs = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back((self, String::new()));

        while let Some((node, code)) = queue.pop_front() {
            match node {
                HuffmanTree::Leaf { index, word, .. } => {
                    pairs.push((*index, code.clone(), *word));
                }
                HuffmanTree::Node { left, right, .. } => {
                    queue.push_back((left, format!("{}0", code)));
                    queue.push_back((right, format!("{}1", code)));
                }
            }
        }
        pairs
    }

    pub fn build_word_code(&self) -> HashMap<u8, String> {
        self.build_code_pairs()
            .into_iter()
            .map(|(_, code, word)| (word, code))
            .collect()
    }

    pub fn build_codes(&self) -> Vec<String> {
        let mut codes = vec![String::new(); self.count_codes()];

        for (index, code, _) in self.build_code_pairs() {
            codes[index] = code;
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

    fn fmt_with_indent(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        let indent_str = "  ".repeat(indent);

        match self {
            HuffmanTree::Leaf {
                word,
                index,
                probability,
            } => {
                write!(
                    f,
                    "Leaf {{ word: {}, index: {}, prob: {:.4} }}",
                    word, index, probability
                )
            }
            HuffmanTree::Node {
                probability,
                left,
                right,
                count_codes,
            } => {
                writeln!(f, "Node {{")?;
                writeln!(f, "{}  probability: {:.4}", indent_str, probability)?;
                writeln!(f, "{}  count_codes: {}", indent_str, count_codes)?;
                write!(f, "{}  left: ", indent_str)?;
                left.fmt_with_indent(f, indent + 1)?;
                writeln!(f, ",")?;
                write!(f, "{}  right: ", indent_str)?;
                right.fmt_with_indent(f, indent + 1)?;
                write!(f, "\n{}}}", indent_str)
            }
        }
    }
}

impl PartialEq for HuffmanTree {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Node {
                    probability: self_probability,
                    left: self_left,
                    right: self_right,
                    count_codes: self_count_codes,
                },
                Self::Node {
                    probability: other_probability,
                    left: other_left,
                    right: other_right,
                    count_codes: other_count_codes,
                },
            ) => {
                self_count_codes == other_count_codes
                    && self_probability == other_probability
                    && self_left.eq(other_left)
                    && self_right.eq(other_right)
            }
            (
                Self::Leaf {
                    probability: self_probability,
                    index: self_index,
                    word: self_word,
                },
                Self::Leaf {
                    probability: other_probability,
                    index: other_index,
                    word: other_word,
                },
            ) => {
                self_probability == other_probability
                    && self_index == other_index
                    && self_word == other_word
            }
            (_, _) => false,
        }
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

impl fmt::Debug for HuffmanTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_indent(f, 0)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub fn new_test_tree() -> HuffmanTree {
        HuffmanTree::Node {
            probability: 0.0,
            left: HuffmanTree::new_leaf(0.0, 0, 0).into(),
            right: HuffmanTree::Node {
                probability: 0.0,
                right: HuffmanTree::new_leaf(0.0, 1, 1).into(),
                left: HuffmanTree::Node {
                    probability: 0.0,
                    left: HuffmanTree::new_leaf(0.0, 2, 2).into(),
                    right: HuffmanTree::Node {
                        probability: 0.0,
                        left: HuffmanTree::new_leaf(0.0, 3, 3).into(),
                        right: HuffmanTree::new_leaf(0.0, 4, 4).into(),
                        count_codes: 0,
                    }
                    .into(),
                    count_codes: 0,
                }
                .into(),
                count_codes: 0,
            }
            .into(),
            count_codes: 5,
        }
    }

    pub fn new_test_codes() -> HashMap<u8, String> {
        HashMap::from([
            (0u8, "0".into()),
            (1, "11".into()),
            (2, "100".into()),
            (3, "1010".into()),
            (4, "1011".into()),
        ])
    }

    #[test]
    fn test_restore_from_word_code() {
        let word_code = new_test_codes();
        let tree = HuffmanTree::restore_from_word_code(&word_code).unwrap();

        assert_eq!(tree.build_word_code(), word_code);
        assert_eq!(tree, new_test_tree());
    }
}

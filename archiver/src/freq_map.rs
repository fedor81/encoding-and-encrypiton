use std::collections::HashMap;

pub struct FrequencyMap {
    hashmap: HashMap<u8, usize>,
    total: usize,
}

/// Накопитель таблицы частот
impl FrequencyMap {
    pub fn new() -> Self {
        Self {
            hashmap: HashMap::new(),
            total: 0,
        }
    }

    pub fn build(&mut self) -> HashMap<u8, f64> {
        self.hashmap
            .iter()
            .map(|(byte, count)| (*byte, *count as f64 / self.total as f64))
            .collect()
    }

    pub fn consume(&mut self, buf: &[u8]) {
        self.total += buf.len();
        for byte in buf {
            if let Some(count) = self.hashmap.get_mut(byte) {
                *count += 1;
            } else {
                self.hashmap.insert(*byte, 1);
            }
        }
    }
}

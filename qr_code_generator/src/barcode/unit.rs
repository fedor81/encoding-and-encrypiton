use super::{CHARS, Encoding};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Unit {
    index: usize,
}

impl Unit {
    pub fn encoding(self) -> Encoding {
        CHARS[self.index].1
    }

    pub fn index(self) -> usize {
        self.index
    }
}

impl From<usize> for Unit {
    fn from(index: usize) -> Self {
        if index < CHARS.len() {
            Unit { index }
        } else {
            panic!("Available units from 0 to {}", CHARS.len())
        }
    }
}

impl From<Encoding> for Unit {
    fn from(pattern: Encoding) -> Self {
        match CHARS.iter().position(|charset| charset.1 == pattern) {
            Some(index) => Unit { index },
            None => panic!("CHARS does not contains code: {:?}", pattern),
        }
    }
}

impl std::fmt::Debug for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Unit").field(&self.index).finish()
    }
}

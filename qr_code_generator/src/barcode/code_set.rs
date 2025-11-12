use anyhow::Result;

use super::{CHARS, Unit};

/// Три набора символов (A, B, C):
/// - `Code Set A` — символы с кодами 0–95: A-Z, 0-9, специальные символы и FNC 1-4
/// - `Code Set B` — ASCII символы с кодами 32–127: A-Z, a-z, 0-9, специальные символы и FNC 1-4
/// - `Code Set C` — используется для парных цифр (00–99). Позволяет компактно кодировать числа: две цифры кодируются одним символом
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeSet {
    A,
    B,
    C,
}

impl Default for CodeSet {
    fn default() -> Self {
        Self::B
    }
}

impl TryFrom<char> for CodeSet {
    type Error = anyhow::Error;

    fn try_from(value: char) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            'À' => CodeSet::A,
            'Ɓ' => CodeSet::B,
            'Ć' => CodeSet::C,
            _ => anyhow::bail!(
                "Cannot define CodeSet by character: {}.\
                CodeSet can be defined by one of the following: À, Ɓ, Ć",
                value
            ),
        })
    }
}

impl CodeSet {
    pub fn index(self) -> usize {
        match self {
            CodeSet::A => 0,
            CodeSet::B => 1,
            CodeSet::C => 2,
        }
    }

    pub fn parse(self, pattern: &str) -> Result<Unit> {
        let set_index = self.index();

        match CHARS
            .iter()
            .position(|charset| charset.0[set_index] == pattern)
        {
            Some(index) => Ok(Unit::from(index)),
            None => anyhow::bail!("CodeSet::{:?} does not contains char: {}", self, pattern),
        }
    }

    pub fn start_unit(self) -> Unit {
        match self {
            CodeSet::A => Unit::from(103),
            CodeSet::B => Unit::from(104),
            CodeSet::C => Unit::from(105),
        }
    }
}

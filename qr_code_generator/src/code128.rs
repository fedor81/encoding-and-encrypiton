//! Code 128:
//!
//! - Кодирует любые символы ASCII (цифры, буквы, специальные символы).
//! - Очень высокая плотность — помещается больше информации в меньшее пространство.
//! - Нет необходимости в регистрации — можно использовать произвольные данные.
//! - Поддерживается большинством сканеров.
//! - Широко используется в логистике, складской системе, упаковке.

use anyhow::Result;

mod characters;

use characters::{CHARS, Encoding, STOP, TERM};

/// # Общие принципы
///
/// 1. Кодирует символы ASCII:
///   - Поддерживает символы с кодами от 0 до 127 (все символы ASCII).
///   - Включает цифры, буквы, знаки препинания, управляющие символы (например, FNC1, FNC2 и др.).
///
/// 2. Три набора символов (A, B, C):
///   - `Code Set A` — символы с кодами 0–95 (A-Z, 0-9, специальные символы и FNC 1-4).
///   - `Code Set B` — ASCII символы с кодами 32–127 (A-Z, a-z, 0-9, специальные символы и FNC 1-4).
///   - `Code Set C` — используется для парных цифр (00–99). Позволяет компактно кодировать числа: две цифры кодируются одним символом.
///
/// 3. Переключение между наборами:
///   - Можно переключаться между наборами внутри одного штрих-кода с помощью специальных символов:
///       - `CODE A, CODE B, CODE C` — переключение на постоянной основе.
///
/// # Структура штрих-кода Code 128
///
/// 1. Стартовый символ:
///   - Определяет набор, с которого начинается кодирование (START A, START B, START C).
///   - Каждый стартовый символ имеет своё значение и влияет на интерпретацию следующих данных.
/// 2. Данные:
///   - Последовательность символов, закодированных в соответствии с текущим набором.
///   - Если используется Set C, числа группируются по 2 цифры.
/// 3. Контрольная сумма (Checksum):
///   - Вычисляется по специальному алгоритму, включает в себя все символы данных и стартовый символ.
///   - Обеспечивает проверку целостности данных при сканировании.
/// 4. Стоп-символ:
///   - Обозначает окончание штрих-кода.
///   - Одинаков для всех наборов.
///
/// # Особенности
///
/// - Каждый символ (включая стартовый, данные и контрольную сумму) представлен шаблоном штрихов и пробелов.
/// - Всего в символе 6 элементов (3 штриха и 3 пробела).
/// - Ширина элементов может быть 1, 2, 3 или 4 модуля.
/// - Общая ширина всех 6 элементов всегда равна 11 модулям.
struct Code128;

#[derive(Clone, Copy, PartialEq, Eq)]
struct Unit {
    index: usize,
}

impl std::fmt::Debug for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Unit").field(&self.index).finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CodeSet {
    A,
    B,
    C,
}

impl Code128 {
    pub fn encode(data: &str) -> Result<Vec<u8>> {
        let encoded = Self::_encode(data)?;
        Ok(Self::convert(
            encoded.into_iter().map(|unit| unit.encoding()),
        ))
    }

    fn _encode(data: &str) -> Result<Vec<Unit>> {
        let mut encoded = Vec::new();

        // Определение стартового набора
        let first_char = data.chars().next().unwrap();
        let mut codeset = CodeSet::try_from(first_char)?;

        encoded.push(
            codeset
                .parse(&format!("START-{}", first_char))
                .expect("If you managed to parse the CodeSet, then this is a valid char"),
        );

        let mut carry = None;

        for current in data.chars().skip(1) {
            let unit;

            match current {
                // Управление набором
                'À' | 'Ɓ' | 'Ć' => {
                    unit = codeset.parse(current.to_string().as_str())?;
                    codeset = CodeSet::try_from(current)?;
                }

                // Парные цифры
                current if current.is_ascii_digit() && codeset == CodeSet::C => match carry {
                    None => {
                        carry = Some(current);
                        continue;
                    }
                    Some(prev) => {
                        unit = codeset.parse(&format!("{}{}", prev, current))?;
                        carry = None;
                    }
                },

                // Простое кодирование
                _ => unit = codeset.parse(current.to_string().as_str())?,
            }

            encoded.push(unit);
        }

        if let Some(carry) = carry {
            anyhow::bail!("Last carry is not empty: {}, CodeSet::{:?}", carry, codeset);
        }

        // Завершение: контрольная сумма и стоп символ
        encoded.push(Self::checksum(&encoded));
        encoded.push(codeset.parse("STOP")?);

        Ok(encoded)
    }

    fn convert<I>(data: I) -> Vec<u8>
    where
        I: IntoIterator<Item = Encoding>,
    {
        let mut result = Vec::new();
        for code in data {
            result.extend_from_slice(&code);
        }
        result
    }

    fn checksum(data: &[Unit]) -> Unit {
        Unit {
            index: data
                .iter()
                .map(|unit| unit.index)
                .enumerate()
                .map(|(position, code_idx)| code_idx * (position + 1))
                .sum::<usize>()
                % 103,
        }
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
    fn index(self) -> usize {
        match self {
            CodeSet::A => 0,
            CodeSet::B => 1,
            CodeSet::C => 2,
        }
    }

    fn parse(self, pattern: &str) -> Result<Unit> {
        let set_index = self.index();

        match CHARS
            .iter()
            .position(|charset| charset.0[set_index] == pattern)
        {
            Some(index) => Ok(Unit { index }),
            None => anyhow::bail!("CodeSet::{:?} does not contains char: {}", self, pattern),
        }
    }
}

impl Unit {
    fn encoding(self) -> Encoding {
        CHARS[self.index].1
    }

    pub fn new(index: usize) -> Self {
        Self { index }
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(CodeSet::A, "A", [1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0])]
    #[case(CodeSet::B, "A", [1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0])]
    #[case(CodeSet::C, "33", [1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0])]
    #[case(CodeSet::B, "*", [1, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0])]
    #[case(CodeSet::C, "52", [1, 1, 0, 1, 1, 1, 0, 0, 0, 1, 0])]
    fn codeset_encode(#[case] codeset: CodeSet, #[case] input: &str, #[case] expected: Encoding) {
        assert_eq!(codeset.parse(input).unwrap().encoding(), expected);
    }

    #[rstest]
    #[case("ÀABCDEF", vec![
        [1, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0], // START-A 103
        [1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0], // 33
        [1, 0, 0, 0, 1, 0, 1, 1, 0, 0, 0], // 34
        [1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0], // 35
        [1, 0, 1, 1, 0, 0, 0, 1, 0, 0, 0], // 36
        [1, 0, 0, 0, 1, 1, 0, 1, 0, 0, 0], // 37
        [1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0], // 38
        [1, 1, 0, 1, 0, 0, 0, 1, 1, 1, 0], // Checksum = 49
        [1, 1, 0, 0, 0, 1, 1, 1, 0, 1, 0]] // STOP
    )]
    #[case("Ɓ0( \\q\u{017B}", vec![
        [1, 1, 0, 1, 0, 0, 1, 0, 0, 0, 0], // START-B 104
        [1, 0, 0, 1, 1, 1, 0, 1, 1, 0, 0], // 16 - 0 
        [1, 0, 0, 0, 1, 1, 0, 0, 1, 0, 0], // 8 - ( 
        [1, 1, 0, 1, 1, 0, 0, 1, 1, 0, 0], // 0 - 
        [1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 0], // 60 - \ 
        [1, 0, 0, 1, 0, 1, 1, 1, 1, 0, 0], // 81 - q 
        [1, 0, 1, 1, 1, 1, 0, 0, 0, 1, 0], // 96 - \u{017B} 
        [1, 0, 0, 0, 0, 1, 1, 0, 1, 0, 0], // Checksum = 73
        [1, 1, 0, 0, 0, 1, 1, 1, 0, 1, 0]] // STOP
    )]
    #[case("Ć4609345677", vec![
        [1, 1, 0, 1, 0, 0, 1, 1, 1, 0, 0], // START-C 105
        [1, 0, 1, 1, 1, 0, 0, 0, 1, 1, 0], // 46
        [1, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0], // 09
        [1, 0, 0, 0, 1, 0, 1, 1, 0, 0, 0], // 34
        [1, 1, 1, 0, 0, 0, 1, 0, 1, 1, 0], // 56 
        [1, 1, 1, 1, 0, 1, 1, 1, 0, 1, 0], // 77
        [1, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0], // Checksum = 72
        [1, 1, 0, 0, 0, 1, 1, 1, 0, 1, 0]] // STOP
    )]
    #[case("À\u{0000}\u{017A}Ć6369Ɓl`", vec![ // CodeSet switching
        [1, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0], // 103 - START-A
        [1, 0, 1, 0, 0, 0, 0, 1, 1, 0, 0], // 64 - \u{0000}
        [1, 1, 1, 1, 0, 1, 0, 1, 0, 0, 0], // 97 - \u{017A}
        [1, 0, 1, 1, 1, 0, 1, 1, 1, 1, 0], // 99 - Switch C
        [1, 0, 1, 0, 0, 1, 1, 0, 0, 0, 0], // 63
        [1, 0, 1, 1, 0, 0, 1, 0, 0, 0, 0], // 69
        [1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 0], // 100 - Switch B
        [1, 1, 0, 0, 1, 0, 1, 0, 0, 0, 0], // 76 - l
        [1, 0, 1, 0, 0, 0, 0, 1, 1, 0, 0], // 64 - `
        [1, 1, 1, 0, 0, 1, 1, 0, 0, 1, 0], // Checksum = 29
        [1, 1, 0, 0, 0, 1, 1, 1, 0, 1, 0]] // STOP
    )]
    fn code128_encode(#[case] input: &str, #[case] expected: Vec<Encoding>) {
        let actual = Code128::_encode(input)
            .unwrap()
            .into_iter()
            .map(|unit| unit.index)
            .collect::<Vec<_>>();
        let expected = expected
            .iter()
            .copied()
            .map(|encoding| Unit::from(encoding).index)
            .collect::<Vec<_>>();

        assert_eq!(expected, actual);
    }
}

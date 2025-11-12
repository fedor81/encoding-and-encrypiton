use anyhow::Result;

mod characters;
mod code_set;
mod unit;

use characters::{CHARS, CODE_LEN, Encoding};
use code_set::CodeSet;
use unit::Unit;

/// # Code 128
///
/// - Кодирует любые символы ASCII (цифры, буквы, специальные символы).
/// - Очень высокая плотность — помещается больше информации в меньшее пространство.
/// - Нет необходимости в регистрации — можно использовать произвольные данные.
/// - Поддерживается большинством сканеров.
/// - Широко используется в логистике, складской системе, упаковке.
///
/// Три набора символов (A, B, C):
///   - `Code Set A` — символы с кодами 0–95: A-Z, 0-9, специальные символы и FNC 1-4.
///   - `Code Set B` — ASCII символы с кодами 32–127: A-Z, a-z, 0-9, специальные символы и FNC 1-4.
///   - `Code Set C` — используется для парных цифр (00–99). Позволяет компактно кодировать числа: две цифры кодируются одним символом.
///
/// Переключение между наборами:
///   - Можно переключаться между наборами внутри одного штрих-кода с помощью специальных символов:
///       - `À | Ɓ | Ć` — переключение на постоянной основе.
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
/// # Examples
///
/// ```
/// use qr_code_generator::barcode::{Code128, CodeSet};
///
/// Code128::encode("ƁHello World").unwrap();
/// Code128::encode_with_codeset("Hello World", CodeSet::B).unwrap();
/// ```
///
#[derive(Debug, Clone, Default)]
pub struct Code128<'a> {
    data: &'a str,
    encoded: Vec<Unit>,
    codeset: CodeSet,
}

impl<'a> Code128<'a> {
    /// # Important
    /// Предполагается, что стартовый символ не включен в data!
    fn new(data: &'a str, codeset: CodeSet) -> Self {
        Self {
            data,
            encoded: vec![codeset.start_unit()],
            codeset,
        }
    }

    /// Определение стартового набора по первому символу и его кодирование
    fn build_from_first_char(data: &'a str) -> Result<Self> {
        let first_char = data.chars().next().unwrap();
        let codeset = CodeSet::try_from(first_char)?;
        let coder = Self::new(&data[2..], codeset); // Обрезаем первый символ: À | Ɓ | Ć
        Ok(coder)
    }

    /// Автоматически определяет набор по первому символу и кодирует данные
    pub fn encode(data: &'a str) -> Result<Vec<u8>> {
        let coder = Self::build_from_first_char(data)?;
        coder.encode_and_convert()
    }

    /// Кодирует сообщение начиная с определённого набора
    pub fn encode_with_codeset(data: &'a str, codeset: CodeSet) -> Result<Vec<u8>> {
        let coder = Self::new(data, codeset);
        coder.encode_and_convert()
    }

    /// Кодирует сообщение и конвертирует в сплошной вектор
    fn encode_and_convert(self) -> Result<Vec<u8>> {
        let encoded = self.encode_payload()?;
        Ok(Self::convert_to_result(
            encoded.into_iter().map(|unit| unit.encoding()),
        ))
    }

    /// # Предупреждение
    /// Предполагается, что стартовый символ не включен в сообщение!
    fn encode_payload(mut self) -> Result<Vec<Unit>> {
        let mut carry = None;

        for current in self.data.chars() {
            let unit;

            match current {
                // Управление набором
                'À' | 'Ɓ' | 'Ć' => {
                    unit = self.codeset.parse(current.to_string().as_str())?;
                    self.codeset = CodeSet::try_from(current)?;
                }

                // Парные цифры
                current if current.is_ascii_digit() && self.codeset == CodeSet::C => match carry {
                    None => {
                        carry = Some(current);
                        continue;
                    }
                    Some(prev) => {
                        unit = self.codeset.parse(&format!("{}{}", prev, current))?;
                        carry = None;
                    }
                },

                // Простое кодирование
                _ => unit = self.codeset.parse(current.to_string().as_str())?,
            }

            self.encoded.push(unit);
        }

        if let Some(carry) = carry {
            anyhow::bail!(
                "Last carry is not empty: {}, CodeSet::{:?}",
                carry,
                self.codeset
            );
        }

        // Завершение: контрольная сумма и стоп символ
        self.encoded.push(Self::checksum(&self.encoded));
        self.encoded.push(self.codeset.parse("STOP")?);

        Ok(self.encoded)
    }

    fn convert_to_result<I>(data: I) -> Vec<u8>
    where
        I: IntoIterator<Item = Encoding>,
    {
        let mut result = Vec::new();
        for code in data {
            result.extend_from_slice(&code);
        }
        result
    }

    #[cfg(test)]
    fn convert_to_units(data: &[u8]) -> Vec<Unit> {
        if data.len() % CODE_LEN != 0 {
            panic!()
        }

        let mut result = Vec::with_capacity(data.len() / CODE_LEN);
        let mut encoding = [0u8; CODE_LEN];

        for i in 0..(data.len() / CODE_LEN) {
            for j in 0..CODE_LEN {
                encoding[j] = data[i * CODE_LEN + j]
            }
            result.push(Unit::from(encoding));
        }
        result
    }

    fn checksum(data: &[Unit]) -> Unit {
        Unit::from(
            data.iter()
                .map(|unit| unit.index())
                .enumerate()
                .map(|(position, code_idx)| code_idx * (position + 1))
                .sum::<usize>()
                % 103,
        )
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
    fn code128_encode_payload(#[case] input: &str, #[case] expected: Vec<Encoding>) {
        let actual = Code128::build_from_first_char(input)
            .unwrap()
            .encode_payload()
            .unwrap()
            .into_iter()
            .map(|unit| unit.index())
            .collect::<Vec<_>>();
        let expected = expected
            .iter()
            .copied()
            .map(|encoding| Unit::from(encoding).index())
            .collect::<Vec<_>>();

        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case("À\u{0000}\u{017A}Ć6369Ɓl`", vec![103, 64, 97, 99, 63, 69, 100, 76, 64, 29, 106], None)]
    #[case("\u{0000}\u{017A}", vec![103, 64, 97, 7, 106], Some(CodeSet::A))]
    fn code128_test(
        #[case] input: &str,
        #[case] expected: Vec<usize>,
        #[case] codeset: Option<CodeSet>,
    ) {
        let actual = if let Some(codeset) = codeset {
            Code128::encode_with_codeset(input, codeset)
        } else {
            Code128::encode(input)
        }
        .unwrap();

        let actual = Code128::convert_to_units(&actual)
            .into_iter()
            .map(|unit| unit.index())
            .collect::<Vec<_>>();

        assert_eq!(expected, actual, "start codeset = {:?}", codeset);
    }
}

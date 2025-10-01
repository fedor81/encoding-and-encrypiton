use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Codes {
    probabilities: Vec<f64>,
    codes: Vec<String>,
    words: Vec<u8>,
}

impl Codes {
    pub fn probabilities(&self) -> &[f64] {
        &self.probabilities
    }

    pub fn codes(&self) -> &[String] {
        &self.codes
    }

    pub fn words(&self) -> &[u8] {
        &self.words
    }

    pub fn new(words: Vec<u8>, probabilities: Vec<f64>, codes: Vec<String>) -> Self {
        assert_eq!(words.len(), probabilities.len());
        assert_eq!(words.len(), codes.len());

        Self {
            probabilities,
            codes,
            words,
        }
    }

    /// Вычисляет среднюю длину кодовой комбинации по формуле:
    ///
    /// `Σ(pi * ki)`.
    ///
    /// `pi` - вероятность появления символа в тексте.
    /// `ki` - длина кодовой комбинации для символа.
    pub fn mean_code_length(&self) -> f64 {
        let mut mean = 0.0;
        for i in 0..self.codes.len() {
            mean += self.codes[i].len() as f64 * self.probabilities[i];
        }
        mean
    }

    /// Вычисляет коэффициент статистического сжатия, который характеризует уменьшение количества двоичных
    /// знаков на символ сообщения при применении ОНК по сравнению с применением методов нестатического
    /// кодирования по формуле:
    ///
    /// `Hmax / Σ(pi * ki)`
    ///
    /// `Hmax` - длина кода при применении методов нестатистического кодирования.
    /// `pi` - вероятность появления символа в тексте.
    /// `ki` - длина кодовой комбинации для символа.
    pub fn statistical_compression_ratio(&self) -> f64 {
        (self.codes.len() as f64).log2().ceil() / self.mean_code_length()
    }

    /// Вычисляет коэффициент относительной эффективности, который показывает, насколько используется
    /// статистическая избыточ-ость передаваемого сообщения. по формуле:
    ///
    /// `H / Σ(pi * ki)`
    ///
    /// `H` - энтропия.
    /// `pi` - вероятность появления символа в тексте.
    /// `ki` - длина кодовой комбинации для символа.
    pub fn relative_efficiency_ratio(&self) -> f64 {
        self.entropy() / self.mean_code_length()
    }

    /// Вычисляет энтропию по формуле: `-Σ(pi * log2(pi))`
    pub fn entropy(&self) -> f64 {
        -(self
            .probabilities
            .iter()
            .map(|&p| p * p.log2())
            .sum::<f64>())
    }
}

impl Into<HashMap<u8, String>> for &Codes {
    fn into(self) -> HashMap<u8, String> {
        let mut word_code = HashMap::new();
        for (&word, code) in self.words.iter().zip(self.codes.iter()) {
            if word_code.insert(word, code.clone()).is_some() {
                panic!("Duplicate code for word: {}", word);
            }
        }
        word_code
    }
}

impl Into<HashMap<u8, String>> for Codes {
    fn into(self) -> HashMap<u8, String> {
        let mut word_code = HashMap::new();
        for (word, code) in self.words.into_iter().zip(self.codes.into_iter()) {
            if word_code.insert(word, code).is_some() {
                panic!("Duplicate code for word: {}", word);
            }
        }
        word_code
    }
}

#[cfg(test)]
mod tests {
    use super::Codes;

    // Вспомогательная функция для преобразования &str в Vec<String>
    fn str_vec(strings: &[&str]) -> Vec<String> {
        strings.iter().map(|&s| s.to_string()).collect()
    }

    fn codes_without_words(probabilities: Vec<f64>, codes: &[&str]) -> Codes {
        Codes::new(vec![0; probabilities.len()], probabilities, str_vec(codes))
    }

    #[test]
    fn test_mean_code_length() {
        let codes = codes_without_words(vec![0.25, 0.25, 0.25, 0.25], &["00", "01", "10", "11"]);
        assert_eq!(2.0, codes.mean_code_length());

        let codes = codes_without_words(vec![0.5, 0.25, 0.125, 0.125], &["0", "10", "110", "111"]);
        assert_eq!(1.75, codes.mean_code_length());

        let codes = codes_without_words(vec![0.4, 0.3, 0.2, 0.1], &["0", "10", "110", "1110"]);
        assert_eq!(2.0, codes.mean_code_length());

        let codes = codes_without_words(vec![0.5, 0.5], &["0", "1"]);
        assert_eq!(1.0, codes.mean_code_length());

        let codes = codes_without_words(vec![0.5, 0.4, 0.1], &["000", "1110000", "0001111"]);
        assert!((codes.mean_code_length() - 5.0).abs() < 0.000001);

        let codes = codes_without_words(vec![0.8, 0.1, 0.1], &["0", "10", "11"]);
        assert_eq!(1.2, codes.mean_code_length());
    }

    #[test]
    fn test_entropy() {
        let codes = codes_without_words(vec![0.25, 0.25, 0.25, 0.25], &["00", "01", "10", "11"]);
        assert_eq!(2.0, codes.entropy());

        let codes = codes_without_words(vec![0.5, 0.25, 0.25], &["0", "10", "11"]);
        assert_eq!(1.5, codes.entropy());

        let codes = codes_without_words(vec![1.0], &["0"]);
        assert_eq!(0.0, codes.entropy());

        let codes = codes_without_words(vec![0.5], &["0"]);
        assert_eq!(0.5, codes.entropy());

        let codes = codes_without_words(vec![0.5, 0.5], &["0", "1"]);
        assert_eq!(1.0, codes.entropy());

        let codes = codes_without_words(vec![0.5, 0.4, 0.1], &["000", "1110000", "0001111"]);
        assert!((codes.entropy() - 1.36).abs() < 0.001);
    }

    #[test]
    fn test_relative_efficiency_ratio() {
        let codes = codes_without_words(vec![0.5, 0.5], &["0", "1"]);
        assert_eq!(1.0, codes.relative_efficiency_ratio());

        let codes = codes_without_words(vec![0.5, 0.4, 0.1], &["000", "1110000", "0001111"]);
        assert!((codes.relative_efficiency_ratio() - 0.272).abs() < 0.001);
    }

    #[test]
    fn test_statistical_compression_ratio() {
        let codes = codes_without_words(vec![0.5, 0.5], &["0", "1"]);
        assert_eq!(1.0, codes.statistical_compression_ratio());

        let codes = codes_without_words(vec![0.8, 0.1, 0.1], &["0", "10", "11"]);
        assert!((codes.statistical_compression_ratio() - 1.66666).abs() < 0.00001);
    }
}

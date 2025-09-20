mod huffman;
mod shannon_fano;

pub use huffman::HuffmanEncoder;
pub use shannon_fano::ShannonFanoEncoder;

pub trait CodesBuilder {
    /// Строит оптимальный код на основе вероятностей вхождений символов.
    /// Сумма `probabilities` должна быть равна `1`.
    /// Возвращает вектор строк, где каждый элемент - код символа.
    fn build_optimal_codes(&mut self, probabilities: Vec<f64>) -> Codes;
}

#[derive(Debug, Default)]
pub struct Codes {
    probabilities: Vec<f64>,
    codes: Vec<String>,
}

impl Codes {
    pub fn new(probabilities: Vec<f64>, codes: Vec<String>) -> Self {
        Self {
            probabilities,
            codes,
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

#[cfg(test)]
mod tests {
    use super::Codes;

    // Вспомогательная функция для преобразования &str в Vec<String>
    fn str_vec(strings: &[&str]) -> Vec<String> {
        strings.iter().map(|&s| s.to_string()).collect()
    }

    #[test]
    fn test_mean_code_length() {
        let codes = Codes::new(
            vec![0.25, 0.25, 0.25, 0.25],
            str_vec(&["00", "01", "10", "11"]),
        );
        assert_eq!(2.0, codes.mean_code_length());

        let codes = Codes::new(
            vec![0.5, 0.25, 0.125, 0.125],
            str_vec(&["0", "10", "110", "111"]),
        );
        assert_eq!(1.75, codes.mean_code_length());

        let codes = Codes::new(
            vec![0.4, 0.3, 0.2, 0.1],
            str_vec(&["0", "10", "110", "1110"]),
        );
        assert_eq!(2.0, codes.mean_code_length());

        let codes = Codes::new(vec![0.5, 0.5], str_vec(&["0", "1"]));
        assert_eq!(1.0, codes.mean_code_length());

        let codes = Codes::new(vec![0.5, 0.4, 0.1], str_vec(&["000", "1110000", "0001111"]));
        assert!((codes.mean_code_length() - 5.0).abs() < 0.000001);

        let codes = Codes::new(vec![0.8, 0.1, 0.1], str_vec(&["0", "10", "11"]));
        assert_eq!(1.2, codes.mean_code_length());
    }

    #[test]
    fn test_entropy() {
        let codes = Codes::new(
            vec![0.25, 0.25, 0.25, 0.25],
            str_vec(&["00", "01", "10", "11"]),
        );
        assert_eq!(2.0, codes.entropy());

        let codes = Codes::new(vec![0.5, 0.25, 0.25], str_vec(&["0", "10", "11"]));
        assert_eq!(1.5, codes.entropy());

        let codes = Codes::new(vec![1.0], str_vec(&["0"]));
        assert_eq!(0.0, codes.entropy());

        let codes = Codes::new(vec![0.5], str_vec(&["0"]));
        assert_eq!(0.5, codes.entropy());

        let codes = Codes::new(vec![0.5, 0.5], str_vec(&["0", "1"]));
        assert_eq!(1.0, codes.entropy());

        let codes = Codes::new(vec![0.5, 0.4, 0.1], str_vec(&["000", "1110000", "0001111"]));
        assert!((codes.entropy() - 1.36).abs() < 0.001);
    }

    #[test]
    fn test_relative_efficiency_ratio() {
        let codes = Codes::new(vec![0.5, 0.5], str_vec(&["0", "1"]));
        assert_eq!(1.0, codes.relative_efficiency_ratio());

        let codes = Codes::new(vec![0.5, 0.4, 0.1], str_vec(&["000", "1110000", "0001111"]));
        assert!((codes.relative_efficiency_ratio() - 0.272).abs() < 0.001);
    }

    #[test]
    fn test_statistical_compression_ratio() {
        let codes = Codes::new(vec![0.5, 0.5], str_vec(&["0", "1"]));
        assert_eq!(1.0, codes.statistical_compression_ratio());

        let codes = Codes::new(vec![0.8, 0.1, 0.1], str_vec(&["0", "10", "11"]));
        assert!((codes.statistical_compression_ratio() - 1.66666).abs() < 0.00001);
    }
}

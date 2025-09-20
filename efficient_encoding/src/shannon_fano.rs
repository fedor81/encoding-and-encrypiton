use std::collections::VecDeque;

use super::{Codes, CodesBuilder};

pub struct ShannonFanoEncoder {}

impl ShannonFanoEncoder {}

impl ShannonFanoEncoder {
    pub fn new() -> Self {
        ShannonFanoEncoder {}
    }
}

impl CodesBuilder for ShannonFanoEncoder {
    fn build_optimal_codes(&mut self, mut probabilities: Vec<f64>) -> Codes {
        match probabilities.len() {
            0 => return Codes::default(),
            1 => return Codes::new(probabilities, vec!["0".into()]),
            _ => {}
        }

        probabilities.sort_by(|a, b| a.partial_cmp(b).unwrap().reverse());

        let mut codes = vec![String::new(); probabilities.len()];
        let mut queue: VecDeque<(&[f64], &mut [String])> = VecDeque::new();

        queue.push_back((probabilities.as_ref(), &mut codes));

        while let Some((probabilities, codes)) = queue.pop_front() {
            if probabilities.len() > 1 {
                // Делим по середине
                let mid = find_index_equal_groups(&probabilities);
                let (l_codes, r_codes) = codes.split_at_mut(mid);
                let (l_probabilities, r_probabilities) = probabilities.split_at(mid);

                // Добавляем коды к группам
                for code in l_codes.iter_mut() {
                    code.push('0');
                }
                for code in r_codes.iter_mut() {
                    code.push('1');
                }

                queue.push_back((l_probabilities, l_codes));
                queue.push_back((r_probabilities, r_codes));
            }
        }

        Codes::new(probabilities, codes)
    }
}

fn find_index_equal_groups(numbers: &[f64]) -> usize {
    let total_sum: f64 = numbers.iter().sum();
    let target_sum = total_sum / 2.0;

    let mut current_sum = 0.0;
    let mut split_index = 0;

    for (i, &prob) in numbers.iter().enumerate() {
        current_sum += prob;

        // Находим индекс, где сумма наиболее близка к половине
        if current_sum >= target_sum {
            // Проверяем, какой вариант лучше - включить текущий элемент или нет
            let diff_with_current = (current_sum - target_sum).abs();
            let diff_without_current = ((current_sum - prob) - target_sum).abs();

            if diff_without_current < diff_with_current {
                split_index = i; // Не включаем текущий элемент
            } else {
                split_index = i + 1; // Включаем текущий элемент
            }
            break;
        }
        split_index = i + 1;
    }

    // Обеспечиваем, чтобы split_index был в допустимых пределах
    split_index.min(numbers.len())
}

/// Функция для разделения вектора на две примерно равные части.
fn split_equal_groups(numbers: &[f64]) -> (&[f64], &[f64]) {
    let split_index = find_index_equal_groups(numbers);
    numbers.split_at(split_index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_equal_groups() {
        assert_eq!(
            (vec![0.1, 0.2, 0.3].as_ref(), vec![0.4].as_ref()),
            split_equal_groups(&vec![0.1, 0.2, 0.3, 0.4])
        );
        assert_eq!(
            (vec![0.25, 0.25].as_ref(), vec![0.25, 0.25].as_ref()),
            split_equal_groups(&vec![0.25, 0.25, 0.25, 0.25])
        );
        assert_eq!(
            (vec![0.4, 0.08].as_ref(), vec![0.48, 0.04].as_ref()),
            split_equal_groups(&vec![0.4, 0.08, 0.48, 0.04])
        );
    }

    #[test]
    fn test_build_optimal_codes() {
        assert_eq!(
            vec!["00", "01", "10", "11"],
            ShannonFanoEncoder::new()
                .build_optimal_codes(vec![0.25, 0.25, 0.25, 0.25])
                .codes
        );
        assert_eq!(
            vec!["0", "10", "110", "111"],
            ShannonFanoEncoder::new()
                .build_optimal_codes(vec![0.5, 0.25, 0.125, 0.125])
                .codes
        );
        assert_eq!(
            vec!["0"],
            ShannonFanoEncoder::new()
                .build_optimal_codes(vec![1.0])
                .codes
        );

        // 0.20 - 00
        // 0.15 - 010
        // 0.14 - 011
        // 0.13 - 100
        // 0.09 - 101
        // 0.08 - 1100
        // 0.06 - 1101
        // 0.05 - 11100
        // 0.04 - 11101
        // 0.03 - 11110
        // 0.02 - 111110
        // 0.01 - 111111
        assert_eq!(
            vec![
                "00", "010", "011", "100", "101", "1100", "1101", "11100", "11101", "11110",
                "111110", "111111",
            ],
            ShannonFanoEncoder::new()
                .build_optimal_codes(vec![
                    0.20, 0.15, 0.14, 0.13, 0.09, 0.08, 0.06, 0.05, 0.04, 0.03, 0.02, 0.01
                ])
                .codes
        );
    }
}

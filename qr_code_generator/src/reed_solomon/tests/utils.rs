use pretty_assertions::assert_eq;

use super::*;

/// Вспомогательная функция для создания кодера с заданным количеством контрольных символов
pub fn create_encoder(control_count: usize) -> ReedSolomon<FastGF256> {
    ReedSolomon::new(control_count, FastGF256::new())
}

/// Подсветка различий в кодированном сообщении
pub fn diff_highlight(encoded: RefPoly, err_encoded: RefPoly) -> String {
    let diff_encoded: Vec<String> = encoded
        .iter()
        .zip(err_encoded.iter())
        .map(|(e, err_e)| {
            if e != err_e {
                format!("\x1b[91m{}\x1b[0m", err_e) // красный цвет для различий
            } else {
                format!("{}", err_e)
            }
        })
        .collect();

    format!("[{}]", diff_encoded.join(", "))
}

pub type ErrorFn = Box<dyn Fn(RefPoly) -> Poly>;

pub struct StressTestConfig {
    pub encoders_count: usize,
    pub tests_by_encoder: usize,

    pub min_control_count: usize,
    pub max_control_count: usize,

    pub min_data_len: usize,
    pub max_data_len: usize,

    error_fn: ErrorFn,
}

impl StressTestConfig {
    pub fn with_error_fn(mut self, error_fn: impl Fn(RefPoly) -> Poly + 'static) -> Self {
        self.error_fn = Box::new(error_fn);
        self
    }

    pub fn error_fn(&self, poly: RefPoly) -> Poly {
        (self.error_fn)(poly)
    }

    pub fn with_n_errors_fn(self, n: usize) -> Self {
        self.with_error_fn(move |poly: RefPoly| {
            let mut modified = poly.to_vec();

            for _ in 0..n {
                // Получение индекса, где не было ошибок
                let index = loop {
                    let index = rand::random_range(0..poly.len());
                    if modified[index] == poly[index] {
                        break index;
                    }
                };

                // Внесение ошибки
                loop {
                    let error = rand::random();
                    if modified[index] != error {
                        modified[index] = error;
                        break;
                    }
                }
            }

            modified
        })
    }

    pub fn new_n_error_config(n: usize) -> Self {
        let mut cf = Self::default();

        // Чтобы кодировщик мог исправить t ошибок, контрольных символов должно быть 2t
        cf.min_control_count = cf.min_control_count.max(2 * n);
        cf.max_control_count = cf.max_control_count.max(2 * n);

        cf.with_n_errors_fn(n)
    }

    pub fn new_one_error_config() -> StressTestConfig {
        Self::new_n_error_config(1)
    }

    pub fn new_five_errors_config() -> StressTestConfig {
        Self::new_n_error_config(5)
    }
}

impl Default for StressTestConfig {
    fn default() -> Self {
        Self {
            encoders_count: 10,
            tests_by_encoder: 100,
            max_control_count: 10,
            min_control_count: 1,
            max_data_len: 20,
            min_data_len: 1,
            error_fn: Box::new(|poly: RefPoly| poly.to_vec()), // Не вносит ошибок
        }
    }
}

impl std::fmt::Debug for StressTestConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StressTestConfig")
            .field("encoders_count", &self.encoders_count)
            .field("tests_by_encoder", &self.tests_by_encoder)
            .field("min_control_count", &self.min_control_count)
            .field("max_control_count", &self.max_control_count)
            .field("min_data_len", &self.min_data_len)
            .field("max_data_len", &self.max_data_len)
            .finish()
    }
}

pub fn stress_test_common<F>(cf: StressTestConfig, test_logic: F)
where
    F: Fn(&mut String, &ReedSolomon<FastGF256>, &[u8], &[u8], &[u8]),
{
    for j in 0..cf.encoders_count {
        let control = rand::random_range(cf.min_control_count..=cf.max_control_count);
        let encoder = create_encoder(control);

        for i in 1..=cf.tests_by_encoder {
            let len = rand::random_range(cf.min_data_len..=cf.max_data_len);
            let message = rand::random_iter().take(len).collect::<Vec<_>>();

            let mut context = format!(
                "\nIteration: {}, \n\
                Control Count: {control} \n\
                Config: {cf:?} \n\
                Message:\t{message:?}",
                j * cf.tests_by_encoder + i // исправлена опечатка: было encoders_count
            );

            let encoded = encoder.encode(&message).unwrap();
            context += &format!("\nEncoded:\t{encoded:?}");

            let err_encoded = cf.error_fn(&encoded);
            context += &format!(
                "\nAfter Errors:\t{} Count: {}",
                diff_highlight(&encoded, &err_encoded),
                encoded
                    .iter()
                    .zip(err_encoded.iter())
                    .filter(|(e, err_e)| e != err_e)
                    .count(),
            );

            assert_eq!(
                encoded.len(),
                err_encoded.len(),
                "err_fn should not change length. {}",
                context
            );

            // Вызов уникальной логики теста
            test_logic(&mut context, &encoder, &message, &encoded, &err_encoded);
        }
    }
}

/// Проверка, что синдромы равны нулю
pub fn check_syndromes(encoder: &ReedSolomon<FastGF256>, encoded: RefPoly) -> Result<()> {
    let syndromes = encoder.calculate_syndromes(&encoded);

    // Проверим деление вручную
    let remainder = encoder.gf.mod_poly(&encoded, &encoder.gen_poly);

    if syndromes.iter().any(|&s| s != 0) {
        anyhow::bail!(
            "Syndromes: {syndromes:?} should be all zero for \n\
            Encoded: {encoded:?} \n\
            Remainder after division encoded / gen_poly: {remainder:?} \n\
            Generator polynomial: {:?} \n\
            Powers Alpha: {:?}",
            encoder.gen_poly,
            (0..encoder.control_count)
                .into_iter()
                .map(|i| encoder.gf.alpha_pow(i as u8))
                .collect::<Vec<_>>()
        )
    }
    Ok(())
}

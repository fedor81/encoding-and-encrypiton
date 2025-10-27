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

    pub max_control_count: usize,
    pub min_control_count: usize,

    pub max_data_len: usize,
    pub min_data_len: usize,

    pub error_fn: ErrorFn,
}

impl StressTestConfig {
    pub fn with_error_fn(mut self, error_fn: impl Fn(RefPoly) -> Poly + 'static) -> Self {
        self.error_fn = Box::new(error_fn);
        self
    }

    pub fn error_fn(&self, poly: RefPoly) -> Poly {
        (self.error_fn)(poly)
    }

    pub fn new_one_error_config() -> StressTestConfig {
        let mut cf = StressTestConfig::default().with_error_fn(|poly| {
            let mut modified = poly.to_vec();
            let index = rand::random_range(0..poly.len()); // Случайный индекс

            modified[index] = rand::random(); // Внесение ошибки
            modified
        });

        // Чтобы кодировщик мог исправить t ошибок, контрольных символов должно быть 2t
        cf.min_control_count = 2;
        cf
    }

    pub fn new_five_errors_config() -> StressTestConfig {
        let mut cf = StressTestConfig::default();

        const ERRORS: usize = 5;

        // Чтобы кодировщик мог исправить t ошибок, контрольных символов должно быть 2t
        cf.min_control_count = 2 * ERRORS;
        cf.max_control_count = 2 * ERRORS;

        cf = cf.with_error_fn(|poly: RefPoly| {
            let mut modified = poly.to_vec();

            for _ in 0..ERRORS {
                let index = rand::random_range(0..poly.len()); // Случайный индекс

                modified[index] = rand::random(); // Внесение ошибки
            }

            modified
        });
        cf
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
                Message:\t{message:?}",
                j * cf.tests_by_encoder + i // исправлена опечатка: было encoders_count
            );

            let encoded = encoder.encode(&message).unwrap();
            context += &format!("\nEncoded:\t{encoded:?}");

            let err_encoded = cf.error_fn(&encoded);
            context += &format!(
                "\nAfter Errors:\t{}",
                diff_highlight(&encoded, &err_encoded)
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

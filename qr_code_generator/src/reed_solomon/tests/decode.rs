use super::*;

fn decode_stress_test_helper(cf: StressTestConfig) {
    stress_test_common(cf, |context, encoder, message, encoded, err_encoded| {
        let decoded = encoder
            .decode(&err_encoded)
            .with_context(|| format!("{}", context))
            .unwrap();
        *context += &format!("\nDecoded: {decoded:?}");

        assert_eq!(message, decoded, "{}", context);
    });
}

#[test]
#[ignore]
/// Самый большой тест на проверку корректности декодирования после внесения ошибок
fn decode_random_stress() {
    let mut cf = StressTestConfig::new_n_error_config(50);

    cf.min_data_len = 100;
    cf.max_data_len = 150;

    cf.encoders_count = 100;
    cf.tests_by_encoder = 50;

    decode_stress_test_helper(cf);
}

#[test]
fn decode_one_error() {
    decode_stress_test_helper(StressTestConfig::new_one_error_config());
}

#[test]
fn decode_no_errors_stress() {
    decode_stress_test_helper(StressTestConfig::default());
}

#[test]
fn decode_up_to_five_errors() {
    decode_stress_test_helper(StressTestConfig::new_five_errors_config());
}

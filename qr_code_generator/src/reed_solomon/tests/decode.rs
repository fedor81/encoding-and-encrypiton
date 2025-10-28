use super::*;

fn decode_stress_test_helper(cf: StressTestConfig) -> Result<()> {
    stress_test_common(cf, |context, encoder, message, encoded, err_encoded| {
        let decoded = encoder
            .decode(&err_encoded)
            .with_context(|| format!("{}", context))
            .unwrap();
        *context += &format!("\nDecoded: {decoded:?}");

        assert_eq!(message, decoded, "{}", context);
    });
    Ok(())
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

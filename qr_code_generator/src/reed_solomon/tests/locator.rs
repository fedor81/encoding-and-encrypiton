use super::*;

#[test]
fn test_error_positions() {
    let rs = create_encoder(10);
    let gf = &rs.gf;
    let data_len = 20;

    // L(x) = (1 - α^5 * x) * (1 - α^10 * x)
    // L(x) = α^15*x^2 + (α^5 + α^10)*x + 1

    // Ошибки в позициях 5 и 10
    let alpha_5 = gf.alpha_pow(5); // α^5
    let alpha_10 = gf.alpha_pow(10); // α^10

    let error_locator = vec![
        1,                                               // x^0: 1
        gf.add(gf.mul(alpha_5, 1), gf.mul(alpha_10, 1)), // x^1: α^5 + α^10
        gf.mul(alpha_5, alpha_10),                       // x^2: α^5 * α^10 = α^15
    ];

    let positions = rs.find_error_positions(&error_locator, data_len).unwrap();
    assert_eq!(positions, vec![5, 10]);
}

#[test]
fn test_find_locator_derivative() {
    let rs = create_encoder(10);

    // Со случайным случайным элементом L(x) = nx^i
    for i in 0..50 {
        let n = rand::random();
        let mut input = vec![0; i + 1];
        input[i] = n;

        let mut expected = vec![0; i];

        if i % 2 == 1 {
            expected[i - 1] = n;
        }

        if expected.iter().all(|&n| n == 0) {
            expected = vec![0];
        }

        assert_eq!(
            expected,
            rs.find_locator_derivative(&input),
            "input: {:?}",
            input
        );
    }

    assert_eq!(
        rs.find_locator_derivative(&vec![1, 6, 5, 4]), // L(x) = 1 + 6x + 5x^2 + 4x^3
        vec![6, 0, 4],                                 // L'(x) = 6 + 4x^2
    );
}

#[test]
fn test_find_error_positions_stress() {
    find_error_positions_stress_helper(StressTestConfig::new_one_error_config());
}

fn find_error_positions_stress_helper(cf: StressTestConfig) {
    stress_test_common(cf, |context, encoder, message, encoded, err_encoded| {
        // Actual
        let syndromes = encoder.calculate_syndromes(&message);
        let error_locator = encoder.find_error_locator(&syndromes).unwrap();
        let mut error_positions = encoder
            .find_error_positions(&error_locator, message.len())
            .unwrap();

        error_positions.sort();

        *context += &format!(
            "\nSyndromes:\t{syndromes:?}\n\
            Error locator:\t{error_locator:?}\n\
            Error positions:\t{error_positions:?}",
        );

        // Expected
        let mut expected = Vec::new();
        for (i, (&enc_char, &err_char)) in encoded.iter().zip(err_encoded.iter()).enumerate() {
            if enc_char != err_char {
                expected.push(i);
            }
        }

        assert_eq!(expected, error_positions, "{}", context);
    });
}

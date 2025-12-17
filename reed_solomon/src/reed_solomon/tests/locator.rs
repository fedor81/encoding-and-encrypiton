use super::*;

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

        assert_eq!(expected, rs.find_locator_derivative(&input), "input: {:?}", input);
    }

    assert_eq!(
        rs.find_locator_derivative(&vec![1, 6, 5, 4]), // L(x) = 1 + 6x + 5x^2 + 4x^3
        vec![6, 0, 4],                                 // L'(x) = 6 + 4x^2
    );
}

#[test]
fn test_find_error_locator_no_errors() -> Result<()> {
    let encoder = create_encoder(4);
    let data = vec![32, 91, 11, 120, 209];

    // Кодируем данные
    let encoded = encoder.encode(&data)?;

    // Вычисляем синдромы (должны быть все нули)
    let syndromes = encoder.calculate_syndromes(&encoded);
    println!("Syndromes for no errors: {:?}", syndromes);

    // Находим локатор ошибок
    let locator = encoder.find_error_locator(&syndromes)?;

    // При отсутствии ошибок локатор должен быть [1]
    assert_eq!(
        locator,
        vec![1],
        "For no errors, locator should be [1], got: {:?}",
        locator
    );

    Ok(())
}

#[test]
fn find_error_locator_error_1_to_10() {
    for n in 1..=10 {
        let cf = StressTestConfig::new_n_error_config(n);

        stress_test_common(cf, |context, encoder, _message, _encoded, err_encoded| {
            let syndromes = encoder.calculate_syndromes(&err_encoded);
            *context += &format!("\nSyndromes:\t{syndromes:?}");

            let locator = encoder
                .find_error_locator(&syndromes)
                .with_context(|| format!("{}", context))
                .unwrap();

            // Локатор должен быть степени n: [1, ..., n]
            assert_eq!(
                locator.len(),
                n + 1,
                "For {n} errors, locator should have leading power x^{n}, got: {locator:?} \n{context}",
            );
        });
    }
}

#[test]
fn test_error_locator_consistency() -> Result<()> {
    // Проверяем, что для одинаковых синдромов получаем одинаковый локатор
    let encoder = create_encoder(6);

    let syndromes1 = vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66];
    let syndromes2 = syndromes1.clone();

    let locator1 = encoder.find_error_locator(&syndromes1)?;
    let locator2 = encoder.find_error_locator(&syndromes2)?;

    assert_eq!(locator1, locator2, "Same syndromes should produce same locator");

    Ok(())
}

use super::*;

#[test]
fn test_encode_single_byte() {
    let encoder = create_encoder(5);
    let data = vec![42];

    let encoded = encoder.encode(&data).unwrap();

    assert_eq!(encoded.len(), 6);
    assert_eq!(encoded[5], 42);

    check_syndromes(&encoder, &encoded)
        .with_context(|| format!("\n Message: {data:?}"))
        .unwrap();
}

#[test]
fn test_encode_max_length() {
    // Максимальная длина для GF(256) - 255 символов
    let control_count = 10;
    let data_len = 255 - control_count;
    let encoder = create_encoder(control_count);

    let data: Vec<u8> = (0..data_len as u8).collect();

    let encoded = encoder.encode(&data).unwrap();

    assert_eq!(encoded.len(), 255);
    assert_eq!(&encoded[control_count..], data.as_slice());

    check_syndromes(&encoder, &encoded)
        .with_context(|| format!("\n Message: {data:?}"))
        .unwrap();
}

#[test]
fn test_encode_exceeds_max_length() {
    let encoder = create_encoder(10);
    let data: Vec<u8> = vec![0; 246]; // 246 + 10 = 256 > 255

    let result = encoder.encode(&data);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("too long"));
}

#[test]
fn test_encode_zeros() {
    let encoder = create_encoder(4);
    let data = vec![0, 0, 0, 0];

    let encoded = encoder.encode(&data).unwrap();

    assert_eq!(encoded.len(), 8);
    assert_eq!(&encoded[4..], data.as_slice());
    // Для нулевых данных контрольные символы также должны быть нулевыми
    assert_eq!(encoded[0], 0);
    assert_eq!(encoded[1], 0);
    assert_eq!(encoded[2], 0);
    assert_eq!(encoded[3], 0);

    check_syndromes(&encoder, &encoded)
        .with_context(|| format!("\n Message: {data:?}"))
        .unwrap();
}

#[test]
fn test_encode_ones() {
    let encoder = create_encoder(3);
    let data = vec![1, 1, 1, 1, 1];

    let encoded = encoder.encode(&data).unwrap();

    assert_eq!(encoded.len(), 8);
    assert_eq!(&encoded[3..], data.as_slice());
    // Контрольные символы не должны быть всеми единицами
    assert!(encoded[0] != 1 || encoded[1] != 1 || encoded[2] != 1);

    check_syndromes(&encoder, &encoded)
        .with_context(|| format!("\n Message: {data:?}"))
        .unwrap();
}

#[test]
fn test_encode_high_values() {
    let encoder = create_encoder(4);
    let data = vec![255, 254, 253, 252];

    let encoded = encoder.encode(&data).unwrap();

    assert_eq!(encoded.len(), 8);
    assert_eq!(&encoded[4..], data.as_slice());

    check_syndromes(&encoder, &encoded)
        .with_context(|| format!("\n Message: {data:?}"))
        .unwrap();
}

#[test]
fn test_encode_consistency() {
    let encoder = create_encoder(6);
    let data = vec![10, 20, 30, 40, 50];

    // Многократное кодирование одних и тех же данных должно давать одинаковый результат
    let encoded1 = encoder.encode(&data).unwrap();
    let encoded2 = encoder.encode(&data).unwrap();
    let encoded3 = encoder.encode(&data).unwrap();

    assert_eq!(encoded1, encoded2);
    assert_eq!(encoded2, encoded3);
}

#[test]
fn test_encode_deterministic() {
    let encoder1 = create_encoder(5);
    let encoder2 = create_encoder(5);
    let data = vec![1, 3, 5, 7, 9, 11];

    let encoded1 = encoder1.encode(&data).unwrap();
    let encoded2 = encoder2.encode(&data).unwrap();

    // Разные экземпляры кодера с одинаковыми параметрами должны давать одинаковый результат
    assert_eq!(encoded1, encoded2);
}

#[test]
fn test_encode_boundary_values() {
    let encoder = create_encoder(4);

    // Тестируем граничные значения байтов
    let test_cases = vec![
        vec![0],
        vec![255],
        vec![0, 255],
        vec![255, 0],
        vec![1, 254],
        vec![128, 127],
    ];

    for data in test_cases {
        let encoded = encoder.encode(&data).unwrap();
        assert_eq!(encoded.len(), data.len() + 4);
        assert_eq!(&encoded[4..], data.as_slice());

        check_syndromes(&encoder, &encoded)
            .with_context(|| format!("\n Message: {data:?}"))
            .unwrap();
    }
}

#[test]
fn test_encode_preserves_data_integrity() {
    let encoder = create_encoder(8);
    let original_data: Vec<u8> = (0..50).collect();

    let encoded = encoder.encode(&original_data).unwrap();

    // Проверяем, что исходные данные не были изменены
    let recovered_data = &encoded[8..];
    assert_eq!(recovered_data, original_data.as_slice());

    check_syndromes(&encoder, &encoded)
        .with_context(|| format!("\n Message: {:?}", original_data))
        .unwrap();
}

// Property-based тесты
#[test]
fn test_encode_length_invariant() {
    let encoder = create_encoder(10);

    for data_len in 0..100 {
        let data: Vec<u8> = (0..data_len).map(|i| (i * 3) as u8).collect();

        let encoded = encoder.encode(&data).unwrap();

        // Инвариант: длина закодированных данных = длина исходных данных + контрольные символы
        assert_eq!(encoded.len(), data.len() + 10);

        // Проверка, что синдромы равны нулю

        check_syndromes(&encoder, &encoded)
            .with_context(|| format!("\n Message: {data:?}"))
            .unwrap();
    }
}

#[test]
fn test_encode_data_preservation_invariant() {
    let encoder = create_encoder(7);

    for data_len in 1..50 {
        let data: Vec<u8> = (0..data_len).map(|i| (i * 5 + 13) as u8).collect();

        let encoded = encoder.encode(&data).unwrap();

        // Инвариант: исходные данные сохраняются в конце закодированного сообщения
        assert_eq!(&encoded[7..], data.as_slice());

        check_syndromes(&encoder, &encoded)
            .with_context(|| format!("\n Message: {data:?}"))
            .unwrap();
    }
}

// Тест на проверку, что разные данные дают разные контрольные символы
#[test]
fn test_encode_different_data_different_control() {
    let encoder = create_encoder(4);

    let data1 = vec![1, 2, 3, 4];
    let data2 = vec![1, 2, 3, 5]; // Один байт отличается

    let encoded1 = encoder.encode(&data1).unwrap();
    let encoded2 = encoder.encode(&data2).unwrap();

    // Контрольные символы должны быть разными для разных данных
    assert_ne!(&encoded1[0..4], &encoded2[0..4]);
}

#[test]
fn test_encode_minimal_control() {
    let encoder = create_encoder(1); // Минимальное количество контрольных символов
    let data = vec![42];

    let encoded = encoder.encode(&data).unwrap();

    assert_eq!(encoded.len(), 2);
    assert_eq!(encoded[1], 42);

    check_syndromes(&encoder, &encoded)
        .with_context(|| format!("\n Message: {data:?}"))
        .unwrap();
}

#[test]
fn test_encode_large_control_small_data() {
    let control = 200;
    let encoder = create_encoder(control);
    let data = vec![1, 2, 3];

    let encoded = encoder.encode(&data).unwrap();

    assert_eq!(encoded.len(), control + data.len());
    assert_eq!(&encoded[control..], data.as_slice());

    check_syndromes(&encoder, &encoded)
        .with_context(|| format!("\n Message: {data:?}"))
        .unwrap();
}

// Многократное кодирование разных данных
#[test]
fn debug_simple_test() {
    let encoder = create_encoder(2);
    let data = vec![1, 2];
    
    println!("Data: {:?}", data);
    
    let encoded = encoder.encode(&data).unwrap();
    println!("Encoded: {:?}", encoded);
    
    // Проверим синдромы
    let syndromes = encoder.calculate_syndromes(&encoded);
    println!("Syndromes: {:?}", syndromes);
    
    // Проверим остаток от деления
    let remainder = encoder.gf.mod_poly(&encoded, &encoder.gen_poly);
    println!("Remainder: {:?}", remainder);
    
    // Проверим порождающий многочлен
    println!("Generator polynomial: {:?}", encoder.gen_poly);
    
    // Проверим корни порождающего многочлена
    for i in 0..2 {
        let root = encoder.gf.alpha_pow(i as u8);
        let value = encoder.gf.eval_poly(&encoder.gen_poly, root);
        println!("Root α^{} = {}, polynomial value = {}", i, root, value);
    }
    
    // Проверим, что синдромы равны нулю
    check_syndromes(&encoder, &encoded).unwrap();
}

#[test]
fn test_random_stress() {
    let encoders_count = 10;
    let tests_count = 100;

    let max_control_count = 10;
    let max_data_len = 20;

    for j in 0..encoders_count {
        let control = rand::random_range(1..=max_control_count);
        let encoder = create_encoder(control);

        for i in 1..=tests_count {
            let len = rand::random_range(1..=max_data_len);
            let data: Vec<u8> = rand::random_iter().take(len).collect();

            let encoded = encoder.encode(&data).unwrap();

            assert_eq!(encoded.len(), data.len() + control);
            assert_eq!(&encoded[control..], data.as_slice());

            check_syndromes(&encoder, &encoded)
                .with_context(|| {
                    format!(
                        "\nIteration: {}, \n\
                        Control Count: {control} \n\
                        Message: {data:?}",
                        j * encoders_count + i
                    )
                })
                .unwrap();
        }
    }
}

/// Проверка, что синдромы равны нулю
fn check_syndromes(encoder: &ReedSolomon<FastGF256>, encoded: RefPoly) -> Result<()> {
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

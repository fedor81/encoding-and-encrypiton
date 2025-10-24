use super::*;

#[test]
fn test_encode_basic() {
    let encoder = create_encoder(4);
    let data = vec![1, 2, 3, 4, 5];

    let encoded = encoder.encode(&data).unwrap();

    // Проверяем, что длина результата = длина данных + контрольные символы
    assert_eq!(encoded.len(), data.len() + 4);
    // Проверяем, что исходные данные сохранились в конце
    assert_eq!(&encoded[4..], data.as_slice());
    // Проверяем, что контрольные символы не нулевые
    assert!(encoded[0] != 0 || encoded[1] != 0 || encoded[2] != 0 || encoded[3] != 0);

    check_syndromes(&encoder, &encoded);
}

#[test]
fn test_encode_single_byte() {
    let encoder = create_encoder(5);
    let data = vec![42];

    let encoded = encoder.encode(&data).unwrap();

    assert_eq!(encoded.len(), 6);
    assert_eq!(encoded[5], 42);

    check_syndromes(&encoder, &encoded);
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

    check_syndromes(&encoder, &encoded);
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
fn test_encode_different_control_counts() {
    for control_count in [1, 2, 4, 8, 16, 32].iter() {
        let encoder = create_encoder(*control_count);
        let data = vec![1, 2, 3, 4, 5];

        let encoded = encoder.encode(&data).unwrap();

        assert_eq!(encoded.len(), data.len() + control_count);
        assert_eq!(&encoded[*control_count..], data.as_slice());

        check_syndromes(&encoder, &encoded);
    }
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

    check_syndromes(&encoder, &encoded);
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

    check_syndromes(&encoder, &encoded);
}

#[test]
fn test_encode_high_values() {
    let encoder = create_encoder(4);
    let data = vec![255, 254, 253, 252];

    let encoded = encoder.encode(&data).unwrap();

    assert_eq!(encoded.len(), 8);
    assert_eq!(&encoded[4..], data.as_slice());

    check_syndromes(&encoder, &encoded);
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

        check_syndromes(&encoder, &encoded);
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

    check_syndromes(&encoder, &encoded);
}

#[test]
fn test_encode_control_symbols_non_trivial() {
    let encoder = create_encoder(4);
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8];

    let encoded = encoder.encode(&data).unwrap();
    let control_symbols = &encoded[0..4];

    // Проверяем, что хотя бы один контрольный символ ненулевой
    // (для нетривиальных данных)
    assert!(control_symbols.iter().any(|&x| x != 0));

    check_syndromes(&encoder, &encoded);
}

#[test]
fn test_encode_large_control_count() {
    let encoder = create_encoder(32);
    let data = vec![100; 50];

    let encoded = encoder.encode(&data).unwrap();

    assert_eq!(encoded.len(), 82);
    assert_eq!(&encoded[32..], data.as_slice());

    check_syndromes(&encoder, &encoded);
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

        check_syndromes(&encoder, &encoded);
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

        check_syndromes(&encoder, &encoded);
    }
}

// Стресс-тест
#[test]
fn test_encode_stress() {
    let encoder = create_encoder(16);

    // Многократное кодирование разных данных
    for i in 0..1000 {
        let data: Vec<u8> = (0..(i % 50 + 1)).map(|j| (i * j) as u8).collect();

        let encoded = encoder.encode(&data).unwrap();

        assert_eq!(encoded.len(), data.len() + 16);
        assert_eq!(&encoded[16..], data.as_slice());

        check_syndromes(&encoder, &encoded);
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

    check_syndromes(&encoder, &encoded);
}

#[test]
fn test_encode_large_control_small_data() {
    let encoder = create_encoder(50);
    let data = vec![1, 2, 3];

    let encoded = encoder.encode(&data).unwrap();

    assert_eq!(encoded.len(), 53);
    assert_eq!(&encoded[50..], data.as_slice());

    check_syndromes(&encoder, &encoded);
}

#[test]
fn test_encode_sequential_data() {
    let encoder = create_encoder(5);
    let data: Vec<u8> = (0..20).collect();

    let encoded = encoder.encode(&data).unwrap();

    assert_eq!(encoded.len(), 25);
    assert_eq!(&encoded[5..], data.as_slice());

    check_syndromes(&encoder, &encoded);
}

#[test]
fn test_encode_random_like_data() {
    let encoder = create_encoder(8);
    // Данные, похожие на случайные
    let data = vec![123, 45, 67, 89, 210, 132, 54, 176, 198, 220];

    let encoded = encoder.encode(&data).unwrap();

    assert_eq!(encoded.len(), 18);
    assert_eq!(&encoded[8..], data.as_slice());

    check_syndromes(&encoder, &encoded);
}

/// Проверка, что синдромы равны нулю
fn check_syndromes(encoder: &ReedSolomon<FastGF256>, encoded: RefPoly) {
    let syndromes = encoder.calculate_syndromes(&encoded);

    assert!(
        syndromes.iter().all(|&s| s == 0),
        "Syndromes: {syndromes:?} should be all zero for \n Encoded: {encoded:?}",
    );
}

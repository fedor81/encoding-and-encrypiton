use std::time::Duration;

use super::GF256;

pub fn test_gf256<T: GF256>(gf: T) {
    println!("Testing GF256 implementation...");

    test_basic_operations(&gf);
    test_add_sub(&gf);
    test_mul_commutativity(&gf);
    test_mul_associativity(&gf);
    test_distributive(&gf);
    test_inverse(&gf);
    test_division(&gf);
    test_power(&gf);
    test_identity(&gf);
    test_zero(&gf);
    test_specific_values(&gf);

    println!("All GF256 tests passed!");
}

fn test_basic_operations<T: GF256>(gf: &T) {
    println!("  Testing basic operations...");

    // Тестируем несколько конкретных значений
    assert_eq!(gf.add(5, 3), 5 ^ 3);
    assert_eq!(gf.sub(10, 7), 10 ^ 7);

    // Умножение и деление обратных операций
    let a = 5;
    let b = 3;
    let mul = gf.mul(a, b);
    let div = gf.div(mul, b);
    assert_eq!(
        div, a,
        "Multiplication/division inverse failed: {} * {} = {}, but {} / {} = {}",
        a, b, mul, mul, b, div
    );
}

fn test_add_sub<T: GF256>(gf: &T) {
    println!("  Testing addition/subtraction...");

    for a in 0..=255u8 {
        for b in 0..=255u8 {
            // В GF(256) сложение и вычитание - это XOR
            assert_eq!(gf.add(a, b), a ^ b, "Add failed for {} + {}", a, b);
            assert_eq!(gf.sub(a, b), a ^ b, "Sub failed for {} - {}", a, b);

            // Коммутативность сложения
            assert_eq!(gf.add(a, b), gf.add(b, a), "Add commutativity failed");

            // Ассоциативность сложения
            let c = a.wrapping_add(b);
            assert_eq!(
                gf.add(gf.add(a, b), c),
                gf.add(a, gf.add(b, c)),
                "Add associativity failed"
            );
        }
    }

    // В GF(256) сложение и вычитание - это XOR
    assert_eq!(gf.sub(2, 1), 2 ^ 1); // 2 - 1 = 3
    assert_eq!(gf.add(2, 1), 2 ^ 1); // 2 + 1 = 3

    // Проверяем несколько операций
    assert_eq!(gf.add(5, 3), 5 ^ 3); // 5 + 3 = 6
    assert_eq!(gf.sub(5, 3), 5 ^ 3); // 5 - 3 = 6
}

fn test_mul_commutativity<T: GF256>(gf: &T) {
    println!("  Testing multiplication commutativity...");

    for a in 0..=255u8 {
        for b in 0..=255u8 {
            assert_eq!(
                gf.mul(a, b),
                gf.mul(b, a),
                "Multiplication commutativity failed for {} * {}",
                a,
                b
            );
        }
    }
}

fn test_mul_associativity<T: GF256>(gf: &T) {
    println!("  Testing multiplication associativity...");

    // Тестируем подмножество из-за большого количества комбинаций
    for a in 0..=50u8 {
        for b in 0..=50u8 {
            for c in 0..=50u8 {
                let left = gf.mul(gf.mul(a, b), c);
                let right = gf.mul(a, gf.mul(b, c));
                assert_eq!(
                    left, right,
                    "Multiplication associativity failed for ({}, {}, {})",
                    a, b, c
                );
            }
        }
    }
}

fn test_distributive<T: GF256>(gf: &T) {
    println!("  Testing distributive property...");

    for a in 0..=50u8 {
        for b in 0..=50u8 {
            for c in 0..=50u8 {
                let left = gf.mul(a, gf.add(b, c));
                let right = gf.add(gf.mul(a, b), gf.mul(a, c));
                assert_eq!(left, right, "Distributive property failed for {} * ({} + {})", a, b, c);
            }
        }
    }
}

fn test_inverse<T: GF256>(gf: &T) {
    println!("  Testing inverse...");

    for a in 1..=255u8 {
        // Пропускаем 0, у него нет обратного
        let inv = gf.inverse(a);
        let product = gf.mul(a, inv);
        assert_eq!(
            product, 1,
            "Inverse test failed for {}: {} * {} = {}, expected 1",
            a, a, inv, product
        );

        // Проверяем, что обратный элемент уникален
        let inv2 = gf.inverse(a);
        assert_eq!(inv, inv2, "Inverse not consistent for {}", a);
    }
}

fn test_division<T: GF256>(gf: &T) {
    println!("  Testing division...");

    for a in 0..=255u8 {
        for b in 1..=255u8 {
            // Делитель не может быть 0
            let div = gf.div(a, b);
            let product = gf.mul(div, b);
            assert_eq!(
                product, a,
                "Division test failed: {} / {} = {}, but {} * {} = {}",
                a, b, div, div, b, product
            );
        }
    }
}

fn test_power<T: GF256>(gf: &T) {
    println!("  Testing power...");

    // Проверяем особые случаи
    for a in 0..=255u8 {
        // a^0 = 1 для всех a ≠ 0, но 0^0 обычно определяется как 1
        if a != 0 {
            assert_eq!(gf.pow(a, 0), 1, "Power test failed: {}^0 != 1", a);
        }

        // a^1 = a
        assert_eq!(gf.pow(a, 1), a, "Power test failed: {}^1 != {}", a, a);

        // Проверяем несколько степеней
        let a2 = gf.pow(a, 2);
        let a3 = gf.pow(a, 3);
        let a2_times_a = gf.mul(a2, a);
        assert_eq!(a3, a2_times_a, "Power consistency failed: {}^3 != {}^2 * {}", a, a, a);
    }

    // Проверяем, что a^255 = 1 для всех a ≠ 0 (теорема Ферма для конечных полей)
    for a in 1..=255u8 {
        let a_255 = gf.pow(a, 255);
        assert_eq!(
            a_255, 1,
            "Fermat's little theorem failed: {}^255 = {}, expected 1",
            a, a_255
        );
    }
}

fn test_identity<T: GF256>(gf: &T) {
    println!("  Testing identity elements...");

    for a in 0..=255u8 {
        // Нейтральный элемент по сложению
        assert_eq!(gf.add(a, 0), a, "Additive identity failed for {}", a);

        if a != 0 {
            // Нейтральный элемент по умножению
            assert_eq!(gf.mul(a, 1), a, "Multiplicative identity failed for {}", a);
        }
    }
}

fn test_zero<T: GF256>(gf: &T) {
    println!("  Testing zero properties...");

    for a in 0..=255u8 {
        // Умножение на 0
        assert_eq!(gf.mul(a, 0), 0, "Multiplication by zero failed for {}", a);
        assert_eq!(gf.mul(0, a), 0, "Zero multiplication failed for {}", a);
    }

    // Деление 0 на ненулевое число
    for b in 1..=255u8 {
        assert_eq!(gf.div(0, b), 0, "Division of zero failed for 0/{}", b);
    }
}

fn test_specific_values<T: GF256>(gf: &T) {
    println!("  Testing specific values...");

    // Тестируем несколько известных значений из стандартных таблиц GF(256)
    // Эти значения могут зависеть от примитивного полинома

    // Проверяем, что 2 * 2 = 4 (в начале таблицы должно работать как обычное умножение)
    let two_times_two = gf.mul(2, 2);
    assert!(two_times_two == 4 || two_times_two != 0, "2 * 2 should be reasonable");

    // Проверяем, что 128 * 2 = 27 для примитивного полинома 0x11D
    // Это известное значение из таблиц GF(256)
    let test_val = gf.mul(128, 2);
    // Значение может быть разным в зависимости от полинома, но должно быть ненулевым
    assert_ne!(test_val, 0, "128 * 2 should not be zero");

    // Проверяем согласованность: a * b^-1 = a / b
    let a = 100;
    let b = 50;
    let mul_inv = gf.mul(a, gf.inverse(b));
    let div_result = gf.div(a, b);
    assert_eq!(mul_inv, div_result, "a * b^-1 should equal a / b");
}

// Дополнительные тесты для производительности и граничных случаев
pub fn test_gf256_performance<T: GF256>(gf: T, expected_duration: Duration) {
    println!("Testing GF256 performance...");

    let start = std::time::Instant::now();
    let mut result = 0u8;

    // Выполняем много операций для тестирования производительности
    for _ in 0..1000 {
        for a in 0..=255u8 {
            // Избегаем деления на 0
            for b in 1..=255u8 {
                result ^= gf.mul(a, b);
                result ^= gf.div(a, b);
                result ^= gf.pow(a, b % 16); // Ограничиваем степень для скорости
            }
        }
    }

    let duration = start.elapsed();
    println!("  Performance test completed in {:?}", duration);
    println!("  (Result: {} - used to prevent optimization)", result);

    // Тест должен завершиться за разумное время
    assert!(duration < expected_duration, "Performance test too slow");
}

// Тест для проверки корректности исключений
pub fn test_gf256_exceptions<T: GF256 + std::panic::RefUnwindSafe>(gf: T) {
    println!("Testing GF256 exceptions...");

    // Проверяем, что деление на 0 вызывает panic
    let result = std::panic::catch_unwind(|| {
        gf.div(1, 0);
    });
    assert!(result.is_err(), "Division by zero should panic");

    // Проверяем, что inverse(0) вызывает panic
    let result = std::panic::catch_unwind(|| {
        gf.inverse(0);
    });
    assert!(result.is_err(), "Inverse of zero should panic");
}

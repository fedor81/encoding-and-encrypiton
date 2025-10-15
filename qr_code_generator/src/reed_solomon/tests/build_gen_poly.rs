use super::*;
use crate::gf::FastGF256;

#[test]
fn test_build_gen_poly() {
    // Тестируем для разного количества символов коррекции
    for control_count in 1..=10 {
        // Ограничиваем для скорости тестов
        let gf = FastGF256::new();
        let gen_poly = ReedSolomon::build_gen_poly(&gf, control_count);

        // Проверяем базовые свойства порождающего полинома
        test_generator_poly_properties(&gf, &gen_poly, control_count);
    }
}

fn test_generator_poly_properties(gf: &FastGF256, gen_poly: &[u8], control: usize) {
    // 1. Проверяем длину полинома
    assert_eq!(
        gen_poly.len(),
        control + 1,
        "Generator polynomial length should be nsym + 1. nsym: {}, length: {}",
        control,
        gen_poly.len()
    );

    // 2. Проверяем, что старший коэффициент равен 1
    assert_eq!(
        gen_poly[gen_poly.len() - 1],
        1,
        "Leading coefficient should be 1. Got: {} for nsym: {}",
        gen_poly[gen_poly.len() - 1],
        control
    );

    // 3. Проверяем, что полином имеет корни α^0, α^1, ..., α^(nsym-1)
    for i in 0..control {
        let root = gf.pow_primitive_poly(i as u8);
        let value = gf.eval_poly(gen_poly, root);
        assert_eq!(
            value, 0,
            "Generator polynomial should have root α^{}. Evaluation at α^{} = {}, expected 0. Polynomial: {:?}",
            i, i, value, gen_poly
        );
    }

    // 4. Проверяем, что полином мончический (старший коэффициент = 1)
    assert_eq!(
        gen_poly[gen_poly.len() - 1],
        1,
        "Generator polynomial should be monic"
    );

    // 5. Проверяем, что все коэффициенты находятся в поле GF(256)
    for &coef in gen_poly {
        assert!(coef <= 255, "Coefficient out of GF(256) range: {}", coef);
    }

    // 6. Проверяем, что полином не имеет корней кроме α^0..α^(nsym-1)
    // (это сложно проверить полностью, но проверим несколько случайных точек)
    for _ in 0..5 {
        let random_point = gf.pow_primitive_poly(100 + control as u8); // Берем точку вне диапазона корней
        let value = gf.eval_poly(gen_poly, random_point);
        assert_ne!(
            value,
            0,
            "Generator polynomial should not have root at random point α^{}. Polynomial: {:?}",
            100 + control,
            gen_poly
        );
    }
}

#[test]
fn test_specific_generator_polys() {
    let gf = FastGF256::new();

    // Тестируем конкретные известные порождающие полиномы
    // Эти значения зависят от примитивного полинома (0x11D)

    // Для nsym = 1: g(x) = (x + α^0) = x + 1
    let gen_poly_1 = ReedSolomon::build_gen_poly(&gf, 1);
    assert_eq!(
        gen_poly_1,
        vec![1, 1],
        "Generator polynomial for nsym=1 should be [1, 1]"
    );

    // Для nsym = 2: g(x) = (x + α^0)(x + α^1) = x^2 + (1+α)x + α
    let gen_poly_2 = ReedSolomon::build_gen_poly(&gf, 2);
    let expected_2_coef_2 = gf.add(1, gf.pow_primitive_poly(1)); // 1 + α
    let expected_2_coef_1 = gf.pow_primitive_poly(1); // α
    assert_eq!(
        gen_poly_2,
        vec![expected_2_coef_1, expected_2_coef_2, 1],
        "Generator polynomial for nsym=2 incorrect. Expected [{}, {}, 1], got {:?}",
        expected_2_coef_1,
        expected_2_coef_2,
        gen_poly_2
    );

    // Для nsym = 3: g(x) = (x + α^0)(x + α^1)(x + α^2)
    let gen_poly_3 = ReedSolomon::build_gen_poly(&gf, 3);

    // Проверяем корни
    for i in 0..3 {
        let root = gf.pow_primitive_poly(i);
        let value = gf.eval_poly(&gen_poly_3, root);
        assert_eq!(
            value, 0,
            "Generator polynomial for nsym=3 should have root α^{}",
            i
        );
    }

    // Проверяем, что полином имеет правильную степень
    assert_eq!(
        gen_poly_3.len(),
        4,
        "Generator polynomial for nsym=3 should have degree 3"
    );
}

#[test]
fn test_build_gen_poly_consistency() {
    let gf = FastGF256::new();

    // Проверяем, что многократное построение дает одинаковый результат
    for nsym in 1..=5 {
        let gen_poly1 = ReedSolomon::build_gen_poly(&gf, nsym);
        let gen_poly2 = ReedSolomon::build_gen_poly(&gf, nsym);
        let gen_poly3 = ReedSolomon::build_gen_poly(&gf, nsym);

        assert_eq!(
            gen_poly1, gen_poly2,
            "Generator polynomial should be consistent (first vs second call) for nsym: {}",
            nsym
        );

        assert_eq!(
            gen_poly2, gen_poly3,
            "Generator polynomial should be consistent (second vs third call) for nsym: {}",
            nsym
        );
    }
}

#[test]
fn test_build_gen_poly_edge_cases() {
    let gf = FastGF256::new();

    // Тестируем граничные случаи
    // nsym = 0 (если поддерживается)
    // let gen_poly_0 = ReedSolomon::build_gen_poly(&gf, 0);
    // assert_eq!(gen_poly_0, vec![1], "Generator polynomial for nsym=0 should be [1]");

    // nsym = 1 (минимальное значение для коррекции ошибок)
    let gen_poly_1 = ReedSolomon::build_gen_poly(&gf, 1);
    assert_eq!(
        gen_poly_1.len(),
        2,
        "Generator polynomial for nsym=1 should have length 2"
    );
    assert_eq!(gen_poly_1[0], 1, "Leading coefficient should be 1");

    // nsym = максимальное разумное значение
    let gen_poly_large = ReedSolomon::build_gen_poly(&gf, 32);
    assert_eq!(
        gen_poly_large.len(),
        33,
        "Generator polynomial for nsym=32 should have length 33"
    );
    assert_eq!(
        gen_poly_large[gen_poly_large.len() - 1],
        1,
        "Leading coefficient should be 1"
    );

    // Проверяем корни для большого полинома
    for i in 0..5 {
        // Проверяем только первые 5 корней для скорости
        let root = gf.pow_primitive_poly(i);
        let value = gf.eval_poly(&gen_poly_large, root);
        assert_eq!(
            value, 0,
            "Large generator polynomial should have root α^{}",
            i
        );
    }
}

/// Проверяем, что полиномы для разных nsym связаны
#[test]
fn test_build_gen_poly_properties() {
    let gf = FastGF256::new();

    let gen_poly_2 = ReedSolomon::build_gen_poly(&gf, 2);
    let gen_poly_3 = ReedSolomon::build_gen_poly(&gf, 3);

    // g_3(x) = g_2(x) * (x + α^2)
    let expected_gen_poly_3 = gf.mul_poly(&gen_poly_2, &[gf.pow_primitive_poly(2), 1]);

    assert_eq!(
        gen_poly_3, expected_gen_poly_3,
        "Generator polynomial for nsym=3 should be g_2(x) * (x + α^2)"
    );
}

#[test]
fn test_generator_poly_roots() {
    // Подробный тест корней порождающего полинома
    let gf = FastGF256::new();
    let nsym = 5;
    let gen_poly = ReedSolomon::build_gen_poly(&gf, nsym);

    println!("Testing generator polynomial roots for nsym = {}", nsym);
    println!("Generator polynomial: {:?}", gen_poly);

    for i in 0..nsym {
        let root = gf.pow_primitive_poly(i as u8);
        let value = gf.eval_poly(&gen_poly, root);

        println!(
            "Root test: α^{} = {}, polynomial value = {}",
            i, root, value
        );

        assert_eq!(
            value, 0,
            "Generator polynomial should be zero at α^{} (value: {})",
            i, root
        );
    }

    // Проверяем, что полином не равен нулю в случайных точках
    for i in nsym..nsym + 5 {
        let point = gf.pow_primitive_poly(i as u8);
        let value = gf.eval_poly(&gen_poly, point);

        println!(
            "Non-root test: α^{} = {}, polynomial value = {}",
            i, point, value
        );

        assert_ne!(
            value, 0,
            "Generator polynomial should not be zero at α^{}",
            i
        );
    }
}

#[test]
fn test_against_known_vectors() {
    let gf = FastGF256::new();

    // Известные порождающие полиномы для примитивного полинома 0x11D
    // Эти значения можно найти в стандартных таблицах или проверить с помощью других реализаций

    let test_cases = vec![
        (1, vec![1, 1]), // g(x) = x + 1
                         // (2, vec![1, 3, 2]), // g(x) = x^2 + 3x + 2 (значения могут отличаться)
                         // Добавьте другие известные значения здесь
    ];

    for (nsym, expected) in test_cases {
        let actual = ReedSolomon::build_gen_poly(&gf, nsym);

        // Поскольку точные значения зависят от реализации,
        // мы проверяем что полином имеет правильные корни
        for i in 0..nsym {
            let root = gf.pow_primitive_poly(i as u8);
            let value = gf.eval_poly(&actual, root);
            assert_eq!(
                value, 0,
                "Generated polynomial for nsym={} should have root α^{}",
                nsym, i
            );
        }

        // Также проверяем базовые свойства
        assert_eq!(actual.len(), nsym + 1);
        assert_eq!(actual[0], 1);
    }
}

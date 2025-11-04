use rand;

use super::super::GF256Poly;
use crate::{Poly, RefPoly};

pub fn test_poly_operations<T: GF256Poly>(gf: T) {
    test_div(&gf);
    test_eval_poly(&gf);
}

pub fn test_eval_poly<T: GF256Poly>(gf: &T) {
    println!("Testing polynomial evaluation...");

    // Полином: 1
    for n in 0..100 {
        let actual = gf.eval_poly(&vec![1], n);
        assert_eq!(1, actual, "eval_poly(1, {}) = {}", n, actual);
    }

    // Полином: x + 1
    for n in 0..100 {
        let actual = gf.eval_poly(&vec![1, 1], n);
        assert_eq!(gf.add(n, 1), actual, "eval_poly(x + 1, {}) = {}", n, actual);
    }

    // Полином: x² + x + 1
    for n in 0..100 {
        let actual = gf.eval_poly(&vec![1, 1, 1], n);

        let mut expected = gf.pow(n, 2);
        expected = gf.add(expected, n);
        expected = gf.add(expected, 1);

        assert_eq!(
            expected, actual,
            "eval_poly(x² + x + 1, {}) = {}",
            n, actual
        );
    }

    // Полином: x³
    for n in 0..100 {
        let actual = gf.eval_poly(&vec![0, 0, 0, 1], n);
        let expected = gf.pow(n, 3);

        assert_eq!(expected, actual, "eval_poly(x³, {}) = {}", n, actual);
    }
}

fn test_div<T: GF256Poly>(gf: &T) {
    println!("  Testing _div()...");

    let check = |input: (&Poly, &Poly), expected: (Poly, Poly)| {
        assert_eq!(
            gf._div_poly(input.0, input.1),
            expected,
            "division failed: (dividend, divisor) = {:?}",
            input
        );
    };

    let gen_poly = |i: usize| {
        rand::random_iter()
            .take(i)
            .map(|n| if n == 0 { 1 } else { n })
            .collect()
    };

    // Деление на самого себя дает (1, 0)
    println!("    Testing _div(1, 0)...");
    for i in 1..=100 {
        let poly = gen_poly(i);
        check((&poly, &poly), (vec![1], vec![0]));
    }

    // Деление 0 дает (0, 0)
    println!("    Testing _div(0, 0)...");
    for i in 1..=100 {
        let poly = gen_poly(i);
        check((&vec![0], &poly), (vec![0], vec![0]));
    }

    // Деление меньшего на больший дает (0, полином)
    println!("    Testing _div(smaller, bigger)...");
    for i in 1..=100 {
        let smaller = gen_poly(i);
        let bigger = gen_poly(i + 1);
        check((&smaller, &bigger), (vec![0], smaller.clone()));
    }

    println!("    Testing _div(cases)...");

    // (x³ + 2x² + 3x + 4) / (x + 1) = x² + 3x с остатком 4 в поле GF(256)
    check((&vec![4, 3, 2, 1], &vec![1, 1]), (vec![0, 3, 1], vec![4]));

    // (x³ + 1) / (x + 1) = x² + x + 1
    check((&vec![1, 0, 0, 1], &vec![1, 1]), (vec![1, 1, 1], vec![0]));

    // (50x² + 1) / 5x = 10x с остатком 1
    check((&vec![1, 0, 50], &vec![0, 5]), (vec![0, 168], vec![1]));
}

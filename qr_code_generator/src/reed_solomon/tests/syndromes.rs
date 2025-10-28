use pretty_assertions::assert_eq;

use super::*;

#[test]
fn test_simple_syndromes() {
    let encoder = create_encoder(5);
    let gf = FastGF256::new();

    let msg = vec![0, 0, 0, 0, 0, 0, 0, 0, 0];
    assert!(
        encoder.calculate_syndromes(&msg).iter().all(|&x| x == 0),
        "syndromes should be all zero"
    );

    let msg = vec![1, 0, 0, 0, 0, 0, 0, 0, 0];
    assert!(
        encoder.calculate_syndromes(&msg).iter().all(|&x| x == 1),
        "syndromes should be all one"
    );

    let msg = vec![0, 1, 0, 0, 0, 0, 0, 0, 0];
    assert_eq!(
        encoder.calculate_syndromes(&msg),
        vec![1, 2, 4, 8, 16],
        "syndromes should be all one"
    );

    let msg = vec![0, 3, 0, 0, 0, 0, 0, 0, 0];
    let expected = vec![1, 2, 4, 8, 16]
        .into_iter()
        .map(|x| gf.mul(x, 3))
        .collect::<Vec<_>>();
    assert_eq!(encoder.calculate_syndromes(&msg), expected);

    let msg = vec![0, 101, 0, 0, 0, 0, 0, 0, 0];
    let expected = vec![1, 2, 4, 8, 16]
        .into_iter()
        .map(|x| gf.mul(x, 101))
        .collect::<Vec<_>>();
    assert_eq!(encoder.calculate_syndromes(&msg), expected);
}

#[test]
fn test_syndromes_for_encoded_stress() {
    let config = StressTestConfig::default();

    stress_test_common(config, |context, encoder, message, encoded, err_encoded| {
        // Проверяем, что для корректно закодированных данных синдромы нулевые
        check_syndromes(&encoder, &encoded).unwrap();
    });
}

#[test]
fn test_syndromes_evaluation_points() {
    let control_count = 4;
    let rs = create_encoder(control_count);
    let gf = &rs.gf;

    let data = vec![1, 2, 3]; // Простой полином: 1 + 2x + 3x²

    let syndromes = rs.calculate_syndromes(&data);

    // Проверяем ручной расчет для первого синдрома (α^0 = 1)
    let manual_s0 = gf.add(gf.add(data[0], gf.mul(data[1], 1)), gf.mul(data[2], 1));
    assert_eq!(
        syndromes[0], manual_s0,
        "S0 should evaluate polynomial at α^0 = 1 \n\
        Manual S0: {manual_s0} \n\
        Syndromes: {syndromes:?}",
    );
}

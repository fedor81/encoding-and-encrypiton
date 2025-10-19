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

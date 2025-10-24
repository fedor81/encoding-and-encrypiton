use super::*;

#[test]
fn decode_fix_len() {
    let encoder = create_encoder(10);
    let len = 10;

    for _ in 0..1000 {
        let message = rand::random_iter().take(len).collect::<Vec<_>>();

        let encoded = encoder.encode(&message).unwrap();
        let decoded = encoder
            .decode(&encoded)
            .with_context(|| format!("message: {message:?}"))
            .unwrap();

        assert_eq!(message, decoded, "encoded: {:?}", encoded);
    }
}

#[test]
fn decode_random_len() {
    let encoder = create_encoder(20);

    for _ in 0..1000 {
        let len = rand::random_range(1..=20); // Случайная длина сообщения
        let message = rand::random_iter().take(len).collect::<Vec<_>>();

        let encoded = encoder.encode(&message).unwrap();
        let decoded = encoder
            .decode(&encoded)
            .with_context(|| format!("message: {message:?}"))
            .unwrap();

        assert_eq!(message, decoded, "encoded: {:?}", encoded);
    }
}

#[test]
fn decode_one_error() {
    let len = 10;
    let control = 10;

    let encoder = create_encoder(control);

    for i in 0..1000 {
        let message = rand::random_iter().take(len).collect::<Vec<_>>();
        let encoded = encoder.encode(&message).unwrap();

        // Внесение ошибки
        let mut err_encoded = encoded.clone();
        let err_index = rand::random_range(0..encoded.len());

        err_encoded[err_index] = rand::random();

        let decoded = encoder
            .decode(&encoded)
            .with_context(|| {
                format!(
                    "\nerror index: {err_index} \n\
                    message: \t{message:?} \n\
                    encoded: \t{encoded:?} \n\
                    err_encoded: \t{err_encoded:?}"
                )
            })
            .unwrap();
        assert_eq!(message, decoded, "encoded: {:?}", encoded);
    }
}

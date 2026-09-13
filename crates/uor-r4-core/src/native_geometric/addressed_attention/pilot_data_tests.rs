use super::*;

#[test]
fn pilot_fixed_window_split_and_oracle_forms() -> std::result::Result<(), EngineError> {
    validate()?;
    let train = training();
    let dev = development();
    assert_eq!(train[0].answer, b" Lima.\n");
    assert_eq!(train[1].answer, b" Oslo.\n");
    assert_eq!(train[2].answer, b" Oslo.\n");
    assert_eq!(train[3].answer, b" Lima.\n");
    assert_eq!(dev[0].answer, b" Rome.\n");
    assert_eq!(dev[1].answer, b" Bath.\n");
    assert_eq!(train[6].answer, b" Oslo.\n");
    assert_eq!(train[7].answer, b" Lima.\n");
    assert_ne!(train[4].prompt, train[5].prompt);
    assert_eq!(train[4].answer, train[5].answer);
    for example in train.iter().chain(&dev) {
        assert!(example.prompt.len() + example.answer.len() + 1 <= 64);
        assert!(example.prompt.ends_with(b"\n"));
        assert!(example.answer.ends_with(b"\n"));
    }
    Ok(())
}

#[test]
fn pilot_ordered_scalar_targets_and_complete_programs() {
    let train = training();
    let dev = development();
    let actual: Vec<_> = train.iter().filter_map(|e| e.expected_scalar).collect();
    assert_eq!(actual, [7, -7, 2, -2, 8, 8, 8, 0]);
    let actual: Vec<_> = dev.iter().filter_map(|e| e.expected_scalar).collect();
    assert_eq!(actual, [5, -5, 12, 0]);
    assert_eq!(dev[4].answer, b"fn main(){println!(\"{}\",5);}\n");
    assert_eq!(dev[5].answer, b"fn main(){println!(\"{}\",-5);}\n");
    // Source compilation/actual generated-program execution belongs to the
    // parent's metered gate; this test does not claim either has occurred.
}

#[test]
fn pilot_digest_binds_every_field_and_order() {
    let original = training();
    let digest = corpus_digest(&original);
    assert_ne!(digest, corpus_digest(&development()));
    for field in 0..6 {
        let mut changed = original.clone();
        match field {
            0 => changed[0].id.push('x'),
            1 => changed[0].family.push('x'),
            2 => changed[0].prompt[0] ^= 1,
            3 => changed[0].answer[0] ^= 1,
            4 => changed[0].expected_scalar = Some(0),
            _ => changed.swap(0, 1),
        }
        assert_ne!(digest, corpus_digest(&changed));
    }
}

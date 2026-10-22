use serde_json::json;
use uor_r4_tokenizer::{training::BpeTrainer, ByteBpeTokenizer};

#[test]
fn reserved_ids_and_count_then_codepoint_tie_order() {
    let mut trainer =
        BpeTrainer::new(261, 2, json!({"type":"ByteLevel","add_prefix_space":false})).unwrap();
    for text in ["ab", "ac", "ab", "ac"] {
        trainer.add_text(text).unwrap();
    }
    let result = trainer.finish().unwrap();
    assert_eq!(result["model"]["merges"][0], json!(["a", "b"]));
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&serde_json::to_vec(&result).unwrap()).unwrap();
    assert_eq!(tokenizer.token_id("<|bos|>"), Some(0));
    assert_eq!(tokenizer.token_id("<|eos|>"), Some(1));
    assert_eq!(tokenizer.token_id("<|unk|>"), Some(2));
    assert_eq!(
        tokenizer.decode(&tokenizer.encode("ab ac λ🦀\n")),
        "ab ac λ🦀\n"
    );
}

#[test]
fn corpus_order_overlap_and_special_boundaries_are_deterministic() {
    let train = |texts: &[&str]| {
        let mut trainer =
            BpeTrainer::new(300, 1, json!({"type":"ByteLevel","add_prefix_space":false})).unwrap();
        for text in texts {
            trainer.add_text(text).unwrap();
        }
        trainer.finish().unwrap()
    };
    let a = train(&["aaaa", "abab", "<|bos|>x<|eos|>y", "aaaa"]);
    let b = train(&["aaaa", "<|bos|>x<|eos|>y", "abab", "aaaa"]);
    assert_eq!(a, b);
    assert_eq!(a["model"]["merges"][0], json!(["a", "a"]));
    assert!(a["model"]["merges"].as_array().unwrap().iter().all(|pair| {
        !pair[0].as_str().unwrap().contains("<|") && !pair[1].as_str().unwrap().contains("<|")
    }));
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&serde_json::to_vec(&a).unwrap()).unwrap();
    assert_eq!(tokenizer.encode("<|bos|>x<|eos|>y")[0], 0);
    assert_eq!(tokenizer.decode(&tokenizer.encode("aaaa")), "aaaa");
}

#[test]
fn digits_prefix_policy_and_limits_are_enforced() {
    use uor_r4_tokenizer::training::TrainingLimits;
    let policy = json!({"type":"Sequence","pretokenizers":[{"type":"Digits","individual_digits":true}, {"type":"ByteLevel","add_prefix_space":true}]});
    let mut trainer = BpeTrainer::new(300, 1, policy).unwrap();
    trainer.add_text("abc123abc123").unwrap();
    let output = trainer.finish().unwrap();
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&serde_json::to_vec(&output).unwrap()).unwrap();
    assert_eq!(tokenizer.decode(&tokenizer.encode("abc123")), " abc 1 2 3");
    assert!(!output["model"]["merges"]
        .as_array()
        .unwrap()
        .iter()
        .any(|pair| pair == &json!(["1", "2"])));
    let mut limited = BpeTrainer::new(259, 1, json!({"type":"ByteLevel"}))
        .unwrap()
        .with_limits(TrainingLimits {
            max_input_bytes: 3,
            max_unique_pretokens: 1,
            max_pretoken_bytes: 3,
        })
        .unwrap();
    assert!(limited.add_text("abcd").is_err());
    assert!(limited.finish().is_err());
    assert!(BpeTrainer::new(258, 1, json!({"type":"ByteLevel"})).is_err());
    assert!(BpeTrainer::new(65537, 1, json!({"type":"ByteLevel"})).is_err());
    assert!(BpeTrainer::new(259, 0, json!({"type":"ByteLevel"})).is_err());
    assert!(BpeTrainer::new(259, 1, json!({"type":"ByteLevel","use_regex":false})).is_err());
    assert!(BpeTrainer::new(259, 1, json!({"type":"Sequence","pretokenizers":[{"type":"Digits","individual_digits":true},{"type":"Digits","individual_digits":false},{"type":"ByteLevel"}]})).is_err());
}

#[test]
fn incremental_merges_match_independent_exhaustive_recount() {
    use std::collections::BTreeMap;
    for case in 0..40u64 {
        let mut state = case + 1;
        let mut texts = Vec::new();
        for _ in 0..24 {
            let mut text = String::new();
            for _ in 0..9 {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                text.push((b'a' + ((state >> 32) % 4) as u8) as char);
            }
            texts.push(text);
        }
        let mut trainer =
            BpeTrainer::new(280, 2, json!({"type":"ByteLevel","add_prefix_space":false})).unwrap();
        for text in &texts {
            trainer.add_text(text).unwrap();
        }
        let actual = trainer.finish().unwrap();
        let mut words: Vec<Vec<String>> = texts
            .iter()
            .map(|s| s.chars().map(|c| c.to_string()).collect())
            .collect();
        let mut vocab: std::collections::BTreeSet<String> =
            (0u8..=255).map(|c| format!("base{c}")).collect();
        let mut expected = Vec::new();
        while vocab.len() + 3 < 280 {
            let mut counts = BTreeMap::<(String, String), u64>::new();
            for word in &words {
                for pair in word.windows(2) {
                    *counts
                        .entry((pair[0].clone(), pair[1].clone()))
                        .or_default() += 1;
                }
            }
            let best = counts
                .into_iter()
                .min_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            let Some(((left, right), count)) = best else {
                break;
            };
            if count < 2 {
                break;
            }
            let merged = format!("{left}{right}");
            vocab.insert(merged.clone());
            expected.push(json!([left, right]));
            for word in &mut words {
                let mut i = 0;
                while i + 1 < word.len() {
                    if word[i] == left && word[i + 1] == right {
                        word[i] = merged.clone();
                        word.remove(i + 1);
                    }
                    i += 1;
                }
            }
        }
        assert_eq!(actual["model"]["merges"], json!(expected), "case {case}");
        let tokenizer =
            ByteBpeTokenizer::from_tokenizer_json_bytes(&serde_json::to_vec(&actual).unwrap())
                .unwrap();
        for (text, symbols) in texts.iter().zip(&words) {
            let expected_ids: Vec<_> = symbols
                .iter()
                .map(|s| actual["model"]["vocab"][s].as_u64().unwrap() as u32)
                .collect();
            assert_eq!(
                tokenizer.encode(text),
                expected_ids,
                "runtime replay case {case}: {text}"
            );
        }
    }
}

#[test]
fn reaches_16384_on_bounded_synthetic_text_and_preserves_runtime_roundtrip() {
    let mut trainer = BpeTrainer::new(
        16_384,
        1,
        json!({"type":"ByteLevel","add_prefix_space":false}),
    )
    .unwrap();
    let mut state = 7u64;
    for _ in 0..8_192 {
        let mut word = String::new();
        for _ in 0..16 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            word.push((b'a' + ((state >> 32) % 26) as u8) as char);
        }
        trainer.add_text(&word).unwrap();
    }
    let output = trainer.finish().unwrap();
    assert_eq!(output["model"]["vocab"].as_object().unwrap().len(), 16_384);
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&serde_json::to_vec(&output).unwrap()).unwrap();
    for text in ["geometric memory", "🦀Привет世界\0", "<|bos|>hello<|eos|>"] {
        let ids = tokenizer.encode(text);
        assert!(ids.iter().all(|&id| u16::try_from(id).is_ok()));
        assert_eq!(tokenizer.decode(&ids), text);
    }
}

#[test]
fn limits_and_exhausted_vocab_fail_closed_without_padding() {
    use uor_r4_tokenizer::training::TrainingLimits;
    let policy = || json!({"type":"ByteLevel","add_prefix_space":false});
    let mut trainer = BpeTrainer::new(16384, 2, policy()).unwrap();
    trainer.add_text("x").unwrap();
    let output = trainer.finish().unwrap();
    assert_eq!(output["model"]["vocab"].as_object().unwrap().len(), 259);
    assert_eq!(output["model"]["merges"], json!([]));
    let mut trainer = BpeTrainer::new(300, 1, policy()).unwrap();
    trainer.add_text("ok").unwrap();
    assert!(trainer.with_limits(TrainingLimits::default()).is_err());
    let mut trainer = BpeTrainer::new(300, 1, policy())
        .unwrap()
        .with_limits(TrainingLimits {
            max_input_bytes: 10,
            max_unique_pretokens: 1,
            max_pretoken_bytes: 3,
        })
        .unwrap();
    trainer.add_text("abc").unwrap();
    assert!(trainer.add_text("def").is_err());
    assert!(trainer.finish().is_err());
    let mut trainer = BpeTrainer::new(300, 1, policy())
        .unwrap()
        .with_limits(TrainingLimits {
            max_input_bytes: 10,
            max_unique_pretokens: 10,
            max_pretoken_bytes: 3,
        })
        .unwrap();
    assert!(trainer.add_text("abcd").is_err());
    assert!(trainer.with_limits(TrainingLimits::default()).is_err());
    assert!(BpeTrainer::new(300, 1, policy()).unwrap().finish().is_err());
}

#[test]
fn unsupported_trim_offsets_is_rejected_instead_of_silently_rewritten() {
    for policy in [
        json!({"type":"ByteLevel","trim_offsets":false}),
        json!({"type":"Sequence","pretokenizers":[{"type":"Digits","individual_digits":true}, {"type":"ByteLevel","trim_offsets":false}]}),
    ] {
        assert!(BpeTrainer::new(300, 1, policy).is_err());
    }
    for policy in [
        json!({"type":"ByteLevel"}),
        json!({"type":"ByteLevel","trim_offsets":true}),
    ] {
        let mut trainer = BpeTrainer::new(300, 1, policy).unwrap();
        trainer.add_text("abc abc").unwrap();
        assert_eq!(
            trainer.finish().unwrap()["pre_tokenizer"]["trim_offsets"],
            true
        );
    }
}

#[test]
fn number_aware_three_arm_comparison_and_digit_piece_census() {
    use uor_r4_tokenizer::training::{
        individual_digits_policy, number_aware_comparison_corpus, number_aware_digits_policy,
        plain_byte_level_policy,
    };

    let render = |v: &serde_json::Value| -> Vec<u8> {
        let mut bytes = serde_json::to_vec(v).unwrap();
        bytes.push(b'\n');
        bytes
    };

    let corpus = number_aware_comparison_corpus();
    assert_eq!(corpus.len(), 170_176);

    for (target_vocab, expected_ctrl1_addr, expected_ctrl2_addr, expected_trt_addr) in [
        (
            4_096usize,
            "blake3:ed3f625d4fbbfbc212ff6ba511943e6130539ce8df533a355648f028b01b548a",
            "blake3:81522e79624f40f83a85df319ef26a12f9a5d313930c9e6873763e91ec70bb5f",
            "blake3:c38dabf516cc142e0c1974d35d807b0040b7c079be32f95c0cdc3fa452d2fc36",
        ),
        (
            16_384usize,
            "blake3:311a1e7c582be16e9620bcc82d66a80fb70d8c160201b02f5f64e5274ff248fa",
            "blake3:ec4c2c66f6bd2a588324bd61ef73fd0be4a59cf1b60c5751bb4f597a297d8a0a",
            "blake3:411c366d7e307a87cfe605e0a8784b7abe184bbb53ea06c455fc2b40f1641ce2",
        ),
    ] {
        // Control 1: Digits(individual_digits=true) + ByteLevel(add_prefix_space=false).
        let mut ctrl1 = BpeTrainer::new(target_vocab, 1, individual_digits_policy(false)).unwrap();
        ctrl1.add_text(&corpus).unwrap();
        let ctrl1_bytes = render(&ctrl1.finish().unwrap());
        let ctrl1_tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&ctrl1_bytes).unwrap();
        assert_eq!(ctrl1_tok.address(), expected_ctrl1_addr);
        let ctrl1_census = ctrl1_tok.digit_piece_census();
        assert_eq!(ctrl1_census.vocab_size, target_vocab);
        assert_eq!(ctrl1_census.pure_single_digit, 10);
        assert_eq!(ctrl1_census.pure_multi_digit, 0);
        assert_eq!(ctrl1_census.space_prefixed_digit, 0);
        assert_eq!(ctrl1_tok.encode("84").len(), 2);
        assert_eq!(ctrl1_tok.encode("48").len(), 2);
        assert_eq!(ctrl1_tok.encode("2026").len(), 4);
        assert_eq!(ctrl1_tok.encode("16384").len(), 5);

        // Control 2: plain ByteLevel(add_prefix_space=false).
        // Because ("Ġ", d1) sums across all d2, it merges before (d1, d2) on space-preceded
        // 2-digit numbers, consuming d1 into "Ġd1" so only the 5 two-digit suffixes of 3+-digit
        // literals form pure multi-digit pieces, while 109+ space-prefixed digit pieces form
        // and unprefixed "48" stays split into 2 single-digit tokens.
        let mut ctrl2 = BpeTrainer::new(target_vocab, 1, plain_byte_level_policy(false)).unwrap();
        ctrl2.add_text(&corpus).unwrap();
        let ctrl2_bytes = render(&ctrl2.finish().unwrap());
        let ctrl2_tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&ctrl2_bytes).unwrap();
        assert_eq!(ctrl2_tok.address(), expected_ctrl2_addr);
        let ctrl2_census = ctrl2_tok.digit_piece_census();
        assert_eq!(ctrl2_census.vocab_size, target_vocab);
        assert_eq!(ctrl2_census.pure_single_digit, 10);
        assert_eq!(ctrl2_census.pure_multi_digit, 5);
        assert!(ctrl2_census.space_prefixed_digit >= 109);
        assert_eq!(ctrl2_tok.encode(" 48").len(), 1);
        assert_eq!(ctrl2_tok.encode("48").len(), 2);

        // Treatment: number-aware Digits(individual_digits=false) + ByteLevel(add_prefix_space=false).
        // Run twice to prove byte-identical output and deterministic identity.
        let mut trt_a =
            BpeTrainer::new(target_vocab, 1, number_aware_digits_policy(false)).unwrap();
        trt_a.add_text(&corpus).unwrap();
        let trt_a_bytes = render(&trt_a.finish().unwrap());

        let mut trt_b =
            BpeTrainer::new(target_vocab, 1, number_aware_digits_policy(false)).unwrap();
        trt_b.add_text(&corpus).unwrap();
        let trt_b_bytes = render(&trt_b.finish().unwrap());
        assert_eq!(trt_a_bytes, trt_b_bytes);

        let trt_tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&trt_a_bytes).unwrap();
        assert_eq!(trt_tok.address(), expected_trt_addr);
        let trt_census = trt_tok.digit_piece_census();
        assert_eq!(trt_census.vocab_size, target_vocab);
        assert_eq!(trt_census.pure_single_digit, 10);
        assert!(trt_census.pure_multi_digit >= 109);
        assert!(trt_census.pure_two_digit >= 100);
        assert!(trt_census.pure_three_digit >= 5);
        assert!(trt_census.pure_four_plus_digit >= 3);
        assert_eq!(trt_census.space_prefixed_digit, 0);
        assert_eq!(trt_census.mixed_digit_other, 0);

        let enc_84 = trt_tok.encode("84");
        let enc_48 = trt_tok.encode("48");
        let enc_sp_84 = trt_tok.encode(" 84");
        let enc_2026 = trt_tok.encode("2026");
        let enc_16384 = trt_tok.encode("16384");
        assert_eq!(enc_84.len(), 1);
        assert_eq!(enc_48.len(), 1);
        assert_ne!(enc_84, enc_48);
        assert_eq!(enc_sp_84.len(), 2);
        assert_eq!(enc_sp_84[1], enc_84[0]);
        assert_eq!(enc_2026.len(), 1);
        assert_eq!(enc_16384.len(), 1);

        let probe = "84 48 100 128 240 256 512 1024 2026 4096 16384 (84) [48] x=84";
        let ids = trt_tok.encode(probe);
        assert_eq!(trt_tok.decode(&ids), probe);
    }
}

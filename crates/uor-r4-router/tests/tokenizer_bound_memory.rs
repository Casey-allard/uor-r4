use uor_r4_router::{decoder_memory::DecoderMemoryError, UorR4Router};

#[test]
fn tokenizer_bound_turns_round_trip_and_remain_identity_scoped() {
    let mut router = UorR4Router::new(0.5);
    let identity = "issue-950-alice";
    let tokenizer = "blake3:tokenizer";
    let adapter = "blake3:adapter";
    let source = "blake3:source";

    let user = router
        .commit_tokenizer_bound_turn(
            identity,
            "user",
            "Please remember that my project color is cobalt blue.",
            &[11, 12, 13, 14],
            tokenizer,
            adapter,
            source,
        )
        .expect("commit user turn");
    let assistant = router
        .commit_tokenizer_bound_turn(
            identity,
            "assistant",
            "I will remember that the project color is cobalt blue.",
            &[21, 22, 23, 24],
            tokenizer,
            adapter,
            source,
        )
        .expect("commit assistant turn");
    assert_eq!((user.sequence, assistant.sequence), (0, 1));
    assert_ne!(user.r4_coordinates, [0.0; 4]);

    let exported = router.export_state();
    let before_cid = router
        .tokenizer_bound_state_cid(identity)
        .expect("bound state cid");
    let mut restored = UorR4Router::new(0.5);
    assert!(restored.import_state_native(&exported));
    let after_cid = restored
        .tokenizer_bound_state_cid(identity)
        .expect("restored state cid");
    assert_eq!(after_cid, before_cid);

    let memories = restored
        .tokenizer_bound_turns(identity, tokenizer, adapter, source)
        .expect("same binding retrieves");
    assert_eq!(memories, vec![user, assistant]);
    assert!(restored
        .tokenizer_bound_turns("issue-950-bob", tokenizer, adapter, source)
        .expect("other identity is empty")
        .is_empty());
    assert_eq!(
        restored
            .tokenizer_bound_turns(identity, "blake3:other-tokenizer", adapter, source)
            .expect_err("cross-tokenizer retrieval must fail closed"),
        DecoderMemoryError::BindingMismatch
    );
    assert_eq!(
        restored
            .commit_tokenizer_bound_turn(
                identity,
                "user",
                "This must not poison the bound stream.",
                &[31, 32],
                "blake3:other-tokenizer",
                adapter,
                source,
            )
            .expect_err("cross-tokenizer commit must fail before mutation"),
        DecoderMemoryError::BindingMismatch
    );
    assert_eq!(
        restored
            .tokenizer_bound_turns(identity, tokenizer, adapter, source)
            .expect("original binding remains readable"),
        memories
    );
}

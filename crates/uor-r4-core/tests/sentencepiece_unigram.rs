//! #639-3a: the SentencePiece Unigram Viterbi core reproduces the reference
//! token ids on the real pinned T5 `spiece.model`.
//!
//! Presence-gated (#599 three-state): when the pinned `spiece.model`
//! snapshot is present the parser + Viterbi must reproduce the reference
//! `sentencepiece` token ids byte-for-byte; when it is absent the test
//! reports UNAVAILABLE and passes vacuously — never a silent skip.
//!
//! Each fixture is `(normalized, ids)` where `normalized` is the exact
//! surface string the reference tokenizer produces after normalization —
//! the concatenation of `sp.encode(text, out_type=str)` pieces, so an
//! unknown character appears as its literal self. Feeding that normalized
//! string to [`UnigramModel::encode_normalized`] isolates the Viterbi core
//! from the normalizer (which lands in #639-3b): the ids must match
//! `sp.encode(text, out_type=int)`. Fixtures were generated from
//! `google-t5/t5-base` revision `a9723ea7…` and span ASCII, accents, mixed
//! case, digits, collapsed whitespace, single unknowns, adjacent-unknown
//! collapse (CJK run), and non-adjacent unknowns.

use std::path::PathBuf;

use uor_r4_core::transformerless::hf_bpe::{adapter_constructor, TokenizerAdapter};
use uor_r4_core::transformerless::sentencepiece::{SentencePieceUnigramTokenizer, UnigramModel};

/// #639-3b full-pipeline fixtures: raw text → the reference `sentencepiece`
/// token ids (from `google-t5/t5-base` revision `a9723ea7…`). Unlike the
/// `FIXTURES` above (which pre-normalize to isolate the Viterbi), these feed
/// RAW text through the whole adapter — normalizer (precompiled charsmap +
/// whitespace) then Viterbi — so they exercise the charsmap folding:
/// fullwidth (Ａ→A, ①②③→123, ＵＯＲ), ligatures (ﬁ→fi, ﬀﬃ), fractions (½),
/// Roman numerals (Ⅻ), squared forms (㍿), tabs/newlines, whitespace
/// collapse, and CJK/emoji → `<unk>`.
const RAW_FIXTURES: &[(&str, &[u32])] = &[
    ("Hello world", &[8774, 296]),
    ("Hello  world", &[8774, 296]),
    ("The quick brown fox.", &[37, 1704, 4216, 3, 20400, 5]),
    ("café", &[11949]),
    ("naïve résumé", &[3, 29, 9, 2, 162, 1417, 4078, 154]),
    (
        "ﬁ ligature and ½ fraction",
        &[361, 3, 17140, 2693, 11, 209, 2, 357, 12211],
    ),
    ("ｆｕｌｌｗｉｄｔｈ", &[423, 12018, 189]),
    ("emoji 😀 test", &[3, 15, 51, 21892, 3, 2, 794]),
    ("日本語のテキスト", &[3, 2]),
    (" leading and  double   spaces ", &[1374, 11, 1486, 4856]),
    ("H2O molecule", &[454, 357, 667, 3, 23098]),
    ("MixedCASE Text", &[28024, 254, 17892, 5027]),
    ("A🎉B🎉C", &[71, 2, 279, 2, 254]),
    (
        "Numbers: 12345 and 6.7",
        &[7720, 7, 10, 3, 14574, 2128, 11, 3, 29045],
    ),
    ("Ａ１２３", &[71, 14574]),
    ("①②③", &[3, 14574]),
    ("ＵＯＲ", &[412, 2990]),
    ("\tTab\nNL", &[10456, 3, 18207]),
    ("Ⅻ roman", &[3, 4, 196, 196, 3408]),
    ("㍿ square", &[3, 2, 2812]),
    ("½+¼", &[209, 2, 357, 18446, 2, 591]),
    (
        "ﬀ ﬃ ligatures",
        &[3, 89, 89, 3, 89, 89, 23, 3, 17140, 10471],
    ),
];

/// Workspace root: `CARGO_MANIFEST_DIR` is `<root>/crates/uor-r4-core`.
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// `(normalized surface string, reference token ids)` from the pinned T5
/// tokenizer.
const FIXTURES: &[(&str, &[u32])] = &[
    ("\u{2581}Hello\u{2581}world", &[8774, 296]),
    (
        "\u{2581}The\u{2581}quick\u{2581}brown\u{2581}fox.",
        &[37, 1704, 4216, 3, 20400, 5],
    ),
    ("\u{2581}H2O\u{2581}molecule", &[454, 357, 667, 3, 23098]),
    (
        "\u{2581}leading\u{2581}and\u{2581}double\u{2581}spaces",
        &[1374, 11, 1486, 4856],
    ),
    (
        "\u{2581}Numbers:\u{2581}12345\u{2581}and\u{2581}6.7",
        &[7720, 7, 10, 3, 14574, 2128, 11, 3, 29045],
    ),
    ("\u{2581}MixedCASE\u{2581}Text", &[28024, 254, 17892, 5027]),
    (
        "\u{2581}naïve\u{2581}résumé",
        &[3, 29, 9, 2, 162, 1417, 4078, 154],
    ),
    (
        "\u{2581}emoji\u{2581}😀\u{2581}test",
        &[3, 15, 51, 21892, 3, 2, 794],
    ),
    ("\u{2581}日本語のテキスト", &[3, 2]),
    ("\u{2581}A🎉B🎉C", &[71, 2, 279, 2, 254]),
    (
        "\u{2581}supercalifragilistic",
        &[1355, 15534, 20791, 173, 3040],
    ),
];

fn spiece_bytes() -> Option<Vec<u8>> {
    // The pinned source directory recorded by #639-1.
    let descriptor = repo_root().join("models").join("t5-base-tokenizer.json");
    let bytes = std::fs::read(&descriptor)
        .unwrap_or_else(|error| panic!("{}: {error}", descriptor.display()));
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).expect("models/t5-base-tokenizer.json parses");
    let source_dir = value["source_directory"]
        .as_str()
        .expect("source_directory is declared");
    let path = repo_root().join(source_dir).join("spiece.model");
    if !path.is_file() {
        eprintln!(
            "UNAVAILABLE: real T5 spiece.model absent at {} — presence-gated Unigram core check skipped",
            path.display()
        );
        return None;
    }
    Some(std::fs::read(&path).expect("read spiece.model"))
}

#[test]
fn real_t5_unigram_core_matches_reference_ids() {
    let Some(bytes) = spiece_bytes() else {
        return;
    };
    let model = UnigramModel::from_spiece_bytes(&bytes).expect("T5 spiece.model parses as Unigram");

    // The pinned T5 vocabulary geometry.
    assert_eq!(model.vocab_size(), 32000, "T5 pins a 32000-piece vocab");
    assert_eq!(model.unk_id(), 2, "T5 <unk> is id 2");

    for (normalized, expected) in FIXTURES {
        let ids = model.encode_normalized(normalized);
        assert_eq!(
            ids, *expected,
            "Unigram Viterbi must reproduce the reference ids for {normalized:?}"
        );
    }
}

#[test]
fn real_t5_unigram_encode_is_deterministic() {
    let Some(bytes) = spiece_bytes() else {
        return;
    };
    let model = UnigramModel::from_spiece_bytes(&bytes).expect("T5 spiece.model parses as Unigram");
    for (normalized, _) in FIXTURES {
        let first = model.encode_normalized(normalized);
        let second = model.encode_normalized(normalized);
        assert_eq!(first, second, "encode is deterministic for {normalized:?}");
    }
}

#[test]
fn real_t5_sentencepiece_adapter_matches_reference_ids() {
    let Some(bytes) = spiece_bytes() else {
        return;
    };
    let tokenizer =
        SentencePieceUnigramTokenizer::from_spiece_bytes(&bytes).expect("T5 spiece.model parses");

    use uor_r4_core::transformerless::hf_bpe::TokenizerModel;
    for (text, expected) in RAW_FIXTURES {
        let ids = tokenizer.encode(text);
        assert_eq!(
            ids, *expected,
            "full adapter (normalizer + Viterbi) must reproduce the reference ids for {text:?}"
        );
        // Determinism: a second encode is byte-identical.
        assert_eq!(
            tokenizer.encode(text),
            ids,
            "encode is deterministic for {text:?}"
        );
    }

    // The declared adapter identity binds the right family/version/CID.
    let adapter = tokenizer.adapter();
    assert_eq!(
        adapter.family,
        TokenizerAdapter::SENTENCEPIECE_UNIGRAM_FAMILY
    );
    assert_eq!(
        adapter.version,
        TokenizerAdapter::SENTENCEPIECE_UNIGRAM_VERSION
    );
    assert_eq!(
        adapter.tokenizer_cid,
        format!("blake3:{}", blake3::hash(&bytes).to_hex()),
        "tokenizer_cid is the blake3 of the spiece.model bytes"
    );
    assert_eq!(
        adapter.adapter_digest,
        adapter.declared_digest(),
        "adapter_digest is the declared-identity digest"
    );
}

#[test]
fn registry_resolves_sentencepiece_unigram_1() {
    // #639-3b: the family that was refused-by-name now resolves to the real
    // Unigram adapter.
    let constructor = adapter_constructor(
        TokenizerAdapter::SENTENCEPIECE_UNIGRAM_FAMILY,
        TokenizerAdapter::SENTENCEPIECE_UNIGRAM_VERSION,
    )
    .expect("sentencepiece-unigram/1 is registered");
    // Malformed bytes stay total (None, not a panic).
    assert!(constructor(b"not a spiece.model").is_none());

    let Some(bytes) = spiece_bytes() else {
        return;
    };
    use uor_r4_core::transformerless::hf_bpe::TokenizerModel;
    let via_registry = constructor(&bytes).expect("registry constructor parses the real model");
    let direct = SentencePieceUnigramTokenizer::from_spiece_bytes(&bytes).expect("direct parse");
    assert_eq!(via_registry.adapter(), direct.adapter());
    assert_eq!(
        via_registry.encode("Hello world"),
        direct.encode("Hello world")
    );
    assert_eq!(
        via_registry.encode("Ａ１２３ ﬁ ½"),
        vec![71, 14574, 361, 209, 2, 357]
    );
}

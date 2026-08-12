//! Versioned tokenizer adapters and differential source fixtures
//! (issue #601).
//!
//! Three suites:
//!
//! 1. **Adapter identity** — the typed [`TokenizerAdapter`] record derived
//!    from a parsed `tokenizer.json`: pinned canonical serialization,
//!    digest, serde round-trip, and the versioned `(family, version)`
//!    registry with its explicit unknown-family rejection (including the
//!    recorded `sentencepiece-unigram` follow-up).
//! 2. **Differential fixtures** — table-driven encode/decode witnesses
//!    against a full-byte-alphabet fixture tokenizer. The Hugging Face
//!    source snapshot is not present in this environment, so the expected
//!    token ids are pinned constants derived from the CURRENT verified
//!    behavior (the post-#242/#253 implementation, whose segmentation the
//!    existing `hf_bpe_tokenizer.rs` witnesses established against the
//!    teacher's rule). They are the differential baseline: any encode or
//!    decode drift fails these tables.
//! 3. **Consumer agreement** — the observation, evaluation, serving-prompt,
//!    and exported-runtime-tokenizer selection seams resolve the same
//!    adapter identity, token ids, and decode bytes for the same input.

use std::path::PathBuf;

use uor_r4_core::transformerless::hf_bpe::{
    adapter_constructor, HfBpeTokenizer, TokenizerAdapter, TokenizerKind,
};
use uor_r4_core::transformerless::scenarios;

// =====================================================================
// Fixture: a byte-level BPE tokenizer with the FULL 256-symbol GPT-2
// byte alphabet (id = byte value), 13 merged tokens (dense ids
// 256..=268), and three added tokens. Multi-byte input without a merge
// (CJK, emoji) therefore encodes to its raw byte values — analytically
// checkable AND pinned.
// =====================================================================

/// Merged-token contents, id = 256 + index.
const MERGED: [&str; 13] = [
    "he", "ll", "hell", "hello", "Ġw", "Ġwo", "Ġwor", "Ġworl", "Ġworld", "Ã©", "12", "ab", "bc",
];

/// Ordered merges (rank = position). Note "b c" (rank 11) outranks
/// "a b" (rank 12): the discriminating rank-order configuration.
const MERGES: [&str; 13] = [
    "h e", "l l", "he ll", "hell o", "Ġ w", "Ġw o", "Ġwo r", "Ġwor l", "Ġworl d", "Ã ©", "1 2",
    "b c", "a b",
];

const ADDED: [(u32, &str); 3] = [(269, "<|bos|>"), (270, "<|eos|>"), (271, "<|end|>")];

/// Independent re-derivation of the GPT-2 byte→unicode table (printable
/// latin-1 bytes map to themselves, the remaining 68 bytes to U+0100..
/// in ascending order), written from the published rule rather than the
/// implementation under test.
fn byte_symbol(byte: u8) -> char {
    let printable = |b: u8| (b'!'..=b'~').contains(&b) || (0xA1..=0xAC).contains(&b) || b >= 0xAE;
    if printable(byte) {
        char::from_u32(u32::from(byte)).expect("latin-1 codepoint")
    } else {
        let extra = (0..byte).filter(|&b| !printable(b)).count() as u32;
        char::from_u32(256 + extra).expect("fallback codepoint")
    }
}

/// Serialize the fixture `tokenizer.json`. `digits` selects the
/// SmolLM2-shaped pre-tokenizer sequence (`Digits{individual_digits:
/// true}` then `ByteLevel`); otherwise plain `ByteLevel`.
fn fixture_json(digits: bool) -> Vec<u8> {
    let mut vocab = serde_json::Map::new();
    for byte in 0u8..=255 {
        vocab.insert(byte_symbol(byte).to_string(), serde_json::json!(byte));
    }
    for (index, token) in MERGED.iter().enumerate() {
        vocab.insert((*token).to_string(), serde_json::json!(256 + index));
    }
    let pre_tokenizer = if digits {
        serde_json::json!({"type": "Sequence", "pretokenizers": [
            {"type": "Digits", "individual_digits": true},
            {"type": "ByteLevel", "add_prefix_space": false, "trim_offsets": true, "use_regex": true}
        ]})
    } else {
        serde_json::json!({"type": "ByteLevel", "add_prefix_space": false})
    };
    let added: Vec<serde_json::Value> = ADDED
        .iter()
        .map(|(id, content)| serde_json::json!({"id": id, "content": content, "special": true}))
        .collect();
    serde_json::to_vec(&serde_json::json!({
        "added_tokens": added,
        "pre_tokenizer": pre_tokenizer,
        "model": {"type": "BPE", "vocab": vocab, "merges": MERGES.to_vec()}
    }))
    .expect("fixture serializes")
}

fn fixture_tokenizer(digits: bool) -> HfBpeTokenizer {
    HfBpeTokenizer::from_tokenizer_json_bytes(&fixture_json(digits))
        .expect("fixture tokenizer.json parses")
}

// =====================================================================
// Suite 2: differential encode/decode fixtures.
// =====================================================================

struct Case {
    name: &'static str,
    text: &'static str,
    /// Pinned expected ids: the current verified behavior recorded as
    /// the differential baseline (see module docs).
    ids: &'static [u32],
}

/// Cases against the SmolLM2-shaped fixture (Digits + ByteLevel).
const DIGITS_CASES: [Case; 12] = [
    Case {
        name: "ascii merges",
        text: "hello world",
        ids: &[259, 264],
    },
    Case {
        name: "ascii whitespace run keeps last space for the next word",
        text: "hello  world",
        ids: &[259, 32, 264],
    },
    Case {
        name: "contraction binds without a preceding space",
        text: "it's",
        ids: &[105, 116, 39, 115],
    },
    Case {
        name: "merge RANK order beats lowest-token-id greedy",
        text: "abc",
        ids: &[97, 268],
    },
    Case {
        name: "accented letter merges through its byte-level symbols",
        text: "héllo",
        ids: &[104, 265, 257, 111],
    },
    Case {
        name: "CJK falls back to per-byte tokens",
        text: "中文",
        ids: &[228, 184, 173, 230, 150, 135],
    },
    Case {
        name: "emoji falls back to per-byte tokens",
        text: "🙂",
        ids: &[240, 159, 153, 130],
    },
    Case {
        name: "emoji ZWJ sequence stays one pre-token of raw bytes",
        text: "👩\u{200d}💻",
        ids: &[240, 159, 145, 169, 226, 128, 141, 240, 159, 146, 187],
    },
    Case {
        name: "byte fallback for a bare latin-1 character",
        text: "ÿ",
        ids: &[195, 191],
    },
    Case {
        name: "added token matched atomically",
        text: "hello<|end|>hello",
        ids: &[259, 271, 259],
    },
    Case {
        name: "explicit BOS/EOS text maps to added tokens; none are auto-inserted",
        text: "<|bos|>hello<|eos|>",
        ids: &[269, 259, 270],
    },
    Case {
        name: "Digits isolates every digit (no merge crosses a digit boundary)",
        text: "hello 12",
        ids: &[259, 32, 49, 50],
    },
];

#[test]
fn differential_fixture_encode_and_round_trip_decode() {
    let tokenizer = fixture_tokenizer(true);
    for case in &DIGITS_CASES {
        let ids = tokenizer.encode(case.text);
        assert_eq!(ids, case.ids, "encode drift on case {:?}", case.name);
        // Round-trip: decoded text and RAW decoded bytes both reproduce
        // the input exactly (byte-level BPE is total; add_prefix_space
        // is false in the fixture).
        assert_eq!(
            tokenizer.decode(&ids),
            case.text,
            "decode drift on case {:?}",
            case.name
        );
        assert_eq!(
            tokenizer.decode_bytes(&ids),
            case.text.as_bytes(),
            "decode-bytes drift on case {:?}",
            case.name
        );
        // encode_lossy replaces nothing: the byte alphabet is total.
        let (lossy_ids, replaced) = tokenizer.encode_lossy(case.text);
        assert_eq!(lossy_ids, ids);
        assert_eq!(replaced, 0, "case {:?}", case.name);
    }
}

#[test]
fn no_bos_or_eos_is_auto_inserted() {
    let tokenizer = fixture_tokenizer(true);
    let ids = tokenizer.encode("hello");
    assert_eq!(ids, vec![259], "no BOS/EOS wraps a plain encode");
}

#[test]
fn digits_pre_tokenization_boundaries_differ_from_plain_byte_level() {
    let digits = fixture_tokenizer(true);
    let plain = fixture_tokenizer(false);
    // Same merges table; only the Digits step separates them. The "1 2"
    // merge applies without Digits and is blocked by it.
    assert_eq!(plain.encode("12"), vec![266]);
    assert_eq!(digits.encode("12"), vec![49, 50]);
    assert_eq!(plain.encode("hello 12"), vec![259, 32, 266]);
    assert_eq!(digits.encode("hello 12"), vec![259, 32, 49, 50]);
    // Individual digits split even inside letter/digit alternation.
    assert_eq!(digits.encode("a1b2"), vec![97, 49, 98, 50]);
}

#[test]
fn partial_utf8_token_decodes_its_true_bytes() {
    let tokenizer = fixture_tokenizer(true);
    // Token id 255 is the single byte 0xFF — not valid UTF-8 alone. The
    // raw byte surface keeps it exact; the string surface is lossy.
    assert_eq!(tokenizer.decode_bytes(&[255]), vec![0xFF]);
    assert_eq!(tokenizer.decode(&[255]), "\u{FFFD}");
    assert_eq!(tokenizer.token_byte_lengths()[255], 1);
}

// =====================================================================
// Suite 1: adapter identity, canonical serialization, registry.
// =====================================================================

/// Independent recomputation of the canonical added-token listing
/// digest: entries sorted by id, each `<id>:<byte length>:<content>\n`.
fn expected_added_tokens_digest() -> String {
    let mut listing = Vec::new();
    for (id, content) in ADDED {
        listing.extend_from_slice(format!("{id}:{}:", content.len()).as_bytes());
        listing.extend_from_slice(content.as_bytes());
        listing.push(b'\n');
    }
    format!("blake3:{}", blake3::hash(&listing).to_hex())
}

#[test]
fn adapter_record_is_pinned_and_canonically_serialized() {
    let bytes = fixture_json(true);
    let tokenizer = HfBpeTokenizer::from_tokenizer_json_bytes(&bytes).expect("fixture parses");
    let adapter = tokenizer.adapter();

    assert_eq!(adapter.family, TokenizerAdapter::HF_BYTE_BPE_FAMILY);
    assert_eq!(adapter.family, "hf-byte-bpe");
    assert_eq!(adapter.version, TokenizerAdapter::HF_BYTE_BPE_VERSION);
    assert_eq!(adapter.version, 1);
    // tokenizer_cid is the blake3 of the raw tokenizer.json bytes —
    // exactly how tokenizer CIDs are formed today (HfBpeTokenizer::address).
    let cid = format!("blake3:{}", blake3::hash(&bytes).to_hex());
    assert_eq!(adapter.tokenizer_cid, cid);
    assert_eq!(adapter.tokenizer_cid, tokenizer.address());

    // Pinned policy tokens.
    assert_eq!(adapter.policy.normalizer, "none");
    assert_eq!(
        adapter.policy.pre_tokenizers,
        vec![
            "digits(individual_digits=true)".to_owned(),
            "byte-level(add_prefix_space=false)".to_owned(),
        ]
    );
    assert_eq!(adapter.policy.byte_fallback, "byte-level-alphabet");
    assert_eq!(adapter.policy.added_tokens_count, 3);
    assert_eq!(
        adapter.policy.added_tokens_digest,
        expected_added_tokens_digest()
    );
    assert_eq!(adapter.policy.bos, "none");
    assert_eq!(adapter.policy.eos, "none");
    assert_eq!(adapter.policy.chat_template_policy, "not-interpreted");

    // Canonical serialization is pinned byte-for-byte: any drift in
    // field order, separators, or policy tokens fails here — the digest
    // identity must not move silently.
    let pinned = format!(
        "uor-r4-tokenizer-adapter/1\n\
         family=hf-byte-bpe\n\
         version=1\n\
         tokenizer_cid={cid}\n\
         policy.normalizer=none\n\
         policy.pre_tokenizers=digits(individual_digits=true),byte-level(add_prefix_space=false)\n\
         policy.byte_fallback=byte-level-alphabet\n\
         policy.added_tokens_count=3\n\
         policy.added_tokens_digest={added}\n\
         policy.bos=none\n\
         policy.eos=none\n\
         policy.chat_template_policy=not-interpreted\n",
        added = expected_added_tokens_digest(),
    );
    assert_eq!(adapter.canonical_bytes(), pinned.as_bytes());
    let expected_digest = format!("blake3:{}", blake3::hash(pinned.as_bytes()).to_hex());
    assert_eq!(adapter.adapter_digest, expected_digest);
    assert_eq!(adapter.declared_digest(), expected_digest);

    // Rebuilding the record reproduces it bit-for-bit.
    assert_eq!(tokenizer.adapter(), adapter);
}

#[test]
fn adapter_record_round_trips_through_serde_json() {
    let adapter = fixture_tokenizer(true).adapter();
    let json = serde_json::to_string(&adapter).expect("serializes");
    let back: TokenizerAdapter = serde_json::from_str(&json).expect("deserializes");
    assert_eq!(adapter, back);
    // Serde-defaulted fields: a legacy/partial document still parses.
    let partial: TokenizerAdapter =
        serde_json::from_str("{\"family\":\"hf-byte-bpe\"}").expect("defaults fill in");
    assert_eq!(partial.family, "hf-byte-bpe");
    assert_eq!(partial.version, 0);
    assert_eq!(partial.policy.pre_tokenizers, Vec::<String>::new());
}

#[test]
fn pre_tokenizer_variant_changes_the_adapter_digest() {
    // Same vocabulary/merges, different pre-tokenizer policy → distinct
    // adapter identity AND distinct tokenizer CID.
    let digits = fixture_tokenizer(true).adapter();
    let plain = fixture_tokenizer(false).adapter();
    assert_ne!(digits.adapter_digest, plain.adapter_digest);
    assert_ne!(digits.tokenizer_cid, plain.tokenizer_cid);
    assert_eq!(
        plain.policy.pre_tokenizers,
        vec!["byte-level(add_prefix_space=false)".to_owned()]
    );
}

#[test]
fn registry_resolves_hf_byte_bpe_1() {
    let constructor = adapter_constructor(
        TokenizerAdapter::HF_BYTE_BPE_FAMILY,
        TokenizerAdapter::HF_BYTE_BPE_VERSION,
    )
    .expect("registered constructor");
    let bytes = fixture_json(true);
    let via_registry = constructor(&bytes).expect("registry constructor parses the fixture");
    let direct = HfBpeTokenizer::from_tokenizer_json_bytes(&bytes).expect("direct parse");
    assert_eq!(via_registry.adapter(), direct.adapter());
    assert_eq!(
        via_registry.encode("hello world"),
        direct.encode("hello world")
    );
    // The constructor stays total in the module's existing convention:
    // malformed bytes are None, not a panic.
    assert!(constructor(b"not json").is_none());
}

#[test]
fn registry_refuses_unknown_family_and_version_by_name() {
    // The recorded SentencePiece/Unigram follow-up family is rejected
    // explicitly (bounded rejection, #601 non-goal), as is any unknown
    // (family, version) pair — including a bumped hf-byte-bpe version.
    for (family, version) in [
        (TokenizerAdapter::SENTENCEPIECE_UNIGRAM_FAMILY, 1u32),
        (TokenizerAdapter::HF_BYTE_BPE_FAMILY, 2),
        ("mystery-tokenizer", 1),
    ] {
        let error = adapter_constructor(family, version)
            .expect_err("unknown (family, version) is not a product");
        match &error.kind {
            uor_r4_model_source::SourceIngestKind::UnknownTokenizerAdapter {
                family: got_family,
                version: got_version,
            } => {
                assert_eq!(got_family, family);
                assert_eq!(*got_version, version);
            }
            other => panic!("wrong failure class: {other:?}"),
        }
        assert!(
            error.reason.contains(family),
            "reason names the family: {error}"
        );
    }
    // The rejection message records the follow-up path by name.
    let error = adapter_constructor(TokenizerAdapter::SENTENCEPIECE_UNIGRAM_FAMILY, 1)
        .expect_err("sentencepiece-unigram has no adapter yet");
    assert!(
        error.reason.contains("sentencepiece-unigram"),
        "follow-up family is named: {error}"
    );
}

// =====================================================================
// Suite 3: consumer agreement across tokenizer selection seams.
// =====================================================================

fn unique_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "uor-r4-i601-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&dir).expect("create fixture dir");
    dir
}

/// The four consumer paths resolve their tokenizer at these seams:
///
/// - **observation path** (`uor-r4-graph-cli::observe_text_command` and
///   `observe_text_batched_command`): `TokenizerKind::HfBpe(Box::new(
///   HfBpeTokenizer::from_dir(&options.source)?))` when the snapshot has
///   a `tokenizer.json` — asserted below by building exactly that value.
/// - **evaluation path** (`uor-r4-graph-cli::evaluate_report`):
///   `HfBpeTokenizer::from_dir(&options.source).ok()` — asserted below
///   by building exactly that value.
/// - **serving prompt path** (`src/server.rs::load_serving_hf_tokenizer`):
///   `HfBpeTokenizer::from_dir(dir)` into `SERVING_HF_TOKENIZER` —
///   asserted below at the same `from_dir` seam (the server's static is
///   private to the binary crate, so the selection expression is
///   exercised, not the static).
/// - **exported runtime tokenizer path** (`observe-text` exports
///   `tokenizer.bin` via `export_hf_bytelevel_tokenizer_with_lengths`;
///   `uor-r4-api::R4Engine::load` pins its blake3 against the graph
///   head's `tokenizer_cid` and serving decode falls back to the legacy
///   `scenarios::Tokenizer` over it): asserted below by exporting,
///   reloading with the legacy loader, and comparing per-id decode
///   bytes. Encode agreement is NOT asserted for the legacy fallback —
///   its greedy segmentation is the recorded #242/#285 serving
///   difference — the runtime path consumes this table for id-space
///   decode, which must and does agree.
#[test]
fn consumer_paths_agree_on_adapter_identity_token_ids_and_decode_bytes() {
    let dir = unique_dir("consumer-agreement");
    let json_bytes = fixture_json(true);
    std::fs::write(dir.join("tokenizer.json"), &json_bytes).expect("write tokenizer.json");

    // Observation-path selection (graph-cli observe-text, serial and
    // batched drivers use this identical expression).
    let observation = TokenizerKind::HfBpe(Box::new(
        HfBpeTokenizer::from_dir(&dir).expect("observation-path tokenizer loads"),
    ));
    // Evaluation-path selection (graph-cli evaluate_report).
    let evaluation = HfBpeTokenizer::from_dir(&dir).expect("evaluation-path tokenizer loads");
    // Serving-prompt-path selection (src/server.rs load_serving_hf_tokenizer).
    let serving = HfBpeTokenizer::from_dir(&dir).expect("serving-path tokenizer loads");

    // Adapter identity: all three HF selections resolve the same record,
    // and its CID is the blake3 of the tokenizer bytes — the same CID
    // rule `R4Engine::load` verifies tokenizer bytes against.
    let observation_adapter = observation
        .adapter()
        .expect("the HF observation path declares an adapter");
    assert_eq!(observation_adapter, evaluation.adapter());
    assert_eq!(observation_adapter, serving.adapter());
    assert_eq!(
        observation_adapter.tokenizer_cid,
        format!("blake3:{}", blake3::hash(&json_bytes).to_hex())
    );
    assert_eq!(
        observation_adapter.adapter_digest,
        observation_adapter.declared_digest()
    );

    // Token ids and decode bytes agree across the encode seams.
    for case in &DIGITS_CASES {
        let ids = observation.encode(case.text);
        assert_eq!(ids, evaluation.encode(case.text), "case {:?}", case.name);
        assert_eq!(ids, serving.encode(case.text), "case {:?}", case.name);
        assert_eq!(
            evaluation.decode_bytes(&ids),
            case.text.as_bytes(),
            "case {:?}",
            case.name
        );
        assert_eq!(observation.decode(&ids), case.text, "case {:?}", case.name);
        assert_eq!(serving.decode(&ids), case.text, "case {:?}", case.name);
    }

    // Exported runtime tokenizer: the compiled tokenizer.bin carries the
    // same id space — per-id decode bytes and byte lengths agree with
    // the HF adapter over the whole exported vocabulary (added tokens
    // are not part of model.vocab and are not exported).
    let lengths = scenarios::export_hf_bytelevel_tokenizer_with_lengths(
        dir.join("tokenizer.json"),
        dir.join("tokenizer.bin"),
    )
    .expect("runtime tokenizer export");
    let runtime = scenarios::Tokenizer::try_load(dir.join("tokenizer.bin"))
        .expect("exported runtime tokenizer loads");
    assert_eq!(runtime.vocab.len(), 256 + MERGED.len());
    let hf_lengths = evaluation.token_byte_lengths();
    for (id, piece) in runtime.vocab.iter().enumerate() {
        let hf_bytes = evaluation.decode_bytes(&[id as u32]);
        assert_eq!(piece, &hf_bytes, "runtime decode bytes drift at id {id}");
        assert_eq!(
            lengths[id] as usize,
            piece.len(),
            "exported length at id {id}"
        );
        assert_eq!(hf_lengths[id], lengths[id], "byte-length table at id {id}");
    }

    std::fs::remove_dir_all(&dir).ok();
}

/// The legacy llama2.c selection declares NO adapter: its κ-pinned
/// baselines predate adapter records, and manifests written through it
/// stay byte-identical (the #601 provenance field remains unset).
#[test]
fn legacy_selection_declares_no_adapter() {
    let dir = unique_dir("legacy-none");
    // Minimal legacy tokenizer.bin: i32 length-prefixed pieces.
    let mut bytes = Vec::new();
    for piece in [&b" "[..], b"a", b"b"] {
        bytes.extend_from_slice(&(piece.len() as i32).to_le_bytes());
        bytes.extend_from_slice(piece);
    }
    let path = dir.join("tokenizer.bin");
    std::fs::write(&path, bytes).expect("write legacy tokenizer");
    let legacy = scenarios::Tokenizer::try_load(&path).expect("legacy tokenizer loads");
    assert!(TokenizerKind::Legacy(legacy).adapter().is_none());
    std::fs::remove_dir_all(&dir).ok();
}

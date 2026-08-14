//! Behavioral witnesses for the Hugging Face byte-level BPE tokenizer
//! (issue #242): merge-RANK order (not lowest-merged-id greedy), atomic
//! added-token matching, encode/decode round-trips, and the GPT-2
//! byte-level mapping (Ġ for a leading space, multi-byte UTF-8 characters).

use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

/// A tiny synthetic tokenizer.json with a caller-chosen pre-tokenizer.
///
/// Vocabulary (note "ab" has a LOWER id than "bc", while the merge list
/// ranks "b c" FIRST — the discriminating configuration for test
/// `merge_rank_order_beats_lowest_token_id_greedy`):
///
///   a:0 b:1 c:2 ab:3 bc:4 h:5 e:6 l:7 o:8 w:9 r:10 d:11 he:12 hel:13
///   hell:14 hello:15 Ġ:16 Ġw:17 Ġwo:18 Ġwor:19 x:20 Ġworl:21 Ġworld:22
///   Ã:23 ©:24 Ã©:25 f:26 1:27 2:28 12:29   +   added token <|end|>:30
fn tokenizer_json_with(pre_tokenizer: &str) -> String {
    format!(
        r#"{{
  "added_tokens": [
    {{"id": 30, "content": "<|end|>", "special": true}}
  ],
  "pre_tokenizer": {pre_tokenizer},
  "model": {{
    "type": "BPE",
    "vocab": {{
      "a": 0, "b": 1, "c": 2, "ab": 3, "bc": 4,
      "h": 5, "e": 6, "l": 7, "o": 8, "w": 9, "r": 10, "d": 11,
      "he": 12, "hel": 13, "hell": 14, "hello": 15,
      "Ġ": 16, "Ġw": 17, "Ġwo": 18, "Ġwor": 19, "x": 20, "Ġworl": 21, "Ġworld": 22,
      "Ã": 23, "©": 24, "Ã©": 25, "f": 26, "1": 27, "2": 28, "12": 29
    }},
    "merges": [
      "b c",
      "a b",
      "h e",
      "he l",
      "hel l",
      "hell o",
      "Ġ w",
      "Ġw o",
      "Ġwo r",
      "Ġwor l",
      "Ġworl d",
      "Ã ©",
      "1 2"
    ]
  }}
}}"#
    )
}

fn tokenizer_json(add_prefix_space: bool) -> String {
    tokenizer_json_with(&format!(
        r#"{{"type": "ByteLevel", "add_prefix_space": {add_prefix_space}}}"#
    ))
}

fn tokenizer() -> HfBpeTokenizer {
    HfBpeTokenizer::from_tokenizer_json_bytes(tokenizer_json(false).as_bytes())
        .expect("synthetic tokenizer.json parses")
}

/// (a) The pair with the lowest merge RANK merges first. On "abc" the
/// merges table ranks ("b","c") before ("a","b"), so rank-ordered BPE
/// yields ["a","bc"] = [0, 4]. The legacy lowest-merged-token-id greedy
/// rule would instead pick "ab" (id 3 < 4) and yield ["ab","c"] = [3, 2].
#[test]
fn merge_rank_order_beats_lowest_token_id_greedy() {
    let tok = tokenizer();
    assert_eq!(tok.encode("abc"), vec![0, 4]);
}

/// (b) Added tokens match atomically (no pre-tokenization or merging
/// through their content) and decode to their literal text.
#[test]
fn special_token_matched_atomically() {
    let tok = tokenizer();
    assert_eq!(tok.encode("<|end|>"), vec![30]);
    assert_eq!(tok.encode("hello<|end|>hello"), vec![15, 30, 15]);
    assert_eq!(tok.decode(&[30]), "<|end|>");
    assert_eq!(tok.decode(&[15, 30, 15]), "hello<|end|>hello");
}

/// (c) Encode/decode round-trips, including repeated encodes through the
/// per-pre-token cache.
#[test]
fn encode_decode_round_trip() {
    let tok = tokenizer();
    for text in ["hello world", "hello", " world", "hello world<|end|>"] {
        let ids = tok.encode(text);
        assert_eq!(tok.decode(&ids), text, "round trip of {text:?}");
        // Second pass hits the cache and must be identical.
        assert_eq!(tok.encode(text), ids, "cached encode of {text:?}");
    }
}

/// (d) GPT-2 byte-level mapping: the leading space of the second word maps
/// to Ġ ("hello world" → ["hello", "Ġworld"]), and a non-ASCII character
/// ("é" = 0xC3 0xA9 → "Ã©") encodes through its byte-level symbols.
#[test]
fn byte_level_mapping_space_and_non_ascii() {
    let tok = tokenizer();
    assert_eq!(tok.encode("hello world"), vec![15, 22]);
    // "é" is a single letter pre-token whose two UTF-8 bytes map to the
    // byte-level chars "Ã" and "©"; merge rank 11 joins them into id 25.
    assert_eq!(tok.encode("é"), vec![25]);
    assert_eq!(tok.decode(&[25]), "é");
    // Double space: `\s+(?!\S)` keeps the last space attached to "world",
    // the first space stands alone as Ġ.
    assert_eq!(tok.encode("hello  world"), vec![15, 16, 22]);
}

#[test]
fn add_prefix_space_prepends_one_space() {
    let with_prefix = HfBpeTokenizer::from_tokenizer_json_bytes(tokenizer_json(true).as_bytes())
        .expect("synthetic tokenizer.json parses");
    // "world" gains a synthetic leading space → "Ġworld" (id 22); an input
    // already starting with a space is unchanged.
    assert_eq!(with_prefix.encode("world"), vec![22]);
    assert_eq!(with_prefix.encode(" world"), vec![22]);
}

/// SmolLM2 declares `Digits { individual_digits: true }` before
/// `ByteLevel`: every digit stands alone, and no merge crosses a digit
/// boundary even when the merges table contains one ("1 2" here).
#[test]
fn digits_pre_tokenizer_isolates_individual_digits() {
    let plain = tokenizer();
    // Without the Digits step, merge "1 2" joins the digits into id 29.
    assert_eq!(plain.encode("12"), vec![29]);
    assert_eq!(plain.encode("hello 12"), vec![15, 16, 29]);

    let digits = HfBpeTokenizer::from_tokenizer_json_bytes(
        tokenizer_json_with(
            r#"{"type": "Sequence", "pretokenizers": [
                {"type": "Digits", "individual_digits": true},
                {"type": "ByteLevel", "add_prefix_space": false, "trim_offsets": true, "use_regex": true}
            ]}"#,
        )
        .as_bytes(),
    )
    .expect("SmolLM2-shaped pre-tokenizer sequence parses");
    assert_eq!(digits.encode("12"), vec![27, 28]);
    // The space split off by Digits stands alone as Ġ.
    assert_eq!(digits.encode("hello 12"), vec![15, 16, 27, 28]);
    assert_eq!(digits.decode(&[15, 16, 27, 28]), "hello 12");
}

#[test]
fn merges_as_array_pairs_parse_identically() {
    // Newer tokenizer.json files store merges as two-element arrays.
    let json = tokenizer_json(false).replace(
        r#""b c",
      "a b","#,
        r#"["b", "c"],
      ["a", "b"],"#,
    );
    let tok = HfBpeTokenizer::from_tokenizer_json_bytes(json.as_bytes())
        .expect("array-format merges parse");
    assert_eq!(tok.encode("abc"), vec![0, 4]);
}

#[test]
fn surface_metadata() {
    let tok = tokenizer();
    // Highest assigned id is the appended special token at 30.
    assert_eq!(tok.vocab_size(), 31);
    let address = tok.address();
    assert!(address.starts_with("blake3:"), "address is {address:?}");
    assert_eq!(address.len(), "blake3:".len() + 64);
    // Byte-level BPE is total: encode_lossy never replaces characters.
    let (ids, replaced) = tok.encode_lossy("hello world");
    assert_eq!(ids, vec![15, 22]);
    assert_eq!(replaced, 0);
}

#[test]
fn malformed_json_is_a_recoverable_error() {
    assert!(HfBpeTokenizer::from_tokenizer_json_bytes(b"not json").is_none());
    assert!(HfBpeTokenizer::from_tokenizer_json_bytes(b"{}").is_none());
    // A non-byte-level pre-tokenizer is rejected (the caller falls back to
    // the legacy tokenizer instead of mis-encoding).
    let json = tokenizer_json(false).replace("\"ByteLevel\"", "\"Whitespace\"");
    assert!(HfBpeTokenizer::from_tokenizer_json_bytes(json.as_bytes()).is_none());
}

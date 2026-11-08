//! Fixture splits under `CARGO_TARGET_TMPDIR`; every count asserted exactly, every refusal pinned.

use reply_census::run;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use uor_r4_core::native_geometric::mmap_corpus::CorpusWriter;

fn base(test: &str) -> PathBuf {
    let p = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("reply-census")
        .join(test);
    if p.exists() {
        fs::remove_dir_all(&p).unwrap();
    }
    fs::create_dir_all(&p).unwrap();
    p
}

fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// Writes the three files; the manifest gets the true sha256 of both data files.
fn write_split(dir: &Path, tokens: &[u16], mask: &[u8], mut manifest: Value) {
    fs::create_dir_all(dir).unwrap();
    let tp = dir.join("tokens.u16");
    let mut w = CorpusWriter::create(&tp, 4096).unwrap();
    w.write_tokens(tokens).unwrap();
    w.finish().unwrap();
    fs::write(dir.join("response_mask.u8"), mask).unwrap();
    manifest["tokens_sha256"] = json!(sha(&fs::read(&tp).unwrap()));
    manifest["mask_sha256"] = json!(sha(mask));
    fs::write(dir.join("manifest.json"), serde_json::to_vec(&manifest).unwrap()).unwrap();
}

fn push(t: &mut Vec<u16>, m: &mut Vec<u8>, ids: &[u16], bit: u8) {
    t.extend_from_slice(ids);
    m.extend(std::iter::repeat(bit).take(ids.len()));
}

/// Split A (eos 1, bos 0): six documents, seven replies.
/// `[20,21,1]` x3 (the third followed by a one-token mask-0 gap and `[30,1,31,32,1]`),
/// `[40..47]` with no EOS, then one mask-0 token and a mask-0 EOS, and `[50..58,1]` x2.
fn split_a() -> (Vec<u16>, Vec<u8>) {
    let (mut t, mut m) = (Vec::new(), Vec::new());
    let r50: Vec<u16> = (50..=58).chain([1]).collect();
    let r40: Vec<u16> = (40..=47).collect();
    push(&mut t, &mut m, &[0, 10, 11], 0);
    push(&mut t, &mut m, &[20, 21, 1], 1);
    push(&mut t, &mut m, &[0, 10, 12], 0);
    push(&mut t, &mut m, &[20, 21, 1], 1);
    push(&mut t, &mut m, &[0, 13], 0);
    push(&mut t, &mut m, &[20, 21, 1], 1);
    push(&mut t, &mut m, &[14], 0);
    push(&mut t, &mut m, &[30, 1, 31, 32, 1], 1);
    push(&mut t, &mut m, &[0, 15], 0);
    push(&mut t, &mut m, &r40, 1);
    push(&mut t, &mut m, &[16, 1], 0);
    push(&mut t, &mut m, &[0, 17], 0);
    push(&mut t, &mut m, &r50, 1);
    push(&mut t, &mut m, &[0, 18], 0);
    push(&mut t, &mut m, &r50, 1);
    (t, m)
}

/// Split B (eos 1, no bos_id): replies `[20,21,1]`, `[60,61,62,1]`, `[50..57,1]`, `[40..47,1]`.
fn split_b() -> (Vec<u16>, Vec<u8>) {
    let (mut t, mut m) = (Vec::new(), Vec::new());
    let r50: Vec<u16> = (50..=57).chain([1]).collect();
    let r40: Vec<u16> = (40..=47).chain([1]).collect();
    push(&mut t, &mut m, &[5], 0);
    push(&mut t, &mut m, &[20, 21, 1], 1);
    push(&mut t, &mut m, &[6], 0);
    push(&mut t, &mut m, &[60, 61, 62, 1], 1);
    push(&mut t, &mut m, &[7], 0);
    push(&mut t, &mut m, &r50, 1);
    push(&mut t, &mut m, &[8], 0);
    push(&mut t, &mut m, &r40, 1);
    (t, m)
}

fn a_manifest() -> Value {
    json!({ "eos_id": 1, "bos_id": 0, "source": "fixture A" })
}

fn make_a(dir: &Path) {
    let (t, m) = split_a();
    write_split(dir, &t, &m, a_manifest());
}

fn args(kv: &[(&str, String)]) -> Vec<String> {
    kv.iter().map(|(k, v)| format!("{k}={v}")).collect()
}

fn ok(kv: &[(&str, String)]) -> Value {
    serde_json::from_str(&run(&args(kv)).unwrap()).unwrap()
}

fn refused(kv: &[(&str, String)], needle: &str) {
    match run(&args(kv)) {
        Ok(_) => panic!("expected a refusal containing {needle:?}"),
        Err(e) => assert!(e.contains(needle), "refusal {e:?} lacks {needle:?}"),
    }
}

fn d(p: &Path) -> String {
    p.display().to_string()
}

#[test]
fn split_a_counts_exactly() {
    let dir = base("a").join("A");
    make_a(&dir);
    let v = ok(&[("split", d(&dir)), ("name", "A".into())]);
    assert_eq!(v["schema"], "uor-r4-reply-census/v1");
    assert_eq!(v["name"], "A");
    assert_eq!(v["eos_id"], 1);
    assert_eq!(v["bos_id"], 0);
    assert_eq!(v["documents"], 6);
    assert_eq!(v["tokens"], 59);
    assert_eq!(v["scored_targets"], 42);
    assert_eq!(v["eos_targets"], 7);
    assert_eq!(v["replies"], 7);
    assert_eq!(v["replies_ending_eos"], 6);
    assert_eq!(v["eos_not_at_reply_end"], 1);
    assert_eq!(v["distinct_replies"], 4);
    assert_eq!(v["reply_length"], json!({ "median_lower": 5, "min": 3, "max": 10 }));
    assert_eq!(v["eos_in_repeated"], json!({ "count": 5, "of": 6, "ppm": 833333 }));
    assert_eq!(v["eos_in_top1"], json!({ "count": 3, "of": 6, "ppm": 500000 }));
    assert_eq!(v["reply_8grams"], json!({ "windows": 7, "distinct": 4 }));
    let r50: Vec<u16> = (50..=58).chain([1]).collect();
    let r40: Vec<u16> = (40..=47).collect();
    assert_eq!(
        v["top"],
        json!([
            { "count": 3, "length": 3, "first_ids": [20, 21, 1] },
            { "count": 2, "length": 10, "first_ids": r50 },
            { "count": 1, "length": 5, "first_ids": [30, 1, 31, 32, 1] },
            { "count": 1, "length": 8, "first_ids": r40 },
        ])
    );
    assert_eq!(v["against"], Value::Null);
    let raw = |f: &str| sha(&fs::read(dir.join(f)).unwrap());
    for f in ["manifest.json", "tokens.u16", "response_mask.u8"] {
        assert_eq!(v["sha256"][f], raw(f), "{f}");
    }
    // top= truncates the sorted list.
    let v2 = ok(&[("split", d(&dir)), ("name", "A".into()), ("top", "1".into())]);
    assert_eq!(v2["top"].as_array().unwrap().len(), 1);
    assert_eq!(v2["top"][0]["count"], 3);
}

#[test]
fn first_ids_stop_at_sixteen() {
    let dir = base("long").join("L");
    let long: Vec<u16> = (100..120).chain([1]).collect();
    let (mut t, mut m) = (Vec::new(), Vec::new());
    push(&mut t, &mut m, &[0, 9], 0);
    push(&mut t, &mut m, &long, 1);
    write_split(&dir, &t, &m, a_manifest());
    let v = ok(&[("split", d(&dir)), ("name", "L".into())]);
    assert_eq!(v["top"][0]["length"], 21);
    assert_eq!(v["top"][0]["first_ids"], json!(&long[..16]));
    assert_eq!(v["reply_8grams"], json!({ "windows": 14, "distinct": 14 }));
}

#[test]
fn against_split_b_without_bos() {
    let root = base("against");
    let (a, b) = (root.join("A"), root.join("B"));
    make_a(&a);
    let (t, m) = split_b();
    write_split(&b, &t, &m, json!({ "eos_id": 1 }));

    let vb = ok(&[("split", d(&b)), ("name", "B".into())]);
    assert_eq!(vb["bos_id"], Value::Null);
    assert_eq!(vb["documents"], Value::Null);
    assert_eq!(vb["replies"], 4);
    assert_eq!(vb["replies_ending_eos"], 4);
    assert_eq!(vb["reply_8grams"], json!({ "windows": 4, "distinct": 4 }));
    // Four replies (lengths 3, 4, 9, 9): an even count, so the lower median (4) differs from the upper (9).
    assert_eq!(vb["reply_length"], json!({ "median_lower": 4, "min": 3, "max": 9 }));

    let v = ok(&[
        ("split", d(&a)),
        ("name", "A".into()),
        ("against", d(&b)),
        ("against_name", "B".into()),
    ]);
    let g = &v["against"];
    assert_eq!(g["name"], "B");
    assert_eq!(g["replies"], 4);
    assert_eq!(g["distinct_replies"], 4);
    assert_eq!(g["distinct_8grams"], 4);
    // Only [20,21,1] is a reply of both; A's [40..47] is not B's [40..47,1].
    assert_eq!(g["replies_in_against"], json!({ "count": 3, "of": 7, "ppm": 428571 }));
    assert_eq!(g["distinct_replies_in_against"], json!({ "count": 1, "of": 4, "ppm": 250000 }));
    // [40..47] and [50..57] are in both; [51..58] and [52..58,1] are A's only.
    assert_eq!(g["distinct_8grams_in_against"], json!({ "count": 2, "of": 4, "ppm": 500000 }));
}

#[test]
fn mask_length_mismatch_refused() {
    let dir = base("len").join("A");
    let (t, mut m) = split_a();
    m.pop();
    write_split(&dir, &t, &m, a_manifest());
    refused(&[("split", d(&dir)), ("name", "A".into())], "58 bytes, but");
}

#[test]
fn manifest_without_eos_refused() {
    let dir = base("noeos").join("A");
    let (t, m) = split_a();
    write_split(&dir, &t, &m, json!({ "bos_id": 0 }));
    refused(&[("split", d(&dir)), ("name", "A".into())], "no eos_id");
}

#[test]
fn mask_byte_not_0_or_1_refused() {
    let dir = base("byte").join("A");
    let (t, mut m) = split_a();
    m[4] = 2;
    write_split(&dir, &t, &m, a_manifest());
    refused(
        &[("split", d(&dir)), ("name", "A".into())],
        "byte 2 at position 4 is not 0 or 1",
    );
}

#[test]
fn declared_sha_mismatch_refused() {
    let dir = base("sha").join("A");
    make_a(&dir);
    let p = dir.join("manifest.json");
    let mut v: Value = serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
    v["mask_sha256"] = json!("00".repeat(32));
    fs::write(&p, serde_json::to_vec(&v).unwrap()).unwrap();
    refused(&[("split", d(&dir)), ("name", "A".into())], "mask_sha256");
}

#[test]
fn argument_refusals() {
    let root = base("args");
    let (a, b) = (root.join("A"), root.join("B"));
    make_a(&a);
    let (t, m) = split_b();
    write_split(&b, &t, &m, json!({ "eos_id": 2 }));
    let s = || ("split", d(&a));
    let n = || ("name", "A".to_string());
    refused(&[s(), n(), ("bogus", "1".into())], "unknown argument bogus=");
    refused(&[s()], "name= is required");
    refused(&[s(), n(), s()], "split= given more than once");
    refused(&[s(), n(), ("against", d(&b))], "go together");
    refused(&[s(), n(), ("top", "-1".into())], "top=-1");
    refused(
        &[s(), n(), ("against", d(&b)), ("against_name", "B".into())],
        "eos_id 2 differs",
    );
}

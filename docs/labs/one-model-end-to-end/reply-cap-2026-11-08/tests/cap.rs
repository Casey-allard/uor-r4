//! Fixture splits under `CARGO_TARGET_TMPDIR`; every output byte and count asserted exactly,
//! every refusal pinned.

use reply_cap::run;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use uor_r4_core::native_geometric::mmap_corpus::CorpusWriter;

fn base(test: &str) -> PathBuf {
    let p = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("reply-cap")
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

/// Ten documents (eos 1, bos 0), 47 tokens, ten replies:
/// A = `[20,21,1]` x3, B = `[30,1]` x2, C = `[50,1]` x1 end on EOS;
/// N = `[40,41,42]` x2 has no EOS; M = `[60,61]` x2 is followed by a mask-0 EOS, so it does not
/// end on EOS either. File order: A B A N B A N M M C.
fn fixture() -> (Vec<u16>, Vec<u8>) {
    let (mut t, mut m) = (Vec::new(), Vec::new());
    let (a, b, c, n, mm): (&[u16], &[u16], &[u16], &[u16], &[u16]) =
        (&[20, 21, 1], &[30, 1], &[50, 1], &[40, 41, 42], &[60, 61]);
    for (doc, reply) in [(10u16, a), (11, b), (12, a), (13, n), (14, b), (15, a), (16, n)] {
        push(&mut t, &mut m, &[0, doc], 0);
        push(&mut t, &mut m, reply, 1);
    }
    for doc in [17u16, 18] {
        push(&mut t, &mut m, &[0, doc], 0);
        push(&mut t, &mut m, mm, 1);
        push(&mut t, &mut m, &[1], 0);
    }
    push(&mut t, &mut m, &[0, 19], 0);
    push(&mut t, &mut m, c, 1);
    (t, m)
}

#[rustfmt::skip]
const MASK_IN: [u8; 47] = [
    0,0,1,1,1,  0,0,1,1,  0,0,1,1,1,  0,0,1,1,1,  0,0,1,1,  0,0,1,1,1,  0,0,1,1,1,
    0,0,1,1,0,  0,0,1,1,0,  0,0,1,1,
];
/// k=1: the second and third A and the second B are masked whole.
#[rustfmt::skip]
const MASK_K1: [u8; 47] = [
    0,0,1,1,1,  0,0,1,1,  0,0,0,0,0,  0,0,1,1,1,  0,0,0,0,  0,0,0,0,0,  0,0,1,1,1,
    0,0,1,1,0,  0,0,1,1,0,  0,0,1,1,
];
/// k=2: only the third A is masked.
#[rustfmt::skip]
const MASK_K2: [u8; 47] = [
    0,0,1,1,1,  0,0,1,1,  0,0,1,1,1,  0,0,1,1,1,  0,0,1,1,  0,0,0,0,0,  0,0,1,1,1,
    0,0,1,1,0,  0,0,1,1,0,  0,0,1,1,
];

fn manifest() -> Value {
    json!({
        "eos_id": 1, "bos_id": 0, "source": "fixture", "revision": "r1",
        "licence": "test-only", "attribution": "claude-321 fixture",
    })
}

fn make(dir: &Path) {
    let (t, m) = fixture();
    assert_eq!(m, MASK_IN);
    write_split(dir, &t, &m, manifest());
}

fn args(kv: &[(&str, String)]) -> Vec<String> {
    kv.iter().map(|(k, v)| format!("{k}={v}")).collect()
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

/// Runs the cap and checks what holds for every k: tokens byte-identical, manifest shas equal
/// the files, every input field kept, the printed line equal to the manifest's `reply_cap`.
fn cap(test: &str, k: &str) -> (Vec<u8>, Value, Value) {
    let root = base(test);
    let (inp, out) = (root.join("in"), root.join("out"));
    make(&inp);
    let line = run(&args(&[("split", d(&inp)), ("out", d(&out)), ("k", k.into())])).unwrap();
    assert!(!line.contains('\n'));
    let printed: Value = serde_json::from_str(&line).unwrap();
    let tin = fs::read(inp.join("tokens.u16")).unwrap();
    let tout = fs::read(out.join("tokens.u16")).unwrap();
    assert_eq!(tin, tout, "tokens.u16 must be byte-identical");
    let mask = fs::read(out.join("response_mask.u8")).unwrap();
    let mf: Value = serde_json::from_slice(&fs::read(out.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(mf["tokens_sha256"], sha(&tout));
    assert_eq!(mf["tokens_sha256"], sha(&tin));
    assert_eq!(mf["mask_sha256"], sha(&mask));
    assert_eq!(mf["reply_cap"], printed);
    assert_eq!(printed["input_tokens_sha256"], sha(&tin));
    assert_eq!(printed["input_mask_sha256"], sha(&MASK_IN));
    assert_eq!(mf["source"], format!("fixture (reply-cap k={k})"));
    assert_eq!(mf["revision"], format!("r1 (reply-cap k={k})"));
    assert_eq!(mf["licence"], "test-only");
    assert_eq!(mf["attribution"], "claude-321 fixture");
    assert_eq!(mf["bos_id"], 0);
    assert_eq!(mf["eos_id"], 1);
    assert_eq!(mf.as_object().unwrap().len(), 9);
    (mask, printed, mf)
}

fn counts(v: &Value) -> Value {
    let mut c = v.clone();
    let o = c.as_object_mut().unwrap();
    o.remove("input_tokens_sha256");
    o.remove("input_mask_sha256");
    c
}

#[test]
fn k1_keeps_first_occurrence_only() {
    let (mask, v, _) = cap("k1", "1");
    assert_eq!(mask, MASK_K1);
    assert_eq!(
        counts(&v),
        json!({
            "k": 1, "replies": 10, "eos_replies": 6, "eos_replies_kept": 3,
            "eos_replies_masked": 3, "distinct_eos_replies": 3,
            "scored_before": 25, "scored_after": 17,
        })
    );
}

#[test]
fn k2_masks_only_the_third_repeat() {
    let (mask, v, _) = cap("k2", "2");
    assert_eq!(mask, MASK_K2);
    assert_eq!(
        counts(&v),
        json!({
            "k": 2, "replies": 10, "eos_replies": 6, "eos_replies_kept": 5,
            "eos_replies_masked": 1, "distinct_eos_replies": 3,
            "scored_before": 25, "scored_after": 22,
        })
    );
}

#[test]
fn k3_changes_nothing() {
    let (mask, v, mf) = cap("k3", "3");
    assert_eq!(mask, MASK_IN);
    assert_eq!(v["eos_replies_masked"], 0);
    assert_eq!(v["scored_after"], 25);
    assert_eq!(mf["mask_sha256"], sha(&MASK_IN));
}

/// N (no EOS, x2) and M (mask-0 EOS after it, x2) repeat but are never capped, even at k=1;
/// the mask-0 EOS bytes stay 0 and every mask-0 byte stays 0.
#[test]
fn non_eos_runs_and_mask0_eos_untouched() {
    let (mask, _, _) = cap("noneos", "1");
    for (s, e) in [(16, 19), (30, 33), (35, 37), (40, 42)] {
        assert_eq!(&mask[s..e], &MASK_IN[s..e], "run {s}..{e}");
        assert!(mask[s..e].iter().all(|&b| b == 1));
    }
    assert_eq!((mask[37], mask[42]), (0, 0));
    let (t, _) = fixture();
    assert_eq!((t[37], t[42]), (1, 1));
    for i in 0..47 {
        if MASK_IN[i] == 0 {
            assert_eq!(mask[i], 0, "position {i}");
        }
    }
}

#[test]
fn out_exists_refused() {
    let root = base("exists");
    let (inp, out) = (root.join("in"), root.join("out"));
    make(&inp);
    fs::create_dir_all(&out).unwrap();
    refused(&[("split", d(&inp)), ("out", d(&out)), ("k", "1".into())], "out already exists");
    assert_eq!(fs::read_dir(&out).unwrap().count(), 0);
}

#[test]
fn k_refusals() {
    let root = base("k");
    let (inp, out) = (root.join("in"), root.join("out"));
    make(&inp);
    for bad in ["0", "abc", "-1", "1.5", ""] {
        refused(
            &[("split", d(&inp)), ("out", d(&out)), ("k", bad.into())],
            &format!("k={bad}: expected a positive integer"),
        );
    }
    assert!(!out.exists());
}

#[test]
fn mask_length_mismatch_refused() {
    let root = base("len");
    let (t, mut m) = fixture();
    m.pop();
    write_split(&root.join("in"), &t, &m, manifest());
    refused(
        &[("split", d(&root.join("in"))), ("out", d(&root.join("out"))), ("k", "1".into())],
        "46 bytes, but",
    );
    assert!(!root.join("out").exists());
}

#[test]
fn manifest_without_eos_refused() {
    let root = base("noeos");
    let (t, m) = fixture();
    write_split(&root.join("in"), &t, &m, json!({ "bos_id": 0 }));
    refused(
        &[("split", d(&root.join("in"))), ("out", d(&root.join("out"))), ("k", "1".into())],
        "no eos_id",
    );
    assert!(!root.join("out").exists());
}

#[test]
fn mask_byte_not_0_or_1_refused() {
    let root = base("byte");
    let (t, mut m) = fixture();
    m[4] = 2;
    write_split(&root.join("in"), &t, &m, manifest());
    refused(
        &[("split", d(&root.join("in"))), ("out", d(&root.join("out"))), ("k", "1".into())],
        "byte 2 at position 4 is not 0 or 1",
    );
}

#[test]
fn declared_sha_mismatch_refused() {
    let root = base("sha");
    let inp = root.join("in");
    make(&inp);
    let p = inp.join("manifest.json");
    let mut v: Value = serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
    v["tokens_sha256"] = json!("00".repeat(32));
    fs::write(&p, serde_json::to_vec(&v).unwrap()).unwrap();
    refused(
        &[("split", d(&inp)), ("out", d(&root.join("out"))), ("k", "1".into())],
        "tokens_sha256",
    );
}

#[test]
fn already_capped_refused() {
    let root = base("twice");
    let (inp, out) = (root.join("in"), root.join("out"));
    make(&inp);
    run(&args(&[("split", d(&inp)), ("out", d(&out)), ("k", "1".into())])).unwrap();
    refused(
        &[("split", d(&out)), ("out", d(&root.join("out2"))), ("k", "1".into())],
        "already has reply_cap",
    );
}

#[test]
fn argument_refusals() {
    let root = base("args");
    let inp = root.join("in");
    make(&inp);
    let s = || ("split", d(&inp));
    let o = || ("out", d(&root.join("out")));
    let k = || ("k", "1".to_string());
    refused(&[s(), o(), k(), ("bogus", "1".into())], "unknown argument bogus=");
    refused(&[s(), o()], "k= is required");
    refused(&[s(), k()], "out= is required");
    refused(&[o(), k()], "split= is required");
    refused(&[s(), o(), k(), k()], "k= given more than once");
    match run(&["split".to_string()]) {
        Err(e) => assert!(e.contains("expected key=value"), "{e}"),
        Ok(_) => panic!("expected a refusal"),
    }
    assert!(!root.join("out").exists());
}

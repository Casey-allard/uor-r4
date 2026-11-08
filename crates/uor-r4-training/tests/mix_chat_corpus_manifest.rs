use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use uor_r4_core::native_geometric::mmap_corpus::{CorpusWriter, MmapCorpusReader};
use uor_r4_tokenizer::dialogue::SCHEMA_V2;

const TOKENIZER: &str = r#"{
  "version": "1.0",
  "truncation": null,
  "padding": null,
  "added_tokens": [
    {"id": 0, "content": "<|bos|>", "single_word": false, "lstrip": false, "rstrip": false, "normalized": false, "special": true},
    {"id": 1, "content": "<|eos|>", "single_word": false, "lstrip": false, "rstrip": false, "normalized": false, "special": true},
    {"id": 2, "content": "<|unk|>", "single_word": false, "lstrip": false, "rstrip": false, "normalized": false, "special": true}
  ],
  "normalizer": null,
  "pre_tokenizer": {
    "type": "ByteLevel",
    "add_prefix_space": false,
    "trim_offsets": true,
    "use_regex": true
  },
  "post_processor": null,
  "decoder": {
    "type": "ByteLevel",
    "add_prefix_space": true,
    "trim_offsets": true,
    "use_regex": true
  },
  "model": {
    "type": "BPE",
    "dropout": null,
    "unk_token": "<|unk|>",
    "continuing_subword_prefix": null,
    "end_of_word_suffix": null,
    "fuse_unk": false,
    "byte_fallback": false,
    "ignore_merges": false,
    "vocab": {"<|bos|>": 0, "<|eos|>": 1, "<|unk|>": 2, "a": 3, "b": 4},
    "merges": []
  }
}"#;

fn root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "uor-mix-chat-corpus-{}-{nonce}",
        std::process::id()
    ))
}

fn sha(path: &Path) -> String {
    uor_r4_training::sha256_file(path).unwrap()
}

fn source(root: &Path, name: &str, token: u16, max_tokens: u64) -> PathBuf {
    let dir = root.join(name);
    fs::create_dir_all(&dir).unwrap();
    let tokens_path = dir.join("tokens.u16");
    let mut writer = CorpusWriter::create(&tokens_path, 5).unwrap();
    writer.write_tokens(&[0, token, 1]).unwrap();
    writer.finish().unwrap();
    let mask = [0u8, 1, 1];
    let mask_path = dir.join("response_mask.u8");
    fs::write(&mask_path, mask).unwrap();
    let manifest = json!({
        "schema": "uor-r4-chat-corpus/v1",
        "mask_schema": "uor-r4-response-mask/u8/v1",
        "split": "train",
        "template_rule": "test",
        "mask_rule": "test",
        "dialogue_protocol": SCHEMA_V2,
        "max_tokens": max_tokens,
        "tokenizer": {
            "path": root.join("tokenizer.json"),
            "sha256": sha(&root.join("tokenizer.json")),
            "vocab_size": 5,
            "bos_id": 0,
            "eos_id": 1,
            "unk_id": 2
        },
        "drops": {
            "rows_dropped_no_messages": 0,
            "rows_dropped_empty": 0,
            "rows_dropped_no_response": 0,
            "rows_dropped_oversized": 0,
            "special_token_occurrences": 0
        },
        "files": [],
        "rows_used": 1,
        "rows_total": 1,
        "tokens": 3,
        "tokens_bytes": fs::metadata(&tokens_path).unwrap().len(),
        "tokens_sha256": sha(&tokens_path),
        "mask_bytes": mask.len(),
        "mask_sha256": sha(&mask_path),
        "response_tokens": 2
    });
    fs::write(
        dir.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    dir
}

fn run_mix(root: &Path, inputs: &[PathBuf], out: &Path) -> std::process::Output {
    let paths = inputs
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(",");
    Command::new(env!("CARGO_BIN_EXE_mix-chat-corpus"))
        .arg(format!("out={}", out.display()))
        .arg(format!(
            "tokenizer={}",
            root.join("tokenizer.json").display()
        ))
        .arg(format!("inputs={paths}"))
        .arg("labels=first,second")
        .output()
        .unwrap()
}

#[test]
fn mixed_chat_manifest_is_accepted_by_the_cpu_trainer() {
    let root = root();
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("tokenizer.json"), TOKENIZER).unwrap();
    let first = source(&root, "first", 3, 8);
    let second = source(&root, "second", 4, 13);
    let out = root.join("mixed");
    let output = run_mix(&root, &[first, second], &out);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let tokens = fs::read(out.join("tokens.u16")).unwrap();
    let mask = fs::read(out.join("response_mask.u8")).unwrap();
    let manifest = fs::read(out.join("manifest.json")).unwrap();
    let reader = MmapCorpusReader::open(out.join("tokens.u16")).unwrap();
    assert_eq!(reader.as_slice(), [0, 3, 1, 0, 4, 1]);
    assert_eq!(mask, [0, 1, 1, 0, 1, 1]);
    let parsed: Value = serde_json::from_slice(&manifest).unwrap();
    assert_eq!(parsed["max_tokens"], 13);
    assert_eq!(parsed["tokenizer"]["vocab_size"], 5);
    assert_eq!(parsed["tokenizer"]["bos_id"], 0);
    assert_eq!(parsed["dialogue_protocol"], SCHEMA_V2);

    let identity = uor_r4_cpu_train::validate_corpus_manifest(&manifest, &tokens, &mask).unwrap();
    assert_eq!(identity.vocab, 5);
    assert_eq!(identity.bos, 0);
    assert_eq!(identity.max_tokens, 13);
    assert_eq!(identity.response_targets, 4);

    fs::remove_dir_all(root).unwrap();
}

fn assert_source_is_refused(
    root: &Path,
    name: &str,
    mutate: impl FnOnce(&mut Value),
    expected: &str,
) {
    let first = source(root, &format!("first-{name}"), 3, 8);
    let second = source(root, name, 4, 13);
    let manifest_path = second.join("manifest.json");
    let mut manifest: Value = serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    mutate(&mut manifest);
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    let out = root.join(format!("mixed-{name}"));
    let output = run_mix(root, &[first, second], &out);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(expected),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!out.exists());
}

#[test]
fn incompatible_source_tokenizer_is_refused_before_output_is_created() {
    let root = root();
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("tokenizer.json"), TOKENIZER).unwrap();

    assert_source_is_refused(
        &root,
        "bad-hash",
        |manifest| manifest["tokenizer"]["sha256"] = json!("11".repeat(32)),
        "tokenizer SHA-256 differs",
    );
    assert_source_is_refused(
        &root,
        "bad-vocabulary",
        |manifest| manifest["tokenizer"]["vocab_size"] = json!(4),
        "tokenizer vocabulary differs",
    );
    assert_source_is_refused(
        &root,
        "bad-cid",
        |manifest| manifest["tokenizer"]["tokenizer_cid"] = json!("00".repeat(32)),
        "tokenizer CID differs",
    );

    fs::remove_dir_all(root).unwrap();
}

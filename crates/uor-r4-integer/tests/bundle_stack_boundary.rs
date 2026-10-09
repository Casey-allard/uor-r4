//! The boundary between the two serving containers: a sealed serving bundle
//! holds a retained recurrent `IntegerModel`, and a `UORLUT01` geometric-stack
//! artifact holds a different model. This test builds a bundle directory that
//! satisfies every container check — a sealed manifest, `bundle.json` with the
//! schema/context/admission contract and the three file hashes, a parseable
//! tokenizer, a model manifest, and a table manifest — and whose model manifest
//! declares the exported chat stack's own shape. The loader then refuses it at
//! the model import: the stack's width 288 is neither the retained profile's
//! 128/256 nor the dialogue profile's 576, and the stack's parameter inventory
//! (`embed`, `l{layer}.rec_in`, `mlp.*`, `head`, sealed tables) shares no name
//! or shape with `JointConfig::shapes()`. No repackaging of stack bytes as a
//! bundle can succeed; the chat stack is served by the stack engine
//! ([`uor_r4_integer::stack`]) on its own container instead.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::json;
use sha2::{Digest, Sha256};
use uor_r4_integer::{report_output, Bundle};

/// The chat stack's own shape, read from the exported artifact header
/// (`uor-r4.lut-stack/1`): vocab 4096, width 288, heads 6, mlp 768,
/// pattern `rrarra`, Lorentz read with rotations, context 256.
fn stack_shape_model() -> serde_json::Value {
    json!({
        "vocab_size": 4096,
        "width": 288,
        "read_width": 64,
        "context": 256,
        "transport": "quaternion",
        "seed": 1,
    })
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// A parseable byte-level tokenizer with the three dialogue specials.
fn tokenizer_json() -> Vec<u8> {
    let mut vocab = serde_json::Map::new();
    for (id, surface) in ["<|bos|>", "<|eos|>", "<|unk|>"].iter().enumerate() {
        vocab.insert((*surface).to_owned(), json!(id));
    }
    vocab.insert("a".to_owned(), json!(3));
    vocab.insert("b".to_owned(), json!(4));
    let added: Vec<serde_json::Value> = ["<|bos|>", "<|eos|>", "<|unk|>"]
        .iter()
        .enumerate()
        .map(|(id, surface)| json!({"id": id, "content": surface}))
        .collect();
    serde_json::to_vec(&json!({
        "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
        "added_tokens": added,
        "model": {"type": "BPE", "vocab": vocab, "merges": []},
    }))
    .expect("tokenizer json")
}

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "uor-integer-boundary-{}-{}-{name}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("scratch directory");
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn a_sealed_bundle_cannot_carry_the_stack_shape() -> Result<(), Box<dyn std::error::Error>> {
    let scratch = Scratch::new("stack-shape");
    let root = scratch.0.as_path();
    let tokenizer = tokenizer_json();
    fs::write(root.join("tokenizer.json"), &tokenizer)?;
    fs::create_dir(root.join("model"))?;
    fs::create_dir(root.join("tables"))?;
    let model_manifest = serde_json::to_vec(&json!({
        "schema": "uor-r4.joint-recurrent-packed-emulator/1",
        "admission": "full",
        "model": stack_shape_model(),
    }))?;
    fs::write(root.join("model/hard-model.json"), &model_manifest)?;
    let tables_manifest = serde_json::to_vec(&json!({"schema": "uor-r4.table-artifact/1"}))?;
    fs::write(root.join("tables/tables.json"), &tables_manifest)?;
    let metadata = serde_json::to_vec_pretty(&json!({
        "schema": "uor-r4.integer-serving-bundle/1",
        "context": 256,
        "admission": "full",
        "tokenizer_sha256": sha256(&tokenizer),
        "model_manifest_sha256": sha256(&model_manifest),
        "tables_manifest_sha256": sha256(&tables_manifest),
    }))?;
    fs::write(root.join("bundle.json"), &metadata)?;
    report_output::seal(root)?;
    report_output::verify(root)?;
    let message = match Bundle::load(root) {
        Ok(_) => panic!("the model import must refuse the stack shape"),
        Err(error) => error.to_string(),
    };
    println!("bundle loader refusal: {message}");
    assert!(
        message.contains("width128/256"),
        "the refusal must be the retained profile's shape rule, got: {message}"
    );
    Ok(())
}

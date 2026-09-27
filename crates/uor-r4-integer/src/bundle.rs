//! Portable, sealed serving bundle binding exact model, tables and tokenizer.
use crate::{format, invalid, report_output, sha256_file, IntegerModel, Result};
use serde_json::{json, Value};
use std::{fs, path::Path};
use uor_r4_tokenizer::ByteBpeTokenizer;

const SCHEMA: &str = "uor-r4.integer-serving-bundle/1";
const MODEL_FILES: [&str; 3] = [
    "hard-model.json",
    "hard-parameters.json",
    "hard-parameters.bin",
];
const TABLE_FILES: [&str; 4] = ["tables.json", "tables.bin", "attempt.json", "manifest.json"];

pub struct Bundle {
    model: IntegerModel,
    tokenizer: ByteBpeTokenizer,
    identity: String,
}
impl Bundle {
    /// Load only files under the sealed bundle; no source/provider lookup.
    pub fn load(root: &Path) -> Result<Self> {
        format::verify_sealed(root)?;
        let metadata: Value = serde_json::from_slice(&fs::read(root.join("bundle.json"))?)?;
        let tokenizer_path = root.join("tokenizer.json");
        if metadata["schema"] != SCHEMA
            || metadata["context"] != 256
            || metadata["admission"] != "full"
            || metadata["tokenizer_sha256"] != sha256_file(&tokenizer_path)?
            || metadata["model_manifest_sha256"]
                != sha256_file(&root.join("model/hard-model.json"))?
            || metadata["tables_manifest_sha256"] != sha256_file(&root.join("tables/tables.json"))?
        {
            return Err(invalid("serving bundle identity/contract mismatch"));
        }
        let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(tokenizer_path)?)
            .ok_or_else(|| invalid("serving bundle tokenizer invalid"))?;
        let model = IntegerModel::load_with_tables(&root.join("model"), &root.join("tables"))?;
        if tokenizer.vocab_size() != model.config().vocab_size
            || tokenizer.encode("<|bos|>") != [0]
            || tokenizer.encode("<|eos|>") != [1]
            || metadata["tokenizer_cid"] != tokenizer.address()
        {
            return Err(invalid("serving tokenizer identity/vocabulary differs"));
        }
        Ok(Self {
            model,
            tokenizer,
            identity: sha256_file(&root.join("bundle.json"))?,
        })
    }
    pub fn model(&self) -> &IntegerModel {
        &self.model
    }
    pub fn tokenizer(&self) -> &ByteBpeTokenizer {
        &self.tokenizer
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }

    /// Construct a synthetic bundle in-memory for testing and offline verification.
    pub fn synthetic_for_test() -> Self {
        let model = IntegerModel::synthetic_for_test();
        let mut vocab_map = serde_json::Map::new();
        vocab_map.insert("<|bos|>".to_string(), json!(0));
        vocab_map.insert("<|eos|>".to_string(), json!(1));
        vocab_map.insert("<|unk|>".to_string(), json!(2));
        vocab_map.insert("<|system|>".to_string(), json!(3));
        vocab_map.insert("<|user|>".to_string(), json!(4));
        vocab_map.insert("<|assistant|>".to_string(), json!(5));
        vocab_map.insert("<|turn_end|>".to_string(), json!(6));
        for i in 7..4096 {
            vocab_map.insert(format!("t{i}"), json!(i));
        }
        let tok_json = json!({
            "pre_tokenizer": {
                "type": "ByteLevel",
                "add_prefix_space": false
            },
            "model": {
                "type": "BPE",
                "vocab": vocab_map,
                "merges": []
            },
            "added_tokens": [
                {"id": 0, "content": "<|bos|>"},
                {"id": 1, "content": "<|eos|>"},
                {"id": 2, "content": "<|unk|>"},
                {"id": 3, "content": "<|system|>"},
                {"id": 4, "content": "<|user|>"},
                {"id": 5, "content": "<|assistant|>"},
                {"id": 6, "content": "<|turn_end|>"}
            ]
        });
        let tok_bytes = serde_json::to_vec(&tok_json).expect("valid synthetic tokenizer json");
        let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tok_bytes)
            .expect("valid synthetic tokenizer");
        Self {
            model,
            tokenizer,
            identity: "synthetic-test-bundle-sha256".to_string(),
        }
    }

    /// Construct a bundle from explicit model, tokenizer, and identity components.
    pub fn from_parts(model: IntegerModel, tokenizer: ByteBpeTokenizer, identity: String) -> Self {
        Self {
            model,
            tokenizer,
            identity,
        }
    }

    /// Construct a synthetic bundle with complete 256-byte vocabulary + role tokens.
    /// Guaranteed to encode and decode arbitrary UTF-8 text deterministically.
    pub fn create_test_bundle_with_byte_vocab() -> Self {
        let model = IntegerModel::synthetic_for_test();
        let mut vocab_map = serde_json::Map::new();

        // Special role tokens (IDs 0..6)
        vocab_map.insert("<|bos|>".to_string(), json!(0));
        vocab_map.insert("<|eos|>".to_string(), json!(1));
        vocab_map.insert("<|unk|>".to_string(), json!(2));
        vocab_map.insert("<|system|>".to_string(), json!(3));
        vocab_map.insert("<|user|>".to_string(), json!(4));
        vocab_map.insert("<|assistant|>".to_string(), json!(5));
        vocab_map.insert("<|turn_end|>".to_string(), json!(6));

        // Standard GPT-2 byte mapping: printable ASCII map to themselves, rest to U+0100..
        let mut assigned = [false; 256];
        for b in (b'!'..=b'~').chain(0xA1..=0xAC).chain(0xAE..=0xFF) {
            let ch = char::from_u32(u32::from(b)).unwrap();
            vocab_map.insert(ch.to_string(), json!(7 + b as usize));
            assigned[b as usize] = true;
        }
        let mut extra = 0u32;
        for (b, &is_assigned) in assigned.iter().enumerate() {
            if !is_assigned {
                let ch = char::from_u32(256 + extra).unwrap();
                vocab_map.insert(ch.to_string(), json!(7 + b));
                extra += 1;
            }
        }

        // Pad remaining vocabulary up to 4096 tokens
        for i in 263..4096 {
            vocab_map.insert(format!("t{i}"), json!(i));
        }

        let tok_json = json!({
            "pre_tokenizer": {
                "type": "ByteLevel",
                "add_prefix_space": false
            },
            "model": {
                "type": "BPE",
                "vocab": vocab_map,
                "merges": []
            },
            "added_tokens": [
                {"id": 0, "content": "<|bos|>"},
                {"id": 1, "content": "<|eos|>"},
                {"id": 2, "content": "<|unk|>"},
                {"id": 3, "content": "<|system|>"},
                {"id": 4, "content": "<|user|>"},
                {"id": 5, "content": "<|assistant|>"},
                {"id": 6, "content": "<|turn_end|>"}
            ]
        });

        let tok_bytes = serde_json::to_vec(&tok_json).expect("valid synthetic tokenizer json");
        let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tok_bytes)
            .expect("valid byte-level tokenizer");
        Self {
            model,
            tokenizer,
            identity: "synthetic-byte-vocab-bundle-sha256".to_string(),
        }
    }
}

/// Construct a synthetic bundle with complete 256-byte vocabulary + role tokens.
pub fn create_test_bundle_with_byte_vocab() -> Bundle {
    Bundle::create_test_bundle_with_byte_vocab()
}

/// Materialize unchanged accepted codes/tables with their training-bound tokenizer.
/// The bundle retains legacy diagnostic metadata; no training or table compilation.
pub fn pack(packed: &Path, tables: &Path, tokenizer: &Path, output: &Path) -> Result<()> {
    report_output::claim(output)?;
    format::verify_sealed(packed)?;
    format::verify_sealed(tables)?;
    let model = IntegerModel::load_with_tables(packed, tables)?;
    let evaluator: Value = serde_json::from_slice(&fs::read(packed.join("evaluator.json"))?)?;
    let tokenizer_sha = sha256_file(tokenizer)?;
    let inputs = evaluator["reference_inputs"]
        .as_array()
        .ok_or_else(|| invalid("parent tokenizer provenance missing"))?;
    let tokenizer_records: Vec<_> = inputs
        .iter()
        .filter(|r| {
            r["path"]
                .as_str()
                .is_some_and(|s| s.ends_with("/tokenizer.json"))
        })
        .collect();
    if tokenizer_records.len() != 1 || tokenizer_records[0]["sha256"] != tokenizer_sha {
        return Err(invalid("tokenizer must match accepted parent provenance"));
    }
    let bytes = fs::read(tokenizer)?;
    let codec = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| invalid("tokenizer parse failed"))?;
    if codec.vocab_size() != model.config().vocab_size
        || codec.encode("<|bos|>") != [0]
        || codec.encode("<|eos|>") != [1]
    {
        return Err(invalid("tokenizer/model vocabulary differs"));
    }
    fs::create_dir(output.join("model"))?;
    fs::create_dir(output.join("tables"))?;
    for name in MODEL_FILES {
        fs::copy(packed.join(name), output.join("model").join(name))?;
    }
    for name in TABLE_FILES {
        fs::copy(tables.join(name), output.join("tables").join(name))?;
    }
    fs::write(output.join("tokenizer.json"), bytes)?;
    let metadata = json!({"schema":SCHEMA,"context":256,"admission":"full","model":model.config(),
        "tokenizer_sha256":tokenizer_sha,"tokenizer_cid":codec.address(),
        "model_manifest_sha256":sha256_file(&packed.join("hard-model.json"))?,
        "tables_manifest_sha256":sha256_file(&tables.join("tables.json"))?,
        "parent_sealed_manifest_sha256":sha256_file(&packed.join("manifest.json"))?,
        "parent_evaluator_sha256":sha256_file(&packed.join("evaluator.json"))?,
        "origin_packed_path":packed,"origin_tables_path":tables,
        "numerical_contract":"PR1396 integer Q48/Q11 computation; full256; signed4 additive maps; dense access and allocation",
        "sampling":"greedy or categorical temperature1 Q48/top-k/xorshift64; new policy, not old floating sampler parity",
        "legacy_metadata":"Old parameter diagnostics/numerical-contract labels retained for identity and loading only; no legacy floating model computation"});
    fs::write(
        output.join("bundle.json"),
        serde_json::to_vec_pretty(&metadata)?,
    )?;
    report_output::seal(output)?;
    Bundle::load(output)?;
    Ok(())
}

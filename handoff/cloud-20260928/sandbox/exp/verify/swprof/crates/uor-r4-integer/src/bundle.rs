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

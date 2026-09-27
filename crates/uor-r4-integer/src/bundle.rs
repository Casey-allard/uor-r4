//! Portable, sealed serving bundle binding exact model, tables and tokenizer.
use crate::{
    config::{packed_numerical_contract_for_profile, ServingProfile},
    format, invalid, report_output, sha256_file, tables, IntegerModel, Result,
};
use serde_json::{json, Value};
use std::{fs, path::Path};
use uor_r4_tokenizer::{dialogue::DialogueProtocol, ByteBpeTokenizer};

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
        let profile = bundle_profile(&metadata)?;
        let model = IntegerModel::load_with_tables_profile(
            &root.join("model"),
            &root.join("tables"),
            profile,
        )?;
        if profile == ServingProfile::Dialogue576 {
            let manifest: Value =
                serde_json::from_slice(&fs::read(root.join("model/hard-model.json"))?)?;
            validate_dialogue_conversion(&manifest, &metadata["tokenizer_sha256"], &tokenizer)?;
            if metadata["model"] != serde_json::to_value(model.config())?
                || metadata["numerical_contract"] != manifest["numerical_contract"]
                || !valid_hash(&metadata["parent_sealed_manifest_sha256"], 64)
            {
                return Err(invalid("dialogue576 bundle model/contract binding differs"));
            }
        }
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

    /// Construct a synthetic Lorentz bundle in-memory with complete byte vocabulary for testing.
    pub fn synthetic_lorentz_for_test() -> Self {
        let mut bundle = Self::create_test_bundle_with_byte_vocab();
        bundle.model = IntegerModel::synthetic_lorentz_for_test();
        bundle.identity = "synthetic-lorentz-test-bundle-sha256".to_string();
        bundle
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
    copy_model_and_tables(packed, tables, output)?;
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

/// The bundle's model and table files; the arcosh table only when present.
fn copy_model_and_tables(packed: &Path, tables: &Path, output: &Path) -> Result<()> {
    fs::create_dir(output.join("model"))?;
    fs::create_dir(output.join("tables"))?;
    for name in MODEL_FILES {
        fs::copy(packed.join(name), output.join("model").join(name))?;
    }
    let arcosh = [tables::ARCOSH_METADATA, tables::ARCOSH_PAYLOAD];
    let optional = if tables.join(tables::ARCOSH_METADATA).try_exists()? {
        &arcosh[..]
    } else {
        &[]
    };
    for name in TABLE_FILES.iter().chain(optional) {
        fs::copy(tables.join(name), output.join("tables").join(name))?;
    }
    Ok(())
}

/// Bundle a development model: one packed by a training tool (for example the
/// `joint-integer-parity` example) rather than an accepted campaign artifact.
/// The layout, sealing and loader are those of [`pack`], but no evaluator
/// provenance exists: the caller supplies the tokenizer, and `bundle.json`
/// records the bundle as a development bundle with no accepted parent.
pub fn pack_development(
    packed: &Path,
    tables: &Path,
    tokenizer: &Path,
    output: &Path,
) -> Result<()> {
    report_output::claim(output)?;
    format::verify_sealed(tables)?;
    let model = IntegerModel::load_with_tables(packed, tables)?;
    let bytes = fs::read(tokenizer)?;
    let codec = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| invalid("tokenizer parse failed"))?;
    if codec.vocab_size() != model.config().vocab_size
        || codec.encode("<|bos|>") != [0]
        || codec.encode("<|eos|>") != [1]
    {
        return Err(invalid("tokenizer/model vocabulary differs"));
    }
    if model.config().context != 256 {
        return Err(invalid("serving bundles keep the context-256 shape"));
    }
    copy_model_and_tables(packed, tables, output)?;
    fs::write(output.join("tokenizer.json"), &bytes)?;
    let geometry = model.config().read_geometry.name();
    let metadata = json!({"schema":SCHEMA,"context":256,"admission":"full","model":model.config(),
        "tokenizer_sha256":sha256_file(tokenizer)?,"tokenizer_cid":codec.address(),
        "model_manifest_sha256":sha256_file(&packed.join("hard-model.json"))?,
        "tables_manifest_sha256":sha256_file(&tables.join("tables.json"))?,
        "provenance":"development: no accepted parent or evaluator; the tokenizer was supplied by the caller",
        "origin_packed_path":packed,"origin_tables_path":tables,
        "numerical_contract":format!("Integer Q48/Q11 computation; {geometry} read; full256; signed4 additive maps; dense access and allocation"),
        "sampling":"greedy or categorical temperature1 Q48/top-k/xorshift64"});
    fs::write(
        output.join("bundle.json"),
        serde_json::to_vec_pretty(&metadata)?,
    )?;
    report_output::seal(output)?;
    Bundle::load(output)?;
    Ok(())
}

/// Package the explicitly converted native width576 model as development
/// evidence. The sealed child carries the verified historical parent and the
/// caller's tokenizer must match that child; no source tree is needed to load it.
pub fn pack_dialogue576(
    packed: &Path,
    tables: &Path,
    tokenizer: &Path,
    output: &Path,
) -> Result<()> {
    report_output::claim(output)?;
    let outcome = pack_dialogue576_claimed(packed, tables, tokenizer, output);
    if let Err(error) = &outcome {
        // A successfully sealed bundle remains immutable even if its final
        // reload fails. Earlier failures preserve a sealed, explicit attempt.
        if !output.join("manifest.json").try_exists()? {
            fs::write(
                output.join("failed-attempt.json"),
                serde_json::to_vec_pretty(&json!({
                    "schema":"uor-r4.native-dialogue576-package-failure/1",
                    "status":"PACKAGING_FAILED","error":error.to_string(),
                    "serving_profile":"dialogue576","provenance_kind":"development"
                }))?,
            )?;
            report_output::seal(output)?;
            report_output::verify(output)?;
        }
    }
    outcome
}

fn pack_dialogue576_claimed(
    packed: &Path,
    tables: &Path,
    tokenizer: &Path,
    output: &Path,
) -> Result<()> {
    format::verify_sealed(packed)?;
    format::verify_sealed(tables)?;
    let profile = ServingProfile::Dialogue576;
    let model = IntegerModel::load_with_tables_profile(packed, tables, profile)?;
    let manifest: Value = serde_json::from_slice(&fs::read(packed.join("hard-model.json"))?)?;
    let bytes = fs::read(tokenizer)?;
    let tokenizer_sha = sha256_file(tokenizer)?;
    let codec = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| invalid("tokenizer parse failed"))?;
    validate_dialogue_conversion(&manifest, &json!(tokenizer_sha), &codec)?;
    if codec.vocab_size() != model.config().vocab_size
        || codec.encode("<|bos|>") != [0]
        || codec.encode("<|eos|>") != [1]
    {
        return Err(invalid("dialogue576 tokenizer/model vocabulary differs"));
    }
    copy_model_and_tables(packed, tables, output)?;
    fs::write(output.join("tokenizer.json"), &bytes)?;
    let metadata = json!({
        "schema":SCHEMA,"context":256,"admission":"full",
        "serving_profile":profile,"provenance_kind":"development",
        "model":model.config(),"tokenizer_sha256":tokenizer_sha,
        "tokenizer_cid":codec.address(),
        "model_manifest_sha256":sha256_file(&packed.join("hard-model.json"))?,
        "tables_manifest_sha256":sha256_file(&tables.join("tables.json"))?,
        "parent_sealed_manifest_sha256":sha256_file(&packed.join("manifest.json"))?,
        "origin_packed_path":packed,"origin_tables_path":tables,
        "numerical_contract":packed_numerical_contract_for_profile(model.config().read_geometry,profile)?,
        "sampling":"greedy or categorical temperature1 Q48/top-k/xorshift64",
        "provenance":"Explicit native dialogue576 conversion; portable parent/source/tokenizer bindings in model/hard-model.json. Development only; numerical retention, generated behavior and efficiency require separate observation. Dense coefficient access and allocation remain."
    });
    fs::write(
        output.join("bundle.json"),
        serde_json::to_vec_pretty(&metadata)?,
    )?;
    report_output::seal(output)?;
    Bundle::load(output)?;
    Ok(())
}

fn bundle_profile(metadata: &Value) -> Result<ServingProfile> {
    let profile: ServingProfile = metadata
        .get("serving_profile")
        .cloned()
        .map(serde_json::from_value)
        .transpose()?
        .unwrap_or_default();
    if profile == ServingProfile::Dialogue576 && metadata["provenance_kind"] != "development" {
        return Err(invalid(
            "dialogue576 requires explicit development provenance",
        ));
    }
    Ok(profile)
}

fn valid_hash(value: &Value, length: usize) -> bool {
    value.as_str().is_some_and(|text| {
        text.len() == length && text.bytes().all(|byte| byte.is_ascii_hexdigit())
    })
}

/// Verify portable lineage consistency, not the truth of historical capability
/// claims. The converter verifies parent bytes before authoring this record.
fn validate_dialogue_conversion(
    manifest: &Value,
    tokenizer_sha: &Value,
    tokenizer: &ByteBpeTokenizer,
) -> Result<()> {
    let provenance = &manifest["conversion_provenance"];
    let parent = &provenance["parent"];
    let quantization = &manifest["quantization"];
    let source_hashes = provenance["source_sha256"].as_object();
    let protocol =
        DialogueProtocol::from_json(&serde_json::to_vec(&parent["protocol"])?, tokenizer)
            .map_err(|error| invalid(error.to_string()))?;
    if manifest["serving_profile"] != "dialogue576"
        || provenance["schema"] != "uor-r4.native-dialogue576-conversion/1"
        || provenance["serving_profile"] != "dialogue576"
        || provenance["admission"] != "full"
        || provenance["tokenizer_sha256"] != *tokenizer_sha
        || parent["tokenizer_sha256"] != *tokenizer_sha
        || !valid_hash(tokenizer_sha, 64)
        || parent["current_model"] != manifest["model"]
        || parent["admission"] != "full"
        || parent["read_geometry"] != "dot"
        || !valid_hash(&parent["fit_seal_sha256"], 64)
        || !valid_hash(&parent["parameter_sha256"], 64)
        || !valid_hash(&parent["prepared_manifest_sha256"], 64)
        || provenance["protocol_identity"]
            != protocol
                .identity()
                .map_err(|error| invalid(error.to_string()))?
        || provenance["protocol_identity"] != parent["protocol_identity"]
        || !valid_hash(&provenance["conversion_source_commit"], 40)
        || !valid_hash(&provenance["conversion_executable_sha256"], 64)
        || source_hashes.is_none_or(|hashes| {
            hashes.is_empty() || hashes.values().any(|value| !valid_hash(value, 64))
        })
        || quantization["preparation"] != "calibrated_for_export"
        || quantization["ramp_steps"] != 1
        || parent["steps_completed"].as_u64().is_none()
        || quantization["start_step"] != parent["steps_completed"]
        || quantization["completed_step"] != parent["steps_completed"]
        || [
            "new_forward_calls",
            "new_generation_calls",
            "new_optimizer_updates",
            "new_model_updates",
        ]
        .iter()
        .any(|field| provenance[*field] != 0)
    {
        return Err(invalid(
            "dialogue576 conversion provenance/tokenizer/clock mismatch",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialogue576_bundle_seals_failed_packaging_without_overwrite() -> Result<()> {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| invalid("test clock before epoch"))?
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "uor-dialogue576-pack-{}-{nonce}",
            std::process::id()
        ));
        let missing = root.join("missing-input");
        assert!(pack_dialogue576(&missing, &missing, &missing, &root).is_err());
        report_output::verify(&root)?;
        let failure: Value = serde_json::from_slice(&fs::read(root.join("failed-attempt.json"))?)?;
        assert_eq!(failure["status"], "PACKAGING_FAILED");
        let original_seal = fs::read(root.join("manifest.json"))?;
        assert!(pack_dialogue576(&missing, &missing, &missing, &root).is_err());
        assert_eq!(fs::read(root.join("manifest.json"))?, original_seal);
        assert!(Bundle::load(&root).is_err());
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn dialogue576_bundle_requires_explicit_development_profile() -> Result<()> {
        assert_eq!(bundle_profile(&json!({}))?, ServingProfile::Retained);
        assert!(bundle_profile(&json!({"serving_profile":"dialogue576"})).is_err());
        assert!(bundle_profile(&json!({"serving_profile":"unknown"})).is_err());
        assert_eq!(
            bundle_profile(
                &json!({"serving_profile":"dialogue576", "provenance_kind":"development"})
            )?,
            ServingProfile::Dialogue576
        );
        Ok(())
    }

    #[test]
    fn dialogue576_bundle_rejects_tokenizer_lineage_and_clock_changes() -> Result<()> {
        // A small metadata-only fixture. Full artifact import is exercised by
        // the converter; this test isolates portable bundle binding failures.
        let hash = "a".repeat(64);
        let tokenizer = Bundle::create_test_bundle_with_byte_vocab().tokenizer;
        let protocol = DialogueProtocol::literal_roles_v1(&tokenizer)
            .map_err(|error| invalid(error.to_string()))?;
        let protocol_identity = protocol
            .identity()
            .map_err(|error| invalid(error.to_string()))?;
        let config = crate::config::JointConfig {
            width: 576,
            ..Default::default()
        };
        let manifest = json!({
            "serving_profile":"dialogue576","model":config,
            "quantization":{"preparation":"calibrated_for_export","start_step":2237,"completed_step":2237,"ramp_steps":1},
            "conversion_provenance":{
                "schema":"uor-r4.native-dialogue576-conversion/1",
                "serving_profile":"dialogue576","admission":"full",
                "tokenizer_sha256":hash,"protocol_identity":protocol_identity,
                "conversion_source_commit":"b".repeat(40),
                "conversion_executable_sha256":hash,"source_sha256":{"example.rs":hash},
                "new_forward_calls":0,"new_generation_calls":0,"new_optimizer_updates":0,"new_model_updates":0,
                "parent":{"current_model":config,"admission":"full","read_geometry":"dot","tokenizer_sha256":hash,
                    "protocol_identity":protocol_identity,"protocol":protocol,"fit_seal_sha256":hash,"parameter_sha256":hash,"prepared_manifest_sha256":hash,"steps_completed":2237}
            }
        });
        validate_dialogue_conversion(&manifest, &json!(hash), &tokenizer)?;
        assert!(
            validate_dialogue_conversion(&manifest, &json!("c".repeat(64)), &tokenizer).is_err()
        );
        for path in [
            "/conversion_provenance/parent/tokenizer_sha256",
            "/conversion_provenance/source_sha256/example.rs",
            "/conversion_provenance/protocol_identity",
        ] {
            let mut changed = manifest.clone();
            *changed
                .pointer_mut(path)
                .ok_or_else(|| invalid("fixture pointer missing"))? = json!("invalid");
            assert!(validate_dialogue_conversion(&changed, &json!(hash), &tokenizer).is_err());
        }
        for path in [
            "/quantization/completed_step",
            "/conversion_provenance/new_optimizer_updates",
            "/conversion_provenance/parent/current_model/width",
        ] {
            let mut changed = manifest.clone();
            *changed
                .pointer_mut(path)
                .ok_or_else(|| invalid("fixture pointer missing"))? = json!(42);
            assert!(validate_dialogue_conversion(&changed, &json!(hash), &tokenizer).is_err());
        }
        Ok(())
    }
}

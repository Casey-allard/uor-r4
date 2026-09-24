//! Shared input/provenance boundary for the offline reference comparison.
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use candle_core::Device;
use serde::Serialize;
use serde_json::{json, Value};

use crate::{invalid, sha256_file, Result, CANDLE_VERSION};

pub struct Evaluator {
    pub document: Value,
    pub sha256: String,
    pub path: PathBuf,
}

pub fn load_evaluator(path: &Path) -> Result<Evaluator> {
    let document: Value = serde_json::from_slice(&fs::read(path)?)?;
    if document["schema"] != "uor-r4.reference-evaluator/2"
        || document["context"] != 256
        || document["stride"] != 256
        || document["vocabulary"] != 4096
        || document["full_blocks"] != 976
        || document["tune_blocks"] != 64
        || document["comparison_blocks"] != 912
        || document["scored_targets"] != 249856
        || document["ngram"]["discount_grid"] != json!([0.5, 0.75, 0.9])
        || document["cache"]["mixture_grid"] != json!([0, 0.05, 0.1, 0.2, 0.4, 0.6])
        || document["cache"]["capacity"] != 256
    {
        return Err(invalid("unsupported or changed evaluator v2 contract"));
    }
    Ok(Evaluator {
        document,
        sha256: sha256_file(path)?,
        path: path.to_path_buf(),
    })
}

/// Rehash the actual local file; the report binds this identity, not its name.
pub fn verify_identity(identity: &Value) -> Result<PathBuf> {
    let path = PathBuf::from(
        identity["path"]
            .as_str()
            .ok_or_else(|| invalid("identity path"))?,
    );
    let bytes = identity["bytes"]
        .as_u64()
        .ok_or_else(|| invalid("identity size"))?;
    let expected = identity["sha256"]
        .as_str()
        .ok_or_else(|| invalid("identity SHA-256"))?;
    if fs::metadata(&path)?.len() != bytes || sha256_file(&path)? != expected {
        return Err(invalid(format!(
            "input identity mismatch: {}",
            path.display()
        )));
    }
    Ok(path)
}

pub fn read_tokens(identity: &Value) -> Result<Vec<u16>> {
    let path = verify_identity(identity)?;
    let bytes = fs::read(&path)?;
    if bytes.len() % 2 != 0 {
        return Err(invalid("token store is not little-endian u16"));
    }
    let tokens: Vec<_> = bytes
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    if tokens.iter().any(|&token| token >= 4096) {
        return Err(invalid("token store has an out-of-vocabulary ID"));
    }
    Ok(tokens)
}

pub fn base_report(protocol: &Evaluator, mode: &str) -> Result<Value> {
    let source = option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNBOUND");
    if source == "UNBOUND" {
        return Err(invalid("build evidence with UOR_BUILD_SOURCE_COMMIT"));
    }
    Ok(json!({
        "schema":"uor-r4.reference-baselines-report/1",
        "mode":mode,
        "source_commit":source,
        "executable_sha256":sha256_file(&std::env::current_exe()?)?,
        "evaluator_path":protocol.path,
        "evaluator_sha256":protocol.sha256,
        "candle_version":CANDLE_VERSION,
        "scope":"Offline D8 reference/count comparison on previously exposed development; no native model promotion or fresh final qualification",
        "neural_optimizer_steps":0
    }))
}

pub fn save_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut file = fs::File::create_new(path)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(())
}

pub fn device(name: &str) -> Result<Device> {
    match name {
        "cpu" => Ok(Device::Cpu),
        "metal" => {
            #[cfg(feature = "metal")]
            {
                Ok(Device::new_metal(0)?)
            }
            #[cfg(not(feature = "metal"))]
            {
                Err(invalid("Metal requested without compiled Metal feature"))
            }
        }
        _ => Err(invalid("device must be cpu or metal; no implicit fallback")),
    }
}

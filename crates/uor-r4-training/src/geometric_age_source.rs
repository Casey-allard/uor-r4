//! Standalone absolute-age shadows enrolled against an immutable saved base.
//!
//! The base lineage and current learned parameter are different identities.
//! This source changes neither the fixed initialization prior nor the integer
//! age kernel. Load regenerates the packed residual and expanded Q24 ages.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::Path;

use candle_core::{Device, Tensor};
use serde::{Deserialize, Serialize};
use uor_r4_integer::geometric_age_q4::AgeResidualQ4Config;

use crate::geometric_read_native::{
    q4_age_residual, residual_metadata, tensor_age, AgeResidualQ4Metadata, ReadBoundFile,
    ReadSourceBinding,
};
use crate::{invalid, sha256_bytes, Result};

pub const SCHEMA: &str = "uor-r4.geometric-age-source/1";
pub const RAW_FILE: &str = "age-raw-f32le.bin";
pub const FILES: [&str; 5] = [
    "metadata.json",
    RAW_FILE,
    "age-residual-coefficients-q4.bin",
    "age-i64le.bin",
    "tokenizer-identity.bin",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgeSourceLineage {
    pub base_files: BTreeMap<String, ReadBoundFile>,
    pub layer: usize,
    pub heads: usize,
    pub context: usize,
    pub value_width: usize,
    pub age_parameter: String,
    pub base_age_f32: ReadBoundFile,
    pub potential_metadata: ReadBoundFile,
    pub tokenizer: ReadBoundFile,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgeSourceMetadata {
    pub schema: String,
    pub lineage: AgeSourceLineage,
    pub current_age_f32: ReadBoundFile,
    pub expanded_age_q24: ReadBoundFile,
    pub residual: AgeResidualQ4Metadata,
}

fn bound(bytes: &[u8]) -> ReadBoundFile {
    ReadBoundFile {
        bytes: bytes.len(),
        sha256: sha256_bytes(bytes),
    }
}
fn f32_bytes(values: &[f32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|v| v.to_bits().to_le_bytes())
        .collect()
}
fn i64_bytes(values: &[i64]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}

/// An admitted immutable snapshot, not a mutable optimizer parameter.
pub struct AgeSource {
    metadata: AgeSourceMetadata,
    raw: Vec<f32>,
    packed: Vec<u8>,
    expanded: Vec<i64>,
    tokenizer: Vec<u8>,
}
impl AgeSource {
    pub fn new(current_absolute: &Tensor, base: &ReadSourceBinding) -> Result<Self> {
        let lineage = base.age_lineage();
        let raw = tensor_age(current_absolute, lineage.heads, lineage.context)?;
        Self::from_raw_bytes(&f32_bytes(&raw), base)
    }

    pub(crate) fn from_raw_bytes(bytes: &[u8], base: &ReadSourceBinding) -> Result<Self> {
        let lineage = base.age_lineage();
        let config = AgeResidualQ4Config {
            heads: lineage.heads,
            context: lineage.context,
        };
        let count = config
            .coefficient_count()
            .map_err(|e| invalid(e.to_string()))?;
        if bytes.len() != count * 4 {
            return Err(invalid("learned age source raw byte length differs"));
        }
        let raw = bytes
            .chunks_exact(4)
            .map(|v| f32::from_le_bytes([v[0], v[1], v[2], v[3]]))
            .collect::<Vec<_>>();
        let codec = q4_age_residual(config, &raw)?;
        let metadata = AgeSourceMetadata {
            schema: SCHEMA.into(),
            lineage,
            current_age_f32: bound(bytes),
            expanded_age_q24: bound(&i64_bytes(codec.age_q24())),
            residual: residual_metadata(config, &codec)?,
        };
        Ok(Self {
            metadata,
            raw,
            packed: codec.packed().to_vec(),
            expanded: codec.age_q24().to_vec(),
            tokenizer: base.age_tokenizer().to_vec(),
        })
    }

    pub fn metadata(&self) -> &AgeSourceMetadata {
        &self.metadata
    }
    pub fn tensor(&self) -> Result<Tensor> {
        Ok(Tensor::from_vec(
            self.raw.clone(),
            (self.metadata.lineage.heads, self.metadata.lineage.context),
            &Device::Cpu,
        )?)
    }
    pub fn packed_coefficients(&self) -> &[u8] {
        &self.packed
    }
    pub fn age_q24(&self) -> &[i64] {
        &self.expanded
    }
    pub(crate) fn raw_bytes(&self) -> Vec<u8> {
        f32_bytes(&self.raw)
    }
    pub(crate) fn raw_values(&self) -> &[f32] {
        &self.raw
    }

    pub fn validate_for(&self, current_absolute: &Tensor, base: &ReadSourceBinding) -> Result<()> {
        self.validate_base(base)?;
        if f32_bytes(&tensor_age(
            current_absolute,
            self.metadata.lineage.heads,
            self.metadata.lineage.context,
        )?) != self.raw_bytes()
        {
            return Err(invalid(
                "learned age source is stale for current raw parameter bits",
            ));
        }
        Ok(())
    }
    pub(crate) fn validate_base(&self, base: &ReadSourceBinding) -> Result<()> {
        if self.metadata.lineage != base.age_lineage() {
            return Err(invalid("learned age source immutable base lineage differs"));
        }
        Ok(())
    }

    pub fn save(&self, directory: &Path) -> Result<()> {
        fs::create_dir(directory)?;
        for (name, bytes) in [
            (FILES[0], serde_json::to_vec_pretty(&self.metadata)?),
            (FILES[1], self.raw_bytes()),
            (FILES[2], self.packed.clone()),
            (FILES[3], i64_bytes(&self.expanded)),
            (FILES[4], self.tokenizer.clone()),
        ] {
            fs::File::create_new(directory.join(name))?.write_all(&bytes)?;
        }
        Ok(())
    }

    pub fn load(directory: &Path, base: &ReadSourceBinding) -> Result<Self> {
        let mut names = BTreeSet::new();
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                return Err(invalid("learned age source has a nonregular file"));
            }
            names.insert(
                entry
                    .file_name()
                    .into_string()
                    .map_err(|_| invalid("learned age source has a non-UTF8 filename"))?,
            );
        }
        if names != FILES.iter().map(|s| s.to_string()).collect() {
            return Err(invalid("learned age source file set differs"));
        }
        let expected = Self::from_raw_bytes(&fs::read(directory.join(RAW_FILE))?, base)?;
        let saved: AgeSourceMetadata =
            serde_json::from_slice(&fs::read(directory.join(FILES[0]))?)?;
        if saved != expected.metadata
            || fs::read(directory.join(FILES[2]))? != expected.packed
            || fs::read(directory.join(FILES[3]))? != i64_bytes(&expected.expanded)
            || fs::read(directory.join(FILES[4]))? != expected.tokenizer
        {
            return Err(invalid(
                "learned age source differs from raw/base/prior regeneration",
            ));
        }
        Ok(expected)
    }
}

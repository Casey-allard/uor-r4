//! Retained packed-code decoder, extracted from the offline trainer.
//! Hash, layout, reserved-code, exponent and inventory checks are preserved.
//! Legacy error statistics are JSON numbers inspected only during loading;
//! parameter decoding and all model computation use integer codes directly.

use crate::{invalid, Result};
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::Path;

pub const SPEC_SCHEMA: &str = "uor-r4.joint-dyadic-quantization/2";
pub const HARD_SCHEMA: &str = "uor-r4.joint-dyadic-hard-parameters/1";
pub const CALIBRATION_RULE: &str = "frozen-parent; signed4 minimum F64 sum of squared F32 reconstruction error over clamped/deduplicated [ceiling-2,ceiling-1,ceiling], ties larger exponent; signed16 ceiling only; ceiling=ceil(log2(maxabs/qmax)) clamped [-24,16]; exact-zero ceiling=0; ties round away from zero";
pub const MIN_EXPONENT: i16 = -24;
pub const MAX_EXPONENT: i16 = 16;
pub const HARD_PARAMETERS_FILE: &str = "hard-parameters.bin";
pub const HARD_PARAMETERS_MANIFEST_FILE: &str = "hard-parameters.json";
pub const HARD_DESCRIPTOR_FILE: &str = HARD_PARAMETERS_MANIFEST_FILE;
const ENCODING: &str = "signed4-low-nibble-first-reserved-minus8;signed16-le-reserved-minus32768";
const MAX_PARAMETERS: usize = 256;
const MAX_TOTAL_ELEMENTS: usize = 16 * 1024 * 1024;
const MAX_DESCRIPTOR_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuantizationSpec {
    pub schema: String,
    #[serde(deserialize_with = "unique_parameters")]
    pub parameters: BTreeMap<String, ParameterQuantization>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParameterQuantization {
    pub shape: Vec<usize>,
    pub bits: u8,
    /// One exponent per output row for matrices; one exponent for a vector.
    pub row_exponents: Vec<i16>,
}

fn unique_parameters<'de, D>(
    deserializer: D,
) -> std::result::Result<BTreeMap<String, ParameterQuantization>, D::Error>
where
    D: Deserializer<'de>,
{
    struct UniqueParameters;
    impl<'de> Visitor<'de> for UniqueParameters {
        type Value = BTreeMap<String, ParameterQuantization>;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a map of unique parameter names")
        }
        fn visit_map<M>(self, mut access: M) -> std::result::Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut result = BTreeMap::new();
            while let Some((name, value)) = access.next_entry::<String, ParameterQuantization>()? {
                if result.len() >= MAX_PARAMETERS || result.insert(name.clone(), value).is_some() {
                    return Err(serde::de::Error::custom(format!(
                        "duplicate parameter or parameter-count limit: {name}"
                    )));
                }
            }
            Ok(result)
        }
    }
    deserializer.deserialize_map(UniqueParameters)
}

fn required_bits(name: &str) -> Result<u8> {
    if name.is_empty()
        || name.len() > 256
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_'))
    {
        return Err(invalid("invalid quantized parameter name"));
    }
    if name.ends_with(".weight") {
        Ok(4)
    } else if name.ends_with(".bias") || name == "read.age" {
        Ok(16)
    } else {
        Err(invalid(format!("unsupported quantized parameter {name}")))
    }
}

fn elements(shape: &[usize]) -> Result<usize> {
    if !matches!(shape.len(), 1 | 2) || shape.contains(&0) {
        return Err(invalid(
            "quantized parameters require nonempty vectors or matrices",
        ));
    }
    let count = shape.iter().try_fold(1usize, |count, &dimension| {
        count
            .checked_mul(dimension)
            .ok_or_else(|| invalid("parameter shape overflow"))
    })?;
    if count > MAX_TOTAL_ELEMENTS {
        return Err(invalid("quantized parameter element limit"));
    }
    Ok(count)
}

impl ParameterQuantization {
    fn validate(&self, name: &str) -> Result<usize> {
        let count = elements(&self.shape)?;
        if self.bits != required_bits(name)? || (self.bits == 16 && self.shape.len() != 1) {
            return Err(invalid(format!(
                "parameter quantization kind differs for {name}"
            )));
        }
        let rows = if self.shape.len() == 2 {
            self.shape[0]
        } else {
            1
        };
        if self.row_exponents.len() != rows {
            return Err(invalid(format!(
                "parameter scale-row count differs for {name}"
            )));
        }
        for &exponent in &self.row_exponents {
            if !(MIN_EXPONENT..=MAX_EXPONENT).contains(&exponent) {
                return Err(invalid("dyadic exponent outside [-24,16]"));
            }
        }
        Ok(count)
    }
}

impl QuantizationSpec {
    /// Structural bounds also apply to decoded, untrusted descriptors.
    pub fn validate_structure(&self) -> Result<()> {
        if self.schema != SPEC_SCHEMA
            || self.parameters.is_empty()
            || self.parameters.len() > MAX_PARAMETERS
        {
            return Err(invalid(
                "invalid dyadic quantization schema or parameter inventory",
            ));
        }
        let mut total = 0usize;
        for (name, parameter) in &self.parameters {
            total = total
                .checked_add(parameter.validate(name)?)
                .ok_or_else(|| invalid("parameter total overflow"))?;
            if total > MAX_TOTAL_ELEMENTS {
                return Err(invalid("hard parameter total element limit"));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CodeEntry {
    name: String,
    offset: u64,
    elements: usize,
    bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HardDescriptor {
    schema: String,
    encoding: String,
    calibration_rule: String,
    specification: QuantizationSpec,
    specification_sha256: String,
    payload_bytes: u64,
    payload_sha256: String,
    binding_sha256: String,
    parameters: Vec<CodeEntry>,
    /// Diagnostics against the values at export, not required for decoding.
    parameter_statistics: BTreeMap<String, ParameterStatistics>,
    numerical_scope: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ParameterStatistics {
    elements: usize,
    clipped_values: usize,
    zero_codes: usize,
    distinct_codes: usize,
    max_absolute_error: serde_json::Number,
    squared_error_sum: serde_json::Number,
}

fn hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn binding_hash(specification: &[u8], payload: &[u8]) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(HARD_SCHEMA.as_bytes());
    hasher.update([0]);
    hasher.update(
        u64::try_from(specification.len())
            .map_err(|_| invalid("spec length overflow"))?
            .to_le_bytes(),
    );
    hasher.update(specification);
    hasher.update(
        u64::try_from(payload.len())
            .map_err(|_| invalid("payload length overflow"))?
            .to_le_bytes(),
    );
    hasher.update(payload);
    Ok(hex::encode(hasher.finalize()))
}

fn layout(spec: &QuantizationSpec) -> Result<(Vec<CodeEntry>, u64)> {
    spec.validate_structure()?;
    let mut entries = Vec::with_capacity(spec.parameters.len());
    let mut offset = 0u64;
    for (name, parameter) in &spec.parameters {
        let count = elements(&parameter.shape)?;
        let length = match parameter.bits {
            4 => count.checked_add(1).map(|value| value / 2),
            16 => count.checked_mul(2),
            _ => None,
        }
        .ok_or_else(|| invalid("hard parameter byte-count overflow"))?;
        let bytes =
            u64::try_from(length).map_err(|_| invalid("hard parameter byte-count overflow"))?;
        entries.push(CodeEntry {
            name: name.clone(),
            offset,
            elements: count,
            bytes,
        });
        offset = offset
            .checked_add(bytes)
            .ok_or_else(|| invalid("hard parameter offset overflow"))?;
    }
    Ok((entries, offset))
}

// serde_json rejects nonfinite JSON numbers. Preserve the prior >=0 diagnostic
// check including negative zero, without converting diagnostics into model data.
fn nonnegative_number(number: &serde_json::Number) -> bool {
    let encoded = number.to_string();
    if !encoded.starts_with('-') {
        return true;
    }
    encoded[1..]
        .split(['e', 'E'])
        .next()
        .is_some_and(|mantissa| !mantissa.bytes().any(|byte| matches!(byte, b'1'..=b'9')))
}

pub fn load_hard_codes(directory: &Path) -> Result<(QuantizationSpec, BTreeMap<String, Vec<i16>>)> {
    let descriptor_path = directory.join(HARD_DESCRIPTOR_FILE);
    if fs::metadata(&descriptor_path)?.len() > MAX_DESCRIPTOR_BYTES {
        return Err(invalid("hard parameter descriptor exceeds size limit"));
    }
    let descriptor: HardDescriptor = serde_json::from_slice(&fs::read(descriptor_path)?)?;
    if descriptor.schema != HARD_SCHEMA
        || descriptor.encoding != ENCODING
        || descriptor.calibration_rule != CALIBRATION_RULE
    {
        return Err(invalid("hard parameter format mismatch"));
    }
    let (expected_entries, expected_bytes) = layout(&descriptor.specification)?;
    if descriptor.parameters != expected_entries || descriptor.payload_bytes != expected_bytes {
        return Err(invalid(
            "hard parameter names/offsets/shapes/lengths differ",
        ));
    }
    if !descriptor
        .parameter_statistics
        .keys()
        .eq(descriptor.specification.parameters.keys())
    {
        return Err(invalid("hard parameter statistics inventory differs"));
    }
    for entry in &expected_entries {
        let stats = &descriptor.parameter_statistics[&entry.name];
        if stats.elements != entry.elements
            || stats.clipped_values > entry.elements
            || stats.zero_codes > entry.elements
            || stats.distinct_codes > entry.elements
            || !nonnegative_number(&stats.max_absolute_error)
            || !nonnegative_number(&stats.squared_error_sum)
        {
            return Err(invalid("invalid hard parameter diagnostic statistics"));
        }
    }
    let data_path = directory.join(HARD_PARAMETERS_FILE);
    if fs::metadata(&data_path)?.len() != expected_bytes {
        return Err(invalid(
            "hard parameter payload truncated or has trailing data",
        ));
    }
    let payload = fs::read(data_path)?;
    let specification = serde_json::to_vec(&descriptor.specification)?;
    if payload.len() as u64 != expected_bytes
        || hash(&specification) != descriptor.specification_sha256
        || hash(&payload) != descriptor.payload_sha256
        || binding_hash(&specification, &payload)? != descriptor.binding_sha256
    {
        return Err(invalid(
            "hard parameter payload/specification hash mismatch",
        ));
    }
    let mut variables = BTreeMap::new();
    for entry in &expected_entries {
        let parameter = &descriptor.specification.parameters[&entry.name];
        let start =
            usize::try_from(entry.offset).map_err(|_| invalid("hard parameter offset overflow"))?;
        let length =
            usize::try_from(entry.bytes).map_err(|_| invalid("hard parameter length overflow"))?;
        let end = start
            .checked_add(length)
            .ok_or_else(|| invalid("hard parameter end overflow"))?;
        let bytes = payload
            .get(start..end)
            .ok_or_else(|| invalid("hard parameter offset outside payload"))?;
        if parameter.bits == 4
            && entry.elements % 2 == 1
            && bytes.last().is_some_and(|byte| byte >> 4 != 0)
        {
            return Err(invalid("nonzero unused hard parameter padding nibble"));
        }
        let mut values = Vec::with_capacity(entry.elements);
        for index in 0..entry.elements {
            let code = if parameter.bits == 4 {
                let nibble = (bytes[index / 2] >> (4 * (index % 2))) & 0x0f;
                if nibble == 8 {
                    return Err(invalid("reserved signed4 code -8"));
                }
                if nibble >= 8 {
                    i32::from(nibble) - 16
                } else {
                    i32::from(nibble)
                }
            } else {
                let offset = index * 2;
                let code = i16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
                if code == i16::MIN {
                    return Err(invalid("reserved signed16 code -32768"));
                }
                i32::from(code)
            };
            values.push(code as i16);
        }
        variables.insert(entry.name.clone(), values);
    }
    descriptor.specification.validate_structure()?;
    Ok((descriptor.specification, variables))
}

/// Verify a sealed report's complete file set without depending on core/model
/// crates. This preserves the artifact table loader's existing BLAKE3 seal
/// contract and rejects duplicate, escaping, symlink or special-file entries.
/// Hash verification is loading work, not numerical inference.
pub fn verify_sealed(directory: &Path) -> Result<()> {
    let manifest_path = directory.join("manifest.json");
    if fs::symlink_metadata(&manifest_path)?
        .file_type()
        .is_symlink()
        || fs::metadata(&manifest_path)?.len() > 4 * 1024 * 1024
    {
        return Err(invalid("sealed manifest type or size differs"));
    }
    let manifest: serde_json::Value = serde_json::from_slice(&fs::read(&manifest_path)?)?;
    let entries = manifest["files"]
        .as_array()
        .filter(|entries| entries.len() <= 4096)
        .ok_or_else(|| invalid("sealed manifest has invalid files"))?;
    if manifest["schema"] != "uor-r4.report-manifest/1" {
        return Err(invalid("sealed manifest schema differs"));
    }
    let mut listed = std::collections::BTreeSet::new();
    for entry in entries {
        let relative = entry["path"]
            .as_str()
            .ok_or_else(|| invalid("sealed manifest entry lacks path"))?;
        let path = Path::new(relative);
        if relative.is_empty()
            || relative == "manifest.json"
            || path
                .components()
                .any(|component| !matches!(component, std::path::Component::Normal(_)))
            || !listed.insert(relative.to_owned())
        {
            return Err(invalid("sealed manifest path escapes or repeats"));
        }
    }
    let mut actual = std::collections::BTreeSet::new();
    collect_sealed(directory, directory, &mut actual, 0)?;
    actual.remove("manifest.json");
    if actual != listed {
        return Err(invalid("sealed directory complete file set differs"));
    }
    for entry in entries {
        let relative = entry["path"]
            .as_str()
            .ok_or_else(|| invalid("sealed path missing"))?;
        let path = directory.join(relative);
        let length = fs::metadata(&path)?.len();
        if length > 128 * 1024 * 1024 || entry["bytes"].as_u64() != Some(length) {
            return Err(invalid(format!("sealed file size differs: {relative}")));
        }
        let bytes = fs::read(path)?;
        if entry["blake3"].as_str() != Some(blake3::hash(&bytes).to_hex().as_str()) {
            return Err(invalid(format!("sealed file changed: {relative}")));
        }
    }
    Ok(())
}

fn collect_sealed(
    root: &Path,
    directory: &Path,
    output: &mut std::collections::BTreeSet<String>,
    depth: usize,
) -> Result<()> {
    if depth > 32 {
        return Err(invalid("sealed directory nesting limit"));
    }
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let path = entry.path();
        if kind.is_dir() {
            collect_sealed(root, &path, output, depth + 1)?;
        } else if kind.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|e| invalid(e.to_string()))?;
            let relative = relative
                .to_str()
                .ok_or_else(|| invalid("non-UTF8 sealed path"))?;
            if !output.insert(relative.to_owned()) || output.len() > 4097 {
                return Err(invalid("sealed file count limit or duplicate"));
            }
        } else {
            return Err(invalid(
                "sealed directory contains a symlink or special file",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct TemporaryDirectory(PathBuf);
    impl TemporaryDirectory {
        fn new() -> Result<Self> {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "uor-integer-format-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path)?;
            Ok(Self(path))
        }
    }
    impl Drop for TemporaryDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn fixture() -> Result<(HardDescriptor, Vec<u8>)> {
        let specification = QuantizationSpec {
            schema: SPEC_SCHEMA.into(),
            parameters: BTreeMap::from([
                (
                    "output.bias".into(),
                    ParameterQuantization {
                        shape: vec![2],
                        bits: 16,
                        row_exponents: vec![-12],
                    },
                ),
                (
                    "output.norm.weight".into(),
                    ParameterQuantization {
                        shape: vec![3],
                        bits: 4,
                        row_exponents: vec![-2],
                    },
                ),
            ]),
        };
        let (parameters, payload_bytes) = layout(&specification)?;
        let parameter_statistics = parameters
            .iter()
            .map(|entry| {
                (
                    entry.name.clone(),
                    ParameterStatistics {
                        elements: entry.elements,
                        clipped_values: 0,
                        zero_codes: 0,
                        distinct_codes: entry.elements,
                        max_absolute_error: 0.into(),
                        squared_error_sum: 0.into(),
                    },
                )
            })
            .collect();
        Ok((
            HardDescriptor {
                schema: HARD_SCHEMA.into(),
                encoding: ENCODING.into(),
                calibration_rule: CALIBRATION_RULE.into(),
                specification,
                specification_sha256: String::new(),
                payload_bytes,
                payload_sha256: String::new(),
                binding_sha256: String::new(),
                parameters,
                parameter_statistics,
                numerical_scope: "fixture integer codes".into(),
            },
            vec![1, 0, 255, 127, 0x79, 0x01],
        ))
    }

    fn write_fixture(
        directory: &Path,
        descriptor: &mut HardDescriptor,
        payload: &[u8],
    ) -> Result<()> {
        let specification = serde_json::to_vec(&descriptor.specification)?;
        descriptor.specification_sha256 = hash(&specification);
        descriptor.payload_sha256 = hash(payload);
        descriptor.binding_sha256 = binding_hash(&specification, payload)?;
        fs::write(directory.join(HARD_PARAMETERS_FILE), payload)?;
        fs::write(
            directory.join(HARD_DESCRIPTOR_FILE),
            serde_json::to_vec(descriptor)?,
        )?;
        Ok(())
    }

    #[test]
    fn packed_codec_preserves_codes_and_rejects_mutated_contracts() -> Result<()> {
        let directory = TemporaryDirectory::new()?;
        let (original, payload) = fixture()?;
        write_fixture(&directory.0, &mut original.clone(), &payload)?;
        let (spec, codes) = load_hard_codes(&directory.0)?;
        assert_eq!(spec, original.specification);
        assert_eq!(codes["output.bias"], [1, 32767]);
        assert_eq!(codes["output.norm.weight"], [-7, 7, 1]);
        let mut changed = payload.clone();
        changed[0] ^= 1;
        fs::write(directory.0.join(HARD_PARAMETERS_FILE), &changed)?;
        assert!(
            load_hard_codes(&directory.0).is_err(),
            "changed payload hash"
        );
        for (offset, bytes) in [(0, vec![0, 128]), (4, vec![0x78]), (5, vec![0x11])] {
            let mut changed = payload.clone();
            changed[offset..offset + bytes.len()].copy_from_slice(&bytes);
            write_fixture(&directory.0, &mut original.clone(), &changed)?;
            assert!(
                load_hard_codes(&directory.0).is_err(),
                "reserved code or padding"
            );
        }
        let mut changed = payload.clone();
        changed.push(0);
        write_fixture(&directory.0, &mut original.clone(), &changed)?;
        assert!(load_hard_codes(&directory.0).is_err(), "trailing payload");
        let mut descriptor = original.clone();
        descriptor.parameters[0].offset = 1;
        write_fixture(&directory.0, &mut descriptor, &payload)?;
        assert!(load_hard_codes(&directory.0).is_err(), "offset binding");
        let mut descriptor = original;
        descriptor
            .specification
            .parameters
            .get_mut("output.bias")
            .ok_or_else(|| invalid("fixture missing bias"))?
            .row_exponents[0] = 17;
        write_fixture(&directory.0, &mut descriptor, &payload)?;
        assert!(load_hard_codes(&directory.0).is_err(), "exponent bound");
        Ok(())
    }

    #[test]
    fn packed_codec_rejects_duplicate_parameters_and_negative_diagnostics() -> Result<()> {
        let duplicate = r#"{"schema":"uor-r4.joint-dyadic-quantization/2","parameters":{"x.bias":{"shape":[1],"bits":16,"row_exponents":[0]},"x.bias":{"shape":[1],"bits":16,"row_exponents":[0]}}}"#;
        assert!(serde_json::from_str::<QuantizationSpec>(duplicate).is_err());
        for value in ["0", "-0.0", "0.25", "1e30"] {
            assert!(nonnegative_number(&serde_json::from_str(value)?));
        }
        for value in ["-1", "-0.25", "-1e-30"] {
            assert!(!nonnegative_number(&serde_json::from_str(value)?));
        }
        let directory = TemporaryDirectory::new()?;
        let (mut descriptor, payload) = fixture()?;
        descriptor
            .parameter_statistics
            .get_mut("output.bias")
            .ok_or_else(|| invalid("fixture missing stats"))?
            .squared_error_sum = (-1).into();
        write_fixture(&directory.0, &mut descriptor, &payload)?;
        assert!(load_hard_codes(&directory.0).is_err());
        Ok(())
    }

    #[test]
    fn sealed_artifact_checks_complete_set_and_hashes() -> Result<()> {
        let directory = TemporaryDirectory::new()?;
        fs::write(directory.0.join("payload"), b"retained")?;
        let manifest = serde_json::json!({"schema":"uor-r4.report-manifest/1", "files":[{
            "path":"payload", "bytes":8, "blake3":blake3::hash(b"retained").to_hex().to_string()
        }]});
        fs::write(
            directory.0.join("manifest.json"),
            serde_json::to_vec(&manifest)?,
        )?;
        verify_sealed(&directory.0)?;
        fs::write(directory.0.join("extra"), b"unlisted")?;
        assert!(verify_sealed(&directory.0).is_err());
        fs::remove_file(directory.0.join("extra"))?;
        fs::write(directory.0.join("payload"), b"modified")?;
        assert!(verify_sealed(&directory.0).is_err());
        Ok(())
    }
}

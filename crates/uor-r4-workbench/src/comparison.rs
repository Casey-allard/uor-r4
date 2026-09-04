//! Separately admitted, non-listening host comparison mode.

use crate::authority::{frozen_accepted_binding, validate_private_release_contract};
use crate::base64::decode_canonical;
use crate::intake::validate_expected_binding;
use crate::launch::{hash_inherited_executable, rust_target, VerifiedExecutable};
use crate::strict_json;
use crate::wire::{FileIdentity, ARTIFACT_SHA256, ORIGINAL_EXPORT_RELEASE_SHA256};
use crate::{BoxError, ARTIFACT_BYTES, SERVICE_CONTRACT_SHA256, TARGET};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use uor_r4_api::learned_reference::{
    floating_point_environment, ComparisonAdmission, ExpectedBinding, LoadedResearchReference,
    RawRequest, RuntimeIdentity, OPERATOR_PROFILE,
};

const MAX_FRAME_BYTES: usize = 65_536;
const MAX_RELEASE_BYTES: u64 = 1_048_576;
const MAX_ADMISSION_BYTES: u64 = 1_048_576;
const MAX_PLAN_BYTES: u64 = 8_388_608;
const MAX_ASSET_MANIFEST_BYTES: u64 = 131_072;
const MAX_BINDING_BYTES: u64 = 65_536;
const MAX_ROWS: u64 = 336;
const MAX_FORWARDS: u64 = 320;
const MAX_REFUSALS: u64 = 16;
const UINT53_MAX: u64 = 9_007_199_254_740_991;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FreshExecutionRelease {
    schema: String,
    issue: u64,
    service_contract_sha256: String,
    source_revision: String,
    source_manifest: FileIdentity,
    build_receipt: FileIdentity,
    native_binary_sha256: String,
    target: String,
    operator_profile: String,
    runtime_receipt: FileIdentity,
    asset_manifest: FileIdentity,
    artifact: FileIdentity,
    expected_binding: FileIdentity,
    original_export_release: FileIdentity,
    comparison_plan: FileIdentity,
    row_cap: u64,
    forward_cap: u64,
    refusal_cap: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct IndependentAdmission {
    schema: String,
    terminal: String,
    release_sha256: String,
    native_binary_sha256: String,
    service_contract_sha256: String,
    comparison_plan_sha256: String,
    one_execution: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PrivateRequest {
    schema: String,
    text_b64: String,
    request_extras: BTreeMap<String, Value>,
}

#[derive(Default, Serialize)]
struct Work {
    model_loads: u64,
    logical_forwards: u64,
    refusal_rows: u64,
    rows: u64,
    qualification_calls: u64,
}

pub fn emit_private_metadata() -> Result<(), BoxError> {
    let executable = VerifiedExecutable::open_current()?;
    let metadata = json!({
        "schema": "uor-r4.workbench-runtime-metadata/1",
        "native_binary_sha256": executable.sha256(),
        "native_binary_bytes": executable.bytes(),
        "target": rust_target(),
        "arch": std::env::consts::ARCH,
        "os": std::env::consts::OS,
        "fpcr": floating_point_environment()?,
        "model_loads": 0,
        "logical_forwards": 0,
        "qualification_calls": 0,
        "listener_binds": 0
    });
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, &metadata)?;
    stdout.write_all(b"\n")?;
    stdout.flush()?;
    Ok(())
}

pub fn run(
    release_path: &Path,
    release_sha256: &str,
    admission_path: &Path,
    admission_sha256: &str,
) -> Result<(), BoxError> {
    let mut work = Work::default();
    let result = run_inner(
        release_path,
        release_sha256,
        admission_path,
        admission_sha256,
        &mut work,
    );
    if let Err(error) = &result {
        let _ = write_frame(
            &mut io::stdout().lock(),
            &json!({
                "schema":"uor-r4.workbench-private-comparison-reply/1",
                "kind":"error",
                "message": bounded_message(error),
                "work":work,
                "fpcr":floating_point_environment().ok()
            }),
        );
    }
    result
}

fn run_inner(
    release_path: &Path,
    release_sha256: &str,
    admission_path: &Path,
    admission_sha256: &str,
    work: &mut Work,
) -> Result<(), BoxError> {
    // Permission is checked against the actual inherited executable image
    // before any release-referenced artifact path is opened.
    validate_private_release_contract()?;
    let (executing_sha256, executing_bytes) = hash_inherited_executable()?;
    validate_hex(release_sha256)?;
    validate_hex(admission_sha256)?;
    let release_bytes = read_explicit(release_path, release_sha256, MAX_RELEASE_BYTES)?;
    let admission_bytes = read_explicit(admission_path, admission_sha256, MAX_ADMISSION_BYTES)?;
    let release: FreshExecutionRelease = strict_json::from_slice(&release_bytes)?;
    let admission: IndependentAdmission = strict_json::from_slice(&admission_bytes)?;
    validate_release(&release, release_sha256, &admission, &executing_sha256)?;

    // Validate every fresh-release binding before opening the model artifact.
    read_identity(&release.source_manifest, MAX_RELEASE_BYTES)?;
    read_identity(&release.build_receipt, MAX_RELEASE_BYTES)?;
    let runtime_receipt = read_identity(&release.runtime_receipt, MAX_RELEASE_BYTES)?;
    read_identity(&release.asset_manifest, MAX_ASSET_MANIFEST_BYTES)?;
    let expected_bytes = read_identity(&release.expected_binding, MAX_BINDING_BYTES)?;
    let original_release = read_identity(&release.original_export_release, MAX_RELEASE_BYTES)?;
    read_identity(&release.comparison_plan, MAX_PLAN_BYTES)?;

    let expected: ExpectedBinding = strict_json::from_slice(&expected_bytes)?;
    let frozen_binding = frozen_accepted_binding()?;
    validate_expected_binding(&expected, &frozen_binding)?;
    if expected.artifact_sha256 != release.artifact.sha256
        || expected.export_release_sha256 != release.original_export_release.sha256
        || expected.contract_sha256 != uor_r4_api::learned_reference::CONTRACT_SHA256
        || expected.operator_profile != OPERATOR_PROFILE
    {
        return Err("expected binding does not match private release".into());
    }

    let runtime = RuntimeIdentity {
        native_binary_sha256: executing_sha256.clone(),
        runtime_receipt_sha256: release.runtime_receipt.sha256.clone(),
    };
    // The runtime receipt is externally adopted opaque evidence at this layer,
    // but its exact bytes were checked above and remain bound by this identity.
    if runtime_receipt.is_empty() {
        return Err("empty runtime receipt".into());
    }
    let admission = ComparisonAdmission::from_trusted_release(
        &original_release,
        &release.original_export_release.sha256,
        runtime,
    )?;

    if release.artifact.bytes != ARTIFACT_BYTES {
        return Err("artifact size is not the fixed service artifact size".into());
    }
    let artifact = read_identity(&release.artifact, ARTIFACT_BYTES)?;
    work.model_loads = 1;
    let (engine, validation_audit) = LoadedResearchReference::load_audited(artifact, &expected);
    let engine = engine?;
    if engine.artifact_sha256() != release.artifact.sha256
        || engine.owned_artifact_bytes() as u64 != ARTIFACT_BYTES
    {
        return Err("loaded artifact identity mismatch".into());
    }

    write_frame(
        &mut io::stdout().lock(),
        &json!({
            "schema":"uor-r4.workbench-private-comparison-reply/1",
            "kind":"ready",
            "native_binary_sha256":executing_sha256,
            "native_binary_bytes":executing_bytes,
            "runtime_receipt_sha256":release.runtime_receipt.sha256,
            "artifact_sha256":engine.artifact_sha256(),
            "native_state_sha256":engine.manifest().native_state_sha256,
            "validation_audit":validation_audit,
            "fpcr":floating_point_environment()?,
            "model_loads":1,
            "logical_forwards":0,
            "qualification_calls":0
        }),
    )?;

    let mut stdin = io::stdin().lock();
    while work.rows < release.row_cap {
        let bytes = read_frame(&mut stdin)?.ok_or("incomplete comparison population")?;
        let packet: PrivateRequest = strict_json::from_slice(&bytes)?;
        if packet.schema != "uor-r4.workbench-private-comparison-request/1" {
            return Err("private request schema mismatch".into());
        }
        let raw = decode_canonical(&packet.text_b64, 8_192)?;
        let request_schema = if packet.request_extras.is_empty() {
            "uor-r4.text-to-clauses/1"
        } else {
            ""
        };
        let mut attempted = 0u32;
        let evaluated = engine.compare(
            RawRequest {
                schema: request_schema,
                text: &raw,
            },
            &admission,
            release.forward_cap.saturating_sub(work.logical_forwards) as u32,
            &mut attempted,
        );
        work.rows += 1;
        work.logical_forwards = work
            .logical_forwards
            .checked_add(attempted as u64)
            .ok_or("forward count overflow")?;
        if work.logical_forwards > release.forward_cap {
            return Err("private comparison forward cap exceeded".into());
        }
        let output = evaluated?;
        if output.diagnostics.is_none() {
            work.refusal_rows += 1;
        }
        if work.refusal_rows > release.refusal_cap {
            return Err("private comparison population cap exceeded".into());
        }
        let tensors = output.diagnostics.as_ref().map(|diagnostics| {
            json!({
                "role_attention": f32_hex(&diagnostics.role_attention),
                "role_vectors": f32_hex(&diagnostics.role_vectors),
                "binding_attention": f32_hex(&diagnostics.binding_attention),
                "logits": f32_hex(&diagnostics.logits)
            })
        });
        let diagnostic_indices = output.diagnostics.as_ref().map(|diagnostics| {
            json!({
                "role_argmax":diagnostics.role_argmax,
                "token_frame_indices":diagnostics.token_frame_indices,
                "clause_frame_indices":diagnostics.clause_frame_indices
            })
        });
        write_frame(
            &mut io::stdout().lock(),
            &json!({
                "schema":"uor-r4.workbench-private-comparison-reply/1",
                "kind":"result",
                "result":output.result,
                "parsed":output.parsed,
                "tensors":tensors,
                "diagnostics":diagnostic_indices,
                "logical_forwards":attempted,
                "receipt":output.receipt
            }),
        )?;
    }

    // `row_cap` is an exact finite population, so completion cannot depend on
    // the sender closing stdin. Emit `done` immediately after the final result;
    // any later bytes are outside this one-execution protocol.
    if work.logical_forwards != release.forward_cap || work.refusal_rows != release.refusal_cap {
        return Err("private comparison population counts do not match release".into());
    }
    write_frame(
        &mut io::stdout().lock(),
        &json!({
            "schema":"uor-r4.workbench-private-comparison-reply/1",
            "kind":"done",
            "rows":work.rows,
            "logical_forwards":work.logical_forwards,
            "refusal_rows":work.refusal_rows,
            "model_loads":work.model_loads,
            "qualification_calls":0,
            "parameter_updates":0,
            "native_state_sha256":engine.manifest().native_state_sha256,
            "fpcr":floating_point_environment()?
        }),
    )
}

fn validate_release(
    release: &FreshExecutionRelease,
    release_sha256: &str,
    admission: &IndependentAdmission,
    executing_sha256: &str,
) -> Result<(), BoxError> {
    if release.schema != "uor-r4.workbench-private-comparison-release/1"
        || release.issue == 0
        || release.issue > UINT53_MAX
        || release.service_contract_sha256 != SERVICE_CONTRACT_SHA256
        || release.native_binary_sha256 != executing_sha256
        || release.target != TARGET
        || release.operator_profile != OPERATOR_PROFILE
        || release.artifact.sha256 != ARTIFACT_SHA256
        || release.original_export_release.sha256 != ORIGINAL_EXPORT_RELEASE_SHA256
        || release.source_revision.len() != 40
        || !release
            .source_revision
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        || release.row_cap == 0
        || release.row_cap > MAX_ROWS
        || release.forward_cap == 0
        || release.forward_cap > MAX_FORWARDS
        || release.refusal_cap > MAX_REFUSALS
        || release.forward_cap + release.refusal_cap != release.row_cap
    {
        return Err("fresh private comparison release rejected".into());
    }
    for identity in [
        &release.source_manifest,
        &release.build_receipt,
        &release.runtime_receipt,
        &release.asset_manifest,
        &release.artifact,
        &release.expected_binding,
        &release.original_export_release,
        &release.comparison_plan,
    ] {
        validate_file_identity(identity)?;
    }
    if admission.schema != "uor-r4.workbench-private-comparison-admission/1"
        || admission.terminal != "ACCEPTED_FOR_ONE_WORKBENCH_HOST_COMPARISON"
        || admission.release_sha256 != release_sha256
        || admission.native_binary_sha256 != executing_sha256
        || admission.service_contract_sha256 != SERVICE_CONTRACT_SHA256
        || admission.comparison_plan_sha256 != release.comparison_plan.sha256
        || !admission.one_execution
    {
        return Err("independent private comparison admission rejected".into());
    }
    Ok(())
}

fn validate_file_identity(identity: &FileIdentity) -> Result<(), BoxError> {
    let path = Path::new(&identity.path);
    if !path.is_absolute()
        || identity.path.as_bytes().len() > 4_096
        || identity.bytes == 0
        || identity.bytes > UINT53_MAX
    {
        return Err("invalid private release file identity".into());
    }
    validate_hex(&identity.sha256)
}

fn read_explicit(path: &Path, expected_sha256: &str, cap: u64) -> Result<Vec<u8>, BoxError> {
    let path = path
        .to_str()
        .ok_or("private release paths must be valid UTF-8")?;
    validate_hex(expected_sha256)?;
    if !Path::new(path).is_absolute() || path.as_bytes().len() > 4_096 {
        return Err("private release path is not an admitted absolute path".into());
    }
    let identity = FileIdentity {
        path: path.to_owned(),
        bytes: regular_length(Path::new(path))?,
        sha256: expected_sha256.to_owned(),
    };
    read_identity(&identity, cap)
}

fn read_identity(identity: &FileIdentity, cap: u64) -> Result<Vec<u8>, BoxError> {
    validate_file_identity(identity)?;
    if identity.bytes > cap {
        return Err("file identity exceeds role cap".into());
    }
    let path = PathBuf::from(&identity.path);
    let mut file = open_regular_nofollow(&path)?;
    let before = file.metadata()?;
    if before.len() != identity.bytes {
        return Err("file length identity mismatch".into());
    }
    let capacity =
        usize::try_from(identity.bytes).map_err(|_| "file length does not fit memory")?;
    let mut bytes = Vec::with_capacity(capacity);
    file.by_ref().take(cap + 1).read_to_end(&mut bytes)?;
    let after = file.metadata()?;
    if bytes.len() as u64 != identity.bytes || after.len() != before.len() {
        return Err("file changed or has trailing bytes".into());
    }
    let actual = format!("{:x}", Sha256::digest(&bytes));
    if actual != identity.sha256 {
        return Err("file SHA256 identity mismatch".into());
    }
    Ok(bytes)
}

fn regular_length(path: &Path) -> Result<u64, BoxError> {
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("file must be a regular non-symlink file".into());
    }
    Ok(metadata.len())
}

#[cfg(unix)]
fn open_regular_nofollow(path: &Path) -> Result<File, BoxError> {
    use std::os::unix::fs::OpenOptionsExt;
    Ok(std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)?)
}

#[cfg(not(unix))]
fn open_regular_nofollow(_path: &Path) -> Result<File, BoxError> {
    Err(format!("unsupported runtime target; required {TARGET}").into())
}

fn validate_hex(value: &str) -> Result<(), BoxError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err("invalid lowercase SHA256".into());
    }
    Ok(())
}

fn read_frame(reader: &mut impl Read) -> Result<Option<Vec<u8>>, BoxError> {
    let mut length = [0u8; 4];
    let mut read = 0usize;
    while read < length.len() {
        match reader.read(&mut length[read..])? {
            0 if read == 0 => return Ok(None),
            0 => return Err("truncated private frame length".into()),
            count => read += count,
        }
    }
    let length = u32::from_le_bytes(length) as usize;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err("private frame length rejected".into());
    }
    let mut bytes = vec![0u8; length];
    reader.read_exact(&mut bytes)?;
    std::str::from_utf8(&bytes)?;
    Ok(Some(bytes))
}

fn write_frame(writer: &mut impl Write, value: &Value) -> Result<(), BoxError> {
    let bytes = serde_json::to_vec(value)?;
    if bytes.is_empty() || bytes.len() > MAX_FRAME_BYTES {
        return Err("private reply frame exceeds limit".into());
    }
    writer.write_all(&(bytes.len() as u32).to_le_bytes())?;
    writer.write_all(&bytes)?;
    writer.flush()?;
    Ok(())
}

fn f32_hex(values: &[f32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(values.len().saturating_mul(8));
    for byte in values.iter().flat_map(|value| value.to_le_bytes()) {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 15) as usize] as char);
    }
    out
}

fn bounded_message(error: &BoxError) -> String {
    error.to_string().chars().take(512).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_caps_are_finite_and_population_balances() {
        assert_eq!(MAX_ROWS, MAX_FORWARDS + MAX_REFUSALS);
        assert_eq!(MAX_FRAME_BYTES, 65_536);
        assert_eq!(ARTIFACT_BYTES, 2_172_252);
    }

    #[test]
    fn private_frames_reject_zero_oversize_and_truncation() {
        assert!(read_frame(&mut &0u32.to_le_bytes()[..]).is_err());
        assert!(read_frame(&mut &(65_537u32).to_le_bytes()[..]).is_err());
        let truncated = [3u32.to_le_bytes().as_slice(), b"{}"].concat();
        assert!(read_frame(&mut &truncated[..]).is_err());
    }

    #[test]
    fn private_protocol_is_not_a_public_or_worker_command() {
        let contract = include_str!("../../../docs/r4_workbench_private_release_1107.json");
        assert!(contract.contains("--private-compare-host"));
        assert!(!["load", "answer", "unload"].contains(&"compare"));
    }
}

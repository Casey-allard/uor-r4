//! Cross-representation audit of one stack contract against its run and artifact.

use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use repo_model::stack::{self, ArtifactProvenance, Probe, StackContract};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

type Fail = Box<dyn Error>;
type Result<T> = std::result::Result<T, Fail>;

const SCHEMA: &str = "uor-r4.stack-audit/1";
const EXPECTATION_KEYS: &[&str] = &[
    "status",
    "stderr_contains",
    "turns",
    "turn_count",
    "all_reply_ids_nonempty",
    "stops",
    "memory",
    "distinct_replies_min",
    "reply_text_contains",
    "same_as",
];

/// Run `cargo xtask stack-audit` from already parsed command-line arguments.
pub fn run(root: &Path, args: &[String]) -> Result<()> {
    let options = Options::parse(root, args)?;
    let contract_text = fs::read_to_string(&options.contract)?;
    let contract: StackContract = toml::from_str(&contract_text)?;
    contract.check(root)?;

    let mut report = Report::new(&options, &contract);
    if let Some(path) = &options.config {
        audit_config(&mut report, path, &contract);
    }
    if let Some(path) = &options.checkpoint {
        audit_checkpoint(&mut report, path, &contract);
    }
    if let Some(path) = &options.artifact {
        audit_artifact(&mut report, path, &options.contract, &contract);
    }
    if let (Some(tokenizer), Some(artifact)) = (&options.tokenizer, &options.artifact) {
        audit_tokenizer(&mut report, tokenizer, artifact, &contract);
    }
    if let (Some(chat), Some(artifact), Some(tokenizer)) =
        (&options.chat_bin, &options.artifact, &options.tokenizer)
    {
        audit_chat(&mut report, chat, artifact, tokenizer, &contract)?;
    } else if options.artifact.is_some() {
        report.record(
            "integer-session",
            AuditStatus::Unavailable,
            Value::String("an executable uor-chat path".to_owned()),
            Value::Null,
            "stack-audit",
            "pass --chat-bin and --tokenizer to load the artifact through the actual integer product path",
        );
    }
    let fixture_dir = options
        .probes
        .clone()
        .unwrap_or_else(|| root.join(stack::DIRECTORY).join("fixtures"));
    audit_probes(
        &mut report,
        &fixture_dir,
        &contract,
        options.chat_bin.as_deref(),
        options.artifact.as_deref(),
        options.tokenizer.as_deref(),
    )?;
    report.finish();
    let json = serde_json::to_string_pretty(&report)? + "\n";
    if let Some(path) = &options.out {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, &json)?;
    }
    print!("{json}");
    if report.overall == "MISMATCH" {
        return Err("stack audit found a cross-representation mismatch".into());
    }
    Ok(())
}

#[derive(Debug)]
struct Options {
    contract: PathBuf,
    artifact: Option<PathBuf>,
    tokenizer: Option<PathBuf>,
    config: Option<PathBuf>,
    checkpoint: Option<PathBuf>,
    chat_bin: Option<PathBuf>,
    probes: Option<PathBuf>,
    out: Option<PathBuf>,
}

impl Options {
    fn parse(root: &Path, args: &[String]) -> Result<Self> {
        let mut contract = None;
        let mut artifact = None;
        let mut tokenizer = None;
        let mut config = None;
        let mut checkpoint = None;
        let mut chat_bin = None;
        let mut probes = None;
        let mut out = None;
        let mut index = 0;
        while index < args.len() {
            let flag = args[index].as_str();
            let value = args
                .get(index + 1)
                .ok_or_else(|| format!("{flag} needs a value"))?;
            let path = {
                let path = PathBuf::from(value);
                if path.is_absolute() {
                    path
                } else {
                    root.join(path)
                }
            };
            match flag {
                "--contract" => contract = Some(path),
                "--artifact" => artifact = Some(path),
                "--tokenizer" => tokenizer = Some(path),
                "--config" => config = Some(path),
                "--checkpoint" => checkpoint = Some(path),
                "--chat-bin" => chat_bin = Some(path),
                "--probes" => probes = Some(path),
                "--out" => out = Some(path),
                other => return Err(format!("unknown stack-audit option {other}").into()),
            }
            index += 2;
        }
        Ok(Self {
            contract: contract.ok_or("--contract <PATH> is required")?,
            artifact,
            tokenizer,
            config,
            checkpoint,
            chat_bin,
            probes,
            out,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum AuditStatus {
    #[serde(rename = "MATCH")]
    Match,
    #[serde(rename = "MISMATCH")]
    Mismatch,
    #[serde(rename = "UNAVAILABLE")]
    Unavailable,
    #[serde(rename = "NOT_RUN")]
    NotRun,
}

#[derive(Debug, Serialize)]
struct AuditCheck {
    id: String,
    status: AuditStatus,
    expected: Value,
    actual: Value,
    source: String,
    detail: String,
}

#[derive(Debug, Serialize)]
struct Report {
    schema: &'static str,
    contract: String,
    source_pin: String,
    contract_semantic_blake3: String,
    artifact: Option<String>,
    artifact_sha256: Option<String>,
    overall: String,
    checks: Vec<AuditCheck>,
}

impl Report {
    fn new(options: &Options, contract: &StackContract) -> Self {
        Self {
            schema: SCHEMA,
            contract: contract.id.clone(),
            source_pin: contract.source_pin.clone(),
            contract_semantic_blake3: blake3::hash(
                &serde_json::to_vec(contract).expect("contract serializes"),
            )
            .to_hex()
            .to_string(),
            artifact: options
                .artifact
                .as_ref()
                .map(|path| path.display().to_string()),
            artifact_sha256: None,
            overall: "NOT_RUN".to_owned(),
            checks: Vec::new(),
        }
    }

    fn record(
        &mut self,
        id: impl Into<String>,
        status: AuditStatus,
        expected: Value,
        actual: Value,
        source: impl Into<String>,
        detail: impl Into<String>,
    ) {
        self.checks.push(AuditCheck {
            id: id.into(),
            status,
            expected,
            actual,
            source: source.into(),
            detail: detail.into(),
        });
    }

    fn compare(
        &mut self,
        id: impl Into<String>,
        expected: Value,
        actual: Value,
        source: &Path,
        detail: &str,
    ) {
        let status = if expected == actual {
            AuditStatus::Match
        } else {
            AuditStatus::Mismatch
        };
        self.record(
            id,
            status,
            expected,
            actual,
            source.display().to_string(),
            detail,
        );
    }

    fn finish(&mut self) {
        self.overall = if self
            .checks
            .iter()
            .any(|check| check.status == AuditStatus::Mismatch)
        {
            "MISMATCH"
        } else if self
            .checks
            .iter()
            .any(|check| check.status == AuditStatus::Unavailable)
        {
            "UNAVAILABLE"
        } else if self.checks.is_empty()
            || self
                .checks
                .iter()
                .all(|check| check.status == AuditStatus::NotRun)
        {
            "NOT_RUN"
        } else {
            "MATCH"
        }
        .to_owned();
    }
}

fn audit_config(report: &mut Report, path: &Path, contract: &StackContract) {
    let config = match read_json(path) {
        Ok(value) => value,
        Err(error) => {
            report.record(
                "run-config",
                AuditStatus::Unavailable,
                Value::Bool(true),
                Value::String(error.to_string()),
                path.display().to_string(),
                "run config is absent or unreadable",
            );
            return;
        }
    };
    compare_optional(
        report,
        "run-config-objective",
        contract.model.objective.clone(),
        &config,
        "objective",
        path,
    );
    compare_optional_usize(
        report,
        "run-config-width",
        contract.model.width,
        &config,
        "width",
        path,
    );
    compare_usize_default(
        report,
        "run-config-context",
        contract
            .model
            .training_context
            .unwrap_or(contract.model.context),
        &config,
        "context",
        192,
        path,
    );
    let hamming = config
        .get("hamming_read")
        .and_then(Value::as_u64)
        .is_some_and(|heads| heads > 0);
    let hamming_mechanisms: Vec<_> = contract
        .mechanisms
        .iter()
        .filter(|mechanism| {
            mechanism
                .sections
                .iter()
                .any(|section| section == "hamming-reads")
                || mechanism
                    .sidecars
                    .iter()
                    .any(|sidecar| sidecar.contains("hamming"))
                || mechanism.id.contains("hamming")
        })
        .collect();
    if !hamming_mechanisms.is_empty() {
        report.compare(
            "run-config-hamming-presence",
            Value::Bool(true),
            Value::Bool(hamming),
            path,
            "the contract's Hamming mechanism is enabled in the run config",
        );
    }
    let memory_contract = contract
        .mechanisms
        .iter()
        .any(|mechanism| mechanism.id.contains("turn-memory") || mechanism.id == "memory");
    if memory_contract {
        let memory = config
            .get("turn_memory")
            .and_then(Value::as_u64)
            .is_some_and(|k| k > 0);
        report.compare(
            "run-config-turn-memory-presence",
            Value::Bool(true),
            Value::Bool(memory),
            path,
            "the contract's turn-memory mechanism is enabled in the run config",
        );
    }
}

fn compare_optional(
    report: &mut Report,
    id: &str,
    expected: String,
    object: &Value,
    key: &str,
    source: &Path,
) {
    match object.get(key).and_then(Value::as_str) {
        Some(actual) => report.compare(
            id,
            Value::String(expected),
            Value::String(actual.to_owned()),
            source,
            &format!("run config `{key}` agrees with the contract"),
        ),
        None => report.record(
            id,
            AuditStatus::Unavailable,
            Value::String(expected),
            Value::Null,
            source.display().to_string(),
            format!("run config does not record `{key}`"),
        ),
    }
}

fn compare_optional_usize(
    report: &mut Report,
    id: &str,
    expected: usize,
    object: &Value,
    key: &str,
    source: &Path,
) {
    match object.get(key).and_then(Value::as_u64) {
        Some(actual) => report.compare(
            id,
            Value::from(expected),
            Value::from(actual),
            source,
            &format!("run config `{key}` agrees with the contract"),
        ),
        None => report.record(
            id,
            AuditStatus::Unavailable,
            Value::from(expected),
            Value::Null,
            source.display().to_string(),
            format!("run config does not record `{key}`"),
        ),
    }
}

fn compare_usize_default(
    report: &mut Report,
    id: &str,
    expected: usize,
    object: &Value,
    key: &str,
    default: u64,
    source: &Path,
) {
    let (actual, detail) = object
        .get(key)
        .map(|value| (value.as_u64(), format!("run config `{key}`")))
        .unwrap_or_else(|| {
            (
                Some(default),
                format!("run config omits `{key}`; trainer default {default}"),
            )
        });
    match actual {
        Some(actual) => report.compare(
            id,
            Value::from(expected),
            Value::from(actual),
            source,
            &format!("{detail} agrees with the contract"),
        ),
        None => report.record(
            id,
            AuditStatus::Mismatch,
            Value::from(expected),
            Value::Null,
            source.display().to_string(),
            format!("run config `{key}` is not an unsigned integer"),
        ),
    }
}

fn audit_checkpoint(report: &mut Report, path: &Path, contract: &StackContract) {
    if !path.is_dir() {
        report.record(
            "checkpoint",
            AuditStatus::Unavailable,
            Value::String("checkpoint directory".to_owned()),
            Value::Null,
            path.display().to_string(),
            "checkpoint is absent or is not a directory",
        );
        return;
    }
    let mut required: BTreeSet<String> = ["config.json", "model.safetensors"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    for mechanism in &contract.mechanisms {
        required.extend(
            mechanism
                .sidecars
                .iter()
                .filter(|name| !name.starts_with("artifact:"))
                .cloned(),
        );
    }
    for name in required {
        let target = path.join(&name);
        report.record(
            format!("checkpoint:{name}"),
            if target.is_file() {
                AuditStatus::Match
            } else {
                AuditStatus::Mismatch
            },
            Value::Bool(true),
            Value::Bool(target.is_file()),
            target.display().to_string(),
            "required checkpoint file or sidecar is present",
        );
    }
}

#[derive(Debug)]
struct ArtifactHeader {
    schema: String,
    shape: Value,
    source: Value,
    sections: BTreeSet<String>,
    has_hamming: bool,
    has_memory: bool,
    has_memory_chain: bool,
    has_point: bool,
}

fn parse_artifact(path: &Path) -> Result<(ArtifactHeader, String)> {
    let bytes = fs::read(path)?;
    if bytes.len() < 16 || &bytes[..8] != b"UORLUT01" {
        return Err(format!("{}: missing UORLUT01 magic", path.display()).into());
    }
    let header_len = u64::from_le_bytes(bytes[8..16].try_into().expect("8 bytes")) as usize;
    let end = 16usize
        .checked_add(header_len)
        .filter(|end| *end <= bytes.len())
        .ok_or_else(|| format!("{}: header length does not fit", path.display()))?;
    let header: Value = serde_json::from_slice(&bytes[16..end])?;
    let sections = header
        .get("matrices")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .chain(
            header
                .get("tables")
                .and_then(Value::as_array)
                .into_iter()
                .flatten(),
        )
        .filter_map(|entry| entry.get("name").and_then(Value::as_str))
        .map(str::to_owned)
        .collect();
    let sha256 = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    Ok((
        ArtifactHeader {
            schema: header
                .get("schema")
                .and_then(Value::as_str)
                .ok_or("artifact header has no schema")?
                .to_owned(),
            shape: header
                .get("shape")
                .cloned()
                .ok_or("artifact has no shape")?,
            source: header.get("source").cloned().unwrap_or(Value::Null),
            sections,
            has_hamming: header.get("hamming").is_some(),
            has_memory: header.get("memory").is_some(),
            has_memory_chain: header.get("memory_chain").is_some(),
            has_point: header.get("point").is_some(),
        },
        sha256,
    ))
}

fn audit_artifact(
    report: &mut Report,
    path: &Path,
    contract_path: &Path,
    contract: &StackContract,
) {
    let (artifact, sha256) = match parse_artifact(path) {
        Ok(parsed) => parsed,
        Err(error) => {
            report.record(
                "artifact-parse",
                if path.exists() {
                    AuditStatus::Mismatch
                } else {
                    AuditStatus::Unavailable
                },
                Value::Bool(true),
                Value::String(error.to_string()),
                path.display().to_string(),
                if path.exists() {
                    "a present artifact does not parse and validate"
                } else {
                    "artifact is absent"
                },
            );
            return;
        }
    };
    report.artifact_sha256 = Some(sha256);
    report.compare(
        "artifact-family",
        Value::String(contract.artifact.family.clone()),
        Value::String("uor-r4.lut-stack".to_owned()),
        path,
        "served container family",
    );
    if let Some(schema) = &contract.artifact.schema {
        report.compare(
            "artifact-schema",
            Value::String(schema.clone()),
            Value::String(artifact.schema.clone()),
            path,
            "exact artifact header schema",
        );
    }
    for (id, key, expected) in [
        ("artifact-vocab", "vocab", contract.model.vocab),
        ("artifact-width", "width", contract.model.width),
        ("artifact-context", "context", contract.model.context),
    ] {
        let actual = artifact
            .shape
            .get(key)
            .and_then(Value::as_u64)
            .map(Value::from)
            .unwrap_or(Value::Null);
        report.compare(id, Value::from(expected), actual, path, key);
    }
    let layers = artifact
        .shape
        .get("pattern")
        .and_then(Value::as_str)
        .map(|pattern| pattern.len())
        .unwrap_or(0);
    report.compare(
        "artifact-layers",
        Value::from(contract.model.layers),
        Value::from(layers),
        path,
        "layer pattern length",
    );
    if contract
        .mechanisms
        .iter()
        .any(|mechanism| mechanism.id.contains("hamming"))
    {
        report.compare(
            "artifact-hamming-presence",
            Value::Bool(true),
            Value::Bool(artifact.has_hamming),
            path,
            "the artifact carries the contract's Hamming reads",
        );
    }
    if contract
        .mechanisms
        .iter()
        .any(|mechanism| mechanism.id.contains("turn-memory") || mechanism.id == "memory")
    {
        report.compare(
            "artifact-turn-memory-presence",
            Value::Bool(true),
            Value::Bool(artifact.has_memory),
            path,
            "the artifact carries the contract's turn memory",
        );
    }
    if contract
        .mechanisms
        .iter()
        .any(|mechanism| mechanism.id.contains("memory-chain"))
    {
        report.compare(
            "artifact-memory-chain-presence",
            Value::Bool(true),
            Value::Bool(artifact.has_memory_chain),
            path,
            "the artifact carries the contract's memory chain",
        );
    }
    if contract
        .mechanisms
        .iter()
        .any(|mechanism| mechanism.id.contains("point-key"))
    {
        report.compare(
            "artifact-point-key-presence",
            Value::Bool(true),
            Value::Bool(artifact.has_point),
            path,
            "the artifact carries the contract's point key",
        );
    }
    for mechanism in &contract.mechanisms {
        for sidecar in &mechanism.sidecars {
            if sidecar.starts_with("artifact:") {
                let prefix = sidecar.trim_start_matches("artifact:");
                let present = artifact
                    .sections
                    .iter()
                    .any(|section| section == prefix || section.starts_with(&format!("{prefix}.")));
                report.compare(
                    format!("artifact-section:{}:{prefix}", mechanism.id),
                    Value::Bool(true),
                    Value::Bool(present),
                    path,
                    "mechanism sidecar naming an artifact section is present",
                );
            }
        }
    }
    report.record(
        "artifact-source",
        if artifact.source.is_null() {
            AuditStatus::Unavailable
        } else {
            AuditStatus::Match
        },
        Value::String("exporter provenance object".to_owned()),
        artifact.source.clone(),
        path.display().to_string(),
        "the source object identifies the exporter; serving arithmetic never reads it",
    );
    match artifact.source.get("stack_contract") {
        Some(provenance) => {
            let expected = fs::read(contract_path)
                .ok()
                .map(Sha256::digest)
                .map(|digest| hex_string(&digest))
                .unwrap_or_default();
            report.compare(
                "artifact-stack-contract-sha256",
                Value::String(expected),
                provenance.get("sha256").cloned().unwrap_or(Value::Null),
                path,
                "the artifact's bound contract sha256 matches its source bytes",
            );
            let expected_semantic =
                blake3::hash(&serde_json::to_vec(contract).expect("contract serializes"))
                    .to_hex()
                    .to_string();
            report.compare(
                "artifact-stack-contract-semantic-blake3",
                Value::String(expected_semantic),
                provenance
                    .get("semantic_blake3")
                    .cloned()
                    .unwrap_or(Value::Null),
                path,
                "the artifact binds this exact parsed contract",
            );
        }
        None => {
            let required = contract.artifact.provenance == Some(ArtifactProvenance::Required);
            report.record(
                "artifact-stack-contract-sha256",
                if required {
                    AuditStatus::Mismatch
                } else {
                    AuditStatus::NotRun
                },
                Value::String(if required {
                    "required stack contract provenance".to_owned()
                } else {
                    "a legacy artifact without stack-contract provenance".to_owned()
                }),
                Value::Null,
                path.display().to_string(),
                if required {
                    "the contract requires stack-contract provenance"
                } else {
                    "a legacy artifact has no stack-contract binding"
                },
            )
        }
    }
    match artifact.source.get("tokenizer_sha256") {
        Some(tokenizer) => report.compare(
            "artifact-tokenizer-sha256",
            Value::String(contract.model.tokenizer_sha256.clone()),
            tokenizer.clone(),
            path,
            "the artifact's bound tokenizer matches the contract tokenizer",
        ),
        None => {
            let required = contract.artifact.provenance == Some(ArtifactProvenance::Required);
            report.record(
                "artifact-tokenizer-sha256",
                if required {
                    AuditStatus::Mismatch
                } else {
                    AuditStatus::NotRun
                },
                Value::String(if required {
                    "required tokenizer provenance".to_owned()
                } else {
                    "a legacy artifact without tokenizer provenance".to_owned()
                }),
                Value::Null,
                path.display().to_string(),
                if required {
                    "the contract requires tokenizer provenance"
                } else {
                    "the artifact predates tokenizer provenance"
                },
            )
        }
    }
}

fn audit_tokenizer(
    report: &mut Report,
    tokenizer: &Path,
    _artifact: &Path,
    contract: &StackContract,
) {
    match fs::read(tokenizer) {
        Ok(bytes) => {
            let actual: String = Sha256::digest(&bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect();
            report.compare(
                "tokenizer-sha256",
                Value::String(contract.model.tokenizer_sha256.clone()),
                Value::String(actual),
                tokenizer,
                "tokenizer bytes match the contract identity",
            );
        }
        Err(error) => report.record(
            "tokenizer-sha256",
            AuditStatus::Unavailable,
            Value::String(contract.model.tokenizer_sha256.clone()),
            Value::String(error.to_string()),
            tokenizer.display().to_string(),
            "tokenizer is unreadable",
        ),
    }
}

fn audit_chat(
    report: &mut Report,
    chat: &Path,
    artifact: &Path,
    tokenizer: &Path,
    contract: &StackContract,
) -> Result<()> {
    let scratch = std::env::temp_dir().join(format!(
        "uor-stack-audit-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default()
    ));
    fs::create_dir_all(&scratch)?;
    let script = scratch.join("chat-script.json");
    fs::write(
        &script,
        br#"{"schema":"uor-r4.chat-script/1","turns":[{"user":"Hello"}]}"#,
    )?;
    let command_args = vec![
        "--stack".to_owned(),
        artifact.display().to_string(),
        "--tokenizer".to_owned(),
        tokenizer.display().to_string(),
        "--script".to_owned(),
        script.display().to_string(),
        "--max-tokens".to_owned(),
        "1".to_owned(),
    ];
    let output = Command::new(chat).args(&command_args).output();
    let _ = fs::remove_dir_all(&scratch);
    match output {
        Ok(output) if output.status.success() => {
            let record: Value = match serde_json::from_slice(&output.stdout) {
                Ok(record) => record,
                Err(error) => {
                    report.record(
                        "chat-integer-session",
                        AuditStatus::Mismatch,
                        Value::String("uor-r4.uor-chat-stack-script/1".to_owned()),
                        Value::String(format!("invalid JSON: {error}")),
                        chat.display().to_string(),
                        "the product chat path did not emit its declared record",
                    );
                    return Ok(());
                }
            };
            report.compare(
                "chat-schema",
                Value::String("uor-r4.uor-chat-stack-script/1".to_owned()),
                record.get("schema").cloned().unwrap_or(Value::Null),
                chat,
                "the real chat path emitted the script record",
            );
            report.compare(
                "chat-artifact-sha256",
                Value::String(report.artifact_sha256.clone().unwrap_or_default()),
                record
                    .get("artifact_sha256")
                    .cloned()
                    .unwrap_or(Value::Null),
                chat,
                "chat loaded the audited artifact",
            );
            report.compare(
                "chat-context",
                Value::from(contract.model.context),
                record.get("context").cloned().unwrap_or(Value::Null),
                chat,
                "chat reports the contract context",
            );
            let nonempty = record
                .get("transcript")
                .and_then(Value::as_array)
                .is_some_and(|turns| {
                    turns.len() == 1
                        && turns[0]
                            .get("reply_ids")
                            .and_then(Value::as_array)
                            .is_some_and(|ids| !ids.is_empty())
                });
            report.record(
                "chat-generated-one-token",
                if nonempty {
                    AuditStatus::Match
                } else {
                    AuditStatus::Mismatch
                },
                Value::Bool(true),
                Value::Bool(nonempty),
                chat.display().to_string(),
                "the integer session generated at least one token through the product path",
            );
        }
        Ok(output) => report.record(
            "chat-integer-session",
            if chat.is_file() && artifact.is_file() && tokenizer.is_file() {
                AuditStatus::Mismatch
            } else {
                AuditStatus::Unavailable
            },
            Value::String("exit 0".to_owned()),
            Value::String(String::from_utf8_lossy(&output.stderr).into_owned()),
            chat.display().to_string(),
            "the present product path, artifact and tokenizer did not load together",
        ),
        Err(error) => report.record(
            "chat-integer-session",
            AuditStatus::Unavailable,
            Value::String("executable".to_owned()),
            Value::String(error.to_string()),
            chat.display().to_string(),
            "the chat executable is absent",
        ),
    }
    Ok(())
}

fn audit_probes(
    report: &mut Report,
    directory: &Path,
    contract: &StackContract,
    chat: Option<&Path>,
    artifact: Option<&Path>,
    tokenizer: Option<&Path>,
) -> Result<()> {
    for probe in &contract.probes {
        let Some(fixture) = &probe.fixture else {
            continue;
        };
        let path = directory.join(
            Path::new(fixture)
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new(fixture)),
        );
        match verify_fixture(&path, probe) {
            Ok(fixture) => {
                if let (Some(chat), Some(artifact), Some(tokenizer)) = (chat, artifact, tokenizer) {
                    run_fixture(report, &path, &fixture, probe, chat, artifact, tokenizer)?;
                } else {
                    report.record(
                        format!("probe-fixture:{}", probe.id),
                        AuditStatus::NotRun,
                        Value::String(probe.id.clone()),
                        Value::from(fixture.cases_len()),
                        path.display().to_string(),
                        "fixture is valid but no chat executable, artifact and tokenizer were supplied",
                    );
                }
            }
            Err(error) => report.record(
                format!("probe-fixture:{}", probe.id),
                AuditStatus::Unavailable,
                Value::String(probe.id.clone()),
                Value::String(error.to_string()),
                path.display().to_string(),
                "fixture is absent or malformed",
            ),
        }
    }
    Ok(())
}

#[derive(Debug)]
struct Fixture {
    value: Value,
}

impl Fixture {
    fn cases(&self) -> &[Value] {
        self.value
            .get("cases")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    fn cases_len(&self) -> usize {
        self.cases().len()
    }
}

fn verify_fixture(path: &Path, probe: &Probe) -> Result<Fixture> {
    let fixture: Value = serde_json::from_slice(&fs::read(path)?)?;
    if fixture.get("schema").and_then(Value::as_str) != Some("uor-r4.stack-probe/1") {
        return Err(format!("{}: wrong fixture schema", path.display()).into());
    }
    if fixture.get("probe").and_then(Value::as_str) != Some(probe.id.as_str()) {
        return Err(format!("{}: fixture names another probe", path.display()).into());
    }
    let count = fixture
        .get("cases")
        .and_then(Value::as_array)
        .map(Vec::len)
        .filter(|count| *count > 0)
        .ok_or_else(|| format!("{}: no executable cases", path.display()))?;
    for case in fixture
        .get("cases")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if case.get("id").and_then(Value::as_str).is_none()
            || case.get("script").is_none()
            || case.get("expect").is_none()
        {
            return Err(format!(
                "{}: each of the {count} cases needs id, script and expect",
                path.display()
            )
            .into());
        }
    }
    Ok(Fixture { value: fixture })
}

fn run_fixture(
    report: &mut Report,
    fixture_path: &Path,
    fixture: &Fixture,
    probe: &Probe,
    chat: &Path,
    artifact: &Path,
    tokenizer: &Path,
) -> Result<()> {
    let mut seen = std::collections::BTreeMap::<String, Value>::new();
    for case in fixture.cases() {
        let case_id = case["id"].as_str().expect("validated case id");
        let id = format!("probe:{}:{case_id}", probe.id);
        if !probe.layers.iter().any(|layer| layer.as_str() == "chat") {
            report.record(
                id,
                AuditStatus::Unavailable,
                Value::String("chat".to_owned()),
                Value::String(
                    probe
                        .layers
                        .iter()
                        .map(|layer| layer.as_str())
                        .collect::<Vec<_>>()
                        .join(","),
                ),
                fixture_path.display().to_string(),
                "this runner only executes the chat layer; the contract asks for another layer",
            );
            continue;
        }
        let scratch = std::env::temp_dir().join(format!(
            "uor-stack-probe-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or_default()
        ));
        fs::create_dir_all(&scratch)?;
        let script_path = scratch.join("script.json");
        let script = match case.get("repeat_user") {
            Some(repeat) => {
                let text = repeat
                    .get("text")
                    .and_then(Value::as_str)
                    .ok_or_else(|| format!("{case_id}: repeat_user.text is not a string"))?;
                let count = repeat
                    .get("count")
                    .and_then(Value::as_u64)
                    .filter(|count| (1..=100_000).contains(count))
                    .ok_or_else(|| format!("{case_id}: repeat_user.count is not 1..=100000"))?;
                serde_json::json!({
                    "schema": "uor-r4.chat-script/1",
                    "turns": [{"user": text.repeat(count as usize)}]
                })
            }
            None => case["script"].clone(),
        };
        fs::write(&script_path, serde_json::to_vec(&script)?)?;
        let mut args = vec![
            "--stack".to_owned(),
            artifact.display().to_string(),
            "--tokenizer".to_owned(),
            tokenizer.display().to_string(),
            "--script".to_owned(),
            script_path.display().to_string(),
        ];
        if let Some(extra) = case.get("args") {
            let Some(extra) = extra
                .as_array()
                .filter(|extra| extra.iter().all(Value::is_string))
            else {
                report.record(
                    id,
                    AuditStatus::Mismatch,
                    Value::String("array of strings".to_owned()),
                    extra.clone(),
                    fixture_path.display().to_string(),
                    "case args must contain only strings",
                );
                continue;
            };
            args.extend(extra.iter().filter_map(Value::as_str).map(str::to_owned));
        }
        let output = Command::new(chat).args(&args).output();
        let _ = fs::remove_dir_all(&scratch);
        match output {
            Ok(output) => {
                let expectation = &case["expect"];
                let want_status = expectation
                    .get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("record");
                if want_status == "error" {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let contains = expectation
                        .get("stderr_contains")
                        .and_then(Value::as_str)
                        .is_none_or(|needle| stderr.contains(needle));
                    let ok = !output.status.success() && contains;
                    report.record(
                        id,
                        if ok {
                            AuditStatus::Match
                        } else {
                            AuditStatus::Mismatch
                        },
                        expectation.clone(),
                        Value::String(stderr.into_owned()),
                        fixture_path.display().to_string(),
                        "the product path refused the malformed or over-boundary case as declared",
                    );
                    continue;
                }
                let record: Value = match serde_json::from_slice(&output.stdout) {
                    Ok(record) => record,
                    Err(error) => {
                        report.record(
                            id,
                            AuditStatus::Mismatch,
                            Value::String("a JSON script record".to_owned()),
                            Value::String(error.to_string()),
                            fixture_path.display().to_string(),
                            "the product chat path did not produce a script record",
                        );
                        continue;
                    }
                };
                let mut failures = check_expectation(&record, expectation);
                if let Some(same_as) = expectation.get("same_as") {
                    if let Some(prior) = same_as.as_str() {
                        match seen.get(prior) {
                            Some(earlier) => {
                                let projection = transcript_projection(&record);
                                if &projection != earlier {
                                    failures.push(format!(
                                        "transcript differs from prior case {prior}"
                                    ));
                                }
                            }
                            None => failures.push(format!(
                                "same_as names {prior}, which has no earlier result"
                            )),
                        }
                    } else {
                        failures.push("same_as is not a string".to_owned());
                    }
                }
                seen.insert(case_id.to_owned(), transcript_projection(&record));
                report.record(
                    id,
                    if output.status.success() && failures.is_empty() {
                        AuditStatus::Match
                    } else {
                        AuditStatus::Mismatch
                    },
                    expectation.clone(),
                    record,
                    fixture_path.display().to_string(),
                    if failures.is_empty() {
                        "the product chat path met the declared structural expectation".to_owned()
                    } else {
                        failures.join("; ")
                    },
                );
            }
            Err(error) => report.record(
                id,
                AuditStatus::Unavailable,
                Value::String("executable chat path".to_owned()),
                Value::String(error.to_string()),
                chat.display().to_string(),
                "probe could not start the chat executable",
            ),
        }
    }
    Ok(())
}

fn check_expectation(record: &Value, expectation: &Value) -> Vec<String> {
    let mut failures = Vec::new();
    if let Some(expect) = expectation.as_object() {
        for key in expect.keys() {
            if !EXPECTATION_KEYS.contains(&key.as_str()) {
                failures.push(format!("unknown expectation {key}"));
            }
        }
    } else {
        failures.push("expectation is not an object".to_owned());
        return failures;
    }
    let turns = record
        .get("transcript")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    for key in ["turns", "turn_count", "distinct_replies_min"] {
        if let Some(value) = expectation.get(key) {
            if value.as_u64().is_none() {
                failures.push(format!("{key} is not an unsigned integer"));
            }
        }
    }
    if expectation
        .get("all_reply_ids_nonempty")
        .is_some_and(|value| value.as_bool() != Some(true))
    {
        failures.push("all_reply_ids_nonempty is not true".to_owned());
    }
    let status = expectation.get("status");
    if status.is_some_and(|value| !matches!(value.as_str(), Some("record" | "error"))) {
        failures.push("status is not record or error".to_owned());
    }
    if let Some(stderr) = expectation.get("stderr_contains") {
        if status.and_then(Value::as_str) != Some("error")
            || stderr.as_str().is_none_or(|text| text.trim().is_empty())
        {
            failures.push("stderr_contains is not non-empty error text".to_owned());
        }
    }
    if let Some(required) = expectation.get("turns").and_then(Value::as_u64) {
        if turns.len() as u64 != required {
            failures.push(format!("turns: got {}, expected {required}", turns.len()));
        }
    }
    if let Some(required) = expectation.get("turn_count").and_then(Value::as_u64) {
        if turns.len() as u64 != required {
            failures.push(format!(
                "turn_count: got {}, expected {required}",
                turns.len()
            ));
        }
    }
    if expectation
        .get("all_reply_ids_nonempty")
        .and_then(Value::as_bool)
        == Some(true)
    {
        for (index, turn) in turns.iter().enumerate() {
            if turn
                .get("reply_ids")
                .and_then(Value::as_array)
                .is_none_or(Vec::is_empty)
            {
                failures.push(format!("turn {index} produced no reply token"));
            }
        }
    }
    if let Some(allowed) = expectation.get("stops").and_then(Value::as_array) {
        let allowed: BTreeSet<&str> = allowed.iter().filter_map(Value::as_str).collect();
        for (index, turn) in turns.iter().enumerate() {
            let stop = turn.get("stop").and_then(Value::as_str).unwrap_or("");
            if !allowed.contains(stop) {
                failures.push(format!("turn {index} stop {stop:?} is not allowed"));
            }
        }
    }
    if let Some(memory) = expectation.get("memory").and_then(Value::as_array) {
        let expected: Vec<&str> = memory.iter().filter_map(Value::as_str).collect();
        if expected.len() != turns.len() {
            failures.push(format!(
                "memory expectation has {} rows for {} turns",
                expected.len(),
                turns.len()
            ));
        } else {
            for (index, (turn, expected)) in turns.iter().zip(expected).enumerate() {
                let actual = turn.get("memory").and_then(Value::as_str).unwrap_or("");
                if actual != expected {
                    failures.push(format!(
                        "turn {index} memory {actual:?}, expected {expected:?}"
                    ));
                }
            }
        }
    }
    if expectation.get("memory").is_some()
        && expectation
            .get("memory")
            .and_then(Value::as_array)
            .is_none()
    {
        failures.push("memory is not an array of strings".to_owned());
    }
    if expectation.get("stops").is_some()
        && expectation.get("stops").and_then(Value::as_array).is_none()
    {
        failures.push("stops is not an array of strings".to_owned());
    }
    if let Some(patterns) = expectation
        .get("reply_text_contains")
        .and_then(Value::as_array)
    {
        if patterns.len() != turns.len() {
            failures.push(format!(
                "reply_text_contains has {} rows for {} turns",
                patterns.len(),
                turns.len()
            ));
        } else {
            for (index, (turn, pattern)) in turns.iter().zip(patterns).enumerate() {
                let pattern = pattern.as_str().unwrap_or("");
                if !pattern.is_empty()
                    && !turn
                        .get("reply_text")
                        .and_then(Value::as_str)
                        .is_some_and(|reply| reply.contains(pattern))
                {
                    failures.push(format!("turn {index} reply does not contain {pattern:?}"));
                }
            }
        }
    }
    if expectation.get("reply_text_contains").is_some()
        && expectation
            .get("reply_text_contains")
            .and_then(Value::as_array)
            .is_none()
    {
        failures.push("reply_text_contains is not an array of strings".to_owned());
    }
    if let Some(minimum) = expectation
        .get("distinct_replies_min")
        .and_then(Value::as_u64)
    {
        let distinct: BTreeSet<&str> = turns
            .iter()
            .filter_map(|turn| turn.get("reply_text").and_then(Value::as_str))
            .collect();
        if distinct.len() < minimum as usize {
            failures.push(format!(
                "distinct replies: got {}, expected at least {minimum}",
                distinct.len()
            ));
        }
    }
    failures
}

fn transcript_projection(record: &Value) -> Value {
    Value::Array(
        record
            .get("transcript")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(|turn| {
                serde_json::json!({
                    "user": turn.get("user").cloned().unwrap_or(Value::Null),
                    "memory": turn.get("memory").cloned().unwrap_or(Value::Null),
                    "memory_text": turn.get("memory_text").cloned().unwrap_or(Value::Null),
                    "reply_text": turn.get("reply_text").cloned().unwrap_or(Value::Null),
                    "reply_ids": turn.get("reply_ids").cloned().unwrap_or(Value::Null),
                    "stop": turn.get("stop").cloned().unwrap_or(Value::Null),
                    "history_tokens": turn.get("history_tokens").cloned().unwrap_or(Value::Null),
                })
            })
            .collect(),
    )
}

fn read_json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn hex_string(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_never_promotes_unavailable_or_empty_to_match() {
        let options = Options {
            contract: PathBuf::new(),
            artifact: None,
            tokenizer: None,
            config: None,
            checkpoint: None,
            chat_bin: None,
            probes: None,
            out: None,
        };
        let contract: StackContract = toml::from_str(include_str!(
            "../../conformance/stack/fbs-b64l-baseline.toml"
        ))
        .expect("contract parses");
        let mut report = Report::new(&options, &contract);
        report.finish();
        assert_eq!(report.overall, "NOT_RUN");
        report.record(
            "missing",
            AuditStatus::Unavailable,
            Value::Bool(true),
            Value::Null,
            "test",
            "missing",
        );
        report.finish();
        assert_eq!(report.overall, "UNAVAILABLE");
        report.record(
            "wrong",
            AuditStatus::Mismatch,
            Value::Bool(true),
            Value::Bool(false),
            "test",
            "wrong",
        );
        report.finish();
        assert_eq!(report.overall, "MISMATCH");
    }

    #[test]
    fn probe_expectations_check_turns_memory_stops_and_reply_ids() {
        let record = serde_json::json!({
            "transcript": [
                {
                    "reply_ids": [7],
                    "stop": "eos",
                    "memory": "none",
                    "reply_text": "Hello."
                },
                {
                    "reply_ids": [8, 9],
                    "stop": "max_new_tokens",
                    "memory": "explicit",
                    "reply_text": "It's 47."
                }
            ]
        });
        let expectation = serde_json::json!({
            "turns": 2,
            "all_reply_ids_nonempty": true,
            "stops": ["eos", "max_new_tokens", "short_cycle"],
            "memory": ["none", "explicit"],
            "distinct_replies_min": 2
        });
        assert!(check_expectation(&record, &expectation).is_empty());

        let wrong = serde_json::json!({
            "turns": 1,
            "stops": ["eos"],
            "memory": ["sieve", "none"],
            "distinct_replies_min": 3
        });
        let failures = check_expectation(&record, &wrong);
        assert!(failures.iter().any(|failure| failure.contains("turns")));
        assert!(failures
            .iter()
            .any(|failure| failure.contains("max_new_tokens")));
        assert!(failures.iter().any(|failure| failure.contains("memory")));
        assert!(failures
            .iter()
            .any(|failure| failure.contains("distinct replies")));

        let unknown = check_expectation(&record, &serde_json::json!({"magic": true}));
        assert!(unknown.iter().any(|failure| failure.contains("unknown")));
    }

    #[test]
    fn a_present_truncated_artifact_is_a_mismatch() {
        let path =
            std::env::temp_dir().join(format!("uor-stack-audit-bad-{}.uorlut", std::process::id()));
        fs::write(&path, b"UORLUT01").expect("bad artifact fixture");
        assert!(parse_artifact(&path).is_err());
        let contract: StackContract =
            toml::from_str(include_str!("../../conformance/stack/fbs-arm-r.toml"))
                .expect("contract parses");
        let options = Options {
            contract: PathBuf::new(),
            artifact: None,
            tokenizer: None,
            config: None,
            checkpoint: None,
            chat_bin: None,
            probes: None,
            out: None,
        };
        let mut report = Report::new(&options, &contract);
        audit_artifact(&mut report, &path, Path::new("contract.toml"), &contract);
        report.finish();
        assert_eq!(report.overall, "MISMATCH");
        fs::remove_file(path).expect("bad artifact cleanup");
    }

    #[test]
    fn an_omitted_run_context_uses_the_trainers_192_default() {
        let contract: StackContract =
            toml::from_str(include_str!("../../conformance/stack/fbs-arm-r.toml"))
                .expect("contract parses");
        let path = std::env::temp_dir().join(format!(
            "uor-stack-audit-config-{}.json",
            std::process::id()
        ));
        fs::write(&path, br#"{"objective":"cell","width":256}"#).expect("config fixture");
        let options = Options {
            contract: PathBuf::new(),
            artifact: None,
            tokenizer: None,
            config: None,
            checkpoint: None,
            chat_bin: None,
            probes: None,
            out: None,
        };
        let mut report = Report::new(&options, &contract);
        audit_config(&mut report, &path, &contract);
        let context = report
            .checks
            .iter()
            .find(|check| check.id == "run-config-context")
            .expect("context check");
        assert_eq!(context.status, AuditStatus::Match);
        assert_eq!(context.actual, Value::from(192));
        fs::remove_file(path).expect("config cleanup");
    }
}

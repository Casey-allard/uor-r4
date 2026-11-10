//! Typed stack contracts: one machine-readable description of a model composition.
//!
//! A stack contract does not prove that a model is capable. It makes the
//! composition explicit across source, configuration, artifact metadata,
//! commands, readings and probes so context resets cannot silently reconstruct
//! a different model.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::ModelError;

/// The stack-contract schema this crate reads.
pub const SPEC: &str = "uor-r4.stack-contract/1";

/// The directory that holds stack-contract TOML files.
pub const DIRECTORY: &str = "conformance/stack";

/// The model lane a contract describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Lane {
    /// The float-trained FBS path exported to an integer stack.
    Fbs,
    /// The integer-from-the-start discrete M1 path.
    Discrete,
    /// A composition that joins lanes through one served artifact.
    Joint,
}

impl Lane {
    /// The kebab-case spelling used in TOML and generated documents.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fbs => "fbs",
            Self::Discrete => "discrete",
            Self::Joint => "joint",
        }
    }
}

/// The role a contract has in the repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContractState {
    /// A specification or design with no current run attached.
    Reference,
    /// A runnable configuration under active investigation.
    Experimental,
    /// A configuration supported by a measured result but not yet accepted.
    Candidate,
    /// The accepted current model for its lane.
    Current,
    /// A preserved baseline or retired configuration.
    Historical,
}

impl ContractState {
    /// The kebab-case spelling used in TOML and generated documents.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Reference => "reference",
            Self::Experimental => "experimental",
            Self::Candidate => "candidate",
            Self::Current => "current",
            Self::Historical => "historical",
        }
    }
}

/// The evidence state of one mechanism in a contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MechanismStatus {
    /// Source exists but is not on this contract's executable path.
    Implemented,
    /// Source is assembled into this contract's executable path.
    Integrated,
    /// A recorded result supports this mechanism under the cited evidence.
    MeasuredPositive,
    /// A recorded result rejects the cited configuration, not necessarily the mechanism.
    MeasuredNegative,
    /// The instrument was invalid, so the result does not answer the mechanism question.
    InstrumentInvalid,
    /// Source exists but no relevant result has been recorded.
    NotRun,
}

impl MechanismStatus {
    /// The kebab-case spelling used in TOML and generated documents.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Implemented => "implemented",
            Self::Integrated => "integrated",
            Self::MeasuredPositive => "measured-positive",
            Self::MeasuredNegative => "measured-negative",
            Self::InstrumentInvalid => "instrument-invalid",
            Self::NotRun => "not-run",
        }
    }
}

/// The kind of evidence a reading records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReadingClass {
    /// Artifact, configuration or trace agreement.
    Structural,
    /// The asked entity, relation, value or abstention is correct.
    Semantic,
    /// Wording, grammar, EOS and surface realization.
    Surface,
    /// Agreement between trainer, exporter, CPU, integer and chat paths.
    Parity,
    /// Tokens per second, latency or resource use.
    Performance,
}

impl ReadingClass {
    /// The kebab-case spelling used in TOML and generated documents.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Structural => "structural",
            Self::Semantic => "semantic",
            Self::Surface => "surface",
            Self::Parity => "parity",
            Self::Performance => "performance",
        }
    }
}

/// The construction rule behind a probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProbeClass {
    /// One side of a declared threshold, the threshold itself, and the other side.
    Boundary,
    /// One cause changes while irrelevant causes remain fixed.
    Metamorphic,
    /// The same probe runs through two execution layers.
    Parity,
    /// A malformed request must be refused.
    Malformed,
    /// A missing or insufficient source must produce the declared abstention.
    Abstention,
}

impl ProbeClass {
    /// The kebab-case spelling used in TOML and generated documents.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Boundary => "boundary",
            Self::Metamorphic => "metamorphic",
            Self::Parity => "parity",
            Self::Malformed => "malformed",
            Self::Abstention => "abstention",
        }
    }
}

/// What a command does in the contract's lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommandRole {
    /// Produce a checkpoint.
    Train,
    /// Score a checkpoint or panel.
    Score,
    /// Convert a checkpoint into a served artifact.
    Export,
    /// Run the artifact through a serving or chat path.
    Serve,
    /// Verify structural invariants and generated files.
    Verify,
}

impl CommandRole {
    /// The kebab-case spelling used in TOML and generated documents.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Train => "train",
            Self::Score => "score",
            Self::Export => "export",
            Self::Serve => "serve",
            Self::Verify => "verify",
        }
    }
}

/// One stack contract.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StackContract {
    /// The schema tag, exactly [`SPEC`].
    pub spec: String,
    /// A stable kebab-case contract identifier.
    pub id: String,
    /// A short human-readable title.
    pub title: String,
    /// The model lane.
    pub lane: Lane,
    /// The role of this contract in the repository.
    pub state: ContractState,
    /// The source commit or object this contract describes.
    pub source_pin: String,
    /// A one-paragraph statement of what the contract describes.
    pub summary: String,
    /// The model's fixed shape and tokenizer identity.
    pub model: ModelShape,
    /// The served artifact's logical family and required sections.
    pub artifact: Artifact,
    /// Data and panel inputs.
    #[serde(default)]
    pub inputs: Vec<Input>,
    /// Mechanisms assembled into this contract.
    pub mechanisms: Vec<Mechanism>,
    /// Commands that reproduce the lifecycle.
    pub commands: Vec<Command>,
    /// Readings the contract distinguishes.
    pub readings: Vec<Reading>,
    /// Boundary, metamorphic, parity, malformed and abstention probes.
    pub probes: Vec<Probe>,
    /// Statements this contract explicitly does not make.
    pub non_claims: Vec<NonClaim>,
}

/// Fixed model shape and tokenizer identity.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelShape {
    /// SHA-256 of `tokenizer.json`.
    pub tokenizer_sha256: String,
    /// Vocabulary size.
    pub vocab: usize,
    /// Hidden width.
    pub width: usize,
    /// Layer count.
    pub layers: usize,
    /// Trained context length.
    pub context: usize,
    /// Training objective name.
    pub objective: String,
}

/// A served artifact family and its logical sections.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    /// The artifact family, without claiming one exact schema revision.
    pub family: String,
    /// Logical sections the exporter and session must agree on.
    pub sections: Vec<ArtifactSection>,
}

/// One logical artifact section.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactSection {
    /// A stable section identifier.
    pub id: String,
    /// Whether this contract requires the section.
    pub required: bool,
    /// What the section carries.
    pub description: String,
}

/// One data or panel input.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// A stable input identifier.
    pub id: String,
    /// A repository- or store-relative location.
    pub source: String,
    /// SHA-256 of the input bytes.
    pub sha256: String,
}

/// One mechanism in the assembly.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Mechanism {
    /// A stable mechanism identifier.
    pub id: String,
    /// Its evidence state in this contract.
    pub status: MechanismStatus,
    /// The primary implementation path, optionally followed by `#symbol`.
    pub implementation: String,
    /// Evidence paths or records that support the stated status.
    pub evidence: Vec<String>,
    /// Artifact section identifiers this mechanism requires.
    #[serde(default)]
    pub sections: Vec<String>,
    /// Properties this mechanism must preserve.
    pub invariants: Vec<String>,
    /// Statements this mechanism does not establish.
    #[serde(default)]
    pub non_claims: Vec<String>,
}

/// One reproduction command.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    /// A stable command identifier.
    pub id: String,
    /// Where the command sits in the lifecycle.
    pub role: CommandRole,
    /// The executable or Cargo subcommand.
    pub program: String,
    /// Arguments after the program.
    pub args: Vec<String>,
}

/// One reading the contract distinguishes.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Reading {
    /// A stable reading identifier.
    pub id: String,
    /// The evidence class.
    pub class: ReadingClass,
    /// What the reading records.
    pub description: String,
    /// The mechanism this reading judges, when it judges one directly.
    #[serde(default)]
    pub mechanism: Option<String>,
}

/// One planned probe.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Probe {
    /// A stable probe identifier.
    pub id: String,
    /// The probe's construction rule.
    pub class: ProbeClass,
    /// The mechanism this probe exercises.
    pub mechanism: String,
    /// The one axis the probe changes or checks.
    pub axis: String,
    /// The expected result rule.
    pub expectation: String,
    /// Concrete cases generated from the rule.
    pub cases: Vec<String>,
}

/// One explicit non-claim.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NonClaim {
    /// A stable non-claim identifier.
    pub id: String,
    /// What the contract does not establish.
    pub statement: String,
}

/// A parsed contract and the file it came from.
#[derive(Debug, Clone)]
pub struct StackContractFile {
    /// The source TOML file.
    pub path: PathBuf,
    /// The parsed contract.
    pub contract: StackContract,
}

impl StackContract {
    /// Validate this contract against itself and the repository it names.
    pub fn check(&self, root: &Path) -> Result<(), ModelError> {
        let bad = |message: String| ModelError::Inconsistent(message);
        if self.spec != SPEC {
            return Err(bad(format!(
                "{}: spec is {:?}, expected {SPEC:?}",
                self.id, self.spec
            )));
        }
        check_slug("contract", &self.id)?;
        check_text("title", &self.title)?;
        check_text("summary", &self.summary)?;
        check_commit("source_pin", &self.source_pin)?;
        check_sha256("model.tokenizer_sha256", &self.model.tokenizer_sha256)?;
        if self.model.vocab == 0
            || self.model.width == 0
            || self.model.layers == 0
            || self.model.context == 0
        {
            return Err(bad(format!(
                "{}: model vocab, width, layers and context must all be non-zero",
                self.id
            )));
        }
        check_text("model.objective", &self.model.objective)?;
        check_text("artifact.family", &self.artifact.family)?;
        if self.artifact.sections.is_empty() {
            return Err(bad(format!("{}: artifact has no section", self.id)));
        }

        let sections = unique_ids("artifact section", &self.artifact.sections, |s| &s.id)?;
        for section in &self.artifact.sections {
            check_slug("artifact section", &section.id)?;
            check_text("artifact section description", &section.description)?;
        }

        if self.inputs.is_empty() {
            return Err(bad(format!("{}: contract has no input", self.id)));
        }
        unique_ids("input", &self.inputs, |i| &i.id)?;
        for input in &self.inputs {
            check_slug("input", &input.id)?;
            check_text("input source", &input.source)?;
            check_sha256("input.sha256", &input.sha256)?;
        }

        if self.mechanisms.is_empty() {
            return Err(bad(format!("{}: contract has no mechanism", self.id)));
        }
        unique_ids("mechanism", &self.mechanisms, |m| &m.id)?;
        for mechanism in &self.mechanisms {
            check_slug("mechanism", &mechanism.id)?;
            check_repo_path(root, "mechanism.implementation", &mechanism.implementation)?;
            if mechanism.evidence.is_empty() {
                return Err(bad(format!(
                    "{}: mechanism {} has no evidence path",
                    self.id, mechanism.id
                )));
            }
            for evidence in &mechanism.evidence {
                check_repo_path(root, "mechanism.evidence", evidence)?;
            }
            if mechanism.invariants.is_empty() {
                return Err(bad(format!(
                    "{}: mechanism {} has no invariant",
                    self.id, mechanism.id
                )));
            }
            for invariant in &mechanism.invariants {
                check_text("mechanism invariant", invariant)?;
            }
            for section in &mechanism.sections {
                if !sections.contains(section.as_str()) {
                    return Err(bad(format!(
                        "{}: mechanism {} names unknown artifact section {section}",
                        self.id, mechanism.id
                    )));
                }
            }
        }

        if self.commands.is_empty() {
            return Err(bad(format!("{}: contract has no command", self.id)));
        }
        unique_ids("command", &self.commands, |c| &c.id)?;
        for command in &self.commands {
            check_slug("command", &command.id)?;
            check_text("command.program", &command.program)?;
            if command.args.is_empty() {
                return Err(bad(format!(
                    "{}: command {} has no arguments",
                    self.id, command.id
                )));
            }
        }

        if self.readings.is_empty() {
            return Err(bad(format!("{}: contract has no reading", self.id)));
        }
        let mechanism_ids: BTreeSet<&str> = self.mechanisms.iter().map(|m| m.id.as_str()).collect();
        unique_ids("reading", &self.readings, |r| &r.id)?;
        for reading in &self.readings {
            check_slug("reading", &reading.id)?;
            check_text("reading.description", &reading.description)?;
            if let Some(mechanism) = &reading.mechanism {
                if !mechanism_ids.contains(mechanism.as_str()) {
                    return Err(bad(format!(
                        "{}: reading {} names unknown mechanism {mechanism}",
                        self.id, reading.id
                    )));
                }
            }
        }

        if self.probes.is_empty() {
            return Err(bad(format!("{}: contract has no probe", self.id)));
        }
        unique_ids("probe", &self.probes, |p| &p.id)?;
        for probe in &self.probes {
            check_slug("probe", &probe.id)?;
            check_text("probe.axis", &probe.axis)?;
            check_text("probe.expectation", &probe.expectation)?;
            if !mechanism_ids.contains(probe.mechanism.as_str()) {
                return Err(bad(format!(
                    "{}: probe {} names unknown mechanism {}",
                    self.id, probe.id, probe.mechanism
                )));
            }
            if probe.cases.is_empty() {
                return Err(bad(format!("{}: probe {} has no case", self.id, probe.id)));
            }
        }

        if self.non_claims.is_empty() {
            return Err(bad(format!("{}: contract has no non-claim", self.id)));
        }
        unique_ids("non-claim", &self.non_claims, |n| &n.id)?;
        for non_claim in &self.non_claims {
            check_slug("non-claim", &non_claim.id)?;
            check_text("non-claim.statement", &non_claim.statement)?;
        }
        Ok(())
    }
}

/// Load every `*.toml` contract in `dir`, sorted by file name.
pub fn load_dir(dir: &Path) -> Result<Vec<StackContractFile>, ModelError> {
    let entries = fs::read_dir(dir).map_err(|error| ModelError::Io(dir.to_path_buf(), error))?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| ModelError::Io(dir.to_path_buf(), error))?;
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "toml") {
            paths.push(path);
        }
    }
    paths.sort();
    if paths.is_empty() {
        return Err(ModelError::Inconsistent(format!(
            "{}: no stack contracts",
            dir.display()
        )));
    }

    let mut files = Vec::with_capacity(paths.len());
    for path in paths {
        let text =
            fs::read_to_string(&path).map_err(|error| ModelError::Io(path.clone(), error))?;
        let contract =
            toml::from_str(&text).map_err(|error| ModelError::Parse(path.clone(), error))?;
        files.push(StackContractFile { path, contract });
    }
    Ok(files)
}

/// Validate every contract and reject duplicate contract identifiers.
pub fn check_all(files: &[StackContractFile], root: &Path) -> Result<(), ModelError> {
    unique_ids("contract", files, |f| &f.contract.id)?;
    for file in files {
        file.contract.check(root)?;
    }
    Ok(())
}

fn unique_ids<'a, T, F>(kind: &str, rows: &'a [T], id: F) -> Result<BTreeSet<&'a str>, ModelError>
where
    F: Fn(&'a T) -> &'a String,
{
    let mut seen = BTreeSet::new();
    for row in rows {
        let value = id(row).as_str();
        if !seen.insert(value) {
            return Err(ModelError::Inconsistent(format!(
                "{kind} {value:?} is registered twice"
            )));
        }
    }
    Ok(seen)
}

fn check_text(name: &str, value: &str) -> Result<(), ModelError> {
    if value.trim().is_empty() {
        return Err(ModelError::Inconsistent(format!("{name} is empty")));
    }
    Ok(())
}

fn check_slug(name: &str, value: &str) -> Result<(), ModelError> {
    check_text(name, value)?;
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        || value.starts_with('-')
        || value.ends_with('-')
        || value.contains("--")
    {
        return Err(ModelError::Inconsistent(format!(
            "{name} {value:?} is not lower kebab-case"
        )));
    }
    Ok(())
}

fn check_sha256(name: &str, value: &str) -> Result<(), ModelError> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ModelError::Inconsistent(format!(
            "{name} must be 64 hexadecimal characters"
        )));
    }
    Ok(())
}

fn check_commit(name: &str, value: &str) -> Result<(), ModelError> {
    if !matches!(value.len(), 40 | 64) || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ModelError::Inconsistent(format!(
            "{name} must be a 40- or 64-character hexadecimal commit"
        )));
    }
    Ok(())
}

fn check_repo_path(root: &Path, name: &str, value: &str) -> Result<(), ModelError> {
    check_text(name, value)?;
    let path = value.split('#').next().unwrap_or(value);
    let relative = Path::new(path);
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(ModelError::Inconsistent(format!(
            "{name} {value:?} must be a relative path inside {}",
            root.display()
        )));
    }
    let canonical_root = root.canonicalize().map_err(|error| {
        ModelError::Inconsistent(format!(
            "{name} cannot canonicalize repository root {}: {error}",
            root.display()
        ))
    })?;
    let candidate = root.join(relative).canonicalize().map_err(|error| {
        ModelError::Inconsistent(format!(
            "{name} {value:?} does not resolve under {}: {error}",
            root.display()
        ))
    })?;
    if !candidate.starts_with(&canonical_root) {
        return Err(ModelError::Inconsistent(format!(
            "{name} {value:?} resolves outside {}",
            root.display()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"
spec = "uor-r4.stack-contract/1"
id = "test-stack"
title = "Test stack"
lane = "fbs"
state = "reference"
source_pin = "0000000000000000000000000000000000000000"
summary = "A test contract."

[model]
tokenizer_sha256 = "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89"
vocab = 4096
width = 256
layers = 6
context = 192
objective = "nearest"

[artifact]
family = "uor-r4.lut-stack"

[[artifact.sections]]
id = "trunk"
required = true
description = "The FBS trunk."

[[inputs]]
id = "train"
source = "store:train"
sha256 = "0000000000000000000000000000000000000000000000000000000000000000"

[[mechanisms]]
id = "trunk"
status = "integrated"
implementation = "crates/uor-r4-float-train/src/geometric_stack_native.rs"
evidence = ["crates/uor-r4-float-train/src/geometric_stack_native.rs"]
sections = ["trunk"]
invariants = ["The trunk writes one width-256 state per position."]

[[commands]]
id = "verify"
role = "verify"
program = "cargo"
args = ["check", "-p", "repo-model"]

[[readings]]
id = "top-one"
class = "semantic"
description = "The nearest token is counted."
mechanism = "trunk"

[[probes]]
id = "context-boundary"
class = "boundary"
mechanism = "trunk"
axis = "context"
expectation = "191, 192 and 193 are distinguished."
cases = ["191", "192", "193"]

[[non_claims]]
id = "not-proof"
statement = "This contract is not a proof of language capability."
"#;

    fn parsed() -> StackContract {
        toml::from_str(VALID).expect("valid contract")
    }

    fn root() -> PathBuf {
        crate::repo_root()
    }

    #[test]
    fn a_valid_contract_checks() {
        parsed().check(&root()).expect("contract checks");
    }

    #[test]
    fn duplicate_mechanisms_are_rejected() {
        let mut contract = parsed();
        contract.mechanisms.push(contract.mechanisms[0].clone());
        let error = contract.check(&root()).unwrap_err().to_string();
        assert!(error.contains("registered twice"), "{error}");
    }

    #[test]
    fn unknown_artifact_sections_are_rejected() {
        let mut contract = parsed();
        contract.mechanisms[0].sections = vec!["missing".to_owned()];
        let error = contract.check(&root()).unwrap_err().to_string();
        assert!(error.contains("unknown artifact section"), "{error}");
    }

    #[test]
    fn evidence_must_resolve_in_the_repository() {
        let mut contract = parsed();
        contract.mechanisms[0].evidence = vec!["no/such/file".to_owned()];
        let error = contract.check(&root()).unwrap_err().to_string();
        assert!(error.contains("does not resolve"), "{error}");
    }

    #[test]
    fn absolute_and_parent_paths_cannot_escape_the_repository() {
        let mut absolute = parsed();
        absolute.mechanisms[0].evidence = vec!["/etc/passwd".to_owned()];
        let error = absolute.check(&root()).unwrap_err().to_string();
        assert!(error.contains("relative path inside"), "{error}");

        let mut parent = parsed();
        parent.mechanisms[0].evidence = vec!["../outside".to_owned()];
        let error = parent.check(&root()).unwrap_err().to_string();
        assert!(error.contains("relative path inside"), "{error}");
    }

    #[test]
    fn required_contract_collections_cannot_be_empty() {
        for clear in [
            |c: &mut StackContract| c.artifact.sections.clear(),
            |c: &mut StackContract| c.inputs.clear(),
            |c: &mut StackContract| c.mechanisms.clear(),
            |c: &mut StackContract| c.commands.clear(),
            |c: &mut StackContract| c.readings.clear(),
            |c: &mut StackContract| c.probes.clear(),
            |c: &mut StackContract| c.non_claims.clear(),
        ] {
            let mut contract = parsed();
            clear(&mut contract);
            assert!(contract.check(&root()).is_err());
        }
    }

    #[test]
    fn readings_and_probes_must_name_a_mechanism() {
        let mut reading = parsed();
        reading.readings[0].mechanism = Some("missing".to_owned());
        let error = reading.check(&root()).unwrap_err().to_string();
        assert!(error.contains("unknown mechanism"), "{error}");

        let mut probe = parsed();
        probe.probes[0].mechanism = "missing".to_owned();
        let error = probe.check(&root()).unwrap_err().to_string();
        assert!(error.contains("unknown mechanism"), "{error}");
    }

    #[test]
    fn identifiers_are_lower_kebab_case() {
        let mut contract = parsed();
        contract.id = "Bad_ID".to_owned();
        let error = contract.check(&root()).unwrap_err().to_string();
        assert!(error.contains("lower kebab-case"), "{error}");
    }
}

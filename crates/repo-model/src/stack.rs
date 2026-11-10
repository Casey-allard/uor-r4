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
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::ModelError;

/// The stack-contract schema this crate writes for new contracts.
pub const SPEC: &str = "uor-r4.stack-contract/2";

/// Historical stack-contract schemas this crate still reads.
pub const LEGACY_SPECS: [&str; 1] = ["uor-r4.stack-contract/1"];

/// The directory that holds stack-contract TOML files.
pub const DIRECTORY: &str = "conformance/stack";

/// The executable probe-fixture schema named by v2 contracts.
pub const FIXTURE_SCHEMA: &str = "uor-r4.stack-probe/1";

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

/// One execution layer that must run a behavioral probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionLayer {
    /// The float trainer's scoring path.
    FloatScorer,
    /// A loaded checkpoint scored without exporting.
    Checkpoint,
    /// The independent CPU forward/audit path.
    Cpu,
    /// The compiled integer stack session.
    Integer,
    /// The product chat command.
    Chat,
}

impl ExecutionLayer {
    /// The kebab-case spelling used in TOML and generated documents.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FloatScorer => "float-scorer",
            Self::Checkpoint => "checkpoint",
            Self::Cpu => "cpu",
            Self::Integer => "integer",
            Self::Chat => "chat",
        }
    }
}

/// The kind of record supporting a claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceKind {
    /// A host or CPU test.
    HostTest,
    /// A GPU-qualified test or run.
    GpuRun,
    /// A sealed model or provenance artifact.
    Artifact,
    /// A served transcript.
    Transcript,
    /// A report or review record.
    Report,
    /// A formal proof or proof build.
    Proof,
    /// A cited external or owner authority.
    Authority,
}

impl EvidenceKind {
    /// The kebab-case spelling used in generated documents.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HostTest => "host-test",
            Self::GpuRun => "gpu-run",
            Self::Artifact => "artifact",
            Self::Transcript => "transcript",
            Self::Report => "report",
            Self::Proof => "proof",
            Self::Authority => "authority",
        }
    }
}

/// What an evidence record established.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceResult {
    /// The evidence supports the named claim.
    Pass,
    /// The evidence rejects the named configuration.
    Fail,
    /// The required run did not happen.
    NotRun,
    /// The input or environment needed to judge was absent.
    Unavailable,
    /// The instrument was invalid, so the result answers no mechanism question.
    InstrumentInvalid,
}

impl EvidenceResult {
    /// The kebab-case spelling used in TOML and generated documents.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::NotRun => "not-run",
            Self::Unavailable => "unavailable",
            Self::InstrumentInvalid => "instrument-invalid",
        }
    }
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
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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
    /// The contract's own monotonic revision.
    #[serde(default = "default_revision")]
    pub revision: u32,
    /// The owner or review decision that made a candidate current.
    #[serde(default)]
    pub decision: Option<String>,
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
    /// Evidence records referenced by mechanisms and artifact runs.
    #[serde(default)]
    pub evidence: Vec<Evidence>,
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
    #[serde(default)]
    pub training_context: Option<usize>,
    /// Served context length.
    pub context: usize,
    /// Training objective name.
    pub objective: String,
    /// Training domain and precision.
    #[serde(default)]
    pub training: Option<String>,
    /// Serving domain and runtime.
    #[serde(default)]
    pub serving: Option<String>,
}

/// A served artifact family and its logical sections.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    /// The artifact family, without claiming one exact schema revision.
    pub family: String,
    /// The exact exported schema revision.
    #[serde(default)]
    pub schema: Option<String>,
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
    /// The mechanism that owns this section.
    #[serde(default)]
    pub owner: Option<String>,
    /// The section's element or tensor format.
    #[serde(default)]
    pub format: Option<String>,
    /// Fixed-point exponent, when applicable.
    #[serde(default)]
    pub exponent: Option<i32>,
    /// Why an optional section may be absent.
    #[serde(default)]
    pub optional_reason: Option<String>,
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
    /// Structured evidence records supporting this mechanism.
    #[serde(default)]
    pub evidence_ids: Vec<String>,
    /// Exact command-line flags that enable this mechanism.
    #[serde(default)]
    pub flags: Vec<FlagSetting>,
    /// Mechanisms that must be absent when this mechanism is enabled.
    #[serde(default)]
    pub conflicts_with: Vec<String>,
    /// Mechanisms that must be present when this mechanism is enabled.
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Checkpoint sidecars this mechanism requires.
    #[serde(default)]
    pub sidecars: Vec<String>,
    /// The mechanism's load position in the thread-local setup order.
    #[serde(default)]
    pub load_order: Option<u32>,
    /// The mechanism's cleanup position.
    #[serde(default)]
    pub cleanup_order: Option<u32>,
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
    /// Required environment variable names.
    #[serde(default)]
    pub env: Vec<String>,
    /// Inputs consumed by this command.
    #[serde(default)]
    pub inputs: Vec<String>,
    /// Outputs produced by this command.
    #[serde(default)]
    pub outputs: Vec<String>,
    /// Maximum expected runtime.
    #[serde(default)]
    pub timeout_seconds: Option<u64>,
}

/// One reading the contract distinguishes.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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
    /// The reading's unit.
    #[serde(default)]
    pub unit: Option<String>,
    /// Whether larger or smaller is better.
    #[serde(default)]
    pub better: Option<ReadingDirection>,
    /// Registered acceptance bar, when one exists.
    #[serde(default)]
    pub bar: Option<f64>,
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
    /// Structured fixture carrying executable cases.
    #[serde(default)]
    pub fixture: Option<String>,
    /// SHA-256 of the fixture.
    #[serde(default)]
    pub fixture_sha256: Option<String>,
    /// Execution layers that must produce the expected result.
    #[serde(default)]
    pub layers: Vec<ExecutionLayer>,
}

/// One exact mechanism flag.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FlagSetting {
    /// The flag name, including leading dashes.
    pub name: String,
    /// The default value serialized as text.
    pub value: String,
    /// Whether the contract requires the flag to be set.
    pub required: bool,
}

/// The direction in which a reading improves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReadingDirection {
    /// Larger values are better.
    Higher,
    /// Smaller values are better.
    Lower,
}

impl ReadingDirection {
    /// The lowercase spelling used in generated documents.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Higher => "higher",
            Self::Lower => "lower",
        }
    }
}

/// One evidence record.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    /// Stable evidence identifier.
    pub id: String,
    /// The evidence kind.
    pub kind: EvidenceKind,
    /// A repository path, store path, URL or receipt locator.
    pub locator: String,
    /// SHA-256 when the record is a file or immutable object.
    #[serde(default)]
    pub sha256: Option<String>,
    /// What the evidence established.
    pub result: EvidenceResult,
    /// The evidence's scope.
    pub scope: String,
    /// Mechanisms covered by this evidence.
    #[serde(default)]
    pub mechanisms: Vec<String>,
    /// What the evidence supports or rejects.
    pub statement: String,
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
        let v2 = self.spec == SPEC;
        if !v2 && !LEGACY_SPECS.contains(&self.spec.as_str()) {
            return Err(bad(format!(
                "{}: unsupported stack-contract spec {:?}",
                self.id, self.spec,
            )));
        }
        check_slug("contract", &self.id)?;
        check_text("title", &self.title)?;
        check_text("summary", &self.summary)?;
        check_commit("source_pin", &self.source_pin)?;
        if self.revision == 0 {
            return Err(bad(format!("{}: revision must be positive", self.id)));
        }
        if v2
            && self.state == ContractState::Current
            && self.decision.as_deref().is_none_or(str::is_empty)
        {
            return Err(bad(format!(
                "{}: a current contract names the decision or review that accepted it",
                self.id
            )));
        }
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
        if v2
            && self
                .model
                .training_context
                .is_none_or(|context| context == 0)
        {
            return Err(bad(format!(
                "{}: model.training_context is required and non-zero",
                self.id
            )));
        }
        check_text("model.objective", &self.model.objective)?;
        if v2 {
            check_required_text("model.training", self.model.training.as_deref())?;
            check_required_text("model.serving", self.model.serving.as_deref())?;
        }
        check_text("artifact.family", &self.artifact.family)?;
        if v2 {
            check_required_text("artifact.schema", self.artifact.schema.as_deref())?;
        }
        if self.artifact.sections.is_empty() {
            return Err(bad(format!("{}: artifact has no section", self.id)));
        }

        let sections = unique_ids("artifact section", &self.artifact.sections, |s| &s.id)?;
        for section in &self.artifact.sections {
            check_slug("artifact section", &section.id)?;
            check_text("artifact section description", &section.description)?;
            if v2 {
                check_required_text("artifact section owner", section.owner.as_deref())?;
                check_required_text("artifact section format", section.format.as_deref())?;
            }
            if section.required {
                if section.optional_reason.is_some() {
                    return Err(bad(format!(
                        "{}: required artifact section {} names an optional reason",
                        self.id, section.id
                    )));
                }
            } else {
                check_optional_text(
                    "artifact section optional_reason",
                    section.optional_reason.as_deref(),
                )?;
                if v2 && section.optional_reason.as_deref().is_none_or(str::is_empty) {
                    return Err(bad(format!(
                        "{}: optional artifact section {} has no reason it may be absent",
                        self.id, section.id
                    )));
                }
            }
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

        if v2 && self.evidence.is_empty() {
            return Err(bad(format!("{}: contract has no evidence", self.id)));
        }
        unique_ids("evidence", &self.evidence, |e| &e.id)?;
        for evidence in &self.evidence {
            check_slug("evidence", &evidence.id)?;
            check_text("evidence.locator", &evidence.locator)?;
            check_text("evidence.scope", &evidence.scope)?;
            check_text("evidence.statement", &evidence.statement)?;
            if let Some(sha256) = &evidence.sha256 {
                check_sha256("evidence.sha256", sha256)?;
            }
            check_evidence_locator(root, &evidence.locator)?;
            if let Some(sha256) = evidence.sha256.as_deref() {
                if !is_external_locator(&evidence.locator) {
                    let actual = file_sha256(&root.join(&evidence.locator))?;
                    if actual != sha256.to_ascii_lowercase() {
                        return Err(bad(format!(
                            "{}: evidence {} locator hash is {actual}, expected {sha256}",
                            self.id, evidence.id
                        )));
                    }
                }
            }
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
            unique_flag_names(&mechanism.flags)?;
            if v2 {
                if mechanism.evidence_ids.is_empty() {
                    return Err(bad(format!(
                        "{}: mechanism {} has no structured evidence",
                        self.id, mechanism.id
                    )));
                }
                for evidence in &mechanism.evidence_ids {
                    let Some(row) = self.evidence.iter().find(|row| row.id == *evidence) else {
                        return Err(bad(format!(
                            "{}: mechanism {} names unknown evidence {evidence}",
                            self.id, mechanism.id
                        )));
                    };
                    if !row.mechanisms.contains(&mechanism.id) {
                        return Err(bad(format!(
                            "{}: mechanism {} cites evidence {} that does not name it",
                            self.id, mechanism.id, evidence
                        )));
                    }
                }
                let evidence_results = mechanism
                    .evidence_ids
                    .iter()
                    .filter_map(|id| self.evidence.iter().find(|row| row.id == *id))
                    .map(|row| row.result)
                    .collect::<Vec<_>>();
                let expected = match mechanism.status {
                    MechanismStatus::MeasuredPositive => Some(EvidenceResult::Pass),
                    MechanismStatus::MeasuredNegative => Some(EvidenceResult::Fail),
                    MechanismStatus::InstrumentInvalid => Some(EvidenceResult::InstrumentInvalid),
                    MechanismStatus::NotRun => Some(EvidenceResult::NotRun),
                    _ => None,
                };
                if expected.is_some_and(|expected| !evidence_results.contains(&expected)) {
                    return Err(bad(format!(
                        "{}: mechanism {} status {} has no matching evidence result",
                        self.id,
                        mechanism.id,
                        mechanism.status.as_str()
                    )));
                }
                for flag in mechanism.flags.iter().filter(|flag| flag.required) {
                    if !self
                        .commands
                        .iter()
                        .any(|command| command_sets_flag(&command.args, flag))
                    {
                        return Err(bad(format!(
                            "{}: mechanism {} requires {} {}, but no command sets that exact pair",
                            self.id, mechanism.id, flag.name, flag.value
                        )));
                    }
                }
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
            if reading.unit.is_some() != reading.better.is_some() {
                return Err(bad(format!(
                    "{}: reading {} declares a unit and direction together, or neither",
                    self.id, reading.id
                )));
            }
            if let Some(unit) = &reading.unit {
                check_text("reading.unit", unit)?;
            }
            if let Some(bar) = reading.bar {
                if !bar.is_finite() {
                    return Err(bad(format!(
                        "{}: reading {} has a non-finite bar",
                        self.id, reading.id
                    )));
                }
            }
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
            if v2 {
                check_required_text("probe.fixture", probe.fixture.as_deref())?;
                check_required_text("probe.fixture_sha256", probe.fixture_sha256.as_deref())?;
                let fixture = probe.fixture.as_deref().unwrap_or_default();
                let fixture_sha = probe.fixture_sha256.as_deref().unwrap_or_default();
                check_sha256("probe.fixture_sha256", fixture_sha)?;
                check_repo_path(root, "probe.fixture", fixture)?;
                let actual = file_sha256(&root.join(fixture))?;
                if actual != fixture_sha.to_ascii_lowercase() {
                    return Err(bad(format!(
                        "{}: probe {} fixture hash is {actual:?}, expected {fixture_sha:?}",
                        self.id, probe.id
                    )));
                }
                check_fixture(&root.join(fixture), probe)?;
                if probe.layers.is_empty() {
                    return Err(bad(format!(
                        "{}: probe {} names no execution layer",
                        self.id, probe.id
                    )));
                }
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

        for evidence in &self.evidence {
            for mechanism in &evidence.mechanisms {
                if !mechanism_ids.contains(mechanism.as_str()) {
                    return Err(bad(format!(
                        "{}: evidence {} names unknown mechanism {mechanism}",
                        self.id, evidence.id
                    )));
                }
            }
        }
        for mechanism in &self.mechanisms {
            for dependency in mechanism.depends_on.iter().chain(&mechanism.conflicts_with) {
                if !mechanism_ids.contains(dependency.as_str()) {
                    return Err(bad(format!(
                        "{}: mechanism {} names unknown mechanism relation {dependency}",
                        self.id, mechanism.id
                    )));
                }
                if dependency == &mechanism.id {
                    return Err(bad(format!(
                        "{}: mechanism {} names itself as a relation",
                        self.id, mechanism.id
                    )));
                }
            }
        }
        check_dependency_cycles(self)?;
        check_unique_orders(self)?;
        for section in &self.artifact.sections {
            if let Some(owner) = &section.owner {
                if !mechanism_ids.contains(owner.as_str()) {
                    return Err(bad(format!(
                        "{}: artifact section {} names unknown owner {owner}",
                        self.id, section.id
                    )));
                }
            }
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
    let mut current = BTreeSet::new();
    for file in files {
        file.contract.check(root)?;
        if file.contract.spec == SPEC
            && file.contract.state == ContractState::Current
            && !current.insert(file.contract.lane.as_str())
        {
            return Err(ModelError::Inconsistent(format!(
                "lane {} has more than one current contract",
                file.contract.lane.as_str()
            )));
        }
    }
    Ok(())
}

const fn default_revision() -> u32 {
    1
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

fn check_optional_text(name: &str, value: Option<&str>) -> Result<(), ModelError> {
    check_text(name, value.unwrap_or_default())
}

fn check_required_text(name: &str, value: Option<&str>) -> Result<(), ModelError> {
    let Some(value) = value else {
        return Err(ModelError::Inconsistent(format!("{name} is required")));
    };
    check_text(name, value)
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

fn check_evidence_locator(root: &Path, value: &str) -> Result<(), ModelError> {
    if is_external_locator(value) {
        Ok(())
    } else {
        check_repo_path(root, "evidence.locator", value)
    }
}

fn is_external_locator(value: &str) -> bool {
    value.starts_with("store:")
        || value.starts_with("https://")
        || value.starts_with("http://")
        || value.starts_with("receipt:")
        || value.starts_with("authority:")
}

fn unique_flag_names(flags: &[FlagSetting]) -> Result<(), ModelError> {
    let mut seen = BTreeSet::new();
    for flag in flags {
        if !flag.name.starts_with("--") || flag.name.len() == 2 {
            return Err(ModelError::Inconsistent(format!(
                "mechanism flag {:?} is not a long option",
                flag.name
            )));
        }
        if !seen.insert(flag.name.as_str()) {
            return Err(ModelError::Inconsistent(format!(
                "mechanism flag {:?} is registered twice",
                flag.name
            )));
        }
    }
    Ok(())
}

fn command_sets_flag(args: &[String], flag: &FlagSetting) -> bool {
    args.windows(2).any(|pair| {
        pair[0] == flag.name
            && (pair[1] == flag.value
                || (flag.value == "true" && (pair[1] == "1" || pair[1].starts_with("--")))
                || (flag.value == "false" && pair[1] == "0"))
    }) || (flag.value == "true" && args.last().is_some_and(|arg| arg == &flag.name))
}

fn check_dependency_cycles(contract: &StackContract) -> Result<(), ModelError> {
    fn visit<'a>(
        id: &'a str,
        contract: &'a StackContract,
        active: &mut BTreeSet<&'a str>,
        done: &mut BTreeSet<&'a str>,
    ) -> Result<(), ModelError> {
        if done.contains(id) {
            return Ok(());
        }
        if !active.insert(id) {
            return Err(ModelError::Inconsistent(format!(
                "{}: mechanism dependency cycle reaches {id}",
                contract.id
            )));
        }
        let mechanism = contract
            .mechanisms
            .iter()
            .find(|mechanism| mechanism.id == id)
            .expect("validated mechanism id");
        for dependency in &mechanism.depends_on {
            visit(dependency, contract, active, done)?;
        }
        active.remove(id);
        done.insert(id);
        Ok(())
    }

    let mut active = BTreeSet::new();
    let mut done = BTreeSet::new();
    for mechanism in &contract.mechanisms {
        visit(&mechanism.id, contract, &mut active, &mut done)?;
    }
    Ok(())
}

fn check_unique_orders(contract: &StackContract) -> Result<(), ModelError> {
    let mut load = BTreeSet::new();
    let mut cleanup = BTreeSet::new();
    for mechanism in &contract.mechanisms {
        if let Some(order) = mechanism.load_order {
            if !load.insert(order) {
                return Err(ModelError::Inconsistent(format!(
                    "{}: load order {order} is assigned more than once",
                    contract.id
                )));
            }
        }
        if let Some(order) = mechanism.cleanup_order {
            if !cleanup.insert(order) {
                return Err(ModelError::Inconsistent(format!(
                    "{}: cleanup order {order} is assigned more than once",
                    contract.id
                )));
            }
        }
    }
    Ok(())
}

fn check_fixture(path: &Path, probe: &Probe) -> Result<(), ModelError> {
    let bytes = fs::read(path).map_err(|error| ModelError::Io(path.to_path_buf(), error))?;
    let fixture: Value = serde_json::from_slice(&bytes).map_err(|error| {
        ModelError::Inconsistent(format!(
            "{}: fixture does not parse: {error}",
            path.display()
        ))
    })?;
    let schema = fixture.get("schema").and_then(Value::as_str);
    if schema != Some(FIXTURE_SCHEMA) {
        return Err(ModelError::Inconsistent(format!(
            "{}: fixture schema {schema:?} is not {FIXTURE_SCHEMA:?}",
            path.display()
        )));
    }
    if fixture.get("probe").and_then(Value::as_str) != Some(probe.id.as_str()) {
        return Err(ModelError::Inconsistent(format!(
            "{}: fixture probe does not name {}",
            path.display(),
            probe.id
        )));
    }
    let cases = fixture
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            ModelError::Inconsistent(format!("{}: fixture has no cases array", path.display()))
        })?;
    if cases.is_empty() {
        return Err(ModelError::Inconsistent(format!(
            "{}: fixture has no executable case",
            path.display()
        )));
    }
    let mut ids = BTreeSet::new();
    for case in cases {
        let id = case.get("id").and_then(Value::as_str).ok_or_else(|| {
            ModelError::Inconsistent(format!(
                "{}: every fixture case names an id",
                path.display()
            ))
        })?;
        check_slug("fixture case", id)?;
        if !ids.insert(id) {
            return Err(ModelError::Inconsistent(format!(
                "{}: fixture case {id} is registered twice",
                path.display()
            )));
        }
        if case.get("script").is_none() || case.get("expect").is_none() {
            return Err(ModelError::Inconsistent(format!(
                "{}: fixture case {id} needs script and expect objects",
                path.display()
            )));
        }
        let script = case["script"].as_object().ok_or_else(|| {
            ModelError::Inconsistent(format!(
                "{}: fixture case {id} script is not an object",
                path.display()
            ))
        })?;
        if script.get("schema").and_then(Value::as_str) != Some("uor-r4.chat-script/1")
            || !script
                .get("turns")
                .and_then(Value::as_array)
                .is_some_and(|turns| (1..=256).contains(&turns.len()))
        {
            return Err(ModelError::Inconsistent(format!(
                "{}: fixture case {id} script is not a 1..=256 turn chat script",
                path.display()
            )));
        }
        let expect = case["expect"].as_object().ok_or_else(|| {
            ModelError::Inconsistent(format!(
                "{}: fixture case {id} expect is not an object",
                path.display()
            ))
        })?;
        const EXPECT_KEYS: &[&str] = &[
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
        if let Some(unknown) = expect
            .keys()
            .find(|key| !EXPECT_KEYS.contains(&key.as_str()))
        {
            return Err(ModelError::Inconsistent(format!(
                "{}: fixture case {id} has unknown expectation {unknown}",
                path.display()
            )));
        }
        if expect.is_empty() {
            return Err(ModelError::Inconsistent(format!(
                "{}: fixture case {id} has no semantic expectation",
                path.display()
            )));
        }
        let semantic = [
            "turns",
            "turn_count",
            "all_reply_ids_nonempty",
            "stops",
            "memory",
            "distinct_replies_min",
            "reply_text_contains",
            "same_as",
        ]
        .iter()
        .any(|key| expect.contains_key(*key))
            || (expect.get("status").and_then(Value::as_str) == Some("error")
                && expect.contains_key("stderr_contains"));
        if !semantic {
            return Err(ModelError::Inconsistent(format!(
                "{}: fixture case {id} has no recognized semantic assertion",
                path.display()
            )));
        }
        check_fixture_expectation(path, id, expect, cases)?;
        if let Some(args) = case.get("args") {
            if !args
                .as_array()
                .is_some_and(|args| args.iter().all(Value::is_string))
            {
                return Err(ModelError::Inconsistent(format!(
                    "{}: fixture case {id} args must be an array of strings",
                    path.display()
                )));
            }
        }
        if let Some(repeat) = case.get("repeat_user") {
            if repeat.get("text").and_then(Value::as_str).is_none()
                || repeat
                    .get("count")
                    .and_then(Value::as_u64)
                    .is_none_or(|count| !(1..=100_000).contains(&count))
            {
                return Err(ModelError::Inconsistent(format!(
                    "{}: fixture case {id} repeat_user needs text and count 1..=100000",
                    path.display()
                )));
            }
        }
    }
    let declared: BTreeSet<String> = probe
        .cases
        .iter()
        .map(|case| normalize_case_name(case))
        .collect();
    let found: BTreeSet<String> = ids.iter().map(|id| normalize_case_name(id)).collect();
    if declared != found {
        return Err(ModelError::Inconsistent(format!(
            "{}: fixture cases {:?} do not match contract cases {:?}",
            path.display(),
            found,
            declared
        )));
    }
    Ok(())
}

fn normalize_case_name(value: &str) -> String {
    value
        .bytes()
        .filter(|byte| byte.is_ascii_alphanumeric())
        .map(|byte| byte.to_ascii_lowercase() as char)
        .collect()
}

fn check_fixture_expectation(
    path: &Path,
    id: &str,
    expect: &serde_json::Map<String, Value>,
    cases: &[Value],
) -> Result<(), ModelError> {
    let bad = |message: &str| {
        ModelError::Inconsistent(format!("{}: fixture case {id} {message}", path.display()))
    };
    for key in ["turns", "turn_count", "distinct_replies_min"] {
        if let Some(value) = expect.get(key) {
            if value.as_u64().is_none_or(|value| value == 0) {
                return Err(bad(&format!("{key} must be a positive integer")));
            }
        }
    }
    if let Some(value) = expect.get("all_reply_ids_nonempty") {
        if value.as_bool() != Some(true) {
            return Err(bad("all_reply_ids_nonempty must be true"));
        }
    }
    for key in ["stops", "memory", "reply_text_contains"] {
        if let Some(value) = expect.get(key) {
            if !value
                .as_array()
                .is_some_and(|items| items.iter().all(Value::is_string))
            {
                return Err(bad(&format!("{key} must be an array of strings")));
            }
        }
    }
    if let Some(stops) = expect.get("stops").and_then(Value::as_array) {
        if stops.is_empty()
            || stops.iter().any(|stop| {
                !matches!(
                    stop.as_str(),
                    Some("eos" | "max_new_tokens" | "short_cycle")
                )
            })
        {
            return Err(bad("stops must name eos, max_new_tokens or short_cycle"));
        }
    }
    if let Some(memory) = expect.get("memory").and_then(Value::as_array) {
        if memory
            .iter()
            .any(|source| !matches!(source.as_str(), Some("none" | "explicit" | "sieve")))
        {
            return Err(bad("memory must name none, explicit or sieve"));
        }
    }
    let status = match expect.get("status") {
        None => None,
        Some(value) => match value.as_str() {
            Some(status @ ("record" | "error")) => Some(status),
            _ => return Err(bad("status must be record or error")),
        },
    };
    if let Some(stderr) = expect.get("stderr_contains") {
        if status != Some("error") || stderr.as_str().is_none_or(|text| text.trim().is_empty()) {
            return Err(bad(
                "stderr_contains must be non-empty text and needs status = error",
            ));
        }
    }
    if let Some(same_as) = expect.get("same_as") {
        let Some(same_as) = same_as.as_str() else {
            return Err(bad("same_as must be a slug string"));
        };
        check_slug("fixture same_as", same_as)?;
        if same_as == id
            || !cases
                .iter()
                .any(|other| other["id"].as_str() == Some(same_as))
        {
            return Err(bad("same_as must name another fixture case"));
        }
    }
    if let Some(patterns) = expect.get("reply_text_contains").and_then(Value::as_array) {
        if !patterns
            .iter()
            .any(|pattern| pattern.as_str().is_some_and(|p| !p.is_empty()))
        {
            return Err(bad("reply_text_contains has no text"));
        }
    }
    Ok(())
}

fn file_sha256(path: &Path) -> Result<String, ModelError> {
    let bytes = fs::read(path).map_err(|error| ModelError::Io(path.to_path_buf(), error))?;
    Ok(Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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

    #[test]
    fn v2_requires_exact_training_serving_and_artifact_metadata() {
        let mut contract = parsed();
        contract.spec = SPEC.to_owned();
        contract.model.training_context = Some(192);
        let error = contract.check(&root()).unwrap_err().to_string();
        assert!(error.contains("model.training is required"), "{error}");

        contract.model.training = Some("float32".to_owned());
        contract.model.serving = Some("integer".to_owned());
        let error = contract.check(&root()).unwrap_err().to_string();
        assert!(error.contains("artifact.schema is required"), "{error}");

        contract.artifact.schema = Some("uor-r4.lut-stack/9".to_owned());
        let error = contract.check(&root()).unwrap_err().to_string();
        assert!(
            error.contains("artifact section owner is required"),
            "{error}"
        );
    }

    #[test]
    fn required_flags_must_match_an_exact_command_pair() {
        let flag = FlagSetting {
            name: "--copy-rule".to_owned(),
            value: "latest-exact".to_owned(),
            required: true,
        };
        assert!(command_sets_flag(
            &[
                "serve".to_owned(),
                "--copy-rule".to_owned(),
                "latest-exact".to_owned()
            ],
            &flag
        ));
        assert!(!command_sets_flag(
            &[
                "serve".to_owned(),
                "--copy-rule".to_owned(),
                "mean".to_owned()
            ],
            &flag
        ));
        assert!(!command_sets_flag(
            &["serve".to_owned(), "--copy-rule=latest-exact".to_owned()],
            &flag
        ));
    }

    #[test]
    fn dependency_cycles_self_relations_and_duplicate_orders_are_rejected() {
        let mut cycle = parsed();
        cycle.mechanisms[0].depends_on = vec!["trunk".to_owned()];
        let error = cycle.check(&root()).unwrap_err().to_string();
        assert!(error.contains("names itself"), "{error}");

        let mut second = cycle.mechanisms[0].clone();
        second.id = "second".to_owned();
        second.evidence_ids.clear();
        cycle.mechanisms[0].depends_on = vec!["second".to_owned()];
        second.depends_on = vec!["trunk".to_owned()];
        cycle.mechanisms.push(second);
        let error = cycle.check(&root()).unwrap_err().to_string();
        assert!(error.contains("dependency cycle"), "{error}");

        let mut order = parsed();
        let mut second = order.mechanisms[0].clone();
        second.id = "second".to_owned();
        second.evidence_ids.clear();
        order.mechanisms[0].load_order = Some(1);
        second.load_order = Some(1);
        order.mechanisms.push(second);
        let error = order.check(&root()).unwrap_err().to_string();
        assert!(error.contains("load order 1"), "{error}");
    }

    #[test]
    fn fixture_schema_probe_and_unique_cases_are_checked() {
        let dir =
            std::env::temp_dir().join(format!("repo-model-stack-fixture-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("temp fixture directory");
        let path = dir.join("fixture.json");
        let probe = Probe {
            id: "cycle-period-boundary".to_owned(),
            class: ProbeClass::Boundary,
            mechanism: "trunk".to_owned(),
            axis: "cycle period".to_owned(),
            expectation: "periods are distinguished".to_owned(),
            cases: vec!["period 1".to_owned()],
            fixture: Some("fixture.json".to_owned()),
            fixture_sha256: Some("0".repeat(64)),
            layers: vec![ExecutionLayer::Chat],
        };
        fs::write(
            &path,
            format!(
                r#"{{"schema":"{FIXTURE_SCHEMA}","probe":"cycle-period-boundary","cases":[{{"id":"period-1","script":{{"schema":"uor-r4.chat-script/1","turns":[{{"user":"Hello"}}]}},"expect":{{"turns":1}}}}]}}"#
            ),
        )
        .expect("fixture write");
        check_fixture(&path, &probe).expect("fixture checks");
        let valid: Value =
            serde_json::from_slice(&fs::read(&path).expect("fixture read")).expect("fixture json");
        for (expectation, expected_error) in [
            (json!({"turn_count": "1"}), "positive integer"),
            (json!({"turns": "1"}), "positive integer"),
            (json!({"distinct_replies_min": "1"}), "positive integer"),
            (json!({"all_reply_ids_nonempty": "true"}), "must be true"),
            (json!({"same_as": 7}), "slug string"),
            (
                json!({"status": "error", "stderr_contains": 42}),
                "non-empty text",
            ),
            (
                json!({"turns": 1, "stderr_contains": "boom"}),
                "needs status = error",
            ),
            (json!({"turns": 1, "status": 7}), "status must be"),
            (json!({"turns": 1, "status": "maybe"}), "status must be"),
            (json!({"stops": ["eos", 7]}), "array of strings"),
            (json!({"memory": 7}), "array of strings"),
            (json!({"memory": ["none", 7]}), "array of strings"),
            (json!({"reply_text_contains": 7}), "array of strings"),
            (
                json!({"reply_text_contains": ["47", 7]}),
                "array of strings",
            ),
        ] {
            let mut fixture = valid.clone();
            fixture["cases"][0]["expect"] = expectation;
            fs::write(&path, serde_json::to_vec(&fixture).expect("fixture json"))
                .expect("fixture write");
            let error = check_fixture(&path, &probe)
                .expect_err("wrong expectation type is refused")
                .to_string();
            assert!(error.contains(expected_error), "{error}");
        }
        let mut fixture = valid.clone();
        fixture["cases"][0]["args"] = json!(["--max-tokens", 7]);
        fs::write(&path, serde_json::to_vec(&fixture).expect("fixture json"))
            .expect("fixture write");
        let error = check_fixture(&path, &probe)
            .expect_err("wrong arg type is refused")
            .to_string();
        assert!(error.contains("array of strings"), "{error}");

        fs::write(
            &path,
            format!(
                r#"{{"schema":"{FIXTURE_SCHEMA}","probe":"other","cases":[{{"id":"period-1","script":{{"schema":"uor-r4.chat-script/1","turns":[{{"user":"Hello"}}]}},"expect":{{"turns":1}}}}]}}"#
            ),
        )
        .expect("fixture write");
        let error = check_fixture(&path, &probe).unwrap_err().to_string();
        assert!(error.contains("does not name"), "{error}");
        fs::remove_dir_all(dir).expect("temp fixture cleanup");
    }
}

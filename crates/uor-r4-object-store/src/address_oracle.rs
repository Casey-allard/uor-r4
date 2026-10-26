//! Pure object-store address-oracle manifest, panel, address and contrast layer.
//!
//! This module composes the existing [`crate::address_read`] exact and sieve
//! reads with a versioned panel manifest. It does not add a second store, a
//! model writer, or a host model evaluation. Store correctness (exact key,
//! requested version, verified content), oracle assistance (true address versus
//! matched controls), address formation (the deterministic sieve) and model
//! generation remain separate quantities.
//!
//! The object store's [`AddressDirectory`] `BTreeMap` exact read is the
//! authority for `Current`, `Previous`, `Absent` and `Evicted`. This layer only
//! classifies those outcomes and never substitutes another slot's value.
#![forbid(unsafe_code)]

use crate::address_read::{
    AddressDirectory, AddressReadOutcome, ObjectTarget, PrimeAddress, VersionSelect,
};
use crate::bindings::PinnedTokenizer;
use crate::{Limits, Manifest, Store};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use uor_r4_tokenizer::ByteBpeTokenizer;

/// Versioned panel schema.
pub const PANEL_SCHEMA: &str = "uor-r4.address-oracle-panel/1";
/// Versioned sieve-lexicon schema.
pub const LEXICON_SCHEMA: &str = "uor-r4.address-oracle-lexicon/1";
/// Panel schema version.
pub const PANEL_VERSION: u32 = 1;
/// Sieve-lexicon schema version.
pub const LEXICON_VERSION: u32 = 1;
/// Deterministic panel seed.
pub const PANEL_SEED: &str = "m1-address-oracle-v1";

/// Current-value rows in the panel.
pub const CURRENT_ROWS: usize = 40;
/// Previous-value rows in the panel.
pub const PREVIOUS_ROWS: usize = 20;
/// Typed-abstention rows in the panel.
pub const ABSTENTION_ROWS: usize = 20;
/// Total panel rows.
pub const PANEL_ROWS: usize = CURRENT_ROWS + PREVIOUS_ROWS + ABSTENTION_ROWS;
/// Absent rows used by the deterministic panel.
pub const ABSENT_ROWS: usize = 10;
/// Evicted rows used by the deterministic panel.
pub const EVICTED_ROWS: usize = 10;

/// Default row cap for a generated panel.
pub const DEFAULT_MAX_ROWS: usize = PANEL_ROWS;
/// Default active-slot cap for the generated 10-absent/10-evicted panel.
pub const DEFAULT_MAX_SLOTS: usize = 150;
/// Default key-atom cap.
pub const DEFAULT_MAX_KEY_ATOMS: usize = 8;
/// Default value-token cap.
pub const DEFAULT_MAX_VALUE_TOKENS: usize = 16;
/// Default value-byte cap.
pub const DEFAULT_MAX_VALUE_BYTES: usize = 32;
/// Default prompt-token cap.
pub const DEFAULT_MAX_PROMPT_TOKENS: usize = 64;
/// Default prompt-byte cap.
pub const DEFAULT_MAX_PROMPT_BYTES: usize = 256;

/// Absolute row cap accepted from any manifest.
pub const HARD_MAX_ROWS: usize = 256;
/// Absolute active-slot cap accepted from any manifest.
pub const HARD_MAX_SLOTS: usize = 512;
/// Absolute key-atom cap accepted from any manifest.
pub const HARD_MAX_KEY_ATOMS: usize = 16;
/// Absolute value-token cap accepted from any manifest.
pub const HARD_MAX_VALUE_TOKENS: usize = 64;
/// Absolute value-byte cap accepted from any manifest.
pub const HARD_MAX_VALUE_BYTES: usize = 256;
/// Absolute prompt-token cap accepted from any manifest.
pub const HARD_MAX_PROMPT_TOKENS: usize = 512;
/// Absolute prompt-byte cap accepted from any manifest.
pub const HARD_MAX_PROMPT_BYTES: usize = 4096;
/// Absolute ordered-pair cap per address.
pub const HARD_MAX_PAIRS: usize = 256;
/// Absolute lexicon-entry cap.
pub const HARD_MAX_LEXICON_ENTRIES: usize = 256;
/// Absolute encoded-panel byte cap before parsing.
pub const HARD_MAX_PANEL_BYTES: usize = 4 * 1024 * 1024;

const VALUE_BYTES: usize = 8;
const VALUE_ALPHABET: &[u8; 26] = b"abcdefghijklmnopqrstuvwxyz";
const EVICTOR_PAIR_BASE: usize = 2 * PANEL_ROWS;

// Twenty-four distinct primes give 276 ordered-ascending prime pairs. The
// generated panel uses pairs 0..159 and the eviction ring starts at 160.
const PRIME_POOL: [u32; 24] = [
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89,
];
const LEXICON_WORDS: [&str; 24] = [
    "amber", "birch", "cedar", "delta", "ember", "frost", "glade", "hazel", "ivory", "jewel",
    "knoll", "larch", "maple", "north", "ocean", "pearl", "quill", "raven", "sable", "tulip",
    "umber", "vivid", "wren", "yarrow",
];

/// The five matched arms. Model arms (`NoMemory`, `NoAddress`) take a model
/// emission from the caller; the other three are pure store reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Arm {
    NoMemory,
    NoAddress,
    Sieve,
    TrueAddress,
    RandomAddress,
}

impl Arm {
    /// Every arm in receipt order.
    pub const ALL: [Self; 5] = [
        Self::NoMemory,
        Self::NoAddress,
        Self::Sieve,
        Self::TrueAddress,
        Self::RandomAddress,
    ];

    /// Whether this arm is scored from a model emission.
    pub fn is_model(self) -> bool {
        matches!(self, Self::NoMemory | Self::NoAddress)
    }

    /// Whether this arm is scored from an object-store read.
    pub fn is_store(self) -> bool {
        !self.is_model()
    }
}

/// Source marker attached to an outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EmissionSource {
    /// Exact `AddressDirectory` read, including the random-address control.
    ExactStore,
    /// `AddressDirectory::read_sieve` read.
    SieveStore,
    /// Caller-supplied model emission.
    Model,
}

/// Panel family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RowFamily {
    Current,
    Previous,
    Abstention,
}

/// Requested exact version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RequestedVersion {
    Current,
    Previous,
}

impl From<RequestedVersion> for VersionSelect {
    fn from(value: RequestedVersion) -> Self {
        match value {
            RequestedVersion::Current => VersionSelect::Current,
            RequestedVersion::Previous => VersionSelect::Previous,
        }
    }
}

/// Typed refusal kind. A missing previous version is `Absent`, never `Current`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AbstentionKind {
    Absent,
    Evicted,
}

/// Declared expected outcome for one row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpectedOutcome {
    Value(ValueSpec),
    Abstention(AbstentionKind),
}

/// Canonical prime-factor address with ordered relation pairs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AddressSpec {
    pub primes: Vec<u32>,
    pub pairs: Vec<(u32, u32)>,
}

impl AddressSpec {
    /// Validate canonical order, primality and pair endpoints, then construct
    /// the exact [`PrimeAddress`] used by the directory.
    pub fn to_prime_address(&self) -> Result<PrimeAddress, String> {
        if self.primes.is_empty() {
            return Err("address has no prime atoms".into());
        }
        if self.primes.windows(2).any(|window| window[0] >= window[1]) {
            return Err("address primes are not strictly ascending".into());
        }
        if self.pairs.windows(2).any(|window| window[0] >= window[1]) {
            return Err("address pairs are not strictly ascending".into());
        }
        for &(left, right) in &self.pairs {
            if self.primes.binary_search(&left).is_err()
                || self.primes.binary_search(&right).is_err()
            {
                return Err("ordered-pair endpoint is outside the prime set".into());
            }
        }
        let address = PrimeAddress::with_pairs(&self.primes, &self.pairs)
            .map_err(|error| format!("invalid prime address: {error:?}"))?;
        if address.primes() != self.primes.as_slice()
            || address.ordered_pairs() != self.pairs.as_slice()
        {
            return Err("address is not canonical".into());
        }
        Ok(address)
    }

    fn validate(&self, max_atoms: usize) -> Result<PrimeAddress, String> {
        if self.primes.len() > max_atoms {
            return Err(format!(
                "address has {} atoms, limit {max_atoms}",
                self.primes.len()
            ));
        }
        let max_pairs = max_atoms.saturating_mul(max_atoms);
        if self.pairs.len() > max_pairs || self.pairs.len() > HARD_MAX_PAIRS {
            return Err(format!(
                "address has {} pairs, limit {max_pairs}",
                self.pairs.len()
            ));
        }
        self.to_prime_address()
    }
}

impl From<&PrimeAddress> for AddressSpec {
    fn from(address: &PrimeAddress) -> Self {
        Self {
            primes: address.primes().to_vec(),
            pairs: address.ordered_pairs().to_vec(),
        }
    }
}

/// Explicit value text and its pinned tokenization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValueSpec {
    pub text: String,
    pub tokens: Vec<u32>,
}

/// A present random-address control binding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RandomBinding {
    pub address: AddressSpec,
    pub value: ValueSpec,
}

/// Explicit manifest bounds, checked before any allocation or store work.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PanelLimits {
    pub max_rows: usize,
    pub max_slots: usize,
    pub max_key_atoms: usize,
    pub max_value_tokens: usize,
    pub max_value_bytes: usize,
    pub max_prompt_tokens: usize,
    pub max_prompt_bytes: usize,
}

impl Default for PanelLimits {
    fn default() -> Self {
        Self {
            max_rows: DEFAULT_MAX_ROWS,
            max_slots: DEFAULT_MAX_SLOTS,
            max_key_atoms: DEFAULT_MAX_KEY_ATOMS,
            max_value_tokens: DEFAULT_MAX_VALUE_TOKENS,
            max_value_bytes: DEFAULT_MAX_VALUE_BYTES,
            max_prompt_tokens: DEFAULT_MAX_PROMPT_TOKENS,
            max_prompt_bytes: DEFAULT_MAX_PROMPT_BYTES,
        }
    }
}

/// One versioned sieve word-to-prime registry entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LexiconEntry {
    pub word: String,
    pub prime: u32,
}

/// Versioned deterministic sieve lexicon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SieveLexicon {
    pub schema: String,
    pub version: u32,
    pub entries: Vec<LexiconEntry>,
}

impl SieveLexicon {
    /// Validate schema, version, canonical word order and unique prime entries.
    pub fn validate(&self) -> Result<(), String> {
        validate_lexicon(self)
    }

    /// Exact word lookup.
    pub fn lookup(&self, word: &str) -> Option<u32> {
        self.entries
            .iter()
            .find(|entry| entry.word.as_str() == word)
            .map(|entry| entry.prime)
    }
}

/// One panel row: prompt, true address, requested version, expected outcome,
/// exact write history and planted random binding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PanelRow {
    pub id: String,
    pub family: RowFamily,
    pub prompt: String,
    pub prompt_tokens: Vec<u32>,
    pub address: AddressSpec,
    pub version: RequestedVersion,
    pub expected: ExpectedOutcome,
    /// Values written to the true key, oldest first. A current row has one, a
    /// previous row has two, an absent row has none, and an evicted row has one.
    pub history: Vec<ValueSpec>,
    pub random: RandomBinding,
}

impl PanelRow {
    /// Exact true-key address.
    pub fn true_address(&self) -> Result<PrimeAddress, String> {
        self.address.to_prime_address()
    }

    /// Exact random-control address.
    pub fn random_address(&self) -> Result<PrimeAddress, String> {
        self.random.address.to_prime_address()
    }

    /// Expected value, when the row is a value row.
    pub fn expected_value(&self) -> Option<&ValueSpec> {
        match &self.expected {
            ExpectedOutcome::Value(value) => Some(value),
            ExpectedOutcome::Abstention(_) => None,
        }
    }

    /// Expected typed abstention, when the row is an abstention row.
    pub fn expected_abstention(&self) -> Option<AbstentionKind> {
        match &self.expected {
            ExpectedOutcome::Abstention(kind) => Some(*kind),
            ExpectedOutcome::Value(_) => None,
        }
    }
}

/// Versioned 40-current / 20-previous / 20-abstention panel manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PanelManifest {
    pub schema: String,
    pub panel_version: u32,
    pub seed: String,
    pub tokenizer_identity: String,
    pub limits: PanelLimits,
    pub lexicon: SieveLexicon,
    pub rows: Vec<PanelRow>,
}

impl PanelManifest {
    /// Validate this manifest. Pass the pinned tokenizer to check token IDs,
    /// tokenization and tokenizer identity; `None` checks only structure.
    pub fn validate(&self, tokenizer: Option<&ByteBpeTokenizer>) -> Result<(), String> {
        validate_panel(self, tokenizer)
    }

    /// Canonical pretty JSON plus a trailing newline. Encoding does not
    /// validate; call [`Self::validate`] before treating bytes as a panel.
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        encode_panel(self)
    }

    /// SHA-256 of [`Self::encode`].
    pub fn sha256(&self) -> Result<String, String> {
        panel_sha256(self)
    }

    /// Parse and structurally validate a manifest.
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        parse_panel(bytes)
    }
}

/// One scored arm outcome.
///
/// `formed_query` and `query_matches_true` describe sieve address formation
/// independently of the slot the read selected. `selected_address_match` is
/// true only when a hit's selected address equals the arm's intended address;
/// typed abstentions have no selected slot and therefore leave it false.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmRecord {
    pub row_id: String,
    pub arm: Arm,
    pub source: EmissionSource,
    pub emitted_text: Option<String>,
    pub abstention: Option<AbstentionKind>,
    pub address: Option<AddressSpec>,
    pub version: Option<u64>,
    pub object_version: Option<String>,
    pub whole_object_verified: bool,
    /// The deterministic sieve query, when this is a sieve record.
    pub formed_query: Option<AddressSpec>,
    /// Whether the formed sieve query equals the row's true address.
    pub query_matches_true: bool,
    /// Whether a hit's selected address equals the arm's intended address.
    pub selected_address_match: bool,
    pub value_check_pass: bool,
    pub typed_abstention_pass: bool,
    pub wrong_substitution: bool,
    /// True when a sieve read on a previous-value row returned a current value.
    pub sieve_current_fallback: bool,
    /// True when the random arm emitted the planted random value at its key.
    pub random_binding_pass: bool,
}

impl ArmRecord {
    /// Strict expected-outcome match: value exactness or the expected typed
    /// abstention. A model emission can never satisfy the typed-abstention half.
    pub fn correct(&self) -> bool {
        self.value_check_pass || self.typed_abstention_pass
    }

    /// Sieve address formation: the formed query equals the true address. This
    /// is deliberately independent of whether any slot was selected.
    pub fn sieve_address_match(&self) -> bool {
        self.arm == Arm::Sieve && self.query_matches_true
    }

    /// Explicit query-formation alias for [`Self::sieve_address_match`].
    pub fn sieve_query_address_match(&self) -> bool {
        self.sieve_address_match()
    }

    /// Sieve selected-slot match: a hit's selected address equals the true
    /// address. Typed abstentions leave this false.
    pub fn sieve_selected_address_match(&self) -> bool {
        self.arm == Arm::Sieve && self.selected_address_match
    }

    /// Sieve value correctness, kept separate from address formation and slot
    /// selection.
    pub fn sieve_value_check_pass(&self) -> bool {
        self.arm == Arm::Sieve && self.value_check_pass
    }
}

/// Paired `b`/`c` counts; `d = b - c`. Rows where both arms agree are omitted.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairedContrast {
    pub rows: u64,
    pub b: u64,
    pub c: u64,
}

impl PairedContrast {
    pub fn d(&self) -> i64 {
        self.b as i64 - self.c as i64
    }
}

/// One family used by a paired contrast. Contrasts never pool families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContrastPopulation {
    Current,
    Previous,
    Abstention,
}

impl ContrastPopulation {
    /// The panel family this population selects.
    pub fn row_family(self) -> RowFamily {
        match self {
            Self::Current => RowFamily::Current,
            Self::Previous => RowFamily::Previous,
            Self::Abstention => RowFamily::Abstention,
        }
    }

    fn includes(self, family: RowFamily) -> bool {
        self.row_family() == family
    }
}

/// Hard true-address gate report.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HardGateReport {
    pub rows: u64,
    pub current_pass: u64,
    pub current_total: u64,
    pub previous_pass: u64,
    pub previous_total: u64,
    pub abstention_pass: u64,
    pub abstention_total: u64,
    pub absent_pass: u64,
    pub absent_total: u64,
    pub evicted_pass: u64,
    pub evicted_total: u64,
    pub wrong_substitutions: u64,
}

impl HardGateReport {
    /// Exactly 40/40 current, 20/20 previous, 20/20 typed abstention,
    /// exactly 10/10 absent and 10/10 evicted, and zero wrong-value
    /// substitutions.
    pub fn passed(&self) -> bool {
        self.rows == PANEL_ROWS as u64
            && self.current_pass == CURRENT_ROWS as u64
            && self.current_total == CURRENT_ROWS as u64
            && self.previous_pass == PREVIOUS_ROWS as u64
            && self.previous_total == PREVIOUS_ROWS as u64
            && self.abstention_pass == ABSTENTION_ROWS as u64
            && self.abstention_total == ABSTENTION_ROWS as u64
            && self.absent_pass == ABSENT_ROWS as u64
            && self.absent_total == ABSENT_ROWS as u64
            && self.evicted_pass == EVICTED_ROWS as u64
            && self.evicted_total == EVICTED_ROWS as u64
            && self.wrong_substitutions == 0
    }
}

/// Composed gate: exact hard gates plus every independent control. All fields
/// are required; a missing or false control fails [`Self::passed`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComposedGateReport {
    pub hard_gates: HardGateReport,
    pub baseline_identical: bool,
    pub random_control_holds: bool,
    /// The runner's 14,287-row D11 parity result.
    pub parity_pass: bool,
    /// No expected/stored value leak in prompts or TRAIN/DEV.
    pub leak_free: bool,
    /// Artifact, checkpoint, tokenizer, config and source identities complete.
    pub identity_complete: bool,
}

impl ComposedGateReport {
    /// Hard gates alone cannot pass this gate.
    pub fn passed(&self) -> bool {
        self.hard_gates.passed()
            && self.baseline_identical
            && self.random_control_holds
            && self.parity_pass
            && self.leak_free
            && self.identity_complete
    }
}

/// Build a composed gate from the exact hard-gate report and every control.
/// There are no optional controls: false or absent means failure.
pub fn composed_gate(
    hard_gates: HardGateReport,
    baseline_identical: bool,
    random_control_holds: bool,
    parity_pass: bool,
    leak_free: bool,
    identity_complete: bool,
) -> ComposedGateReport {
    ComposedGateReport {
        hard_gates,
        baseline_identical,
        random_control_holds,
        parity_pass,
        leak_free,
        identity_complete,
    }
}

/// Per-family outcome class. `NoGain` is terminal for this panel and carries no
/// training recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OutcomeClass {
    InstrumentFailure,
    Ceiling,
    Gain,
    NoGain,
}

/// One family's classified oracle outcome. No training recommendation field
/// exists, including for [`OutcomeClass::NoGain`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FamilyOutcome {
    pub family: ContrastPopulation,
    pub class: OutcomeClass,
    pub contrast: PairedContrast,
    pub control_exact: bool,
    pub hard_gates_pass: bool,
}

/// Classify one family only. Hard-gate failure is an instrument failure;
/// otherwise an already-exact control is a ceiling, a positive treatment
/// difference is a gain, and everything else is explicit no-gain.
pub fn classify_family(
    panel: &PanelManifest,
    family: ContrastPopulation,
    hard_gates: &HardGateReport,
    treatment: &[ArmRecord],
    control: &[ArmRecord],
) -> Result<FamilyOutcome, String> {
    if family == ContrastPopulation::Previous
        && treatment
            .iter()
            .chain(control.iter())
            .any(|record| record.arm == Arm::Sieve)
    {
        return Err("Previous sieve rows are descriptive and excluded from contrasts".into());
    }
    let contrast = paired_contrast(panel, treatment, control, family)?;
    let control_exact = family_control_exact(panel, family, control)?;
    let hard_gates_pass = hard_gates.passed();
    let class = if !hard_gates_pass {
        OutcomeClass::InstrumentFailure
    } else if control_exact {
        OutcomeClass::Ceiling
    } else if contrast.d() > 0 {
        OutcomeClass::Gain
    } else {
        OutcomeClass::NoGain
    };
    Ok(FamilyOutcome {
        family,
        class,
        contrast,
        control_exact,
        hard_gates_pass,
    })
}

fn family_control_exact(
    panel: &PanelManifest,
    family: ContrastPopulation,
    control: &[ArmRecord],
) -> Result<bool, String> {
    if control.len() != panel.rows.len() {
        return Err("control record count differs from the panel".into());
    }
    let mut rows = 0usize;
    let mut exact = true;
    for (row, record) in panel.rows.iter().zip(control) {
        if record.row_id != row.id {
            return Err(format!("control row order differs at {}", row.id));
        }
        if row.family != family.row_family() {
            continue;
        }
        rows += 1;
        if !record.correct() {
            exact = false;
        }
    }
    if rows == 0 {
        return Err("family has no panel rows".into());
    }
    Ok(exact)
}

/// Deterministically generated panel. The same tokenizer always produces
/// byte-identical [`PanelManifest::encode`] output.
pub fn build_panel(tokenizer: &ByteBpeTokenizer) -> Result<PanelManifest, String> {
    let pinned = PinnedTokenizer::new(tokenizer)
        .map_err(|error| format!("pinned tokenizer rejected: {error:?}"))?;
    let tokenizer_identity = hex32(&pinned.identity());
    let lexicon = build_lexicon()?;
    let word_for_prime: BTreeMap<u32, &str> = lexicon
        .entries
        .iter()
        .map(|entry| (entry.prime, entry.word.as_str()))
        .collect();

    let mut pending = Vec::with_capacity(PANEL_ROWS);
    for index in 0..CURRENT_ROWS {
        let row_index = pending.len();
        let true_index = index;
        let prompt_index = if index.is_multiple_of(2) {
            true_index
        } else {
            true_index + 1
        };
        pending.push(PendingRow {
            id: format!("current-{index:03}"),
            family: RowFamily::Current,
            prompt: match_prompt(prompt_index, &word_for_prime)?,
            prompt_tokens: Vec::new(),
            address: address_spec(true_index)?,
            version: RequestedVersion::Current,
            random_address: address_spec(PANEL_ROWS + row_index)?,
            kind: PendingKind::Current,
        });
    }
    for index in 0..PREVIOUS_ROWS {
        let row_index = pending.len();
        let true_index = CURRENT_ROWS + index;
        pending.push(PendingRow {
            id: format!("previous-{index:03}"),
            family: RowFamily::Previous,
            prompt: match_prompt(true_index, &word_for_prime)?,
            prompt_tokens: Vec::new(),
            address: address_spec(true_index)?,
            version: RequestedVersion::Previous,
            random_address: address_spec(PANEL_ROWS + row_index)?,
            kind: PendingKind::Previous,
        });
    }
    for index in 0..ABSENT_ROWS {
        let row_index = pending.len();
        let true_index = CURRENT_ROWS + PREVIOUS_ROWS + index;
        pending.push(PendingRow {
            id: format!("abstention-absent-{index:03}"),
            family: RowFamily::Abstention,
            prompt: match_prompt(true_index, &word_for_prime)?,
            prompt_tokens: Vec::new(),
            address: address_spec(true_index)?,
            version: RequestedVersion::Current,
            random_address: address_spec(PANEL_ROWS + row_index)?,
            kind: PendingKind::Absent,
        });
    }
    for index in 0..EVICTED_ROWS {
        let row_index = pending.len();
        let true_index = CURRENT_ROWS + PREVIOUS_ROWS + ABSENT_ROWS + index;
        pending.push(PendingRow {
            id: format!("abstention-evicted-{index:03}"),
            family: RowFamily::Abstention,
            prompt: match_prompt(true_index, &word_for_prime)?,
            prompt_tokens: Vec::new(),
            address: address_spec(true_index)?,
            version: RequestedVersion::Current,
            random_address: address_spec(PANEL_ROWS + row_index)?,
            kind: PendingKind::Evicted,
        });
    }

    for row in &mut pending {
        row.prompt_tokens = tokenizer.encode(&row.prompt);
    }
    let prompt_texts: Vec<String> = pending.iter().map(|row| row.prompt.clone()).collect();
    let prompt_token_lists: Vec<Vec<u32>> = pending
        .iter()
        .map(|row| row.prompt_tokens.clone())
        .collect();

    let mut used_values = BTreeSet::new();
    let mut rows = Vec::with_capacity(PANEL_ROWS);
    for pending_row in pending {
        let PendingRow {
            id,
            family,
            prompt,
            prompt_tokens,
            address,
            version,
            random_address,
            kind,
        } = pending_row;
        let random_value = generate_value(
            &format!("{id}-random"),
            tokenizer,
            &mut used_values,
            &prompt_texts,
            &prompt_token_lists,
        )?;
        let (expected, history) = match kind {
            PendingKind::Current => {
                let value = generate_value(
                    &format!("{id}-value"),
                    tokenizer,
                    &mut used_values,
                    &prompt_texts,
                    &prompt_token_lists,
                )?;
                (ExpectedOutcome::Value(value.clone()), vec![value])
            }
            PendingKind::Previous => {
                let previous = generate_value(
                    &format!("{id}-previous"),
                    tokenizer,
                    &mut used_values,
                    &prompt_texts,
                    &prompt_token_lists,
                )?;
                let current = generate_value(
                    &format!("{id}-current"),
                    tokenizer,
                    &mut used_values,
                    &prompt_texts,
                    &prompt_token_lists,
                )?;
                (
                    ExpectedOutcome::Value(previous.clone()),
                    vec![previous, current],
                )
            }
            PendingKind::Absent => (
                ExpectedOutcome::Abstention(AbstentionKind::Absent),
                Vec::new(),
            ),
            PendingKind::Evicted => {
                let value = generate_value(
                    &format!("{id}-evicted"),
                    tokenizer,
                    &mut used_values,
                    &prompt_texts,
                    &prompt_token_lists,
                )?;
                (
                    ExpectedOutcome::Abstention(AbstentionKind::Evicted),
                    vec![value],
                )
            }
        };
        rows.push(PanelRow {
            id,
            family,
            prompt,
            prompt_tokens,
            address,
            version,
            expected,
            history,
            random: RandomBinding {
                address: random_address,
                value: random_value,
            },
        });
    }

    let (required_slots, _) = panel_slot_plan(&rows)?;
    let mut limits = PanelLimits::default();
    limits.max_slots = required_slots;
    let panel = PanelManifest {
        schema: PANEL_SCHEMA.to_owned(),
        panel_version: PANEL_VERSION,
        seed: PANEL_SEED.to_owned(),
        tokenizer_identity,
        limits,
        lexicon,
        rows,
    };
    validate_panel(&panel, Some(tokenizer))?;
    Ok(panel)
}

/// Structural and (when pinned) token-level manifest validation.
pub fn validate_panel(
    panel: &PanelManifest,
    tokenizer: Option<&ByteBpeTokenizer>,
) -> Result<(), String> {
    if panel.schema != PANEL_SCHEMA {
        return Err(format!("panel schema {:?} differs", panel.schema));
    }
    if panel.panel_version != PANEL_VERSION {
        return Err(format!("panel version {} differs", panel.panel_version));
    }
    if panel.seed != PANEL_SEED {
        return Err(format!("panel seed {:?} differs", panel.seed));
    }
    validate_tokenizer_identity(&panel.tokenizer_identity)?;
    if let Some(tokenizer) = tokenizer {
        let pinned = PinnedTokenizer::new(tokenizer)
            .map_err(|error| format!("pinned tokenizer rejected: {error:?}"))?;
        if hex32(&pinned.identity()) != panel.tokenizer_identity {
            return Err("tokenizer identity differs from the panel".into());
        }
    }
    validate_limits(&panel.limits, panel.rows.len())?;
    validate_lexicon(&panel.lexicon)?;

    if panel.rows.len() != PANEL_ROWS {
        return Err(format!(
            "panel has {} rows, expected {PANEL_ROWS}",
            panel.rows.len()
        ));
    }
    let current = panel
        .rows
        .iter()
        .filter(|row| row.family == RowFamily::Current)
        .count();
    let previous = panel
        .rows
        .iter()
        .filter(|row| row.family == RowFamily::Previous)
        .count();
    let abstention = panel
        .rows
        .iter()
        .filter(|row| row.family == RowFamily::Abstention)
        .count();
    if (current, previous, abstention) != (CURRENT_ROWS, PREVIOUS_ROWS, ABSTENTION_ROWS) {
        return Err(format!(
            "family counts {current}/{previous}/{abstention} differ"
        ));
    }
    let absent = panel
        .rows
        .iter()
        .filter(|row| row.expected_abstention() == Some(AbstentionKind::Absent))
        .count();
    let evicted = panel
        .rows
        .iter()
        .filter(|row| row.expected_abstention() == Some(AbstentionKind::Evicted))
        .count();
    if absent != ABSENT_ROWS || evicted != EVICTED_ROWS {
        return Err(format!(
            "abstention split {absent} absent + {evicted} evicted differs from {ABSENT_ROWS}/{EVICTED_ROWS}"
        ));
    }

    let mut ids = BTreeSet::new();
    let mut true_addresses = BTreeSet::new();
    let mut random_addresses = BTreeSet::new();
    let mut stored_values = BTreeSet::new();
    for row in &panel.rows {
        if row.id.is_empty()
            || row.id.len() > 64
            || !row
                .id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
        {
            return Err(format!("invalid row id {:?}", row.id));
        }
        if !ids.insert(row.id.clone()) {
            return Err(format!("duplicate row id {}", row.id));
        }
        let true_address = row.address.validate(panel.limits.max_key_atoms)?;
        let random_address = row.random.address.validate(panel.limits.max_key_atoms)?;
        if random_address == true_address {
            return Err(format!("{} random address equals the true address", row.id));
        }
        if !true_addresses.insert(true_address) {
            return Err(format!("{} duplicates another true address", row.id));
        }
        if !random_addresses.insert(random_address) {
            return Err(format!("{} duplicates another random address", row.id));
        }

        validate_prompt(row, tokenizer, &panel.limits)?;

        match row.family {
            RowFamily::Current => {
                if row.version != RequestedVersion::Current
                    || row.expected_value().is_none()
                    || row.history.len() != 1
                    || row.expected_value() != row.history.first()
                {
                    return Err(format!("{} current-row shape differs", row.id));
                }
            }
            RowFamily::Previous => {
                if row.version != RequestedVersion::Previous
                    || row.expected_value().is_none()
                    || row.history.len() != 2
                    || row.expected_value() != row.history.first()
                    || row.history[0].text == row.history[1].text
                {
                    return Err(format!("{} previous-row shape differs", row.id));
                }
            }
            RowFamily::Abstention => {
                if row.version != RequestedVersion::Current {
                    return Err(format!("{} abstention version differs", row.id));
                }
                match row.expected_abstention() {
                    Some(AbstentionKind::Absent) if row.history.is_empty() => {}
                    Some(AbstentionKind::Evicted) if row.history.len() == 1 => {}
                    _ => return Err(format!("{} abstention shape differs", row.id)),
                }
            }
        }

        for value in row.history.iter().chain(std::iter::once(&row.random.value)) {
            validate_value(value, tokenizer, &panel.limits)?;
            if !stored_values.insert(value.text.clone()) {
                return Err(format!("{} reuses stored value {}", row.id, value.text));
            }
        }
        if let Some(expected) = row.expected_value() {
            if row.history.first() != Some(expected) {
                return Err(format!("{} expected value differs from history", row.id));
            }
            if row.random.value.text == expected.text {
                return Err(format!("{} random value equals the expected value", row.id));
            }
        }

        let query = form_sieve_query(&panel.lexicon, &row.prompt)?;
        if query.primes().len() > panel.limits.max_key_atoms {
            return Err(format!(
                "{} sieve query has {} atoms, limit {}",
                row.id,
                query.primes().len(),
                panel.limits.max_key_atoms
            ));
        }
    }
    if random_addresses
        .intersection(&true_addresses)
        .next()
        .is_some()
    {
        return Err("random address collides with a true address".into());
    }
    let (required_slots, _) = panel_slot_plan(&panel.rows)?;
    if panel.limits.max_slots != required_slots {
        return Err(format!(
            "max_slots {} differs from required {required_slots}",
            panel.limits.max_slots
        ));
    }
    let leaks = prompt_value_leaks(panel);
    if !leaks.is_empty() {
        return Err(format!(
            "stored values leak into prompts: {}",
            leaks.join(",")
        ));
    }
    Ok(())
}

/// Canonical pretty JSON plus a trailing newline. Encoding does not validate;
/// call [`validate_panel`] before using the bytes as a panel.
pub fn encode_panel(panel: &PanelManifest) -> Result<Vec<u8>, String> {
    let mut bytes =
        serde_json::to_vec_pretty(panel).map_err(|error| format!("panel encode: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// SHA-256 of [`encode_panel`].
pub fn panel_sha256(panel: &PanelManifest) -> Result<String, String> {
    let bytes = encode_panel(panel)?;
    let digest = Sha256::digest(&bytes);
    Ok(hex32(digest.as_slice()))
}

/// Parse and structurally validate a bounded manifest.
pub fn parse_panel(bytes: &[u8]) -> Result<PanelManifest, String> {
    if bytes.len() > HARD_MAX_PANEL_BYTES {
        return Err(format!(
            "panel is {} bytes, limit {HARD_MAX_PANEL_BYTES}",
            bytes.len()
        ));
    }
    let panel: PanelManifest =
        serde_json::from_slice(bytes).map_err(|error| format!("panel parse: {error}"))?;
    validate_panel(&panel, None)?;
    Ok(panel)
}

/// Split a prompt into words, keep lexicon words in order, and form an
/// ordered-pair `PrimeAddress` between adjacent kept words. This is the
/// deterministic sieve address baseline, not address formation evidence.
pub fn form_sieve_query(lexicon: &SieveLexicon, prompt: &str) -> Result<PrimeAddress, String> {
    if prompt.len() > HARD_MAX_PROMPT_BYTES {
        return Err(format!(
            "prompt is {} bytes, limit {HARD_MAX_PROMPT_BYTES}",
            prompt.len()
        ));
    }
    let mut primes = Vec::new();
    for raw in prompt.split_whitespace() {
        let word = raw
            .trim_matches(|character: char| !character.is_ascii_alphanumeric())
            .to_ascii_lowercase();
        if word.is_empty() {
            continue;
        }
        if let Some(prime) = lexicon.lookup(&word) {
            primes.push(prime);
            if primes.len() > HARD_MAX_KEY_ATOMS {
                return Err(format!(
                    "sieve query exceeds {HARD_MAX_KEY_ATOMS} key atoms"
                ));
            }
        }
    }
    if primes.is_empty() {
        return Err("sieve query has no lexicon word".into());
    }
    if primes.len() == 1 {
        return PrimeAddress::new(&primes)
            .map_err(|error| format!("sieve query address: {error:?}"));
    }
    let mut pairs = Vec::with_capacity(primes.len() - 1);
    for pair in primes.windows(2) {
        pairs.push((pair[0], pair[1]));
    }
    PrimeAddress::with_pairs(&primes, &pairs)
        .map_err(|error| format!("sieve query address: {error:?}"))
}

/// Row ids whose any stored value (history, expected or random) appears in any
/// model-visible prompt, by text or by token sequence. This is a superset of
/// the scored expected values.
pub fn prompt_value_leaks(panel: &PanelManifest) -> Vec<String> {
    let mut leaks = BTreeSet::new();
    for row in &panel.rows {
        for value in row_values(row) {
            for prompt_row in &panel.rows {
                if prompt_row.prompt.contains(&value.text)
                    || contains_sequence(&prompt_row.prompt_tokens, &value.tokens)
                {
                    leaks.insert(row.id.clone());
                }
            }
        }
    }
    leaks.into_iter().collect()
}

/// Row ids whose any stored value token sequence appears in `corpus` (TRAIN or
/// DEV). Returns row ids in sorted order.
pub fn corpus_value_leaks(panel: &PanelManifest, corpus: &[u32]) -> Vec<String> {
    let mut leaks = BTreeSet::new();
    for row in &panel.rows {
        if row_values(row)
            .into_iter()
            .any(|value| contains_sequence(corpus, &value.tokens))
        {
            leaks.insert(row.id.clone());
        }
    }
    leaks.into_iter().collect()
}

/// Score a store read for a store arm. Model arms are rejected; callers score
/// them with [`score_model_arm`] from the scorer's own emission. For
/// [`Arm::Sieve`], `lexicon` is required: the deterministic query is formed
/// here and recorded in `formed_query`, independently of the selected slot.
pub fn score_store_read(
    row: &PanelRow,
    arm: Arm,
    outcome: AddressReadOutcome,
    lexicon: Option<&SieveLexicon>,
) -> Result<ArmRecord, String> {
    let source = match arm {
        Arm::TrueAddress | Arm::RandomAddress => EmissionSource::ExactStore,
        Arm::Sieve => EmissionSource::SieveStore,
        Arm::NoMemory | Arm::NoAddress => {
            return Err("model arm cannot be scored from a store read".into());
        }
    };
    match arm {
        Arm::TrueAddress => match (&outcome, row.version) {
            (AddressReadOutcome::Previous { .. }, RequestedVersion::Current) => {
                return Err("Previous outcome for a Current request".into());
            }
            (AddressReadOutcome::Hit { .. }, RequestedVersion::Previous) => {
                return Err("Hit outcome for a Previous request".into());
            }
            _ => {}
        },
        Arm::RandomAddress | Arm::Sieve => {
            if matches!(&outcome, AddressReadOutcome::Previous { .. }) {
                return Err("unexpected Previous outcome".into());
            }
        }
        Arm::NoMemory | Arm::NoAddress => unreachable!(),
    }

    let emitted_text = match &outcome {
        AddressReadOutcome::Hit { bytes, .. } | AddressReadOutcome::Previous { bytes, .. } => Some(
            String::from_utf8(bytes.clone()).map_err(|_| "store hit is not UTF-8".to_string())?,
        ),
        _ => None,
    };
    let (address, version, object_version, whole_object_verified) = match &outcome {
        AddressReadOutcome::Hit {
            address,
            version,
            object_version,
            whole_object_verified,
            ..
        }
        | AddressReadOutcome::Previous {
            address,
            version,
            object_version,
            whole_object_verified,
            ..
        } => (
            Some(AddressSpec::from(address)),
            Some(*version),
            Some(hex32(object_version)),
            *whole_object_verified,
        ),
        AddressReadOutcome::Evicted { address, .. } => {
            (Some(AddressSpec::from(address)), None, None, false)
        }
        _ => (None, None, None, false),
    };
    let abstention = match &outcome {
        AddressReadOutcome::Absent => Some(AbstentionKind::Absent),
        AddressReadOutcome::Evicted { .. } => Some(AbstentionKind::Evicted),
        _ => None,
    };

    let true_address = row.true_address()?;
    let random_address = row.random_address()?;
    let random_spec = AddressSpec::from(&random_address);
    let (formed_query, query_matches_true) = if arm == Arm::Sieve {
        let lexicon =
            lexicon.ok_or_else(|| "sieve scoring requires the sieve lexicon".to_string())?;
        let query = form_sieve_query(lexicon, &row.prompt)?;
        let query_matches_true = query == true_address;
        (Some(AddressSpec::from(&query)), query_matches_true)
    } else {
        (None, false)
    };
    let selected_address_match = match (&outcome, arm) {
        (
            AddressReadOutcome::Hit { address, .. } | AddressReadOutcome::Previous { address, .. },
            Arm::TrueAddress | Arm::Sieve,
        ) => address == &true_address,
        (AddressReadOutcome::Hit { address, .. }, Arm::RandomAddress) => address == &random_address,
        _ => false,
    };
    let value_check_pass = match (&emitted_text, row.expected_value()) {
        (Some(text), Some(expected)) => text == &expected.text,
        _ => false,
    };
    let typed_abstention_pass = arm != Arm::RandomAddress
        && abstention.is_some_and(|kind| row.expected_abstention() == Some(kind));
    let wrong_substitution = emitted_text.is_some() && !value_check_pass;
    let random_binding_pass = arm == Arm::RandomAddress
        && emitted_text.as_deref() == Some(row.random.value.text.as_str())
        && address.as_ref() == Some(&random_spec);
    let sieve_current_fallback =
        arm == Arm::Sieve && row.family == RowFamily::Previous && emitted_text.is_some();

    Ok(ArmRecord {
        row_id: row.id.clone(),
        arm,
        source,
        emitted_text,
        abstention,
        address,
        version,
        object_version,
        whole_object_verified,
        formed_query,
        query_matches_true,
        selected_address_match,
        value_check_pass,
        typed_abstention_pass,
        wrong_substitution,
        sieve_current_fallback,
        random_binding_pass,
    })
}

/// Score a caller-supplied model emission for a model arm. Typed abstention is
/// a store outcome, so a model emission never satisfies it.
pub fn score_model_arm(row: &PanelRow, arm: Arm, text: Option<&str>) -> Result<ArmRecord, String> {
    if !arm.is_model() {
        return Err("store arm needs a store read".into());
    }
    let emitted_text = text
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let value_check_pass = match (&emitted_text, row.expected_value()) {
        (Some(text), Some(expected)) => text == &expected.text,
        _ => false,
    };
    let wrong_substitution = emitted_text.is_some() && !value_check_pass;
    Ok(ArmRecord {
        row_id: row.id.clone(),
        arm,
        source: EmissionSource::Model,
        emitted_text,
        abstention: None,
        address: None,
        version: None,
        object_version: None,
        whole_object_verified: false,
        formed_query: None,
        query_matches_true: false,
        selected_address_match: false,
        value_check_pass,
        typed_abstention_pass: false,
        wrong_substitution,
        sieve_current_fallback: false,
        random_binding_pass: false,
    })
}

/// Decode model bytes without turning an invalid emission into an instrument abort.
pub fn decode_emission_bytes(bytes: &[u8]) -> (String, bool) {
    match String::from_utf8(bytes.to_vec()) {
        Ok(text) => (text, true),
        Err(_) => (String::from_utf8_lossy(bytes).into_owned(), false),
    }
}

/// Count actual emitted tokens for rates, never the expected value's length.
pub fn emission_token_count(
    tokenizer: Option<&ByteBpeTokenizer>,
    record: &ArmRecord,
    model_tokens: usize,
) -> Result<usize, String> {
    if record.arm.is_model() {
        return Ok(model_tokens);
    }
    match record.emitted_text.as_deref() {
        None => Ok(0),
        Some(text) => {
            let tokenizer =
                tokenizer.ok_or_else(|| "store emission needs a tokenizer".to_string())?;
            Ok(tokenizer.encode(text).len())
        }
    }
}

/// No-memory and no-address records must be emission-identical; arm labels are
/// deliberately ignored.
pub fn baseline_records_identical(
    no_memory: &[ArmRecord],
    no_address: &[ArmRecord],
) -> Result<bool, String> {
    if no_memory.len() != no_address.len() {
        return Err("baseline record lengths differ".into());
    }
    if no_memory.iter().any(|record| record.arm != Arm::NoMemory) {
        return Err("no-memory records carry another arm".into());
    }
    if no_address.iter().any(|record| record.arm != Arm::NoAddress) {
        return Err("no-address records carry another arm".into());
    }
    for (left, right) in no_memory.iter().zip(no_address) {
        if left.row_id != right.row_id {
            return Err("baseline row ids differ".into());
        }
        if left.source != right.source
            || left.emitted_text != right.emitted_text
            || left.abstention != right.abstention
            || left.address != right.address
            || left.version != right.version
            || left.object_version != right.object_version
            || left.whole_object_verified != right.whole_object_verified
            || left.formed_query != right.formed_query
            || left.query_matches_true != right.query_matches_true
            || left.selected_address_match != right.selected_address_match
            || left.value_check_pass != right.value_check_pass
            || left.typed_abstention_pass != right.typed_abstention_pass
            || left.wrong_substitution != right.wrong_substitution
            || left.sieve_current_fallback != right.sieve_current_fallback
            || left.random_binding_pass != right.random_binding_pass
        {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Paired `b`/`c` counts for exactly one family. Both record slices must cover
/// every panel row in panel order; `population` selects the one family used in
/// the counts. There is no pooled or multi-family population.
pub fn paired_contrast(
    panel: &PanelManifest,
    treatment: &[ArmRecord],
    control: &[ArmRecord],
    population: ContrastPopulation,
) -> Result<PairedContrast, String> {
    if treatment.len() != panel.rows.len() || control.len() != panel.rows.len() {
        return Err("contrast record count differs from the panel".into());
    }
    let mut result = PairedContrast::default();
    for ((row, treatment_record), control_record) in panel.rows.iter().zip(treatment).zip(control) {
        if treatment_record.row_id != row.id || control_record.row_id != row.id {
            return Err(format!("contrast row order differs at {}", row.id));
        }
        if !population.includes(row.family) {
            continue;
        }
        result.rows += 1;
        match (treatment_record.correct(), control_record.correct()) {
            (true, false) => result.b += 1,
            (false, true) => result.c += 1,
            _ => {}
        }
    }
    Ok(result)
}

fn reject_previous_sieve(population: ContrastPopulation) -> Result<(), String> {
    if population == ContrastPopulation::Previous {
        return Err(
            "read_sieve selects current values only; Previous is not a supported sieve contrast"
                .into(),
        );
    }
    Ok(())
}

/// True-address versus sieve on one family. `Current` and `Abstention` are
/// supported; `Previous` is rejected because the sieve has no version selector.
pub fn true_vs_sieve(
    panel: &PanelManifest,
    true_records: &[ArmRecord],
    sieve_records: &[ArmRecord],
    population: ContrastPopulation,
) -> Result<PairedContrast, String> {
    reject_previous_sieve(population)?;
    ensure_arm(true_records, Arm::TrueAddress)?;
    ensure_arm(sieve_records, Arm::Sieve)?;
    paired_contrast(panel, true_records, sieve_records, population)
}

/// True-address versus no-memory on exactly one family. All three families are
/// supported and are reported separately.
pub fn true_vs_no_memory(
    panel: &PanelManifest,
    true_records: &[ArmRecord],
    no_memory_records: &[ArmRecord],
    population: ContrastPopulation,
) -> Result<PairedContrast, String> {
    ensure_arm(true_records, Arm::TrueAddress)?;
    ensure_arm(no_memory_records, Arm::NoMemory)?;
    paired_contrast(panel, true_records, no_memory_records, population)
}

/// Sieve versus no-memory on one family. `Current` and `Abstention` are
/// supported; `Previous` is rejected.
pub fn sieve_vs_no_memory(
    panel: &PanelManifest,
    sieve_records: &[ArmRecord],
    no_memory_records: &[ArmRecord],
    population: ContrastPopulation,
) -> Result<PairedContrast, String> {
    reject_previous_sieve(population)?;
    ensure_arm(sieve_records, Arm::Sieve)?;
    ensure_arm(no_memory_records, Arm::NoMemory)?;
    paired_contrast(panel, sieve_records, no_memory_records, population)
}

/// Hard true-address gate: exact family bars and zero wrong substitutions.
pub fn hard_gates(
    panel: &PanelManifest,
    true_records: &[ArmRecord],
) -> Result<HardGateReport, String> {
    if true_records.len() != panel.rows.len() {
        return Err("true-address record count differs from the panel".into());
    }
    let mut report = HardGateReport::default();
    for (row, record) in panel.rows.iter().zip(true_records) {
        if record.row_id != row.id {
            return Err(format!("true-address row order differs at {}", row.id));
        }
        if record.arm != Arm::TrueAddress {
            return Err(format!("{} does not carry the true-address arm", row.id));
        }
        report.rows += 1;
        if record.wrong_substitution {
            report.wrong_substitutions += 1;
        }
        match row.family {
            RowFamily::Current => {
                report.current_total += 1;
                if record.correct()
                    && !record.wrong_substitution
                    && record.source == EmissionSource::ExactStore
                    && record.selected_address_match
                    && record.address.as_ref() == Some(&row.address)
                    && record.version == Some(1)
                    && record.object_version.is_some()
                    && record.whole_object_verified
                {
                    report.current_pass += 1;
                }
            }
            RowFamily::Previous => {
                report.previous_total += 1;
                if record.correct()
                    && !record.wrong_substitution
                    && record.source == EmissionSource::ExactStore
                    && record.selected_address_match
                    && record.address.as_ref() == Some(&row.address)
                    && record.version == Some(1)
                    && record.object_version.is_some()
                    && record.whole_object_verified
                {
                    report.previous_pass += 1;
                }
            }
            RowFamily::Abstention => {
                report.abstention_total += 1;
                let pass = record.typed_abstention_pass
                    && record.abstention == row.expected_abstention()
                    && record.emitted_text.is_none()
                    && record.source == EmissionSource::ExactStore
                    && (record.abstention != Some(AbstentionKind::Evicted)
                        || record.address.as_ref() == Some(&row.address));
                if pass {
                    report.abstention_pass += 1;
                }
                match row.expected_abstention() {
                    Some(AbstentionKind::Absent) => {
                        report.absent_total += 1;
                        if pass {
                            report.absent_pass += 1;
                        }
                    }
                    Some(AbstentionKind::Evicted) => {
                        report.evicted_total += 1;
                        if pass {
                            report.evicted_pass += 1;
                        }
                    }
                    None => return Err(format!("{} lacks a typed abstention", row.id)),
                }
            }
        }
    }
    Ok(report)
}

/// Every random-address row must hit its own present key with the planted
/// value and must not emit the row's expected value.
pub fn random_control_holds(panel: &PanelManifest, records: &[ArmRecord]) -> Result<bool, String> {
    if records.len() != panel.rows.len() {
        return Err("random record count differs from the panel".into());
    }
    for (row, record) in panel.rows.iter().zip(records) {
        if record.row_id != row.id {
            return Err(format!("random row order differs at {}", row.id));
        }
        if record.arm != Arm::RandomAddress {
            return Err(format!("{} does not carry the random arm", row.id));
        }
        if !record.random_binding_pass
            || record.address.as_ref() != Some(&row.random.address)
            || !record.selected_address_match
            || record.version != Some(1)
            || record.object_version.is_none()
            || !record.whole_object_verified
        {
            return Ok(false);
        }
        if let Some(expected) = row.expected_value() {
            if record.value_check_pass
                || record.emitted_text.as_deref() == Some(expected.text.as_str())
            {
                return Ok(false);
            }
        } else if record.emitted_text.is_none() {
            return Ok(false);
        }
    }
    Ok(true)
}

/// A built oracle composition over the one [`Store`] and one
/// [`AddressDirectory`]. No second store or token-map exact path exists here.
#[derive(Debug)]
pub struct OracleStore {
    panel: PanelManifest,
    store: Store,
    directory: AddressDirectory,
}

impl OracleStore {
    /// Validate structurally and build. Callers that own the pinned tokenizer
    /// should use [`Self::build_with_tokenizer`] so token bindings are checked.
    pub fn build(panel: &PanelManifest) -> Result<Self, String> {
        validate_panel(panel, None)?;
        Self::build_unchecked(panel)
    }

    /// Validate against the pinned tokenizer, then build.
    pub fn build_with_tokenizer(
        panel: &PanelManifest,
        tokenizer: &ByteBpeTokenizer,
    ) -> Result<Self, String> {
        validate_panel(panel, Some(tokenizer))?;
        Self::build_unchecked(panel)
    }

    fn build_unchecked(panel: &PanelManifest) -> Result<Self, String> {
        let limits = store_limits(&panel.limits)?;
        let mut store = Store::new(limits);
        let mut directory = AddressDirectory::new(panel.limits.max_slots)
            .map_err(|error| format!("address directory: {error:?}"))?;
        let mut folded = BTreeMap::<String, Manifest>::new();

        // Evicted true keys are written first so the capacity ring removes
        // exactly those keys and later writes evict nothing else.
        for row in &panel.rows {
            if row.expected_abstention() == Some(AbstentionKind::Evicted) {
                let value = row
                    .history
                    .first()
                    .ok_or_else(|| format!("{} has no evicted value", row.id))?;
                let manifest = fold_text(&mut store, &mut folded, &value.text)?;
                directory
                    .write(row.true_address()?, ObjectTarget::WholeObject { manifest })
                    .map_err(|error| format!("write {}: {error:?}", row.id))?;
            }
        }
        for row in &panel.rows {
            if row.family == RowFamily::Current {
                let value = row
                    .history
                    .first()
                    .ok_or_else(|| format!("{} has no current value", row.id))?;
                let manifest = fold_text(&mut store, &mut folded, &value.text)?;
                directory
                    .write(row.true_address()?, ObjectTarget::WholeObject { manifest })
                    .map_err(|error| format!("write {}: {error:?}", row.id))?;
            }
        }
        for row in &panel.rows {
            if row.family == RowFamily::Previous {
                if row.history.len() != 2 {
                    return Err(format!("{} previous history differs", row.id));
                }
                for value in &row.history {
                    let manifest = fold_text(&mut store, &mut folded, &value.text)?;
                    directory
                        .write(row.true_address()?, ObjectTarget::WholeObject { manifest })
                        .map_err(|error| format!("write {}: {error:?}", row.id))?;
                }
            }
        }
        for row in &panel.rows {
            let manifest = fold_text(&mut store, &mut folded, &row.random.value.text)?;
            directory
                .write(
                    row.random_address()?,
                    ObjectTarget::WholeObject { manifest },
                )
                .map_err(|error| format!("write random {}: {error:?}", row.id))?;
        }

        let (_, evicted_count) = panel_slot_plan(&panel.rows)?;
        let mut used_addresses = BTreeSet::new();
        let mut used_values = BTreeSet::new();
        for row in &panel.rows {
            used_addresses.insert(row.true_address()?);
            used_addresses.insert(row.random_address()?);
            for value in &row.history {
                used_values.insert(value.text.clone());
            }
            used_values.insert(row.random.value.text.clone());
        }
        for index in 0..evicted_count {
            let address = evictor_address(index, &used_addresses)?;
            let text = evictor_text(
                index,
                &used_values,
                &panel.rows,
                panel.limits.max_value_bytes,
            )?;
            used_addresses.insert(address.clone());
            used_values.insert(text.clone());
            let manifest = fold_text(&mut store, &mut folded, &text)?;
            directory
                .write(address, ObjectTarget::WholeObject { manifest })
                .map_err(|error| format!("write evictor {index}: {error:?}"))?;
        }

        let oracle = Self {
            panel: panel.clone(),
            store,
            directory,
        };
        oracle.verify_built_store()?;
        Ok(oracle)
    }

    /// Panel snapshot used by this composition.
    pub fn panel(&self) -> &PanelManifest {
        &self.panel
    }

    /// The single object store.
    pub fn store(&self) -> &Store {
        &self.store
    }

    /// The single exact/sieve directory.
    pub fn directory(&self) -> &AddressDirectory {
        &self.directory
    }

    /// Exact read at the requested version.
    pub fn read_exact(
        &self,
        address: &PrimeAddress,
        version: RequestedVersion,
    ) -> Result<AddressReadOutcome, String> {
        self.directory
            .read_exact(&self.store, None, address, version.into())
            .map_err(|error| format!("exact read: {error:?}"))
    }

    /// Sorted-merge GCD sieve read.
    pub fn read_sieve(&self, query: &PrimeAddress) -> Result<AddressReadOutcome, String> {
        self.directory
            .read_sieve(&self.store, None, query)
            .map_err(|error| format!("sieve read: {error:?}"))
    }

    /// Read one arm. Model arms return `None` and must be scored from a model
    /// emission; store arms return their read outcome.
    pub fn read_arm(&self, row: &PanelRow, arm: Arm) -> Result<Option<AddressReadOutcome>, String> {
        match arm {
            Arm::NoMemory | Arm::NoAddress => Ok(None),
            Arm::TrueAddress => self.read_exact(&row.true_address()?, row.version).map(Some),
            Arm::RandomAddress => self
                .read_exact(&row.random_address()?, RequestedVersion::Current)
                .map(Some),
            Arm::Sieve => {
                let query = form_sieve_query(&self.panel.lexicon, &row.prompt)?;
                if query.primes().len() > self.panel.limits.max_key_atoms {
                    return Err("sieve query exceeds the panel key-atom limit".into());
                }
                self.read_sieve(&query).map(Some)
            }
        }
    }

    /// Read and score one store arm.
    pub fn observe_store_arm(&self, row: &PanelRow, arm: Arm) -> Result<ArmRecord, String> {
        if arm.is_model() {
            return Err("model arm needs a model emission".into());
        }
        let outcome = self
            .read_arm(row, arm)?
            .ok_or_else(|| "store arm produced no read".to_string())?;
        let lexicon = if arm == Arm::Sieve {
            Some(&self.panel.lexicon)
        } else {
            None
        };
        score_store_read(row, arm, outcome, lexicon)
    }

    fn read_select(
        &self,
        address: &PrimeAddress,
        version: VersionSelect,
        row: &PanelRow,
    ) -> Result<AddressReadOutcome, String> {
        self.directory
            .read_exact(&self.store, None, address, version)
            .map_err(|error| format!("read {}: {error:?}", row.id))
    }

    fn verify_built_store(&self) -> Result<(), String> {
        if self.directory.len() != self.panel.limits.max_slots {
            return Err(format!(
                "directory holds {} of {} slots",
                self.directory.len(),
                self.panel.limits.max_slots
            ));
        }
        for row in &self.panel.rows {
            let true_address = row.true_address()?;
            let random_address = row.random_address()?;
            match (&row.expected, row.version) {
                (ExpectedOutcome::Value(expected), RequestedVersion::Current) => {
                    match self.read_select(&true_address, VersionSelect::Current, row)? {
                        AddressReadOutcome::Hit { address, bytes, .. } => {
                            if address != true_address
                                || bytes.as_slice() != expected.text.as_bytes()
                            {
                                return Err(format!("{} current hit differs", row.id));
                            }
                        }
                        other => {
                            return Err(format!("{} expected current Hit, got {other:?}", row.id));
                        }
                    }
                    if !matches!(
                        self.read_select(&true_address, VersionSelect::Previous, row)?,
                        AddressReadOutcome::Absent
                    ) {
                        return Err(format!("{} unexpectedly has a previous version", row.id));
                    }
                }
                (ExpectedOutcome::Value(expected), RequestedVersion::Previous) => {
                    if row.history.len() != 2 {
                        return Err(format!("{} previous history differs", row.id));
                    }
                    match self.read_select(&true_address, VersionSelect::Previous, row)? {
                        AddressReadOutcome::Previous { address, bytes, .. } => {
                            if address != true_address
                                || bytes.as_slice() != expected.text.as_bytes()
                            {
                                return Err(format!("{} previous hit differs", row.id));
                            }
                        }
                        other => {
                            return Err(format!("{} expected Previous, got {other:?}", row.id));
                        }
                    }
                    match self.read_select(&true_address, VersionSelect::Current, row)? {
                        AddressReadOutcome::Hit { address, bytes, .. } => {
                            if address != true_address
                                || bytes.as_slice() != row.history[1].text.as_bytes()
                            {
                                return Err(format!("{} current fallback differs", row.id));
                            }
                        }
                        other => {
                            return Err(format!("{} expected current Hit, got {other:?}", row.id));
                        }
                    }
                }
                (
                    ExpectedOutcome::Abstention(AbstentionKind::Absent),
                    RequestedVersion::Current,
                ) => {
                    if !matches!(
                        self.read_select(&true_address, VersionSelect::Current, row)?,
                        AddressReadOutcome::Absent
                    ) {
                        return Err(format!("{} expected Absent", row.id));
                    }
                    if !matches!(
                        self.read_select(&true_address, VersionSelect::Previous, row)?,
                        AddressReadOutcome::Absent
                    ) {
                        return Err(format!("{} expected Absent previous", row.id));
                    }
                }
                (
                    ExpectedOutcome::Abstention(AbstentionKind::Evicted),
                    RequestedVersion::Current,
                ) => {
                    match self.read_select(&true_address, VersionSelect::Current, row)? {
                        AddressReadOutcome::Evicted { address, .. } => {
                            if address != true_address {
                                return Err(format!("{} wrong evicted key", row.id));
                            }
                        }
                        other => {
                            return Err(format!("{} expected Evicted, got {other:?}", row.id));
                        }
                    }
                    if !matches!(
                        self.read_select(&true_address, VersionSelect::Previous, row)?,
                        AddressReadOutcome::Evicted { .. }
                    ) {
                        return Err(format!("{} expected Evicted previous", row.id));
                    }
                }
                _ => {
                    return Err(format!("{} version and expected outcome differ", row.id));
                }
            }
            match self.read_select(&random_address, VersionSelect::Current, row)? {
                AddressReadOutcome::Hit { address, bytes, .. } => {
                    if address != random_address
                        || bytes.as_slice() != row.random.value.text.as_bytes()
                    {
                        return Err(format!("{} random binding differs", row.id));
                    }
                }
                other => {
                    return Err(format!("{} expected random Hit, got {other:?}", row.id));
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Internal construction and validation helpers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
enum PendingKind {
    Current,
    Previous,
    Absent,
    Evicted,
}

#[derive(Debug)]
struct PendingRow {
    id: String,
    family: RowFamily,
    prompt: String,
    prompt_tokens: Vec<u32>,
    address: AddressSpec,
    version: RequestedVersion,
    random_address: AddressSpec,
    kind: PendingKind,
}

fn build_lexicon() -> Result<SieveLexicon, String> {
    if PRIME_POOL.len() != LEXICON_WORDS.len() {
        return Err("prime pool and lexicon word counts differ".into());
    }
    let mut entries: Vec<LexiconEntry> = PRIME_POOL
        .iter()
        .copied()
        .zip(LEXICON_WORDS.iter().copied())
        .map(|(prime, word)| LexiconEntry {
            word: word.to_owned(),
            prime,
        })
        .collect();
    entries.sort_by(|left, right| left.word.cmp(&right.word));
    let lexicon = SieveLexicon {
        schema: LEXICON_SCHEMA.to_owned(),
        version: LEXICON_VERSION,
        entries,
    };
    validate_lexicon(&lexicon)?;
    Ok(lexicon)
}

fn validate_lexicon(lexicon: &SieveLexicon) -> Result<(), String> {
    if lexicon.schema != LEXICON_SCHEMA {
        return Err(format!("lexicon schema {:?} differs", lexicon.schema));
    }
    if lexicon.version != LEXICON_VERSION {
        return Err(format!("lexicon version {} differs", lexicon.version));
    }
    if lexicon.entries.is_empty() || lexicon.entries.len() > HARD_MAX_LEXICON_ENTRIES {
        return Err(format!(
            "lexicon has {} entries, limit {HARD_MAX_LEXICON_ENTRIES}",
            lexicon.entries.len()
        ));
    }
    let mut words = BTreeSet::new();
    let mut primes = BTreeSet::new();
    let mut previous: Option<&str> = None;
    for entry in &lexicon.entries {
        if entry.word.is_empty()
            || entry.word.len() > 32
            || !entry.word.bytes().all(|byte| byte.is_ascii_lowercase())
        {
            return Err(format!("invalid lexicon word {:?}", entry.word));
        }
        if let Some(previous_word) = previous {
            if previous_word >= entry.word.as_str() {
                return Err("lexicon entries are not strictly ascending".into());
            }
        }
        previous = Some(entry.word.as_str());
        if !words.insert(entry.word.as_str()) {
            return Err(format!("duplicate lexicon word {}", entry.word));
        }
        if !primes.insert(entry.prime) {
            return Err(format!("duplicate lexicon prime {}", entry.prime));
        }
        PrimeAddress::new(&[entry.prime])
            .map_err(|_| format!("lexicon prime {} is not prime", entry.prime))?;
    }
    Ok(())
}

fn validate_tokenizer_identity(identity: &str) -> Result<(), String> {
    if identity.len() != 64
        || !identity
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("tokenizer identity is not 32 lowercase hex bytes".into());
    }
    Ok(())
}

fn validate_limits(limits: &PanelLimits, rows: usize) -> Result<(), String> {
    let checks = [
        ("max_rows", limits.max_rows, HARD_MAX_ROWS),
        ("max_slots", limits.max_slots, HARD_MAX_SLOTS),
        ("max_key_atoms", limits.max_key_atoms, HARD_MAX_KEY_ATOMS),
        (
            "max_value_tokens",
            limits.max_value_tokens,
            HARD_MAX_VALUE_TOKENS,
        ),
        (
            "max_value_bytes",
            limits.max_value_bytes,
            HARD_MAX_VALUE_BYTES,
        ),
        (
            "max_prompt_tokens",
            limits.max_prompt_tokens,
            HARD_MAX_PROMPT_TOKENS,
        ),
        (
            "max_prompt_bytes",
            limits.max_prompt_bytes,
            HARD_MAX_PROMPT_BYTES,
        ),
    ];
    for (name, value, hard) in checks {
        if value == 0 || value > hard {
            return Err(format!("{name} {value} is outside 1..={hard}"));
        }
    }
    if rows > limits.max_rows {
        return Err(format!("{rows} rows exceed max_rows {}", limits.max_rows));
    }
    Ok(())
}

fn validate_prompt(
    row: &PanelRow,
    tokenizer: Option<&ByteBpeTokenizer>,
    limits: &PanelLimits,
) -> Result<(), String> {
    if row.prompt.is_empty() {
        return Err(format!("{} has an empty prompt", row.id));
    }
    if row.prompt.len() > limits.max_prompt_bytes {
        return Err(format!("{} prompt exceeds max_prompt_bytes", row.id));
    }
    if !row.prompt.is_ascii() {
        return Err(format!("{} prompt is not ASCII", row.id));
    }
    if row.prompt_tokens.is_empty() {
        return Err(format!("{} has no prompt tokens", row.id));
    }
    if row.prompt_tokens.len() > limits.max_prompt_tokens {
        return Err(format!("{} prompt exceeds max_prompt_tokens", row.id));
    }
    if let Some(tokenizer) = tokenizer {
        check_token_ids(tokenizer, &row.prompt_tokens)?;
        if tokenizer.encode(&row.prompt) != row.prompt_tokens {
            return Err(format!(
                "{} prompt tokens differ from the tokenizer",
                row.id
            ));
        }
        if tokenizer.decode_bytes(&row.prompt_tokens) != row.prompt.as_bytes() {
            return Err(format!("{} prompt does not round-trip", row.id));
        }
    }
    Ok(())
}

fn validate_value(
    value: &ValueSpec,
    tokenizer: Option<&ByteBpeTokenizer>,
    limits: &PanelLimits,
) -> Result<(), String> {
    if value.text.is_empty() {
        return Err("value text is empty".into());
    }
    if value.text.len() > limits.max_value_bytes {
        return Err(format!("value {:?} exceeds max_value_bytes", value.text));
    }
    if !value.text.bytes().all(|byte| byte.is_ascii_lowercase()) {
        return Err(format!("value {:?} is not lowercase ASCII", value.text));
    }
    if value.tokens.is_empty() {
        return Err(format!("value {:?} has no tokens", value.text));
    }
    if value.tokens.len() > limits.max_value_tokens {
        return Err(format!("value {:?} exceeds max_value_tokens", value.text));
    }
    if let Some(tokenizer) = tokenizer {
        check_token_ids(tokenizer, &value.tokens)?;
        if tokenizer.encode(&value.text) != value.tokens {
            return Err(format!(
                "value {:?} tokens differ from the tokenizer",
                value.text
            ));
        }
        if tokenizer.decode_bytes(&value.tokens) != value.text.as_bytes() {
            return Err(format!("value {:?} does not round-trip", value.text));
        }
    }
    Ok(())
}

fn check_token_ids(tokenizer: &ByteBpeTokenizer, ids: &[u32]) -> Result<(), String> {
    let vocab = u32::try_from(tokenizer.vocab_size())
        .map_err(|_| "tokenizer vocabulary is too large".to_string())?;
    for &id in ids {
        if id >= vocab || tokenizer.token_byte_len(id).is_none() {
            return Err(format!("invalid token id {id}"));
        }
    }
    Ok(())
}

fn panel_slot_plan(rows: &[PanelRow]) -> Result<(usize, usize), String> {
    let mut present_true = 0usize;
    let mut evicted = 0usize;
    for row in rows {
        match (&row.family, &row.expected) {
            (_, ExpectedOutcome::Abstention(AbstentionKind::Absent)) => {
                if !row.history.is_empty() {
                    return Err(format!("{} absent row has stored history", row.id));
                }
            }
            (_, ExpectedOutcome::Abstention(AbstentionKind::Evicted)) => {
                present_true = present_true
                    .checked_add(1)
                    .ok_or_else(|| "slot count overflow".to_string())?;
                evicted = evicted
                    .checked_add(1)
                    .ok_or_else(|| "slot count overflow".to_string())?;
            }
            (RowFamily::Current | RowFamily::Previous, ExpectedOutcome::Value(_)) => {
                present_true = present_true
                    .checked_add(1)
                    .ok_or_else(|| "slot count overflow".to_string())?;
            }
            _ => return Err(format!("{} family and expected outcome differ", row.id)),
        }
    }
    let required = present_true
        .checked_add(rows.len())
        .ok_or_else(|| "slot count overflow".to_string())?;
    Ok((required, evicted))
}

fn row_values(row: &PanelRow) -> Vec<&ValueSpec> {
    let mut values: Vec<&ValueSpec> = row.history.iter().collect();
    values.push(&row.random.value);
    if let Some(expected) = row.expected_value() {
        values.push(expected);
    }
    values
}

fn contains_sequence(haystack: &[u32], needle: &[u32]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

fn ensure_arm(records: &[ArmRecord], arm: Arm) -> Result<(), String> {
    for record in records {
        if record.arm != arm {
            return Err(format!(
                "record {} carries {:?}, expected {:?}",
                record.row_id, record.arm, arm
            ));
        }
    }
    Ok(())
}

fn prime_pair(index: usize) -> Result<(u32, u32), String> {
    let mut count = 0usize;
    for left in 0..PRIME_POOL.len() {
        for right in (left + 1)..PRIME_POOL.len() {
            if count == index {
                return Ok((PRIME_POOL[left], PRIME_POOL[right]));
            }
            count += 1;
        }
    }
    Err(format!(
        "prime pair index {index} is outside {}",
        pair_count()
    ))
}

fn pair_count() -> usize {
    PRIME_POOL.len() * (PRIME_POOL.len() - 1) / 2
}

fn match_prompt(index: usize, words: &BTreeMap<u32, &str>) -> Result<String, String> {
    let (left, right) = prime_pair(index)?;
    let left = words
        .get(&left)
        .copied()
        .ok_or_else(|| format!("lexicon lacks prime {left}"))?;
    let right = words
        .get(&right)
        .copied()
        .ok_or_else(|| format!("lexicon lacks prime {right}"))?;
    Ok(format!("record {left} and {right}"))
}

fn address_spec(index: usize) -> Result<AddressSpec, String> {
    let (left, right) = prime_pair(index)?;
    let address = PrimeAddress::with_pairs(&[left, right], &[(left, right)])
        .map_err(|error| format!("generated address: {error:?}"))?;
    Ok(AddressSpec::from(&address))
}

fn value_candidate(label: &str, candidate: usize) -> String {
    let digest = Sha256::digest(format!("{PANEL_SEED}|{label}|{candidate}").as_bytes());
    digest
        .iter()
        .take(VALUE_BYTES)
        .map(|byte| VALUE_ALPHABET[usize::from(*byte % VALUE_ALPHABET.len() as u8)] as char)
        .collect()
}

fn generate_value(
    label: &str,
    tokenizer: &ByteBpeTokenizer,
    used: &mut BTreeSet<String>,
    prompts: &[String],
    prompt_tokens: &[Vec<u32>],
) -> Result<ValueSpec, String> {
    for candidate in 0..100_000usize {
        let text = value_candidate(label, candidate);
        if used.contains(&text) {
            continue;
        }
        let tokens = tokenizer.encode(&text);
        if tokens.is_empty() {
            return Err(format!("tokenizer produced no tokens for value {text:?}"));
        }
        if tokenizer.decode_bytes(&tokens) != text.as_bytes() {
            return Err(format!("tokenizer does not round-trip value {text:?}"));
        }
        if prompts.iter().any(|prompt| prompt.contains(&text)) {
            continue;
        }
        if prompt_tokens
            .iter()
            .any(|tokens_in_prompt| contains_sequence(tokens_in_prompt, &tokens))
        {
            continue;
        }
        used.insert(text.clone());
        return Ok(ValueSpec { text, tokens });
    }
    Err(format!("no unique value for {label}"))
}

fn store_limits(limits: &PanelLimits) -> Result<Limits, String> {
    let max_object_bytes = u64::try_from(limits.max_value_bytes)
        .map_err(|_| "value byte limit does not fit u64".to_string())?;
    let max_records = limits
        .max_rows
        .checked_mul(4)
        .ok_or_else(|| "record limit overflow".to_string())?;
    let max_record_bytes = max_records
        .checked_mul(limits.max_value_bytes.saturating_add(64))
        .ok_or_else(|| "record byte limit overflow".to_string())?;
    let max_visits = max_records
        .checked_mul(4)
        .ok_or_else(|| "visit limit overflow".to_string())?;
    let max_occurrences = limits
        .max_rows
        .checked_mul(limits.max_value_tokens)
        .and_then(|value| value.checked_mul(2))
        .ok_or_else(|| "occurrence limit overflow".to_string())?;
    Ok(Limits {
        max_object_bytes,
        max_records,
        max_record_bytes,
        max_returned_bytes: max_object_bytes,
        max_visits,
        max_occurrences,
    })
}

fn fold_text(
    store: &mut Store,
    cache: &mut BTreeMap<String, Manifest>,
    text: &str,
) -> Result<Manifest, String> {
    if let Some(manifest) = cache.get(text) {
        return Ok(manifest.clone());
    }
    let folded = store
        .fold(text.as_bytes())
        .map_err(|error| format!("fold {text:?}: {error:?}"))?;
    cache.insert(text.to_owned(), folded.manifest.clone());
    Ok(folded.manifest)
}

fn evictor_address(index: usize, used: &BTreeSet<PrimeAddress>) -> Result<PrimeAddress, String> {
    let total = pair_count();
    for offset in 0..total {
        let pair_index = (EVICTOR_PAIR_BASE + index + offset) % total;
        let (left, right) = prime_pair(pair_index)?;
        let address = PrimeAddress::with_pairs(&[left, right], &[(left, right)])
            .map_err(|error| format!("evictor address: {error:?}"))?;
        if !used.contains(&address) {
            return Ok(address);
        }
    }
    Err("no free evictor address".into())
}

fn evictor_text(
    index: usize,
    used: &BTreeSet<String>,
    rows: &[PanelRow],
    max_bytes: usize,
) -> Result<String, String> {
    let length = max_bytes.clamp(1, VALUE_BYTES);
    for candidate in 0..100_000usize {
        let text: String = value_candidate(&format!("evictor-{index}"), candidate)
            .chars()
            .take(length)
            .collect();
        if used.contains(&text) {
            continue;
        }
        if rows.iter().any(|row| row.prompt.contains(&text)) {
            continue;
        }
        return Ok(text);
    }
    Err(format!("no unique evictor value for {index}"))
}

fn hex32(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

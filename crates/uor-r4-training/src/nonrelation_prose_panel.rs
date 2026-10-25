//! Pre-registered M3 non-relation prose panel loader and grader (`#2856`).
//!
//! Grades a [`TurnCompiler`] and [`StackStore`] over the frozen 50-turn
//! `data/panels/nonrelation-prose-v1.txt` panel. Every turn in the panel is
//! ordinary non-relation prose (`small_talk`, `opinion`, `nonrelation_question`,
//! `anaphoric`, `statement_near_miss`) that states, updates, or queries none of
//! the 10 `world=v2` schema relations (`COMPILER_LABEL_SCHEMA =
//! "m-world-v2.relations-acts/1"`).
//!
//! All counts and ratios are exact integers (CHARTER §3.1–§3.3).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::sha256_bytes;
use crate::stack_store::{HistoryView, StackStore, StoreRead, Update};
use crate::turn_action::{
    CompiledAction, CompilerIdentity, GroundedSessionError, MemoryEffect, RelationLabel,
    SourceSpan, TurnCompiler,
};

/// Relative path from the repository root to the frozen v1 non-relation prose panel.
pub const NONRELATION_PROSE_PANEL_PATH: &str = "data/panels/nonrelation-prose-v1.txt";

/// Frozen SHA-256 digest of `data/panels/nonrelation-prose-v1.txt`.
pub const NONRELATION_PROSE_PANEL_SHA256: &str =
    "7c29c665f97bae126db0cf82cee1795d2160c2ad35f852f64fffc92b836a3de7";

/// Total number of turns in `nonrelation-prose-v1.txt`.
pub const NONRELATION_PROSE_PANEL_TURNS: usize = 50;

/// Number of turns per split (`in_domain` and `heldout_v2_disjoint`).
pub const NONRELATION_PROSE_TURNS_PER_SPLIT: usize = 25;

/// Number of turns per category across the panel.
pub const NONRELATION_PROSE_TURNS_PER_CATEGORY: usize = 10;

/// The 10 relation names of `world=v2` (`m-world-v2.relations-acts/1`), in ID order `1..=10`.
pub const WORLD_V2_RELATION_NAMES: [&str; 10] = [
    "user_name",
    "pet_name",
    "friend_name",
    "hometown",
    "lucky_number",
    "code_word",
    "job",
    "home",
    "favorite_food",
    "favorite_color",
];

/// Panel split distinguishing in-domain conversational probes from the
/// `world=v2`-disjoint held-out slice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProseSplit {
    InDomain,
    HeldoutV2Disjoint,
}

impl ProseSplit {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "in_domain" => Some(Self::InDomain),
            "heldout_v2_disjoint" => Some(Self::HeldoutV2Disjoint),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::InDomain => "in_domain",
            Self::HeldoutV2Disjoint => "heldout_v2_disjoint",
        }
    }
}

/// The five non-relation prose categories required by `#2856`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProseCategory {
    SmallTalk,
    Opinion,
    NonrelationQuestion,
    Anaphoric,
    StatementNearMiss,
}

impl ProseCategory {
    pub const ALL: [Self; 5] = [
        Self::SmallTalk,
        Self::Opinion,
        Self::NonrelationQuestion,
        Self::Anaphoric,
        Self::StatementNearMiss,
    ];

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "small_talk" => Some(Self::SmallTalk),
            "opinion" => Some(Self::Opinion),
            "nonrelation_question" => Some(Self::NonrelationQuestion),
            "anaphoric" => Some(Self::Anaphoric),
            "statement_near_miss" => Some(Self::StatementNearMiss),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::SmallTalk => "small_talk",
            Self::Opinion => "opinion",
            Self::NonrelationQuestion => "nonrelation_question",
            Self::Anaphoric => "anaphoric",
            Self::StatementNearMiss => "statement_near_miss",
        }
    }
}

/// One frozen row of `nonrelation-prose-v1.txt`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProsePanelRow {
    pub id: String,
    pub split: ProseSplit,
    pub category: ProseCategory,
    pub text: String,
}

/// Loaded and SHA-256-verified non-relation prose panel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProsePanel {
    pub sha256: String,
    pub rows: Vec<ProsePanelRow>,
}

impl ProsePanel {
    /// Load and verify the frozen panel from `<repo_root>/data/panels/nonrelation-prose-v1.txt`.
    pub fn load_frozen(repo_root: &Path) -> Result<Self, GroundedSessionError> {
        let path = repo_root.join(NONRELATION_PROSE_PANEL_PATH);
        let bytes = fs::read(&path)?;
        Self::parse_and_verify(&bytes, NONRELATION_PROSE_PANEL_SHA256)
    }

    /// Verify `bytes` against `expected_sha256` and parse all panel rows.
    pub fn parse_and_verify(
        bytes: &[u8],
        expected_sha256: &str,
    ) -> Result<Self, GroundedSessionError> {
        let actual_sha256 = sha256_bytes(bytes);
        if actual_sha256 != expected_sha256 {
            return Err(GroundedSessionError::Binding(format!(
                "non-relation prose panel SHA-256 mismatch: expected {expected_sha256}, got {actual_sha256}"
            )));
        }
        let content = std::str::from_utf8(bytes).map_err(|_| {
            GroundedSessionError::Input("non-relation prose panel is not valid UTF-8".into())
        })?;
        let mut rows = Vec::new();
        let mut ids = BTreeSet::new();
        let mut by_split: BTreeMap<ProseSplit, usize> = BTreeMap::new();
        let mut by_category: BTreeMap<ProseCategory, usize> = BTreeMap::new();

        for (line_no, raw_line) in content.lines().enumerate() {
            let trimmed = raw_line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = raw_line.splitn(4, '\t').collect();
            if parts.len() != 4 {
                return Err(GroundedSessionError::Input(format!(
                    "panel line {} does not have 4 tab-separated columns",
                    line_no + 1
                )));
            }
            let id = parts[0].trim();
            if id.is_empty() || !ids.insert(id.to_owned()) {
                return Err(GroundedSessionError::Input(format!(
                    "panel line {} has empty or duplicate id `{id}`",
                    line_no + 1
                )));
            }
            let split = ProseSplit::parse(parts[1].trim()).ok_or_else(|| {
                GroundedSessionError::Input(format!(
                    "panel line {} has unknown split `{}`",
                    line_no + 1,
                    parts[1]
                ))
            })?;
            let category = ProseCategory::parse(parts[2].trim()).ok_or_else(|| {
                GroundedSessionError::Input(format!(
                    "panel line {} has unknown category `{}`",
                    line_no + 1,
                    parts[2]
                ))
            })?;
            let text = parts[3].trim();
            if text.is_empty() {
                return Err(GroundedSessionError::Input(format!(
                    "panel line {} has empty turn text",
                    line_no + 1
                )));
            }
            *by_split.entry(split).or_default() += 1;
            *by_category.entry(category).or_default() += 1;
            rows.push(ProsePanelRow {
                id: id.to_owned(),
                split,
                category,
                text: text.to_owned(),
            });
        }

        if rows.len() != NONRELATION_PROSE_PANEL_TURNS {
            return Err(GroundedSessionError::Input(format!(
                "expected {NONRELATION_PROSE_PANEL_TURNS} panel turns, found {}",
                rows.len()
            )));
        }
        for split in [ProseSplit::InDomain, ProseSplit::HeldoutV2Disjoint] {
            let count = by_split.get(&split).copied().unwrap_or(0);
            if count != NONRELATION_PROSE_TURNS_PER_SPLIT {
                return Err(GroundedSessionError::Input(format!(
                    "split `{}` has {count} turns; expected {NONRELATION_PROSE_TURNS_PER_SPLIT}",
                    split.as_str()
                )));
            }
        }
        for category in ProseCategory::ALL {
            let count = by_category.get(&category).copied().unwrap_or(0);
            if count != NONRELATION_PROSE_TURNS_PER_CATEGORY {
                return Err(GroundedSessionError::Input(format!(
                    "category `{}` has {count} turns; expected {NONRELATION_PROSE_TURNS_PER_CATEGORY}",
                    category.as_str()
                )));
            }
        }

        Ok(Self {
            sha256: actual_sha256,
            rows,
        })
    }
}

/// Per-turn grading outcome on a non-relation prose turn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProseTurnVerdict {
    /// Compiler returned `CompiledAction::Unresolved` and the store was untouched.
    Clean,
    /// Compiler returned `CompiledAction::Assert` on a non-relation prose turn.
    FalseAssert {
        relation: u32,
        overwrote_stored_fact: bool,
    },
    /// Compiler returned `CompiledAction::Correct` on a prose turn and mutated a stored fact.
    FalseCorrectOverwrite { relation: u32 },
    /// Compiler returned `CompiledAction::Correct` on a prose turn without mutating a stored fact
    /// (for example when writes are disabled or the target relation slot was empty).
    FalseCorrectNoOverwrite { relation: u32 },
    /// Compiler returned `CompiledAction::QueryCurrent` or `CompiledAction::Query` on a prose turn.
    FalseQuery { relation: u32 },
}

/// Per-turn graded record for a row in `nonrelation-prose-v1.txt`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProseTurnGrade {
    pub id: String,
    pub split: ProseSplit,
    pub category: ProseCategory,
    pub action: CompiledAction,
    pub memory: MemoryEffect,
    pub verdict: ProseTurnVerdict,
    pub overwrote_stored_fact: bool,
}

/// Summary report of grading a [`TurnCompiler`] over the 50-turn non-relation prose panel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProsePanelReport {
    pub panel_sha256: String,
    pub compiler_artifact_sha256: String,
    pub total_turns: usize,
    pub clean_unresolved_count: usize,
    pub false_assert_count: usize,
    pub false_overwrite_count: usize,
    pub false_correct_action_count: usize,
    pub false_query_count: usize,
    pub per_category_false_assert: BTreeMap<ProseCategory, usize>,
    pub per_category_false_overwrite: BTreeMap<ProseCategory, usize>,
    pub per_split_false_assert: BTreeMap<ProseSplit, usize>,
    pub per_split_false_overwrite: BTreeMap<ProseSplit, usize>,
    pub turns: Vec<ProseTurnGrade>,
}

impl ProsePanelReport {
    /// True only when every turn in the panel compiled to `Unresolved` with zero false actions.
    pub fn is_all_clean(&self) -> bool {
        self.clean_unresolved_count == self.total_turns
            && self.false_assert_count == 0
            && self.false_overwrite_count == 0
            && self.false_correct_action_count == 0
            && self.false_query_count == 0
    }

    /// Exact `(numerator, denominator)` rational for the false-assert rate.
    pub fn false_assert_ratio(&self) -> (usize, usize) {
        (self.false_assert_count, self.total_turns)
    }

    /// Exact `(numerator, denominator)` rational for the false-overwrite rate.
    pub fn false_overwrite_ratio(&self) -> (usize, usize) {
        (self.false_overwrite_count, self.total_turns)
    }
}

/// Standard 10-relation `world=v2` label list (`id: 1..=10`).
pub fn world_v2_relation_labels() -> Vec<RelationLabel> {
    WORLD_V2_RELATION_NAMES
        .iter()
        .enumerate()
        .map(|(idx, name)| RelationLabel {
            id: u32::try_from(idx + 1).expect("relation id fits in u32"),
            name: (*name).to_owned(),
        })
        .collect()
}

/// Build a `StackStore` pre-seeded with one baseline fact (`[100 + id]`) at every
/// `world=v2` relation `id in 1..=10` under scope `b"user"` and entity `&[1]`.
pub fn seed_world_v2_store() -> Result<StackStore, GroundedSessionError> {
    let mut store = StackStore::new(1, 16)?;
    for id in 1..=10u32 {
        store.write_from(b"user", &[1], id, &[100 + id], Update::Assert, 1)?;
    }
    Ok(store)
}

fn current_tokens(
    store: &StackStore,
    relation: u32,
) -> Result<Option<Vec<u32>>, GroundedSessionError> {
    match store.read(b"user", &[1], relation, HistoryView::Current)? {
        StoreRead::Found(val) => Ok(Some(val.tokens)),
        StoreRead::Absent | StoreRead::Evicted | StoreRead::NoHistory => Ok(None),
    }
}

fn encode_span_tokens(source: &str, span: SourceSpan) -> Result<Vec<u32>, GroundedSessionError> {
    let slice = source
        .get(span.start..span.end)
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| {
            GroundedSessionError::Span("empty, out-of-range or non-UTF8-boundary span".into())
        })?;
    Ok(slice.bytes().map(|b| u32::from(b) + 1).collect())
}

/// Apply a compiled action to `store` using the session store contract and report
/// both the resulting [`MemoryEffect`] and whether a pre-existing stored fact was overwritten.
pub fn apply_prose_action(
    store: &mut StackStore,
    compiler_identity: &CompilerIdentity,
    source: &str,
    action: &CompiledAction,
    turn: u64,
    write: bool,
) -> Result<(MemoryEffect, bool), GroundedSessionError> {
    if source.trim().is_empty() {
        return Err(GroundedSessionError::Input("empty user turn".into()));
    }
    if let Some((relation, view)) = action.query() {
        if !compiler_identity.relations.iter().any(|r| r.id == relation) {
            return Err(GroundedSessionError::Compiler(
                "predicted relation is outside the saved label schema".into(),
            ));
        }
        return Ok((
            MemoryEffect::Read {
                read: store.read(b"user", &[1], relation, view)?,
            },
            false,
        ));
    }
    let (relation, span, update) = match action {
        CompiledAction::Assert { relation, span } => (*relation, *span, Update::Assert),
        CompiledAction::Correct { relation, span } => (*relation, *span, Update::Correct),
        CompiledAction::Unresolved { reason } => {
            if reason.trim().is_empty() {
                return Err(GroundedSessionError::Compiler(
                    "unresolved prediction has no reason".into(),
                ));
            }
            return Ok((MemoryEffect::Unresolved, false));
        }
        CompiledAction::QueryCurrent { .. } | CompiledAction::Query { .. } => {
            return Err(GroundedSessionError::Compiler(
                "unhandled query action".into(),
            ));
        }
    };
    if !compiler_identity.relations.iter().any(|r| r.id == relation) {
        return Err(GroundedSessionError::Compiler(
            "predicted relation is outside the saved label schema".into(),
        ));
    }
    let value_tokens = encode_span_tokens(source, span)?;
    if !write {
        return Ok((MemoryEffect::WriteDisabled { value_tokens }, false));
    }
    let before = current_tokens(store, relation)?;
    let written = store.write_from(b"user", &[1], relation, &value_tokens, update, turn)?;
    let after = current_tokens(store, relation)?;
    let overwrote_stored_fact = match (&before, &after) {
        (Some(prev), Some(curr)) => {
            prev != curr && (written.action == Update::Correct || written.conflict)
        }
        _ => false,
    };
    Ok((
        MemoryEffect::Write {
            written,
            value_tokens,
        },
        overwrote_stored_fact,
    ))
}

/// Grade `compiler` over every turn in `panel`.
///
/// When `seed_store_per_turn` is `true`, each turn is evaluated against a fresh
/// `StackStore` pre-seeded via [`seed_world_v2_store`] so any `Correct` or conflicting
/// `Assert` that mutates an existing relation fact is detected independently.
pub fn grade_prose_panel<C: TurnCompiler>(
    panel: &ProsePanel,
    compiler: &C,
    seed_store_per_turn: bool,
    write_enabled: bool,
) -> Result<ProsePanelReport, GroundedSessionError> {
    let mut clean_unresolved_count = 0usize;
    let mut false_assert_count = 0usize;
    let mut false_overwrite_count = 0usize;
    let mut false_correct_action_count = 0usize;
    let mut false_query_count = 0usize;

    let mut per_category_false_assert: BTreeMap<ProseCategory, usize> = BTreeMap::new();
    let mut per_category_false_overwrite: BTreeMap<ProseCategory, usize> = BTreeMap::new();
    for category in ProseCategory::ALL {
        per_category_false_assert.insert(category, 0);
        per_category_false_overwrite.insert(category, 0);
    }

    let mut per_split_false_assert: BTreeMap<ProseSplit, usize> = BTreeMap::from([
        (ProseSplit::InDomain, 0),
        (ProseSplit::HeldoutV2Disjoint, 0),
    ]);
    let mut per_split_false_overwrite: BTreeMap<ProseSplit, usize> = BTreeMap::from([
        (ProseSplit::InDomain, 0),
        (ProseSplit::HeldoutV2Disjoint, 0),
    ]);

    let mut turns = Vec::with_capacity(panel.rows.len());

    for (idx, row) in panel.rows.iter().enumerate() {
        let mut store = if seed_store_per_turn {
            seed_world_v2_store()?
        } else {
            StackStore::new(1, 16)?
        };
        let commit_before = store.commit();
        let action = compiler.compile(&row.text)?;
        let turn_no = u64::try_from(idx + 2).expect("turn index fits in u64");
        let (memory, overwrote_stored_fact) = apply_prose_action(
            &mut store,
            compiler.identity(),
            &row.text,
            &action,
            turn_no,
            write_enabled,
        )?;
        let commit_after = store.commit();

        let verdict = match &action {
            CompiledAction::Unresolved { .. } => {
                if memory == MemoryEffect::Unresolved && commit_before == commit_after {
                    clean_unresolved_count += 1;
                }
                ProseTurnVerdict::Clean
            }
            CompiledAction::Assert { relation, .. } => {
                false_assert_count += 1;
                *per_category_false_assert.entry(row.category).or_default() += 1;
                *per_split_false_assert.entry(row.split).or_default() += 1;
                if overwrote_stored_fact {
                    false_overwrite_count += 1;
                    *per_category_false_overwrite
                        .entry(row.category)
                        .or_default() += 1;
                    *per_split_false_overwrite.entry(row.split).or_default() += 1;
                }
                ProseTurnVerdict::FalseAssert {
                    relation: *relation,
                    overwrote_stored_fact,
                }
            }
            CompiledAction::Correct { relation, .. } => {
                false_correct_action_count += 1;
                if overwrote_stored_fact {
                    false_overwrite_count += 1;
                    *per_category_false_overwrite
                        .entry(row.category)
                        .or_default() += 1;
                    *per_split_false_overwrite.entry(row.split).or_default() += 1;
                    ProseTurnVerdict::FalseCorrectOverwrite {
                        relation: *relation,
                    }
                } else {
                    ProseTurnVerdict::FalseCorrectNoOverwrite {
                        relation: *relation,
                    }
                }
            }
            CompiledAction::QueryCurrent { relation } | CompiledAction::Query { relation, .. } => {
                false_query_count += 1;
                ProseTurnVerdict::FalseQuery {
                    relation: *relation,
                }
            }
        };

        turns.push(ProseTurnGrade {
            id: row.id.clone(),
            split: row.split,
            category: row.category,
            action,
            memory,
            verdict,
            overwrote_stored_fact,
        });
    }

    Ok(ProsePanelReport {
        panel_sha256: panel.sha256.clone(),
        compiler_artifact_sha256: compiler.identity().artifact_sha256.clone(),
        total_turns: panel.rows.len(),
        clean_unresolved_count,
        false_assert_count,
        false_overwrite_count,
        false_correct_action_count,
        false_query_count,
        per_category_false_assert,
        per_category_false_overwrite,
        per_split_false_assert,
        per_split_false_overwrite,
        turns,
    })
}

/// Extract normalized lowercase words (alphanumeric and apostrophe) from `text`.
pub fn normalized_words(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_alphanumeric() || ch == '\'' {
            for lower in ch.to_lowercase() {
                current.push(lower);
            }
        } else if !current.is_empty() {
            out.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct StubProseCompiler {
        identity: CompilerIdentity,
        bytes: Vec<u8>,
        overrides: BTreeMap<String, CompiledAction>,
    }

    impl StubProseCompiler {
        fn new(overrides: BTreeMap<String, CompiledAction>) -> Self {
            let bytes = b"stub-prose-compiler-v1".to_vec();
            Self {
                identity: CompilerIdentity {
                    schema: "TEST_ONLY/nonrelation-prose-stub/1".into(),
                    artifact_sha256: sha256_bytes(&bytes),
                    tokenizer_sha256: sha256_bytes(b"stub-tokenizer"),
                    label_schema: "m-world-v2.relations-acts/1".into(),
                    relations: world_v2_relation_labels(),
                    encoder: None,
                },
                bytes,
                overrides,
            }
        }
    }

    impl TurnCompiler for StubProseCompiler {
        fn identity(&self) -> &CompilerIdentity {
            &self.identity
        }

        fn artifact_bytes(&self) -> &[u8] {
            &self.bytes
        }

        fn compile(&self, source: &str) -> Result<CompiledAction, GroundedSessionError> {
            if let Some(action) = self.overrides.get(source) {
                return Ok(action.clone());
            }
            Ok(CompiledAction::Unresolved {
                reason: "non-relation prose turn".into(),
            })
        }
    }

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonical repo root")
    }

    #[test]
    fn frozen_panel_loads_and_matches_sha256_and_category_counts() {
        let panel = ProsePanel::load_frozen(&repo_root()).expect("load frozen panel");
        assert_eq!(panel.sha256, NONRELATION_PROSE_PANEL_SHA256);
        assert_eq!(panel.rows.len(), NONRELATION_PROSE_PANEL_TURNS);

        // Tampered SHA-256 is rejected.
        let bytes = fs::read(repo_root().join(NONRELATION_PROSE_PANEL_PATH)).expect("read panel");
        let err = ProsePanel::parse_and_verify(
            &bytes,
            "0000000000000000000000000000000000000000000000000000000000000000",
        )
        .expect_err("wrong sha256 must fail");
        assert!(err.to_string().contains("SHA-256 mismatch"));
    }

    #[test]
    fn clean_stub_compiler_scores_zero_false_assert_and_zero_false_overwrite() {
        let panel = ProsePanel::load_frozen(&repo_root()).expect("load frozen panel");
        let compiler = StubProseCompiler::new(BTreeMap::new());
        let report = grade_prose_panel(&panel, &compiler, true, true).expect("grade clean panel");

        assert!(report.is_all_clean());
        assert_eq!(report.total_turns, 50);
        assert_eq!(report.clean_unresolved_count, 50);
        assert_eq!(report.false_assert_ratio(), (0, 50));
        assert_eq!(report.false_overwrite_ratio(), (0, 50));
        assert_eq!(report.false_correct_action_count, 0);
        assert_eq!(report.false_query_count, 0);
    }

    #[test]
    fn grader_detects_false_assert_and_false_overwrite_including_issue_1718_relation_4() {
        let panel = ProsePanel::load_frozen(&repo_root()).expect("load frozen panel");
        let turn_small_talk = panel.rows[0].text.clone(); // nrp-001 (in_domain, small_talk)
        let turn_anaphoric_1718 = panel.rows[15].text.clone(); // nrp-016 (in_domain, anaphoric: "Actually, ...")
        let turn_heldout_near_miss = panel.rows[45].text.clone(); // nrp-046 (heldout_v2_disjoint, statement_near_miss)
        let turn_question = panel.rows[10].text.clone(); // nrp-011 (in_domain, nonrelation_question)

        let mut overrides = BTreeMap::new();
        // 1. False ASSERT on relation 7 (`job`) that also conflicts with the seeded store fact.
        overrides.insert(
            turn_small_talk,
            CompiledAction::Assert {
                relation: 7,
                span: SourceSpan { start: 0, end: 5 },
            },
        );
        // 2. Issue #1718 regression case: anaphoric "Actually, ..." turn compiles to CORRECT
        //    on relation 4 (`hometown`) and overwrites the stored fact.
        overrides.insert(
            turn_anaphoric_1718,
            CompiledAction::Correct {
                relation: 4,
                span: SourceSpan { start: 10, end: 16 },
            },
        );
        // 3. Held-out near-miss turn compiles to ASSERT on relation 9 (`favorite_food`).
        overrides.insert(
            turn_heldout_near_miss,
            CompiledAction::Assert {
                relation: 9,
                span: SourceSpan { start: 4, end: 16 },
            },
        );
        // 4. Question turn compiles to QueryCurrent on relation 1 (`user_name`).
        overrides.insert(turn_question, CompiledAction::QueryCurrent { relation: 1 });

        let compiler = StubProseCompiler::new(overrides);

        // With a pre-seeded store and writes enabled:
        // - 2 false asserts (nrp-001 on rel 7, nrp-046 on rel 9)
        // - 1 false correct action (nrp-016 on rel 4)
        // - 3 false overwrites total (both false asserts conflict-overwrite seeded facts, plus
        //   the #1718 false CORRECT on relation 4 overwrites the seeded hometown fact)
        // - 1 false query (nrp-011 on rel 1)
        // - 46 clean unresolved turns
        let seeded_report =
            grade_prose_panel(&panel, &compiler, true, true).expect("grade seeded panel");
        assert!(!seeded_report.is_all_clean());
        assert_eq!(seeded_report.clean_unresolved_count, 46);
        assert_eq!(seeded_report.false_assert_count, 2);
        assert_eq!(seeded_report.false_correct_action_count, 1);
        assert_eq!(seeded_report.false_overwrite_count, 3);
        assert_eq!(seeded_report.false_query_count, 1);
        assert_eq!(
            seeded_report.per_category_false_assert[&ProseCategory::SmallTalk],
            1
        );
        assert_eq!(
            seeded_report.per_category_false_assert[&ProseCategory::StatementNearMiss],
            1
        );
        assert_eq!(
            seeded_report.per_category_false_overwrite[&ProseCategory::Anaphoric],
            1
        );
        assert_eq!(
            seeded_report.per_split_false_assert[&ProseSplit::InDomain],
            1
        );
        assert_eq!(
            seeded_report.per_split_false_assert[&ProseSplit::HeldoutV2Disjoint],
            1
        );
        assert_eq!(
            seeded_report.turns[15].verdict,
            ProseTurnVerdict::FalseCorrectOverwrite { relation: 4 }
        );

        // With writes disabled (`TurnControls { write: false }`), the compiler actions are
        // still flagged as FalseAssert and FalseCorrectNoOverwrite, but 0 store facts are
        // overwritten (`false_overwrite_count == 0`).
        let write_disabled_report =
            grade_prose_panel(&panel, &compiler, true, false).expect("grade write-disabled panel");
        assert_eq!(write_disabled_report.false_assert_count, 2);
        assert_eq!(write_disabled_report.false_correct_action_count, 1);
        assert_eq!(write_disabled_report.false_overwrite_count, 0);
        assert_eq!(
            write_disabled_report.turns[15].verdict,
            ProseTurnVerdict::FalseCorrectNoOverwrite { relation: 4 }
        );
    }

    #[test]
    fn heldout_v2_disjoint_slice_shares_no_four_word_run_with_world_v2_sources() {
        let root = repo_root();
        let panel = ProsePanel::load_frozen(&root).expect("load frozen panel");
        let world_v1 =
            fs::read_to_string(root.join("crates/uor-r4-training/src/milestone_world.rs"))
                .expect("read milestone_world.rs");
        let world_v2 =
            fs::read_to_string(root.join("crates/uor-r4-training/src/milestone_world_v2.rs"))
                .expect("read milestone_world_v2.rs");

        let mut world_fourgrams = BTreeSet::new();
        for src in [&world_v1, &world_v2] {
            let toks = normalized_words(src);
            for window in toks.windows(4) {
                world_fourgrams.insert([
                    window[0].clone(),
                    window[1].clone(),
                    window[2].clone(),
                    window[3].clone(),
                ]);
            }
        }

        let heldout_rows: Vec<&ProsePanelRow> = panel
            .rows
            .iter()
            .filter(|r| r.split == ProseSplit::HeldoutV2Disjoint)
            .collect();
        assert_eq!(heldout_rows.len(), NONRELATION_PROSE_TURNS_PER_SPLIT);

        for row in heldout_rows {
            let toks = normalized_words(&row.text);
            assert!(
                toks.len() >= 4,
                "heldout row {} must have at least 4 words",
                row.id
            );
            for window in toks.windows(4) {
                let gram = [
                    window[0].clone(),
                    window[1].clone(),
                    window[2].clone(),
                    window[3].clone(),
                ];
                assert!(
                    !world_fourgrams.contains(&gram),
                    "heldout row {} leaked 4-word run {:?} from milestone_world*",
                    row.id,
                    gram
                );
            }
        }
    }
}

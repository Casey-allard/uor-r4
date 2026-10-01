//! A learned temporal view head over an unchanged saved relation compiler.
//!
//! The sparse softmax learner is shared with `relation_compiler`. Features
//! are word identities and ordered adjacent word pairs; no word dispatches
//! directly to a view. Only a base-predicted current query is decorated.
//! This is a floating development compiler, not a D11 serving kernel or a
//! claim of geometric advantage, general temporal understanding or dialogue
//! quality. Training rows and base bytes are retained in the new artifact.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uor_r4_core::native_geometric::learner::realtext_support::sha256_hex;

use crate::relation_compiler::{
    f64_bits, word_spans, HeadParts, SavedCompiler, SparseSoftmax, COMPILER_SCHEMA,
};
use crate::stack_grounded_session::{
    CompiledAction, CompilerIdentity, GroundedSessionError, TurnCompiler,
};
use crate::stack_store::HistoryView;
use crate::{invalid, Result};

pub const TEMPORAL_COMPILER_SCHEMA: &str = "uor-r4.temporal-compiler/1";
pub const TEMPORAL_LABEL_SCHEMA: &str = "uor-r4.relations-acts-temporal-views/1";
const FEATURE_SCHEMA: &str = "uor-r4.word-presence-ordered-adjacent-pairs/1";

/// Fixed classifier order, including explicitly labelled unresolved inputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalView {
    Current,
    Initial,
    PreviousAssertion,
    PreviousDistinctValue,
    Unresolved,
}

pub const TEMPORAL_VIEWS: [TemporalView; 5] = [
    TemporalView::Current,
    TemporalView::Initial,
    TemporalView::PreviousAssertion,
    TemporalView::PreviousDistinctValue,
    TemporalView::Unresolved,
];

impl TemporalView {
    fn class(self) -> usize {
        match self {
            Self::Current => 0,
            Self::Initial => 1,
            Self::PreviousAssertion => 2,
            Self::PreviousDistinctValue => 3,
            Self::Unresolved => 4,
        }
    }

    fn action(self, relation: u32) -> CompiledAction {
        match self {
            Self::Current => CompiledAction::QueryCurrent { relation },
            Self::Initial => CompiledAction::Query {
                relation,
                view: HistoryView::Initial,
            },
            Self::PreviousAssertion => CompiledAction::Query {
                relation,
                view: HistoryView::PreviousAssertion,
            },
            Self::PreviousDistinctValue => CompiledAction::Query {
                relation,
                view: HistoryView::PreviousDistinctValue,
            },
            Self::Unresolved => CompiledAction::Unresolved {
                reason: "the temporal head predicts unresolved".into(),
            },
        }
    }
}

/// Explicit supervision. Relation is checked against the frozen base schema
/// and used for scoring only; it is never an inference feature or address.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemporalExample {
    pub text: String,
    pub relation: Option<String>,
    pub view: TemporalView,
    pub source_group: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TemporalSettings {
    pub steps: usize,
    pub rate: f64,
    pub l2: f64,
}

impl Default for TemporalSettings {
    fn default() -> Self {
        Self {
            steps: 400,
            rate: 0.5,
            l2: 1e-4,
        }
    }
}

impl TemporalSettings {
    fn validate(self) -> Result<()> {
        if self.steps == 0
            || !self.rate.is_finite()
            || self.rate <= 0.0
            || !self.l2.is_finite()
            || self.l2 < 0.0
            || !(self.rate * self.l2).is_finite()
            || self.rate * self.l2 > 1.0
        {
            return Err(invalid("temporal settings need positive steps/rate, nonnegative finite l2 and rate*l2 <= 1"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
enum Feature {
    Word(String),
    Pair(String, String),
}

fn features(text: &str) -> BTreeSet<Feature> {
    let words: Vec<String> = word_spans(text).into_iter().map(|span| span.word).collect();
    words
        .iter()
        .cloned()
        .map(Feature::Word)
        .chain(
            words
                .windows(2)
                .map(|pair| Feature::Pair(pair[0].clone(), pair[1].clone())),
        )
        .collect()
}

fn vocabulary(rows: &[TemporalExample]) -> Vec<Feature> {
    rows.iter()
        .flat_map(|row| features(&row.text))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn feature_row(index: &BTreeMap<Feature, usize>, source: &str) -> Vec<(usize, f64)> {
    features(source)
        .iter()
        .filter_map(|feature| index.get(feature).map(|&id| (id, 1.0)))
        .collect()
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TemporalArtifact {
    schema: String,
    label_schema: String,
    feature_schema: String,
    tokenizer_sha256: String,
    base_sha256: String,
    /// Original canonical UTF-8 bytes, not a parsed/re-encoded base object.
    base_artifact: String,
    labels: Vec<TemporalView>,
    features: Vec<Feature>,
    head: HeadParts,
    steps: usize,
    #[serde(with = "f64_bits")]
    rate_l2: Vec<f64>,
    /// Digest of the canonical ordered row array below, including groups.
    rows_sha256: String,
    rows: Vec<TemporalExample>,
    training: Value,
    #[serde(with = "f64_bits")]
    loss_before_after: Vec<f64>,
}

struct TemporalState {
    head: SparseSoftmax,
    index: BTreeMap<Feature, usize>,
    rows: Vec<TemporalExample>,
    loss: [f64; 2],
    rows_sha256: String,
}

#[derive(Clone)]
pub struct TemporalCompiler {
    bytes: Vec<u8>,
    base: SavedCompiler,
    state: Arc<TemporalState>,
    identity: CompilerIdentity,
}

impl TemporalCompiler {
    /// Fit only the new view head. The supplied base artifact is frozen.
    /// The caller admits row/feature/step and wall/storage costs before fit.
    pub fn fit(
        base: SavedCompiler,
        train: &[TemporalExample],
        training: Value,
        settings: TemporalSettings,
    ) -> Result<Self> {
        settings.validate()?;
        validate_rows(&base, train, true)?;
        validate_provenance(&training)?;
        let vocabulary = vocabulary(train);
        let index: BTreeMap<_, _> = vocabulary
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, f)| (f, i))
            .collect();
        let dim = index.len();
        checked_width(dim)?;
        let x: Vec<_> = train
            .iter()
            .map(|row| feature_row(&index, &row.text))
            .collect();
        let y: Vec<_> = train.iter().map(|row| row.view.class()).collect();
        let initial = SparseSoftmax::fit(
            &x,
            &y,
            TEMPORAL_VIEWS.len(),
            dim,
            0,
            settings.rate,
            settings.l2,
        )?;
        let before = mean_loss(&initial, &x, &y)?;
        let head = SparseSoftmax::fit(
            &x,
            &y,
            TEMPORAL_VIEWS.len(),
            dim,
            settings.steps,
            settings.rate,
            settings.l2,
        )?;
        let after = mean_loss(&head, &x, &y)?;
        let artifact = TemporalArtifact {
            schema: TEMPORAL_COMPILER_SCHEMA.into(),
            label_schema: TEMPORAL_LABEL_SCHEMA.into(),
            feature_schema: FEATURE_SCHEMA.into(),
            tokenizer_sha256: base.identity().tokenizer_sha256.clone(),
            base_sha256: sha256_hex(base.bytes()),
            base_artifact: String::from_utf8(base.bytes().to_vec())
                .map_err(|_| invalid("base compiler is not UTF-8"))?,
            labels: TEMPORAL_VIEWS.to_vec(),
            features: vocabulary,
            head: HeadParts::of(&head),
            steps: settings.steps,
            rate_l2: vec![settings.rate, settings.l2],
            rows_sha256: sha256_hex(&serde_json::to_vec(train)?),
            rows: train.to_vec(),
            training,
            loss_before_after: vec![before, after],
        };
        Self::from_bytes(serde_json::to_vec(&artifact)?)
    }

    /// Load a complete canonical artifact, including the exact frozen base.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        let artifact: TemporalArtifact = serde_json::from_slice(&bytes)?;
        if artifact.schema != TEMPORAL_COMPILER_SCHEMA
            || artifact.label_schema != TEMPORAL_LABEL_SCHEMA
            || artifact.feature_schema != FEATURE_SCHEMA
            || artifact.labels != TEMPORAL_VIEWS
            || artifact.rate_l2.len() != 2
            || artifact.loss_before_after.len() != 2
        {
            return Err(invalid(
                "unknown temporal schema, features, labels or settings shape",
            ));
        }
        if artifact.base_sha256 != sha256_hex(artifact.base_artifact.as_bytes()) {
            return Err(invalid("temporal artifact base digest differs"));
        }
        let base = SavedCompiler::from_bytes(artifact.base_artifact.as_bytes().to_vec())?;
        if artifact.tokenizer_sha256 != base.identity().tokenizer_sha256 {
            return Err(invalid("temporal and base tokenizers differ"));
        }
        TemporalSettings {
            steps: artifact.steps,
            rate: artifact.rate_l2[0],
            l2: artifact.rate_l2[1],
        }
        .validate()?;
        validate_rows(&base, &artifact.rows, true)?;
        validate_provenance(&artifact.training)?;
        if artifact.rows_sha256 != sha256_hex(&serde_json::to_vec(&artifact.rows)?)
            || artifact.features != vocabulary(&artifact.rows)
        {
            return Err(invalid(
                "temporal training rows/digest/feature vocabulary disagree",
            ));
        }
        checked_width(artifact.features.len())?;
        if serde_json::to_vec(&artifact)? != bytes {
            return Err(invalid("temporal artifact bytes are not canonical"));
        }
        let head = artifact
            .head
            .head(TEMPORAL_VIEWS.len(), artifact.features.len())?;
        let index: BTreeMap<_, _> = artifact
            .features
            .into_iter()
            .enumerate()
            .map(|(i, feature)| (feature, i))
            .collect();
        let x: Vec<_> = artifact
            .rows
            .iter()
            .map(|row| feature_row(&index, &row.text))
            .collect();
        let y: Vec<_> = artifact.rows.iter().map(|row| row.view.class()).collect();
        let actual_after = mean_loss(&head, &x, &y)?;
        let initial = SparseSoftmax::fit(
            &x,
            &y,
            TEMPORAL_VIEWS.len(),
            index.len(),
            0,
            artifact.rate_l2[0],
            artifact.rate_l2[1],
        )?;
        let expected_before = mean_loss(&initial, &x, &y)?;
        for (recorded, actual) in artifact
            .loss_before_after
            .iter()
            .zip([expected_before, actual_after])
        {
            if !recorded.is_finite()
                || *recorded < 0.0
                || (*recorded - actual).abs() > 1e-12 * actual.abs().max(1.0)
            {
                return Err(invalid(
                    "temporal loss diagnostics disagree with loaded parameters/rows",
                ));
            }
        }
        let identity = CompilerIdentity {
            schema: TEMPORAL_COMPILER_SCHEMA.into(),
            artifact_sha256: sha256_hex(&bytes),
            tokenizer_sha256: base.identity().tokenizer_sha256.clone(),
            label_schema: TEMPORAL_LABEL_SCHEMA.into(),
            relations: base.identity().relations.clone(),
            encoder: base.identity().encoder.clone(),
        };
        Ok(Self {
            bytes,
            base,
            identity,
            state: Arc::new(TemporalState {
                head,
                index,
                rows: artifact.rows,
                loss: [artifact.loss_before_after[0], artifact.loss_before_after[1]],
                rows_sha256: artifact.rows_sha256,
            }),
        })
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn base(&self) -> &SavedCompiler {
        &self.base
    }

    pub fn predict_view(&self, source: &str) -> Result<TemporalView> {
        let row = feature_row(&self.state.index, source);
        checked_probabilities(&self.state.head, &row)?;
        Ok(TEMPORAL_VIEWS[self.state.head.predict(&row)])
    }

    pub fn action(&self, source: &str) -> Result<CompiledAction> {
        let action = self.base.action(source)?;
        match action {
            CompiledAction::QueryCurrent { relation } => {
                Ok(self.predict_view(source)?.action(relation))
            }
            other => Ok(other),
        }
    }

    pub fn diagnostics(&self) -> Value {
        let mut counts = [0usize; 5];
        for row in &self.state.rows {
            counts[row.view.class()] += 1;
        }
        json!({
            "rows": self.state.rows.len(), "features": self.state.index.len(),
            "labels": TEMPORAL_VIEWS, "class_counts": counts,
            "rows_sha256": self.state.rows_sha256,
            "source_groups": groups(&self.state.rows),
            "loss_before": self.state.loss[0], "loss_after": self.state.loss[1],
            "base_sha256": sha256_hex(self.base.bytes()),
            "scope": "source-trained sparse view head; base relation/act/span frozen; construction diagnostics"
        })
    }

    /// Joint score includes every supplied row, including a wrong base act
    /// or relation. View accuracy is explicitly a separate head diagnostic.
    /// Caller supplies the evaluation partition; no automatic split occurs.
    pub fn score(&self, rows: &[TemporalExample]) -> Result<Value> {
        validate_rows(&self.base, rows, false)?;
        let no_source = self.state.head.predict(&[]);
        checked_probabilities(&self.state.head, &[])?;
        let mut scored = Vec::new();
        let (mut view_pass, mut joint_pass, mut base_pass, mut no_source_pass) =
            (0usize, 0usize, 0usize, 0usize);
        for row in rows {
            let base_action = self.base.action(&row.text)?;
            let prediction = self.predict_view(&row.text)?;
            let composite = match base_action {
                CompiledAction::QueryCurrent { relation } => prediction.action(relation),
                ref other => other.clone(),
            };
            let expected = match &row.relation {
                Some(name) => row.view.action(
                    self.base
                        .relation_id(name)
                        .ok_or_else(|| invalid("unknown scoring relation"))?,
                ),
                None => TemporalView::Unresolved.action(0),
            };
            let joint = same_action(&composite, &expected);
            view_pass += usize::from(prediction == row.view);
            joint_pass += usize::from(joint);
            base_pass += usize::from(same_action(&base_action, &expected));
            no_source_pass += usize::from(no_source == row.view.class());
            scored.push(json!({
                "text": row.text, "source_group": row.source_group, "relation": row.relation,
                "view": row.view, "base_action": base_action, "view_prediction": prediction,
                "composite_action": composite, "expected_action": expected, "joint_pass": joint,
            }));
        }
        let train_groups = groups(&self.state.rows);
        let evaluation_groups = groups(rows);
        let training_texts: BTreeSet<_> = self
            .state
            .rows
            .iter()
            .map(|row| row.text.as_str())
            .collect();
        Ok(json!({
            "rows": scored, "of": rows.len(),
            "joint": {"pass": joint_pass, "of": rows.len()},
            "view_head_diagnostic": {"pass": view_pass, "of": rows.len()},
            "base_current_only": {"pass": base_pass, "of": rows.len()},
            "no_source_head_diagnostic": {"pass": no_source_pass, "of": rows.len()},
            "source_group_overlap": evaluation_groups.intersection(&train_groups).collect::<Vec<_>>(),
            "exact_training_text_overlap": rows.iter().filter(|row| training_texts.contains(row.text.as_str())).count(),
            "evaluation_rows_sha256": sha256_hex(&serde_json::to_vec(rows)?),
            "split": "caller-supplied; overlap is disclosed, not removed",
        }))
    }
}

impl TurnCompiler for TemporalCompiler {
    fn identity(&self) -> &CompilerIdentity {
        &self.identity
    }
    fn artifact_bytes(&self) -> &[u8] {
        self.bytes()
    }
    fn compile(&self, source: &str) -> std::result::Result<CompiledAction, GroundedSessionError> {
        self.action(source)
            .map_err(|error| GroundedSessionError::Compiler(error.to_string()))
    }
}

/// Explicit schema dispatch; malformed temporal bytes never fall back to a
/// legacy compiler. Both variants use the same GroundedSession consumer.
#[derive(Clone)]
pub enum GroundedCompiler {
    Legacy(SavedCompiler),
    Temporal(TemporalCompiler),
}

impl GroundedCompiler {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        let header: Value = serde_json::from_slice(&bytes)?;
        match header.get("schema").and_then(Value::as_str) {
            Some(COMPILER_SCHEMA) => Ok(Self::Legacy(SavedCompiler::from_bytes(bytes)?)),
            Some(TEMPORAL_COMPILER_SCHEMA) => {
                Ok(Self::Temporal(TemporalCompiler::from_bytes(bytes)?))
            }
            _ => Err(invalid("unknown grounded compiler schema")),
        }
    }
    pub fn bytes(&self) -> &[u8] {
        match self {
            Self::Legacy(base) => base.bytes(),
            Self::Temporal(compiler) => compiler.bytes(),
        }
    }
    pub fn base(&self) -> &SavedCompiler {
        match self {
            Self::Legacy(base) => base,
            Self::Temporal(compiler) => compiler.base(),
        }
    }
}

impl TurnCompiler for GroundedCompiler {
    fn identity(&self) -> &CompilerIdentity {
        match self {
            Self::Legacy(base) => base.identity(),
            Self::Temporal(compiler) => compiler.identity(),
        }
    }
    fn artifact_bytes(&self) -> &[u8] {
        self.bytes()
    }
    fn compile(&self, source: &str) -> std::result::Result<CompiledAction, GroundedSessionError> {
        match self {
            Self::Legacy(base) => base.compile(source),
            Self::Temporal(compiler) => compiler.compile(source),
        }
    }
}

fn checked_width(dim: usize) -> Result<()> {
    if dim == 0 || TEMPORAL_VIEWS.len().checked_mul(dim).is_none() {
        return Err(invalid(
            "temporal feature vocabulary is empty or overflows head shape",
        ));
    }
    Ok(())
}

fn checked_probabilities(head: &SparseSoftmax, row: &[(usize, f64)]) -> Result<Vec<f64>> {
    let probabilities = head.probabilities(row);
    if probabilities.len() != TEMPORAL_VIEWS.len()
        || probabilities
            .iter()
            .any(|p| !p.is_finite() || *p < 0.0 || *p > 1.0)
    {
        return Err(invalid(
            "temporal classifier produced invalid probabilities",
        ));
    }
    Ok(probabilities)
}

fn mean_loss(head: &SparseSoftmax, rows: &[Vec<(usize, f64)>], labels: &[usize]) -> Result<f64> {
    let mut loss = 0.0;
    for (row, &label) in rows.iter().zip(labels) {
        let p = checked_probabilities(head, row)?[label];
        if p <= 0.0 {
            return Err(invalid("temporal target probability underflowed"));
        }
        loss -= p.ln() / rows.len() as f64;
    }
    if !loss.is_finite() {
        return Err(invalid("temporal loss is non-finite"));
    }
    Ok(loss)
}

fn groups(rows: &[TemporalExample]) -> BTreeSet<&str> {
    rows.iter().map(|row| row.source_group.as_str()).collect()
}

fn validate_rows(base: &SavedCompiler, rows: &[TemporalExample], require_all: bool) -> Result<()> {
    if rows.is_empty() {
        return Err(invalid("temporal data is empty"));
    }
    let mut by_text = BTreeMap::new();
    let mut present = BTreeSet::new();
    for row in rows {
        if row.text.trim().is_empty()
            || row.source_group.trim().is_empty()
            || features(&row.text).is_empty()
        {
            return Err(invalid(
                "temporal rows need nonempty source words and source_group",
            ));
        }
        match (&row.relation, row.view) {
            (None, TemporalView::Unresolved) => {}
            (Some(name), view)
                if view != TemporalView::Unresolved && base.relation_id(name).is_some() => {}
            _ => {
                return Err(invalid(
                    "temporal relation must be a base label, or null only for Unresolved",
                ))
            }
        }
        let target = (&row.relation, row.view);
        if by_text
            .insert(row.text.as_str(), target)
            .is_some_and(|previous| previous != target)
        {
            return Err(invalid(
                "identical temporal source has contradictory labels",
            ));
        }
        present.insert(row.view);
    }
    if require_all && present.len() != TEMPORAL_VIEWS.len() {
        return Err(invalid(
            "temporal fit needs all five explicitly labelled classes",
        ));
    }
    Ok(())
}

fn validate_provenance(value: &Value) -> Result<()> {
    let valid = match value {
        Value::String(_) => true,
        Value::Number(number) => number.is_u64() || number.is_i64(),
        Value::Array(values) => values
            .iter()
            .all(|value| validate_provenance(value).is_ok()),
        Value::Object(values) => values
            .values()
            .all(|value| validate_provenance(value).is_ok()),
        Value::Null | Value::Bool(_) => false,
    };
    if valid {
        Ok(())
    } else {
        Err(invalid(
            "temporal provenance holds integers/strings and their arrays/objects only",
        ))
    }
}

fn same_action(a: &CompiledAction, b: &CompiledAction) -> bool {
    matches!(
        (a, b),
        (
            CompiledAction::Unresolved { .. },
            CompiledAction::Unresolved { .. }
        )
    ) || a == b
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::Device;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::OnceLock;
    use uor_r4_core::report_output;

    use crate::geometric_stack::{
        PointerConfig, ReadScore, StackArch, StackConfig, StackModel, TransportSnap,
    };
    use crate::relation_compiler::{CompilerSettings, Example, NONE};
    use crate::stack_checkpoint::{
        save_checkpoint, sealed_manifest_sha256, CheckpointIdentity, DataIdentity,
    };
    use crate::stack_grounded_session::{
        ContextPolicy, GroundedSession, MemoryEffect, RecallDisposition, SessionLimits,
        SessionScope, TurnControls, TurnOutcome,
    };
    use crate::stack_store::{StackStore, StoreRead};

    const CURRENT: &str = "What is my name now?";
    const INITIAL: &str = "What name did I record first?";
    const PREVIOUS: &str = "What name did I record immediately before the latest record?";
    const DISTINCT: &str = "What different name did I record before the latest recorded value?";
    const UNRESOLVED: &str = "What is my name at an unspecified time?";
    // Synthetic order labels test parameter/feature flow, not natural language.
    const ORDER_A: &str = "What is my name alpha beta?";
    const ORDER_B: &str = "What is my name beta alpha?";

    fn rows() -> Vec<TemporalExample> {
        [
            (CURRENT, TemporalView::Current),
            (INITIAL, TemporalView::Initial),
            (PREVIOUS, TemporalView::PreviousAssertion),
            (DISTINCT, TemporalView::PreviousDistinctValue),
            (UNRESOLVED, TemporalView::Unresolved),
            (ORDER_A, TemporalView::Initial),
            (ORDER_B, TemporalView::PreviousAssertion),
        ]
        .into_iter()
        .map(|(text, view)| TemporalExample {
            text: text.into(),
            relation: (view != TemporalView::Unresolved).then(|| "user_name".into()),
            view,
            source_group: "TEST_ONLY/authored-construction".into(),
        })
        .collect()
    }

    fn templated(template: &str, value: &str, relation: &str, act: &'static str) -> Example {
        Example {
            text: template.replace("{v}", value),
            relation: relation.into(),
            act,
            template: Some(template.into()),
        }
    }

    fn fit_base() -> SavedCompiler {
        let names = ["Sam", "Tam", "Rook", "Vel", "Bram", "Ilo"];
        let towns = ["Hobton", "Marsk", "Pelford", "Quill"];
        let mut examples = Vec::new();
        for name in names {
            examples.push(templated("My name is {v}.", name, "user_name", "assert"));
            examples.push(templated(
                "Actually, my name is {v}.",
                name,
                "user_name",
                "update",
            ));
            examples.push(templated(
                "Call me {v}, please.",
                name,
                "user_name",
                "assert",
            ));
            examples.push(templated("What is my name?", name, "user_name", "query"));
            examples.push(templated("The weather is nice today.", name, NONE, NONE));
        }
        for town in towns {
            examples.push(templated("I grew up in {v}.", town, "hometown", "assert"));
            examples.push(templated(
                "Sorry, I grew up in {v}.",
                town,
                "hometown",
                "update",
            ));
            examples.push(templated("Where am I from?", town, "hometown", "query"));
        }
        for row in rows() {
            examples.push(templated(&row.text, "", "user_name", "query"));
        }
        SavedCompiler::fit(
            &examples,
            &sha256_hex(&tokenizer_bytes()),
            json!({"scope":"TEST_ONLY/base-construction","rows":examples.len()}),
            CompilerSettings::default(),
        )
        .expect("fit base")
    }

    fn fitted() -> &'static TemporalCompiler {
        static MODEL: OnceLock<TemporalCompiler> = OnceLock::new();
        MODEL.get_or_init(|| {
            TemporalCompiler::fit(
                fit_base(),
                &rows(),
                json!({"scope":"TEST_ONLY/view-construction"}),
                TemporalSettings::default(),
            )
            .expect("fit view")
        })
    }

    #[test]
    fn temporal_head_learns_source_order_and_reloads_exactly() {
        let compiler = fitted();
        assert!(compiler.state.loss[1] < compiler.state.loss[0]);
        let reloaded =
            TemporalCompiler::from_bytes(compiler.bytes().to_vec()).expect("independent load");
        assert_eq!(reloaded.bytes(), compiler.bytes());
        assert_eq!(reloaded.identity(), compiler.identity());
        assert_eq!(reloaded.base().bytes(), compiler.base().bytes());
        for row in rows() {
            assert_eq!(
                compiler.predict_view(&row.text).expect("prediction"),
                row.view,
                "{}",
                row.text
            );
            assert_eq!(
                reloaded.compile(&row.text).expect("loaded action"),
                compiler.compile(&row.text).expect("action")
            );
        }
        let a_words: BTreeSet<_> = word_spans(ORDER_A)
            .into_iter()
            .map(|span| span.word)
            .collect();
        let b_words: BTreeSet<_> = word_spans(ORDER_B)
            .into_iter()
            .map(|span| span.word)
            .collect();
        assert_eq!(a_words, b_words);
        assert_ne!(features(ORDER_A), features(ORDER_B));
        let mut relabelled = rows();
        relabelled[5].view = TemporalView::PreviousAssertion;
        relabelled[6].view = TemporalView::Initial;
        let swapped = TemporalCompiler::fit(
            compiler.base().clone(),
            &relabelled,
            json!({"scope":"TEST_ONLY/label-swap"}),
            TemporalSettings::default(),
        )
        .expect("fit swapped labels");
        assert_eq!(
            swapped.predict_view(ORDER_A).expect("swapped"),
            TemporalView::PreviousAssertion
        );
        assert_eq!(
            swapped.predict_view(ORDER_B).expect("swapped"),
            TemporalView::Initial
        );
        assert_eq!(swapped.base().bytes(), compiler.base().bytes());
        let report = compiler.score(&rows()).expect("score");
        assert_eq!(report["view_head_diagnostic"]["pass"], 7);
        assert_eq!(report["joint"]["pass"], 7);
        assert!(
            report["no_source_head_diagnostic"]["pass"]
                .as_u64()
                .expect("count")
                < 7
        );
        assert_eq!(report["exact_training_text_overlap"], 7);
    }

    #[test]
    fn temporal_adapter_preserves_base_actions_and_scores_wrong_gates() {
        let compiler = fitted();
        let legacy =
            GroundedCompiler::from_bytes(compiler.base().bytes().to_vec()).expect("legacy");
        let composite = GroundedCompiler::from_bytes(compiler.bytes().to_vec()).expect("composite");
        assert_eq!(legacy.bytes(), compiler.base().bytes());
        assert_eq!(composite.base().bytes(), legacy.bytes());
        for text in [
            "My name is Aster.",
            "Actually, my name is Beryl.",
            "The weather is nice today.",
        ] {
            let base = legacy.compile(text).expect("base action");
            assert!(!matches!(base, CompiledAction::QueryCurrent { .. }));
            assert_eq!(composite.compile(text).expect("unchanged action"), base);
        }
        assert_eq!(
            composite.compile(CURRENT).expect("current"),
            legacy.compile(CURRENT).expect("base current")
        );
        let wrong_gate = [TemporalExample {
            text: "My name is Aster.".into(),
            relation: Some("user_name".into()),
            view: TemporalView::Current,
            source_group: "TEST_ONLY/wrong-base-gate".into(),
        }];
        let scored = compiler.score(&wrong_gate).expect("score wrong gate");
        assert_eq!(scored["of"], 1);
        assert_eq!(scored["joint"]["of"], 1);
        assert_eq!(scored["joint"]["pass"], 0);
        assert_eq!(scored["rows"][0]["base_action"]["kind"], "assert");
    }

    #[test]
    fn temporal_artifact_and_supervision_reject_inconsistent_inputs() {
        let compiler = fitted();
        let artifact =
            || serde_json::from_slice::<TemporalArtifact>(compiler.bytes()).expect("artifact");
        for case in 0..10 {
            let mut changed = artifact();
            match case {
                0 => changed.schema = "unknown".into(),
                1 => changed.schema = COMPILER_SCHEMA.into(),
                2 => changed.base_sha256 = "00".repeat(32),
                3 => changed.tokenizer_sha256 = "00".repeat(32),
                4 => changed.labels.swap(0, 1),
                5 => changed.rows[0].text.push_str(" changed"),
                6 => changed.features.reverse(),
                7 => {
                    let mut head = serde_json::to_value(&changed.head).expect("head");
                    head["weights"][0] = json!(f64::NAN.to_bits());
                    changed.head = serde_json::from_value(head).expect("head bits");
                }
                8 => {
                    let mut head = serde_json::to_value(&changed.head).expect("head");
                    head["bias"].as_array_mut().expect("bias").pop();
                    changed.head = serde_json::from_value(head).expect("head shape");
                }
                9 => changed.loss_before_after[1] += 1.0,
                _ => unreachable!(),
            }
            assert!(
                GroundedCompiler::from_bytes(
                    serde_json::to_vec(&changed).expect("canonical mutation")
                )
                .is_err(),
                "case {case}"
            );
        }
        assert!(TemporalCompiler::from_bytes(
            serde_json::to_vec_pretty(&artifact()).expect("pretty")
        )
        .is_err());
        for case in 0..5 {
            let mut bad = rows();
            match case {
                0 => {
                    bad.pop();
                    bad.retain(|row| row.view != TemporalView::Unresolved);
                }
                1 => {
                    let mut conflict = bad[0].clone();
                    conflict.view = TemporalView::Initial;
                    bad.push(conflict);
                }
                2 => bad[0].relation = Some("unknown".into()),
                3 => bad[0].source_group.clear(),
                4 => bad[4].relation = Some("user_name".into()),
                _ => unreachable!(),
            }
            assert!(
                TemporalCompiler::fit(
                    compiler.base().clone(),
                    &bad,
                    json!({}),
                    TemporalSettings::default()
                )
                .is_err(),
                "rows case {case}"
            );
        }
        for settings in [
            TemporalSettings {
                steps: 0,
                ..TemporalSettings::default()
            },
            TemporalSettings {
                rate: f64::NAN,
                ..TemporalSettings::default()
            },
            TemporalSettings {
                l2: -1.0,
                ..TemporalSettings::default()
            },
        ] {
            assert!(
                TemporalCompiler::fit(compiler.base().clone(), &rows(), json!({}), settings)
                    .is_err()
            );
        }
        assert!(TemporalCompiler::fit(
            compiler.base().clone(),
            &rows(),
            json!({"rate":0.5}),
            TemporalSettings::default()
        )
        .is_err());
        let mut row = serde_json::to_value(&rows()[0]).expect("row");
        row["extra"] = json!("rejected");
        assert!(serde_json::from_value::<TemporalExample>(row).is_err());
    }

    fn tokenizer_bytes() -> Vec<u8> {
        let mut printable: Vec<u32> = (u32::from(b'!')..=u32::from(b'~')).collect();
        printable.extend(0xA1..=0xAC);
        printable.extend(0xAE..=0xFF);
        let mut vocab = serde_json::Map::new();
        let specials = ["<|bos|>", "<|eos|>", "<|unk|>"];
        for (id, token) in specials.iter().enumerate() {
            vocab.insert((*token).into(), json!(id));
        }
        let mut extra = 0;
        for byte in 0u32..256 {
            let code = if printable.contains(&byte) {
                byte
            } else {
                extra += 1;
                255 + extra
            };
            vocab.insert(
                char::from_u32(code).expect("alphabet").to_string(),
                json!(byte + 3),
            );
        }
        let added: Vec<Value> = specials
            .iter()
            .enumerate()
            .map(|(id, token)| json!({"id":id,"content":token}))
            .collect();
        serde_json::to_vec(&json!({
            "pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},
            "added_tokens":added,"model":{"type":"BPE","vocab":vocab,"merges":[]}
        }))
        .expect("tokenizer")
    }

    fn scratch() -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "uor-temporal-compiler-{}-{nonce}",
            std::process::id()
        ))
    }

    fn checkpoint(root: &Path) -> PathBuf {
        let tokenizer = tokenizer_bytes();
        let training = root.join("test-training-report");
        report_output::claim(&training).expect("claim");
        fs::write(training.join("fixture.json"), b"{\"scope\":\"TEST_ONLY\"}").expect("report");
        report_output::seal(&training).expect("seal");
        let identity = CheckpointIdentity::from_tokenizer(
            &tokenizer,
            vec![DataIdentity {
                label: "TEST_ONLY".into(),
                bytes: 7,
                sha256: sha256_hex(b"fixture"),
            }],
            sealed_manifest_sha256(&training).expect("manifest"),
        )
        .expect("identity");
        let mut model = StackModel::new(
            StackConfig {
                arch: StackArch::Geometric,
                vocab_size: 259,
                width: 8,
                heads: 2,
                mlp_hidden: 16,
                context: 256,
                pattern: "ra".into(),
                read: ReadScore::Lorentz,
                rotation: true,
                seed: 7,
                memory: None,
                select: None,
                pointer: Some(PointerConfig::new(4)),
            },
            &Device::Cpu,
        )
        .expect("model");
        model
            .set_transport_snap(Some(TransportSnap::Icosian))
            .expect("snap");
        let checkpoint = root.join("checkpoint");
        save_checkpoint(
            &checkpoint,
            &model,
            &identity,
            Some(&StackStore::new(41, 8).expect("store")),
        )
        .expect("checkpoint");
        checkpoint
    }

    fn open(checkpoint: &Path, compiler: GroundedCompiler) -> GroundedSession<GroundedCompiler> {
        GroundedSession::from_checkpoint_path(
            checkpoint,
            tokenizer_bytes(),
            compiler,
            SessionScope {
                scope: b"test-user".to_vec(),
                entity: vec![3],
            },
            SessionLimits {
                max_new_tokens: 2,
                max_turns: 48,
                max_source_bytes: 8192,
                max_history_tokens: 8192,
                max_store_records: 16,
                context_policy: ContextPolicy::WholeCompletedTurns,
            },
            &Device::Cpu,
        )
        .expect("open actual session")
    }

    fn read_id(outcome: &TurnOutcome) -> u64 {
        match &outcome.memory {
            MemoryEffect::Read {
                read: StoreRead::Found(value),
            } => value.record,
            other => panic!("expected Found, got {other:?}"),
        }
    }

    #[test]
    fn learned_temporal_compiler_drives_store_emitter_controls_and_fresh_reload() {
        let root = scratch();
        fs::create_dir(&root).expect("root");
        let checkpoint = checkpoint(&root);
        let compiler =
            GroundedCompiler::from_bytes(fitted().bytes().to_vec()).expect("load fitted compiler");
        let mut session = open(&checkpoint, compiler);
        // These call the real emitter; expected record IDs are assertions
        // after execution, never inputs to compilation or generation.
        for (i, source) in [
            "My name is Aster.",
            "Actually, my name is Beryl.",
            "Actually, my name is Cinder.",
            "My name is Cinder.",
        ]
        .into_iter()
        .enumerate()
        {
            let outcome = session.turn(source).expect("actual generated write turn");
            assert!(
                matches!(outcome.memory, MemoryEffect::Write { written, .. } if written.id == i as u64 + 1)
            );
            assert!(!outcome.reply_ids.is_empty());
        }
        for (source, id) in [(CURRENT, 4), (INITIAL, 1), (PREVIOUS, 3), (DISTINCT, 2)] {
            let outcome = session.turn(source).expect("actual temporal query");
            assert_eq!(read_id(&outcome), id, "{source}");
            assert_eq!(outcome.recall, RecallDisposition::Value);
        }
        let unresolved = session.turn(UNRESOLVED).expect("unresolved turn");
        assert!(matches!(
            unresolved.action,
            CompiledAction::Unresolved { .. }
        ));
        assert_eq!(unresolved.memory, MemoryEffect::Unresolved);
        let no_read = session
            .turn_with_controls(
                INITIAL,
                TurnControls {
                    read: false,
                    write: true,
                },
            )
            .expect("NoRead");
        assert_eq!(read_id(&no_read), 1);
        assert_eq!(no_read.recall, RecallDisposition::Disabled);
        let commit = session.store().commit();
        let no_write = session
            .turn_with_controls(
                "Actually, my name is Dover.",
                TurnControls {
                    read: true,
                    write: false,
                },
            )
            .expect("UpdateDisabled");
        assert!(matches!(
            no_write.memory,
            MemoryEffect::WriteDisabled { .. }
        ));
        assert_eq!(session.store().commit(), commit);
        assert_eq!(
            read_id(&session.turn(CURRENT).expect("query unchanged current")),
            4
        );

        let snapshot = root.join("snapshot");
        session
            .save(&snapshot)
            .expect("save compiler/model/store/history");
        let expected = [INITIAL, DISTINCT]
            .into_iter()
            .map(|source| session.turn(source).expect("uninterrupted continuation"))
            .collect::<Vec<_>>();
        let output = root.join("fresh-output.json");
        let status = std::process::Command::new(std::env::current_exe().expect("test executable"))
            .args([
                "--ignored",
                "--exact",
                "temporal_compiler::tests::temporal_fresh_process_child",
            ])
            .env("UOR_TEMPORAL_TEST_SNAPSHOT", &snapshot)
            .env("UOR_TEMPORAL_TEST_OUTPUT", &output)
            .status()
            .expect("fresh child");
        assert!(status.success());
        let actual: Vec<TurnOutcome> =
            serde_json::from_slice(&fs::read(output).expect("child output")).expect("outcomes");
        assert_eq!(actual, expected);
        fs::remove_dir_all(root).expect("clean");
    }

    #[test]
    #[ignore = "invoked by the artifact-bound fresh-process integration test"]
    fn temporal_fresh_process_child() {
        let root = PathBuf::from(std::env::var_os("UOR_TEMPORAL_TEST_SNAPSHOT").expect("snapshot"));
        let output = PathBuf::from(std::env::var_os("UOR_TEMPORAL_TEST_OUTPUT").expect("output"));
        let compiler = GroundedCompiler::from_bytes(
            fs::read(root.join(crate::stack_grounded_session::COMPILER_FILE))
                .expect("compiler bytes"),
        )
        .expect("independently loaded compiler");
        let mut session = GroundedSession::load(&root, compiler, &Device::Cpu)
            .expect("independently loaded session");
        let results = [INITIAL, DISTINCT]
            .into_iter()
            .map(|source| session.turn(source).expect("loaded generated continuation"))
            .collect::<Vec<_>>();
        fs::write(output, serde_json::to_vec(&results).expect("json")).expect("child result");
    }
}

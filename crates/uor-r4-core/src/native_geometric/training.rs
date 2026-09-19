//! Host-only lexical fitting, finite geometry compilation, count estimation,
//! serialization and evaluation. Floating point is confined to compilation
//! of phase increments and log scores, never the session update/read kernel.

use super::*;
use crate::bounded_global_exact_spin_attention::ExactSpinState;
use crate::canonical_lexical_ingestion::{
    canonical_lexical_piece_bytes, validate_h4_binary_icosahedral_closure, OpaqueH4TableIndex,
};
use crate::prime_route_attention::{zeta_phase_delta, PrimeAtom, ZETA_GRID_REVISION};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const SNAPSHOT_SCHEMA: &str = "uor-r4.native-geometric-count-training/1";
const MAX_SERIALIZED_BYTES: usize = 256 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CountRow {
    feature: Feature,
    total: u64,
    targets: BTreeMap<u32, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    schema: String,
    template: Model,
    prior: Vec<u64>,
    rows: Vec<CountRow>,
    progress: TrainingProgress,
}

#[derive(Debug, Clone)]
pub struct Trainer {
    template: Model,
    prior: Vec<u64>,
    rows: BTreeMap<Feature, CountRow>,
    progress: TrainingProgress,
}

fn source_error(error: impl std::fmt::Display) -> Error {
    Error(error.to_string())
}
pub(super) fn receipt(document: &Document) -> DocumentReceipt {
    DocumentReceipt {
        id: document.id.clone(),
        text_cid: format!("blake3:{}", blake3::hash(document.text.as_bytes()).to_hex()),
        bytes: document.text.len(),
    }
}

fn build_codec(
    config: &Config,
    documents: &[Document],
) -> Result<(Vec<Vec<u8>>, Vec<DocumentReceipt>)> {
    if documents.is_empty() {
        return Err(Error("construction corpus is empty".into()));
    }
    let mut ids = BTreeSet::new();
    let mut frequencies = BTreeMap::<Vec<u8>, u64>::new();
    let mut receipts = Vec::new();
    for document in documents {
        if document.id.trim().is_empty() || !ids.insert(document.id.clone()) {
            return Err(Error(
                "construction document IDs must be nonempty and unique".into(),
            ));
        }
        for piece in
            canonical_lexical_piece_bytes(document.text.as_bytes()).map_err(source_error)?
        {
            let count = frequencies.entry(piece).or_default();
            *count = count
                .checked_add(1)
                .ok_or_else(|| Error("codec frequency overflow".into()))?;
        }
        receipts.push(receipt(document));
    }
    if frequencies.is_empty() {
        return Err(Error("construction corpus contains no text".into()));
    }
    let mut ranked: Vec<_> = frequencies.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    ranked.truncate(config.max_lexical_pieces);
    let mut pieces: Vec<_> = ranked.into_iter().map(|(piece, _)| piece).collect();
    pieces.sort();
    receipts.sort_by(|a, b| a.id.cmp(&b.id));
    Ok((pieces, receipts))
}

pub(super) fn exact_sign([a, b]: [i64; 2]) -> i8 {
    let p = i128::from(a) * 2 + i128::from(b);
    let q = i128::from(b);
    if q == 0 {
        return p.signum() as i8;
    }
    if p == 0 {
        return q.signum() as i8;
    }
    if p.signum() == q.signum() {
        return p.signum() as i8;
    }
    let difference = p * p - 5 * q * q;
    (difference.signum() * p.signum()) as i8
}

pub(super) fn geometry(token_count: usize, context_tokens: usize) -> Result<Geometry> {
    let table = validate_h4_binary_icosahedral_closure().map_err(source_error)?;
    let primes =
        crate::corpus_induced_spin_placement::first_primes(token_count).map_err(source_error)?;
    let origin = PrimeAtom::new(2).map_err(source_error)?;
    let mut tokens = Vec::with_capacity(token_count);
    for (token, prime) in primes.into_iter().enumerate() {
        let prime = u32::try_from(prime).map_err(source_error)?;
        let atom = PrimeAtom::new(prime).map_err(source_error)?;
        let mut phases = [0_u16; PHASE_CHANNELS];
        if token != BOS as usize {
            for (channel, phase) in phases.iter_mut().enumerate() {
                let delta = zeta_phase_delta(channel as u16, origin, atom).map_err(source_error)?;
                let turns = delta.to_radians() / (2.0 * std::f64::consts::PI);
                *phase = (libm::round(turns * 65536.0) as i64).rem_euclid(65536) as u16;
            }
        }
        tokens.push(TokenGeometry {
            prime,
            leaf: if token == BOS as usize {
                table.identity_index
            } else {
                (prime % 120) as u16
            },
            phases,
        });
    }
    let mut orientation = Vec::with_capacity(120);
    for offset in 0..120 {
        let index = OpaqueH4TableIndex::from_table_offset(offset, &table)
            .ok_or_else(|| Error("canonical H4 root index unavailable".into()))?;
        let state = ExactSpinState::from_table_index_and_phases(index, 0, 0, &table)
            .map_err(source_error)?;
        let root = state
            .root_coordinate(&table)
            .map_err(source_error)?
            .scaled_zphi_quaternion;
        orientation.push(((exact_sign(root[0]) + 1) as u8) * 3 + (exact_sign(root[1]) + 1) as u8);
    }
    let anchors = super::anchors::compile_anchor_table(&table).map_err(source_error)?;
    let square_offset = (context_tokens as i64) * 4;
    let squares = (-square_offset..=square_offset)
        .map(|value| value * value)
        .collect();
    Ok(Geometry {
        root_cid: table.h4_root_table_kappa,
        product_cid: table.multiplication_table_kappa,
        zeta_grid: format!("{ZETA_GRID_REVISION};channels=0..8;phase=u16-turn;origin-prime=2"),
        identity: table.identity_index,
        row_bases: (0..120).map(|row| row * 120).collect(),
        products: table.multiplication_indices,
        inverses: table.inverse_indices,
        orientation,
        anchors,
        square_offset,
        squares,
        tokens,
    })
}

impl Trainer {
    /// Fit only the lexical codec here. Each subsequent train_documents call
    /// is an explicit additional presentation (epoch) of the named documents.
    /// A caller checkpoint tracks its next document to avoid double-counting.
    pub fn new(config: Config, construction: &[Document]) -> Result<Self> {
        config.validate()?;
        let (lexical_pieces, receipts) = build_codec(&config, construction)?;
        let token_count = LEXICAL_BASE as usize + lexical_pieces.len();
        let mut template = Model {
            historical_version_intent: None,
            current_query_handoff: None,
            historical_query_context: None,
            historical_field_composition: None,
            relation_start_refinement: None,
            writer_role: None,
            current_source: None,
            historical_read: None,
            writer_choice: None,
            field_composition: None,
            writer_lexical: None,
            source_role_refinement: None,
            word_emission: None,
            shared_operator_refinement: None,
            action_emission: None,
            mixed_operators: None,
            composed_output: None,
            instruction_binding: None,
            lexical_emission: None,
            operation_transition: None,
            typed_role_refinement: None,
            relation_writer_refinement: None,
            relation_writer: None,
            dependent_read: None,
            typed_routing: None,
            joint_admission: None,
            relation_start: None,
            relation_start_context: None,
            relation_reverse_spans: None,
            relation_spans: None,
            source_span_context: None,
            source_span: None,
            source_context: None,
            literal_routing_refinement: None,
            typed_roles: None,
            no_read_completion: None,
            typed_literals: None,
            source_routing: None,
            source_routing_refinement: None,
            learned_routing: None,
            geometric_prose_tables: None,
            schema: SCHEMA.into(),
            artifact_cid: String::new(),
            uor_model_address: String::new(),
            geometry: geometry(token_count, config.context_tokens)?,
            config,
            training: TrainingProgress::default(),
            construction: receipts,
            lexical_pieces,
            prior_scores: vec![0; token_count],
            prior_postings: vec![EOS],
            rows: Vec::new(),
            readout: super::mixture::Readout::default(),
            readout_training: Vec::new(),
            values: None,
            completion: None,
            response_entry: None,
            memory_read: None,
        };
        template.refresh_identity()?;
        Ok(Self {
            template,
            prior: vec![0; token_count],
            rows: BTreeMap::new(),
            progress: TrainingProgress::default(),
        })
    }

    pub fn config(&self) -> &Config {
        &self.template.config
    }
    pub fn progress(&self) -> &TrainingProgress {
        &self.progress
    }
    pub fn construction(&self) -> &[DocumentReceipt] {
        &self.template.construction
    }

    /// Count estimation is the learning rule. New feature/target associations
    /// stop at the declared storage ceilings and rejected events are counted.
    /// Existing associations continue learning; no capacity overflow is hidden.
    pub fn train_documents(&mut self, documents: &[Document]) -> Result<TrainingProgress> {
        // Validate the complete batch before any counts change.
        for document in documents {
            let observed = receipt(document);
            if !self.template.construction.contains(&observed) {
                return Err(Error(format!(
                    "document {} differs from the frozen construction corpus",
                    document.id
                )));
            }
        }
        for document in documents {
            let mut stream = self.template.encode(&document.text)?;
            stream.push(EOS);
            let projected = self
                .progress
                .target_positions
                .checked_add(stream.len() as u64)
                .ok_or_else(|| Error("training position count overflow".into()))?;
            if projected > u64::MAX / super::runtime::FEATURE_COUNT as u64 {
                return Err(Error("training feature count would overflow".into()));
            }
            let mut session = self.template.session(Control::Full)?;
            session.observe(&self.template, BOS)?;
            for token in stream {
                self.prior[token as usize] = self.prior[token as usize]
                    .checked_add(1)
                    .ok_or_else(|| Error("training count overflow".into()))?;
                self.progress.target_positions += 1;
                for feature in session.features(&self.template) {
                    self.progress.feature_events += 1;
                    if !self.rows.contains_key(&feature) {
                        if self.rows.len() >= self.template.config.max_rows {
                            self.progress.dropped_feature_events += 1;
                            continue;
                        }
                        self.rows.insert(
                            feature,
                            CountRow {
                                feature,
                                total: 0,
                                targets: BTreeMap::new(),
                            },
                        );
                    }
                    let row = self
                        .rows
                        .get_mut(&feature)
                        .ok_or_else(|| Error("training row insertion failed".into()))?;
                    row.total += 1;
                    if !row.targets.contains_key(&token) {
                        if self.progress.learned_associations
                            >= self.template.config.max_associations
                        {
                            self.progress.dropped_feature_events += 1;
                            continue;
                        }
                        self.progress.learned_associations += 1;
                    }
                    *row.targets.entry(token).or_default() += 1;
                }
                session.observe(&self.template, token)?;
            }
            self.progress.documents_completed += 1;
        }
        self.progress.learned_rows = self.rows.len();
        Ok(self.progress.clone())
    }

    pub fn compile(&self) -> Result<Model> {
        if self.progress.target_positions == 0 {
            return Err(Error("train at least one document before exporting".into()));
        }
        let mut model = self.template.clone();
        let vocab = self.prior.len() as f64 - 1.0;
        let total = self.prior.iter().map(|&count| count as f64).sum::<f64>();
        model.prior_scores = self
            .prior
            .iter()
            .map(|&count| quantized_log((count as f64 + 1.0) / (total + vocab)))
            .collect();
        model.prior_postings = top_counts(
            self.prior
                .iter()
                .enumerate()
                .filter(|(token, _)| *token != BOS as usize)
                .map(|(token, &count)| (token as u32, count)),
            model.config.candidate_limit,
        );
        model.rows = self
            .rows
            .values()
            .map(|row| {
                let denominator = row.total as f64 + vocab;
                ScoreRow {
                    feature: row.feature,
                    default_score: quantized_log(1.0 / denominator),
                    scores: row
                        .targets
                        .iter()
                        .map(|(&token, &count)| TokenScore {
                            token,
                            score: quantized_log((count as f64 + 1.0) / denominator),
                        })
                        .collect(),
                    postings: top_counts(
                        row.targets.iter().map(|(&token, &count)| (token, count)),
                        model.config.postings_per_row,
                    ),
                }
            })
            .collect();
        model.training = self.progress.clone();
        model.refresh_identity()?;
        model.validate()?;
        Ok(model)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let bytes = serde_json::to_vec(&Snapshot {
            schema: SNAPSHOT_SCHEMA.into(),
            template: self.template.clone(),
            prior: self.prior.clone(),
            rows: self.rows.values().cloned().collect(),
            progress: self.progress.clone(),
        })
        .map_err(source_error)?;
        check_envelope_size(&bytes)?;
        Ok(bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        check_envelope_size(bytes)?;
        let snapshot: Snapshot = serde_json::from_slice(bytes).map_err(source_error)?;
        if snapshot.schema != SNAPSHOT_SCHEMA {
            return Err(Error("unsupported training snapshot schema".into()));
        }
        snapshot.template.validate()?;
        if snapshot.prior.len() != snapshot.template.geometry.tokens.len()
            || snapshot.prior[BOS as usize] != 0
            || snapshot.rows.len() > snapshot.template.config.max_rows
            || snapshot
                .rows
                .windows(2)
                .any(|pair| pair[0].feature >= pair[1].feature)
        {
            return Err(Error("training snapshot shape/order mismatch".into()));
        }
        let mut associations = 0_usize;
        for row in &snapshot.rows {
            if !valid_feature(row.feature)
                || row.total > snapshot.progress.target_positions
                || row
                    .targets
                    .keys()
                    .any(|&token| token == BOS || token as usize >= snapshot.prior.len())
                || row.targets.values().any(|&count| count == 0)
                || row
                    .targets
                    .values()
                    .try_fold(0_u64, |sum, &n| sum.checked_add(n))
                    .is_none_or(|sum| sum > row.total)
            {
                return Err(Error(
                    "training snapshot contains an invalid count row".into(),
                ));
            }
            associations = associations
                .checked_add(row.targets.len())
                .ok_or_else(|| Error("association count overflow".into()))?;
        }
        if associations != snapshot.progress.learned_associations
            || associations > snapshot.template.config.max_associations
            || snapshot.progress.learned_rows != snapshot.rows.len()
            || snapshot
                .progress
                .target_positions
                .checked_mul(super::runtime::FEATURE_COUNT as u64)
                != Some(snapshot.progress.feature_events)
            || snapshot.progress.dropped_feature_events > snapshot.progress.feature_events
            || snapshot.progress.documents_completed as u64 > snapshot.progress.target_positions
            || snapshot
                .prior
                .iter()
                .try_fold(0_u64, |sum, &n| sum.checked_add(n))
                != Some(snapshot.progress.target_positions)
        {
            return Err(Error("training snapshot progress/count mismatch".into()));
        }
        Ok(Self {
            template: snapshot.template,
            prior: snapshot.prior,
            rows: snapshot
                .rows
                .into_iter()
                .map(|row| (row.feature, row))
                .collect(),
            progress: snapshot.progress,
        })
    }
}

fn quantized_log(probability: f64) -> i32 {
    libm::round(libm::log(probability) * SCORE_SCALE) as i32
}
fn top_counts(values: impl Iterator<Item = (u32, u64)>, maximum: usize) -> Vec<u32> {
    let mut values: Vec<_> = values.collect();
    values.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    values.truncate(maximum);
    values.into_iter().map(|(token, _)| token).collect()
}

fn check_envelope_size(bytes: &[u8]) -> Result<()> {
    if bytes.len() > MAX_SERIALIZED_BYTES {
        return Err(Error("native artifact/snapshot exceeds 256 MiB".into()));
    }
    Ok(())
}
fn valid_feature(feature: Feature) -> bool {
    match feature.kind {
        0 => feature.value <= u64::from(u32::MAX),
        1 => true,
        2 => feature.value < 120,
        3 => feature.value >> 16 < 120 && feature.value & 65535 < 120,
        4 => feature.value < 9,
        5 => feature.value < 1920,
        6 => feature.value < 120,
        7 => {
            (feature.value >> 32) <= 536_870_912
                && (-1_073_741_824..=1_073_741_824).contains(&(feature.value as u32 as i32))
        }
        8..=15 => feature.value < 16,
        16..=23 => (-8192..=8192).contains(&(feature.value as i64)),
        24..=25 => feature.value < 120,
        _ => false,
    }
}

impl Model {
    pub fn config(&self) -> &Config {
        &self.config
    }
    pub fn training(&self) -> &TrainingProgress {
        &self.training
    }
    pub fn construction(&self) -> &[DocumentReceipt] {
        &self.construction
    }
    pub fn artifact_cid(&self) -> &str {
        &self.artifact_cid
    }
    /// UOR JSON structural address of the typed model body with both identity
    /// fields cleared. Distinct from the byte-content artifact identity.
    pub fn uor_model_address(&self) -> &str {
        &self.uor_model_address
    }
    pub fn anchor_identities(&self) -> (&str, &str) {
        (
            &self.geometry.anchors.icosian_profile_kappa,
            &self.geometry.anchors.icosian_operator_table_kappa,
        )
    }
    pub fn vocabulary_size(&self) -> usize {
        self.geometry.tokens.len()
    }
    pub fn geometry_identities(&self) -> (&str, &str, &str) {
        (
            &self.geometry.root_cid,
            &self.geometry.product_cid,
            &self.geometry.zeta_grid,
        )
    }
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let bytes = serde_json::to_vec(self).map_err(source_error)?;
        check_envelope_size(&bytes)?;
        Ok(bytes)
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        check_envelope_size(bytes)?;
        serde_json::from_slice(bytes).map_err(source_error)
    }
    pub(super) fn refresh_identity(&mut self) -> Result<()> {
        self.artifact_cid.clear();
        self.uor_model_address.clear();
        self.uor_model_address = uor_addr::json::address_blake3(&self.to_bytes()?)
            .map(|outcome| outcome.address.to_string())
            .map_err(|error| Error(format!("UOR model addressing: {error:?}")))?;
        self.artifact_cid = format!("blake3:{}", blake3::hash(&self.to_bytes()?).to_hex());
        Ok(())
    }
    /// Bind retained phrase transport to the complete unchanged learned parent.
    pub fn with_retained_relation_spans(&self) -> Result<Model> {
        // Candidate validation below reconstructs and validates this entire parent.
        if self.relation_spans.is_some()
            || self.source_span_context.is_none()
            || relation::head(self).is_none()
        {
            return Err(Error(
                "retained spans require contextual span and relation parent".into(),
            ));
        }
        let mut model = self.clone();
        model.relation_spans = Some(self.artifact_cid.clone());
        model.refresh_identity()?;
        model.validate()?;
        Ok(model)
    }
    /// Bind endpoint-bounded reverse extents to the unchanged retained-span parent.
    pub fn with_reverse_relation_spans(&self) -> Result<Model> {
        if self.relation_reverse_spans.is_some() || self.relation_spans.is_none() {
            return Err(Error(
                "reverse spans require a retained relation span parent".into(),
            ));
        }
        let mut model = self.clone();
        model.relation_reverse_spans = Some(self.artifact_cid.clone());
        model.refresh_identity()?;
        model.validate()?;
        Ok(model)
    }
    pub(super) fn validate(&self) -> Result<()> {
        self.config.validate()?;
        if let Some(witness) = &self.historical_version_intent {
            return witness.validate(self);
        }
        if let Some(witness) = &self.current_query_handoff {
            return witness.validate(self);
        }
        if let Some(witness) = &self.historical_query_context {
            return witness.validate(self);
        }
        if let Some(witness) = &self.historical_field_composition {
            return witness.validate(self);
        }
        if let Some(witness) = &self.relation_start_refinement {
            return witness.validate(self);
        }
        if let Some(block) = &self.historical_read {
            self.without_historical_read()?;
            block.validate(self)?;
            let mut duplicate = self.clone();
            duplicate.refresh_identity()?;
            if duplicate.artifact_cid != self.artifact_cid
                || duplicate.uor_model_address != self.uor_model_address
            {
                return Err(Error("historical read identity differs".into()));
            }
            return Ok(());
        }
        // The source refinement restores its previous router before validating
        // inner witnesses, including the frozen contextual writer-role parent.
        if let Some(witness) = &self.current_source {
            return witness.validate(self);
        }
        if let Some(witness) = &self.writer_role {
            return witness.validate(self);
        }
        if let Some(witness) = &self.writer_choice {
            return witness.validate(self);
        }
        if let Some(witness) = &self.field_composition {
            return witness.validate(self);
        }
        if let Some(witness) = &self.writer_lexical {
            return witness.validate(self);
        }
        if let Some(witness) = &self.source_role_refinement {
            return witness.validate(self);
        }
        if let Some(witness) = &self.word_emission {
            return witness.validate(self);
        }
        if let Some(witness) = &self.shared_operator_refinement {
            return witness.validate(self);
        }
        if let Some(witness) = &self.action_emission {
            return witness.validate(self);
        }
        if let Some(witness) = &self.mixed_operators {
            return witness.validate(self);
        }
        if let Some(witness) = &self.composed_output {
            return witness.validate(self);
        }
        if let Some(witness) = &self.instruction_binding {
            return witness.validate(self);
        }
        if let Some(block) = &self.lexical_emission {
            return block.validate(self);
        }
        if let Some(block) = &self.operation_transition {
            return block.validate(self);
        }
        if let Some(witness) = &self.typed_role_refinement {
            return witness.validate(self);
        }
        // Restore the complete accepted model before peeling its earlier layers.
        // A writer refinement changes the active writer beneath those layers.
        if let Some(witness) = &self.relation_writer_refinement {
            return witness.validate(self);
        }
        if self.relation_start_context.is_some() && self.relation_start.is_none() {
            return Err(Error(
                "relation start context has no learned selector".into(),
            ));
        }
        if let Some(block) = &self.relation_start {
            if self.relation_reverse_spans.is_none() {
                return Err(Error("relation start reverse parent absent".into()));
            }
            relation_start_training::validate(block, self)?;
            let mut parent = self.clone();
            parent.relation_start = None;
            parent.relation_start_context = None;
            parent.refresh_identity()?;
            if parent.artifact_cid != block.parent_artifact {
                return Err(Error("relation start frozen parent differs".into()));
            }
            parent.validate()?;
            let mut duplicate = self.clone();
            duplicate.refresh_identity()?;
            if duplicate.artifact_cid != self.artifact_cid
                || duplicate.uor_model_address != self.uor_model_address
            {
                return Err(Error("relation start identity differs".into()));
            }
            return Ok(());
        }
        if let Some(parent_cid) = &self.relation_reverse_spans {
            if self.relation_spans.is_none() {
                return Err(Error("reverse span retained parent absent".into()));
            }
            let mut parent = self.clone();
            parent.relation_reverse_spans = None;
            parent.refresh_identity()?;
            if &parent.artifact_cid != parent_cid {
                return Err(Error("reverse span frozen parent differs".into()));
            }
            parent.validate()?;
            let mut duplicate = self.clone();
            duplicate.refresh_identity()?;
            if duplicate.artifact_cid != self.artifact_cid
                || duplicate.uor_model_address != self.uor_model_address
            {
                return Err(Error("reverse span identity differs".into()));
            }
            return Ok(());
        }
        if let Some(parent_cid) = &self.relation_spans {
            if self.source_span_context.is_none() || relation::head(self).is_none() {
                return Err(Error("retained span parent operators absent".into()));
            }
            let mut parent = self.clone();
            parent.relation_spans = None;
            parent.refresh_identity()?;
            if &parent.artifact_cid != parent_cid {
                return Err(Error("retained span frozen parent differs".into()));
            }
            parent.validate()?;
            let mut duplicate = self.clone();
            duplicate.refresh_identity()?;
            if duplicate.artifact_cid != self.artifact_cid
                || duplicate.uor_model_address != self.uor_model_address
            {
                return Err(Error("retained span identity differs".into()));
            }
            return Ok(());
        }
        if self.source_span_context.is_some() && self.source_span.is_none() {
            return Err(Error("source span context without operator".into()));
        }
        if let Some(block) = &self.source_span {
            if self.source_context.is_none()
                || !block.config.role_context_only
                || block.config.learned_features
                    > if self.source_span_context.is_some() {
                        64
                    } else {
                        16
                    }
            {
                return Err(Error("invalid source span parent/config".into()));
            }
            block.validate_shape(
                self,
                2,
                if self.source_span_context.is_some() {
                    3
                } else {
                    1
                },
            )?;
            if let Some(registry) = &self.source_span_context {
                source_span_training::validate_registry(registry)?;
            }
            let known_prime = |prime: u64| {
                self.source_span_context.as_ref().is_some_and(|registry| {
                    registry.iter().any(|word| u64::from(word.prime) == prime)
                })
            };
            if block.codes.iter().any(|c| match c.feature.kind {
                0 => c.feature.a > 255 || c.feature.b != 0,
                1 => c.feature.a == 0 || !known_prime(c.feature.a) || c.feature.b != 0,
                2 => {
                    c.feature.a == 0
                        || !known_prime(c.feature.a)
                        || (c.feature.b != 0 && !known_prime(c.feature.b))
                }
                _ => true,
            }) {
                return Err(Error(
                    "invalid source span separator or source prime".into(),
                ));
            }
            let mut parent = self.clone();
            parent.source_span = None;
            parent.source_span_context = None;
            parent.refresh_identity()?;
            if parent.artifact_cid != block.parent_artifact {
                return Err(Error("source span frozen parent differs".into()));
            }
            parent.validate()?;
            let mut duplicate = self.clone();
            duplicate.refresh_identity()?;
            if duplicate.artifact_cid != self.artifact_cid
                || duplicate.uor_model_address != self.uor_model_address
            {
                return Err(Error("source span identity differs".into()));
            }
            return Ok(());
        }
        if let Some(witness) = &self.source_context {
            let block = self
                .source_routing
                .as_ref()
                .ok_or_else(|| Error("source context router absent".into()))?;
            if self.literal_routing_refinement.is_none()
                || block.parent_artifact != witness.parent_artifact
                || block.config.role_context_only != witness.previous.config.role_context_only
                || witness.previous.codes.iter().any(|old| {
                    block
                        .codes
                        .binary_search_by_key(&old.feature, |c| c.feature)
                        .is_err()
                })
            {
                return Err(Error("invalid retained source context support".into()));
            }
            block.validate(self)?;
            let mut parent = self.clone();
            parent.source_context = None;
            parent.source_routing = Some(witness.previous.clone());
            parent.refresh_identity()?;
            if parent.artifact_cid != witness.parent_artifact {
                return Err(Error("source context frozen parent differs".into()));
            }
            parent.validate()?;
            let mut duplicate = self.clone();
            duplicate.refresh_identity()?;
            if duplicate.artifact_cid != self.artifact_cid
                || duplicate.uor_model_address != self.uor_model_address
            {
                return Err(Error("source context identity differs".into()));
            }
            return Ok(());
        }
        if let Some(witness) = &self.literal_routing_refinement {
            let block = self
                .typed_literals
                .as_ref()
                .ok_or_else(|| Error("literal refinement component absent".into()))?;
            if self.joint_admission.is_none()
                || block.router.parent_artifact != witness.previous.parent_artifact
                || block.router.config.role_context_only
                    != witness.previous.config.role_context_only
                || witness.previous.codes.iter().any(|old| {
                    block
                        .router
                        .codes
                        .binary_search_by_key(&old.feature, |c| c.feature)
                        .is_err()
                })
            {
                return Err(Error("invalid literal refinement support".into()));
            }
            let mut parent = self.clone();
            parent.literal_routing_refinement = None;
            parent
                .typed_literals
                .as_mut()
                .ok_or_else(|| Error("literal refinement absent".into()))?
                .router = witness.previous.clone();
            parent.refresh_identity()?;
            if parent.artifact_cid != witness.parent_artifact {
                return Err(Error("literal refinement parent differs".into()));
            }
            parent.validate()?;
            block.validate(self, true)?;
            let mut duplicate = self.clone();
            duplicate.refresh_identity()?;
            if duplicate.artifact_cid != self.artifact_cid
                || duplicate.uor_model_address != self.uor_model_address
            {
                return Err(Error("literal refinement identity differs".into()));
            }
            return Ok(());
        }
        if let Some(gate) = &self.joint_admission {
            if self.typed_literals.is_none() || self.source_routing.is_none() {
                return Err(Error(
                    "joint admission requires literal/source components".into(),
                ));
            }
            let mut parent = self.clone();
            parent.joint_admission = None;
            parent.refresh_identity()?;
            if parent.artifact_cid != gate.router.parent_artifact {
                return Err(Error("joint admission frozen parent differs".into()));
            }
            parent.validate()?;
            gate.router.validate_shape(self, 2, 8)?;
            if gate.router.config.role_context_only {
                return Err(Error("invalid admission feature mode".into()));
            }
            let mut duplicate = self.clone();
            duplicate.refresh_identity()?;
            if duplicate.artifact_cid != self.artifact_cid
                || duplicate.uor_model_address != self.uor_model_address
            {
                return Err(Error("joint admission identity differs".into()));
            }
            return Ok(());
        }
        if let Some(witness) = &self.source_routing_refinement {
            let block = self
                .source_routing
                .as_ref()
                .ok_or_else(|| Error("source refinement router absent".into()))?;
            if self.learned_routing.is_some()
                || block.parent_artifact != witness.parent_artifact
                || block.config.role_context_only != witness.previous.config.role_context_only
                || block.codes.len() < witness.previous.codes.len()
                || witness.previous.codes.iter().any(|old| {
                    block
                        .codes
                        .binary_search_by_key(&old.feature, |c| c.feature)
                        .is_err()
                })
            {
                return Err(Error("invalid source refinement composition".into()));
            }
            // The witness is load-time provenance, never another executing head.
            // Restoring the only replaced component must recover the entire
            // exact training parent, including every descendant's original CID.
            let mut parent = self.clone();
            parent.source_routing_refinement = None;
            parent.source_routing = Some(witness.previous.clone());
            parent.refresh_identity()?;
            if parent.artifact_cid != witness.parent_artifact {
                return Err(Error("source refinement frozen parent differs".into()));
            }
            parent.validate()?;
            block.validate(self)?;
            let mut duplicate = self.clone();
            duplicate.refresh_identity()?;
            if duplicate.artifact_cid != self.artifact_cid
                || duplicate.uor_model_address != self.uor_model_address
            {
                return Err(Error("source refinement identity differs".into()));
            }
            return Ok(());
        }
        if let Some(head) = &self.no_read_completion {
            if head.copy.is_some() || super::role_read::head(self).is_none() {
                return Err(Error(
                    "NoRead completion requires the committed role reader".into(),
                ));
            }
            head.validate_shape(
                self,
                33,
                super::response_entry_types::NO_READ_COMPLETION_SCHEMA,
            )?;
            let mut parent = self.clone();
            parent.no_read_completion = None;
            parent.refresh_identity()?;
            if parent.artifact_cid != head.baseline_artifact {
                return Err(Error("NoRead completion parent differs".into()));
            }
            parent.validate()?;
            let mut duplicate = self.clone();
            duplicate.refresh_identity()?;
            if duplicate.artifact_cid != self.artifact_cid
                || duplicate.uor_model_address != self.uor_model_address
            {
                return Err(Error("NoRead completion identity differs".into()));
            }
            return Ok(());
        }
        if let Some(block) = &self.typed_literals {
            if !block.literal_answers
                || self
                    .typed_roles
                    .as_ref()
                    .is_none_or(|r| r.literal_answers || !r.local_query)
            {
                return Err(Error(
                    "literal routing requires protected local computed roles".into(),
                ));
            }
            let mut parent = self.clone();
            parent.typed_literals = None;
            parent.refresh_identity()?;
            if parent.artifact_cid != block.router.parent_artifact {
                return Err(Error("literal routing parent differs".into()));
            }
            parent.validate()?;
            block.validate(self, true)?;
            let mut duplicate = self.clone();
            duplicate.refresh_identity()?;
            if duplicate.artifact_cid != self.artifact_cid
                || duplicate.uor_model_address != self.uor_model_address
            {
                return Err(Error("literal routing identity differs".into()));
            }
            return Ok(());
        }
        if let Some(block) = &self.typed_roles {
            let mut parent = self.clone();
            parent.typed_roles = None;
            parent.refresh_identity()?;
            if parent.artifact_cid != block.router.parent_artifact {
                return Err(Error("typed routing parent differs".into()));
            }
            parent.validate()?;
            block.validate(self, true)?;
            let mut duplicate = self.clone();
            duplicate.refresh_identity()?;
            if duplicate.artifact_cid != self.artifact_cid
                || duplicate.uor_model_address != self.uor_model_address
            {
                return Err(Error("typed routing identity differs".into()));
            }
            return Ok(());
        }
        if let Some(block) = &self.typed_routing {
            let mut parent = self.clone();
            parent.typed_routing = None;
            parent.refresh_identity()?;
            if parent.artifact_cid != block.router.parent_artifact {
                return Err(Error("typed routing parent differs".into()));
            }
            parent.validate()?;
            block.validate(self, false)?;
            let mut duplicate = self.clone();
            duplicate.refresh_identity()?;
            if duplicate.artifact_cid != self.artifact_cid
                || duplicate.uor_model_address != self.uor_model_address
            {
                return Err(Error("typed routing identity differs".into()));
            }
            return Ok(());
        }
        if let Some(writer) = &self.relation_writer {
            let mut parent = self.clone();
            parent.relation_writer = None;
            parent.refresh_identity()?;
            if parent.artifact_cid != writer.parent {
                return Err(Error("writer revision parent differs".into()));
            }
            parent.validate()?;
            writer.validate(self)?;
            let mut duplicate = self.clone();
            duplicate.refresh_identity()?;
            if duplicate.artifact_cid != self.artifact_cid
                || duplicate.uor_model_address != self.uor_model_address
            {
                return Err(Error("writer revision identity differs".into()));
            }
            return Ok(());
        }
        if let Some(block) = &self.dependent_read {
            let mut parent = self.clone();
            parent.dependent_read = None;
            parent.refresh_identity()?;
            if parent.artifact_cid != block.router.parent_artifact {
                return Err(Error("dependent read parent differs".into()));
            }
            parent.validate()?;
            block.validate(self)?;
            let mut duplicate = self.clone();
            duplicate.refresh_identity()?;
            if duplicate.artifact_cid != self.artifact_cid
                || duplicate.uor_model_address != self.uor_model_address
            {
                return Err(Error("dependent read identity differs".into()));
            }
            return Ok(());
        }
        // The routing residual is fitted last. Inherited response heads remain
        // bound to the exact model on which they were trained, including all
        // of their original nested provenance checks.
        if self.source_routing.is_some() && self.learned_routing.is_some() {
            return Err(Error(
                "source and residual routing cannot be stacked in this version".into(),
            ));
        }
        let parent_id = self
            .learned_routing
            .as_ref()
            .map(|b| b.parent_artifact.as_str())
            .or_else(|| {
                self.source_routing
                    .as_ref()
                    .map(|b| b.parent_artifact.as_str())
            });
        let inherited = if let Some(parent_id) = parent_id {
            let mut parent = self.clone();
            parent.learned_routing = None;
            parent.source_routing = None;
            parent.refresh_identity()?;
            if parent.artifact_cid != parent_id {
                return Err(Error(
                    "learned routing frozen parent identity differs".into(),
                ));
            }
            Some(parent)
        } else {
            None
        };
        let parent = inherited.as_ref().unwrap_or(self);
        if let Some(values) = &self.values {
            values.validate()?;
        }
        if let Some(completion) = &self.completion {
            completion.validate(parent)?;
        }
        if let Some(entry) = &self.response_entry {
            entry.validate(parent)?;
        }
        self.readout.validate(parent)?;
        if let Some(memory) = &self.memory_read {
            memory.validate(parent)?;
        }
        if self.schema != SCHEMA
            || self.lexical_pieces.is_empty()
            || self.lexical_pieces.len() > self.config.max_lexical_pieces
            || self.lexical_pieces.windows(2).any(|p| p[0] >= p[1])
            || self
                .lexical_pieces
                .iter()
                .any(|p| p.is_empty() || std::str::from_utf8(p).is_err())
            || self.prior_scores.len() != LEXICAL_BASE as usize + self.lexical_pieces.len()
            || self.rows.len() > self.config.max_rows
            || self.rows.windows(2).any(|p| p[0].feature >= p[1].feature)
            || self.prior_postings.is_empty()
            || self.prior_postings.len() > self.config.candidate_limit
        {
            return Err(Error(
                "native geometric artifact structure is invalid".into(),
            ));
        }
        let mut duplicate = self.clone();
        duplicate.refresh_identity()?;
        if duplicate.artifact_cid != self.artifact_cid
            || duplicate.uor_model_address != self.uor_model_address
        {
            return Err(Error(
                "native artifact content/structural identity mismatch".into(),
            ));
        }
        if geometry(self.prior_scores.len(), self.config.context_tokens)? != self.geometry {
            return Err(Error("native artifact prime, H4, orientation or fixed-zeta tables differ from the named construction".into()));
        }
        if let Some(block) = &self.learned_routing {
            block.validate(self)?;
        }
        if let Some(block) = &self.source_routing {
            block.validate(self)?;
        }
        let mut ids = BTreeSet::new();
        if self.construction.is_empty()
            || self
                .construction
                .iter()
                .any(|d| d.id.trim().is_empty() || !ids.insert(&d.id))
        {
            return Err(Error(
                "native artifact construction receipts are invalid".into(),
            ));
        }
        let valid_token =
            |token: &u32| *token != BOS && (*token as usize) < self.prior_scores.len();
        if !self.prior_postings.iter().all(valid_token)
            || self.prior_postings.iter().collect::<BTreeSet<_>>().len()
                != self.prior_postings.len()
        {
            return Err(Error("native artifact global postings are invalid".into()));
        }
        let mut associations = 0_usize;
        for row in &self.rows {
            associations = associations
                .checked_add(row.scores.len())
                .ok_or_else(|| Error("artifact association overflow".into()))?;
            if !valid_feature(row.feature)
                || row.scores.windows(2).any(|p| p[0].token >= p[1].token)
                || row.scores.iter().any(|item| {
                    !valid_token(&item.token) || !(-1_000_000..=0).contains(&item.score)
                })
                || !(-1_000_000..=0).contains(&row.default_score)
                || row.postings.len() > self.config.postings_per_row
                || row.postings.iter().collect::<BTreeSet<_>>().len() != row.postings.len()
                || row.postings.iter().any(|token| {
                    row.scores
                        .binary_search_by_key(token, |item| item.token)
                        .is_err()
                })
            {
                return Err(Error("native artifact feature-score row is invalid".into()));
            }
        }
        if associations > self.config.max_associations
            || self
                .prior_scores
                .iter()
                .any(|score| !(-1_000_000..=0).contains(score))
        {
            return Err(Error(
                "native artifact score/storage bounds are invalid".into(),
            ));
        }
        Ok(())
    }

    pub fn encode(&self, text: &str) -> Result<Vec<u32>> {
        let mut tokens = Vec::new();
        for piece in canonical_lexical_piece_bytes(text.as_bytes()).map_err(source_error)? {
            match self.lexical_pieces.binary_search(&piece) {
                Ok(index) => tokens.push(LEXICAL_BASE + index as u32),
                Err(_) => tokens.extend(piece.into_iter().map(|byte| u32::from(byte) + 2)),
            }
        }
        Ok(tokens)
    }
    pub fn decode(&self, tokens: &[u32]) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        for &token in tokens {
            match token {
                BOS | EOS => {}
                2..=257 => bytes.push((token - 2) as u8),
                _ => bytes.extend_from_slice(
                    self.lexical_pieces
                        .get((token - LEXICAL_BASE) as usize)
                        .ok_or_else(|| Error("decode token outside model vocabulary".into()))?,
                ),
            }
        }
        Ok(bytes)
    }
    pub fn lexical_pieces(&self) -> &[Vec<u8>] {
        &self.lexical_pieces
    }
    pub fn session(&self, control: Control) -> Result<Session> {
        self.config.validate()?;
        Ok(Session::new(self, control))
    }
    pub fn generate(
        &self,
        prompt: &str,
        max_new_tokens: usize,
        control: Control,
    ) -> Result<Generation> {
        if !(1..=4096).contains(&max_new_tokens) {
            return Err(Error("generation budget must be 1..=4096 tokens".into()));
        }
        let mut session = self.session(control)?;
        session.observe(self, BOS)?;
        for token in self.encode(prompt)? {
            session.observe(self, token)?;
        }
        session.begin_response(self)?;
        let mut token_ids = Vec::new();
        let mut response_trace = Vec::new();
        let mut value_trace = Vec::new();
        let mut completion_trace = Vec::new();
        let mut response_entry_trace = Vec::new();
        let mut word_copy_trace = Vec::new();
        let mut stop = "token_budget".to_owned();
        for _ in 0..max_new_tokens {
            let token = session.predict(self)?.token;
            if let Some(decision) = session.word_copy_decision() {
                if word_copy_trace.len() < 96 {
                    word_copy_trace.push(decision);
                }
            }
            if let Some(decision) = session.response_entry_decision() {
                if response_entry_trace.len() < 96 {
                    response_entry_trace.push(decision);
                }
            }
            if let Some(decision) = session.completion_decision() {
                if completion_trace.len() < 96 {
                    completion_trace.push(decision);
                }
            }
            if let Some(decision) = session.value_decision() {
                if value_trace.len() < 96 {
                    value_trace.push(decision);
                }
            }
            if let Some(decision) = session.response_decision() {
                if response_trace.len() < 96 {
                    response_trace.push(decision);
                }
            }
            if token == EOS {
                if session.response_decision().is_some() || self.values.is_some() {
                    session.observe(self, token)?;
                }
                stop = "end_of_document".into();
                break;
            }
            token_ids.push(token);
            session.observe(self, token)?;
            // A completed numeral does not authorize another operation.
            // The next contextual request begins through begin_response.
        }
        let bytes = self.decode(&token_ids)?;
        let utf8_valid = std::str::from_utf8(&bytes).is_ok();
        Ok(Generation {
            text: String::from_utf8_lossy(&bytes).into_owned(),
            utf8_valid,
            bytes,
            token_ids,
            response_trace,
            value_trace,
            completion_trace,
            response_entry_trace,
            word_copy_trace,
            stop,
            work: session.work,
            state: session.state(),
        })
    }

    /// Independent-document next-token evaluation; labels enter only after
    /// prediction. Both identity and exact-byte overlap are refused.
    pub fn evaluate(&self, documents: &[Document], control: Control) -> Result<Evaluation> {
        if documents.is_empty() {
            return Err(Error("evaluation corpus is empty".into()));
        }
        let mut seen = BTreeSet::new();
        for document in documents {
            let candidate = receipt(document);
            if document.id.trim().is_empty()
                || !seen.insert(&document.id)
                || self
                    .construction
                    .iter()
                    .chain(&self.readout_training)
                    .chain(self.memory_read_training())
                    .chain(self.value_training())
                    .chain(self.value_completion_training())
                    .chain(self.response_entry_training())
                    .chain(self.word_copy_training())
                    .chain(self.role_read_training())
                    .chain(self.relation_training())
                    .chain(
                        self.learned_routing
                            .iter()
                            .flat_map(|block| &block.training),
                    )
                    .any(|known| known.id == candidate.id || known.text_cid == candidate.text_cid)
            {
                return Err(Error(format!(
                    "evaluation document {} overlaps construction or repeats an ID",
                    document.id
                )));
            }
        }
        let mut result = Evaluation {
            documents: documents.len(),
            positions: 0,
            correct: 0,
            candidate_hits: 0,
            geometric_row_positions: 0,
            top1: 0.0,
            candidate_coverage: 0.0,
            work: Work::default(),
        };
        for document in documents {
            let mut session = self.session(control)?;
            session.observe(self, BOS)?;
            let mut stream = self.encode(&document.text)?;
            stream.push(EOS);
            for token in stream {
                let prediction = session.predict(self)?;
                result.positions += 1;
                result.correct += u64::from(prediction.token == token);
                result.candidate_hits += u64::from(
                    session
                        .candidates()
                        .iter()
                        .any(|candidate| candidate.token == token),
                );
                result.geometric_row_positions += u64::from(prediction.geometric_rows > 0);
                session.observe(self, token)?;
            }
            add_work(&mut result.work, session.work);
        }
        result.top1 = result.correct as f64 / result.positions as f64;
        result.candidate_coverage = result.candidate_hits as f64 / result.positions as f64;
        Ok(result)
    }
}

fn add_work(total: &mut Work, work: Work) {
    total.learned_routing.add(work.learned_routing);
    total.word_copy.routing.add(work.word_copy.routing);
    add_completion_work(&mut total.word_copy.selector, work.word_copy.selector);
    total.word_copy.dictionary_lookups += work.word_copy.dictionary_lookups;
    total.word_copy.dictionary_comparisons += work.word_copy.dictionary_comparisons;
    total.word_copy.dictionary_byte_comparisons += work.word_copy.dictionary_byte_comparisons;
    total.word_copy.word_candidates += work.word_copy.word_candidates;
    total.word_copy.word_record_reads += work.word_copy.word_record_reads;
    total.word_copy.bound_rejections += work.word_copy.bound_rejections;
    total.word_copy.byte_reads += work.word_copy.byte_reads;
    total.values.admission_legality_checks += work.values.admission_legality_checks;
    total.values.admission_decisions += work.values.admission_decisions;
    total.values.admission_rejections += work.values.admission_rejections;
    total.values.input_bytes += work.values.input_bytes;
    total.values.literal_writes += work.values.literal_writes;
    total.values.record_evictions += work.values.record_evictions;
    total.values.proposals += work.values.proposals;
    total.values.alias_self_add_rejections += work.values.alias_self_add_rejections;
    total.values.routing.add(work.values.routing);
    total.values.operator_executions += work.values.operator_executions;
    total.values.selection_comparisons += work.values.selection_comparisons;
    total.values.selection_passes += work.values.selection_passes;
    total.values.additions += work.values.additions;
    total.values.subtractions += work.values.subtractions;
    total.values.multiplications += work.values.multiplications;
    total.values.source_refreshes += work.values.source_refreshes;
    total.values.overflow_rejections += work.values.overflow_rejections;
    total.values.feature_lookups += work.values.feature_lookups;
    total.values.feature_comparisons += work.values.feature_comparisons;
    total.values.cue_comparisons += work.values.cue_comparisons;
    total.values.lexical_comparisons += work.values.lexical_comparisons;
    total.values.lexical_byte_comparisons += work.values.lexical_byte_comparisons;
    total.values.lexical_writes += work.values.lexical_writes;
    total.values.h4_reads += work.values.h4_reads;
    total.values.phase_updates += work.values.phase_updates;
    total.values.numeral_steps += work.values.numeral_steps;
    total.values.derived_writes += work.values.derived_writes;
    total.values.emission_commits += work.values.emission_commits;
    total.values.emission_mismatches += work.values.emission_mismatches;
    add_completion_work(&mut total.completion, work.completion);
    add_completion_work(&mut total.response_entry, work.response_entry);
    total.response_query_captures += work.response_query_captures;
    total.response_commits += work.response_commits;
    total.response_requeries += work.response_requeries;
    total.response_continuations += work.response_continuations;
    total.response_base_steps += work.response_base_steps;
    total.response_stops += work.response_stops;
    total.response_mismatches += work.response_mismatches;
    total.response_reference_reads += work.response_reference_reads;
    total.memory_cue_reads += work.memory_cue_reads;
    total.memory_index_reads += work.memory_index_reads;
    total.memory_index_writes += work.memory_index_writes;
    total.memory_stale_rejections += work.memory_stale_rejections;
    total.memory_candidates += work.memory_candidates;
    total.memory_score_lookups += work.memory_score_lookups;
    total.memory_composed_candidates += work.memory_composed_candidates;
    total.memory_composition_feature_offers += work.memory_composition_feature_offers;
    total.memory_composition_duplicate_features += work.memory_composition_duplicate_features;
    total.memory_composition_comparisons += work.memory_composition_comparisons;
    total.memory_composition_feature_moves += work.memory_composition_feature_moves;
    total.memory_h4_reads += work.memory_h4_reads;
    total.memory_phase_updates += work.memory_phase_updates;
    total.mixture_gate_reads += work.mixture_gate_reads;
    total.observed_tokens += work.observed_tokens;
    total.evictions += work.evictions;
    total.h4_table_reads += work.h4_table_reads;
    total.phase_additions += work.phase_additions;
    total.orientation_table_reads += work.orientation_table_reads;
    total.anchor_table_reads += work.anchor_table_reads;
    total.radial_square_reads += work.radial_square_reads;
    total.feature_queries += work.feature_queries;
    total.matched_rows += work.matched_rows;
    total.candidate_offers += work.candidate_offers;
    total.candidate_evaluations += work.candidate_evaluations;
    total.score_lookups += work.score_lookups;
}

fn add_completion_work(total: &mut CompletionWork, work: CompletionWork) {
    total.observations += work.observations;
    total.anchors += work.anchors;
    total.metadata_reads += work.metadata_reads;
    total.state_copies += work.state_copies;
    total.feature_queries += work.feature_queries;
    total.row_comparisons += work.row_comparisons;
    total.matched_rows += work.matched_rows;
    total.posting_offers += work.posting_offers;
    total.candidate_comparisons += work.candidate_comparisons;
    total.candidate_writes += work.candidate_writes;
    total.candidate_drops += work.candidate_drops;
    total.candidate_evaluations += work.candidate_evaluations;
    total.score_lookups += work.score_lookups;
    total.score_comparisons += work.score_comparisons;
    total.h4_reads += work.h4_reads;
    total.orientation_reads += work.orientation_reads;
    total.phase_subtractions += work.phase_subtractions;
    total.commits += work.commits;
    total.base_steps += work.base_steps;
    total.mismatches += work.mismatches;
    total.stops += work.stops;
    total.step_limits += work.step_limits;
}

#[cfg(test)]
mod work_tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn native_evaluation_adds_every_nested_work_counter() {
        // Seed the individually omitted fields as well as the optional groups
        // so the independent JSON oracle covers every current counter.
        let seed = Work {
            values: ValueWork {
                routing: RoutingWork {
                    predictions: 1,
                    ..RoutingWork::default()
                },
                alias_self_add_rejections: 1,
                operator_executions: 1,
                selection_comparisons: 1,
                selection_passes: 1,
                lexical_comparisons: 1,
                lexical_byte_comparisons: 1,
                lexical_writes: 1,
                subtractions: 1,
                multiplications: 1,
                source_refreshes: 1,
                ..ValueWork::default()
            },
            completion: CompletionWork {
                observations: 1,
                ..CompletionWork::default()
            },
            response_entry: CompletionWork {
                observations: 1,
                ..CompletionWork::default()
            },
            ..Work::default()
        };
        fn fill(value: &mut Value, next: &mut u64) {
            if let Some(fields) = value.as_object_mut() {
                for field in fields.values_mut() {
                    fill(field, next);
                }
            } else {
                assert!(value.is_u64());
                *value = Value::from(*next);
                *next += 1;
            }
        }
        fn double(value: &mut Value) {
            if let Some(fields) = value.as_object_mut() {
                for field in fields.values_mut() {
                    double(field);
                }
            } else {
                *value = Value::from(value.as_u64().unwrap() * 2);
            }
        }
        let mut expected = serde_json::to_value(seed).unwrap();
        fill(&mut expected, &mut 1);
        let work: Work = serde_json::from_value(expected.clone()).unwrap();
        let mut total = Work::default();
        add_work(&mut total, work);
        add_work(&mut total, work);
        double(&mut expected);
        assert_eq!(serde_json::to_value(total).unwrap(), expected);
    }
}

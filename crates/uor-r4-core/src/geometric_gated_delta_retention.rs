//! Construction-only reference cell for predictive geometric retention.
//!
//! `GeometricGatedDeltaRetentionR4V1` is intentionally a compiler-side
//! mechanism-discovery implementation.  It keeps the immutable exact
//! prime/spin leaves as route identities, learns separate token-local key,
//! value, and query placements, transports four multirate bounded banks over
//! the same last-context delta stream through the exact route connection, and
//! applies a causal gated-delta write only after the next token is observed.
//! It does not yet ingest separate current/previous/last-two/hierarchy input
//! channels, and it does not claim an
//! integer/table lowering and it never widens the candidate support supplied
//! by the caller (the intended caller is the unchanged #953 maximum-count
//! admission path).

use serde::Serialize;
use std::collections::BTreeSet;

use crate::bounded_global_exact_spin_attention::ExactSpinState;
use crate::canonical_lexical_ingestion::{
    validate_h4_binary_icosahedral_closure, H4BinaryIcosahedralClosure,
};
use crate::corpus_induced_spin_placement::{compile_identity_leaves, leaf_for_token};

const ARTIFACT_MAGIC: &[u8; 8] = b"GGDR0001";
const ARTIFACT_SCHEMA: u32 = 1;
const DIMENSION: usize = 4;
const BANK_COUNT: usize = 4;
const SCORE_EPSILON: f64 = 1.0e-12;
const GOLDEN_RATIO: f64 = 1.618_033_988_749_895;

pub const CANONICAL_PRIME_SPIN_LEAF_POLICY: &str = concat!(
    "schema=1\n",
    "bos=exact-identity\n",
    "h4=token-mod-4-over-fixed-canonical-roots\n",
    "fiber=wrap(prime[token]*1000003+token*17071)-q29\n",
    "torsion=wrap(prime[token]*-97409+token*7919)-q29\n",
    "primes=ascending-first-primes-through-maximum-token-id"
);

pub const GEOMETRIC_RETENTION_SUPPORT_POLICY: &str = concat!(
    "schema=1\n",
    "caller-asserted-source=#953-local-same-object-context-placement\n",
    "admission=unchanged-maximum-count-tie\n",
    "artifact-binding=source-free-table-cid+multiscale-overlay-cid+construction-partition-id\n",
    "per-event-support-origin-proof=NOT_YET_BOUND\n",
    "raw-support-validation=namespace,nonempty,strict-order,target-membership\n",
    "cell-may-rank-but-must-not-widen-support"
);

/// Frozen mechanism contract.  The byte identity is embedded in every
/// artifact before any learned placement bytes.
pub const GEOMETRIC_GATED_DELTA_RETENTION_POLICY: &str = concat!(
    "schema=1\n",
    "scope=construction-only-host-reference\n",
    "route-identity=immutable-exact-H4-plus-central-fiber-Q29-plus-torsion-Q29-leaves\n",
    "placements=separate-learned-token-local-k-v-q\n",
    "banks=four-multirate-last-context-delta-banks-in-one-cumulative-prefix-frame\n",
    "transport-v1=P*S*P^-1;P=four-dimensional-left-quaternion-representation-",
    "of-exact-H4-projection\n",
    "central-fiber-torsion=identity-bound-in-leaf-kappa-not-independent-R4-operators\n",
    "write=after-observation-only;rho*transported+eta*(v-transported*k)outer-k\n",
    "read=r=sum-bank-weight*(S*k-current-context);",
    "score=dot(transported-q-candidate,r);v=write-only\n",
    "admission=caller-supplied-unchanged-support-only\n",
    "optimizer=sorted-documents,causal-order,local-contrastive-delta,no-bptt\n",
    "v-surrogate=post-observation-frame(Q-target-Q-negative)*",
    "sum-bank(read-weight*actual-eta*norm2(framed-context-key));backtransport-to-local\n",
    "controls=full,plain-delta,no-delta-overwrite,transport-permuted,",
    "left-fold-route,last-only\n",
    "not-claimed=heldout-promotion,runtime-lowering,language-model,",
    "separate-current-previous-last-two-hierarchy-input-channels"
);

type Vector4 = [f64; DIMENSION];
type Matrix4 = [[f64; DIMENSION]; DIMENSION];

#[derive(Debug, Clone, PartialEq)]
pub enum GeometricGatedDeltaRetentionError {
    Invalid(String),
    ExactRoute(String),
    Arithmetic(String),
}

impl std::fmt::Display for GeometricGatedDeltaRetentionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(reason) => formatter.write_str(reason),
            Self::ExactRoute(reason) => write!(formatter, "exact route: {reason}"),
            Self::Arithmetic(reason) => write!(formatter, "arithmetic: {reason}"),
        }
    }
}

impl std::error::Error for GeometricGatedDeltaRetentionError {}

/// One pre-declared next-token event from the construction partition.
///
/// `admitted_support` is supplied by #953 (or a matched fixture) and must be
/// strictly ascending and duplicate-free.  The cell scores exactly these
/// tokens and cannot add a candidate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GeometricRetentionConstructionStep {
    pub admitted_support: Vec<u32>,
    pub observed_token: u32,
}

/// One construction sequence.  `initial_token` seeds the causal state; every
/// step is predicted before its `observed_token` is passed to the write path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GeometricRetentionConstructionSequence {
    pub document_id: String,
    pub initial_token: u32,
    pub steps: Vec<GeometricRetentionConstructionStep>,
}

/// Frozen compiler-side optimizer and recurrent-bank parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct GeometricGatedDeltaRetentionConfig {
    pub epochs: u32,
    pub learning_rate: f64,
    pub retention: [f64; BANK_COUNT],
    pub write_gate: [f64; BANK_COUNT],
    pub read_weight: [f64; BANK_COUNT],
}

/// Immutable provenance for the unchanged #953 admission path.  The model
/// cannot establish these identities itself; its caller must supply the
/// table and overlay artifacts that produced every admitted support.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GeometricRetentionSupportBinding {
    table_artifact_cid: String,
    overlay_artifact_cid: String,
    construction_partition_identity: String,
}

impl GeometricRetentionSupportBinding {
    pub fn new(
        table_artifact_cid: impl Into<String>,
        overlay_artifact_cid: impl Into<String>,
        construction_partition_identity: impl Into<String>,
    ) -> Result<Self, GeometricGatedDeltaRetentionError> {
        let binding = Self {
            table_artifact_cid: table_artifact_cid.into(),
            overlay_artifact_cid: overlay_artifact_cid.into(),
            construction_partition_identity: construction_partition_identity.into(),
        };
        validate_cid(&binding.table_artifact_cid, "source-free table artifact")?;
        validate_cid(&binding.overlay_artifact_cid, "multiscale overlay artifact")?;
        validate_identity(
            &binding.construction_partition_identity,
            "construction partition identity",
        )?;
        Ok(binding)
    }

    pub fn table_artifact_cid(&self) -> &str {
        &self.table_artifact_cid
    }

    pub fn overlay_artifact_cid(&self) -> &str {
        &self.overlay_artifact_cid
    }

    pub fn construction_partition_identity(&self) -> &str {
        &self.construction_partition_identity
    }

    pub fn policy_identity(&self) -> &'static str {
        GEOMETRIC_RETENTION_SUPPORT_POLICY
    }
}

impl Default for GeometricGatedDeltaRetentionConfig {
    fn default() -> Self {
        Self {
            epochs: 4,
            learning_rate: 0.025,
            retention: [0.10, 0.55, 0.90, 0.985],
            write_gate: [1.0, 0.55, 0.20, 0.06],
            read_weight: [0.40, 0.30, 0.20, 0.10],
        }
    }
}

impl GeometricGatedDeltaRetentionConfig {
    fn validate(self) -> Result<Self, GeometricGatedDeltaRetentionError> {
        if self.epochs == 0 || self.epochs > 1_024 {
            return Err(GeometricGatedDeltaRetentionError::Invalid(
                "retention compile epochs must be in 1..=1024".to_owned(),
            ));
        }
        if !self.learning_rate.is_finite() || self.learning_rate <= 0.0 || self.learning_rate > 1.0
        {
            return Err(GeometricGatedDeltaRetentionError::Invalid(
                "retention learning rate must be finite and in (0,1]".to_owned(),
            ));
        }
        for (name, values, upper) in [
            ("retention", self.retention, 0.999_999),
            ("write gate", self.write_gate, 1.0),
            ("read weight", self.read_weight, 1.0),
        ] {
            if values
                .iter()
                .any(|value| !value.is_finite() || *value < 0.0 || *value > upper)
            {
                return Err(GeometricGatedDeltaRetentionError::Invalid(format!(
                    "{name} values must be finite and in [0,{upper}]"
                )));
            }
        }
        if self.read_weight.iter().sum::<f64>() <= 0.0 {
            return Err(GeometricGatedDeltaRetentionError::Invalid(
                "at least one read weight must be positive".to_owned(),
            ));
        }
        Ok(self)
    }
}

/// The matched causal interventions.  Every arm has four 4x4 banks and the
/// same number of token-local K/V/Q parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeometricRetentionControl {
    FullGeometric,
    PlainDelta,
    NoDeltaOverwrite,
    TransportPermuted,
    LeftFoldRoute,
    LastOnly,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TokenPlacement {
    key: Vector4,
    value: Vector4,
    query: Vector4,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GeometricRetentionPlacementTrace {
    pub token: u32,
    pub key: Vector4,
    pub value: Vector4,
    pub query: Vector4,
    pub pairwise_distinct: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct RetentionBank {
    memory: Matrix4,
}

impl Default for RetentionBank {
    fn default() -> Self {
        Self {
            memory: zero_matrix(),
        }
    }
}

/// Fixed-size recurrent state.  It stores four 4x4 matrices, one exact route
/// frame, the last observed token, and counters; it stores no prefix or corpus.
#[derive(Debug, Clone)]
pub struct GeometricGatedDeltaRetentionState {
    control: GeometricRetentionControl,
    banks: [RetentionBank; BANK_COUNT],
    route_frame: ExactSpinState,
    last_token: Option<u32>,
    observations: u64,
}

impl GeometricGatedDeltaRetentionState {
    pub const fn control(&self) -> GeometricRetentionControl {
        self.control
    }

    pub const fn observations(&self) -> u64 {
        self.observations
    }

    pub const fn bounded_scalar_state_len(&self) -> usize {
        BANK_COUNT * DIMENSION * DIMENSION
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GeometricRetentionCandidateScore {
    pub token: u32,
    pub score: f64,
    pub candidate_query_connection_table_offset: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GeometricRetentionPredictionTrace {
    pub control: GeometricRetentionControl,
    pub table_artifact_cid: String,
    pub overlay_artifact_cid: String,
    pub construction_partition_identity: String,
    pub support_policy: String,
    pub admitted_support: Vec<u32>,
    pub scores: Vec<GeometricRetentionCandidateScore>,
    pub selected_token: u32,
    pub state_checksum: String,
    pub bank_reads: u64,
    pub dot_products: u64,
    pub token_local_vector_transports: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GeometricRetentionObservationTrace {
    pub control: GeometricRetentionControl,
    pub observed_token: u32,
    pub previous_route: GeometricRetentionExactRouteTrace,
    pub current_route: GeometricRetentionExactRouteTrace,
    pub connection_route: GeometricRetentionExactRouteTrace,
    pub before_state_checksum: String,
    pub after_state_checksum: String,
    pub retention_gates: [f64; BANK_COUNT],
    pub write_gates: [f64; BANK_COUNT],
    pub mean_delta_residual_norm: f64,
    pub delta_overwrite_enabled: bool,
    pub write_performed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct GeometricRetentionExactRouteTrace {
    pub h4_table_offset: u16,
    pub fiber_q29: i64,
    pub torsion_q29: i64,
}

/// Compiler-side learned artifact.  Exact route leaves remain immutable and
/// are not optimizer parameters.
#[derive(Debug, Clone)]
pub struct GeometricGatedDeltaRetentionR4V1 {
    maximum_token_id: u32,
    config: GeometricGatedDeltaRetentionConfig,
    exact_route_table: H4BinaryIcosahedralClosure,
    exact_route_leaves: Vec<ExactSpinState>,
    geometric_placements: Vec<TokenPlacement>,
    plain_placements: Vec<TokenPlacement>,
    support_binding: GeometricRetentionSupportBinding,
    exact_leaf_map_kappa: String,
    learning_update_counts: [[u64; 3]; 2],
    construction_document_ids: Vec<String>,
    construction_event_count: u64,
    construction_population_kappa: String,
}

impl GeometricGatedDeltaRetentionR4V1 {
    /// Deterministically fit the bounded reference cell on construction-only
    /// sequences.  Documents are sorted by ID before every epoch; event order
    /// within a document remains causal.
    pub fn compile(
        maximum_token_id: u32,
        construction_sequences: &[GeometricRetentionConstructionSequence],
        config: GeometricGatedDeltaRetentionConfig,
        support_binding: GeometricRetentionSupportBinding,
    ) -> Result<Self, GeometricGatedDeltaRetentionError> {
        let config = config.validate()?;
        if construction_sequences.is_empty() {
            return Err(GeometricGatedDeltaRetentionError::Invalid(
                "geometric retention construction population is empty".to_owned(),
            ));
        }
        let mut ordered = construction_sequences.iter().collect::<Vec<_>>();
        ordered.sort_by(|left, right| left.document_id.cmp(&right.document_id));
        let mut document_ids = BTreeSet::new();
        let mut event_count = 0_u64;
        for sequence in &ordered {
            validate_sequence(sequence, maximum_token_id)?;
            if !document_ids.insert(sequence.document_id.clone()) {
                return Err(GeometricGatedDeltaRetentionError::Invalid(format!(
                    "duplicate construction document id {}",
                    sequence.document_id
                )));
            }
            event_count = event_count
                .checked_add(u64::try_from(sequence.steps.len()).map_err(|_| {
                    GeometricGatedDeltaRetentionError::Arithmetic(
                        "construction event count does not fit u64".to_owned(),
                    )
                })?)
                .ok_or_else(|| {
                    GeometricGatedDeltaRetentionError::Arithmetic(
                        "construction event count overflow".to_owned(),
                    )
                })?;
        }
        if event_count == 0 {
            return Err(GeometricGatedDeltaRetentionError::Invalid(
                "geometric retention construction population has no events".to_owned(),
            ));
        }
        let construction_population_kappa =
            construction_population_kappa(&ordered, &support_binding);

        let exact_route_table = validate_h4_binary_icosahedral_closure()
            .map_err(|error| GeometricGatedDeltaRetentionError::ExactRoute(error.to_string()))?;
        let exact_route_leaves = compile_identity_leaves(maximum_token_id, &exact_route_table)
            .map_err(|error| GeometricGatedDeltaRetentionError::ExactRoute(error.to_string()))?;
        let exact_leaf_map_kappa = exact_leaf_map_kappa(&exact_route_leaves, &exact_route_table)?;
        let mut geometric_placements = Vec::with_capacity(exact_route_leaves.len());
        let mut plain_placements = Vec::with_capacity(exact_route_leaves.len());
        for (token_index, leaf) in exact_route_leaves.iter().copied().enumerate() {
            let token = u32::try_from(token_index).map_err(|_| {
                GeometricGatedDeltaRetentionError::Arithmetic(
                    "token placement index does not fit u32".to_owned(),
                )
            })?;
            geometric_placements.push(geometric_seed_placement(token, leaf, &exact_route_table)?);
            plain_placements.push(plain_seed_placement(token));
        }
        let mut model = Self {
            maximum_token_id,
            config,
            exact_route_table,
            exact_route_leaves,
            geometric_placements,
            plain_placements,
            support_binding,
            exact_leaf_map_kappa,
            learning_update_counts: [[0; 3]; 2],
            construction_document_ids: document_ids.into_iter().collect(),
            construction_event_count: event_count,
            construction_population_kappa,
        };

        for _epoch in 0..config.epochs {
            for sequence in &ordered {
                model.train_sequence(sequence, GeometricRetentionControl::FullGeometric)?;
                model.train_sequence(sequence, GeometricRetentionControl::PlainDelta)?;
            }
        }
        model.validate_learned_state()?;
        Ok(model)
    }

    pub const fn maximum_token_id(&self) -> u32 {
        self.maximum_token_id
    }

    pub const fn construction_event_count(&self) -> u64 {
        self.construction_event_count
    }

    pub fn policy_identity(&self) -> &'static str {
        GEOMETRIC_GATED_DELTA_RETENTION_POLICY
    }

    pub fn construction_document_ids(&self) -> &[String] {
        &self.construction_document_ids
    }

    pub fn construction_population_kappa(&self) -> &str {
        &self.construction_population_kappa
    }

    pub fn support_binding(&self) -> &GeometricRetentionSupportBinding {
        &self.support_binding
    }

    pub fn h4_root_table_kappa(&self) -> &str {
        &self.exact_route_table.h4_root_table_kappa
    }

    pub fn h4_product_table_kappa(&self) -> &str {
        &self.exact_route_table.multiplication_table_kappa
    }

    pub fn exact_leaf_map_kappa(&self) -> &str {
        &self.exact_leaf_map_kappa
    }

    /// `[key, value, query]` non-zero optimizer steps for the geometric and
    /// matched plain arms, respectively.
    pub const fn learning_update_counts(&self) -> [[u64; 3]; 2] {
        self.learning_update_counts
    }

    pub fn start_state(
        &self,
        control: GeometricRetentionControl,
    ) -> Result<GeometricGatedDeltaRetentionState, GeometricGatedDeltaRetentionError> {
        let route_frame = ExactSpinState::identity(&self.exact_route_table)
            .map_err(|error| GeometricGatedDeltaRetentionError::ExactRoute(error.to_string()))?;
        Ok(GeometricGatedDeltaRetentionState {
            control,
            banks: [RetentionBank::default(); BANK_COUNT],
            route_frame,
            last_token: None,
            observations: 0,
        })
    }

    pub fn placement_trace(
        &self,
        token: u32,
        plain: bool,
    ) -> Result<GeometricRetentionPlacementTrace, GeometricGatedDeltaRetentionError> {
        let placement = self.placement(token, plain)?;
        Ok(GeometricRetentionPlacementTrace {
            token,
            key: placement.key,
            value: placement.value,
            query: placement.query,
            pairwise_distinct: vector_bits(placement.key) != vector_bits(placement.value)
                && vector_bits(placement.key) != vector_bits(placement.query)
                && vector_bits(placement.value) != vector_bits(placement.query),
        })
    }

    /// Read without mutation.  The observed target is deliberately absent
    /// from this signature, so changing a future target cannot affect the
    /// pre-observation choice.
    pub fn predict(
        &self,
        state: &GeometricGatedDeltaRetentionState,
        admitted_support: &[u32],
    ) -> Result<GeometricRetentionPredictionTrace, GeometricGatedDeltaRetentionError> {
        validate_support(admitted_support, self.maximum_token_id)?;
        let last_token = state.last_token.ok_or_else(|| {
            GeometricGatedDeltaRetentionError::Invalid(
                "retention prediction requires an observed causal seed".to_owned(),
            )
        })?;
        let read = self.read_vector(state, last_token)?;
        let plain = state.control == GeometricRetentionControl::PlainDelta;
        let mut scores = Vec::with_capacity(admitted_support.len());
        let mut selected_token = admitted_support[0];
        let mut selected_score = f64::NEG_INFINITY;
        for &token in admitted_support {
            let placement = self.placement(token, plain)?;
            let (candidate_query, connection_offset, _) =
                self.local_vector_in_frame(token, placement.query, state.route_frame, plain)?;
            let score = dot(candidate_query, read);
            require_finite_scalar(score, "candidate score")?;
            scores.push(GeometricRetentionCandidateScore {
                token,
                score,
                candidate_query_connection_table_offset: connection_offset,
            });
            if score > selected_score + SCORE_EPSILON {
                selected_score = score;
                selected_token = token;
            }
        }
        Ok(GeometricRetentionPredictionTrace {
            control: state.control,
            table_artifact_cid: self.support_binding.table_artifact_cid.clone(),
            overlay_artifact_cid: self.support_binding.overlay_artifact_cid.clone(),
            construction_partition_identity: self
                .support_binding
                .construction_partition_identity
                .clone(),
            support_policy: GEOMETRIC_RETENTION_SUPPORT_POLICY.to_owned(),
            admitted_support: admitted_support.to_vec(),
            scores,
            selected_token,
            state_checksum: state_checksum(state),
            bank_reads: BANK_COUNT as u64,
            dot_products: u64::try_from(admitted_support.len()).map_err(|_| {
                GeometricGatedDeltaRetentionError::Arithmetic(
                    "candidate count does not fit work ledger".to_owned(),
                )
            })?,
            token_local_vector_transports: u64::try_from(admitted_support.len())
                .map_err(|_| {
                    GeometricGatedDeltaRetentionError::Arithmetic(
                        "candidate count does not fit transport ledger".to_owned(),
                    )
                })?
                .checked_add(1)
                .ok_or_else(|| {
                    GeometricGatedDeltaRetentionError::Arithmetic(
                        "candidate transport work overflow".to_owned(),
                    )
                })?,
        })
    }

    /// Advance the causal state after one token has become observable.
    pub fn observe(
        &self,
        state: &mut GeometricGatedDeltaRetentionState,
        observed_token: u32,
    ) -> Result<GeometricRetentionObservationTrace, GeometricGatedDeltaRetentionError> {
        self.placement(observed_token, false)?;
        let before_state_checksum = state_checksum(state);
        let previous_route = state.route_frame;
        let observed_leaf = leaf_for_token(&self.exact_route_leaves, observed_token)
            .map_err(|error| GeometricGatedDeltaRetentionError::ExactRoute(error.to_string()))?;
        let natural_next = previous_route
            .compose(observed_leaf, &self.exact_route_table)
            .map_err(|error| GeometricGatedDeltaRetentionError::ExactRoute(error.to_string()))?;
        let current_route = match state.control {
            GeometricRetentionControl::LeftFoldRoute => observed_leaf
                .compose(previous_route, &self.exact_route_table)
                .map_err(|error| {
                    GeometricGatedDeltaRetentionError::ExactRoute(error.to_string())
                })?,
            GeometricRetentionControl::LastOnly => observed_leaf,
            _ => natural_next,
        };
        let connection = match state.control {
            GeometricRetentionControl::TransportPermuted => {
                let permutation_modulus =
                    self.maximum_token_id.checked_add(1).ok_or_else(|| {
                        GeometricGatedDeltaRetentionError::Arithmetic(
                            "token permutation modulus overflow".to_owned(),
                        )
                    })?;
                let permuted_token = observed_token
                    .checked_add(1)
                    .unwrap_or(0)
                    .rem_euclid(permutation_modulus);
                let permuted_leaf = leaf_for_token(&self.exact_route_leaves, permuted_token)
                    .map_err(|error| {
                        GeometricGatedDeltaRetentionError::ExactRoute(error.to_string())
                    })?;
                previous_route
                    .compose(permuted_leaf, &self.exact_route_table)
                    .and_then(|fake_next| {
                        previous_route
                            .inverse(&self.exact_route_table)
                            .and_then(|inverse| fake_next.compose(inverse, &self.exact_route_table))
                    })
                    .map_err(|error| {
                        GeometricGatedDeltaRetentionError::ExactRoute(error.to_string())
                    })?
            }
            _ => previous_route
                .inverse(&self.exact_route_table)
                .and_then(|inverse| current_route.compose(inverse, &self.exact_route_table))
                .map_err(|error| {
                    GeometricGatedDeltaRetentionError::ExactRoute(error.to_string())
                })?,
        };
        let transport = if state.control == GeometricRetentionControl::PlainDelta {
            identity_matrix()
        } else {
            exact_connection_matrix(connection, &self.exact_route_table)?
        };
        let plain = state.control == GeometricRetentionControl::PlainDelta;
        let observed_placement = self.placement(observed_token, plain)?;
        let (observed_value, _, _) = self.local_vector_in_frame(
            observed_token,
            observed_placement.value,
            current_route,
            plain,
        )?;
        let (observed_query, _, _) = self.local_vector_in_frame(
            observed_token,
            observed_placement.query,
            current_route,
            plain,
        )?;
        let framed_key = state
            .last_token
            .map(|token| {
                self.placement(token, plain).and_then(|placement| {
                    self.local_vector_in_frame(token, placement.key, current_route, plain)
                        .map(|(key, _, _)| key)
                })
            })
            .transpose()?;
        let gate_coupling = framed_key
            .map(|key| dot(key, observed_query).abs())
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);
        let mut retention_gates = [0.0; BANK_COUNT];
        let mut write_gates = [0.0; BANK_COUNT];
        let mut residual_norm_sum = 0.0;

        for bank_index in 0..BANK_COUNT {
            let retained = if state.control == GeometricRetentionControl::LastOnly {
                zero_matrix()
            } else {
                conjugate(state.banks[bank_index].memory, transport)
            };
            let rho = (self.config.retention[bank_index]
                + (1.0 - self.config.retention[bank_index]) * gate_coupling * 0.05)
                .clamp(0.0, 0.999_999);
            let eta = (self.config.write_gate[bank_index] * (0.75 + 0.25 * gate_coupling))
                .clamp(0.0, 1.0);
            retention_gates[bank_index] = rho;
            write_gates[bank_index] = eta;
            let mut next_memory = scale_matrix(retained, rho);
            if let Some(key) = framed_key {
                let previous_value = matrix_vector(retained, key);
                let write_vector = if state.control == GeometricRetentionControl::NoDeltaOverwrite {
                    observed_value
                } else {
                    subtract(observed_value, previous_value)
                };
                residual_norm_sum += norm(write_vector);
                next_memory = add_matrix(next_memory, scale_matrix(outer(write_vector, key), eta));
            }
            require_finite_matrix(next_memory, "retention bank update")?;
            state.banks[bank_index].memory = next_memory;
        }
        state.route_frame = current_route;
        state.last_token = Some(observed_token);
        state.observations = state.observations.checked_add(1).ok_or_else(|| {
            GeometricGatedDeltaRetentionError::Arithmetic(
                "retention observation count overflow".to_owned(),
            )
        })?;
        let trace = GeometricRetentionObservationTrace {
            control: state.control,
            observed_token,
            previous_route: exact_route_trace(previous_route),
            current_route: exact_route_trace(current_route),
            connection_route: exact_route_trace(connection),
            before_state_checksum,
            after_state_checksum: state_checksum(state),
            retention_gates,
            write_gates,
            mean_delta_residual_norm: if framed_key.is_some() {
                residual_norm_sum / BANK_COUNT as f64
            } else {
                0.0
            },
            delta_overwrite_enabled: state.control != GeometricRetentionControl::NoDeltaOverwrite,
            write_performed: framed_key.is_some(),
        };
        require_finite_scalar(trace.mean_delta_residual_norm, "delta residual")?;
        Ok(trace)
    }

    /// Inspect the deployed candidate-relative readout for one bank without
    /// modifying state: `dot(Q(candidate), S * K(key))`. V remains write-only.
    pub fn association_score(
        &self,
        state: &GeometricGatedDeltaRetentionState,
        bank_index: usize,
        key_token: u32,
        candidate_token: u32,
    ) -> Result<f64, GeometricGatedDeltaRetentionError> {
        let bank = state.banks.get(bank_index).ok_or_else(|| {
            GeometricGatedDeltaRetentionError::Invalid(format!(
                "retention bank index {bank_index} is outside 0..{BANK_COUNT}"
            ))
        })?;
        let plain = state.control == GeometricRetentionControl::PlainDelta;
        let key_placement = self.placement(key_token, plain)?;
        let candidate_placement = self.placement(candidate_token, plain)?;
        let (key, _, _) =
            self.local_vector_in_frame(key_token, key_placement.key, state.route_frame, plain)?;
        let (candidate_query, _, _) = self.local_vector_in_frame(
            candidate_token,
            candidate_placement.query,
            state.route_frame,
            plain,
        )?;
        let score = dot(candidate_query, matrix_vector(bank.memory, key));
        require_finite_scalar(score, "association score")?;
        Ok(score)
    }

    pub fn state_checksum(&self, state: &GeometricGatedDeltaRetentionState) -> String {
        state_checksum(state)
    }

    /// Canonical compiler artifact bytes.  Floating-point parameters are
    /// bound by their IEEE-754 bit patterns in canonical token order.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(ARTIFACT_MAGIC);
        bytes.extend_from_slice(&ARTIFACT_SCHEMA.to_le_bytes());
        push_bytes(
            &mut bytes,
            GEOMETRIC_GATED_DELTA_RETENTION_POLICY.as_bytes(),
        );
        push_bytes(&mut bytes, GEOMETRIC_RETENTION_SUPPORT_POLICY.as_bytes());
        push_bytes(&mut bytes, CANONICAL_PRIME_SPIN_LEAF_POLICY.as_bytes());
        bytes.extend_from_slice(&self.maximum_token_id.to_le_bytes());
        bytes.extend_from_slice(&self.config.epochs.to_le_bytes());
        bytes.extend_from_slice(&self.config.learning_rate.to_bits().to_le_bytes());
        push_f64_array(&mut bytes, self.config.retention);
        push_f64_array(&mut bytes, self.config.write_gate);
        push_f64_array(&mut bytes, self.config.read_weight);
        push_bytes(
            &mut bytes,
            self.exact_route_table.h4_root_table_kappa.as_bytes(),
        );
        push_bytes(
            &mut bytes,
            self.exact_route_table.multiplication_table_kappa.as_bytes(),
        );
        push_bytes(&mut bytes, self.exact_leaf_map_kappa.as_bytes());
        push_bytes(
            &mut bytes,
            self.support_binding.table_artifact_cid.as_bytes(),
        );
        push_bytes(
            &mut bytes,
            self.support_binding.overlay_artifact_cid.as_bytes(),
        );
        push_bytes(
            &mut bytes,
            self.support_binding
                .construction_partition_identity
                .as_bytes(),
        );
        push_bytes(&mut bytes, self.construction_population_kappa.as_bytes());
        for arm in self.learning_update_counts {
            for count in arm {
                bytes.extend_from_slice(&count.to_le_bytes());
            }
        }
        bytes.extend_from_slice(&self.construction_event_count.to_le_bytes());
        bytes.extend_from_slice(&(self.construction_document_ids.len() as u64).to_le_bytes());
        for id in &self.construction_document_ids {
            push_bytes(&mut bytes, id.as_bytes());
        }
        bytes.extend_from_slice(&(self.geometric_placements.len() as u64).to_le_bytes());
        for (geometric, plain) in self.geometric_placements.iter().zip(&self.plain_placements) {
            push_placement(&mut bytes, *geometric);
            push_placement(&mut bytes, *plain);
        }
        bytes
    }

    pub fn artifact_cid(&self) -> String {
        format!("blake3:{}", blake3::hash(&self.to_bytes()).to_hex())
    }

    fn train_sequence(
        &mut self,
        sequence: &GeometricRetentionConstructionSequence,
        control: GeometricRetentionControl,
    ) -> Result<(), GeometricGatedDeltaRetentionError> {
        let mut state = self.start_state(control)?;
        self.observe(&mut state, sequence.initial_token)?;
        for step in &sequence.steps {
            if step.admitted_support.len() > 1 {
                let prediction = self.predict(&state, &step.admitted_support)?;
                let negative = prediction
                    .scores
                    .iter()
                    .filter(|score| score.token != step.observed_token)
                    .max_by(|left, right| {
                        left.score
                            .total_cmp(&right.score)
                            .then_with(|| right.token.cmp(&left.token))
                    })
                    .map(|score| score.token)
                    .ok_or_else(|| {
                        GeometricGatedDeltaRetentionError::Invalid(
                            "contrastive event has no admitted distractor".to_owned(),
                        )
                    })?;
                self.local_contrastive_update(
                    &state,
                    step.observed_token,
                    negative,
                    control == GeometricRetentionControl::PlainDelta,
                )?;
            }
            self.observe(&mut state, step.observed_token)?;
        }
        Ok(())
    }

    fn local_contrastive_update(
        &mut self,
        state: &GeometricGatedDeltaRetentionState,
        target: u32,
        negative: u32,
        plain: bool,
    ) -> Result<(), GeometricGatedDeltaRetentionError> {
        let context = state.last_token.ok_or_else(|| {
            GeometricGatedDeltaRetentionError::Invalid(
                "contrastive update requires a causal context".to_owned(),
            )
        })?;
        let aggregate = self.aggregate_memory(state);
        let context_placement = self.placement(context, plain)?;
        let target_placement = self.placement(target, plain)?;
        let negative_placement = self.placement(negative, plain)?;
        let (context_key, _, context_transport) =
            self.local_vector_in_frame(context, context_placement.key, state.route_frame, plain)?;
        let (target_query, _, target_transport) =
            self.local_vector_in_frame(target, target_placement.query, state.route_frame, plain)?;
        let (negative_query, _, negative_transport) = self.local_vector_in_frame(
            negative,
            negative_placement.query,
            state.route_frame,
            plain,
        )?;
        let read = matrix_vector(aggregate, context_key);
        let query_contrast = subtract(target_query, negative_query);
        let key_gradient_current = matrix_transpose_vector(aggregate, query_contrast);
        let rate = self.config.learning_rate;
        let key_step = scale(
            matrix_transpose_vector(context_transport, key_gradient_current),
            rate,
        );
        let target_query_step = scale(matrix_transpose_vector(target_transport, read), rate);
        let negative_query_step = scale(matrix_transpose_vector(negative_transport, read), rate);
        // One-step write-role surrogate for V_y. Derive the exact cumulative
        // destination frame that observe(y) will enter, transport K(context),
        // Q(y), and Q(negative) into that frame, and differentiate the new
        // multibank delta-write contribution to the deployed Q readout. For
        // bank s, d(S_s K)/dV_y contributes eta_s * ||K||^2. The read weights
        // combine those terms before the contrast is backtransported to y's
        // token-local V coordinates. This is local, causal, and no-BPTT.
        let target_leaf = leaf_for_token(&self.exact_route_leaves, target)
            .map_err(|error| GeometricGatedDeltaRetentionError::ExactRoute(error.to_string()))?;
        let post_observation_frame = state
            .route_frame
            .compose(target_leaf, &self.exact_route_table)
            .map_err(|error| GeometricGatedDeltaRetentionError::ExactRoute(error.to_string()))?;
        let (post_key, _, _) = self.local_vector_in_frame(
            context,
            context_placement.key,
            post_observation_frame,
            plain,
        )?;
        let (post_target_query, _, post_target_transport) = self.local_vector_in_frame(
            target,
            target_placement.query,
            post_observation_frame,
            plain,
        )?;
        let (post_negative_query, _, _) = self.local_vector_in_frame(
            negative,
            negative_placement.query,
            post_observation_frame,
            plain,
        )?;
        let gate_coupling = dot(post_key, post_target_query).abs().clamp(0.0, 1.0);
        let key_norm_squared = dot(post_key, post_key);
        let mut multibank_write_contribution = 0.0;
        for bank_index in 0..BANK_COUNT {
            let eta = (self.config.write_gate[bank_index] * (0.75 + 0.25 * gate_coupling))
                .clamp(0.0, 1.0);
            multibank_write_contribution +=
                self.config.read_weight[bank_index] * eta * key_norm_squared;
        }
        require_finite_scalar(
            multibank_write_contribution,
            "multibank V surrogate contribution",
        )?;
        let value_gradient_post = scale(
            subtract(post_target_query, post_negative_query),
            multibank_write_contribution,
        );
        let value_step = scale(
            matrix_transpose_vector(post_target_transport, value_gradient_post),
            rate,
        );
        let arm_index = usize::from(plain);
        if norm(key_step) > SCORE_EPSILON {
            self.learning_update_counts[arm_index][0] = self.learning_update_counts[arm_index][0]
                .checked_add(1)
                .ok_or_else(|| {
                    GeometricGatedDeltaRetentionError::Arithmetic(
                        "key learning update count overflow".to_owned(),
                    )
                })?;
        }
        if norm(value_step) > SCORE_EPSILON {
            self.learning_update_counts[arm_index][1] = self.learning_update_counts[arm_index][1]
                .checked_add(1)
                .ok_or_else(|| {
                    GeometricGatedDeltaRetentionError::Arithmetic(
                        "value learning update count overflow".to_owned(),
                    )
                })?;
        }
        if norm(target_query_step) > SCORE_EPSILON || norm(negative_query_step) > SCORE_EPSILON {
            self.learning_update_counts[arm_index][2] = self.learning_update_counts[arm_index][2]
                .checked_add(1)
                .ok_or_else(|| {
                    GeometricGatedDeltaRetentionError::Arithmetic(
                        "query learning update count overflow".to_owned(),
                    )
                })?;
        }
        let placements = if plain {
            &mut self.plain_placements
        } else {
            &mut self.geometric_placements
        };
        let context_index = checked_token_index(context, placements.len())?;
        let target_index = checked_token_index(target, placements.len())?;
        let negative_index = checked_token_index(negative, placements.len())?;
        placements[context_index].key = normalize(add(placements[context_index].key, key_step))?;
        placements[target_index].value =
            normalize(add(placements[target_index].value, value_step))?;
        placements[target_index].query =
            normalize(add(placements[target_index].query, target_query_step))?;
        placements[negative_index].query = normalize(subtract(
            placements[negative_index].query,
            negative_query_step,
        ))?;
        Ok(())
    }

    fn read_vector(
        &self,
        state: &GeometricGatedDeltaRetentionState,
        context_token: u32,
    ) -> Result<Vector4, GeometricGatedDeltaRetentionError> {
        let plain = state.control == GeometricRetentionControl::PlainDelta;
        let placement = self.placement(context_token, plain)?;
        let (key, _, _) =
            self.local_vector_in_frame(context_token, placement.key, state.route_frame, plain)?;
        let mut read = zero_vector();
        for (bank_index, bank) in state.banks.iter().enumerate() {
            let weight = if state.control == GeometricRetentionControl::LastOnly {
                if bank_index == 0 {
                    1.0
                } else {
                    0.0
                }
            } else {
                self.config.read_weight[bank_index]
            };
            read = add(read, scale(matrix_vector(bank.memory, key), weight));
        }
        require_finite_vector(read, "retention read")?;
        Ok(read)
    }

    fn aggregate_memory(&self, state: &GeometricGatedDeltaRetentionState) -> Matrix4 {
        let mut aggregate = zero_matrix();
        for (bank_index, bank) in state.banks.iter().enumerate() {
            let weight = if state.control == GeometricRetentionControl::LastOnly {
                if bank_index == 0 {
                    1.0
                } else {
                    0.0
                }
            } else {
                self.config.read_weight[bank_index]
            };
            aggregate = add_matrix(aggregate, scale_matrix(bank.memory, weight));
        }
        aggregate
    }

    /// Map one token-local placement vector into a cumulative route frame.
    /// The exact relative element is `destination * leaf(token)^-1`; the
    /// compiler-side matrix is only a host reference for that exact element.
    fn local_vector_in_frame(
        &self,
        token: u32,
        vector: Vector4,
        destination: ExactSpinState,
        plain: bool,
    ) -> Result<(Vector4, u16, Matrix4), GeometricGatedDeltaRetentionError> {
        let leaf = leaf_for_token(&self.exact_route_leaves, token)
            .map_err(|error| GeometricGatedDeltaRetentionError::ExactRoute(error.to_string()))?;
        let connection = leaf
            .inverse(&self.exact_route_table)
            .and_then(|inverse| destination.compose(inverse, &self.exact_route_table))
            .map_err(|error| GeometricGatedDeltaRetentionError::ExactRoute(error.to_string()))?;
        let transport = if plain {
            identity_matrix()
        } else {
            exact_connection_matrix(connection, &self.exact_route_table)?
        };
        let framed = matrix_vector(transport, vector);
        require_finite_vector(framed, "token-local vector transport")?;
        Ok((framed, connection.table_index().table_offset(), transport))
    }

    fn placement(
        &self,
        token: u32,
        plain: bool,
    ) -> Result<TokenPlacement, GeometricGatedDeltaRetentionError> {
        let placements = if plain {
            &self.plain_placements
        } else {
            &self.geometric_placements
        };
        let index = checked_token_index(token, placements.len())?;
        Ok(placements[index])
    }

    fn validate_learned_state(&self) -> Result<(), GeometricGatedDeltaRetentionError> {
        if self.geometric_placements.len() != self.plain_placements.len()
            || self.geometric_placements.len() != self.exact_route_leaves.len()
        {
            return Err(GeometricGatedDeltaRetentionError::Invalid(
                "retention placement and route namespaces differ".to_owned(),
            ));
        }
        for placement in self
            .geometric_placements
            .iter()
            .chain(&self.plain_placements)
        {
            for vector in [placement.key, placement.value, placement.query] {
                require_finite_vector(vector, "learned placement")?;
                if (norm(vector) - 1.0).abs() > 1.0e-9 {
                    return Err(GeometricGatedDeltaRetentionError::Arithmetic(
                        "learned placement is not unit normalized".to_owned(),
                    ));
                }
            }
            if vector_bits(placement.key) == vector_bits(placement.value)
                || vector_bits(placement.key) == vector_bits(placement.query)
                || vector_bits(placement.value) == vector_bits(placement.query)
            {
                return Err(GeometricGatedDeltaRetentionError::Invalid(
                    "learned key, value, and query placements must remain distinct".to_owned(),
                ));
            }
        }
        for (arm_name, counts) in [
            ("geometric", self.learning_update_counts[0]),
            ("plain", self.learning_update_counts[1]),
        ] {
            if counts.contains(&0) {
                return Err(GeometricGatedDeltaRetentionError::Invalid(format!(
                    "{arm_name} arm did not apply non-zero key, value, and query learning updates"
                )));
            }
        }
        Ok(())
    }
}

fn validate_sequence(
    sequence: &GeometricRetentionConstructionSequence,
    maximum_token_id: u32,
) -> Result<(), GeometricGatedDeltaRetentionError> {
    if sequence.document_id.is_empty() || sequence.document_id.contains(['\n', '\r', '\0']) {
        return Err(GeometricGatedDeltaRetentionError::Invalid(
            "construction document id is empty or contains a control character".to_owned(),
        ));
    }
    if sequence.initial_token > maximum_token_id {
        return Err(GeometricGatedDeltaRetentionError::Invalid(format!(
            "initial token {} exceeds fitted maximum {}",
            sequence.initial_token, maximum_token_id
        )));
    }
    for step in &sequence.steps {
        validate_support(&step.admitted_support, maximum_token_id)?;
        if step.observed_token > maximum_token_id {
            return Err(GeometricGatedDeltaRetentionError::Invalid(format!(
                "observed token {} exceeds fitted maximum {}",
                step.observed_token, maximum_token_id
            )));
        }
        if step
            .admitted_support
            .binary_search(&step.observed_token)
            .is_err()
        {
            return Err(GeometricGatedDeltaRetentionError::Invalid(format!(
                "observed target {} is not in its admitted support",
                step.observed_token
            )));
        }
    }
    Ok(())
}

fn validate_support(
    support: &[u32],
    maximum_token_id: u32,
) -> Result<(), GeometricGatedDeltaRetentionError> {
    if support.is_empty() {
        return Err(GeometricGatedDeltaRetentionError::Invalid(
            "geometric retention requires nonempty admitted support".to_owned(),
        ));
    }
    if support.iter().any(|token| *token > maximum_token_id) {
        return Err(GeometricGatedDeltaRetentionError::Invalid(
            "admitted support contains a token outside the fitted namespace".to_owned(),
        ));
    }
    if support.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(GeometricGatedDeltaRetentionError::Invalid(
            "admitted support must be strictly ascending and duplicate-free".to_owned(),
        ));
    }
    Ok(())
}

fn validate_cid(cid: &str, label: &str) -> Result<(), GeometricGatedDeltaRetentionError> {
    let digest = cid.strip_prefix("blake3:").ok_or_else(|| {
        GeometricGatedDeltaRetentionError::Invalid(format!(
            "{label} CID must use the blake3: prefix"
        ))
    })?;
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(GeometricGatedDeltaRetentionError::Invalid(format!(
            "{label} CID must contain exactly 64 hexadecimal digest characters"
        )));
    }
    Ok(())
}

fn validate_identity(identity: &str, label: &str) -> Result<(), GeometricGatedDeltaRetentionError> {
    if identity.is_empty() || identity.contains(['\n', '\r', '\0']) {
        return Err(GeometricGatedDeltaRetentionError::Invalid(format!(
            "{label} must be nonempty and contain no control characters"
        )));
    }
    Ok(())
}

fn construction_population_kappa(
    sequences: &[&GeometricRetentionConstructionSequence],
    binding: &GeometricRetentionSupportBinding,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4.ggdr-construction-population/1\0");
    hasher.update(GEOMETRIC_RETENTION_SUPPORT_POLICY.as_bytes());
    hash_length_prefixed(&mut hasher, binding.table_artifact_cid.as_bytes());
    hash_length_prefixed(&mut hasher, binding.overlay_artifact_cid.as_bytes());
    hash_length_prefixed(
        &mut hasher,
        binding.construction_partition_identity.as_bytes(),
    );
    hasher.update(&(sequences.len() as u64).to_le_bytes());
    for sequence in sequences {
        hash_length_prefixed(&mut hasher, sequence.document_id.as_bytes());
        hasher.update(&sequence.initial_token.to_le_bytes());
        hasher.update(&(sequence.steps.len() as u64).to_le_bytes());
        for step in &sequence.steps {
            hasher.update(&(step.admitted_support.len() as u64).to_le_bytes());
            for token in &step.admitted_support {
                hasher.update(&token.to_le_bytes());
            }
            hasher.update(&step.observed_token.to_le_bytes());
        }
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn hash_length_prefixed(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

fn exact_leaf_map_kappa(
    leaves: &[ExactSpinState],
    table: &H4BinaryIcosahedralClosure,
) -> Result<String, GeometricGatedDeltaRetentionError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4.ggdr-exact-prime-spin-leaf-map/1\0");
    hasher.update(CANONICAL_PRIME_SPIN_LEAF_POLICY.as_bytes());
    hasher.update(table.h4_root_table_kappa.as_bytes());
    hasher.update(table.multiplication_table_kappa.as_bytes());
    hasher.update(&(leaves.len() as u64).to_le_bytes());
    for (token, leaf) in leaves.iter().copied().enumerate() {
        let trace = leaf
            .trace(table)
            .map_err(|error| GeometricGatedDeltaRetentionError::ExactRoute(error.to_string()))?;
        hasher.update(&(token as u64).to_le_bytes());
        hasher.update(&leaf.table_index().table_offset().to_le_bytes());
        hasher.update(&trace.fiber_q29.to_le_bytes());
        hasher.update(&trace.torsion_q29.to_le_bytes());
        for [integer, phi] in trace.h4_coordinate.scaled_zphi_quaternion {
            hasher.update(&integer.to_le_bytes());
            hasher.update(&phi.to_le_bytes());
        }
    }
    Ok(format!("blake3:{}", hasher.finalize().to_hex()))
}

fn checked_token_index(
    token: u32,
    placement_len: usize,
) -> Result<usize, GeometricGatedDeltaRetentionError> {
    let index = usize::try_from(token).map_err(|_| {
        GeometricGatedDeltaRetentionError::Arithmetic(
            "token identifier does not fit platform index".to_owned(),
        )
    })?;
    if index >= placement_len {
        return Err(GeometricGatedDeltaRetentionError::Invalid(format!(
            "token {token} is outside the learned placement namespace"
        )));
    }
    Ok(index)
}

fn geometric_seed_placement(
    token: u32,
    leaf: ExactSpinState,
    table: &H4BinaryIcosahedralClosure,
) -> Result<TokenPlacement, GeometricGatedDeltaRetentionError> {
    let coordinate = leaf
        .root_coordinate(table)
        .map_err(|error| GeometricGatedDeltaRetentionError::ExactRoute(error.to_string()))?;
    let mut route = [0.0; DIMENSION];
    for (target, [a, b]) in route.iter_mut().zip(coordinate.scaled_zphi_quaternion) {
        *target = (a as f64 + b as f64 * GOLDEN_RATIO) * 0.5;
    }
    route = normalize(route)?;
    let token_jitter = deterministic_unit_vector(b"uor-r4.ggdr.geometric-jitter/1", token);
    let key = normalize(add(route, scale(token_jitter, 0.031_25)))?;
    let value = normalize(add(
        [-route[1], route[0], route[3], -route[2]],
        scale(token_jitter, 0.019_531_25),
    ))?;
    let query = normalize(add(
        [route[2], -route[3], -route[0], route[1]],
        scale(token_jitter, -0.011_718_75),
    ))?;
    Ok(TokenPlacement { key, value, query })
}

fn plain_seed_placement(token: u32) -> TokenPlacement {
    TokenPlacement {
        key: deterministic_unit_vector(b"uor-r4.ggdr.plain-key/1", token),
        value: deterministic_unit_vector(b"uor-r4.ggdr.plain-value/1", token),
        query: deterministic_unit_vector(b"uor-r4.ggdr.plain-query/1", token),
    }
}

fn deterministic_unit_vector(domain: &[u8], token: u32) -> Vector4 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(&token.to_le_bytes());
    let mut reader = hasher.finalize_xof();
    let mut bytes = [0_u8; 32];
    reader.fill(&mut bytes);
    let mut vector = [0.0; DIMENSION];
    for (index, target) in vector.iter_mut().enumerate() {
        let offset = index * 8;
        let raw = u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap_or([0; 8]));
        let unit = (raw as f64) / (u64::MAX as f64);
        *target = unit.mul_add(2.0, -1.0);
    }
    // A BLAKE3 XOF cannot produce the all-zero vector for every lane under
    // this policy in practice; keep a deterministic fail-safe nonetheless.
    normalize(vector).unwrap_or([1.0, 0.0, 0.0, 0.0])
}

fn exact_connection_matrix(
    connection: ExactSpinState,
    table: &H4BinaryIcosahedralClosure,
) -> Result<Matrix4, GeometricGatedDeltaRetentionError> {
    let trace = connection
        .trace(table)
        .map_err(|error| GeometricGatedDeltaRetentionError::ExactRoute(error.to_string()))?;
    let mut quaternion = [0.0; DIMENSION];
    for (target, [a, b]) in quaternion
        .iter_mut()
        .zip(trace.h4_coordinate.scaled_zphi_quaternion)
    {
        *target = (a as f64 + b as f64 * GOLDEN_RATIO) * 0.5;
    }
    quaternion = normalize(quaternion)?;
    let [w, x, y, z] = quaternion;
    let transport = [[w, -x, -y, -z], [x, w, -z, y], [y, z, w, -x], [z, -y, x, w]];
    // The exact route law carries two additional central Q29 phases. There is
    // no faithful independent T^2 action in the four-dimensional irreducible
    // left-quaternion representation that also commutes with every H4
    // element. Arbitrary plane rotations break connection composition. This
    // V1 therefore uses the genuine H4/Spin projection as P and binds both
    // phase coordinates in the exact leaf-map kappa; a higher-dimensional
    // phase representation is a separate design.
    require_finite_matrix(transport, "exact connection transport")?;
    let orthogonality = matrix_multiply(transport, transpose(transport));
    for (row, values) in orthogonality.iter().enumerate() {
        for (column, value) in values.iter().enumerate() {
            let expected = if row == column { 1.0 } else { 0.0 };
            if (*value - expected).abs() > 1.0e-9 {
                return Err(GeometricGatedDeltaRetentionError::Arithmetic(
                    "exact-route-derived transport is not numerically orthogonal".to_owned(),
                ));
            }
        }
    }
    Ok(transport)
}

fn zero_vector() -> Vector4 {
    [0.0; DIMENSION]
}

fn zero_matrix() -> Matrix4 {
    [[0.0; DIMENSION]; DIMENSION]
}

fn identity_matrix() -> Matrix4 {
    let mut matrix = zero_matrix();
    for (index, row) in matrix.iter_mut().enumerate() {
        row[index] = 1.0;
    }
    matrix
}

fn add(left: Vector4, right: Vector4) -> Vector4 {
    let mut result = zero_vector();
    for index in 0..DIMENSION {
        result[index] = left[index] + right[index];
    }
    result
}

fn subtract(left: Vector4, right: Vector4) -> Vector4 {
    let mut result = zero_vector();
    for index in 0..DIMENSION {
        result[index] = left[index] - right[index];
    }
    result
}

fn scale(vector: Vector4, scalar: f64) -> Vector4 {
    let mut result = zero_vector();
    for index in 0..DIMENSION {
        result[index] = vector[index] * scalar;
    }
    result
}

fn dot(left: Vector4, right: Vector4) -> f64 {
    left.iter()
        .zip(right)
        .map(|(left, right)| left * right)
        .sum()
}

fn norm(vector: Vector4) -> f64 {
    dot(vector, vector).sqrt()
}

fn normalize(vector: Vector4) -> Result<Vector4, GeometricGatedDeltaRetentionError> {
    require_finite_vector(vector, "vector normalization input")?;
    let magnitude = norm(vector);
    if !magnitude.is_finite() || magnitude <= f64::EPSILON {
        return Err(GeometricGatedDeltaRetentionError::Arithmetic(
            "cannot normalize a zero or non-finite vector".to_owned(),
        ));
    }
    Ok(scale(vector, magnitude.recip()))
}

fn matrix_vector(matrix: Matrix4, vector: Vector4) -> Vector4 {
    let mut result = zero_vector();
    for row in 0..DIMENSION {
        result[row] = dot(matrix[row], vector);
    }
    result
}

fn matrix_transpose_vector(matrix: Matrix4, vector: Vector4) -> Vector4 {
    matrix_vector(transpose(matrix), vector)
}

fn transpose(matrix: Matrix4) -> Matrix4 {
    let mut result = zero_matrix();
    for row in 0..DIMENSION {
        for column in 0..DIMENSION {
            result[row][column] = matrix[column][row];
        }
    }
    result
}

fn matrix_multiply(left: Matrix4, right: Matrix4) -> Matrix4 {
    let mut result = zero_matrix();
    for row in 0..DIMENSION {
        for column in 0..DIMENSION {
            for inner in 0..DIMENSION {
                result[row][column] += left[row][inner] * right[inner][column];
            }
        }
    }
    result
}

fn conjugate(memory: Matrix4, transport: Matrix4) -> Matrix4 {
    matrix_multiply(transport, matrix_multiply(memory, transpose(transport)))
}

fn outer(left: Vector4, right: Vector4) -> Matrix4 {
    let mut result = zero_matrix();
    for row in 0..DIMENSION {
        for column in 0..DIMENSION {
            result[row][column] = left[row] * right[column];
        }
    }
    result
}

fn scale_matrix(matrix: Matrix4, scalar: f64) -> Matrix4 {
    let mut result = zero_matrix();
    for row in 0..DIMENSION {
        for column in 0..DIMENSION {
            result[row][column] = matrix[row][column] * scalar;
        }
    }
    result
}

fn add_matrix(left: Matrix4, right: Matrix4) -> Matrix4 {
    let mut result = zero_matrix();
    for row in 0..DIMENSION {
        for column in 0..DIMENSION {
            result[row][column] = left[row][column] + right[row][column];
        }
    }
    result
}

fn require_finite_scalar(value: f64, label: &str) -> Result<(), GeometricGatedDeltaRetentionError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(GeometricGatedDeltaRetentionError::Arithmetic(format!(
            "{label} is non-finite"
        )))
    }
}

fn require_finite_vector(
    vector: Vector4,
    label: &str,
) -> Result<(), GeometricGatedDeltaRetentionError> {
    if vector.iter().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(GeometricGatedDeltaRetentionError::Arithmetic(format!(
            "{label} contains a non-finite value"
        )))
    }
}

fn require_finite_matrix(
    matrix: Matrix4,
    label: &str,
) -> Result<(), GeometricGatedDeltaRetentionError> {
    if matrix.iter().flatten().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(GeometricGatedDeltaRetentionError::Arithmetic(format!(
            "{label} contains a non-finite value"
        )))
    }
}

fn vector_bits(vector: Vector4) -> [u64; DIMENSION] {
    vector.map(f64::to_bits)
}

fn exact_route_trace(state: ExactSpinState) -> GeometricRetentionExactRouteTrace {
    GeometricRetentionExactRouteTrace {
        h4_table_offset: state.table_index().table_offset(),
        fiber_q29: state.fiber_q29(),
        torsion_q29: state.torsion_q29(),
    }
}

fn state_checksum(state: &GeometricGatedDeltaRetentionState) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4.ggdr-state/1\0");
    hasher.update(&[state.control as u8]);
    hasher.update(&state.observations.to_le_bytes());
    let route = exact_route_trace(state.route_frame);
    hasher.update(&route.h4_table_offset.to_le_bytes());
    hasher.update(&route.fiber_q29.to_le_bytes());
    hasher.update(&route.torsion_q29.to_le_bytes());
    hasher.update(&state.last_token.unwrap_or(u32::MAX).to_le_bytes());
    for bank in &state.banks {
        for value in bank.memory.iter().flatten() {
            hasher.update(&value.to_bits().to_le_bytes());
        }
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn push_bytes(target: &mut Vec<u8>, value: &[u8]) {
    target.extend_from_slice(&(value.len() as u64).to_le_bytes());
    target.extend_from_slice(value);
}

fn push_f64_array(target: &mut Vec<u8>, values: [f64; BANK_COUNT]) {
    for value in values {
        target.extend_from_slice(&value.to_bits().to_le_bytes());
    }
}

fn push_placement(target: &mut Vec<u8>, placement: TokenPlacement) {
    for vector in [placement.key, placement.value, placement.query] {
        for value in vector {
            target.extend_from_slice(&value.to_bits().to_le_bytes());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_sequences() -> Vec<GeometricRetentionConstructionSequence> {
        vec![
            GeometricRetentionConstructionSequence {
                document_id: "construction-a".to_owned(),
                initial_token: 1,
                steps: vec![
                    GeometricRetentionConstructionStep {
                        admitted_support: vec![2, 3],
                        observed_token: 2,
                    },
                    GeometricRetentionConstructionStep {
                        admitted_support: vec![1, 3],
                        observed_token: 3,
                    },
                    GeometricRetentionConstructionStep {
                        admitted_support: vec![1, 2],
                        observed_token: 1,
                    },
                ],
            },
            GeometricRetentionConstructionSequence {
                document_id: "construction-b".to_owned(),
                initial_token: 2,
                steps: vec![
                    GeometricRetentionConstructionStep {
                        admitted_support: vec![1, 3],
                        observed_token: 3,
                    },
                    GeometricRetentionConstructionStep {
                        admitted_support: vec![1, 2],
                        observed_token: 1,
                    },
                    GeometricRetentionConstructionStep {
                        admitted_support: vec![2, 3],
                        observed_token: 2,
                    },
                ],
            },
        ]
    }

    fn fixture_binding() -> GeometricRetentionSupportBinding {
        let table_cid = format!(
            "blake3:{}",
            blake3::hash(b"fixture-source-free-table").to_hex()
        );
        let overlay_cid = format!(
            "blake3:{}",
            blake3::hash(b"fixture-multiscale-overlay").to_hex()
        );
        GeometricRetentionSupportBinding::new(table_cid, overlay_cid, "fixture-d3-construction/1")
            .expect("support binding")
    }

    fn fixture_model() -> GeometricGatedDeltaRetentionR4V1 {
        GeometricGatedDeltaRetentionR4V1::compile(
            8,
            &fixture_sequences(),
            GeometricGatedDeltaRetentionConfig {
                epochs: 2,
                ..GeometricGatedDeltaRetentionConfig::default()
            },
            fixture_binding(),
        )
        .expect("fixture compiles")
    }

    #[test]
    fn two_compiles_are_byte_identical_and_k_v_q_remain_distinct() {
        let left = fixture_model();
        let right = fixture_model();
        assert_eq!(left.to_bytes(), right.to_bytes());
        assert_eq!(left.artifact_cid(), right.artifact_cid());
        for token in 0..=left.maximum_token_id() {
            assert!(
                left.placement_trace(token, false)
                    .expect("geometric placement")
                    .pairwise_distinct
            );
            assert!(
                left.placement_trace(token, true)
                    .expect("plain placement")
                    .pairwise_distinct
            );
        }
    }

    #[test]
    fn v_receives_a_real_candidate_conditioned_write_role_update() {
        let model = fixture_model();
        let token = 2;
        let leaf = leaf_for_token(&model.exact_route_leaves, token).expect("leaf");
        let geometric_seed =
            geometric_seed_placement(token, leaf, &model.exact_route_table).expect("seed");
        let plain_seed = plain_seed_placement(token);
        let geometric_learned = model.placement(token, false).expect("geometric learned");
        let plain_learned = model.placement(token, true).expect("plain learned");
        assert_ne!(
            vector_bits(geometric_seed.value),
            vector_bits(geometric_learned.value)
        );
        assert_ne!(
            vector_bits(plain_seed.value),
            vector_bits(plain_learned.value)
        );
        for counts in model.learning_update_counts() {
            assert!(counts.iter().all(|count| *count > 0));
        }
    }

    #[test]
    fn construction_kappa_binds_partition_events_and_target_membership() {
        let baseline = fixture_model();
        let alternate_partition = GeometricRetentionSupportBinding::new(
            baseline.support_binding().table_artifact_cid(),
            baseline.support_binding().overlay_artifact_cid(),
            "fixture-d3-construction/2",
        )
        .expect("alternate partition");
        let alternate = GeometricGatedDeltaRetentionR4V1::compile(
            8,
            &fixture_sequences(),
            GeometricGatedDeltaRetentionConfig {
                epochs: 2,
                ..GeometricGatedDeltaRetentionConfig::default()
            },
            alternate_partition,
        )
        .expect("alternate compiles");
        assert_ne!(
            baseline.construction_population_kappa(),
            alternate.construction_population_kappa()
        );

        let mut invalid = fixture_sequences();
        invalid[0].steps[0].observed_token = 8;
        let error = GeometricGatedDeltaRetentionR4V1::compile(
            8,
            &invalid,
            GeometricGatedDeltaRetentionConfig::default(),
            fixture_binding(),
        )
        .expect_err("target outside support must fail closed");
        assert!(error.to_string().contains("not in its admitted support"));
    }

    #[test]
    fn prediction_is_pre_observation_and_support_is_unchanged() {
        let model = fixture_model();
        let mut state = model
            .start_state(GeometricRetentionControl::FullGeometric)
            .expect("state");
        model.observe(&mut state, 1).expect("causal seed");
        let support = vec![2, 3];
        let before_left = model.predict(&state, &support).expect("prediction");
        let before_right = model.predict(&state, &support).expect("prediction");
        assert_eq!(before_left, before_right);
        assert_eq!(before_left.admitted_support, support);

        let mut target_two = state.clone();
        let mut target_three = state.clone();
        model.observe(&mut target_two, 2).expect("observe two");
        model.observe(&mut target_three, 3).expect("observe three");
        assert_ne!(
            model.state_checksum(&target_two),
            model.state_checksum(&target_three)
        );
        assert_eq!(
            before_left.selected_token, before_right.selected_token,
            "future target mutation cannot alter the already-made choice"
        );
    }

    #[test]
    fn route_order_transport_and_delta_interventions_change_bounded_state() {
        let model = fixture_model();
        let controls = [
            GeometricRetentionControl::FullGeometric,
            GeometricRetentionControl::PlainDelta,
            GeometricRetentionControl::NoDeltaOverwrite,
            GeometricRetentionControl::TransportPermuted,
            GeometricRetentionControl::LeftFoldRoute,
            GeometricRetentionControl::LastOnly,
        ];
        let mut checksums = BTreeSet::new();
        for control in controls {
            let mut state = model.start_state(control).expect("state");
            model.observe(&mut state, 1).expect("seed");
            model.observe(&mut state, 2).expect("second");
            let trace = model.observe(&mut state, 3).expect("third");
            assert!(trace.write_performed);
            assert!(checksums.insert(model.state_checksum(&state)));
            let prediction = model.predict(&state, &[1, 2, 3]).expect("prediction");
            assert_eq!(prediction.admitted_support, vec![1, 2, 3]);
            assert_eq!(state.bounded_scalar_state_len(), 64);
        }
    }

    #[test]
    fn delta_overwrite_changes_a_key_specific_association() {
        let model = fixture_model();
        let mut full = model
            .start_state(GeometricRetentionControl::FullGeometric)
            .expect("full state");
        let mut no_delta = model
            .start_state(GeometricRetentionControl::NoDeltaOverwrite)
            .expect("no-delta state");
        for token in [1, 2, 1, 3] {
            let full_trace = model.observe(&mut full, token).expect("full observation");
            let no_delta_trace = model
                .observe(&mut no_delta, token)
                .expect("no-delta observation");
            assert_eq!(full_trace.write_performed, no_delta_trace.write_performed);
        }
        let full_score = model
            .association_score(&full, 0, 1, 3)
            .expect("full association");
        let key_placement = model.placement(1, false).expect("key placement");
        let candidate_placement = model.placement(3, false).expect("candidate placement");
        let (key, _, _) = model
            .local_vector_in_frame(1, key_placement.key, full.route_frame, false)
            .expect("key frame");
        let (candidate_query, _, _) = model
            .local_vector_in_frame(3, candidate_placement.query, full.route_frame, false)
            .expect("candidate frame");
        let direct = dot(candidate_query, matrix_vector(full.banks[0].memory, key));
        assert!((full_score - direct).abs() <= 1.0e-12);
        let no_delta_score = model
            .association_score(&no_delta, 0, 1, 3)
            .expect("no-delta association");
        assert!((full_score - no_delta_score).abs() > 1.0e-9);
        assert_ne!(model.state_checksum(&full), model.state_checksum(&no_delta));
    }

    #[test]
    fn state_identity_and_observation_trace_bind_exact_route_phases() {
        let model = fixture_model();
        let state = model
            .start_state(GeometricRetentionControl::FullGeometric)
            .expect("state");
        let mut phase_variant = state.clone();
        phase_variant.route_frame = ExactSpinState::from_table_index_and_phases(
            state.route_frame.table_index(),
            17,
            -29,
            &model.exact_route_table,
        )
        .expect("phase variant");
        assert_eq!(
            state.route_frame.table_index(),
            phase_variant.route_frame.table_index()
        );
        assert_ne!(
            model.state_checksum(&state),
            model.state_checksum(&phase_variant)
        );
        let variant_trace = exact_route_trace(phase_variant.route_frame);
        assert_eq!(variant_trace.fiber_q29, 17);
        assert_eq!(variant_trace.torsion_q29, -29);

        let mut observed = model
            .start_state(GeometricRetentionControl::FullGeometric)
            .expect("observed state");
        let trace = model.observe(&mut observed, 1).expect("observation");
        assert_eq!(trace.current_route, exact_route_trace(observed.route_frame));
        assert_eq!(trace.previous_route.fiber_q29, 0);
        assert_eq!(trace.previous_route.torsion_q29, 0);
    }

    #[test]
    fn exact_h4_connection_transport_composes_and_is_frame_equivariant() {
        let model = fixture_model();
        let table = &model.exact_route_table;
        let a = leaf_for_token(&model.exact_route_leaves, 1).expect("route a");
        let b = a
            .compose(
                leaf_for_token(&model.exact_route_leaves, 2).expect("leaf b"),
                table,
            )
            .expect("route b");
        let c = b
            .compose(
                leaf_for_token(&model.exact_route_leaves, 3).expect("leaf c"),
                table,
            )
            .expect("route c");
        let connection = |destination: ExactSpinState, source: ExactSpinState| {
            source
                .inverse(table)
                .and_then(|inverse| destination.compose(inverse, table))
                .expect("exact connection")
        };
        let p_ba = exact_connection_matrix(connection(b, a), table).expect("P(b,a)");
        let p_cb = exact_connection_matrix(connection(c, b), table).expect("P(c,b)");
        let p_ca = exact_connection_matrix(connection(c, a), table).expect("P(c,a)");
        let composed = matrix_multiply(p_cb, p_ba);
        for row in 0..DIMENSION {
            for column in 0..DIMENSION {
                assert!(
                    (composed[row][column] - p_ca[row][column]).abs() <= 1.0e-12,
                    "connection representation must compose at ({row},{column})"
                );
            }
        }

        let local = model.placement_trace(1, false).expect("placement").query;
        let direct = matrix_vector(p_ca, local);
        let stepwise = matrix_vector(p_cb, matrix_vector(p_ba, local));
        let (helper, _, _) = model
            .local_vector_in_frame(1, local, c, false)
            .expect("direct token-local transport");
        for index in 0..DIMENSION {
            assert!((direct[index] - stepwise[index]).abs() <= 1.0e-12);
            assert!((direct[index] - helper[index]).abs() <= 1.0e-12);
        }
    }
}

//! Construction-transfer candidate-conditioned causal-return hypothesis.
//!
//! This is the frozen candidate mechanism under qualification in issue #983;
//! its existence does not establish attention. Raw queries accept only an
//! already-observed route history and candidates naturally admitted by the
//! existing schema-2 support path. Construction actions require a separately
//! frozen partition authorization and are never an input to raw encoding.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::canonical_lexical_ingestion::{
    h4_leaf_state_for_address, validate_ordered_h4_table_exact, CanonicalLexicalCodec,
    CanonicalLexicalError, CanonicalRouteArtifact, H4BinaryIcosahedralClosure, H4RootCoordinate,
    OrderedH4FoldState,
};
use crate::prime_route_attention::{GeometricAddress, PrimeRouteError};
use crate::prime_route_geometric_attention::{
    AttentionSourceCounts, AttentionSupportTrace, CausalPathAttentionState,
    GeometricAttentionArtifact, GeometricAttentionError, H4S3AngularShell,
};

pub const CONSTRUCTION_CAUSAL_RETURN_FIXTURE_IDENTITY: &str =
    "uor-r4.construction-causal-return-fixture/1";
pub const CONSTRUCTION_CAUSAL_RETURN_CODEC_IDENTITY: &str =
    "uor-r4.construction-causal-return-codec/1";
pub const CONSTRUCTION_CAUSAL_RETURN_CLASS_MAP_IDENTITY: &str =
    "uor-r4.construction-causal-return-class-map/1";
pub const CONSTRUCTION_CAUSAL_RETURN_POLICY_IDENTITY: &str =
    "uor-r4.construction-causal-return-policy/1";
pub const CONSTRUCTION_CAUSAL_RETURN_VALIDATION_INPUT_IDENTITY: &str =
    "uor-r4.construction-causal-return-validation-input/1";
pub const CONSTRUCTION_CAUSAL_RETURN_RAW_CENSUS_IDENTITY: &str =
    "uor-r4.construction-causal-return-raw-census/1";
pub const CONSTRUCTION_CAUSAL_RETURN_CONSTRUCTION_LABEL_JOIN_IDENTITY: &str =
    "uor-r4.construction-causal-return-construction-label-join/1";
pub const CONSTRUCTION_CAUSAL_RETURN_VALIDATION_LABEL_JOIN_IDENTITY: &str =
    "uor-r4.construction-causal-return-validation-label-join/1";
pub const CONSTRUCTION_CAUSAL_RETURN_OUTCOME_IDENTITY: &str =
    "uor-r4.construction-causal-return-outcome/1";

pub const CONSTRUCTION_CAUSAL_RETURN_SCHEMA: u32 = 1;
pub const CONSTRUCTION_CAUSAL_RETURN_PREFIX_SLOTS: usize = 8;
pub const CONSTRUCTION_CAUSAL_RETURN_CANDIDATES_PER_DECISION: usize = 2;
pub const CONSTRUCTION_CAUSAL_RETURN_PROTOTYPES_PER_CANDIDATE: usize = 2;
pub const CONSTRUCTION_CAUSAL_RETURN_CONSTRUCTION_TRANSITIONS: usize = 12;
pub const CONSTRUCTION_CAUSAL_RETURN_CONSTRUCTION_ROWS: usize = 24;
pub const CONSTRUCTION_CAUSAL_RETURN_CONSTRUCTION_CANDIDATES: usize = 6;
pub const CONSTRUCTION_CAUSAL_RETURN_FRAME_MISMATCH: &str = "UNAVAILABLE_FRAME_MISMATCH";

const CLASS_SERIALIZATION_ORDER: [&str; 5] = [
    "exact_signed_h4_coordinate",
    "angular_shell",
    "observed_lease_age_increasing",
    "multiplicity_increasing",
    "occupancy",
];
const R_MIN_SELECTION_ORDER: [&str; 3] = [
    "angular_shell_ascending",
    "multiplicity_descending",
    "observed_lease_age_ascending",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstructionCausalReturnError {
    Attention(String),
    Canonical(String),
    PrimeRoute(String),
    Serialization(String),
    Invalid(String),
    ArithmeticOverflow,
    UnavailableFrameMismatch,
}

impl std::fmt::Display for ConstructionCausalReturnError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Attention(reason) => write!(formatter, "geometric attention: {reason}"),
            Self::Canonical(reason) => write!(formatter, "canonical H4 operation: {reason}"),
            Self::PrimeRoute(reason) => write!(formatter, "prime-route address: {reason}"),
            Self::Serialization(reason) => write!(formatter, "canonical serialization: {reason}"),
            Self::Invalid(reason) => write!(formatter, "invalid causal-return mechanism: {reason}"),
            Self::ArithmeticOverflow => {
                formatter.write_str("causal-return mechanism arithmetic overflow")
            }
            Self::UnavailableFrameMismatch => {
                formatter.write_str(CONSTRUCTION_CAUSAL_RETURN_FRAME_MISMATCH)
            }
        }
    }
}

impl std::error::Error for ConstructionCausalReturnError {}

impl From<GeometricAttentionError> for ConstructionCausalReturnError {
    fn from(error: GeometricAttentionError) -> Self {
        Self::Attention(error.to_string())
    }
}

impl From<CanonicalLexicalError> for ConstructionCausalReturnError {
    fn from(error: CanonicalLexicalError) -> Self {
        Self::Canonical(error.to_string())
    }
}

impl From<PrimeRouteError> for ConstructionCausalReturnError {
    fn from(error: PrimeRouteError) -> Self {
        Self::PrimeRoute(error.to_string())
    }
}

/// The eleven frozen causal derangements.  They are bound into the policy
/// identity even when a caller materializes their reports outside this core
/// real-arm encoder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstructionCausalReturnNegativeControl {
    StateDisabled,
    LastOnly,
    OrderShuffledHistory,
    CausalReturnLeaseDisabled,
    ConstructionContentCurrentPairingShuffle,
    CandidatePrototypePlacementPermutation,
    PrimePlacementPermutation,
    ExactRecallOnly,
    ContentSwap,
    ConstructionKeyShuffle,
    IncoherentCandidateRelabeling,
}

pub const CONSTRUCTION_CAUSAL_RETURN_NEGATIVE_CONTROLS: [ConstructionCausalReturnNegativeControl;
    11] = [
    ConstructionCausalReturnNegativeControl::StateDisabled,
    ConstructionCausalReturnNegativeControl::LastOnly,
    ConstructionCausalReturnNegativeControl::OrderShuffledHistory,
    ConstructionCausalReturnNegativeControl::CausalReturnLeaseDisabled,
    ConstructionCausalReturnNegativeControl::ConstructionContentCurrentPairingShuffle,
    ConstructionCausalReturnNegativeControl::CandidatePrototypePlacementPermutation,
    ConstructionCausalReturnNegativeControl::PrimePlacementPermutation,
    ConstructionCausalReturnNegativeControl::ExactRecallOnly,
    ConstructionCausalReturnNegativeControl::ContentSwap,
    ConstructionCausalReturnNegativeControl::ConstructionKeyShuffle,
    ConstructionCausalReturnNegativeControl::IncoherentCandidateRelabeling,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstructionCausalReturnMetamorphicControl {
    CoherentFullArtifactCandidateRelabeling,
    FullHistoryIncrementalPrefixReproduction,
    DeterministicBuildAndOutcomeReplay,
}

pub const CONSTRUCTION_CAUSAL_RETURN_METAMORPHIC_CONTROLS:
    [ConstructionCausalReturnMetamorphicControl; 3] = [
    ConstructionCausalReturnMetamorphicControl::CoherentFullArtifactCandidateRelabeling,
    ConstructionCausalReturnMetamorphicControl::FullHistoryIncrementalPrefixReproduction,
    ConstructionCausalReturnMetamorphicControl::DeterministicBuildAndOutcomeReplay,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstructionCausalReturnPopulationRole {
    Construction,
    Validation,
}

/// One canonical address-to-leaf rotation for the prime-placement control.
/// Population addresses are sorted in the exact native address order.  The
/// address at index `j` receives the native H4 leaf of `(j+1) mod n`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConstructionCausalReturnPrimePlacementEntry {
    pub canonical_population_index: usize,
    pub address_kappa: String,
    pub native_leaf_coordinate: H4RootCoordinate,
    pub permuted_from_address_kappa: String,
    pub permuted_leaf_coordinate: H4RootCoordinate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConstructionCausalReturnPrimePlacementReport {
    pub schema: u32,
    pub domain: &'static str,
    pub direction: &'static str,
    pub h4_root_table_kappa: String,
    pub multiplication_table_kappa: String,
    pub entries: Vec<ConstructionCausalReturnPrimePlacementEntry>,
    pub permutation_kappa: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ConstructionCausalReturnPrimePlacementSeed<'a> {
    schema: u32,
    domain: &'static str,
    direction: &'static str,
    h4_root_table_kappa: &'a str,
    multiplication_table_kappa: &'a str,
    entries: &'a [ConstructionCausalReturnPrimePlacementEntry],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstructionCausalReturnPrimePlacementPermutation {
    h4_root_table_kappa: String,
    multiplication_table_kappa: String,
    entries: Vec<ConstructionCausalReturnPrimePlacementEntry>,
    leaf_by_address_kappa: BTreeMap<String, OrderedH4FoldState>,
    permutation_kappa: String,
}

impl ConstructionCausalReturnPrimePlacementPermutation {
    pub fn canonical_one_step(
        population_addresses: &[GeometricAddress],
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<Self, ConstructionCausalReturnError> {
        validate_ordered_h4_table_exact(table)?;
        let ordered = population_addresses
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        if ordered.len() != population_addresses.len() || ordered.len() < 2 {
            return Err(ConstructionCausalReturnError::Invalid(
                "prime-placement control requires at least two unique population addresses"
                    .to_owned(),
            ));
        }

        let address_kappas = ordered
            .iter()
            .map(GeometricAddress::canonical_kappa)
            .collect::<Result<Vec<_>, _>>()?;
        let native_leaves = ordered
            .iter()
            .map(|address| h4_leaf_state_for_address(address, table))
            .collect::<Result<Vec<_>, _>>()?;
        let mut entries = Vec::with_capacity(ordered.len());
        let mut leaf_by_address_kappa = BTreeMap::new();
        for target_index in 0..ordered.len() {
            let source_index = (target_index + 1) % ordered.len();
            let target_kappa = address_kappas[target_index].clone();
            let native_leaf = native_leaves[target_index];
            let permuted_leaf = native_leaves[source_index];
            leaf_by_address_kappa.insert(target_kappa.clone(), permuted_leaf);
            entries.push(ConstructionCausalReturnPrimePlacementEntry {
                canonical_population_index: target_index,
                address_kappa: target_kappa,
                native_leaf_coordinate: native_leaf.root_coordinate(table)?,
                permuted_from_address_kappa: address_kappas[source_index].clone(),
                permuted_leaf_coordinate: permuted_leaf.root_coordinate(table)?,
            });
        }
        let mut permutation = Self {
            h4_root_table_kappa: table.h4_root_table_kappa.clone(),
            multiplication_table_kappa: table.multiplication_table_kappa.clone(),
            entries,
            leaf_by_address_kappa,
            permutation_kappa: String::new(),
        };
        permutation.permutation_kappa = permutation.reproduce_permutation_kappa()?;
        Ok(permutation)
    }

    pub fn permutation_kappa(&self) -> &str {
        &self.permutation_kappa
    }

    pub fn entries(&self) -> &[ConstructionCausalReturnPrimePlacementEntry] {
        &self.entries
    }

    pub fn canonical_report(&self) -> ConstructionCausalReturnPrimePlacementReport {
        ConstructionCausalReturnPrimePlacementReport {
            schema: CONSTRUCTION_CAUSAL_RETURN_SCHEMA,
            domain: "uor-r4.construction-causal-return-prime-placement-control/1",
            direction: "canonical target j receives native leaf (j+1) mod population_size",
            h4_root_table_kappa: self.h4_root_table_kappa.clone(),
            multiplication_table_kappa: self.multiplication_table_kappa.clone(),
            entries: self.entries.clone(),
            permutation_kappa: self.permutation_kappa.clone(),
        }
    }

    pub fn reproduce_permutation_kappa(&self) -> Result<String, ConstructionCausalReturnError> {
        canonical_kappa(&ConstructionCausalReturnPrimePlacementSeed {
            schema: CONSTRUCTION_CAUSAL_RETURN_SCHEMA,
            domain: "uor-r4.construction-causal-return-prime-placement-control/1",
            direction: "canonical target j receives native leaf (j+1) mod population_size",
            h4_root_table_kappa: &self.h4_root_table_kappa,
            multiplication_table_kappa: &self.multiplication_table_kappa,
            entries: &self.entries,
        })
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ConstructionCausalReturnError> {
        if self.reproduce_permutation_kappa()? != self.permutation_kappa {
            return Err(ConstructionCausalReturnError::UnavailableFrameMismatch);
        }
        canonical_json(&self.canonical_report())
    }

    fn leaf_for_address(
        &self,
        address: &GeometricAddress,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<OrderedH4FoldState, ConstructionCausalReturnError> {
        if self.h4_root_table_kappa != table.h4_root_table_kappa
            || self.multiplication_table_kappa != table.multiplication_table_kappa
            || self.reproduce_permutation_kappa().ok().as_deref()
                != Some(self.permutation_kappa.as_str())
        {
            return Err(ConstructionCausalReturnError::UnavailableFrameMismatch);
        }
        let address_kappa = address.canonical_kappa()?;
        self.leaf_by_address_kappa
            .get(&address_kappa)
            .copied()
            .ok_or_else(|| {
                ConstructionCausalReturnError::Invalid(
                    "prime-placement population does not cover a required route".to_owned(),
                )
            })
    }
}

/// Immutable, observed-only input for one representation-level control.
/// Construction pairing/key shuffles and candidate relabeling intentionally
/// have no variant here because they act only after a raw report freezes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstructionCausalReturnControlledEncoder {
    StateDisabled,
    LastOnly,
    OrderShuffledHistory,
    CausalReturnLeaseDisabled,
    CandidatePrototypePlacementPermutation,
    PrimePlacementPermutation(ConstructionCausalReturnPrimePlacementPermutation),
    ExactRecallOnly,
    ContentSwap {
        swapped_observed_history: Vec<GeometricAddress>,
    },
}

impl ConstructionCausalReturnControlledEncoder {
    pub const fn control(&self) -> ConstructionCausalReturnNegativeControl {
        match self {
            Self::StateDisabled => ConstructionCausalReturnNegativeControl::StateDisabled,
            Self::LastOnly => ConstructionCausalReturnNegativeControl::LastOnly,
            Self::OrderShuffledHistory => {
                ConstructionCausalReturnNegativeControl::OrderShuffledHistory
            }
            Self::CausalReturnLeaseDisabled => {
                ConstructionCausalReturnNegativeControl::CausalReturnLeaseDisabled
            }
            Self::CandidatePrototypePlacementPermutation => {
                ConstructionCausalReturnNegativeControl::CandidatePrototypePlacementPermutation
            }
            Self::PrimePlacementPermutation(_) => {
                ConstructionCausalReturnNegativeControl::PrimePlacementPermutation
            }
            Self::ExactRecallOnly => ConstructionCausalReturnNegativeControl::ExactRecallOnly,
            Self::ContentSwap { .. } => ConstructionCausalReturnNegativeControl::ContentSwap,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstructionCausalReturnTransitionBindingInput {
    pub transition_id: String,
    pub predecessor_history: Vec<GeometricAddress>,
    pub observed_next: GeometricAddress,
    pub candidate_union: [GeometricAddress; CONSTRUCTION_CAUSAL_RETURN_CANDIDATES_PER_DECISION],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConstructionCausalReturnTransitionBinding {
    pub transition_id: String,
    pub predecessor_history_kappa: String,
    pub predecessor_address_kappas: Vec<String>,
    pub observed_next_address_kappa: String,
    pub candidate_union_address_kappas:
        [String; CONSTRUCTION_CAUSAL_RETURN_CANDIDATES_PER_DECISION],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConstructionCausalReturnConstructionPartitionReport {
    pub schema: u32,
    pub domain: &'static str,
    pub transition_count: usize,
    pub construction_row_count: usize,
    pub candidate_count: usize,
    pub transitions: Vec<ConstructionCausalReturnTransitionBinding>,
    pub partition_kappa: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ConstructionCausalReturnConstructionPartitionSeed<'a> {
    schema: u32,
    domain: &'static str,
    transition_count: usize,
    construction_row_count: usize,
    candidate_count: usize,
    transitions: &'a [ConstructionCausalReturnTransitionBinding],
}

/// Exact authorization boundary between a label-free raw query and the frozen
/// construction label join. Validation histories are absent by construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstructionCausalReturnConstructionPartition {
    transitions: Vec<ConstructionCausalReturnTransitionBinding>,
    transition_by_id: BTreeMap<String, usize>,
    partition_kappa: String,
}

impl ConstructionCausalReturnConstructionPartition {
    pub fn compile(
        inputs: &[ConstructionCausalReturnTransitionBindingInput],
    ) -> Result<Self, ConstructionCausalReturnError> {
        if inputs.len() != CONSTRUCTION_CAUSAL_RETURN_CONSTRUCTION_TRANSITIONS {
            return Err(ConstructionCausalReturnError::Invalid(format!(
                "construction partition requires exactly {CONSTRUCTION_CAUSAL_RETURN_CONSTRUCTION_TRANSITIONS} distinct transitions"
            )));
        }
        let mut transitions = Vec::with_capacity(inputs.len());
        let mut transition_ids = BTreeSet::new();
        let mut observed_next_counts = BTreeMap::<String, usize>::new();
        let mut candidate_union_counts = BTreeMap::<String, usize>::new();
        let mut predecessor_histories = BTreeSet::new();
        for input in inputs {
            if input.transition_id.trim().is_empty()
                || !transition_ids.insert(input.transition_id.clone())
                || input.predecessor_history.is_empty()
                || input.predecessor_history.len() > CONSTRUCTION_CAUSAL_RETURN_PREFIX_SLOTS
            {
                return Err(ConstructionCausalReturnError::Invalid(
                    "construction transitions require unique non-empty IDs and 1--8 route predecessors"
                        .to_owned(),
                ));
            }
            let predecessor_address_kappas = address_kappas(&input.predecessor_history)?;
            let predecessor_history_kappa =
                history_kappa_from_address_kappas(&predecessor_address_kappas)?;
            if !predecessor_histories.insert(predecessor_history_kappa.clone()) {
                return Err(ConstructionCausalReturnError::Invalid(
                    "construction predecessor histories must be distinct".to_owned(),
                ));
            }
            let observed_next_address_kappa = input.observed_next.canonical_kappa()?;
            let mut candidate_union_address_kappas = input
                .candidate_union
                .iter()
                .map(GeometricAddress::canonical_kappa)
                .collect::<Result<Vec<_>, _>>()?;
            candidate_union_address_kappas.sort();
            candidate_union_address_kappas.dedup();
            if candidate_union_address_kappas.len()
                != CONSTRUCTION_CAUSAL_RETURN_CANDIDATES_PER_DECISION
                || !candidate_union_address_kappas.contains(&observed_next_address_kappa)
            {
                return Err(ConstructionCausalReturnError::Invalid(
                    "each construction transition requires two distinct candidates including the observed next"
                        .to_owned(),
                ));
            }
            let candidate_union_address_kappas: [String;
                CONSTRUCTION_CAUSAL_RETURN_CANDIDATES_PER_DECISION] =
                candidate_union_address_kappas.try_into().map_err(|_| {
                    ConstructionCausalReturnError::Invalid(
                        "construction candidate union has the wrong width".to_owned(),
                    )
                })?;
            increment_usize_count(
                &mut observed_next_counts,
                observed_next_address_kappa.clone(),
            )?;
            for candidate in &candidate_union_address_kappas {
                increment_usize_count(&mut candidate_union_counts, candidate.clone())?;
            }
            transitions.push(ConstructionCausalReturnTransitionBinding {
                transition_id: input.transition_id.clone(),
                predecessor_history_kappa,
                predecessor_address_kappas,
                observed_next_address_kappa,
                candidate_union_address_kappas,
            });
        }
        if observed_next_counts.len() != CONSTRUCTION_CAUSAL_RETURN_CONSTRUCTION_CANDIDATES
            || candidate_union_counts.len() != CONSTRUCTION_CAUSAL_RETURN_CONSTRUCTION_CANDIDATES
            || observed_next_counts
                .values()
                .any(|count| *count != CONSTRUCTION_CAUSAL_RETURN_PROTOTYPES_PER_CANDIDATE)
            || candidate_union_counts
                .values()
                .any(|count| *count != CONSTRUCTION_CAUSAL_RETURN_PROTOTYPES_PER_CANDIDATE * 2)
        {
            return Err(ConstructionCausalReturnError::Invalid(
                "construction partition requires six candidates, two observed transitions and two matched reject rows per candidate"
                    .to_owned(),
            ));
        }

        transitions.sort_by(|left, right| left.transition_id.cmp(&right.transition_id));
        let transition_by_id = transitions
            .iter()
            .enumerate()
            .map(|(index, transition)| (transition.transition_id.clone(), index))
            .collect();
        let mut partition = Self {
            transitions,
            transition_by_id,
            partition_kappa: String::new(),
        };
        partition.partition_kappa = partition.reproduce_partition_kappa()?;
        Ok(partition)
    }

    pub fn partition_kappa(&self) -> &str {
        &self.partition_kappa
    }

    pub fn transitions(&self) -> &[ConstructionCausalReturnTransitionBinding] {
        &self.transitions
    }

    pub fn canonical_report(&self) -> ConstructionCausalReturnConstructionPartitionReport {
        ConstructionCausalReturnConstructionPartitionReport {
            schema: CONSTRUCTION_CAUSAL_RETURN_SCHEMA,
            domain: CONSTRUCTION_CAUSAL_RETURN_FIXTURE_IDENTITY,
            transition_count: self.transitions.len(),
            construction_row_count: self.transitions.len()
                * CONSTRUCTION_CAUSAL_RETURN_CANDIDATES_PER_DECISION,
            candidate_count: CONSTRUCTION_CAUSAL_RETURN_CONSTRUCTION_CANDIDATES,
            transitions: self.transitions.clone(),
            partition_kappa: self.partition_kappa.clone(),
        }
    }

    pub fn reproduce_partition_kappa(&self) -> Result<String, ConstructionCausalReturnError> {
        canonical_kappa(&ConstructionCausalReturnConstructionPartitionSeed {
            schema: CONSTRUCTION_CAUSAL_RETURN_SCHEMA,
            domain: CONSTRUCTION_CAUSAL_RETURN_FIXTURE_IDENTITY,
            transition_count: self.transitions.len(),
            construction_row_count: self.transitions.len()
                * CONSTRUCTION_CAUSAL_RETURN_CANDIDATES_PER_DECISION,
            candidate_count: CONSTRUCTION_CAUSAL_RETURN_CONSTRUCTION_CANDIDATES,
            transitions: &self.transitions,
        })
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ConstructionCausalReturnError> {
        if self.reproduce_partition_kappa()? != self.partition_kappa {
            return Err(ConstructionCausalReturnError::UnavailableFrameMismatch);
        }
        canonical_json(&self.canonical_report())
    }

    pub fn authorize_label_join(
        &self,
        raw: &ConstructionCausalReturnRawQuery,
        transition_id: &str,
        observed_next: &GeometricAddress,
    ) -> Result<Vec<ConstructionCausalReturnObservation>, ConstructionCausalReturnError> {
        self.validate_raw_authorization(
            raw.frame.construction_partition_kappa(),
            &raw.observed_history_kappa,
            &raw.candidates
                .iter()
                .map(|candidate| candidate.candidate_address_kappa.clone())
                .collect::<Vec<_>>(),
            transition_id,
            observed_next,
        )?;
        let binding = self.binding(transition_id)?;
        Ok(raw
            .candidates
            .iter()
            .map(|candidate| ConstructionCausalReturnObservation {
                domain: CONSTRUCTION_CAUSAL_RETURN_CONSTRUCTION_LABEL_JOIN_IDENTITY,
                partition_kappa: self.partition_kappa.clone(),
                transition_id: binding.transition_id.clone(),
                predecessor_history_kappa: binding.predecessor_history_kappa.clone(),
                observed_next_address_kappa: binding.observed_next_address_kappa.clone(),
                candidate_address_kappa: candidate.candidate_address_kappa.clone(),
                representation: candidate.representation.clone(),
                action: if candidate.candidate == *observed_next {
                    ConstructionCausalReturnAction::Select
                } else {
                    ConstructionCausalReturnAction::Reject
                },
            })
            .collect())
    }

    pub fn authorize_controlled_label_join(
        &self,
        raw: &ConstructionCausalReturnControlledRawQuery,
        transition_id: &str,
        observed_next: &GeometricAddress,
    ) -> Result<Vec<ConstructionCausalReturnControlledObservation>, ConstructionCausalReturnError>
    {
        if raw.population_role != ConstructionCausalReturnPopulationRole::Construction {
            return Err(ConstructionCausalReturnError::Invalid(
                "validation-role control reports cannot enter a construction label join".to_owned(),
            ));
        }
        self.validate_raw_authorization(
            &raw.construction_partition_kappa,
            &raw.observed_history_kappa,
            &raw.candidates
                .iter()
                .map(|candidate| candidate.candidate_address_kappa.clone())
                .collect::<Vec<_>>(),
            transition_id,
            observed_next,
        )?;
        let binding = self.binding(transition_id)?;
        Ok(raw
            .candidates
            .iter()
            .map(|candidate| ConstructionCausalReturnControlledObservation {
                domain: CONSTRUCTION_CAUSAL_RETURN_CONSTRUCTION_LABEL_JOIN_IDENTITY,
                frame_kappa: raw.frame_kappa.clone(),
                partition_kappa: self.partition_kappa.clone(),
                transition_id: binding.transition_id.clone(),
                predecessor_history_kappa: binding.predecessor_history_kappa.clone(),
                observed_next_address_kappa: binding.observed_next_address_kappa.clone(),
                control: raw.control,
                control_input_kappa: raw.control_input_kappa.clone(),
                candidate_address_kappa: candidate.candidate_address_kappa.clone(),
                representation: candidate.representation.clone(),
                action: if candidate.candidate == *observed_next {
                    ConstructionCausalReturnAction::Select
                } else {
                    ConstructionCausalReturnAction::Reject
                },
            })
            .collect())
    }

    fn binding(
        &self,
        transition_id: &str,
    ) -> Result<&ConstructionCausalReturnTransitionBinding, ConstructionCausalReturnError> {
        self.transition_by_id
            .get(transition_id)
            .and_then(|index| self.transitions.get(*index))
            .ok_or_else(|| {
                ConstructionCausalReturnError::Invalid(
                    "raw history is not an authorized construction transition".to_owned(),
                )
            })
    }

    fn validate_raw_authorization(
        &self,
        frame_partition_kappa: &str,
        observed_history_kappa: &str,
        candidate_address_kappas: &[String],
        transition_id: &str,
        observed_next: &GeometricAddress,
    ) -> Result<(), ConstructionCausalReturnError> {
        if frame_partition_kappa != self.partition_kappa
            || self.reproduce_partition_kappa().ok().as_deref()
                != Some(self.partition_kappa.as_str())
        {
            return Err(ConstructionCausalReturnError::UnavailableFrameMismatch);
        }
        let binding = self.binding(transition_id)?;
        let observed_next_address_kappa = observed_next.canonical_kappa()?;
        let mut candidate_union = candidate_address_kappas.to_vec();
        candidate_union.sort();
        candidate_union.dedup();
        if binding.predecessor_history_kappa != observed_history_kappa
            || binding.observed_next_address_kappa != observed_next_address_kappa
            || candidate_union.as_slice() != binding.candidate_union_address_kappas
        {
            return Err(ConstructionCausalReturnError::Invalid(
                "raw query does not reproduce the frozen construction transition binding"
                    .to_owned(),
            ));
        }
        Ok(())
    }
}

/// Canonical, versioned definition of the fixed mechanism.  Formula strings
/// are report metadata; execution below uses the typed H4 operations directly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConstructionCausalReturnPolicyReport {
    pub schema: u32,
    pub domain: &'static str,
    pub h4_product_orientation: &'static str,
    pub prefix_formula: &'static str,
    pub suffix_formula: &'static str,
    pub relation_formula: &'static str,
    pub excluded_prefix: &'static str,
    pub maximum_observed_prefixes: usize,
    pub fixed_prefix_slots: usize,
    pub candidates_per_decision: usize,
    pub retained_prototypes_per_candidate: usize,
    pub class_serialization_order: [&'static str; 5],
    pub r_min_selection_order: [&'static str; 3],
    pub padding_rule: &'static str,
    pub construction_promotion_rule: &'static str,
    pub selector_rule: &'static str,
    pub negative_controls: [ConstructionCausalReturnNegativeControl; 11],
    pub metamorphic_controls: [ConstructionCausalReturnMetamorphicControl; 3],
    pub diagnostic_comparator: &'static str,
}

pub const fn construction_causal_return_policy_report() -> ConstructionCausalReturnPolicyReport {
    ConstructionCausalReturnPolicyReport {
        schema: CONSTRUCTION_CAUSAL_RETURN_SCHEMA,
        domain: CONSTRUCTION_CAUSAL_RETURN_POLICY_IDENTITY,
        h4_product_orientation: "row-major left * right; quaternion basis (1,i,j,k); right-handed",
        prefix_formula: "P_0=identity; P_i=P_{i-1}*L(x_i)",
        suffix_formula: "S_i=P_i^-1*P_t",
        relation_formula: "R_i=((S_i*L(c))*S_i^-1)*L(c)^-1",
        excluded_prefix: "i=t",
        maximum_observed_prefixes: CONSTRUCTION_CAUSAL_RETURN_PREFIX_SLOTS,
        fixed_prefix_slots: CONSTRUCTION_CAUSAL_RETURN_PREFIX_SLOTS,
        candidates_per_decision: CONSTRUCTION_CAUSAL_RETURN_CANDIDATES_PER_DECISION,
        retained_prototypes_per_candidate: CONSTRUCTION_CAUSAL_RETURN_PROTOTYPES_PER_CANDIDATE,
        class_serialization_order: CLASS_SERIALIZATION_ORDER,
        r_min_selection_order: R_MIN_SELECTION_ORDER,
        padding_rule: "occupancy-false exact-identity P/S/R no-op; never aliases occupied identity",
        construction_promotion_rule:
            "pure R_min resolves directly; only impure R_min promotes to covered pure R_full",
        selector_rule:
            "select iff exactly one admitted candidate is SELECT and the other is REJECT",
        negative_controls: CONSTRUCTION_CAUSAL_RETURN_NEGATIVE_CONTROLS,
        metamorphic_controls: CONSTRUCTION_CAUSAL_RETURN_METAMORPHIC_CONTROLS,
        diagnostic_comparator: "count_only_last_anchor_non_geometric",
    }
}

pub fn construction_causal_return_policy_kappa() -> Result<String, ConstructionCausalReturnError> {
    canonical_kappa(&construction_causal_return_policy_report())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ConstructionCausalReturnFrameSeed<'a> {
    schema: u32,
    domain: &'static str,
    codec_kappa: &'a str,
    vocabulary_kappa: &'a str,
    schema2_manifest_kappa: &'a str,
    route_address_mapping_kappa: &'a str,
    h4_root_table_kappa: &'a str,
    multiplication_table_kappa: &'a str,
    policy_kappa: &'a str,
    maximum_observed_prefixes: usize,
    fixed_prefix_slots: usize,
    candidates_per_decision: usize,
    retained_prototypes_per_candidate: usize,
    class_serialization_order: [&'static str; 5],
    r_min_selection_order: [&'static str; 3],
    occupancy_rule: &'static str,
    construction_partition_kappa: &'a str,
    negative_controls: [ConstructionCausalReturnNegativeControl; 11],
    metamorphic_controls: [ConstructionCausalReturnMetamorphicControl; 3],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConstructionCausalReturnFrameReport {
    pub schema: u32,
    pub domain: &'static str,
    pub codec_kappa: String,
    pub vocabulary_kappa: String,
    pub schema2_manifest_kappa: String,
    pub route_address_mapping_kappa: String,
    pub h4_root_table_kappa: String,
    pub multiplication_table_kappa: String,
    pub policy_kappa: String,
    pub maximum_observed_prefixes: usize,
    pub fixed_prefix_slots: usize,
    pub candidates_per_decision: usize,
    pub retained_prototypes_per_candidate: usize,
    pub class_serialization_order: [&'static str; 5],
    pub r_min_selection_order: [&'static str; 3],
    pub occupancy_rule: &'static str,
    pub construction_partition_kappa: String,
    pub negative_controls: [ConstructionCausalReturnNegativeControl; 11],
    pub metamorphic_controls: [ConstructionCausalReturnMetamorphicControl; 3],
    pub frame_kappa: String,
}

/// Complete compiler/query frame.  Its identity binds the external codec,
/// vocabulary, schema-2 manifest, route-address mapping and construction
/// partition to the repository's exact H4 tables and fixed policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConstructionCausalReturnFrame {
    schema: u32,
    domain: String,
    codec_kappa: String,
    vocabulary_kappa: String,
    schema2_manifest_kappa: String,
    route_address_mapping_kappa: String,
    h4_root_table_kappa: String,
    multiplication_table_kappa: String,
    policy_kappa: String,
    construction_partition_kappa: String,
    frame_kappa: String,
}

impl ConstructionCausalReturnFrame {
    fn new(
        codec_kappa: impl Into<String>,
        vocabulary_kappa: impl Into<String>,
        schema2_manifest_kappa: impl Into<String>,
        route_address_mapping_kappa: impl Into<String>,
        construction_partition_kappa: impl Into<String>,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<Self, ConstructionCausalReturnError> {
        validate_ordered_h4_table_exact(table)?;
        let policy_kappa = construction_causal_return_policy_kappa()?;
        let mut frame = Self {
            schema: CONSTRUCTION_CAUSAL_RETURN_SCHEMA,
            domain: CONSTRUCTION_CAUSAL_RETURN_FIXTURE_IDENTITY.to_owned(),
            codec_kappa: codec_kappa.into(),
            vocabulary_kappa: vocabulary_kappa.into(),
            schema2_manifest_kappa: schema2_manifest_kappa.into(),
            route_address_mapping_kappa: route_address_mapping_kappa.into(),
            h4_root_table_kappa: table.h4_root_table_kappa.clone(),
            multiplication_table_kappa: table.multiplication_table_kappa.clone(),
            policy_kappa,
            construction_partition_kappa: construction_partition_kappa.into(),
            frame_kappa: String::new(),
        };
        frame.validate_labels()?;
        frame.frame_kappa = frame.reproduce_frame_kappa()?;
        Ok(frame)
    }

    /// Bind directly to the native codec and route artifacts.  The route
    /// artifact manifest is the exact route-address mapping identity, while
    /// its embedded schema-2 manifest must match the attention artifact.
    pub fn from_canonical_artifacts(
        codec: &CanonicalLexicalCodec,
        routes: &CanonicalRouteArtifact,
        attention: &GeometricAttentionArtifact,
        construction_partition: &ConstructionCausalReturnConstructionPartition,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<Self, ConstructionCausalReturnError> {
        routes.canonical_bytes()?;
        construction_partition.canonical_bytes()?;
        if codec.codec_kappa() != routes.codec_kappa()
            || codec.vocabulary_kappa() != routes.vocabulary_kappa()
            || routes.embedded_spin_manifest_kappa() != attention.manifest_kappa()
        {
            return Err(ConstructionCausalReturnError::UnavailableFrameMismatch);
        }
        Self::new(
            codec.codec_kappa(),
            codec.vocabulary_kappa(),
            attention.manifest_kappa(),
            routes.manifest_kappa(),
            construction_partition.partition_kappa(),
            table,
        )
    }

    pub fn codec_kappa(&self) -> &str {
        &self.codec_kappa
    }

    pub fn vocabulary_kappa(&self) -> &str {
        &self.vocabulary_kappa
    }

    pub fn schema2_manifest_kappa(&self) -> &str {
        &self.schema2_manifest_kappa
    }

    pub fn route_address_mapping_kappa(&self) -> &str {
        &self.route_address_mapping_kappa
    }

    pub fn h4_root_table_kappa(&self) -> &str {
        &self.h4_root_table_kappa
    }

    pub fn multiplication_table_kappa(&self) -> &str {
        &self.multiplication_table_kappa
    }

    pub fn policy_kappa(&self) -> &str {
        &self.policy_kappa
    }

    pub fn construction_partition_kappa(&self) -> &str {
        &self.construction_partition_kappa
    }

    pub fn frame_kappa(&self) -> &str {
        &self.frame_kappa
    }

    pub fn reproduce_frame_kappa(&self) -> Result<String, ConstructionCausalReturnError> {
        canonical_kappa(&self.seed())
    }

    pub fn canonical_report(&self) -> ConstructionCausalReturnFrameReport {
        ConstructionCausalReturnFrameReport {
            schema: self.schema,
            domain: CONSTRUCTION_CAUSAL_RETURN_FIXTURE_IDENTITY,
            codec_kappa: self.codec_kappa.clone(),
            vocabulary_kappa: self.vocabulary_kappa.clone(),
            schema2_manifest_kappa: self.schema2_manifest_kappa.clone(),
            route_address_mapping_kappa: self.route_address_mapping_kappa.clone(),
            h4_root_table_kappa: self.h4_root_table_kappa.clone(),
            multiplication_table_kappa: self.multiplication_table_kappa.clone(),
            policy_kappa: self.policy_kappa.clone(),
            maximum_observed_prefixes: CONSTRUCTION_CAUSAL_RETURN_PREFIX_SLOTS,
            fixed_prefix_slots: CONSTRUCTION_CAUSAL_RETURN_PREFIX_SLOTS,
            candidates_per_decision: CONSTRUCTION_CAUSAL_RETURN_CANDIDATES_PER_DECISION,
            retained_prototypes_per_candidate: CONSTRUCTION_CAUSAL_RETURN_PROTOTYPES_PER_CANDIDATE,
            class_serialization_order: CLASS_SERIALIZATION_ORDER,
            r_min_selection_order: R_MIN_SELECTION_ORDER,
            occupancy_rule:
                "occupancy-false exact-identity P/S/R no-op; occupied identity remains distinct",
            construction_partition_kappa: self.construction_partition_kappa.clone(),
            negative_controls: CONSTRUCTION_CAUSAL_RETURN_NEGATIVE_CONTROLS,
            metamorphic_controls: CONSTRUCTION_CAUSAL_RETURN_METAMORPHIC_CONTROLS,
            frame_kappa: self.frame_kappa.clone(),
        }
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ConstructionCausalReturnError> {
        self.validate_labels()?;
        if self.reproduce_frame_kappa()? != self.frame_kappa {
            return Err(ConstructionCausalReturnError::UnavailableFrameMismatch);
        }
        canonical_json(&self.canonical_report())
    }

    /// Label-free, future-free raw query encoder.
    pub fn raw_query(
        &self,
        attention: &GeometricAttentionArtifact,
        observed_history: &[GeometricAddress],
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<ConstructionCausalReturnRawQuery, ConstructionCausalReturnError> {
        encode_raw_query(self, attention, observed_history, table)
    }

    /// Audit seam for incremental reproduction. `path_state` may be built by
    /// repeated `observe_path`; `observed_history` supplies only the same
    /// already-observed addresses needed by the unchanged support lookup.
    pub fn raw_query_from_path_state(
        &self,
        attention: &GeometricAttentionArtifact,
        observed_history: &[GeometricAddress],
        path_state: &CausalPathAttentionState,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<ConstructionCausalReturnRawQuery, ConstructionCausalReturnError> {
        encode_raw_query_from_path_state(self, attention, observed_history, path_state, table)
    }

    /// Positive-control seam that reuses the exact natural support frozen in
    /// `raw` after reproducing its frame, observed history, and incremental H4
    /// path. No support object is accepted from the caller.
    pub fn raw_query_from_path_state_and_frozen_raw(
        &self,
        raw: &ConstructionCausalReturnRawQuery,
        observed_history: &[GeometricAddress],
        path_state: &CausalPathAttentionState,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<ConstructionCausalReturnRawQuery, ConstructionCausalReturnError> {
        encode_raw_query_from_path_state_and_frozen_raw(
            self,
            raw,
            observed_history,
            path_state,
            table,
        )
    }

    pub fn controlled_raw_query(
        &self,
        attention: &GeometricAttentionArtifact,
        observed_history: &[GeometricAddress],
        population_role: ConstructionCausalReturnPopulationRole,
        control: &ConstructionCausalReturnControlledEncoder,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<ConstructionCausalReturnControlledRawQuery, ConstructionCausalReturnError> {
        encode_controlled_raw_query(
            self,
            attention,
            observed_history,
            population_role,
            control,
            table,
        )
    }

    /// Recompute only one controlled fixed-width representation while cloning
    /// the exact natural support already frozen in `raw`. Frame/history/support
    /// correspondence is reproduced before any controlled geometry executes.
    pub fn controlled_raw_query_from_frozen_raw(
        &self,
        raw: &ConstructionCausalReturnRawQuery,
        observed_history: &[GeometricAddress],
        population_role: ConstructionCausalReturnPopulationRole,
        control: &ConstructionCausalReturnControlledEncoder,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<ConstructionCausalReturnControlledRawQuery, ConstructionCausalReturnError> {
        encode_controlled_raw_query_from_frozen_raw(
            self,
            raw,
            observed_history,
            population_role,
            control,
            table,
        )
    }

    fn seed(&self) -> ConstructionCausalReturnFrameSeed<'_> {
        ConstructionCausalReturnFrameSeed {
            schema: self.schema,
            domain: CONSTRUCTION_CAUSAL_RETURN_FIXTURE_IDENTITY,
            codec_kappa: &self.codec_kappa,
            vocabulary_kappa: &self.vocabulary_kappa,
            schema2_manifest_kappa: &self.schema2_manifest_kappa,
            route_address_mapping_kappa: &self.route_address_mapping_kappa,
            h4_root_table_kappa: &self.h4_root_table_kappa,
            multiplication_table_kappa: &self.multiplication_table_kappa,
            policy_kappa: &self.policy_kappa,
            maximum_observed_prefixes: CONSTRUCTION_CAUSAL_RETURN_PREFIX_SLOTS,
            fixed_prefix_slots: CONSTRUCTION_CAUSAL_RETURN_PREFIX_SLOTS,
            candidates_per_decision: CONSTRUCTION_CAUSAL_RETURN_CANDIDATES_PER_DECISION,
            retained_prototypes_per_candidate: CONSTRUCTION_CAUSAL_RETURN_PROTOTYPES_PER_CANDIDATE,
            class_serialization_order: CLASS_SERIALIZATION_ORDER,
            r_min_selection_order: R_MIN_SELECTION_ORDER,
            occupancy_rule:
                "occupancy-false exact-identity P/S/R no-op; occupied identity remains distinct",
            construction_partition_kappa: &self.construction_partition_kappa,
            negative_controls: CONSTRUCTION_CAUSAL_RETURN_NEGATIVE_CONTROLS,
            metamorphic_controls: CONSTRUCTION_CAUSAL_RETURN_METAMORPHIC_CONTROLS,
        }
    }

    fn validate_labels(&self) -> Result<(), ConstructionCausalReturnError> {
        if self.schema != CONSTRUCTION_CAUSAL_RETURN_SCHEMA
            || self.domain != CONSTRUCTION_CAUSAL_RETURN_FIXTURE_IDENTITY
        {
            return Err(ConstructionCausalReturnError::UnavailableFrameMismatch);
        }
        for (value, field) in [
            (&self.codec_kappa, "codec kappa"),
            (&self.vocabulary_kappa, "vocabulary kappa"),
            (&self.schema2_manifest_kappa, "schema-2 manifest kappa"),
            (
                &self.route_address_mapping_kappa,
                "route-address mapping kappa",
            ),
            (&self.h4_root_table_kappa, "H4 root-table kappa"),
            (
                &self.multiplication_table_kappa,
                "H4 multiplication-table kappa",
            ),
            (&self.policy_kappa, "causal-return policy kappa"),
            (
                &self.construction_partition_kappa,
                "construction partition kappa",
            ),
        ] {
            validate_blake3_label(value, field)?;
        }
        Ok(())
    }

    fn validate_query_binding(
        &self,
        attention: &GeometricAttentionArtifact,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<(), ConstructionCausalReturnError> {
        self.validate_table_binding(table)?;
        if self.schema2_manifest_kappa != attention.manifest_kappa() {
            return Err(ConstructionCausalReturnError::UnavailableFrameMismatch);
        }
        Ok(())
    }

    fn validate_table_binding(
        &self,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<(), ConstructionCausalReturnError> {
        self.validate_labels()
            .map_err(|_| ConstructionCausalReturnError::UnavailableFrameMismatch)?;
        let exact_table = validate_ordered_h4_table_exact(table).is_ok();
        let reproduced_policy = construction_causal_return_policy_kappa().ok();
        let reproduced_frame = self.reproduce_frame_kappa().ok();
        if !exact_table
            || self.h4_root_table_kappa != table.h4_root_table_kappa
            || self.multiplication_table_kappa != table.multiplication_table_kappa
            || reproduced_policy.as_deref() != Some(self.policy_kappa.as_str())
            || reproduced_frame.as_deref() != Some(self.frame_kappa.as_str())
        {
            return Err(ConstructionCausalReturnError::UnavailableFrameMismatch);
        }
        Ok(())
    }
}

/// Exact serialized class tuple.  Field declaration is the frozen canonical
/// class serialization order.  Deriving `Ord` here is for deterministic
/// artifact inventory only; `R_min` uses the separate explicit comparator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ConstructionCausalReturnClassEvent {
    pub relation_coordinate: H4RootCoordinate,
    pub angular_shell: H4S3AngularShell,
    pub observed_lease_age: u8,
    pub multiplicity: u8,
    pub occupied: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ConstructionCausalReturnFullWord {
    pub slots: [ConstructionCausalReturnClassEvent; CONSTRUCTION_CAUSAL_RETURN_PREFIX_SLOTS],
}

/// Auditable exact witness for one fixed slot.  Padding retains the exact H4
/// identity for every state and coordinate while its class occupancy is false.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ConstructionCausalReturnWitnessSlot {
    pub class_event: ConstructionCausalReturnClassEvent,
    pub slot_index: u8,
    pub prefix_state: OrderedH4FoldState,
    pub prefix_coordinate: H4RootCoordinate,
    pub suffix_state: OrderedH4FoldState,
    pub suffix_coordinate: H4RootCoordinate,
    pub relation_state: OrderedH4FoldState,
    pub relation_coordinate: H4RootCoordinate,
}

/// Candidate-conditioned representation generated only by the raw encoder.
/// Fields are immutable outside this module so label-join callers cannot
/// manufacture or modify a class after observing an outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConstructionCausalReturnRepresentation {
    frame_kappa: String,
    observed_routes: u8,
    candidate_leaf_state: OrderedH4FoldState,
    candidate_leaf_coordinate: H4RootCoordinate,
    r_min: ConstructionCausalReturnClassEvent,
    r_min_slot_index: u8,
    r_full: ConstructionCausalReturnFullWord,
    slots: [ConstructionCausalReturnWitnessSlot; CONSTRUCTION_CAUSAL_RETURN_PREFIX_SLOTS],
}

impl ConstructionCausalReturnRepresentation {
    pub fn frame_kappa(&self) -> &str {
        &self.frame_kappa
    }

    pub const fn observed_routes(&self) -> u8 {
        self.observed_routes
    }

    pub const fn candidate_leaf_state(&self) -> OrderedH4FoldState {
        self.candidate_leaf_state
    }

    pub const fn candidate_leaf_coordinate(&self) -> H4RootCoordinate {
        self.candidate_leaf_coordinate
    }

    pub const fn r_min(&self) -> ConstructionCausalReturnClassEvent {
        self.r_min
    }

    pub const fn r_min_slot_index(&self) -> u8 {
        self.r_min_slot_index
    }

    pub const fn r_full(&self) -> ConstructionCausalReturnFullWord {
        self.r_full
    }

    pub const fn slots(
        &self,
    ) -> &[ConstructionCausalReturnWitnessSlot; CONSTRUCTION_CAUSAL_RETURN_PREFIX_SLOTS] {
        &self.slots
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstructionCausalReturnControlledSlotKind {
    Operative,
    ControlNoOp,
    Padding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ConstructionCausalReturnControlledWitnessSlot {
    pub kind: ConstructionCausalReturnControlledSlotKind,
    pub witness: ConstructionCausalReturnWitnessSlot,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ConstructionCausalReturnExactRecallKey {
    pub predecessor_history_kappa: String,
    pub candidate_address_kappa: String,
}

/// Immutable control-side representation.  `r_min` is absent when a control
/// disables every operative geometric slot or uses exact recall instead of a
/// geometric class.  The eight witnesses and their typed no-ops remain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConstructionCausalReturnControlledRepresentation {
    control: ConstructionCausalReturnNegativeControl,
    observed_routes: u8,
    candidate_leaf_state: OrderedH4FoldState,
    candidate_leaf_coordinate: H4RootCoordinate,
    r_min: Option<ConstructionCausalReturnClassEvent>,
    r_min_slot_index: Option<u8>,
    r_full: ConstructionCausalReturnFullWord,
    exact_recall_key: Option<ConstructionCausalReturnExactRecallKey>,
    slots: [ConstructionCausalReturnControlledWitnessSlot; CONSTRUCTION_CAUSAL_RETURN_PREFIX_SLOTS],
}

impl ConstructionCausalReturnControlledRepresentation {
    pub const fn control(&self) -> ConstructionCausalReturnNegativeControl {
        self.control
    }

    pub const fn observed_routes(&self) -> u8 {
        self.observed_routes
    }

    pub const fn candidate_leaf_state(&self) -> OrderedH4FoldState {
        self.candidate_leaf_state
    }

    pub const fn candidate_leaf_coordinate(&self) -> H4RootCoordinate {
        self.candidate_leaf_coordinate
    }

    pub const fn r_min(&self) -> Option<ConstructionCausalReturnClassEvent> {
        self.r_min
    }

    pub const fn r_min_slot_index(&self) -> Option<u8> {
        self.r_min_slot_index
    }

    pub const fn r_full(&self) -> ConstructionCausalReturnFullWord {
        self.r_full
    }

    pub fn exact_recall_key(&self) -> Option<&ConstructionCausalReturnExactRecallKey> {
        self.exact_recall_key.as_ref()
    }

    pub const fn slots(
        &self,
    ) -> &[ConstructionCausalReturnControlledWitnessSlot; CONSTRUCTION_CAUSAL_RETURN_PREFIX_SLOTS]
    {
        &self.slots
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ConstructionCausalReturnSourceCounts {
    pub last_one: u32,
    pub last_two: u32,
    pub ordered_sentence: u32,
    pub divisor: u32,
    pub adjacent_spin: u32,
}

impl From<AttentionSourceCounts> for ConstructionCausalReturnSourceCounts {
    fn from(counts: AttentionSourceCounts) -> Self {
        Self {
            last_one: counts.last_one,
            last_two: counts.last_two,
            ordered_sentence: counts.ordered_sentence,
            divisor: counts.divisor,
            adjacent_spin: counts.adjacent_spin,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstructionCausalReturnRawCandidate {
    candidate: GeometricAddress,
    candidate_address_kappa: String,
    source_counts: AttentionSourceCounts,
    representation: ConstructionCausalReturnRepresentation,
}

impl ConstructionCausalReturnRawCandidate {
    pub fn candidate(&self) -> &GeometricAddress {
        &self.candidate
    }

    pub fn candidate_address_kappa(&self) -> &str {
        &self.candidate_address_kappa
    }

    pub const fn source_counts(&self) -> AttentionSourceCounts {
        self.source_counts
    }

    pub fn representation(&self) -> &ConstructionCausalReturnRepresentation {
        &self.representation
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConstructionCausalReturnRawCandidateReport {
    pub candidate_address_kappa: String,
    pub source_counts: ConstructionCausalReturnSourceCounts,
    pub representation: ConstructionCausalReturnRepresentation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ConstructionCausalReturnWorkReport {
    pub support_rows_read: usize,
    pub candidate_entries_available: usize,
    pub candidate_entries_examined: usize,
    pub candidate_entries_admitted: usize,
    pub natural_candidates: usize,
    pub observed_prefix_products: usize,
    pub fixed_relation_slots_per_candidate: usize,
    pub relation_slots: usize,
    pub populated_relation_slots: usize,
    pub padded_relation_slots: usize,
    pub h4_leaf_mappings: usize,
    pub h4_product_table_reads: usize,
    pub h4_inverse_table_reads: usize,
    pub declared_prototype_class_slots_per_candidate: usize,
    pub declared_prototype_class_slots: usize,
    pub performed_prototype_class_slot_reads: usize,
    pub declared_payload_inversions_per_candidate: usize,
    pub declared_payload_inversions: usize,
    pub performed_payload_inversions: usize,
    pub source_inputs: usize,
    pub provider_inputs: usize,
    pub teacher_inputs: usize,
    pub future_route_inputs: usize,
    pub validation_label_inputs: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConstructionCausalReturnRawQueryReport {
    pub schema: u32,
    pub domain: &'static str,
    pub frame_kappa: String,
    pub schema2_manifest_kappa: String,
    pub policy_kappa: String,
    pub observed_history_kappa: String,
    pub observed_history_address_kappas: Vec<String>,
    pub support_query_policy_identity: String,
    pub support_query_policy_kappa: String,
    pub support_fallback_active: bool,
    pub support_candidate_ceiling: usize,
    pub support_unique_candidates_before_ceiling: usize,
    pub observed_routes: u8,
    pub populated_padding_aliases: usize,
    pub work: ConstructionCausalReturnWorkReport,
    pub candidates: Vec<ConstructionCausalReturnRawCandidateReport>,
}

/// Selection-blind raw inventory.  It retains the unchanged support trace for
/// audit, exact P/S/R witnesses for every candidate, and no action or expected
/// continuation field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstructionCausalReturnRawQuery {
    frame: ConstructionCausalReturnFrame,
    observed_history_kappa: String,
    observed_history_address_kappas: Vec<String>,
    observed_routes: u8,
    support: AttentionSupportTrace,
    candidates: Vec<ConstructionCausalReturnRawCandidate>,
    populated_padding_aliases: usize,
    work: ConstructionCausalReturnWorkReport,
}

impl ConstructionCausalReturnRawQuery {
    pub fn frame(&self) -> &ConstructionCausalReturnFrame {
        &self.frame
    }

    pub const fn observed_routes(&self) -> u8 {
        self.observed_routes
    }

    pub fn observed_history_kappa(&self) -> &str {
        &self.observed_history_kappa
    }

    pub fn observed_history_address_kappas(&self) -> &[String] {
        &self.observed_history_address_kappas
    }

    pub fn support(&self) -> &AttentionSupportTrace {
        &self.support
    }

    pub fn candidates(&self) -> &[ConstructionCausalReturnRawCandidate] {
        &self.candidates
    }

    pub const fn populated_padding_aliases(&self) -> usize {
        self.populated_padding_aliases
    }

    pub const fn work(&self) -> ConstructionCausalReturnWorkReport {
        self.work
    }

    pub fn canonical_report(&self) -> ConstructionCausalReturnRawQueryReport {
        ConstructionCausalReturnRawQueryReport {
            schema: CONSTRUCTION_CAUSAL_RETURN_SCHEMA,
            domain: CONSTRUCTION_CAUSAL_RETURN_RAW_CENSUS_IDENTITY,
            frame_kappa: self.frame.frame_kappa.clone(),
            schema2_manifest_kappa: self.frame.schema2_manifest_kappa.clone(),
            policy_kappa: self.frame.policy_kappa.clone(),
            observed_history_kappa: self.observed_history_kappa.clone(),
            observed_history_address_kappas: self.observed_history_address_kappas.clone(),
            support_query_policy_identity: self.support.query_policy.identity().to_owned(),
            support_query_policy_kappa: self.support.query_policy_kappa.clone(),
            support_fallback_active: self.support.fallback_active,
            support_candidate_ceiling: self.support.candidate_ceiling,
            support_unique_candidates_before_ceiling: self.support.unique_candidates_before_ceiling,
            observed_routes: self.observed_routes,
            populated_padding_aliases: self.populated_padding_aliases,
            work: self.work,
            candidates: self
                .candidates
                .iter()
                .map(|candidate| ConstructionCausalReturnRawCandidateReport {
                    candidate_address_kappa: candidate.candidate_address_kappa.clone(),
                    source_counts: candidate.source_counts.into(),
                    representation: candidate.representation.clone(),
                })
                .collect(),
        }
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ConstructionCausalReturnError> {
        canonical_json(&self.canonical_report())
    }

    pub fn raw_census_kappa(&self) -> Result<String, ConstructionCausalReturnError> {
        canonical_kappa(&self.canonical_report())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstructionCausalReturnControlledRawCandidate {
    candidate: GeometricAddress,
    candidate_address_kappa: String,
    source_counts: AttentionSourceCounts,
    representation: ConstructionCausalReturnControlledRepresentation,
}

impl ConstructionCausalReturnControlledRawCandidate {
    pub fn candidate(&self) -> &GeometricAddress {
        &self.candidate
    }

    pub fn candidate_address_kappa(&self) -> &str {
        &self.candidate_address_kappa
    }

    pub const fn source_counts(&self) -> AttentionSourceCounts {
        self.source_counts
    }

    pub fn representation(&self) -> &ConstructionCausalReturnControlledRepresentation {
        &self.representation
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConstructionCausalReturnControlledRawCandidateReport {
    pub candidate_address_kappa: String,
    pub source_counts: ConstructionCausalReturnSourceCounts,
    pub representation: ConstructionCausalReturnControlledRepresentation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConstructionCausalReturnControlledRawQueryReport {
    pub schema: u32,
    pub domain: &'static str,
    pub frame_kappa: String,
    pub construction_partition_kappa: String,
    pub policy_kappa: String,
    pub control: ConstructionCausalReturnNegativeControl,
    pub population_role: ConstructionCausalReturnPopulationRole,
    pub control_input_kappa: String,
    pub observed_history_kappa: String,
    pub geometric_history_kappa: String,
    pub prime_placement_permutation_kappa: Option<String>,
    pub support_query_policy_identity: String,
    pub support_query_policy_kappa: String,
    pub support_fallback_active: bool,
    pub support_candidate_ceiling: usize,
    pub support_unique_candidates_before_ceiling: usize,
    pub support_unchanged_from_observed_history: bool,
    pub support_reused_from_frozen_raw: bool,
    pub support_admission_queries_performed: usize,
    pub observed_routes: u8,
    pub operative_relation_slots: usize,
    pub control_noop_slots: usize,
    pub padding_slots: usize,
    pub work: ConstructionCausalReturnWorkReport,
    pub candidates: Vec<ConstructionCausalReturnControlledRawCandidateReport>,
}

/// Control-side raw inventory. It exposes immutable exact witnesses and the
/// unchanged natural support. Candidate selection remains withheld until the
/// selection-blind Gate 0 succeeds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstructionCausalReturnControlledRawQuery {
    frame_kappa: String,
    construction_partition_kappa: String,
    policy_kappa: String,
    control: ConstructionCausalReturnNegativeControl,
    population_role: ConstructionCausalReturnPopulationRole,
    control_input_kappa: String,
    observed_history_kappa: String,
    geometric_history_kappa: String,
    prime_placement_permutation_kappa: Option<String>,
    support: AttentionSupportTrace,
    support_reused_from_frozen_raw: bool,
    observed_routes: u8,
    candidates: Vec<ConstructionCausalReturnControlledRawCandidate>,
    work: ConstructionCausalReturnWorkReport,
}

impl ConstructionCausalReturnControlledRawQuery {
    pub fn frame_kappa(&self) -> &str {
        &self.frame_kappa
    }

    pub const fn control(&self) -> ConstructionCausalReturnNegativeControl {
        self.control
    }

    pub const fn population_role(&self) -> ConstructionCausalReturnPopulationRole {
        self.population_role
    }

    pub fn control_input_kappa(&self) -> &str {
        &self.control_input_kappa
    }

    pub fn observed_history_kappa(&self) -> &str {
        &self.observed_history_kappa
    }

    pub fn geometric_history_kappa(&self) -> &str {
        &self.geometric_history_kappa
    }

    pub fn support(&self) -> &AttentionSupportTrace {
        &self.support
    }

    pub fn candidates(&self) -> &[ConstructionCausalReturnControlledRawCandidate] {
        &self.candidates
    }

    pub const fn work(&self) -> ConstructionCausalReturnWorkReport {
        self.work
    }

    pub fn canonical_report(&self) -> ConstructionCausalReturnControlledRawQueryReport {
        let (operative_relation_slots, control_noop_slots, padding_slots) = self
            .candidates
            .iter()
            .flat_map(|candidate| candidate.representation.slots.iter())
            .fold((0usize, 0usize, 0usize), |mut counts, slot| {
                match slot.kind {
                    ConstructionCausalReturnControlledSlotKind::Operative => counts.0 += 1,
                    ConstructionCausalReturnControlledSlotKind::ControlNoOp => counts.1 += 1,
                    ConstructionCausalReturnControlledSlotKind::Padding => counts.2 += 1,
                }
                counts
            });
        ConstructionCausalReturnControlledRawQueryReport {
            schema: CONSTRUCTION_CAUSAL_RETURN_SCHEMA,
            domain: CONSTRUCTION_CAUSAL_RETURN_RAW_CENSUS_IDENTITY,
            frame_kappa: self.frame_kappa.clone(),
            construction_partition_kappa: self.construction_partition_kappa.clone(),
            policy_kappa: self.policy_kappa.clone(),
            control: self.control,
            population_role: self.population_role,
            control_input_kappa: self.control_input_kappa.clone(),
            observed_history_kappa: self.observed_history_kappa.clone(),
            geometric_history_kappa: self.geometric_history_kappa.clone(),
            prime_placement_permutation_kappa: self.prime_placement_permutation_kappa.clone(),
            support_query_policy_identity: self.support.query_policy.identity().to_owned(),
            support_query_policy_kappa: self.support.query_policy_kappa.clone(),
            support_fallback_active: self.support.fallback_active,
            support_candidate_ceiling: self.support.candidate_ceiling,
            support_unique_candidates_before_ceiling: self.support.unique_candidates_before_ceiling,
            support_unchanged_from_observed_history: true,
            support_reused_from_frozen_raw: self.support_reused_from_frozen_raw,
            support_admission_queries_performed: usize::from(!self.support_reused_from_frozen_raw),
            observed_routes: self.observed_routes,
            operative_relation_slots,
            control_noop_slots,
            padding_slots,
            work: self.work,
            candidates: self
                .candidates
                .iter()
                .map(
                    |candidate| ConstructionCausalReturnControlledRawCandidateReport {
                        candidate_address_kappa: candidate.candidate_address_kappa.clone(),
                        source_counts: candidate.source_counts.into(),
                        representation: candidate.representation.clone(),
                    },
                )
                .collect(),
        }
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ConstructionCausalReturnError> {
        canonical_json(&self.canonical_report())
    }

    pub fn raw_census_kappa(&self) -> Result<String, ConstructionCausalReturnError> {
        canonical_kappa(&self.canonical_report())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConstructionCausalReturnAction {
    Select,
    Reject,
}

/// The only type that carries construction labels.  It is constructed from an
/// already-frozen raw candidate and is deliberately absent from `raw_query`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConstructionCausalReturnObservation {
    domain: &'static str,
    partition_kappa: String,
    transition_id: String,
    predecessor_history_kappa: String,
    observed_next_address_kappa: String,
    candidate_address_kappa: String,
    representation: ConstructionCausalReturnRepresentation,
    action: ConstructionCausalReturnAction,
}

impl ConstructionCausalReturnObservation {
    pub fn transition_id(&self) -> &str {
        &self.transition_id
    }

    pub fn candidate_address_kappa(&self) -> &str {
        &self.candidate_address_kappa
    }

    pub fn representation(&self) -> &ConstructionCausalReturnRepresentation {
        &self.representation
    }

    pub const fn action(&self) -> ConstructionCausalReturnAction {
        self.action
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConstructionCausalReturnControlledObservation {
    domain: &'static str,
    frame_kappa: String,
    partition_kappa: String,
    transition_id: String,
    predecessor_history_kappa: String,
    observed_next_address_kappa: String,
    control: ConstructionCausalReturnNegativeControl,
    control_input_kappa: String,
    candidate_address_kappa: String,
    representation: ConstructionCausalReturnControlledRepresentation,
    action: ConstructionCausalReturnAction,
}

impl ConstructionCausalReturnControlledObservation {
    pub fn frame_kappa(&self) -> &str {
        &self.frame_kappa
    }

    pub fn transition_id(&self) -> &str {
        &self.transition_id
    }

    pub const fn control(&self) -> ConstructionCausalReturnNegativeControl {
        self.control
    }

    pub fn control_input_kappa(&self) -> &str {
        &self.control_input_kappa
    }

    pub fn candidate_address_kappa(&self) -> &str {
        &self.candidate_address_kappa
    }

    pub fn representation(&self) -> &ConstructionCausalReturnControlledRepresentation {
        &self.representation
    }

    pub const fn action(&self) -> ConstructionCausalReturnAction {
        self.action
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstructionCausalReturnRepresentationLevel {
    RMin,
    RFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstructionCausalReturnLookupAbstention {
    MalformedClass,
    UnseenMinimumClass,
    UnseenRichClass,
    MultiplyMappedRichClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ConstructionCausalReturnLookup {
    Resolved {
        action: ConstructionCausalReturnAction,
        representation: ConstructionCausalReturnRepresentationLevel,
    },
    Abstain {
        reason: ConstructionCausalReturnLookupAbstention,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConstructionCausalReturnRichClassRecord {
    pub r_full: ConstructionCausalReturnFullWord,
    pub construction_rows: usize,
    pub select_rows: usize,
    pub reject_rows: usize,
    pub pure_action: Option<ConstructionCausalReturnAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConstructionCausalReturnMinimumClassRecord {
    pub r_min: ConstructionCausalReturnClassEvent,
    pub construction_rows: usize,
    pub select_rows: usize,
    pub reject_rows: usize,
    pub promoted_to_r_full: bool,
    pub direct_action: Option<ConstructionCausalReturnAction>,
    pub rich_classes: Vec<ConstructionCausalReturnRichClassRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConstructionCausalReturnArtifactReport {
    pub schema: u32,
    pub domain: &'static str,
    pub frame_kappa: String,
    pub policy_kappa: String,
    pub identity_coordinate: H4RootCoordinate,
    pub construction_rows: usize,
    pub minimum_classes: Vec<ConstructionCausalReturnMinimumClassRecord>,
    pub artifact_kappa: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ConstructionCausalReturnArtifactSeed<'a> {
    schema: u32,
    domain: &'static str,
    frame_kappa: &'a str,
    policy_kappa: &'a str,
    identity_coordinate: H4RootCoordinate,
    construction_rows: usize,
    minimum_classes: &'a [ConstructionCausalReturnMinimumClassRecord],
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MinimumClassResolution {
    Direct(ConstructionCausalReturnAction),
    Promoted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RichClassResolution {
    Pure(ConstructionCausalReturnAction),
    MultiplyMapped,
}

/// Construction-only compiled class map.  It does not own candidate admission
/// rows, payloads, validation labels, or a semantic tie-break.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstructionCausalReturnV1 {
    frame: ConstructionCausalReturnFrame,
    identity_state: OrderedH4FoldState,
    identity_coordinate: H4RootCoordinate,
    construction_rows: usize,
    minimum_classes: BTreeMap<ConstructionCausalReturnClassEvent, MinimumClassResolution>,
    rich_classes: BTreeMap<
        (
            ConstructionCausalReturnClassEvent,
            ConstructionCausalReturnFullWord,
        ),
        RichClassResolution,
    >,
    class_inventory: Vec<ConstructionCausalReturnMinimumClassRecord>,
    artifact_kappa: String,
}

impl ConstructionCausalReturnV1 {
    pub fn compile(
        frame: ConstructionCausalReturnFrame,
        table: &H4BinaryIcosahedralClosure,
        construction_rows: &[ConstructionCausalReturnObservation],
    ) -> Result<Self, ConstructionCausalReturnError> {
        frame.validate_labels()?;
        if construction_rows.len() != CONSTRUCTION_CAUSAL_RETURN_CONSTRUCTION_ROWS {
            return Err(ConstructionCausalReturnError::Invalid(
                format!(
                    "construction class compiler requires exactly {CONSTRUCTION_CAUSAL_RETURN_CONSTRUCTION_ROWS} authorized rows"
                ),
            ));
        }
        if frame.h4_root_table_kappa != table.h4_root_table_kappa
            || frame.multiplication_table_kappa != table.multiplication_table_kappa
            || validate_ordered_h4_table_exact(table).is_err()
            || frame.reproduce_frame_kappa().ok().as_deref() != Some(frame.frame_kappa.as_str())
        {
            return Err(ConstructionCausalReturnError::UnavailableFrameMismatch);
        }

        let identity_state = OrderedH4FoldState::identity(table)?;
        let identity_coordinate = identity_state.root_coordinate(table)?;
        let mut grouped = BTreeMap::<
            ConstructionCausalReturnClassEvent,
            BTreeMap<ConstructionCausalReturnFullWord, Vec<ConstructionCausalReturnAction>>,
        >::new();
        let mut rows_per_candidate = BTreeMap::<&str, (usize, usize)>::new();
        let mut rows_per_transition =
            BTreeMap::<&str, Vec<&ConstructionCausalReturnObservation>>::new();
        let mut distinct_rows = BTreeSet::<(&str, &str)>::new();
        let mut candidate_identities = BTreeSet::<&str>::new();

        for row in construction_rows {
            if row.domain != CONSTRUCTION_CAUSAL_RETURN_CONSTRUCTION_LABEL_JOIN_IDENTITY
                || row.partition_kappa != frame.construction_partition_kappa
                || row.representation.frame_kappa != frame.frame_kappa
                || row.transition_id.trim().is_empty()
                || validate_blake3_label(
                    &row.predecessor_history_kappa,
                    "construction predecessor history kappa",
                )
                .is_err()
                || validate_blake3_label(
                    &row.observed_next_address_kappa,
                    "construction observed-next address kappa",
                )
                .is_err()
                || validate_blake3_label(
                    &row.candidate_address_kappa,
                    "construction candidate address kappa",
                )
                .is_err()
                || !validate_representation(
                    &row.representation,
                    identity_state,
                    identity_coordinate,
                )
            {
                return Err(ConstructionCausalReturnError::UnavailableFrameMismatch);
            }
            if !distinct_rows.insert((
                row.transition_id.as_str(),
                row.candidate_address_kappa.as_str(),
            )) {
                return Err(ConstructionCausalReturnError::Invalid(
                    "duplicate construction transition/candidate row cannot satisfy the frozen cardinality"
                        .to_owned(),
                ));
            }
            rows_per_transition
                .entry(row.transition_id.as_str())
                .or_default()
                .push(row);
            candidate_identities.insert(row.candidate_address_kappa.as_str());
            let counts = rows_per_candidate
                .entry(row.candidate_address_kappa.as_str())
                .or_default();
            match row.action {
                ConstructionCausalReturnAction::Select => {
                    counts.0 = counts
                        .0
                        .checked_add(1)
                        .ok_or(ConstructionCausalReturnError::ArithmeticOverflow)?;
                }
                ConstructionCausalReturnAction::Reject => {
                    counts.1 = counts
                        .1
                        .checked_add(1)
                        .ok_or(ConstructionCausalReturnError::ArithmeticOverflow)?;
                }
            }
            grouped
                .entry(row.representation.r_min)
                .or_default()
                .entry(row.representation.r_full)
                .or_default()
                .push(row.action);
        }

        if rows_per_transition.len() != CONSTRUCTION_CAUSAL_RETURN_CONSTRUCTION_TRANSITIONS
            || candidate_identities.len() != CONSTRUCTION_CAUSAL_RETURN_CONSTRUCTION_CANDIDATES
            || rows_per_candidate.values().any(|(select, reject)| {
                *select != CONSTRUCTION_CAUSAL_RETURN_PROTOTYPES_PER_CANDIDATE
                    || *reject != CONSTRUCTION_CAUSAL_RETURN_PROTOTYPES_PER_CANDIDATE
            })
        {
            return Err(ConstructionCausalReturnError::Invalid(format!(
                "construction requires 12 distinct transitions, 6 candidates, and exactly {CONSTRUCTION_CAUSAL_RETURN_PROTOTYPES_PER_CANDIDATE} SELECT prototypes plus matched REJECT rows per candidate"
            )));
        }
        for rows in rows_per_transition.values() {
            if rows.len() != CONSTRUCTION_CAUSAL_RETURN_CANDIDATES_PER_DECISION
                || rows[0].predecessor_history_kappa != rows[1].predecessor_history_kappa
                || rows[0].observed_next_address_kappa != rows[1].observed_next_address_kappa
                || rows[0].candidate_address_kappa == rows[1].candidate_address_kappa
            {
                return Err(ConstructionCausalReturnError::Invalid(
                    "each construction transition must contain one exact two-candidate label join"
                        .to_owned(),
                ));
            }
            let select_rows = rows
                .iter()
                .filter(|row| row.action == ConstructionCausalReturnAction::Select)
                .collect::<Vec<_>>();
            let reject_rows = rows
                .iter()
                .filter(|row| row.action == ConstructionCausalReturnAction::Reject)
                .collect::<Vec<_>>();
            if select_rows.len() != 1
                || reject_rows.len() != 1
                || select_rows[0].candidate_address_kappa
                    != select_rows[0].observed_next_address_kappa
                || reject_rows[0].candidate_address_kappa
                    == reject_rows[0].observed_next_address_kappa
            {
                return Err(ConstructionCausalReturnError::Invalid(
                    "construction transition must bind observed=SELECT and other=REJECT".to_owned(),
                ));
            }
        }

        let mut minimum_classes = BTreeMap::new();
        let mut compiled_rich_classes = BTreeMap::new();
        let mut class_inventory = Vec::with_capacity(grouped.len());
        for (r_min, rich_groups) in grouped {
            let all_actions = rich_groups.values().flatten().copied().collect::<Vec<_>>();
            let (select_rows, reject_rows) = action_counts(&all_actions)?;
            let actions = all_actions.iter().copied().collect::<BTreeSet<_>>();
            let complete_rich_inventory = rich_groups
                .iter()
                .map(|(r_full, actions)| {
                    let (rich_select_rows, rich_reject_rows) = action_counts(actions)?;
                    let distinct = actions.iter().copied().collect::<BTreeSet<_>>();
                    let pure_action = (distinct.len() == 1)
                        .then(|| distinct.iter().next().copied())
                        .flatten();
                    Ok(ConstructionCausalReturnRichClassRecord {
                        r_full: *r_full,
                        construction_rows: actions.len(),
                        select_rows: rich_select_rows,
                        reject_rows: rich_reject_rows,
                        pure_action,
                    })
                })
                .collect::<Result<Vec<_>, ConstructionCausalReturnError>>()?;
            if actions.len() == 1 {
                let action = *actions.iter().next().ok_or_else(|| {
                    ConstructionCausalReturnError::Invalid(
                        "non-empty construction class lost its action".to_owned(),
                    )
                })?;
                minimum_classes.insert(r_min, MinimumClassResolution::Direct(action));
                class_inventory.push(ConstructionCausalReturnMinimumClassRecord {
                    r_min,
                    construction_rows: all_actions.len(),
                    select_rows,
                    reject_rows,
                    promoted_to_r_full: false,
                    direct_action: Some(action),
                    rich_classes: complete_rich_inventory,
                });
                continue;
            }

            for (r_full, actions) in rich_groups {
                let distinct = actions.iter().copied().collect::<BTreeSet<_>>();
                let pure_action = (distinct.len() == 1)
                    .then(|| distinct.iter().next().copied())
                    .flatten();
                let resolution = pure_action.map_or(
                    RichClassResolution::MultiplyMapped,
                    RichClassResolution::Pure,
                );
                compiled_rich_classes.insert((r_min, r_full), resolution);
            }
            minimum_classes.insert(r_min, MinimumClassResolution::Promoted);
            class_inventory.push(ConstructionCausalReturnMinimumClassRecord {
                r_min,
                construction_rows: all_actions.len(),
                select_rows,
                reject_rows,
                promoted_to_r_full: true,
                direct_action: None,
                rich_classes: complete_rich_inventory,
            });
        }

        let mut artifact = Self {
            frame,
            identity_state,
            identity_coordinate,
            construction_rows: construction_rows.len(),
            minimum_classes,
            rich_classes: compiled_rich_classes,
            class_inventory,
            artifact_kappa: String::new(),
        };
        artifact.artifact_kappa = artifact.reproduce_artifact_kappa()?;
        Ok(artifact)
    }

    pub fn frame(&self) -> &ConstructionCausalReturnFrame {
        &self.frame
    }

    pub fn artifact_kappa(&self) -> &str {
        &self.artifact_kappa
    }

    pub const fn construction_rows(&self) -> usize {
        self.construction_rows
    }

    pub fn class_inventory(&self) -> &[ConstructionCausalReturnMinimumClassRecord] {
        &self.class_inventory
    }

    pub fn canonical_report(&self) -> ConstructionCausalReturnArtifactReport {
        ConstructionCausalReturnArtifactReport {
            schema: CONSTRUCTION_CAUSAL_RETURN_SCHEMA,
            domain: CONSTRUCTION_CAUSAL_RETURN_CLASS_MAP_IDENTITY,
            frame_kappa: self.frame.frame_kappa.clone(),
            policy_kappa: self.frame.policy_kappa.clone(),
            identity_coordinate: self.identity_coordinate,
            construction_rows: self.construction_rows,
            minimum_classes: self.class_inventory.clone(),
            artifact_kappa: self.artifact_kappa.clone(),
        }
    }

    pub fn reproduce_artifact_kappa(&self) -> Result<String, ConstructionCausalReturnError> {
        canonical_kappa(&ConstructionCausalReturnArtifactSeed {
            schema: CONSTRUCTION_CAUSAL_RETURN_SCHEMA,
            domain: CONSTRUCTION_CAUSAL_RETURN_CLASS_MAP_IDENTITY,
            frame_kappa: &self.frame.frame_kappa,
            policy_kappa: &self.frame.policy_kappa,
            identity_coordinate: self.identity_coordinate,
            construction_rows: self.construction_rows,
            minimum_classes: &self.class_inventory,
        })
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ConstructionCausalReturnError> {
        self.frame.canonical_bytes()?;
        if self.reproduce_artifact_kappa()? != self.artifact_kappa {
            return Err(ConstructionCausalReturnError::UnavailableFrameMismatch);
        }
        canonical_json(&self.canonical_report())
    }

    pub fn raw_query(
        &self,
        attention: &GeometricAttentionArtifact,
        observed_history: &[GeometricAddress],
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<ConstructionCausalReturnRawQuery, ConstructionCausalReturnError> {
        self.frame.raw_query(attention, observed_history, table)
    }

    pub fn lookup_action(
        &self,
        representation: &ConstructionCausalReturnRepresentation,
    ) -> Result<ConstructionCausalReturnLookup, ConstructionCausalReturnError> {
        if representation.frame_kappa != self.frame.frame_kappa {
            return Err(ConstructionCausalReturnError::UnavailableFrameMismatch);
        }
        // Every candidate performs both fixed class-slot reads, including an
        // unseen or malformed class. Pure R_min resolution ignores the typed
        // rich-class probe; it does not omit it.
        let minimum_resolution = self.minimum_classes.get(&representation.r_min);
        let rich_resolution = self
            .rich_classes
            .get(&(representation.r_min, representation.r_full));
        if !validate_representation(
            representation,
            self.identity_state,
            self.identity_coordinate,
        ) {
            return Ok(ConstructionCausalReturnLookup::Abstain {
                reason: ConstructionCausalReturnLookupAbstention::MalformedClass,
            });
        }
        let Some(resolution) = minimum_resolution else {
            return Ok(ConstructionCausalReturnLookup::Abstain {
                reason: ConstructionCausalReturnLookupAbstention::UnseenMinimumClass,
            });
        };
        match resolution {
            MinimumClassResolution::Direct(action) => {
                Ok(ConstructionCausalReturnLookup::Resolved {
                    action: *action,
                    representation: ConstructionCausalReturnRepresentationLevel::RMin,
                })
            }
            MinimumClassResolution::Promoted => {
                let Some(rich) = rich_resolution else {
                    return Ok(ConstructionCausalReturnLookup::Abstain {
                        reason: ConstructionCausalReturnLookupAbstention::UnseenRichClass,
                    });
                };
                match rich {
                    RichClassResolution::Pure(action) => {
                        Ok(ConstructionCausalReturnLookup::Resolved {
                            action: *action,
                            representation: ConstructionCausalReturnRepresentationLevel::RFull,
                        })
                    }
                    RichClassResolution::MultiplyMapped => {
                        Ok(ConstructionCausalReturnLookup::Abstain {
                            reason:
                                ConstructionCausalReturnLookupAbstention::MultiplyMappedRichClass,
                        })
                    }
                }
            }
        }
    }

    /// Gate-0-only per-candidate lookup. It resolves no winner and performs no
    /// payload inversion; the fixed downstream work is declared explicitly.
    pub fn lookup_action_report(
        &self,
        candidate: &ConstructionCausalReturnRawCandidate,
    ) -> Result<ConstructionCausalReturnActionLookupReport, ConstructionCausalReturnError> {
        Ok(ConstructionCausalReturnActionLookupReport {
            candidate_address_kappa: candidate.candidate_address_kappa.clone(),
            class_slots_read: CONSTRUCTION_CAUSAL_RETURN_PROTOTYPES_PER_CANDIDATE,
            declared_payload_inversions: 1,
            performed_payload_inversions: 0,
            lookup: self.lookup_action(&candidate.representation)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConstructionCausalReturnActionLookupReport {
    pub candidate_address_kappa: String,
    pub class_slots_read: usize,
    pub declared_payload_inversions: usize,
    pub performed_payload_inversions: usize,
    pub lookup: ConstructionCausalReturnLookup,
}

fn encode_raw_query(
    frame: &ConstructionCausalReturnFrame,
    attention: &GeometricAttentionArtifact,
    observed_history: &[GeometricAddress],
    table: &H4BinaryIcosahedralClosure,
) -> Result<ConstructionCausalReturnRawQuery, ConstructionCausalReturnError> {
    frame.validate_query_binding(attention, table)?;
    if observed_history.is_empty()
        || observed_history.len() > CONSTRUCTION_CAUSAL_RETURN_PREFIX_SLOTS
    {
        return Err(ConstructionCausalReturnError::Invalid(format!(
            "causal-return raw query requires 1--{CONSTRUCTION_CAUSAL_RETURN_PREFIX_SLOTS} observed routes"
        )));
    }

    let path = attention.causal_path_state_from_history(observed_history, table)?;
    encode_raw_query_with_path_state(frame, attention, observed_history, &path, table)
}

fn encode_raw_query_from_path_state(
    frame: &ConstructionCausalReturnFrame,
    attention: &GeometricAttentionArtifact,
    observed_history: &[GeometricAddress],
    path_state: &CausalPathAttentionState,
    table: &H4BinaryIcosahedralClosure,
) -> Result<ConstructionCausalReturnRawQuery, ConstructionCausalReturnError> {
    frame.validate_query_binding(attention, table)?;
    if observed_history.is_empty()
        || observed_history.len() > CONSTRUCTION_CAUSAL_RETURN_PREFIX_SLOTS
    {
        return Err(ConstructionCausalReturnError::Invalid(format!(
            "incremental causal-return query requires 1--{CONSTRUCTION_CAUSAL_RETURN_PREFIX_SLOTS} observed routes"
        )));
    }
    let full_path = attention.causal_path_state_from_history(observed_history, table)?;
    if path_state.manifest_kappa() != full_path.manifest_kappa()
        || path_state.h4_root_table_kappa() != full_path.h4_root_table_kappa()
        || path_state.multiplication_table_kappa() != full_path.multiplication_table_kappa()
        || path_state.observed_routes() != full_path.observed_routes()
        || path_state.prefix_states() != full_path.prefix_states()
    {
        return Err(ConstructionCausalReturnError::UnavailableFrameMismatch);
    }
    encode_raw_query_with_path_state(frame, attention, observed_history, path_state, table)
}

fn encode_raw_query_from_path_state_and_frozen_raw(
    frame: &ConstructionCausalReturnFrame,
    raw: &ConstructionCausalReturnRawQuery,
    observed_history: &[GeometricAddress],
    path_state: &CausalPathAttentionState,
    table: &H4BinaryIcosahedralClosure,
) -> Result<ConstructionCausalReturnRawQuery, ConstructionCausalReturnError> {
    validate_frozen_raw_reuse_binding(frame, raw, observed_history, table)?;
    validate_path_state_reproduction(frame, observed_history, path_state, table)?;
    encode_raw_query_with_path_state_and_support(
        frame,
        observed_history,
        path_state,
        raw.support.clone(),
        table,
    )
}

fn encode_raw_query_with_path_state(
    frame: &ConstructionCausalReturnFrame,
    attention: &GeometricAttentionArtifact,
    observed_history: &[GeometricAddress],
    path: &CausalPathAttentionState,
    table: &H4BinaryIcosahedralClosure,
) -> Result<ConstructionCausalReturnRawQuery, ConstructionCausalReturnError> {
    frame.validate_query_binding(attention, table)?;
    let causal = attention.causal_state_from_history(observed_history)?;
    let support = attention.query_support_only(&causal)?;
    validate_natural_candidate_width(&support)?;
    encode_raw_query_with_path_state_and_support(frame, observed_history, path, support, table)
}

fn encode_raw_query_with_path_state_and_support(
    frame: &ConstructionCausalReturnFrame,
    observed_history: &[GeometricAddress],
    path: &CausalPathAttentionState,
    support: AttentionSupportTrace,
    table: &H4BinaryIcosahedralClosure,
) -> Result<ConstructionCausalReturnRawQuery, ConstructionCausalReturnError> {
    frame.validate_table_binding(table)?;
    validate_natural_candidate_width(&support)?;
    if path.manifest_kappa() != frame.schema2_manifest_kappa
        || path.h4_root_table_kappa() != frame.h4_root_table_kappa
        || path.multiplication_table_kappa() != frame.multiplication_table_kappa
    {
        return Err(ConstructionCausalReturnError::UnavailableFrameMismatch);
    }

    let observed_routes = u8::try_from(observed_history.len())
        .map_err(|_| ConstructionCausalReturnError::ArithmeticOverflow)?;
    let observed_history_address_kappas = address_kappas(observed_history)?;
    let observed_history_kappa =
        history_kappa_from_address_kappas(&observed_history_address_kappas)?;
    let identity = OrderedH4FoldState::identity(table)?;
    let identity_coordinate = identity.root_coordinate(table)?;
    let terminal = path.fold_state();
    let prefix_states = path.prefix_states();
    let mut candidates = Vec::with_capacity(support.candidates.len());
    let mut populated_padding_aliases = 0usize;

    for support_candidate in &support.candidates {
        let candidate_leaf = h4_leaf_state_for_address(&support_candidate.next, table)?;
        let candidate_leaf_coordinate = candidate_leaf.root_coordinate(table)?;
        let candidate_inverse = candidate_leaf.inverse(table)?;
        let padding_class = ConstructionCausalReturnClassEvent {
            relation_coordinate: identity_coordinate,
            angular_shell: H4S3AngularShell::Coincident,
            observed_lease_age: 0,
            multiplicity: 0,
            occupied: false,
        };
        let mut slots = std::array::from_fn(|slot_index| ConstructionCausalReturnWitnessSlot {
            class_event: padding_class,
            // The array constructor is statically bounded to eight slots.
            slot_index: slot_index as u8,
            prefix_state: identity,
            prefix_coordinate: identity_coordinate,
            suffix_state: identity,
            suffix_coordinate: identity_coordinate,
            relation_state: identity,
            relation_coordinate: identity_coordinate,
        });

        // All eight slots execute the same inverse/product shape.  Padding
        // substitutes typed identities before the operations, so it is an
        // exact no-op without becoming an occupied identity event.
        for (slot_index, slot) in slots.iter_mut().enumerate() {
            let occupied = slot_index < observed_history.len();
            let prefix = if occupied {
                *prefix_states.get(slot_index).ok_or_else(|| {
                    ConstructionCausalReturnError::Invalid(
                        "causal prefix state is absent from the bounded path".to_owned(),
                    )
                })?
            } else {
                identity
            };
            let endpoint = if occupied { terminal } else { identity };
            let prefix_inverse = prefix.inverse(table)?;
            let suffix = prefix_inverse.compose(endpoint, table)?;
            let suffix_inverse = suffix.inverse(table)?;
            let relation = suffix
                .compose(candidate_leaf, table)?
                .compose(suffix_inverse, table)?
                .compose(candidate_inverse, table)?;
            let prefix_coordinate = prefix.root_coordinate(table)?;
            let suffix_coordinate = suffix.root_coordinate(table)?;
            let relation_coordinate = relation.root_coordinate(table)?;
            let angular_shell = shell_for_coordinate(relation_coordinate)?;
            let observed_lease_age = if occupied {
                observed_routes
                    .checked_sub(
                        u8::try_from(slot_index)
                            .map_err(|_| ConstructionCausalReturnError::ArithmeticOverflow)?,
                    )
                    .ok_or(ConstructionCausalReturnError::ArithmeticOverflow)?
            } else {
                0
            };
            *slot = ConstructionCausalReturnWitnessSlot {
                class_event: ConstructionCausalReturnClassEvent {
                    relation_coordinate,
                    angular_shell,
                    observed_lease_age,
                    multiplicity: 0,
                    occupied,
                },
                slot_index: u8::try_from(slot_index)
                    .map_err(|_| ConstructionCausalReturnError::ArithmeticOverflow)?,
                prefix_state: prefix,
                prefix_coordinate,
                suffix_state: suffix,
                suffix_coordinate,
                relation_state: relation,
                relation_coordinate,
            };
        }

        let mut multiplicities = BTreeMap::<H4RootCoordinate, u8>::new();
        for slot in slots.iter().filter(|slot| slot.class_event.occupied) {
            let count = multiplicities
                .entry(slot.class_event.relation_coordinate)
                .or_default();
            *count = count
                .checked_add(1)
                .ok_or(ConstructionCausalReturnError::ArithmeticOverflow)?;
        }
        for slot in slots.iter_mut().filter(|slot| slot.class_event.occupied) {
            slot.class_event.multiplicity = *multiplicities
                .get(&slot.class_event.relation_coordinate)
                .ok_or_else(|| {
                    ConstructionCausalReturnError::Invalid(
                        "occupied relation lost its exact multiplicity".to_owned(),
                    )
                })?;
        }

        let (r_min_slot_index, r_min) = slots
            .iter()
            .filter(|slot| slot.class_event.occupied)
            .min_by(|left, right| r_min_cmp(left.class_event, right.class_event))
            .map(|slot| (slot.slot_index, slot.class_event))
            .ok_or_else(|| {
                ConstructionCausalReturnError::Invalid(
                    "causal-return representation has no occupied class".to_owned(),
                )
            })?;
        let r_full = ConstructionCausalReturnFullWord {
            slots: std::array::from_fn(|index| slots[index].class_event),
        };
        for occupied in r_full.slots.iter().filter(|event| event.occupied) {
            for padding in r_full.slots.iter().filter(|event| !event.occupied) {
                if occupied == padding {
                    populated_padding_aliases = populated_padding_aliases
                        .checked_add(1)
                        .ok_or(ConstructionCausalReturnError::ArithmeticOverflow)?;
                }
            }
        }

        candidates.push(ConstructionCausalReturnRawCandidate {
            candidate_address_kappa: support_candidate.next.canonical_kappa()?,
            candidate: support_candidate.next.clone(),
            source_counts: support_candidate.source_counts,
            representation: ConstructionCausalReturnRepresentation {
                frame_kappa: frame.frame_kappa.clone(),
                observed_routes,
                candidate_leaf_state: candidate_leaf,
                candidate_leaf_coordinate,
                r_min,
                r_min_slot_index,
                r_full,
                slots,
            },
        });
    }

    let candidate_count = candidates.len();
    let relation_slots = checked_mul(candidate_count, CONSTRUCTION_CAUSAL_RETURN_PREFIX_SLOTS)?;
    let populated_relation_slots = checked_mul(candidate_count, observed_history.len())?;
    let padded_relation_slots = relation_slots
        .checked_sub(populated_relation_slots)
        .ok_or(ConstructionCausalReturnError::ArithmeticOverflow)?;
    let h4_leaf_mappings = observed_history
        .len()
        .checked_add(candidate_count)
        .ok_or(ConstructionCausalReturnError::ArithmeticOverflow)?;
    // Prefix folding performs t products once.  Per candidate: every fixed
    // slot performs four products and two inverses, and L(c)^-1 is shared by
    // its eight slots.
    let h4_product_table_reads = observed_history
        .len()
        .checked_add(checked_mul(relation_slots, 4)?)
        .ok_or(ConstructionCausalReturnError::ArithmeticOverflow)?;
    let h4_inverse_table_reads = checked_mul(relation_slots, 2)?
        .checked_add(candidate_count)
        .ok_or(ConstructionCausalReturnError::ArithmeticOverflow)?;
    let declared_prototype_class_slots = checked_mul(
        candidate_count,
        CONSTRUCTION_CAUSAL_RETURN_PROTOTYPES_PER_CANDIDATE,
    )?;
    let work = ConstructionCausalReturnWorkReport {
        support_rows_read: support.rows_read.len(),
        candidate_entries_available: support.candidate_entries_available,
        candidate_entries_examined: support.candidate_entries_examined,
        candidate_entries_admitted: support.candidate_entries_admitted,
        natural_candidates: candidate_count,
        observed_prefix_products: observed_history.len(),
        fixed_relation_slots_per_candidate: CONSTRUCTION_CAUSAL_RETURN_PREFIX_SLOTS,
        relation_slots,
        populated_relation_slots,
        padded_relation_slots,
        h4_leaf_mappings,
        h4_product_table_reads,
        h4_inverse_table_reads,
        declared_prototype_class_slots_per_candidate:
            CONSTRUCTION_CAUSAL_RETURN_PROTOTYPES_PER_CANDIDATE,
        declared_prototype_class_slots,
        performed_prototype_class_slot_reads: 0,
        declared_payload_inversions_per_candidate: 1,
        declared_payload_inversions: candidate_count,
        performed_payload_inversions: 0,
        source_inputs: 0,
        provider_inputs: 0,
        teacher_inputs: 0,
        future_route_inputs: 0,
        validation_label_inputs: 0,
    };

    Ok(ConstructionCausalReturnRawQuery {
        frame: frame.clone(),
        observed_history_kappa,
        observed_history_address_kappas,
        observed_routes,
        support,
        candidates,
        populated_padding_aliases,
        work,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ControlledGeometryMode {
    Standard,
    StateDisabled,
    LastOnly,
    LeaseDisabled,
    ExactRecall,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ConstructionCausalReturnControlInputSeed<'a> {
    schema: u32,
    domain: &'static str,
    frame_kappa: &'a str,
    control: ConstructionCausalReturnNegativeControl,
    population_role: ConstructionCausalReturnPopulationRole,
    observed_history_kappa: &'a str,
    geometric_history_kappa: &'a str,
    prime_placement_permutation_kappa: Option<&'a str>,
}

fn encode_controlled_raw_query(
    frame: &ConstructionCausalReturnFrame,
    attention: &GeometricAttentionArtifact,
    observed_history: &[GeometricAddress],
    population_role: ConstructionCausalReturnPopulationRole,
    control: &ConstructionCausalReturnControlledEncoder,
    table: &H4BinaryIcosahedralClosure,
) -> Result<ConstructionCausalReturnControlledRawQuery, ConstructionCausalReturnError> {
    frame.validate_query_binding(attention, table)?;
    validate_observed_history_width(observed_history, "controlled causal-return query")?;
    let causal = attention.causal_state_from_history(observed_history)?;
    let support = attention.query_support_only(&causal)?;
    validate_natural_candidate_width(&support)?;
    encode_controlled_raw_query_with_support(
        frame,
        observed_history,
        population_role,
        control,
        support,
        false,
        table,
    )
}

fn encode_controlled_raw_query_from_frozen_raw(
    frame: &ConstructionCausalReturnFrame,
    raw: &ConstructionCausalReturnRawQuery,
    observed_history: &[GeometricAddress],
    population_role: ConstructionCausalReturnPopulationRole,
    control: &ConstructionCausalReturnControlledEncoder,
    table: &H4BinaryIcosahedralClosure,
) -> Result<ConstructionCausalReturnControlledRawQuery, ConstructionCausalReturnError> {
    validate_frozen_raw_reuse_binding(frame, raw, observed_history, table)?;
    encode_controlled_raw_query_with_support(
        frame,
        observed_history,
        population_role,
        control,
        raw.support.clone(),
        true,
        table,
    )
}

fn encode_controlled_raw_query_with_support(
    frame: &ConstructionCausalReturnFrame,
    observed_history: &[GeometricAddress],
    population_role: ConstructionCausalReturnPopulationRole,
    control: &ConstructionCausalReturnControlledEncoder,
    support: AttentionSupportTrace,
    support_reused_from_frozen_raw: bool,
    table: &H4BinaryIcosahedralClosure,
) -> Result<ConstructionCausalReturnControlledRawQuery, ConstructionCausalReturnError> {
    frame.validate_table_binding(table)?;
    validate_observed_history_width(observed_history, "controlled causal-return query")?;
    validate_natural_candidate_width(&support)?;
    match (control, population_role) {
        (
            ConstructionCausalReturnControlledEncoder::CandidatePrototypePlacementPermutation,
            ConstructionCausalReturnPopulationRole::Validation,
        ) => {
            return Err(ConstructionCausalReturnError::Invalid(
                "candidate-prototype placement permutation is a construction-role control"
                    .to_owned(),
            ));
        }
        (
            ConstructionCausalReturnControlledEncoder::ContentSwap { .. },
            ConstructionCausalReturnPopulationRole::Construction,
        ) => {
            return Err(ConstructionCausalReturnError::Invalid(
                "content swap is a validation-role control".to_owned(),
            ));
        }
        _ => {}
    }

    // Support is the immutable natural row union supplied by either the one
    // real query or a validated clone of that query. Geometry controls cannot
    // change, widen, or inject candidates.
    let observed_history_address_kappas = address_kappas(observed_history)?;
    let observed_history_kappa =
        history_kappa_from_address_kappas(&observed_history_address_kappas)?;
    let identity = OrderedH4FoldState::identity(table)?;

    let mut geometric_history = observed_history.to_vec();
    let (mode, prime_placement) = match control {
        ConstructionCausalReturnControlledEncoder::StateDisabled => {
            (ControlledGeometryMode::StateDisabled, None)
        }
        ConstructionCausalReturnControlledEncoder::LastOnly => {
            (ControlledGeometryMode::LastOnly, None)
        }
        ConstructionCausalReturnControlledEncoder::OrderShuffledHistory => {
            geometric_history.reverse();
            (ControlledGeometryMode::Standard, None)
        }
        ConstructionCausalReturnControlledEncoder::CausalReturnLeaseDisabled => {
            (ControlledGeometryMode::LeaseDisabled, None)
        }
        ConstructionCausalReturnControlledEncoder::CandidatePrototypePlacementPermutation => {
            (ControlledGeometryMode::Standard, None)
        }
        ConstructionCausalReturnControlledEncoder::PrimePlacementPermutation(permutation) => {
            (ControlledGeometryMode::Standard, Some(permutation))
        }
        ConstructionCausalReturnControlledEncoder::ExactRecallOnly => {
            (ControlledGeometryMode::ExactRecall, None)
        }
        ConstructionCausalReturnControlledEncoder::ContentSwap {
            swapped_observed_history,
        } => {
            validate_content_swap(observed_history, swapped_observed_history)?;
            geometric_history = swapped_observed_history.clone();
            (ControlledGeometryMode::Standard, None)
        }
    };

    let (prefix_states, terminal) = if mode == ControlledGeometryMode::StateDisabled {
        // Preserve t leaf mappings and t table products while substituting
        // typed inactive identities for every history operand.
        let mut prefixes = Vec::with_capacity(observed_history.len() + 1);
        let mut fold = identity;
        prefixes.push(fold);
        for observed in observed_history {
            let _discarded_leaf = h4_leaf_state_for_address(observed, table)?;
            fold = fold.compose(identity, table)?;
            prefixes.push(fold);
        }
        (prefixes, fold)
    } else if let Some(permutation) = prime_placement {
        let mut prefixes = Vec::with_capacity(geometric_history.len() + 1);
        let mut fold = identity;
        prefixes.push(fold);
        for observed in &geometric_history {
            let leaf = permutation.leaf_for_address(observed, table)?;
            fold = fold.compose(leaf, table)?;
            prefixes.push(fold);
        }
        (prefixes, fold)
    } else {
        exact_prefix_states(&geometric_history, table)?
    };

    let geometric_history_address_kappas = address_kappas(&geometric_history)?;
    let geometric_history_kappa =
        history_kappa_from_address_kappas(&geometric_history_address_kappas)?;
    let observed_routes = u8::try_from(observed_history.len())
        .map_err(|_| ConstructionCausalReturnError::ArithmeticOverflow)?;
    let mut candidates = build_controlled_candidates(
        &support,
        observed_routes,
        &prefix_states,
        terminal,
        mode,
        control.control(),
        &observed_history_kappa,
        prime_placement,
        table,
    )?;
    if matches!(
        control,
        ConstructionCausalReturnControlledEncoder::CandidatePrototypePlacementPermutation
    ) {
        if candidates.len() != CONSTRUCTION_CAUSAL_RETURN_CANDIDATES_PER_DECISION {
            return Err(ConstructionCausalReturnError::Invalid(
                "candidate-prototype placement permutation requires exactly two natural candidates"
                    .to_owned(),
            ));
        }
        let (first, rest) = candidates.split_at_mut(1);
        std::mem::swap(&mut first[0].representation, &mut rest[0].representation);
    }

    let prime_placement_permutation_kappa =
        prime_placement.map(|permutation| permutation.permutation_kappa.clone());
    let control_input_kappa = canonical_kappa(&ConstructionCausalReturnControlInputSeed {
        schema: CONSTRUCTION_CAUSAL_RETURN_SCHEMA,
        domain: CONSTRUCTION_CAUSAL_RETURN_RAW_CENSUS_IDENTITY,
        frame_kappa: &frame.frame_kappa,
        control: control.control(),
        population_role,
        observed_history_kappa: &observed_history_kappa,
        geometric_history_kappa: &geometric_history_kappa,
        prime_placement_permutation_kappa: prime_placement_permutation_kappa.as_deref(),
    })?;
    let work = work_report(&support, observed_history.len(), candidates.len())?;
    Ok(ConstructionCausalReturnControlledRawQuery {
        frame_kappa: frame.frame_kappa.clone(),
        construction_partition_kappa: frame.construction_partition_kappa.clone(),
        policy_kappa: frame.policy_kappa.clone(),
        control: control.control(),
        population_role,
        control_input_kappa,
        observed_history_kappa,
        geometric_history_kappa,
        prime_placement_permutation_kappa,
        support,
        support_reused_from_frozen_raw,
        observed_routes,
        candidates,
        work,
    })
}

#[allow(clippy::too_many_arguments)]
fn build_controlled_candidates(
    support: &AttentionSupportTrace,
    observed_routes: u8,
    prefix_states: &[OrderedH4FoldState],
    terminal: OrderedH4FoldState,
    mode: ControlledGeometryMode,
    control: ConstructionCausalReturnNegativeControl,
    observed_history_kappa: &str,
    prime_placement: Option<&ConstructionCausalReturnPrimePlacementPermutation>,
    table: &H4BinaryIcosahedralClosure,
) -> Result<Vec<ConstructionCausalReturnControlledRawCandidate>, ConstructionCausalReturnError> {
    let identity = OrderedH4FoldState::identity(table)?;
    let identity_coordinate = identity.root_coordinate(table)?;
    let observed = usize::from(observed_routes);
    let mut candidates = Vec::with_capacity(support.candidates.len());
    for support_candidate in &support.candidates {
        let candidate_address_kappa = support_candidate.next.canonical_kappa()?;
        let candidate_leaf = match prime_placement {
            Some(permutation) => permutation.leaf_for_address(&support_candidate.next, table)?,
            None => h4_leaf_state_for_address(&support_candidate.next, table)?,
        };
        let candidate_leaf_coordinate = candidate_leaf.root_coordinate(table)?;
        let candidate_inverse = candidate_leaf.inverse(table)?;
        let padding_class = ConstructionCausalReturnClassEvent {
            relation_coordinate: identity_coordinate,
            angular_shell: H4S3AngularShell::Coincident,
            observed_lease_age: 0,
            multiplicity: 0,
            occupied: false,
        };
        let mut slots =
            std::array::from_fn(|slot_index| ConstructionCausalReturnControlledWitnessSlot {
                kind: ConstructionCausalReturnControlledSlotKind::Padding,
                witness: ConstructionCausalReturnWitnessSlot {
                    class_event: padding_class,
                    slot_index: slot_index as u8,
                    prefix_state: identity,
                    prefix_coordinate: identity_coordinate,
                    suffix_state: identity,
                    suffix_coordinate: identity_coordinate,
                    relation_state: identity,
                    relation_coordinate: identity_coordinate,
                },
            });

        for (slot_index, slot) in slots.iter_mut().enumerate() {
            let within_history = slot_index < observed;
            let operative =
                within_history
                    && match mode {
                        ControlledGeometryMode::Standard
                        | ControlledGeometryMode::LeaseDisabled => true,
                        ControlledGeometryMode::LastOnly => slot_index + 1 == observed,
                        ControlledGeometryMode::StateDisabled
                        | ControlledGeometryMode::ExactRecall => false,
                    };
            let kind = if !within_history {
                ConstructionCausalReturnControlledSlotKind::Padding
            } else if operative {
                ConstructionCausalReturnControlledSlotKind::Operative
            } else {
                ConstructionCausalReturnControlledSlotKind::ControlNoOp
            };
            let use_real_prefix = operative || mode == ControlledGeometryMode::LeaseDisabled;
            let prefix = if use_real_prefix {
                *prefix_states.get(slot_index).ok_or_else(|| {
                    ConstructionCausalReturnError::Invalid(
                        "controlled prefix state is absent from the bounded path".to_owned(),
                    )
                })?
            } else {
                identity
            };
            let endpoint = if use_real_prefix { terminal } else { identity };
            let prefix_inverse = prefix.inverse(table)?;
            let computed_suffix = prefix_inverse.compose(endpoint, table)?;
            let computed_suffix_inverse = computed_suffix.inverse(table)?;
            let (suffix, relation) = if mode == ControlledGeometryMode::LeaseDisabled && operative {
                let relation = identity
                    .compose(candidate_leaf, table)?
                    .compose(identity, table)?
                    .compose(candidate_inverse, table)?;
                let _discarded_suffix_inverse = computed_suffix_inverse;
                (identity, relation)
            } else {
                let relation = computed_suffix
                    .compose(candidate_leaf, table)?
                    .compose(computed_suffix_inverse, table)?
                    .compose(candidate_inverse, table)?;
                (computed_suffix, relation)
            };
            let prefix_coordinate = prefix.root_coordinate(table)?;
            let suffix_coordinate = suffix.root_coordinate(table)?;
            let relation_coordinate = relation.root_coordinate(table)?;
            let observed_lease_age = if operative && mode != ControlledGeometryMode::LeaseDisabled {
                observed_routes
                    .checked_sub(
                        u8::try_from(slot_index)
                            .map_err(|_| ConstructionCausalReturnError::ArithmeticOverflow)?,
                    )
                    .ok_or(ConstructionCausalReturnError::ArithmeticOverflow)?
            } else {
                0
            };
            *slot = ConstructionCausalReturnControlledWitnessSlot {
                kind,
                witness: ConstructionCausalReturnWitnessSlot {
                    class_event: ConstructionCausalReturnClassEvent {
                        relation_coordinate,
                        angular_shell: shell_for_coordinate(relation_coordinate)?,
                        observed_lease_age,
                        multiplicity: 0,
                        occupied: operative,
                    },
                    slot_index: u8::try_from(slot_index)
                        .map_err(|_| ConstructionCausalReturnError::ArithmeticOverflow)?,
                    prefix_state: prefix,
                    prefix_coordinate,
                    suffix_state: suffix,
                    suffix_coordinate,
                    relation_state: relation,
                    relation_coordinate,
                },
            };
        }

        let mut multiplicities = BTreeMap::<H4RootCoordinate, u8>::new();
        for slot in slots
            .iter()
            .filter(|slot| slot.kind == ConstructionCausalReturnControlledSlotKind::Operative)
        {
            let count = multiplicities
                .entry(slot.witness.class_event.relation_coordinate)
                .or_default();
            *count = count
                .checked_add(1)
                .ok_or(ConstructionCausalReturnError::ArithmeticOverflow)?;
        }
        for slot in slots
            .iter_mut()
            .filter(|slot| slot.kind == ConstructionCausalReturnControlledSlotKind::Operative)
        {
            slot.witness.class_event.multiplicity = *multiplicities
                .get(&slot.witness.class_event.relation_coordinate)
                .ok_or_else(|| {
                    ConstructionCausalReturnError::Invalid(
                        "controlled relation lost its exact multiplicity".to_owned(),
                    )
                })?;
        }
        let minimum = slots
            .iter()
            .filter(|slot| slot.kind == ConstructionCausalReturnControlledSlotKind::Operative)
            .min_by(|left, right| {
                r_min_cmp(left.witness.class_event, right.witness.class_event).then_with(|| {
                    Reverse(left.witness.slot_index).cmp(&Reverse(right.witness.slot_index))
                })
            });
        let (r_min_slot_index, r_min) = minimum
            .map(|slot| {
                (
                    Some(slot.witness.slot_index),
                    Some(slot.witness.class_event),
                )
            })
            .unwrap_or((None, None));
        let r_full = ConstructionCausalReturnFullWord {
            slots: std::array::from_fn(|index| slots[index].witness.class_event),
        };
        let exact_recall_key = (mode == ControlledGeometryMode::ExactRecall).then(|| {
            ConstructionCausalReturnExactRecallKey {
                predecessor_history_kappa: observed_history_kappa.to_owned(),
                candidate_address_kappa: candidate_address_kappa.clone(),
            }
        });
        candidates.push(ConstructionCausalReturnControlledRawCandidate {
            candidate: support_candidate.next.clone(),
            candidate_address_kappa,
            source_counts: support_candidate.source_counts,
            representation: ConstructionCausalReturnControlledRepresentation {
                control,
                observed_routes,
                candidate_leaf_state: candidate_leaf,
                candidate_leaf_coordinate,
                r_min,
                r_min_slot_index,
                r_full,
                exact_recall_key,
                slots,
            },
        });
    }
    Ok(candidates)
}

fn validate_content_swap(
    observed_history: &[GeometricAddress],
    swapped_history: &[GeometricAddress],
) -> Result<(), ConstructionCausalReturnError> {
    if observed_history.len() != swapped_history.len()
        || observed_history.is_empty()
        || observed_history.len() > CONSTRUCTION_CAUSAL_RETURN_PREFIX_SLOTS
    {
        return Err(ConstructionCausalReturnError::Invalid(
            "content-swap control requires a same-width observed history".to_owned(),
        ));
    }
    let mut observed_multiset = observed_history.to_vec();
    let mut swapped_multiset = swapped_history.to_vec();
    observed_multiset.sort();
    swapped_multiset.sort();
    let changed_positions = observed_history
        .iter()
        .zip(swapped_history)
        .filter(|(left, right)| left != right)
        .count();
    if observed_multiset != swapped_multiset || changed_positions != 2 {
        return Err(ConstructionCausalReturnError::Invalid(
            "content-swap control requires one exact two-position transposition of the observed multiset"
                .to_owned(),
        ));
    }
    Ok(())
}

fn work_report(
    support: &AttentionSupportTrace,
    observed_routes: usize,
    candidate_count: usize,
) -> Result<ConstructionCausalReturnWorkReport, ConstructionCausalReturnError> {
    let relation_slots = checked_mul(candidate_count, CONSTRUCTION_CAUSAL_RETURN_PREFIX_SLOTS)?;
    let populated_relation_slots = checked_mul(candidate_count, observed_routes)?;
    let padded_relation_slots = relation_slots
        .checked_sub(populated_relation_slots)
        .ok_or(ConstructionCausalReturnError::ArithmeticOverflow)?;
    let h4_leaf_mappings = observed_routes
        .checked_add(candidate_count)
        .ok_or(ConstructionCausalReturnError::ArithmeticOverflow)?;
    let h4_product_table_reads = observed_routes
        .checked_add(checked_mul(relation_slots, 4)?)
        .ok_or(ConstructionCausalReturnError::ArithmeticOverflow)?;
    let h4_inverse_table_reads = checked_mul(relation_slots, 2)?
        .checked_add(candidate_count)
        .ok_or(ConstructionCausalReturnError::ArithmeticOverflow)?;
    let declared_prototype_class_slots = checked_mul(
        candidate_count,
        CONSTRUCTION_CAUSAL_RETURN_PROTOTYPES_PER_CANDIDATE,
    )?;
    Ok(ConstructionCausalReturnWorkReport {
        support_rows_read: support.rows_read.len(),
        candidate_entries_available: support.candidate_entries_available,
        candidate_entries_examined: support.candidate_entries_examined,
        candidate_entries_admitted: support.candidate_entries_admitted,
        natural_candidates: candidate_count,
        observed_prefix_products: observed_routes,
        fixed_relation_slots_per_candidate: CONSTRUCTION_CAUSAL_RETURN_PREFIX_SLOTS,
        relation_slots,
        populated_relation_slots,
        padded_relation_slots,
        h4_leaf_mappings,
        h4_product_table_reads,
        h4_inverse_table_reads,
        declared_prototype_class_slots_per_candidate:
            CONSTRUCTION_CAUSAL_RETURN_PROTOTYPES_PER_CANDIDATE,
        declared_prototype_class_slots,
        performed_prototype_class_slot_reads: 0,
        declared_payload_inversions_per_candidate: 1,
        declared_payload_inversions: candidate_count,
        performed_payload_inversions: 0,
        source_inputs: 0,
        provider_inputs: 0,
        teacher_inputs: 0,
        future_route_inputs: 0,
        validation_label_inputs: 0,
    })
}

fn validate_natural_candidate_width(
    support: &AttentionSupportTrace,
) -> Result<(), ConstructionCausalReturnError> {
    if support.candidates.len() != CONSTRUCTION_CAUSAL_RETURN_CANDIDATES_PER_DECISION {
        return Err(ConstructionCausalReturnError::Invalid(format!(
            "causal-return mechanism requires exactly {CONSTRUCTION_CAUSAL_RETURN_CANDIDATES_PER_DECISION} naturally admitted candidates"
        )));
    }
    Ok(())
}

fn validate_observed_history_width(
    observed_history: &[GeometricAddress],
    operation: &str,
) -> Result<(), ConstructionCausalReturnError> {
    if observed_history.is_empty()
        || observed_history.len() > CONSTRUCTION_CAUSAL_RETURN_PREFIX_SLOTS
    {
        return Err(ConstructionCausalReturnError::Invalid(format!(
            "{operation} requires 1--{CONSTRUCTION_CAUSAL_RETURN_PREFIX_SLOTS} observed routes"
        )));
    }
    Ok(())
}

fn exact_prefix_states(
    observed_history: &[GeometricAddress],
    table: &H4BinaryIcosahedralClosure,
) -> Result<(Vec<OrderedH4FoldState>, OrderedH4FoldState), ConstructionCausalReturnError> {
    validate_observed_history_width(observed_history, "causal-return prefix fold")?;
    let mut prefixes = Vec::with_capacity(observed_history.len() + 1);
    let mut fold = OrderedH4FoldState::identity(table)?;
    prefixes.push(fold);
    for observed in observed_history {
        let leaf = h4_leaf_state_for_address(observed, table)?;
        fold = fold.compose(leaf, table)?;
        prefixes.push(fold);
    }
    Ok((prefixes, fold))
}

fn validate_path_state_reproduction(
    frame: &ConstructionCausalReturnFrame,
    observed_history: &[GeometricAddress],
    path_state: &CausalPathAttentionState,
    table: &H4BinaryIcosahedralClosure,
) -> Result<(), ConstructionCausalReturnError> {
    frame.validate_table_binding(table)?;
    let (expected_prefixes, _) = exact_prefix_states(observed_history, table)?;
    if path_state.manifest_kappa() != frame.schema2_manifest_kappa
        || path_state.h4_root_table_kappa() != frame.h4_root_table_kappa
        || path_state.multiplication_table_kappa() != frame.multiplication_table_kappa
        || path_state.observed_routes() != observed_history.len()
        || path_state.prefix_states() != expected_prefixes
    {
        return Err(ConstructionCausalReturnError::UnavailableFrameMismatch);
    }
    Ok(())
}

fn validate_frozen_raw_reuse_binding(
    frame: &ConstructionCausalReturnFrame,
    raw: &ConstructionCausalReturnRawQuery,
    observed_history: &[GeometricAddress],
    table: &H4BinaryIcosahedralClosure,
) -> Result<(), ConstructionCausalReturnError> {
    frame.validate_table_binding(table)?;
    validate_observed_history_width(observed_history, "frozen-support reuse")?;
    let observed_history_address_kappas = address_kappas(observed_history)?;
    let observed_history_kappa =
        history_kappa_from_address_kappas(&observed_history_address_kappas)?;
    let observed_routes = u8::try_from(observed_history.len())
        .map_err(|_| ConstructionCausalReturnError::ArithmeticOverflow)?;
    validate_natural_candidate_width(&raw.support)?;
    if raw.frame != *frame
        || raw.observed_history_kappa != observed_history_kappa
        || raw.observed_history_address_kappas != observed_history_address_kappas
        || raw.observed_routes != observed_routes
        || raw.candidates.len() != raw.support.candidates.len()
        || raw.work.support_rows_read != raw.support.rows_read.len()
        || raw.work.candidate_entries_available != raw.support.candidate_entries_available
        || raw.work.candidate_entries_examined != raw.support.candidate_entries_examined
        || raw.work.candidate_entries_admitted != raw.support.candidate_entries_admitted
        || raw.work.natural_candidates != raw.support.candidates.len()
    {
        return Err(ConstructionCausalReturnError::UnavailableFrameMismatch);
    }

    let identity_state = OrderedH4FoldState::identity(table)?;
    let identity_coordinate = identity_state.root_coordinate(table)?;
    for (support_candidate, raw_candidate) in
        raw.support.candidates.iter().zip(raw.candidates.iter())
    {
        let candidate_address_kappa = support_candidate.next.canonical_kappa()?;
        let candidate_leaf_state = h4_leaf_state_for_address(&support_candidate.next, table)?;
        if support_candidate.next != raw_candidate.candidate
            || support_candidate.source_counts != raw_candidate.source_counts
            || raw_candidate.candidate_address_kappa != candidate_address_kappa
            || raw_candidate.representation.frame_kappa != frame.frame_kappa
            || raw_candidate.representation.observed_routes != observed_routes
            || raw_candidate.representation.candidate_leaf_state != candidate_leaf_state
            || !validate_representation(
                &raw_candidate.representation,
                identity_state,
                identity_coordinate,
            )
        {
            return Err(ConstructionCausalReturnError::UnavailableFrameMismatch);
        }
    }
    Ok(())
}

fn validate_representation(
    representation: &ConstructionCausalReturnRepresentation,
    identity_state: OrderedH4FoldState,
    identity_coordinate: H4RootCoordinate,
) -> bool {
    let observed = usize::from(representation.observed_routes);
    if observed == 0 || observed > CONSTRUCTION_CAUSAL_RETURN_PREFIX_SLOTS {
        return false;
    }
    if representation.slots[0].prefix_state != identity_state
        || representation.slots[0].prefix_coordinate != identity_coordinate
    {
        return false;
    }
    for (index, slot) in representation.slots.iter().enumerate() {
        let should_be_occupied = index < observed;
        if usize::from(slot.slot_index) != index
            || slot.class_event.occupied != should_be_occupied
            || slot.class_event.relation_coordinate != slot.relation_coordinate
            || shell_for_coordinate(slot.relation_coordinate).ok()
                != Some(slot.class_event.angular_shell)
        {
            return false;
        }
        if should_be_occupied {
            let expected_age = representation
                .observed_routes
                .saturating_sub(slot.slot_index);
            if slot.class_event.observed_lease_age != expected_age
                || slot.class_event.multiplicity == 0
            {
                return false;
            }
        } else {
            let padding = ConstructionCausalReturnClassEvent {
                relation_coordinate: identity_coordinate,
                angular_shell: H4S3AngularShell::Coincident,
                observed_lease_age: 0,
                multiplicity: 0,
                occupied: false,
            };
            if slot.class_event != padding
                || slot.prefix_state != identity_state
                || slot.suffix_state != identity_state
                || slot.relation_state != identity_state
                || slot.prefix_coordinate != identity_coordinate
                || slot.suffix_coordinate != identity_coordinate
                || slot.relation_coordinate != identity_coordinate
            {
                return false;
            }
        }
        if representation.r_full.slots[index] != slot.class_event {
            return false;
        }
    }

    let mut multiplicities = BTreeMap::<H4RootCoordinate, u8>::new();
    for slot in representation
        .slots
        .iter()
        .filter(|slot| slot.class_event.occupied)
    {
        let Some(next) = multiplicities
            .entry(slot.class_event.relation_coordinate)
            .or_default()
            .checked_add(1)
        else {
            return false;
        };
        multiplicities.insert(slot.class_event.relation_coordinate, next);
    }
    if representation
        .slots
        .iter()
        .filter(|slot| slot.class_event.occupied)
        .any(|slot| {
            multiplicities
                .get(&slot.class_event.relation_coordinate)
                .copied()
                != Some(slot.class_event.multiplicity)
        })
    {
        return false;
    }

    let Some(expected_min) = representation
        .slots
        .iter()
        .filter(|slot| slot.class_event.occupied)
        .min_by(|left, right| r_min_cmp(left.class_event, right.class_event))
    else {
        return false;
    };
    expected_min.slot_index == representation.r_min_slot_index
        && expected_min.class_event == representation.r_min
}

fn r_min_cmp(
    left: ConstructionCausalReturnClassEvent,
    right: ConstructionCausalReturnClassEvent,
) -> std::cmp::Ordering {
    left.angular_shell
        .cmp(&right.angular_shell)
        .then_with(|| Reverse(left.multiplicity).cmp(&Reverse(right.multiplicity)))
        .then_with(|| left.observed_lease_age.cmp(&right.observed_lease_age))
}

fn shell_for_coordinate(
    coordinate: H4RootCoordinate,
) -> Result<H4S3AngularShell, ConstructionCausalReturnError> {
    match coordinate.scaled_zphi_quaternion[0] {
        [2, 0] => Ok(H4S3AngularShell::Coincident),
        [0, 1] => Ok(H4S3AngularShell::Degrees36),
        [1, 0] => Ok(H4S3AngularShell::Degrees60),
        [-1, 1] => Ok(H4S3AngularShell::Degrees72),
        [0, 0] => Ok(H4S3AngularShell::Orthogonal),
        [1, -1] => Ok(H4S3AngularShell::Degrees108),
        [-1, 0] => Ok(H4S3AngularShell::Degrees120),
        [0, -1] => Ok(H4S3AngularShell::Degrees144),
        [-2, 0] => Ok(H4S3AngularShell::Antipodal),
        other => Err(ConstructionCausalReturnError::Invalid(format!(
            "H4 relation has noncanonical signed S3 real coordinate {other:?}"
        ))),
    }
}

fn action_counts(
    actions: &[ConstructionCausalReturnAction],
) -> Result<(usize, usize), ConstructionCausalReturnError> {
    let mut select = 0usize;
    let mut reject = 0usize;
    for action in actions {
        let target = match action {
            ConstructionCausalReturnAction::Select => &mut select,
            ConstructionCausalReturnAction::Reject => &mut reject,
        };
        *target = target
            .checked_add(1)
            .ok_or(ConstructionCausalReturnError::ArithmeticOverflow)?;
    }
    Ok((select, reject))
}

fn checked_mul(left: usize, right: usize) -> Result<usize, ConstructionCausalReturnError> {
    left.checked_mul(right)
        .ok_or(ConstructionCausalReturnError::ArithmeticOverflow)
}

#[derive(Debug, Serialize)]
struct ConstructionCausalReturnObservedHistorySeed<'a> {
    schema: u32,
    domain: &'static str,
    address_kappas: &'a [String],
}

fn address_kappas(
    addresses: &[GeometricAddress],
) -> Result<Vec<String>, ConstructionCausalReturnError> {
    addresses
        .iter()
        .map(GeometricAddress::canonical_kappa)
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

fn history_kappa_from_address_kappas(
    address_kappas: &[String],
) -> Result<String, ConstructionCausalReturnError> {
    canonical_kappa(&ConstructionCausalReturnObservedHistorySeed {
        schema: CONSTRUCTION_CAUSAL_RETURN_SCHEMA,
        domain: "uor-r4.construction-causal-return-observed-history/1",
        address_kappas,
    })
}

fn increment_usize_count(
    counts: &mut BTreeMap<String, usize>,
    key: String,
) -> Result<(), ConstructionCausalReturnError> {
    let count = counts.entry(key).or_default();
    *count = count
        .checked_add(1)
        .ok_or(ConstructionCausalReturnError::ArithmeticOverflow)?;
    Ok(())
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, ConstructionCausalReturnError> {
    serde_json::to_vec(value)
        .map_err(|error| ConstructionCausalReturnError::Serialization(error.to_string()))
}

fn canonical_kappa<T: Serialize>(value: &T) -> Result<String, ConstructionCausalReturnError> {
    let bytes = canonical_json(value)?;
    Ok(format!("blake3:{}", blake3::hash(&bytes).to_hex().as_str()))
}

fn validate_blake3_label(value: &str, field: &str) -> Result<(), ConstructionCausalReturnError> {
    let Some(hex) = value.strip_prefix("blake3:") else {
        return Err(ConstructionCausalReturnError::Invalid(format!(
            "{field} must use a blake3 label"
        )));
    };
    if hex.len() != 64
        || !hex
            .as_bytes()
            .iter()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
    {
        return Err(ConstructionCausalReturnError::Invalid(format!(
            "{field} must contain 64 lowercase hexadecimal digits"
        )));
    }
    Ok(())
}

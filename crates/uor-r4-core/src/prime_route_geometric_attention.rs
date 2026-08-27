//! Bounded causal geometric attention over a compiled prime-route manifest.
//!
//! Compilation may evaluate the fixed zeta basis with floating point. The
//! query path is deliberately narrower: three direct rows, one divisor row,
//! three adjacent spin-sector rows, and integer/table-backed energy terms.
//! It accepts only accumulated causal state; there is no future-route input.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU16;

use serde::Serialize;

use crate::canonical_lexical_ingestion::{
    h4_leaf_state_for_address, validate_ordered_h4_table_exact, CanonicalLexicalError,
    H4BinaryIcosahedralClosure, H4RootCoordinate, OrderedH4FoldState,
};
use crate::prime_route_attention::{
    zeta_phase_delta, CandidateRow, CompiledSpinManifest, GeometricAddress, OrderedRouteKappa,
    OrderedSentenceRouteState, PhaseQ29, PrimeAtom, PrimeRouteError, RouteIndexes,
    MANIFEST_MAX_ADDRESSES, MANIFEST_MAX_CANDIDATES_PER_ROW, MANIFEST_MAX_REBUILD_WITNESSES,
};

/// Fixed, sparse channels from the manifest-bound 512-zero grid. The indices
/// follow a Fibonacci spread and are part of this stage's compile contract.
pub const ATTENTION_ZETA_CHANNELS: [u16; 8] = [0, 1, 2, 3, 5, 8, 13, 21];
pub const ATTENTION_TORSION_BINS: u8 = 16;
pub const ATTENTION_ADJACENT_SPIN_ROWS: usize = 3;
pub const ATTENTION_DIRECT_ROWS: usize = 3;
pub const ATTENTION_DIVISOR_ROWS_PER_QUERY: usize = 1;
pub const ATTENTION_ROWS_PER_QUERY: usize =
    ATTENTION_DIRECT_ROWS + ATTENTION_DIVISOR_ROWS_PER_QUERY + ATTENTION_ADJACENT_SPIN_ROWS;
pub const ATTENTION_MAX_DIVISOR_ROWS: usize = MANIFEST_MAX_ADDRESSES;
pub const ATTENTION_MAX_SPIN_ROWS: usize = 8 * ATTENTION_TORSION_BINS as usize;
pub const ATTENTION_MAX_PHASE_ATOMS: usize = MANIFEST_MAX_ADDRESSES;
pub const ATTENTION_MAX_CANDIDATE_ENTRIES_PER_QUERY: usize =
    ATTENTION_ROWS_PER_QUERY * MANIFEST_MAX_CANDIDATES_PER_ROW as usize;
/// #969's first local attention mechanism is deliberately bounded to the
/// 2--8 lexical-unit loop named by the issue. The state keeps exact prefix
/// products rather than a digest so every earlier route can participate in
/// candidate-relative path closure.
pub const LOCAL_PATH_ATTENTION_MAX_UNITS: usize = 8;

// round(pi * 2^29) and round(2*pi * 2^29), fixed by the manifest's Q29 chart.
const PHASE_HALF_Q29: i64 = 1_686_629_713;
const PHASE_MODULUS_Q29: i64 = 3_373_259_426;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeometricAttentionError {
    Manifest(PrimeRouteError),
    Invalid(String),
    ArithmeticOverflow,
}

impl std::fmt::Display for GeometricAttentionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manifest(error) => write!(formatter, "prime-route manifest rejected: {error}"),
            Self::Invalid(reason) => write!(formatter, "invalid geometric attention: {reason}"),
            Self::ArithmeticOverflow => {
                formatter.write_str("geometric-attention arithmetic overflow")
            }
        }
    }
}

impl std::error::Error for GeometricAttentionError {}

impl From<PrimeRouteError> for GeometricAttentionError {
    fn from(error: PrimeRouteError) -> Self {
        Self::Manifest(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpinSector {
    pub hopf_octant: u8,
    pub torsion_bin: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttentionControl {
    /// Manifest geometry and fixed-zeta phase continuation.
    RealGeometry,
    /// The same candidates and geometry evaluations, assigned to candidates
    /// by a deterministic cyclic permutation.
    PermutedGeometry,
    /// The same candidates and row ceilings, ranked only by observed counts.
    CountOnly,
}

/// Diagnostic perturbations of already-observed continuation deltas. These
/// alter no lookup key, row, candidate, or count and carry no future route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttentionGeometryIntervention {
    None,
    PhaseDeltaOffset(PhaseQ29),
    TorsionDeltaOffset(PhaseQ29),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttentionRowSource {
    LastOne,
    LastTwo,
    OrderedSentence,
    Divisor,
    AdjacentSpin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttentionRowKey {
    LastOne(GeometricAddress),
    LastTwo {
        previous: GeometricAddress,
        last: GeometricAddress,
    },
    LastTwoUnavailable,
    OrderedSentence(String),
    Divisor(PrimeAtom),
    AdjacentSpin(SpinSector),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttentionRowRead {
    pub source: AttentionRowSource,
    pub key: AttentionRowKey,
    pub hit: bool,
    pub candidate_entries_examined: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AttentionSourceCounts {
    pub last_one: u32,
    pub last_two: u32,
    pub ordered_sentence: u32,
    pub divisor: u32,
    pub adjacent_spin: u32,
}

impl AttentionSourceCounts {
    pub fn source_breadth(self) -> u8 {
        u8::from(self.last_one > 0)
            + u8::from(self.last_two > 0)
            + u8::from(self.ordered_sentence > 0)
            + u8::from(self.divisor > 0)
            + u8::from(self.adjacent_spin > 0)
    }

    pub fn total(self) -> u64 {
        u64::from(self.last_one)
            + u64::from(self.last_two)
            + u64::from(self.ordered_sentence)
            + u64::from(self.divisor)
            + u64::from(self.adjacent_spin)
    }

    fn add(
        &mut self,
        source: AttentionRowSource,
        count: u32,
    ) -> Result<(), GeometricAttentionError> {
        let target = match source {
            AttentionRowSource::LastOne => &mut self.last_one,
            AttentionRowSource::LastTwo => &mut self.last_two,
            AttentionRowSource::OrderedSentence => &mut self.ordered_sentence,
            AttentionRowSource::Divisor => &mut self.divisor,
            AttentionRowSource::AdjacentSpin => &mut self.adjacent_spin,
        };
        *target = target
            .checked_add(count)
            .ok_or(GeometricAttentionError::ArithmeticOverflow)?;
        Ok(())
    }
}

/// Lexicographic least-energy vector. No cross-domain floating-point weights
/// are invented: phase, torsion, spin, then factor energy break ties in that
/// declared order before empirical support and canonical address.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct AttentionEnergy {
    pub phase: u64,
    pub torsion: u64,
    pub spin: u64,
    pub factor: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttentionTieBreakStage {
    PhaseEnergy,
    TorsionEnergy,
    SpinEnergy,
    FactorEnergy,
    SourceBreadth,
    TotalSupport,
    OrderedSentenceSupport,
    LastTwoSupport,
    LastOneSupport,
    DivisorSupport,
    AdjacentSpinSupport,
    CanonicalAddress,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttentionCandidateTrace {
    pub next: GeometricAddress,
    pub source_counts: AttentionSourceCounts,
    /// Energy measured for this candidate under the real manifest geometry.
    pub measured_energy: AttentionEnergy,
    /// Energy used for ranking under the selected control.
    pub ranking_energy: AttentionEnergy,
    /// Candidate whose measured geometry supplied `ranking_energy`. It is the
    /// candidate itself for real/count-only and another support member for
    /// the deterministic permutation control.
    pub geometry_source_next: GeometricAddress,
}

/// One naturally admitted continuation before any geometric field is read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttentionSupportCandidateTrace {
    pub next: GeometricAddress,
    pub source_counts: AttentionSourceCounts,
}

/// Pre-selection result of the bounded seven-row candidate lookup. This trace
/// ends after count/source-breadth admission and canonical ordering: it carries
/// no measured candidate energy, H4 state, path cost, or selected candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttentionSupportTrace {
    pub manifest_kappa: String,
    pub rows_read: Vec<AttentionRowRead>,
    pub candidate_entries_examined: usize,
    pub candidate_entry_ceiling: usize,
    pub unique_candidates_before_ceiling: usize,
    pub candidate_ceiling: usize,
    pub support_admission: AttentionSupportAdmission,
    pub candidates: Vec<AttentionSupportCandidateTrace>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeometricAttentionTrace {
    pub manifest_kappa: String,
    pub control: AttentionControl,
    pub geometry_intervention: AttentionGeometryIntervention,
    pub rows_read: Vec<AttentionRowRead>,
    pub candidate_entries_examined: usize,
    pub candidate_entry_ceiling: usize,
    pub unique_candidates_before_ceiling: usize,
    pub candidate_ceiling: usize,
    /// Count/source-breadth admission applied before any least-energy ranking.
    pub support_admission: AttentionSupportAdmission,
    pub geometry_evaluations: usize,
    pub tie_break_stages: Vec<AttentionTieBreakStage>,
    /// Candidates in final deterministic rank order.
    pub candidates: Vec<AttentionCandidateTrace>,
    /// Full support record for the selected candidate.
    pub selected: Option<AttentionCandidateTrace>,
}

/// Which causal state is allowed to influence the #969 path-lease score.
/// Every arm performs the same number of candidate/key comparisons; controls
/// repeat their one active key to keep the bounded work denominator equal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathLeaseControl {
    /// Candidate-appended full ordered state against every earlier prefix.
    FullPath,
    /// Only the last observed route and the candidate remain active.
    LastOnly,
    /// Candidate geometry only; all observed history is disabled.
    StateDisabled,
}

/// Exact finite proxy for round-S3 great-circle lease cost. `angular_shell` is
/// the signed real-coordinate shell of `key^-1 * query` in the canonical H4
/// root set. `lease_age` prefers the most recent equally close causal prefix.
/// No opaque table offset, digest, or payload identity is interpreted as a
/// scalar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum H4S3AngularShell {
    Coincident,
    Degrees36,
    Degrees60,
    Degrees72,
    Orthogonal,
    Degrees108,
    Degrees120,
    Degrees144,
    Antipodal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct PathLeaseCost {
    pub angular_shell: H4S3AngularShell,
    pub lease_age: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathLeaseCandidateTrace {
    pub next: GeometricAddress,
    pub source_counts: AttentionSourceCounts,
    pub query_state: H4RootCoordinate,
    pub best_prefix_index: u8,
    pub best_prefix_state: H4RootCoordinate,
    pub best_relative_state: H4RootCoordinate,
    pub cost: PathLeaseCost,
}

/// API-neutral select-or-abstain trace for the first causal R4/S3 attention
/// mechanism. `support` is the unchanged pre-selection schema-2 bounded row
/// union and carries no candidate energy or selection of its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathLeaseAttentionTrace {
    pub manifest_kappa: String,
    pub control: PathLeaseControl,
    pub observed_routes: u8,
    pub memory_keys_per_candidate: usize,
    pub path_geometry_evaluations: usize,
    pub support: AttentionSupportTrace,
    pub candidates: Vec<PathLeaseCandidateTrace>,
    pub minimum_cost: Option<PathLeaseCost>,
    pub tie: bool,
    pub abstained: bool,
    pub selected: Option<PathLeaseCandidateTrace>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeometricAttentionCompileStats {
    pub witnessed_transitions: usize,
    pub divisor_rows: usize,
    pub spin_rows: usize,
    pub phase_atoms: usize,
    pub maximum_candidates_per_row: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeometricAttentionLookupBounds {
    pub rows_per_query: usize,
    pub candidate_entries_per_query: usize,
    pub unique_candidates_after_ceiling: usize,
}

/// The common, geometry-independent admission rule applied when the bounded
/// row union contains more unique routes than the manifest candidate ceiling.
/// Least-energy ranking applies only to the support retained by this rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttentionSupportAdmission {
    SourceBreadthThenTotalCountThenCanonicalAddress,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AttentionRowCandidate {
    next: GeometricAddress,
    count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AttentionRow {
    candidates: Vec<AttentionRowCandidate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AtomPhaseSignature {
    phases: [i32; ATTENTION_ZETA_CHANNELS.len()],
}

/// Incremental state accepted by the production query. `observe` appends a
/// route that has already happened; candidate/future routes are not part of
/// the query type or method signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CausalAttentionState {
    manifest_kappa: String,
    previous: Option<GeometricAddress>,
    last: GeometricAddress,
    sentence: OrderedSentenceRouteState,
}

/// API-neutral ordered history accumulator. It carries no candidate rows and
/// is deliberately separate from [`CausalAttentionState`], so introducing the
/// associative fold cannot alter the existing query or admission path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CausalOrderedH4State {
    manifest_kappa: String,
    h4_root_table_kappa: String,
    multiplication_table_kappa: String,
    fold_state: OrderedH4FoldState,
    observed_routes: u32,
}

/// Bounded causal state for the local path-lease selector. `prefix_states`
/// contains `P_0 = 1` followed by the exact ordered product after each
/// observation. Unlike [`OrderedSentenceRouteState`], these are usable
/// non-digest geometric states; unlike a stored continuation row, they contain
/// no future candidate or payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CausalPathAttentionState {
    causal: CausalAttentionState,
    manifest_kappa: String,
    h4_root_table_kappa: String,
    multiplication_table_kappa: String,
    prefix_states: Vec<OrderedH4FoldState>,
}

impl CausalPathAttentionState {
    pub fn observed_routes(&self) -> usize {
        self.prefix_states.len().saturating_sub(1)
    }

    pub fn manifest_kappa(&self) -> &str {
        &self.manifest_kappa
    }

    pub fn h4_root_table_kappa(&self) -> &str {
        &self.h4_root_table_kappa
    }

    pub fn multiplication_table_kappa(&self) -> &str {
        &self.multiplication_table_kappa
    }

    /// Current exact ordered route product `P_t`.
    pub fn fold_state(&self) -> OrderedH4FoldState {
        // Construction and observation always retain P_0 plus at least one
        // observed prefix.
        self.prefix_states[self.prefix_states.len() - 1]
    }

    /// Exact prefix products, including the identity `P_0` and current `P_t`.
    pub fn prefix_states(&self) -> &[OrderedH4FoldState] {
        &self.prefix_states
    }
}

impl CausalOrderedH4State {
    fn from_first(
        manifest_kappa: String,
        first_observation: &GeometricAddress,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<Self, GeometricAttentionError> {
        Ok(Self {
            manifest_kappa,
            h4_root_table_kappa: table.h4_root_table_kappa.clone(),
            multiplication_table_kappa: table.multiplication_table_kappa.clone(),
            fold_state: h4_leaf_state_for_address(first_observation, table)
                .map_err(ordered_h4_error)?,
            observed_routes: 1,
        })
    }

    /// Append one already-observed route in causal left-to-right order.
    fn observe(
        &mut self,
        observed_route: &GeometricAddress,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<(), GeometricAttentionError> {
        let leaf = h4_leaf_state_for_address(observed_route, table).map_err(ordered_h4_error)?;
        let next_fold_state = self
            .fold_state
            .compose(leaf, table)
            .map_err(ordered_h4_error)?;
        let next_observed_routes = self
            .observed_routes
            .checked_add(1)
            .ok_or(GeometricAttentionError::ArithmeticOverflow)?;
        self.fold_state = next_fold_state;
        self.observed_routes = next_observed_routes;
        Ok(())
    }

    pub const fn fold_state(&self) -> OrderedH4FoldState {
        self.fold_state
    }

    pub const fn observed_routes(&self) -> u32 {
        self.observed_routes
    }

    pub fn manifest_kappa(&self) -> &str {
        &self.manifest_kappa
    }

    pub fn h4_root_table_kappa(&self) -> &str {
        &self.h4_root_table_kappa
    }

    pub fn multiplication_table_kappa(&self) -> &str {
        &self.multiplication_table_kappa
    }

    /// Resolve the opaque fold state to its exact scaled `Z[phi]` quaternion.
    /// The numeric table key remains an addressing detail and is never exposed
    /// as a scalar coordinate or distance.
    pub fn root_coordinate(
        &self,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<H4RootCoordinate, GeometricAttentionError> {
        validate_ordered_h4_table_exact(table).map_err(ordered_h4_error)?;
        if self.h4_root_table_kappa != table.h4_root_table_kappa
            || self.multiplication_table_kappa != table.multiplication_table_kappa
        {
            return Err(GeometricAttentionError::Invalid(
                "causal ordered H4 state is bound to a different exact table".to_owned(),
            ));
        }
        self.fold_state
            .root_coordinate(table)
            .map_err(ordered_h4_error)
    }
}

fn ordered_h4_error(error: CanonicalLexicalError) -> GeometricAttentionError {
    GeometricAttentionError::Invalid(format!("ordered H4 state: {error}"))
}

impl CausalAttentionState {
    fn new(
        manifest_kappa: String,
        first_observation: GeometricAddress,
    ) -> Result<Self, GeometricAttentionError> {
        let mut sentence = OrderedSentenceRouteState::new()?;
        sentence.append(&first_observation)?;
        Ok(Self {
            manifest_kappa,
            previous: None,
            last: first_observation,
            sentence,
        })
    }

    fn observe(&mut self, observed_route: GeometricAddress) -> Result<(), GeometricAttentionError> {
        self.sentence.append(&observed_route)?;
        self.previous = Some(std::mem::replace(&mut self.last, observed_route));
        Ok(())
    }

    pub fn previous(&self) -> Option<&GeometricAddress> {
        self.previous.as_ref()
    }

    pub const fn last(&self) -> &GeometricAddress {
        &self.last
    }

    pub fn sentence_key(&self) -> Result<&OrderedRouteKappa, GeometricAttentionError> {
        self.sentence.key().ok_or_else(|| {
            GeometricAttentionError::Invalid(
                "causal attention state has no ordered sentence key".to_owned(),
            )
        })
    }

    pub const fn observed_routes(&self) -> u32 {
        self.sentence.route_count()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeometricAttentionArtifact {
    manifest_kappa: String,
    maximum_candidates: NonZeroU16,
    address_registry: BTreeSet<GeometricAddress>,
    atom_payloads: BTreeMap<PrimeAtom, String>,
    direct_indexes: RouteIndexes,
    divisor_rows: BTreeMap<PrimeAtom, AttentionRow>,
    spin_rows: BTreeMap<SpinSector, AttentionRow>,
    atom_phases: BTreeMap<PrimeAtom, AtomPhaseSignature>,
    stats: GeometricAttentionCompileStats,
}

impl GeometricAttentionArtifact {
    /// Compile bounded fallback tables solely from the manifest's canonical
    /// public rebuild witnesses. The resulting artifact is pinned to the
    /// manifest kappa and never retains the witness/corpus population.
    pub fn compile_from_manifest_witnesses(
        manifest: &CompiledSpinManifest,
    ) -> Result<Self, GeometricAttentionError> {
        // This validates the manifest shape and proves the supplied kappa
        // reproduces before the attention artifact binds to it.
        manifest.canonical_bytes()?;
        if manifest.rebuild_witnesses.len() > MANIFEST_MAX_REBUILD_WITNESSES {
            return Err(GeometricAttentionError::Invalid(
                "rebuild witness count exceeds the hard attention ceiling".to_owned(),
            ));
        }
        if manifest.maximum_candidates.get() > MANIFEST_MAX_CANDIDATES_PER_ROW {
            return Err(GeometricAttentionError::Invalid(
                "manifest candidate ceiling exceeds the attention hard cap".to_owned(),
            ));
        }

        let mut divisor_counts = BTreeMap::<PrimeAtom, BTreeMap<GeometricAddress, u32>>::new();
        let mut spin_counts = BTreeMap::<SpinSector, BTreeMap<GeometricAddress, u32>>::new();
        let mut witnessed_transitions = 0usize;
        for witness in &manifest.rebuild_witnesses {
            for pair in witness.address_indices.windows(2) {
                let current = manifest
                    .addresses
                    .get(usize::from(pair[0]))
                    .ok_or_else(|| {
                        GeometricAttentionError::Invalid(
                            "rebuild witness current-address index is out of range".to_owned(),
                        )
                    })?;
                let next = manifest
                    .addresses
                    .get(usize::from(pair[1]))
                    .ok_or_else(|| {
                        GeometricAttentionError::Invalid(
                            "rebuild witness next-address index is out of range".to_owned(),
                        )
                    })?;
                increment_count(
                    divisor_counts
                        .entry(current.atom)
                        .or_default()
                        .entry(next.clone())
                        .or_default(),
                )?;
                increment_count(
                    spin_counts
                        .entry(spin_sector(current))
                        .or_default()
                        .entry(next.clone())
                        .or_default(),
                )?;
                witnessed_transitions = witnessed_transitions
                    .checked_add(1)
                    .ok_or(GeometricAttentionError::ArithmeticOverflow)?;
            }
        }

        let maximum_candidates = manifest.maximum_candidates;
        let divisor_rows = finalize_rows(divisor_counts, maximum_candidates);
        let spin_rows = finalize_rows(spin_counts, maximum_candidates);
        if divisor_rows.len() > ATTENTION_MAX_DIVISOR_ROWS {
            return Err(GeometricAttentionError::Invalid(
                "compiled divisor row count exceeds the hard ceiling".to_owned(),
            ));
        }
        if spin_rows.len() > ATTENTION_MAX_SPIN_ROWS {
            return Err(GeometricAttentionError::Invalid(
                "compiled spin row count exceeds the hard ceiling".to_owned(),
            ));
        }

        let atoms = manifest
            .addresses
            .iter()
            .map(|address| address.atom)
            .collect::<BTreeSet<_>>();
        if atoms.len() > ATTENTION_MAX_PHASE_ATOMS {
            return Err(GeometricAttentionError::Invalid(
                "compiled zeta phase-atom count exceeds the hard ceiling".to_owned(),
            ));
        }
        let phase_origin = PrimeAtom::new(2)?;
        let mut atom_phases = BTreeMap::new();
        for atom in atoms {
            let mut phases = [0i32; ATTENTION_ZETA_CHANNELS.len()];
            for (target, channel) in phases.iter_mut().zip(ATTENTION_ZETA_CHANNELS) {
                *target = zeta_phase_delta(channel, phase_origin, atom)?.raw();
            }
            atom_phases.insert(atom, AtomPhaseSignature { phases });
        }

        let stats = GeometricAttentionCompileStats {
            witnessed_transitions,
            divisor_rows: divisor_rows.len(),
            spin_rows: spin_rows.len(),
            phase_atoms: atom_phases.len(),
            maximum_candidates_per_row: maximum_candidates.get(),
        };
        let address_registry = manifest.addresses.iter().cloned().collect();
        let atom_payloads = manifest
            .prime_registry
            .bindings
            .iter()
            .map(|binding| (binding.atom, binding.payload_cid.clone()))
            .collect();
        Ok(Self {
            manifest_kappa: manifest.manifest_kappa.clone(),
            maximum_candidates,
            address_registry,
            atom_payloads,
            direct_indexes: manifest.indexes.clone(),
            divisor_rows,
            spin_rows,
            atom_phases,
            stats,
        })
    }

    pub fn manifest_kappa(&self) -> &str {
        &self.manifest_kappa
    }

    pub fn causal_state(
        &self,
        first_observation: GeometricAddress,
    ) -> Result<CausalAttentionState, GeometricAttentionError> {
        self.validate_observed_address(&first_observation)?;
        CausalAttentionState::new(self.manifest_kappa.clone(), first_observation)
    }

    pub fn causal_state_from_history(
        &self,
        history: &[GeometricAddress],
    ) -> Result<CausalAttentionState, GeometricAttentionError> {
        let first = history.first().cloned().ok_or_else(|| {
            GeometricAttentionError::Invalid(
                "causal attention requires at least one observed route".to_owned(),
            )
        })?;
        let mut state = self.causal_state(first)?;
        for observation in &history[1..] {
            self.observe(&mut state, observation.clone())?;
        }
        Ok(state)
    }

    /// Fold the same already-observed address history into the independent H4
    /// ordered-state overlay. Candidate support, admission, and ranking are not
    /// consulted by this helper.
    pub fn causal_ordered_state_from_history(
        &self,
        history: &[GeometricAddress],
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<CausalOrderedH4State, GeometricAttentionError> {
        validate_ordered_h4_table_exact(table).map_err(ordered_h4_error)?;
        let first = history.first().ok_or_else(|| {
            GeometricAttentionError::Invalid(
                "causal ordered H4 state requires at least one observed route".to_owned(),
            )
        })?;
        self.validate_observed_address(first)?;
        let mut state =
            CausalOrderedH4State::from_first(self.manifest_kappa.clone(), first, table)?;
        for observation in &history[1..] {
            self.validate_observed_address(observation)?;
            state.observe(observation, table)?;
        }
        Ok(state)
    }

    /// Append one observed route to the independent ordered-state overlay.
    pub fn observe_ordered(
        &self,
        state: &mut CausalOrderedH4State,
        observed_route: &GeometricAddress,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<(), GeometricAttentionError> {
        validate_ordered_h4_table_exact(table).map_err(ordered_h4_error)?;
        if state.manifest_kappa != self.manifest_kappa {
            return Err(GeometricAttentionError::Invalid(
                "causal ordered H4 state is bound to a different manifest".to_owned(),
            ));
        }
        if state.h4_root_table_kappa != table.h4_root_table_kappa
            || state.multiplication_table_kappa != table.multiplication_table_kappa
        {
            return Err(GeometricAttentionError::Invalid(
                "causal ordered H4 state is bound to a different exact table".to_owned(),
            ));
        }
        self.validate_observed_address(observed_route)?;
        state.observe(observed_route, table)
    }

    /// Build the bounded, exact prefix memory used by the #969 local
    /// path-lease selector. The history is observed-only and may contain at
    /// most [`LOCAL_PATH_ATTENTION_MAX_UNITS`] routes.
    pub fn causal_path_state_from_history(
        &self,
        history: &[GeometricAddress],
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<CausalPathAttentionState, GeometricAttentionError> {
        validate_ordered_h4_table_exact(table).map_err(ordered_h4_error)?;
        if history.is_empty() || history.len() > LOCAL_PATH_ATTENTION_MAX_UNITS {
            return Err(GeometricAttentionError::Invalid(format!(
                "causal path attention requires 1--{LOCAL_PATH_ATTENTION_MAX_UNITS} observed routes"
            )));
        }
        let causal = self.causal_state_from_history(history)?;
        let mut prefix_states =
            Vec::with_capacity(LOCAL_PATH_ATTENTION_MAX_UNITS.saturating_add(1));
        let mut fold = OrderedH4FoldState::identity(table).map_err(ordered_h4_error)?;
        prefix_states.push(fold);
        for observed in history {
            self.validate_observed_address(observed)?;
            let leaf = h4_leaf_state_for_address(observed, table).map_err(ordered_h4_error)?;
            fold = fold.compose(leaf, table).map_err(ordered_h4_error)?;
            prefix_states.push(fold);
        }
        Ok(CausalPathAttentionState {
            causal,
            manifest_kappa: self.manifest_kappa.clone(),
            h4_root_table_kappa: table.h4_root_table_kappa.clone(),
            multiplication_table_kappa: table.multiplication_table_kappa.clone(),
            prefix_states,
        })
    }

    /// Append one already-observed route to both the existing bounded lookup
    /// state and the exact prefix-product memory. The update fails before
    /// mutation when the 8-unit local bound would be exceeded.
    pub fn observe_path(
        &self,
        state: &mut CausalPathAttentionState,
        observed_route: GeometricAddress,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<(), GeometricAttentionError> {
        self.validate_path_state_binding(state, table)?;
        if state.observed_routes() >= LOCAL_PATH_ATTENTION_MAX_UNITS {
            return Err(GeometricAttentionError::Invalid(format!(
                "causal path attention exceeded its {LOCAL_PATH_ATTENTION_MAX_UNITS}-route bound"
            )));
        }
        self.validate_observed_address(&observed_route)?;
        let leaf = h4_leaf_state_for_address(&observed_route, table).map_err(ordered_h4_error)?;
        let next_fold = state
            .fold_state()
            .compose(leaf, table)
            .map_err(ordered_h4_error)?;
        self.observe(&mut state.causal, observed_route)?;
        state.prefix_states.push(next_fold);
        Ok(())
    }

    pub fn observe(
        &self,
        state: &mut CausalAttentionState,
        observed_route: GeometricAddress,
    ) -> Result<(), GeometricAttentionError> {
        self.validate_state_binding(state)?;
        self.validate_observed_address(&observed_route)?;
        state.observe(observed_route)
    }

    pub const fn compile_stats(&self) -> GeometricAttentionCompileStats {
        self.stats
    }

    pub fn lookup_bounds(&self) -> GeometricAttentionLookupBounds {
        let maximum = usize::from(self.maximum_candidates.get());
        GeometricAttentionLookupBounds {
            rows_per_query: ATTENTION_ROWS_PER_QUERY,
            candidate_entries_per_query: ATTENTION_ROWS_PER_QUERY * maximum,
            unique_candidates_after_ceiling: maximum,
        }
    }

    // BEGIN GEOMETRIC_ATTENTION_BOUNDED_LOOKUP
    /// Read and admit the bounded natural candidate support without evaluating
    /// candidate energy or H4 path geometry. The frozen row keys, including
    /// adjacent-spin admission rows, are still queried but only row/count data
    /// is returned. This is the common support seam for geometric queries,
    /// path-lease selection, and support-only preflight inspection.
    pub fn query_support_only(
        &self,
        state: &CausalAttentionState,
    ) -> Result<AttentionSupportTrace, GeometricAttentionError> {
        // Binding and exact address membership are checked before allocating a
        // trace or consulting any direct/fallback row.
        self.validate_state_binding(state)?;
        let mut rows_read = Vec::with_capacity(ATTENTION_ROWS_PER_QUERY);
        let mut merged = BTreeMap::<GeometricAddress, AttentionSourceCounts>::new();
        let sentence_key = state.sentence_key()?;

        self.read_direct_row(
            AttentionRowSource::LastOne,
            AttentionRowKey::LastOne(state.last.clone()),
            self.direct_indexes.last_one(&state.last),
            &mut merged,
            &mut rows_read,
        )?;
        let last_two_row = state
            .previous
            .as_ref()
            .and_then(|previous| self.direct_indexes.last_two(previous, &state.last));
        let last_two_key = match state.previous.as_ref() {
            Some(previous) => AttentionRowKey::LastTwo {
                previous: previous.clone(),
                last: state.last.clone(),
            },
            None => AttentionRowKey::LastTwoUnavailable,
        };
        self.read_direct_row(
            AttentionRowSource::LastTwo,
            last_two_key,
            last_two_row,
            &mut merged,
            &mut rows_read,
        )?;
        self.read_direct_row(
            AttentionRowSource::OrderedSentence,
            AttentionRowKey::OrderedSentence(sentence_key.as_str().to_owned()),
            self.direct_indexes.sentence_precomputed(sentence_key),
            &mut merged,
            &mut rows_read,
        )?;

        let divisor_row = self.divisor_rows.get(&state.last.atom);
        self.read_attention_row(
            AttentionRowSource::Divisor,
            AttentionRowKey::Divisor(state.last.atom),
            divisor_row,
            &mut merged,
            &mut rows_read,
        )?;

        for sector in adjacent_spin_sectors(spin_sector(&state.last)) {
            self.read_attention_row(
                AttentionRowSource::AdjacentSpin,
                AttentionRowKey::AdjacentSpin(sector),
                self.spin_rows.get(&sector),
                &mut merged,
                &mut rows_read,
            )?;
        }

        if rows_read.len() != ATTENTION_ROWS_PER_QUERY {
            return Err(GeometricAttentionError::Invalid(
                "bounded lookup did not account for every declared row".to_owned(),
            ));
        }
        let candidate_entries_examined = rows_read.iter().try_fold(0usize, |total, row| {
            total
                .checked_add(row.candidate_entries_examined)
                .ok_or(GeometricAttentionError::ArithmeticOverflow)
        })?;
        let bounds = self.lookup_bounds();
        if candidate_entries_examined > bounds.candidate_entries_per_query
            || candidate_entries_examined > ATTENTION_MAX_CANDIDATE_ENTRIES_PER_QUERY
        {
            return Err(GeometricAttentionError::Invalid(
                "lookup exceeded its declared candidate-entry ceiling".to_owned(),
            ));
        }

        let unique_candidates_before_ceiling = merged.len();
        let mut candidates = merged.into_iter().collect::<Vec<_>>();
        // This is explicit pre-geometric support admission. Consequently,
        // every downstream ranking is only over the admitted support, never a
        // claim about the full untruncated row union.
        candidates.sort_by(|(left_next, left_counts), (right_next, right_counts)| {
            (
                Reverse(left_counts.source_breadth()),
                Reverse(left_counts.total()),
                left_next,
            )
                .cmp(&(
                    Reverse(right_counts.source_breadth()),
                    Reverse(right_counts.total()),
                    right_next,
                ))
        });
        candidates.truncate(bounds.unique_candidates_after_ceiling);
        // Canonical order makes later geometry permutation independent of map
        // insertion order and count ordering.
        candidates.sort_by(|left, right| left.0.cmp(&right.0));

        Ok(AttentionSupportTrace {
            manifest_kappa: self.manifest_kappa.clone(),
            rows_read,
            candidate_entries_examined,
            candidate_entry_ceiling: bounds.candidate_entries_per_query,
            unique_candidates_before_ceiling,
            candidate_ceiling: bounds.unique_candidates_after_ceiling,
            support_admission:
                AttentionSupportAdmission::SourceBreadthThenTotalCountThenCanonicalAddress,
            candidates: candidates
                .into_iter()
                .map(|(next, source_counts)| AttentionSupportCandidateTrace {
                    next,
                    source_counts,
                })
                .collect(),
        })
    }

    pub fn query(
        &self,
        state: &CausalAttentionState,
        control: AttentionControl,
    ) -> Result<GeometricAttentionTrace, GeometricAttentionError> {
        self.query_with_intervention(state, control, AttentionGeometryIntervention::None)
    }

    /// Diagnostic counterpart to [`Self::query`]. The intervention changes
    /// only an accumulated phase/torsion delta; lookup support remains fixed.
    pub fn query_with_intervention(
        &self,
        state: &CausalAttentionState,
        control: AttentionControl,
        intervention: AttentionGeometryIntervention,
    ) -> Result<GeometricAttentionTrace, GeometricAttentionError> {
        let AttentionSupportTrace {
            manifest_kappa,
            rows_read,
            candidate_entries_examined,
            candidate_entry_ceiling,
            unique_candidates_before_ceiling,
            candidate_ceiling,
            support_admission,
            candidates: support_candidates,
        } = self.query_support_only(state)?;

        let mut measured = Vec::with_capacity(support_candidates.len());
        for candidate in &support_candidates {
            measured.push(self.measure_energy(
                state,
                &candidate.next,
                candidate.source_counts,
                intervention,
            )?);
        }
        let permutation_offset =
            deterministic_permutation_offset(&self.manifest_kappa, measured.len());
        let canonical_next = support_candidates
            .iter()
            .map(|candidate| candidate.next.clone())
            .collect::<Vec<_>>();
        let mut candidates = Vec::with_capacity(support_candidates.len());
        for (index, (candidate, measured_energy)) in support_candidates
            .into_iter()
            .zip(measured.iter().copied())
            .enumerate()
        {
            let AttentionSupportCandidateTrace {
                next,
                source_counts,
            } = candidate;
            let (ranking_energy, geometry_source_next) = match control {
                AttentionControl::RealGeometry => (measured_energy, next.clone()),
                AttentionControl::PermutedGeometry => {
                    let source_index = (index + permutation_offset) % measured.len();
                    (measured[source_index], canonical_next[source_index].clone())
                }
                AttentionControl::CountOnly => (AttentionEnergy::default(), next.clone()),
            };
            candidates.push(AttentionCandidateTrace {
                next,
                source_counts,
                measured_energy,
                ranking_energy,
                geometry_source_next,
            });
        }
        let tie_break_stages = tie_break_stages(control);
        candidates.sort_by(|left, right| compare_candidates(left, right, control));
        let selected = candidates.first().cloned();
        Ok(GeometricAttentionTrace {
            manifest_kappa,
            control,
            geometry_intervention: intervention,
            rows_read,
            candidate_entries_examined,
            candidate_entry_ceiling,
            unique_candidates_before_ceiling,
            candidate_ceiling,
            support_admission,
            geometry_evaluations: candidates.len(),
            tie_break_stages,
            candidates,
            selected,
        })
    }

    /// Select one naturally admitted route by exact causal path closure, or
    /// abstain when the minimum lease cost is shared. Admission is delegated
    /// unchanged to the schema-2 bounded lookup. Candidate ranking uses only
    /// exact H4 multiplication/inverse tables plus the signed S3 distance
    /// class of each relative state.
    pub fn select_path_or_abstain(
        &self,
        state: &CausalPathAttentionState,
        table: &H4BinaryIcosahedralClosure,
        control: PathLeaseControl,
    ) -> Result<PathLeaseAttentionTrace, GeometricAttentionError> {
        self.validate_path_state_binding(state, table)?;
        let observed_routes = state.observed_routes();
        if observed_routes < 2 {
            return Err(GeometricAttentionError::Invalid(
                "path-lease selection requires at least two observed routes".to_owned(),
            ));
        }
        if observed_routes >= LOCAL_PATH_ATTENTION_MAX_UNITS {
            return Err(GeometricAttentionError::Invalid(format!(
                "path-lease selection has no output slot inside its {LOCAL_PATH_ATTENTION_MAX_UNITS}-route bound"
            )));
        }

        // Freeze the natural support/admission before any H4 path operation.
        // This pre-selection trace has no candidate energy or selection of its own.
        let support = self.query_support_only(&state.causal)?;
        let memory_keys_per_candidate = observed_routes;
        let path_geometry_evaluations = support
            .candidates
            .len()
            .checked_mul(memory_keys_per_candidate)
            .ok_or(GeometricAttentionError::ArithmeticOverflow)?;
        let identity = OrderedH4FoldState::identity(table).map_err(ordered_h4_error)?;
        let current = state.fold_state();
        let last_prefix_index = observed_routes.checked_sub(1).ok_or_else(|| {
            GeometricAttentionError::Invalid(
                "path-lease state has no previous prefix for last-only control".to_owned(),
            )
        })?;

        let mut candidates = Vec::with_capacity(support.candidates.len());
        for admitted in &support.candidates {
            let candidate =
                h4_leaf_state_for_address(&admitted.next, table).map_err(ordered_h4_error)?;
            let query = match control {
                PathLeaseControl::FullPath | PathLeaseControl::LastOnly => current
                    .compose(candidate, table)
                    .map_err(ordered_h4_error)?,
                // Preserve the candidate-append table lookup performed by
                // the active arms while replacing only the causal state.
                PathLeaseControl::StateDisabled => identity
                    .compose(candidate, table)
                    .map_err(ordered_h4_error)?,
            };
            let mut best: Option<(PathLeaseCost, usize, OrderedH4FoldState, OrderedH4FoldState)> =
                None;
            for comparison_index in 0..memory_keys_per_candidate {
                let (prefix_index, key, lease_age) = match control {
                    PathLeaseControl::FullPath => {
                        let prefix_index = comparison_index;
                        let key = state.prefix_states[prefix_index];
                        let age = observed_routes
                            .checked_add(1)
                            .and_then(|value| value.checked_sub(prefix_index))
                            .ok_or(GeometricAttentionError::ArithmeticOverflow)?;
                        (
                            prefix_index,
                            key,
                            u8::try_from(age)
                                .map_err(|_| GeometricAttentionError::ArithmeticOverflow)?,
                        )
                    }
                    PathLeaseControl::LastOnly => {
                        (last_prefix_index, state.prefix_states[last_prefix_index], 2)
                    }
                    PathLeaseControl::StateDisabled => (0, identity, 1),
                };
                let relative = key
                    .inverse(table)
                    .and_then(|inverse| inverse.compose(query, table))
                    .map_err(ordered_h4_error)?;
                let cost = PathLeaseCost {
                    angular_shell: h4_s3_angular_shell(relative, table)?,
                    lease_age,
                };
                if best
                    .as_ref()
                    .is_none_or(|(best_cost, _, _, _)| cost < *best_cost)
                {
                    best = Some((cost, prefix_index, key, relative));
                }
            }
            let (cost, best_prefix_index, best_prefix, best_relative) = best.ok_or_else(|| {
                GeometricAttentionError::Invalid(
                    "path-lease candidate had no causal memory comparison".to_owned(),
                )
            })?;
            candidates.push(PathLeaseCandidateTrace {
                next: admitted.next.clone(),
                source_counts: admitted.source_counts,
                query_state: query.root_coordinate(table).map_err(ordered_h4_error)?,
                best_prefix_index: u8::try_from(best_prefix_index)
                    .map_err(|_| GeometricAttentionError::ArithmeticOverflow)?,
                best_prefix_state: best_prefix
                    .root_coordinate(table)
                    .map_err(ordered_h4_error)?,
                best_relative_state: best_relative
                    .root_coordinate(table)
                    .map_err(ordered_h4_error)?,
                cost,
            });
        }

        let minimum_cost = candidates.iter().map(|candidate| candidate.cost).min();
        let minimum_count = minimum_cost.map_or(0, |minimum| {
            candidates
                .iter()
                .filter(|candidate| candidate.cost == minimum)
                .count()
        });
        // Canonical address orders the trace only. Equal exact cost always
        // abstains; identity order never resolves a semantic tie.
        candidates.sort_by(|left, right| (&left.cost, &left.next).cmp(&(&right.cost, &right.next)));
        let selected = (minimum_count == 1).then(|| candidates[0].clone());
        let tie = minimum_count > 1;
        let abstained = selected.is_none();
        Ok(PathLeaseAttentionTrace {
            manifest_kappa: self.manifest_kappa.clone(),
            control,
            observed_routes: u8::try_from(observed_routes)
                .map_err(|_| GeometricAttentionError::ArithmeticOverflow)?,
            memory_keys_per_candidate,
            path_geometry_evaluations,
            support,
            candidates,
            minimum_cost,
            tie,
            abstained,
            selected,
        })
    }

    fn validate_path_state_binding(
        &self,
        state: &CausalPathAttentionState,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<(), GeometricAttentionError> {
        validate_ordered_h4_table_exact(table).map_err(ordered_h4_error)?;
        self.validate_state_binding(&state.causal)?;
        if state.manifest_kappa != self.manifest_kappa
            || state.h4_root_table_kappa != table.h4_root_table_kappa
            || state.multiplication_table_kappa != table.multiplication_table_kappa
        {
            return Err(GeometricAttentionError::Invalid(
                "causal path-attention state is bound to different manifest/table bytes".to_owned(),
            ));
        }
        if state.prefix_states.len() < 2
            || state.prefix_states.len() > LOCAL_PATH_ATTENTION_MAX_UNITS.saturating_add(1)
        {
            return Err(GeometricAttentionError::Invalid(
                "causal path-attention prefix memory violates its local bound".to_owned(),
            ));
        }
        Ok(())
    }

    fn validate_observed_address(
        &self,
        address: &GeometricAddress,
    ) -> Result<(), GeometricAttentionError> {
        let expected_payload = self.atom_payloads.get(&address.atom).ok_or_else(|| {
            GeometricAttentionError::Invalid(format!(
                "observed prime {} is absent from the bound manifest registry",
                address.atom.value()
            ))
        })?;
        if expected_payload != &address.payload_cid {
            return Err(GeometricAttentionError::Invalid(format!(
                "observed prime {} carries a payload outside the bound manifest registry",
                address.atom.value()
            )));
        }
        if !self.address_registry.contains(address) {
            return Err(GeometricAttentionError::Invalid(
                "observed route is not an exact member of the bound manifest address registry"
                    .to_owned(),
            ));
        }
        Ok(())
    }

    fn validate_state_binding(
        &self,
        state: &CausalAttentionState,
    ) -> Result<(), GeometricAttentionError> {
        if state.manifest_kappa != self.manifest_kappa {
            return Err(GeometricAttentionError::Invalid(
                "causal attention state is bound to a different manifest".to_owned(),
            ));
        }
        if let Some(previous) = state.previous.as_ref() {
            self.validate_observed_address(previous)?;
        }
        self.validate_observed_address(&state.last)
    }

    fn read_direct_row(
        &self,
        source: AttentionRowSource,
        key: AttentionRowKey,
        row: Option<&CandidateRow>,
        merged: &mut BTreeMap<GeometricAddress, AttentionSourceCounts>,
        trace: &mut Vec<AttentionRowRead>,
    ) -> Result<(), GeometricAttentionError> {
        let entries = row.map_or(&[][..], CandidateRow::candidates);
        self.validate_row_entries(entries.len())?;
        for candidate in entries {
            merged
                .entry(candidate.next.clone())
                .or_default()
                .add(source, candidate.count)?;
        }
        trace.push(AttentionRowRead {
            source,
            key,
            hit: row.is_some(),
            candidate_entries_examined: entries.len(),
        });
        Ok(())
    }

    fn read_attention_row(
        &self,
        source: AttentionRowSource,
        key: AttentionRowKey,
        row: Option<&AttentionRow>,
        merged: &mut BTreeMap<GeometricAddress, AttentionSourceCounts>,
        trace: &mut Vec<AttentionRowRead>,
    ) -> Result<(), GeometricAttentionError> {
        let entries = row.map_or(&[][..], |row| row.candidates.as_slice());
        self.validate_row_entries(entries.len())?;
        for candidate in entries {
            merged
                .entry(candidate.next.clone())
                .or_default()
                .add(source, candidate.count)?;
        }
        trace.push(AttentionRowRead {
            source,
            key,
            hit: row.is_some(),
            candidate_entries_examined: entries.len(),
        });
        Ok(())
    }

    fn validate_row_entries(&self, entries: usize) -> Result<(), GeometricAttentionError> {
        if entries > usize::from(self.maximum_candidates.get())
            || entries > MANIFEST_MAX_CANDIDATES_PER_ROW as usize
        {
            return Err(GeometricAttentionError::Invalid(
                "attention row exceeds its candidate ceiling".to_owned(),
            ));
        }
        Ok(())
    }

    fn measure_energy(
        &self,
        state: &CausalAttentionState,
        next: &GeometricAddress,
        counts: AttentionSourceCounts,
        intervention: AttentionGeometryIntervention,
    ) -> Result<AttentionEnergy, GeometricAttentionError> {
        let phase_offset = match intervention {
            AttentionGeometryIntervention::PhaseDeltaOffset(offset) => offset.raw(),
            _ => 0,
        };
        let torsion_offset = match intervention {
            AttentionGeometryIntervention::TorsionDeltaOffset(offset) => offset.raw(),
            _ => 0,
        };
        let phase = self.phase_energy(state.previous.as_ref(), &state.last, next, phase_offset)?;
        let torsion = continuation_phase_energy(
            state
                .previous
                .as_ref()
                .map(|address| address.spin.torsion.raw()),
            state.last.spin.torsion.raw(),
            next.spin.torsion.raw(),
            torsion_offset,
        );
        let spin = continuation_spin_energy(
            state
                .previous
                .as_ref()
                .map(|address| address.spin.hopf.raw()),
            state.last.spin.hopf.raw(),
            next.spin.hopf.raw(),
        )?;
        let divisor_penalty = if counts.divisor > 0 {
            0
        } else {
            PHASE_MODULUS_Q29 as u64
        };
        let factor = u64::from(state.last.atom.value().abs_diff(next.atom.value()))
            .checked_add(divisor_penalty)
            .ok_or(GeometricAttentionError::ArithmeticOverflow)?;
        Ok(AttentionEnergy {
            phase,
            torsion,
            spin,
            factor,
        })
    }

    fn phase_energy(
        &self,
        previous: Option<&GeometricAddress>,
        last: &GeometricAddress,
        next: &GeometricAddress,
        observed_delta_offset: i32,
    ) -> Result<u64, GeometricAttentionError> {
        let Some(previous) = previous else {
            return Ok(0);
        };
        let previous_phase = self.atom_phase(previous.atom)?;
        let last_phase = self.atom_phase(last.atom)?;
        let next_phase = self.atom_phase(next.atom)?;
        previous_phase
            .phases
            .iter()
            .zip(last_phase.phases)
            .zip(next_phase.phases)
            .try_fold(0u64, |total, ((previous, last), next)| {
                let observed = circular_add_delta(
                    circular_signed_delta(*previous, last),
                    i64::from(observed_delta_offset),
                );
                let proposed = circular_signed_delta(last, next);
                total
                    .checked_add(circular_delta_distance(observed, proposed))
                    .ok_or(GeometricAttentionError::ArithmeticOverflow)
            })
    }

    fn atom_phase(&self, atom: PrimeAtom) -> Result<&AtomPhaseSignature, GeometricAttentionError> {
        self.atom_phases.get(&atom).ok_or_else(|| {
            GeometricAttentionError::Invalid(format!(
                "prime {} has no compiled fixed-zeta phase signature",
                atom.value()
            ))
        })
    }
}

/// Exact monotone class of the S3 great-circle distance from the identity.
/// Canonical H4 roots are stored at coordinate scale two in `Z[phi]`; their
/// signed real coordinate has exactly these nine values. For unit
/// quaternions, `Re(key^-1 * query) = <key, query>`, so descending real
/// coordinate is the same ordering as ascending `acos` distance. Equal shells
/// remain ties, with the full signed relative quaternion retained in the trace.
fn h4_s3_angular_shell(
    relative: OrderedH4FoldState,
    table: &H4BinaryIcosahedralClosure,
) -> Result<H4S3AngularShell, GeometricAttentionError> {
    let real = relative
        .root_coordinate(table)
        .map_err(ordered_h4_error)?
        .scaled_zphi_quaternion[0];
    match real {
        [2, 0] => Ok(H4S3AngularShell::Coincident),
        [0, 1] => Ok(H4S3AngularShell::Degrees36),
        [1, 0] => Ok(H4S3AngularShell::Degrees60),
        [-1, 1] => Ok(H4S3AngularShell::Degrees72),
        [0, 0] => Ok(H4S3AngularShell::Orthogonal),
        [1, -1] => Ok(H4S3AngularShell::Degrees108),
        [-1, 0] => Ok(H4S3AngularShell::Degrees120),
        [0, -1] => Ok(H4S3AngularShell::Degrees144),
        [-2, 0] => Ok(H4S3AngularShell::Antipodal),
        other => Err(GeometricAttentionError::Invalid(format!(
            "H4 relative state has noncanonical signed S3 real coordinate {other:?}"
        ))),
    }
}

fn increment_count(count: &mut u32) -> Result<(), GeometricAttentionError> {
    *count = count
        .checked_add(1)
        .ok_or(GeometricAttentionError::ArithmeticOverflow)?;
    Ok(())
}

fn finalize_rows<K: Ord>(
    counts: BTreeMap<K, BTreeMap<GeometricAddress, u32>>,
    maximum_candidates: NonZeroU16,
) -> BTreeMap<K, AttentionRow> {
    counts
        .into_iter()
        .map(|(key, candidates)| {
            let mut candidates = candidates
                .into_iter()
                .map(|(next, count)| AttentionRowCandidate { next, count })
                .collect::<Vec<_>>();
            candidates.sort_by(|left, right| {
                (Reverse(left.count), &left.next).cmp(&(Reverse(right.count), &right.next))
            });
            candidates.truncate(usize::from(maximum_candidates.get()));
            (key, AttentionRow { candidates })
        })
        .collect()
}

fn spin_sector(address: &GeometricAddress) -> SpinSector {
    let hopf = address.spin.hopf.raw();
    let hopf_octant =
        u8::from(hopf[0] >= 0) | (u8::from(hopf[1] >= 0) << 1) | (u8::from(hopf[2] >= 0) << 2);
    let shifted = i64::from(address.spin.torsion.raw()) + PHASE_HALF_Q29;
    let bin = (shifted * i64::from(ATTENTION_TORSION_BINS)) / PHASE_MODULUS_Q29;
    SpinSector {
        hopf_octant,
        torsion_bin: bin.clamp(0, i64::from(ATTENTION_TORSION_BINS - 1)) as u8,
    }
}

fn adjacent_spin_sectors(center: SpinSector) -> [SpinSector; ATTENTION_ADJACENT_SPIN_ROWS] {
    let previous = if center.torsion_bin == 0 {
        ATTENTION_TORSION_BINS - 1
    } else {
        center.torsion_bin - 1
    };
    let next = (center.torsion_bin + 1) % ATTENTION_TORSION_BINS;
    [
        center,
        SpinSector {
            hopf_octant: center.hopf_octant,
            torsion_bin: previous,
        },
        SpinSector {
            hopf_octant: center.hopf_octant,
            torsion_bin: next,
        },
    ]
}

fn circular_signed_delta(from: i32, to: i32) -> i64 {
    let mut delta = i64::from(to) - i64::from(from);
    if delta >= PHASE_HALF_Q29 {
        delta -= PHASE_MODULUS_Q29;
    }
    if delta < -PHASE_HALF_Q29 {
        delta += PHASE_MODULUS_Q29;
    }
    delta
}

fn circular_delta_distance(left: i64, right: i64) -> u64 {
    let mut delta = left - right;
    if delta >= PHASE_HALF_Q29 {
        delta -= PHASE_MODULUS_Q29;
    }
    if delta < -PHASE_HALF_Q29 {
        delta += PHASE_MODULUS_Q29;
    }
    delta.unsigned_abs()
}

fn circular_add_delta(left: i64, right: i64) -> i64 {
    let mut sum = left + right;
    if sum >= PHASE_HALF_Q29 {
        sum -= PHASE_MODULUS_Q29;
    }
    if sum < -PHASE_HALF_Q29 {
        sum += PHASE_MODULUS_Q29;
    }
    sum
}

fn continuation_phase_energy(
    previous: Option<i32>,
    last: i32,
    next: i32,
    observed_delta_offset: i32,
) -> u64 {
    match previous {
        Some(previous) => circular_delta_distance(
            circular_add_delta(
                circular_signed_delta(previous, last),
                i64::from(observed_delta_offset),
            ),
            circular_signed_delta(last, next),
        ),
        None => circular_delta_distance(
            i64::from(observed_delta_offset),
            circular_signed_delta(last, next),
        ),
    }
}

fn continuation_spin_energy(
    previous: Option<[i32; 3]>,
    last: [i32; 3],
    next: [i32; 3],
) -> Result<u64, GeometricAttentionError> {
    last.into_iter()
        .zip(next)
        .enumerate()
        .try_fold(0u64, |total, (index, (last, next))| {
            let delta = match previous {
                Some(previous) => {
                    i64::from(next) - 2 * i64::from(last) + i64::from(previous[index])
                }
                None => i64::from(next) - i64::from(last),
            };
            total
                .checked_add(delta.unsigned_abs())
                .ok_or(GeometricAttentionError::ArithmeticOverflow)
        })
}

fn deterministic_permutation_offset(manifest_kappa: &str, population: usize) -> usize {
    if population <= 1 {
        return 0;
    }
    let digest = blake3::hash(manifest_kappa.as_bytes());
    let bytes = digest.as_bytes();
    let seed = u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]);
    let offset = (seed % population as u64) as usize;
    if offset == 0 {
        1
    } else {
        offset
    }
}

fn tie_break_stages(control: AttentionControl) -> Vec<AttentionTieBreakStage> {
    let mut stages = Vec::with_capacity(12);
    if control != AttentionControl::CountOnly {
        stages.extend([
            AttentionTieBreakStage::PhaseEnergy,
            AttentionTieBreakStage::TorsionEnergy,
            AttentionTieBreakStage::SpinEnergy,
            AttentionTieBreakStage::FactorEnergy,
        ]);
    }
    stages.extend([
        AttentionTieBreakStage::SourceBreadth,
        AttentionTieBreakStage::TotalSupport,
        AttentionTieBreakStage::OrderedSentenceSupport,
        AttentionTieBreakStage::LastTwoSupport,
        AttentionTieBreakStage::LastOneSupport,
        AttentionTieBreakStage::DivisorSupport,
        AttentionTieBreakStage::AdjacentSpinSupport,
        AttentionTieBreakStage::CanonicalAddress,
    ]);
    stages
}

fn compare_candidates(
    left: &AttentionCandidateTrace,
    right: &AttentionCandidateTrace,
    control: AttentionControl,
) -> std::cmp::Ordering {
    let support_key = |candidate: &AttentionCandidateTrace| {
        (
            Reverse(candidate.source_counts.source_breadth()),
            Reverse(candidate.source_counts.total()),
            Reverse(candidate.source_counts.ordered_sentence),
            Reverse(candidate.source_counts.last_two),
            Reverse(candidate.source_counts.last_one),
            Reverse(candidate.source_counts.divisor),
            Reverse(candidate.source_counts.adjacent_spin),
            candidate.next.clone(),
        )
    };
    if control == AttentionControl::CountOnly {
        support_key(left).cmp(&support_key(right))
    } else {
        (&left.ranking_energy, support_key(left)).cmp(&(&right.ranking_energy, support_key(right)))
    }
}
// END GEOMETRIC_ATTENTION_BOUNDED_LOOKUP

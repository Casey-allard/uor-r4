//! Bounded causal geometric attention over a compiled prime-route manifest.
//!
//! Compilation may evaluate the fixed zeta basis with floating point. The
//! query path is deliberately narrower: three direct rows, one divisor row,
//! three adjacent spin-sector rows, and integer/table-backed energy terms.
//! It accepts only accumulated causal state; there is no future-route input.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU16;

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
        let mut support = merged.into_iter().collect::<Vec<_>>();
        // This is explicit pre-geometric support admission. Consequently,
        // least-energy ranking below is only over the admitted support, never
        // a claim about the full untruncated row union.
        support.sort_by(|(left_next, left_counts), (right_next, right_counts)| {
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
        support.truncate(bounds.unique_candidates_after_ceiling);
        // Canonical order makes the geometry permutation independent of map
        // insertion order and count ordering.
        support.sort_by(|left, right| left.0.cmp(&right.0));

        let mut measured = Vec::with_capacity(support.len());
        for (next, counts) in &support {
            measured.push(self.measure_energy(state, next, *counts, intervention)?);
        }
        let permutation_offset =
            deterministic_permutation_offset(&self.manifest_kappa, measured.len());
        let canonical_next = support
            .iter()
            .map(|(next, _)| next.clone())
            .collect::<Vec<_>>();
        let mut candidates = Vec::with_capacity(support.len());
        for (index, ((next, counts), measured_energy)) in support
            .into_iter()
            .zip(measured.iter().copied())
            .enumerate()
        {
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
                source_counts: counts,
                measured_energy,
                ranking_energy,
                geometry_source_next,
            });
        }
        let tie_break_stages = tie_break_stages(control);
        candidates.sort_by(|left, right| compare_candidates(left, right, control));
        let selected = candidates.first().cloned();
        Ok(GeometricAttentionTrace {
            manifest_kappa: self.manifest_kappa.clone(),
            control,
            geometry_intervention: intervention,
            rows_read,
            candidate_entries_examined,
            candidate_entry_ceiling: bounds.candidate_entries_per_query,
            unique_candidates_before_ceiling,
            candidate_ceiling: bounds.unique_candidates_after_ceiling,
            support_admission:
                AttentionSupportAdmission::SourceBreadthThenTotalCountThenCanonicalAddress,
            geometry_evaluations: candidates.len(),
            tie_break_stages,
            candidates,
            selected,
        })
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

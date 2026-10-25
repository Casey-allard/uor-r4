//! Lawful integer pointer-copy head and serving-time forced-copy span oracle.
//!
//! This module implements the D32 discrete integer construction for
//! `r-lawful-pointer-forced-copy-oracle` (#2880):
//!
//! 1. **Geometric source admission without inner products or matrix contractions:**
//!    - [`SlotAddressKey`]: square-free semiprime slot key
//!      $K_{\text{slot}} = p_{\text{entity}} \cdot p_{\text{attribute}}$ in $\mathbb{Z}$,
//!      admitting a stored clause iff $K_{\text{slot}} \mid \pi_j$
//!      ($\gcd(K_{\text{slot}}, \pi_j) = K_{\text{slot}}$).
//!    - [`RouteAdmissionMode::SharedAtomGcd`]: prime-atom divisibility admission
//!      $\gcd(q, \pi_j) > 1$ weighted by integer bit-rank rarity [`ilog2_gcd_weight_q8`].
//!    - [`RouteAdmissionMode::OrderedNlet`]: exact ordered $n$-let equality over
//!      canonical folded token IDs via [`CanonicalFoldTable`].
//! 2. **Hard integer rank/margin selection without normalized-exponential mixtures:**
//!    - Replaces the retired dense mixture with an exact integer margin comparator
//!      between the base vocabulary margin table and the admitted source span's
//!      integer route margin plus gate margin.
//! 3. **Interventional serving-time forced-copy oracle ([`CopySpanOracle`]):**
//!    - `Off`: bit-for-bit identity with the unforced lawful pointer and base scorer.
//!    - `O1SpanLearnedGate`: forces source selection onto a supplied window span
//!      `[start, end)` with its canonical route score while keeping `learned_gate_margin`
//!      and the `total_copy_margin > base_top_margin` comparison (isolating source
//!      addressing alone).
//!    - `O2SpanForcedGate`: forces both the supplied window span `[start, end)` and
//!      an above-threshold gate margin (isolating gating from multi-token span emission).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{invalid, Result};

/// Binary/Euclidean greatest common divisor on `u128`.
pub const fn gcd_u128(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

/// Deterministic integer primality check for `u64` prime-registry atoms.
pub const fn is_prime_u64(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 || n == 3 {
        return true;
    }
    if n.is_multiple_of(2) || n.is_multiple_of(3) {
        return false;
    }
    let mut d = 5u64;
    while d.saturating_mul(d) <= n {
        if n.is_multiple_of(d) || n.is_multiple_of(d + 2) {
            return false;
        }
        d += 6;
    }
    true
}

/// Integer bit-rank rarity weight in Q8 fixed-point units for a shared divisor.
///
/// Returns `0` when `shared_gcd <= 1`, and `floor(log2(shared_gcd)) * 256`
/// when `shared_gcd > 1`. Larger (rarer) prime factors strictly dominate smaller
/// prime factors without floating-point arithmetic.
pub const fn ilog2_gcd_weight_q8(shared_gcd: u128) -> i64 {
    if shared_gcd <= 1 {
        0
    } else {
        ((127 - shared_gcd.leading_zeros()) as i64) << 8
    }
}

/// Exact slot-address key $K_{\text{slot}} = p_{\text{entity}} \cdot p_{\text{attribute}}$.
///
/// By unique factorization in $\mathbb{Z}$, the square-free semiprime of two
/// distinct primes uniquely identifies the unordered slot pair
/// `{entity_prime, attribute_prime}`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SlotAddressKey {
    entity_prime: u64,
    attribute_prime: u64,
    semiprime: u128,
}

impl SlotAddressKey {
    /// Construct a slot-address key from distinct entity and attribute primes.
    pub fn from_entity_attribute_primes(entity_prime: u64, attribute_prime: u64) -> Result<Self> {
        if !is_prime_u64(entity_prime) {
            return Err(invalid(format!(
                "entity atom {entity_prime} is not a prime number"
            )));
        }
        if !is_prime_u64(attribute_prime) {
            return Err(invalid(format!(
                "attribute atom {attribute_prime} is not a prime number"
            )));
        }
        if entity_prime == attribute_prime {
            return Err(invalid(
                "slot-address key requires distinct entity and attribute primes",
            ));
        }
        let semiprime = (entity_prime as u128)
            .checked_mul(attribute_prime as u128)
            .ok_or_else(|| invalid("slot-address semiprime overflowed u128"))?;
        Ok(Self {
            entity_prime,
            attribute_prime,
            semiprime,
        })
    }

    pub const fn entity_prime(&self) -> u64 {
        self.entity_prime
    }

    pub const fn attribute_prime(&self) -> u64 {
        self.attribute_prime
    }

    pub const fn semiprime(&self) -> u128 {
        self.semiprime
    }

    /// True iff `candidate_product` is a nonzero multiple of `self.semiprime`,
    /// equivalently $\gcd(K_{\text{slot}}, \pi_j) = K_{\text{slot}}$.
    pub const fn divides(&self, candidate_product: u128) -> bool {
        candidate_product != 0 && candidate_product.is_multiple_of(self.semiprime)
    }
}

/// Fixed D11-legal canonical token-id fold table (case + at most one leading space).
///
/// Used exclusively to decide key admission matches; never modifies the emitted
/// token ID copied from the source window.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalFoldTable {
    canonical: Vec<u32>,
}

fn canonical_fold_key(bytes: &[u8]) -> Option<Vec<u8>> {
    let stripped = bytes.strip_prefix(b" ").unwrap_or(bytes);
    (!stripped.is_empty() && stripped.iter().all(u8::is_ascii_alphabetic))
        .then(|| stripped.to_ascii_lowercase())
}

impl CanonicalFoldTable {
    /// Build a canonical fold table from decoded vocabulary byte pieces.
    pub fn from_pieces(pieces: &[Vec<u8>]) -> Result<Self> {
        let vocab_len = u32::try_from(pieces.len())
            .map_err(|_| invalid("canonical fold vocabulary length must fit in u32"))?;
        let mut least: BTreeMap<Vec<u8>, u32> = BTreeMap::new();
        let mut canonical = Vec::with_capacity(pieces.len());
        for (id, piece) in (0..vocab_len).zip(pieces) {
            let mapped = match canonical_fold_key(piece) {
                Some(key) => *least.entry(key).or_insert(id),
                None => id,
            };
            canonical.push(mapped);
        }
        Ok(Self { canonical })
    }

    /// Identity fold table where every token ID maps only to itself.
    pub fn identity(vocab_size: usize) -> Result<Self> {
        let vocab_len = u32::try_from(vocab_size)
            .map_err(|_| invalid("canonical fold vocabulary length must fit in u32"))?;
        Ok(Self {
            canonical: (0..vocab_len).collect(),
        })
    }

    pub fn len(&self) -> usize {
        self.canonical.len()
    }

    pub fn is_empty(&self) -> bool {
        self.canonical.is_empty()
    }

    pub fn as_slice(&self) -> &[u32] {
        &self.canonical
    }

    /// Map `token` to its canonical equivalence-class representative.
    pub fn fold(&self, token: u32) -> Result<u32> {
        self.canonical
            .get(token as usize)
            .copied()
            .ok_or_else(|| invalid(format!("token id {token} is out of range for fold table")))
    }
}

/// Candidate value span `[start, end)` in the earlier context window.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpanCandidate {
    /// Half-open token interval `[start, end)` in `window_tokens`.
    pub span: (usize, usize),
    /// Product of prime atoms associated with this candidate's clause.
    pub slot_product: u128,
    /// Preceding context token IDs before `span.0`, used for ordered $n$-let admission.
    pub preceding_tokens: Vec<u32>,
    /// Learned integer rank/margin offset for this candidate.
    pub learned_rank_bonus: i64,
}

impl SourceSpanCandidate {
    pub fn validate(&self, window_len: usize) -> Result<()> {
        let (start, end) = self.span;
        if start >= end {
            return Err(invalid(format!(
                "candidate span [{start}, {end}) must be non-empty with start < end"
            )));
        }
        if end > window_len {
            return Err(invalid(format!(
                "candidate span [{start}, {end}) exceeds window length {window_len}"
            )));
        }
        Ok(())
    }
}

/// Geometric admission law for selecting a copy source span.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteAdmissionMode {
    /// Exact slot-address semiprime divisibility: $\gcd(K_{\text{slot}}, \pi_j) = K_{\text{slot}}$.
    SlotAddressSemiprime,
    /// Shared prime-atom divisibility: $\gcd(q, \pi_j) > 1$, ranked by `ilog2_gcd_weight_q8`.
    SharedAtomGcd,
    /// Exact ordered $n$-let equality over canonical folded token IDs.
    OrderedNlet { width: usize },
}

/// Serving-time interventional forced-copy oracle condition (`N` / `O1` / `O2`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CopySpanOracle {
    /// Normal (`N`): no span override; runs the head's own admission and gate margins.
    #[default]
    Off,
    /// Condition `O1`: overrides source admission to `[start, end)` with its canonical
    /// route score while keeping `learned_gate_margin` and the `total_copy_margin > base_top_margin`
    /// comparison. Isolates source addressing alone.
    O1SpanLearnedGate { start: usize, end: usize },
    /// Condition `O2`: overrides source admission to `[start, end)` and forces the
    /// copy gate margin to `forced_gate_margin` (above threshold).
    O2SpanForcedGate {
        start: usize,
        end: usize,
        forced_gate_margin: i64,
    },
}

impl CopySpanOracle {
    pub fn validate(&self, window_len: usize) -> Result<()> {
        match *self {
            Self::Off => Ok(()),
            Self::O1SpanLearnedGate { start, end } | Self::O2SpanForcedGate { start, end, .. } => {
                if start >= end {
                    return Err(invalid(format!(
                        "copy-span oracle [{start}, {end}) must have start < end"
                    )));
                }
                if end > window_len {
                    return Err(invalid(format!(
                        "copy-span oracle [{start}, {end}) exceeds window length {window_len}"
                    )));
                }
                Ok(())
            }
        }
    }
}

/// Trace of one step's integer decision between the base scorer and the pointer head.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PointerDecision {
    /// Final emitted token ID.
    pub emitted_token: u32,
    /// Best token ID from the base margin table alone.
    pub base_top_token: u32,
    /// Margin of `base_top_token` in `base_margins`.
    pub base_top_margin: i64,
    /// Whether the copy branch won and emitted a token from `selected_span`.
    pub copied: bool,
    /// Selected source span `[start, end)` if a span was admitted or forced.
    pub selected_span: Option<(usize, usize)>,
    /// Token ID at `window_tokens[start + step_offset]` when a span was selected.
    pub candidate_copy_token: Option<u32>,
    /// Integer route score of the selected span.
    pub route_score: Option<i64>,
    /// Effective integer gate margin (`learned_gate_margin` or `forced_gate_margin`).
    pub effective_gate_margin: i64,
}

/// Lawful integer pointer-copy head with serving-time forced-copy oracle support.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LawfulPointerHead {
    admission_mode: RouteAdmissionMode,
    fold_table: CanonicalFoldTable,
    gate_threshold_margin: i64,
    oracle: CopySpanOracle,
}

impl LawfulPointerHead {
    pub fn new(
        admission_mode: RouteAdmissionMode,
        fold_table: CanonicalFoldTable,
        gate_threshold_margin: i64,
    ) -> Self {
        Self {
            admission_mode,
            fold_table,
            gate_threshold_margin,
            oracle: CopySpanOracle::Off,
        }
    }

    pub const fn admission_mode(&self) -> RouteAdmissionMode {
        self.admission_mode
    }

    pub const fn gate_threshold_margin(&self) -> i64 {
        self.gate_threshold_margin
    }

    pub fn fold_table(&self) -> &CanonicalFoldTable {
        &self.fold_table
    }

    pub const fn copy_span_oracle(&self) -> CopySpanOracle {
        self.oracle
    }

    /// Set or clear the serving-time copy-span oracle (`N`, `O1`, or `O2`).
    ///
    /// Validates `oracle` against `window_len` immediately so invalid spans fail
    /// at the setter without silent fallback.
    pub fn set_copy_span_oracle(
        &mut self,
        oracle: CopySpanOracle,
        window_len: usize,
    ) -> Result<()> {
        oracle.validate(window_len)?;
        self.oracle = oracle;
        Ok(())
    }

    /// Clear the serving-time copy-span oracle back to `CopySpanOracle::Off`.
    pub fn clear_copy_span_oracle(&mut self) {
        self.oracle = CopySpanOracle::Off;
    }

    fn admit_candidate(
        &self,
        candidate: &SourceSpanCandidate,
        query_slot: Option<SlotAddressKey>,
        query_product: u128,
        query_preceding: &[u32],
    ) -> Result<Option<i64>> {
        match self.admission_mode {
            RouteAdmissionMode::SlotAddressSemiprime => {
                let Some(slot) = query_slot else {
                    return Ok(None);
                };
                if slot.divides(candidate.slot_product) {
                    let weight = ilog2_gcd_weight_q8(slot.semiprime());
                    Ok(Some(weight.saturating_add(candidate.learned_rank_bonus)))
                } else {
                    Ok(None)
                }
            }
            RouteAdmissionMode::SharedAtomGcd => {
                let shared = gcd_u128(query_product, candidate.slot_product);
                if shared > 1 {
                    let weight = ilog2_gcd_weight_q8(shared);
                    Ok(Some(weight.saturating_add(candidate.learned_rank_bonus)))
                } else {
                    Ok(None)
                }
            }
            RouteAdmissionMode::OrderedNlet { width } => {
                if width == 0
                    || candidate.preceding_tokens.len() < width
                    || query_preceding.len() < width
                {
                    return Ok(None);
                }
                let cand_tail =
                    &candidate.preceding_tokens[candidate.preceding_tokens.len() - width..];
                let query_tail = &query_preceding[query_preceding.len() - width..];
                for (&c_tok, &q_tok) in cand_tail.iter().zip(query_tail.iter()) {
                    if self.fold_table.fold(c_tok)? != self.fold_table.fold(q_tok)? {
                        return Ok(None);
                    }
                }
                let weight = (width as i64) << 8;
                Ok(Some(weight.saturating_add(candidate.learned_rank_bonus)))
            }
        }
    }

    /// Canonical integer route score for an oracle-forced span `[start, end)`.
    ///
    /// Uses the query's full admitted route weight under `self.admission_mode` plus
    /// the candidate's `learned_rank_bonus` when `[start, end)` appears in `candidates`
    /// (or `0` when absent), so `O1` and `N` share the exact same margin law when the
    /// same span is selected.
    fn oracle_route_score(
        &self,
        start: usize,
        end: usize,
        query_slot: Option<SlotAddressKey>,
        query_product: u128,
        candidates: &[SourceSpanCandidate],
    ) -> i64 {
        let rank_bonus = candidates
            .iter()
            .rev()
            .find(|c| c.span == (start, end))
            .map(|c| c.learned_rank_bonus)
            .unwrap_or(0);
        let weight = match self.admission_mode {
            RouteAdmissionMode::SlotAddressSemiprime => query_slot
                .map(|slot| ilog2_gcd_weight_q8(slot.semiprime()))
                .unwrap_or_else(|| ilog2_gcd_weight_q8(query_product)),
            RouteAdmissionMode::SharedAtomGcd => ilog2_gcd_weight_q8(query_product),
            RouteAdmissionMode::OrderedNlet { width } => (width as i64) << 8,
        };
        weight.saturating_add(rank_bonus)
    }

    /// Select the next token at `step_offset` within an admitted or oracle-forced value span.
    #[allow(clippy::too_many_arguments)]
    pub fn select_step(
        &self,
        window_tokens: &[u32],
        base_margins: &[i64],
        learned_gate_margin: i64,
        step_offset: usize,
        query_slot: Option<SlotAddressKey>,
        query_product: u128,
        query_preceding: &[u32],
        candidates: &[SourceSpanCandidate],
    ) -> Result<PointerDecision> {
        if base_margins.is_empty() {
            return Err(invalid("base_margins must be non-empty"));
        }
        self.oracle.validate(window_tokens.len())?;
        for candidate in candidates {
            candidate.validate(window_tokens.len())?;
        }
        for &tok in window_tokens {
            if (tok as usize) >= self.fold_table.len() {
                return Err(invalid(format!(
                    "window token {tok} exceeds fold table vocabulary size {}",
                    self.fold_table.len()
                )));
            }
        }

        // Deterministic argmax on integer base margins (lowest token ID breaks ties).
        let mut base_top_token = 0u32;
        let mut base_top_margin = base_margins[0];
        for (idx, &margin) in base_margins.iter().enumerate().skip(1) {
            if margin > base_top_margin {
                base_top_margin = margin;
                base_top_token = idx as u32;
            }
        }

        let (selected, effective_gate_margin) = match self.oracle {
            CopySpanOracle::Off => {
                // Evaluate admitted candidates; highest route_score wins, breaking ties
                // by latest span start (`span.0`).
                let mut best: Option<((usize, usize), i64)> = None;
                for candidate in candidates {
                    if let Some(score) =
                        self.admit_candidate(candidate, query_slot, query_product, query_preceding)?
                    {
                        let replace = match best {
                            None => true,
                            Some((best_span, best_score)) => {
                                score > best_score
                                    || (score == best_score && candidate.span.0 > best_span.0)
                            }
                        };
                        if replace {
                            best = Some((candidate.span, score));
                        }
                    }
                }
                (best, learned_gate_margin)
            }
            CopySpanOracle::O1SpanLearnedGate { start, end } => {
                let score =
                    self.oracle_route_score(start, end, query_slot, query_product, candidates);
                (Some(((start, end), score)), learned_gate_margin)
            }
            CopySpanOracle::O2SpanForcedGate {
                start,
                end,
                forced_gate_margin,
            } => {
                let score =
                    self.oracle_route_score(start, end, query_slot, query_product, candidates);
                (Some(((start, end), score)), forced_gate_margin)
            }
        };

        let Some((span, route_score)) = selected else {
            return Ok(PointerDecision {
                emitted_token: base_top_token,
                base_top_token,
                base_top_margin,
                copied: false,
                selected_span: None,
                candidate_copy_token: None,
                route_score: None,
                effective_gate_margin,
            });
        };

        let span_len = span.1 - span.0;
        if step_offset >= span_len {
            return Ok(PointerDecision {
                emitted_token: base_top_token,
                base_top_token,
                base_top_margin,
                copied: false,
                selected_span: Some(span),
                candidate_copy_token: None,
                route_score: Some(route_score),
                effective_gate_margin,
            });
        }

        let copy_token = window_tokens[span.0 + step_offset];
        let total_copy_margin = route_score.saturating_add(effective_gate_margin);
        let gate_open = effective_gate_margin >= self.gate_threshold_margin
            && total_copy_margin > base_top_margin;

        Ok(PointerDecision {
            emitted_token: if gate_open {
                copy_token
            } else {
                base_top_token
            },
            base_top_token,
            base_top_margin,
            copied: gate_open,
            selected_span: Some(span),
            candidate_copy_token: Some(copy_token),
            route_score: Some(route_score),
            effective_gate_margin,
        })
    }
}

fn has_alphanumeric_boundary(flat_bytes: &[u8], pos: usize, len: usize) -> bool {
    let first_alnum = flat_bytes[pos].is_ascii_alphanumeric();
    let last_alnum = flat_bytes[pos + len - 1].is_ascii_alphanumeric();
    let left_ok = !first_alnum || pos == 0 || !flat_bytes[pos - 1].is_ascii_alphanumeric();
    let right_ok = !last_alnum
        || pos + len == flat_bytes.len()
        || !flat_bytes[pos + len].is_ascii_alphanumeric();
    left_ok && right_ok
}

/// Locate the last occurrence of `expected_value` in the decoded bytes of `window_tokens`,
/// returning the minimal token interval `[start, end)` covering that occurrence.
///
/// Enforces ASCII alphanumeric word boundaries so a short value such as `"4"` never
/// falsely matches inside `"47"` or `"14"`, while still matching values that cross
/// token boundaries (for example, `"47"` tokenized as `[" 4", "7"]`).
pub fn locate_last_value_span(
    window_tokens: &[u32],
    pieces: &[Vec<u8>],
    expected_value: &[u8],
) -> Result<Option<(usize, usize)>> {
    if expected_value.is_empty() || window_tokens.is_empty() {
        return Ok(None);
    }
    let mut flat_bytes = Vec::new();
    let mut token_ranges = Vec::with_capacity(window_tokens.len());
    for &tok in window_tokens {
        let piece = pieces
            .get(tok as usize)
            .ok_or_else(|| invalid(format!("window token {tok} out of range for pieces")))?;
        let start_byte = flat_bytes.len();
        flat_bytes.extend_from_slice(piece);
        let end_byte = flat_bytes.len();
        token_ranges.push((start_byte, end_byte));
    }
    if flat_bytes.len() < expected_value.len() {
        return Ok(None);
    }

    let match_start = (0..=flat_bytes.len() - expected_value.len())
        .rev()
        .find(|&pos| {
            &flat_bytes[pos..pos + expected_value.len()] == expected_value
                && has_alphanumeric_boundary(&flat_bytes, pos, expected_value.len())
        });

    let Some(byte_start) = match_start else {
        return Ok(None);
    };
    let byte_end = byte_start + expected_value.len();

    let first_tok = token_ranges
        .iter()
        .position(|&(_, t_end)| t_end > byte_start);
    let last_tok = token_ranges
        .iter()
        .rposition(|&(t_start, _)| t_start < byte_end);

    match (first_tok, last_tok) {
        (Some(s), Some(e)) if s <= e => Ok(Some((s, e + 1))),
        _ => Ok(None),
    }
}

/// Mutually exclusive, collectively exhaustive four-branch diagnostic outcome
/// for the forced-copy span oracle (`N` vs `O1` vs `O2`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OracleDiagnosticBranch {
    /// Branch A: `C_O1 - C_N >= 32` and `P_O1 - P_N >= 16`.
    BranchAAddressingBottleneck,
    /// Branch B: Branch A not met, and `C_O2 - C_N >= 32` and `P_O2 - P_N >= 16`.
    BranchBGateMarginBottleneck,
    /// Branch C: Branch A not met, and `C_O2 - C_N < 32` and `P_O2 - P_N < 16`.
    BranchCEmissionBottleneck,
    /// Branch D: Branch A and Branch B not met, and `(C_O2 - C_N >= 32) != (P_O2 - P_N >= 16)`.
    BranchDDiscordantInconclusive,
}

/// Pre-registered instrument/control stop failures that halt the oracle evaluation
/// before any mechanism branch is called.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OracleStopFailure {
    /// Condition `N` (`CopySpanOracle::Off`) differed from the bound checkpoint's recorded baseline.
    BaselineIdentityMismatch {
        token_mismatches: usize,
        verdict_mismatches: usize,
    },
    /// Condition `W` (wrong-span distractor control) scored `P_W > 0` or `C_W > C_N`.
    WrongSpanControlLeak { c_w: usize, c_n: usize, p_w: usize },
    /// `locate_last_value_span` failed to locate a target or distractor span in `window_tokens`.
    MissingSpanCoverage {
        missing_target_spans: usize,
        missing_distractor_spans: usize,
    },
}

/// Exact integer square root $\lfloor \sqrt{n} \rfloor$ on `u128`.
pub const fn isqrt_u128(n: u128) -> u128 {
    if n < 2 {
        return n;
    }
    let shift = ((128 - n.leading_zeros()) + 1) & !1;
    let mut result = 0u128;
    let mut bit = 1u128 << (shift - 2);
    let mut rem = n;
    while bit != 0 {
        let cand = result + bit;
        result >>= 1;
        if rem >= cand {
            rem -= cand;
            result += bit;
        }
        bit >>= 2;
    }
    result
}

/// Classify `(c_n, c_o1, c_o2, p_n, p_o1, p_o2)` into the four-branch diagnostic partition.
pub const fn classify_oracle_outcome(
    c_n: usize,
    c_o1: usize,
    c_o2: usize,
    p_n: usize,
    p_o1: usize,
    p_o2: usize,
) -> OracleDiagnosticBranch {
    let o1_rows_ok = c_o1 >= c_n.saturating_add(32);
    let o1_pairs_ok = p_o1 >= p_n.saturating_add(16);
    if o1_rows_ok && o1_pairs_ok {
        return OracleDiagnosticBranch::BranchAAddressingBottleneck;
    }
    let o2_rows_ok = c_o2 >= c_n.saturating_add(32);
    let o2_pairs_ok = p_o2 >= p_n.saturating_add(16);
    if o2_rows_ok && o2_pairs_ok {
        OracleDiagnosticBranch::BranchBGateMarginBottleneck
    } else if !o2_rows_ok && !o2_pairs_ok {
        OracleDiagnosticBranch::BranchCEmissionBottleneck
    } else {
        OracleDiagnosticBranch::BranchDDiscordantInconclusive
    }
}

/// Compute the paired cluster jackknife standard error $\lfloor 1000 \cdot \mathrm{SE}_{\Delta} \rfloor$
/// (in milli-units) from per-block paired differences `block_deltas` using exact integer arithmetic:
///
/// $$\widehat{\mathrm{Var}}(\Delta) = \frac{B}{B - 1} \sum_{b=0}^{B-1} \left(d_b - \frac{\Delta}{B}\right)^2 = \frac{\sum_{b=0}^{B-1} (B\,d_b - \Delta)^2}{B(B - 1)}$$
pub fn compute_paired_cluster_se_milli(block_deltas: &[i64]) -> Result<u64> {
    if block_deltas.len() < 2 {
        return Err(invalid(
            "paired cluster SE requires at least 2 cluster blocks",
        ));
    }
    let b = block_deltas.len() as i128;
    let total: i128 = block_deltas.iter().map(|&d| d as i128).sum();
    let mut sum_sq: u128 = 0;
    for &d in block_deltas {
        let diff = b
            .checked_mul(d as i128)
            .and_then(|bd| bd.checked_sub(total))
            .ok_or_else(|| invalid("paired cluster block delta overflowed i128"))?;
        let sq = diff
            .unsigned_abs()
            .checked_mul(diff.unsigned_abs())
            .ok_or_else(|| invalid("paired cluster squared deviation overflowed u128"))?;
        sum_sq = sum_sq
            .checked_add(sq)
            .ok_or_else(|| invalid("paired cluster sum of squares overflowed u128"))?;
    }
    let denom = (b as u128) * ((b - 1) as u128);
    let scaled_var = sum_sq
        .checked_mul(1_000_000)
        .ok_or_else(|| invalid("paired cluster scaled variance overflowed u128"))?
        / denom;
    let se_milli = isqrt_u128(scaled_var);
    u64::try_from(se_milli).map_err(|_| invalid("paired cluster SE overflowed u64"))
}

/// Input specification for one panel row in the batch forced-copy oracle evaluator.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleRowInput {
    pub row_id: String,
    pub pair_id: String,
    pub block_index: usize,
    pub window_tokens: Vec<u32>,
    pub query_slot: Option<SlotAddressKey>,
    pub query_product: u128,
    pub query_preceding: Vec<u32>,
    pub candidates: Vec<SourceSpanCandidate>,
    pub expected_value_bytes: Vec<u8>,
    pub distractor_value_bytes: Vec<u8>,
    pub accepted_reply_tokens: Vec<Vec<u32>>,
    pub step_base_margins: Vec<Vec<i64>>,
    pub step_learned_gate_margins: Vec<i64>,
    pub recorded_baseline_tokens: Vec<u32>,
    pub forced_gate_margin: i64,
}

/// Summary of one oracle condition (`N`, `O1`, `O2`, or `W`) across all panel rows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleConditionSummary {
    pub complete_rows: usize,
    pub pairs_both_complete_distinct: usize,
    pub block_row_counts: Vec<usize>,
    pub block_pair_counts: Vec<usize>,
    pub emitted_by_row: Vec<Vec<u32>>,
}

/// Complete diagnostic report from `evaluate_forced_copy_oracle_batch`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleBatchReport {
    pub rows: usize,
    pub pairs: usize,
    pub blocks: usize,
    pub target_spans: Vec<(usize, usize)>,
    pub distractor_spans: Vec<(usize, usize)>,
    pub n: OracleConditionSummary,
    pub o1: OracleConditionSummary,
    pub o2: OracleConditionSummary,
    pub w: OracleConditionSummary,
    pub se_delta_o1_n_rows_milli: u64,
    pub se_delta_o1_n_pairs_milli: u64,
    pub se_delta_o2_n_rows_milli: u64,
    pub se_delta_o2_n_pairs_milli: u64,
    pub branch: OracleDiagnosticBranch,
}

fn grade_condition_rows_and_pairs(
    rows: &[OracleRowInput],
    emitted_by_row: Vec<Vec<u32>>,
    num_blocks: usize,
) -> Result<OracleConditionSummary> {
    let mut complete_rows = 0usize;
    let mut block_row_counts = vec![0usize; num_blocks];
    let mut pair_groups: BTreeMap<String, Vec<(usize, bool, Vec<u32>)>> = BTreeMap::new();

    for (row, emitted) in rows.iter().zip(emitted_by_row.iter()) {
        if row.block_index >= num_blocks {
            return Err(invalid(format!(
                "row {} block_index {} >= num_blocks {num_blocks}",
                row.row_id, row.block_index
            )));
        }
        let accepted = row.accepted_reply_tokens.iter().any(|acc| acc == emitted);
        if accepted {
            complete_rows += 1;
            block_row_counts[row.block_index] += 1;
        }
        pair_groups.entry(row.pair_id.clone()).or_default().push((
            row.block_index,
            accepted,
            emitted.clone(),
        ));
    }

    let mut pairs_both_complete_distinct = 0usize;
    let mut block_pair_counts = vec![0usize; num_blocks];
    for (pair_id, arms) in &pair_groups {
        if arms.len() != 2 {
            return Err(invalid(format!(
                "pair_id {pair_id} has {} rows (expected 2)",
                arms.len()
            )));
        }
        if arms[0].0 != arms[1].0 {
            return Err(invalid(format!(
                "pair_id {pair_id} spans multiple blocks ({} vs {})",
                arms[0].0, arms[1].0
            )));
        }
        if arms[0].1 && arms[1].1 && arms[0].2 != arms[1].2 {
            pairs_both_complete_distinct += 1;
            block_pair_counts[arms[0].0] += 1;
        }
    }

    Ok(OracleConditionSummary {
        complete_rows,
        pairs_both_complete_distinct,
        block_row_counts,
        block_pair_counts,
        emitted_by_row,
    })
}

/// Evaluate all four pre-registered forced-copy oracle conditions (`N`, `O1`, `O2`, `W`)
/// across `rows`, enforcing the pre-registered span-coverage, baseline-identity (`N`),
/// and wrong-span (`W`) stop rules before classifying into `Branches A, B, C, D`.
pub fn evaluate_forced_copy_oracle_batch(
    head: &LawfulPointerHead,
    pieces: &[Vec<u8>],
    rows: &[OracleRowInput],
    num_blocks: usize,
) -> Result<std::result::Result<OracleBatchReport, OracleStopFailure>> {
    if rows.is_empty() {
        return Err(invalid("oracle evaluation rows must be non-empty"));
    }
    if num_blocks < 2 {
        return Err(invalid("oracle evaluation requires num_blocks >= 2"));
    }

    let mut target_spans = Vec::with_capacity(rows.len());
    let mut distractor_spans = Vec::with_capacity(rows.len());
    let mut missing_target_spans = 0usize;
    let mut missing_distractor_spans = 0usize;

    for row in rows {
        if row.step_base_margins.len() != row.step_learned_gate_margins.len()
            || row.step_base_margins.len() != row.recorded_baseline_tokens.len()
        {
            return Err(invalid(format!(
                "row {} step count mismatch between base_margins, learned_gate_margins, and recorded_baseline_tokens",
                row.row_id
            )));
        }
        match locate_last_value_span(&row.window_tokens, pieces, &row.expected_value_bytes)? {
            Some(span) => target_spans.push(span),
            None => missing_target_spans += 1,
        }
        match locate_last_value_span(&row.window_tokens, pieces, &row.distractor_value_bytes)? {
            Some(span) => distractor_spans.push(span),
            None => missing_distractor_spans += 1,
        }
    }

    if missing_target_spans > 0 || missing_distractor_spans > 0 {
        return Ok(Err(OracleStopFailure::MissingSpanCoverage {
            missing_target_spans,
            missing_distractor_spans,
        }));
    }

    let run_condition = |mode: u8| -> Result<Vec<Vec<u32>>> {
        let mut all_emitted = Vec::with_capacity(rows.len());
        let mut h = head.clone();
        for (idx, row) in rows.iter().enumerate() {
            let oracle = match mode {
                0 => CopySpanOracle::Off,
                1 => CopySpanOracle::O1SpanLearnedGate {
                    start: target_spans[idx].0,
                    end: target_spans[idx].1,
                },
                2 => CopySpanOracle::O2SpanForcedGate {
                    start: target_spans[idx].0,
                    end: target_spans[idx].1,
                    forced_gate_margin: row.forced_gate_margin,
                },
                _ => CopySpanOracle::O2SpanForcedGate {
                    start: distractor_spans[idx].0,
                    end: distractor_spans[idx].1,
                    forced_gate_margin: row.forced_gate_margin,
                },
            };
            h.set_copy_span_oracle(oracle, row.window_tokens.len())?;
            let mut row_tokens = Vec::with_capacity(row.step_base_margins.len());
            for (step_offset, (base_margins, &learned_gate)) in row
                .step_base_margins
                .iter()
                .zip(&row.step_learned_gate_margins)
                .enumerate()
            {
                let dec = h.select_step(
                    &row.window_tokens,
                    base_margins,
                    learned_gate,
                    step_offset,
                    row.query_slot,
                    row.query_product,
                    &row.query_preceding,
                    &row.candidates,
                )?;
                row_tokens.push(dec.emitted_token);
            }
            all_emitted.push(row_tokens);
        }
        Ok(all_emitted)
    };

    let emitted_n = run_condition(0)?;
    let mut token_mismatches = 0usize;
    let mut verdict_mismatches = 0usize;
    for (row, n_tokens) in rows.iter().zip(&emitted_n) {
        if n_tokens != &row.recorded_baseline_tokens {
            token_mismatches += 1;
        }
        let n_ok = row.accepted_reply_tokens.iter().any(|acc| acc == n_tokens);
        let rec_ok = row
            .accepted_reply_tokens
            .iter()
            .any(|acc| acc == &row.recorded_baseline_tokens);
        if n_ok != rec_ok {
            verdict_mismatches += 1;
        }
    }
    if token_mismatches > 0 || verdict_mismatches > 0 {
        return Ok(Err(OracleStopFailure::BaselineIdentityMismatch {
            token_mismatches,
            verdict_mismatches,
        }));
    }

    let n = grade_condition_rows_and_pairs(rows, emitted_n, num_blocks)?;
    let o1 = grade_condition_rows_and_pairs(rows, run_condition(1)?, num_blocks)?;
    let o2 = grade_condition_rows_and_pairs(rows, run_condition(2)?, num_blocks)?;
    let w = grade_condition_rows_and_pairs(rows, run_condition(3)?, num_blocks)?;

    if w.pairs_both_complete_distinct > 0 || w.complete_rows > n.complete_rows {
        return Ok(Err(OracleStopFailure::WrongSpanControlLeak {
            c_w: w.complete_rows,
            c_n: n.complete_rows,
            p_w: w.pairs_both_complete_distinct,
        }));
    }

    let delta_vec = |a: &[usize], b: &[usize]| -> Vec<i64> {
        a.iter()
            .zip(b)
            .map(|(&x, &y)| (x as i64) - (y as i64))
            .collect()
    };

    let se_delta_o1_n_rows_milli =
        compute_paired_cluster_se_milli(&delta_vec(&o1.block_row_counts, &n.block_row_counts))?;
    let se_delta_o1_n_pairs_milli =
        compute_paired_cluster_se_milli(&delta_vec(&o1.block_pair_counts, &n.block_pair_counts))?;
    let se_delta_o2_n_rows_milli =
        compute_paired_cluster_se_milli(&delta_vec(&o2.block_row_counts, &n.block_row_counts))?;
    let se_delta_o2_n_pairs_milli =
        compute_paired_cluster_se_milli(&delta_vec(&o2.block_pair_counts, &n.block_pair_counts))?;

    let branch = classify_oracle_outcome(
        n.complete_rows,
        o1.complete_rows,
        o2.complete_rows,
        n.pairs_both_complete_distinct,
        o1.pairs_both_complete_distinct,
        o2.pairs_both_complete_distinct,
    );

    Ok(Ok(OracleBatchReport {
        rows: rows.len(),
        pairs: rows.len() / 2,
        blocks: num_blocks,
        target_spans,
        distractor_spans,
        n,
        o1,
        o2,
        w,
        se_delta_o1_n_rows_milli,
        se_delta_o1_n_pairs_milli,
        se_delta_o2_n_rows_milli,
        se_delta_o2_n_pairs_milli,
        branch,
    }))
}

#[cfg(feature = "sieve-cuda")]
pub mod device {
    //! Ahead-of-time `nvcc` CUDA launcher for `LawfulPointerHead` (`lawful_pointer.ptx`).

    use std::sync::Arc;
    use std::time::Instant;

    use cudarc::driver::{CudaContext, CudaFunction, CudaStream, LaunchConfig, Ptx, PushKernelArg};

    use super::{
        CopySpanOracle, LawfulPointerHead, PointerDecision, RouteAdmissionMode, SlotAddressKey,
        SourceSpanCandidate,
    };
    use crate::{invalid, sha256_bytes, Result};

    /// Ahead-of-time PTX compiled by `build.rs` from `src/lawful_pointer.cu`.
    pub const PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/lawful_pointer.ptx"));

    /// SHA-256 digest of the compiled `lawful_pointer.ptx` module.
    pub fn ptx_sha256() -> String {
        sha256_bytes(PTX.as_bytes())
    }

    fn cuda(error: impl std::fmt::Display) -> crate::TrainingError {
        invalid(format!("CUDA: {error}"))
    }

    fn len_u32(len: usize) -> Result<u32> {
        u32::try_from(len).map_err(|_| invalid("device array too large for u32"))
    }

    fn nonempty<T: Copy + Default>(values: &[T]) -> Vec<T> {
        if values.is_empty() {
            vec![T::default()]
        } else {
            values.to_vec()
        }
    }

    const fn split_u128(v: u128) -> (u64, u64) {
        (v as u64, (v >> 64) as u64)
    }

    /// One step-selection item in a GPU batch launch of `lawful_pointer_select_batch`.
    #[derive(Clone, Debug)]
    pub struct PointerStepItem {
        pub window_tokens: Vec<u32>,
        pub base_margins: Vec<i64>,
        pub learned_gate_margin: i64,
        pub step_offset: usize,
        pub query_slot: Option<SlotAddressKey>,
        pub query_product: u128,
        pub query_preceding: Vec<u32>,
        pub candidates: Vec<SourceSpanCandidate>,
        pub oracle: CopySpanOracle,
    }

    /// CUDA device handle for `lawful_pointer_select_batch`.
    pub struct PointerDevice {
        stream: Arc<CudaStream>,
        kernel: CudaFunction,
    }

    impl PointerDevice {
        pub fn new(ordinal: usize) -> Result<Self> {
            let context = CudaContext::new(ordinal).map_err(cuda)?;
            let stream = context.default_stream();
            let module = context.load_module(Ptx::from_src(PTX)).map_err(cuda)?;
            let kernel = module
                .load_function("lawful_pointer_select_batch")
                .map_err(cuda)?;
            Ok(Self { stream, kernel })
        }

        /// Execute `lawful_pointer_select_batch` on the GPU for all `items` in one kernel launch,
        /// returning `(decisions, elapsed_microseconds)`.
        #[allow(unsafe_code)]
        pub fn select_batch(
            &self,
            head: &LawfulPointerHead,
            items: &[PointerStepItem],
        ) -> Result<(Vec<PointerDecision>, u64)> {
            if items.is_empty() {
                return Err(invalid("select_batch requires at least one item"));
            }
            let n_items = len_u32(items.len())?;
            let (admission_mode, nlet_width) = match head.admission_mode() {
                RouteAdmissionMode::SlotAddressSemiprime => (0u32, 0u32),
                RouteAdmissionMode::SharedAtomGcd => (1u32, 0u32),
                RouteAdmissionMode::OrderedNlet { width } => (2u32, len_u32(width)?),
            };
            let gate_threshold_margin = head.gate_threshold_margin();
            let fold_slice = head.fold_table().as_slice();
            let fold_len = len_u32(fold_slice.len())?;

            let mut window_tokens = Vec::new();
            let mut window_off = Vec::with_capacity(items.len());
            let mut window_len = Vec::with_capacity(items.len());

            let mut base_margins = Vec::new();
            let mut base_off = Vec::with_capacity(items.len());
            let mut base_len = Vec::with_capacity(items.len());

            let mut learned_gate_margin = Vec::with_capacity(items.len());
            let mut step_offset = Vec::with_capacity(items.len());
            let mut query_has_slot = Vec::with_capacity(items.len());
            let mut query_slot_lo = Vec::with_capacity(items.len());
            let mut query_slot_hi = Vec::with_capacity(items.len());
            let mut query_prod_lo = Vec::with_capacity(items.len());
            let mut query_prod_hi = Vec::with_capacity(items.len());

            let mut query_prec_tokens = Vec::new();
            let mut query_prec_off = Vec::with_capacity(items.len());
            let mut query_prec_len = Vec::with_capacity(items.len());

            let mut cand_off = Vec::with_capacity(items.len());
            let mut cand_count = Vec::with_capacity(items.len());
            let mut cand_span_start = Vec::new();
            let mut cand_span_end = Vec::new();
            let mut cand_slot_lo = Vec::new();
            let mut cand_slot_hi = Vec::new();
            let mut cand_prec_tokens = Vec::new();
            let mut cand_prec_off = Vec::new();
            let mut cand_prec_len = Vec::new();
            let mut cand_rank_bonus = Vec::new();

            let mut oracle_mode = Vec::with_capacity(items.len());
            let mut oracle_start = Vec::with_capacity(items.len());
            let mut oracle_end = Vec::with_capacity(items.len());
            let mut oracle_forced_gate = Vec::with_capacity(items.len());

            for item in items {
                window_off.push(len_u32(window_tokens.len())?);
                window_len.push(len_u32(item.window_tokens.len())?);
                window_tokens.extend_from_slice(&item.window_tokens);

                base_off.push(len_u32(base_margins.len())?);
                base_len.push(len_u32(item.base_margins.len())?);
                base_margins.extend_from_slice(&item.base_margins);

                learned_gate_margin.push(item.learned_gate_margin);
                step_offset.push(len_u32(item.step_offset)?);

                match item.query_slot {
                    Some(slot) => {
                        let (lo, hi) = split_u128(slot.semiprime());
                        query_has_slot.push(1u32);
                        query_slot_lo.push(lo);
                        query_slot_hi.push(hi);
                    }
                    None => {
                        query_has_slot.push(0u32);
                        query_slot_lo.push(0u64);
                        query_slot_hi.push(0u64);
                    }
                }
                let (qp_lo, qp_hi) = split_u128(item.query_product);
                query_prod_lo.push(qp_lo);
                query_prod_hi.push(qp_hi);

                query_prec_off.push(len_u32(query_prec_tokens.len())?);
                query_prec_len.push(len_u32(item.query_preceding.len())?);
                query_prec_tokens.extend_from_slice(&item.query_preceding);

                cand_off.push(len_u32(cand_span_start.len())?);
                cand_count.push(len_u32(item.candidates.len())?);
                for cand in &item.candidates {
                    cand_span_start.push(len_u32(cand.span.0)?);
                    cand_span_end.push(len_u32(cand.span.1)?);
                    let (cs_lo, cs_hi) = split_u128(cand.slot_product);
                    cand_slot_lo.push(cs_lo);
                    cand_slot_hi.push(cs_hi);
                    cand_prec_off.push(len_u32(cand_prec_tokens.len())?);
                    cand_prec_len.push(len_u32(cand.preceding_tokens.len())?);
                    cand_prec_tokens.extend_from_slice(&cand.preceding_tokens);
                    cand_rank_bonus.push(cand.learned_rank_bonus);
                }

                match item.oracle {
                    CopySpanOracle::Off => {
                        oracle_mode.push(0u32);
                        oracle_start.push(0u32);
                        oracle_end.push(0u32);
                        oracle_forced_gate.push(0i64);
                    }
                    CopySpanOracle::O1SpanLearnedGate { start, end } => {
                        oracle_mode.push(1u32);
                        oracle_start.push(len_u32(start)?);
                        oracle_end.push(len_u32(end)?);
                        oracle_forced_gate.push(0i64);
                    }
                    CopySpanOracle::O2SpanForcedGate {
                        start,
                        end,
                        forced_gate_margin,
                    } => {
                        oracle_mode.push(2u32);
                        oracle_start.push(len_u32(start)?);
                        oracle_end.push(len_u32(end)?);
                        oracle_forced_gate.push(forced_gate_margin);
                    }
                }
            }

            let s = &self.stream;
            let started = Instant::now();

            let d_fold = s.clone_htod(&nonempty(fold_slice)).map_err(cuda)?;
            let d_window_tokens = s.clone_htod(&nonempty(&window_tokens)).map_err(cuda)?;
            let d_window_off = s.clone_htod(&window_off).map_err(cuda)?;
            let d_window_len = s.clone_htod(&window_len).map_err(cuda)?;
            let d_base_margins = s.clone_htod(&nonempty(&base_margins)).map_err(cuda)?;
            let d_base_off = s.clone_htod(&base_off).map_err(cuda)?;
            let d_base_len = s.clone_htod(&base_len).map_err(cuda)?;
            let d_learned_gate = s.clone_htod(&learned_gate_margin).map_err(cuda)?;
            let d_step_offset = s.clone_htod(&step_offset).map_err(cuda)?;
            let d_query_has_slot = s.clone_htod(&query_has_slot).map_err(cuda)?;
            let d_query_slot_lo = s.clone_htod(&query_slot_lo).map_err(cuda)?;
            let d_query_slot_hi = s.clone_htod(&query_slot_hi).map_err(cuda)?;
            let d_query_prod_lo = s.clone_htod(&query_prod_lo).map_err(cuda)?;
            let d_query_prod_hi = s.clone_htod(&query_prod_hi).map_err(cuda)?;
            let d_query_prec_tokens = s.clone_htod(&nonempty(&query_prec_tokens)).map_err(cuda)?;
            let d_query_prec_off = s.clone_htod(&query_prec_off).map_err(cuda)?;
            let d_query_prec_len = s.clone_htod(&query_prec_len).map_err(cuda)?;
            let d_cand_off = s.clone_htod(&cand_off).map_err(cuda)?;
            let d_cand_count = s.clone_htod(&cand_count).map_err(cuda)?;
            let d_cand_span_start = s.clone_htod(&nonempty(&cand_span_start)).map_err(cuda)?;
            let d_cand_span_end = s.clone_htod(&nonempty(&cand_span_end)).map_err(cuda)?;
            let d_cand_slot_lo = s.clone_htod(&nonempty(&cand_slot_lo)).map_err(cuda)?;
            let d_cand_slot_hi = s.clone_htod(&nonempty(&cand_slot_hi)).map_err(cuda)?;
            let d_cand_prec_tokens = s.clone_htod(&nonempty(&cand_prec_tokens)).map_err(cuda)?;
            let d_cand_prec_off = s.clone_htod(&nonempty(&cand_prec_off)).map_err(cuda)?;
            let d_cand_prec_len = s.clone_htod(&nonempty(&cand_prec_len)).map_err(cuda)?;
            let d_cand_rank_bonus = s.clone_htod(&nonempty(&cand_rank_bonus)).map_err(cuda)?;
            let d_oracle_mode = s.clone_htod(&oracle_mode).map_err(cuda)?;
            let d_oracle_start = s.clone_htod(&oracle_start).map_err(cuda)?;
            let d_oracle_end = s.clone_htod(&oracle_end).map_err(cuda)?;
            let d_oracle_forced_gate = s.clone_htod(&oracle_forced_gate).map_err(cuda)?;

            let n = items.len();
            let mut out_status = s.alloc_zeros::<u32>(n).map_err(cuda)?;
            let mut out_emitted_token = s.alloc_zeros::<u32>(n).map_err(cuda)?;
            let mut out_base_top_token = s.alloc_zeros::<u32>(n).map_err(cuda)?;
            let mut out_base_top_margin = s.alloc_zeros::<i64>(n).map_err(cuda)?;
            let mut out_copied = s.alloc_zeros::<u32>(n).map_err(cuda)?;
            let mut out_has_span = s.alloc_zeros::<u32>(n).map_err(cuda)?;
            let mut out_span_start = s.alloc_zeros::<u32>(n).map_err(cuda)?;
            let mut out_span_end = s.alloc_zeros::<u32>(n).map_err(cuda)?;
            let mut out_has_copy_token = s.alloc_zeros::<u32>(n).map_err(cuda)?;
            let mut out_candidate_copy_token = s.alloc_zeros::<u32>(n).map_err(cuda)?;
            let mut out_has_route_score = s.alloc_zeros::<u32>(n).map_err(cuda)?;
            let mut out_route_score = s.alloc_zeros::<i64>(n).map_err(cuda)?;
            let mut out_effective_gate_margin = s.alloc_zeros::<i64>(n).map_err(cuda)?;

            let mut launch = s.launch_builder(&self.kernel);
            launch
                .arg(&n_items)
                .arg(&admission_mode)
                .arg(&nlet_width)
                .arg(&gate_threshold_margin)
                .arg(&d_fold)
                .arg(&fold_len)
                .arg(&d_window_tokens)
                .arg(&d_window_off)
                .arg(&d_window_len)
                .arg(&d_base_margins)
                .arg(&d_base_off)
                .arg(&d_base_len)
                .arg(&d_learned_gate)
                .arg(&d_step_offset)
                .arg(&d_query_has_slot)
                .arg(&d_query_slot_lo)
                .arg(&d_query_slot_hi)
                .arg(&d_query_prod_lo)
                .arg(&d_query_prod_hi)
                .arg(&d_query_prec_tokens)
                .arg(&d_query_prec_off)
                .arg(&d_query_prec_len)
                .arg(&d_cand_off)
                .arg(&d_cand_count)
                .arg(&d_cand_span_start)
                .arg(&d_cand_span_end)
                .arg(&d_cand_slot_lo)
                .arg(&d_cand_slot_hi)
                .arg(&d_cand_prec_tokens)
                .arg(&d_cand_prec_off)
                .arg(&d_cand_prec_len)
                .arg(&d_cand_rank_bonus)
                .arg(&d_oracle_mode)
                .arg(&d_oracle_start)
                .arg(&d_oracle_end)
                .arg(&d_oracle_forced_gate)
                .arg(&mut out_status)
                .arg(&mut out_emitted_token)
                .arg(&mut out_base_top_token)
                .arg(&mut out_base_top_margin)
                .arg(&mut out_copied)
                .arg(&mut out_has_span)
                .arg(&mut out_span_start)
                .arg(&mut out_span_end)
                .arg(&mut out_has_copy_token)
                .arg(&mut out_candidate_copy_token)
                .arg(&mut out_has_route_score)
                .arg(&mut out_route_score)
                .arg(&mut out_effective_gate_margin);

            let threads = n_items.clamp(1, 128);
            let config = LaunchConfig {
                grid_dim: (n_items.div_ceil(threads), 1, 1),
                block_dim: (threads, 1, 1),
                shared_mem_bytes: 0,
            };
            // SAFETY: argument types and counts match `lawful_pointer_select_batch` in
            // `src/lawful_pointer.cu`; all slice lengths are checked on host and in the kernel.
            unsafe { launch.launch(config) }.map_err(cuda)?;

            let status = s.clone_dtoh(&out_status).map_err(cuda)?;
            let emitted_token = s.clone_dtoh(&out_emitted_token).map_err(cuda)?;
            let base_top_token = s.clone_dtoh(&out_base_top_token).map_err(cuda)?;
            let base_top_margin = s.clone_dtoh(&out_base_top_margin).map_err(cuda)?;
            let copied = s.clone_dtoh(&out_copied).map_err(cuda)?;
            let has_span = s.clone_dtoh(&out_has_span).map_err(cuda)?;
            let span_start = s.clone_dtoh(&out_span_start).map_err(cuda)?;
            let span_end = s.clone_dtoh(&out_span_end).map_err(cuda)?;
            let has_copy_token = s.clone_dtoh(&out_has_copy_token).map_err(cuda)?;
            let candidate_copy_token = s.clone_dtoh(&out_candidate_copy_token).map_err(cuda)?;
            let has_route_score = s.clone_dtoh(&out_has_route_score).map_err(cuda)?;
            let route_score = s.clone_dtoh(&out_route_score).map_err(cuda)?;
            let effective_gate_margin = s.clone_dtoh(&out_effective_gate_margin).map_err(cuda)?;
            let elapsed = u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX);

            let mut decisions = Vec::with_capacity(n);
            for i in 0..n {
                if status[i] != 0 {
                    return Err(invalid(format!(
                        "lawful_pointer_select_batch reported invalid input at item {i}"
                    )));
                }
                decisions.push(PointerDecision {
                    emitted_token: emitted_token[i],
                    base_top_token: base_top_token[i],
                    base_top_margin: base_top_margin[i],
                    copied: copied[i] != 0,
                    selected_span: (has_span[i] != 0)
                        .then_some((span_start[i] as usize, span_end[i] as usize)),
                    candidate_copy_token: (has_copy_token[i] != 0)
                        .then_some(candidate_copy_token[i]),
                    route_score: (has_route_score[i] != 0).then_some(route_score[i]),
                    effective_gate_margin: effective_gate_margin[i],
                });
            }
            Ok((decisions, elapsed))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_pieces() -> Vec<Vec<u8>> {
        vec![
            b"<pad>".to_vec(),   // 0
            b"Ana".to_vec(),     // 1
            b" ana".to_vec(),    // 2
            b"Bob".to_vec(),     // 3
            b"locker".to_vec(),  // 4
            b" locker".to_vec(), // 5
            b"pin".to_vec(),     // 6
            b" is".to_vec(),     // 7
            b" 4".to_vec(),      // 8
            b"7".to_vec(),       // 9
            b" 9".to_vec(),      // 10
            b"2".to_vec(),       // 11
            b".".to_vec(),       // 12
        ]
    }

    #[test]
    fn slot_address_semiprime_admits_exact_entity_attribute_pair_only() {
        // Entity primes: Ana = 5, Bob = 7. Attribute primes: locker = 11, pin = 13.
        let ana_locker = SlotAddressKey::from_entity_attribute_primes(5, 11).expect("valid primes");
        assert_eq!(ana_locker.semiprime(), 55);

        // Composite primes or equal primes are rejected at construction.
        assert!(SlotAddressKey::from_entity_attribute_primes(4, 11).is_err());
        assert!(SlotAddressKey::from_entity_attribute_primes(5, 5).is_err());

        let fold = CanonicalFoldTable::from_pieces(&sample_pieces()).expect("fold table");
        let head = LawfulPointerHead::new(RouteAdmissionMode::SlotAddressSemiprime, fold, 100);

        // Window: "Ana locker is 4 7 . Bob locker is 9 2 ."
        let window = vec![1, 5, 7, 8, 9, 12, 3, 5, 7, 10, 11, 12];
        let candidates = vec![
            // Candidate 0: Ana's locker span [3, 5) (" 4", "7"), slot_product = 5 * 11 = 55
            SourceSpanCandidate {
                span: (3, 5),
                slot_product: 5 * 11,
                preceding_tokens: vec![1, 5, 7],
                learned_rank_bonus: 50,
            },
            // Candidate 1: Bob's locker span [9, 11) (" 9", "2"), slot_product = 7 * 11 = 77
            SourceSpanCandidate {
                span: (9, 11),
                slot_product: 7 * 11,
                preceding_tokens: vec![3, 5, 7],
                learned_rank_bonus: 500, // Even with a higher bonus, wrong entity is not admitted!
            },
        ];

        let mut base_margins = vec![0i64; 13];
        base_margins[7] = 200; // Base model prefers frame token " is" (token 7)

        // Step 0 and Step 1 of Ana's locker value span:
        let step0 = head
            .select_step(
                &window,
                &base_margins,
                150,
                0,
                Some(ana_locker),
                55,
                &[],
                &candidates,
            )
            .expect("step 0");
        assert!(step0.copied);
        assert_eq!(step0.selected_span, Some((3, 5)));
        assert_eq!(step0.emitted_token, 8); // " 4"

        let step1 = head
            .select_step(
                &window,
                &base_margins,
                150,
                1,
                Some(ana_locker),
                55,
                &[],
                &candidates,
            )
            .expect("step 1");
        assert!(step1.copied);
        assert_eq!(step1.selected_span, Some((3, 5)));
        assert_eq!(step1.emitted_token, 9); // "7"
    }

    #[test]
    fn oracle_off_reproduces_base_scorer_bit_for_bit_when_gate_closed_or_unadmitted() {
        let fold = CanonicalFoldTable::from_pieces(&sample_pieces()).expect("fold table");
        let head = LawfulPointerHead::new(RouteAdmissionMode::SlotAddressSemiprime, fold, 100);
        let window = vec![1, 5, 7, 8, 9, 12];
        let base_margins = vec![10, 20, 15, 40, 5, 99, 12, 105, 8, 7, 6, 4, 1];
        let candidates = vec![SourceSpanCandidate {
            span: (3, 5),
            slot_product: 55,
            preceding_tokens: vec![1, 5, 7],
            learned_rank_bonus: 0,
        }];
        let ana_locker = SlotAddressKey::from_entity_attribute_primes(5, 11).expect("valid");
        let bob_locker = SlotAddressKey::from_entity_attribute_primes(7, 11).expect("valid");

        // Case 1 (gate closed): candidate is admitted, but gate margin 40 < threshold 100.
        let gate_closed = head
            .select_step(
                &window,
                &base_margins,
                40,
                0,
                Some(ana_locker),
                55,
                &[],
                &candidates,
            )
            .expect("gate closed");
        assert!(!gate_closed.copied);
        assert_eq!(gate_closed.selected_span, Some((3, 5)));
        assert_eq!(gate_closed.emitted_token, 7);
        assert_eq!(gate_closed.base_top_token, 7);
        assert_eq!(gate_closed.base_top_margin, 105);

        // Case 2 (unadmitted): gate margin 500 >= threshold 100, but query is Bob's locker (77)
        // which does not divide candidate slot_product (55).
        let unadmitted = head
            .select_step(
                &window,
                &base_margins,
                500,
                0,
                Some(bob_locker),
                77,
                &[],
                &candidates,
            )
            .expect("unadmitted");
        assert!(!unadmitted.copied);
        assert_eq!(unadmitted.selected_span, None);
        assert_eq!(unadmitted.route_score, None);
        assert_eq!(unadmitted.emitted_token, 7);
        assert_eq!(unadmitted.base_top_token, 7);
        assert_eq!(unadmitted.base_top_margin, 105);
    }

    #[test]
    fn base_margin_comparison_blocks_copy_when_base_outscores_copy_in_both_n_and_o1() {
        let fold = CanonicalFoldTable::from_pieces(&sample_pieces()).expect("fold table");
        let mut head = LawfulPointerHead::new(RouteAdmissionMode::SlotAddressSemiprime, fold, 100);
        let window = vec![1, 5, 7, 8, 9, 12];
        let ana_locker = SlotAddressKey::from_entity_attribute_primes(5, 11).expect("valid");
        // ilog2_gcd_weight_q8(55) = floor(log2(55)) * 256 = 5 * 256 = 1280.
        // With learned_rank_bonus = 74 and learned_gate_margin = 120 (>= threshold 100),
        // route_score = 1354 and total_copy_margin = 1354 + 120 = 1474.
        let candidates = vec![SourceSpanCandidate {
            span: (3, 5),
            slot_product: 55,
            preceding_tokens: vec![1, 5, 7],
            learned_rank_bonus: 74,
        }];

        // When base_top_margin = 1500 >= total_copy_margin (1474), N does NOT copy even though
        // learned_gate_margin (120) >= gate_threshold_margin (100).
        let mut high_base = vec![0i64; 13];
        high_base[7] = 1500;
        let n_blocked = head
            .select_step(
                &window,
                &high_base,
                120,
                0,
                Some(ana_locker),
                55,
                &[],
                &candidates,
            )
            .expect("N blocked by base margin");
        assert!(!n_blocked.copied);
        assert_eq!(n_blocked.route_score, Some(1354));
        assert_eq!(n_blocked.emitted_token, 7);

        // O1 on the same span [3, 5) preserves the exact same base-margin comparison, so O1
        // is also blocked and matches N bit-for-bit!
        head.set_copy_span_oracle(
            CopySpanOracle::O1SpanLearnedGate { start: 3, end: 5 },
            window.len(),
        )
        .expect("set O1");
        let o1_blocked = head
            .select_step(
                &window,
                &high_base,
                120,
                0,
                Some(ana_locker),
                55,
                &[],
                &candidates,
            )
            .expect("O1 blocked by base margin");
        assert_eq!(o1_blocked, n_blocked);

        // Lowering base_top_margin to 1400 (< 1474) allows the copy in both N and O1.
        let mut low_base = vec![0i64; 13];
        low_base[7] = 1400;
        let o1_open = head
            .select_step(
                &window,
                &low_base,
                120,
                0,
                Some(ana_locker),
                55,
                &[],
                &candidates,
            )
            .expect("O1 open");
        assert!(o1_open.copied);
        assert_eq!(o1_open.emitted_token, 8);
        head.clear_copy_span_oracle();
        let n_open = head
            .select_step(
                &window,
                &low_base,
                120,
                0,
                Some(ana_locker),
                55,
                &[],
                &candidates,
            )
            .expect("N open");
        assert_eq!(n_open, o1_open);
    }

    #[test]
    fn shared_atom_gcd_ranks_rarer_primes_higher_and_breaks_ties_by_latest_span_start() {
        let fold = CanonicalFoldTable::from_pieces(&sample_pieces()).expect("fold table");
        let head = LawfulPointerHead::new(RouteAdmissionMode::SharedAtomGcd, fold, 100);
        let window = vec![1, 5, 7, 8, 9, 12, 3, 5, 7, 10, 11, 12];
        let base_margins = vec![0i64; 13];

        // Query product has common prime 3 (ilog2=1 -> 256) and rare prime 13 (ilog2=3 -> 768).
        let query_product = 3u128 * 13u128;
        let candidates_rarity = vec![
            SourceSpanCandidate {
                span: (3, 5),
                slot_product: 3,
                preceding_tokens: vec![],
                learned_rank_bonus: 0,
            },
            SourceSpanCandidate {
                span: (9, 11),
                slot_product: 13,
                preceding_tokens: vec![],
                learned_rank_bonus: 0,
            },
        ];
        let rare_wins = head
            .select_step(
                &window,
                &base_margins,
                150,
                0,
                None,
                query_product,
                &[],
                &candidates_rarity,
            )
            .expect("rare prime wins");
        assert!(rare_wins.copied);
        assert_eq!(rare_wins.selected_span, Some((9, 11)));
        assert_eq!(rare_wins.route_score, Some(768));
        assert_eq!(rare_wins.emitted_token, 10);

        // Equal route_score tie-break: latest span.0 ((9, 11) over (3, 5)) wins.
        let candidates_tie = vec![
            SourceSpanCandidate {
                span: (3, 5),
                slot_product: 13,
                preceding_tokens: vec![],
                learned_rank_bonus: 0,
            },
            SourceSpanCandidate {
                span: (9, 11),
                slot_product: 13,
                preceding_tokens: vec![],
                learned_rank_bonus: 0,
            },
        ];
        let latest_wins = head
            .select_step(
                &window,
                &base_margins,
                150,
                0,
                None,
                query_product,
                &[],
                &candidates_tie,
            )
            .expect("latest span start wins tie");
        assert!(latest_wins.copied);
        assert_eq!(latest_wins.selected_span, Some((9, 11)));
        assert_eq!(latest_wins.emitted_token, 10);
    }

    #[test]
    fn copy_span_oracle_separates_o1_learned_gate_from_o2_forced_gate_and_restores_on_clear() {
        let fold = CanonicalFoldTable::from_pieces(&sample_pieces()).expect("fold table");
        let mut head = LawfulPointerHead::new(RouteAdmissionMode::SlotAddressSemiprime, fold, 100);
        // Window where Bob's locker [9, 11) is " 9", "2"
        let window = vec![1, 5, 7, 8, 9, 12, 3, 5, 7, 10, 11, 12];
        let mut base_margins = vec![0i64; 13];
        base_margins[7] = 300; // Base model predicts frame token 7 (" is")
        let bob_locker = SlotAddressKey::from_entity_attribute_primes(7, 11).expect("valid");

        // Suppose the head's learned gate margin is 25 (below threshold 100) and no candidates match.
        let normal = head
            .select_step(
                &window,
                &base_margins,
                25,
                0,
                Some(bob_locker),
                77,
                &[],
                &[],
            )
            .expect("normal");
        assert!(!normal.copied);
        assert_eq!(normal.emitted_token, 7);

        // O1 (span supplied, learned gate kept at 25 < 100): addressing is solved, but gate blocks copy.
        head.set_copy_span_oracle(
            CopySpanOracle::O1SpanLearnedGate { start: 9, end: 11 },
            window.len(),
        )
        .expect("set O1");
        let o1 = head
            .select_step(
                &window,
                &base_margins,
                25,
                0,
                Some(bob_locker),
                77,
                &[],
                &[],
            )
            .expect("O1");
        assert!(!o1.copied);
        assert_eq!(o1.selected_span, Some((9, 11)));
        assert_eq!(o1.candidate_copy_token, Some(10));
        assert_eq!(o1.route_score, Some(1536));
        assert_eq!(o1.emitted_token, 7);

        // O2 (span supplied, gate forced to 250 >= 100): both tokens of the span (" 9", "2") emit!
        head.set_copy_span_oracle(
            CopySpanOracle::O2SpanForcedGate {
                start: 9,
                end: 11,
                forced_gate_margin: 250,
            },
            window.len(),
        )
        .expect("set O2");
        let o2_step0 = head
            .select_step(
                &window,
                &base_margins,
                25,
                0,
                Some(bob_locker),
                77,
                &[],
                &[],
            )
            .expect("O2 step 0");
        let o2_step1 = head
            .select_step(
                &window,
                &base_margins,
                25,
                1,
                Some(bob_locker),
                77,
                &[],
                &[],
            )
            .expect("O2 step 1");
        assert!(o2_step0.copied);
        assert_eq!(o2_step0.emitted_token, 10); // " 9"
        assert!(o2_step1.copied);
        assert_eq!(o2_step1.emitted_token, 11); // "2"

        // Clearing restores normal behavior bit-for-bit.
        head.clear_copy_span_oracle();
        let restored = head
            .select_step(
                &window,
                &base_margins,
                25,
                0,
                Some(bob_locker),
                77,
                &[],
                &[],
            )
            .expect("restored");
        assert_eq!(restored, normal);
    }

    #[test]
    fn copy_span_oracle_refuses_empty_reversed_or_out_of_window_spans() {
        let fold = CanonicalFoldTable::identity(16).expect("identity fold");
        let mut head = LawfulPointerHead::new(RouteAdmissionMode::SharedAtomGcd, fold, 50);
        assert!(head
            .set_copy_span_oracle(CopySpanOracle::O1SpanLearnedGate { start: 4, end: 4 }, 8)
            .is_err());
        assert!(head
            .set_copy_span_oracle(CopySpanOracle::O1SpanLearnedGate { start: 5, end: 3 }, 8)
            .is_err());
        assert!(head
            .set_copy_span_oracle(CopySpanOracle::O1SpanLearnedGate { start: 2, end: 9 }, 8)
            .is_err());
    }

    #[test]
    fn locate_last_value_span_finds_last_occurrence_across_token_boundaries() {
        let pieces = sample_pieces();
        // First statement: "Ana locker is 47." ([1, 5, 7, 8, 9, 12])
        // Second statement (update): "Ana locker is 92." ([1, 5, 7, 10, 11, 12])
        // Repeated earlier value "47" at [3, 5) and again at [15, 17).
        let window = vec![
            1, 5, 7, 8, 9, 12, // "Ana locker is 47."
            1, 5, 7, 10, 11, 12, // "Ana locker is 92."
            1, 5, 7, 8, 9, 12, // "Ana locker is 47."
        ];
        // "47" crosses token 8 (" 4") and token 9 ("7"); last occurrence is [15, 17).
        let span_47 = locate_last_value_span(&window, &pieces, b"47")
            .expect("valid pieces")
            .expect("found 47");
        assert_eq!(span_47, (15, 17));

        let span_92 = locate_last_value_span(&window, &pieces, b"92")
            .expect("valid pieces")
            .expect("found 92");
        assert_eq!(span_92, (9, 11));

        let missing = locate_last_value_span(&window, &pieces, b"38").expect("valid pieces");
        assert_eq!(missing, None);
    }

    #[test]
    fn locate_last_value_span_enforces_alphanumeric_boundaries() {
        let pieces = sample_pieces();
        // Window: "Ana locker is 47." ([1, 5, 7, 8, 9, 12]) where "47" is tokens 8 (" 4") + 9 ("7").
        let window_with_47_only = vec![1, 5, 7, 8, 9, 12];
        // Searching for single-digit "4" or "7" must NOT match inside "47".
        assert_eq!(
            locate_last_value_span(&window_with_47_only, &pieces, b"4").expect("valid"),
            None
        );
        assert_eq!(
            locate_last_value_span(&window_with_47_only, &pieces, b"7").expect("valid"),
            None
        );

        // When standalone "4" is stated ("Ana locker is 4." -> [1, 5, 7, 8, 12]) followed by
        // "Bob locker is 47." ([3, 5, 7, 8, 9, 12]), searching for "4" skips "47" and finds [3, 4)!
        let window_4_then_47 = vec![1, 5, 7, 8, 12, 3, 5, 7, 8, 9, 12];
        assert_eq!(
            locate_last_value_span(&window_4_then_47, &pieces, b"4").expect("valid"),
            Some((3, 4))
        );
    }

    #[test]
    fn canonical_fold_table_matches_case_and_leading_space_without_altering_emitted_token() {
        let pieces = sample_pieces();
        let fold = CanonicalFoldTable::from_pieces(&pieces).expect("fold");
        // Token 1 is "Ana", Token 2 is " ana"; both fold to canonical ID 1.
        assert_eq!(fold.fold(1).unwrap(), 1);
        assert_eq!(fold.fold(2).unwrap(), 1);
        // Token 4 is "locker", Token 5 is " locker"; both fold to canonical ID 4.
        assert_eq!(fold.fold(4).unwrap(), 4);
        assert_eq!(fold.fold(5).unwrap(), 4);

        let head = LawfulPointerHead::new(RouteAdmissionMode::OrderedNlet { width: 2 }, fold, 100);
        // Window uses " ana" (2) and " locker" (5) before value token " 9" (10)
        let window = vec![2, 5, 7, 10, 12];
        let candidates = vec![SourceSpanCandidate {
            span: (3, 4),
            slot_product: 1,
            preceding_tokens: vec![2, 5], // " ana", " locker"
            learned_rank_bonus: 50,
        }];
        let base_margins = vec![0i64; 13];
        // Query uses "Ana" (1) and "locker" (4) — matches via CanonicalFoldTable,
        // and emits raw token 10 from the window untouched.
        let decision = head
            .select_step(
                &window,
                &base_margins,
                120,
                0,
                None,
                1,
                &[1, 4],
                &candidates,
            )
            .expect("nlet decision");
        assert!(decision.copied);
        assert_eq!(decision.emitted_token, 10);
    }

    #[test]
    fn diagnostic_partition_covers_all_four_branches_exclusively_and_exhaustively() {
        // Branch A: O1 clears both +32 rows and +16 pairs.
        assert_eq!(
            classify_oracle_outcome(20, 52, 60, 10, 26, 30),
            OracleDiagnosticBranch::BranchAAddressingBottleneck
        );
        // Branch B: O1 misses pair bar (+15 < 16), O2 clears both +32 rows and +16 pairs.
        assert_eq!(
            classify_oracle_outcome(20, 55, 64, 10, 25, 32),
            OracleDiagnosticBranch::BranchBGateMarginBottleneck
        );
        // Branch C: Neither O1 nor O2 clears +32 rows or +16 pairs.
        assert_eq!(
            classify_oracle_outcome(20, 30, 45, 10, 15, 22),
            OracleDiagnosticBranch::BranchCEmissionBottleneck
        );
        // Branch D: O1 misses, and O2 clears +32 rows (52 - 20 = 32) but misses +16 pairs (20 - 10 = 10).
        assert_eq!(
            classify_oracle_outcome(20, 25, 52, 10, 12, 20),
            OracleDiagnosticBranch::BranchDDiscordantInconclusive
        );
    }

    #[test]
    fn paired_cluster_jackknife_se_milli_computes_exact_integer_block_dispersion() {
        // Fewer than 2 blocks is rejected.
        assert!(compute_paired_cluster_se_milli(&[4]).is_err());

        // Identical block deltas have zero dispersion -> SE = 0.
        assert_eq!(compute_paired_cluster_se_milli(&[3; 16]).unwrap(), 0);

        // 16 blocks where 8 blocks have delta +4 and 8 blocks have delta 0:
        // Total Delta = 32, mean = 2. Each block has (d_b - 2)^2 = 4, sum = 64.
        // Var(Delta) = (16 / 15) * 64 = 1024 / 15 = 68.2666...
        // floor(1000 * sqrt(1024 / 15)) = floor(sqrt(68_266_666)) = 8262.
        let mut blocks = vec![4i64; 8];
        blocks.extend(vec![0i64; 8]);
        assert_eq!(compute_paired_cluster_se_milli(&blocks).unwrap(), 8262);
    }

    fn make_pair_rows(
        pair_idx: usize,
        block_index: usize,
        learned_gate_margin: i64,
    ) -> [OracleRowInput; 2] {
        // Window: "Ana locker is 4 7 . Bob locker is 9 2 ."
        let window_tokens = vec![1, 5, 7, 8, 9, 12, 3, 5, 7, 10, 11, 12];
        let mut base_margins = vec![0i64; 13];
        base_margins[7] = 300; // Base model predicts frame token 7 (" is") on both steps
        let ana_locker = SlotAddressKey::from_entity_attribute_primes(5, 11).expect("valid");
        let bob_locker = SlotAddressKey::from_entity_attribute_primes(7, 11).expect("valid");

        let pair_id = format!("pair-{pair_idx:03}");
        [
            OracleRowInput {
                row_id: format!("{pair_id}-a"),
                pair_id: pair_id.clone(),
                block_index,
                window_tokens: window_tokens.clone(),
                query_slot: Some(ana_locker),
                query_product: 55,
                query_preceding: vec![],
                candidates: vec![],
                expected_value_bytes: b"47".to_vec(),
                distractor_value_bytes: b"92".to_vec(),
                accepted_reply_tokens: vec![vec![8, 9]],
                step_base_margins: vec![base_margins.clone(), base_margins.clone()],
                step_learned_gate_margins: vec![learned_gate_margin, learned_gate_margin],
                recorded_baseline_tokens: vec![7, 7],
                forced_gate_margin: 500,
            },
            OracleRowInput {
                row_id: format!("{pair_id}-b"),
                pair_id,
                block_index,
                window_tokens,
                query_slot: Some(bob_locker),
                query_product: 77,
                query_preceding: vec![],
                candidates: vec![],
                expected_value_bytes: b"92".to_vec(),
                distractor_value_bytes: b"47".to_vec(),
                accepted_reply_tokens: vec![vec![10, 11]],
                step_base_margins: vec![base_margins.clone(), base_margins],
                step_learned_gate_margins: vec![learned_gate_margin, learned_gate_margin],
                recorded_baseline_tokens: vec![7, 7],
                forced_gate_margin: 500,
            },
        ]
    }

    #[test]
    fn batch_oracle_evaluator_halts_on_baseline_mismatch_wrong_span_leak_and_missing_span() {
        let pieces = sample_pieces();
        let fold = CanonicalFoldTable::from_pieces(&pieces).expect("fold");
        let head = LawfulPointerHead::new(RouteAdmissionMode::SlotAddressSemiprime, fold, 100);

        // 1. Missing span coverage halts with MissingSpanCoverage.
        let mut missing_rows = Vec::new();
        let [mut r0, r1] = make_pair_rows(0, 0, 25);
        r0.expected_value_bytes = b"99".to_vec();
        missing_rows.push(r0);
        missing_rows.push(r1);
        missing_rows.extend(make_pair_rows(1, 1, 25));
        let stop_missing = evaluate_forced_copy_oracle_batch(&head, &pieces, &missing_rows, 2)
            .expect("eval ok")
            .expect_err("must halt on missing span");
        assert_eq!(
            stop_missing,
            OracleStopFailure::MissingSpanCoverage {
                missing_target_spans: 1,
                missing_distractor_spans: 0,
            }
        );

        // 2. Baseline identity mismatch halts with BaselineIdentityMismatch.
        let mut mismatch_rows = Vec::new();
        let [mut m0, m1] = make_pair_rows(0, 0, 25);
        m0.recorded_baseline_tokens = vec![8, 9];
        mismatch_rows.push(m0);
        mismatch_rows.push(m1);
        mismatch_rows.extend(make_pair_rows(1, 1, 25));
        let stop_mismatch = evaluate_forced_copy_oracle_batch(&head, &pieces, &mismatch_rows, 2)
            .expect("eval ok")
            .expect_err("must halt on baseline mismatch");
        assert_eq!(
            stop_mismatch,
            OracleStopFailure::BaselineIdentityMismatch {
                token_mismatches: 1,
                verdict_mismatches: 1,
            }
        );

        // 3. Wrong-span leak halts with WrongSpanControlLeak if accepted_reply_tokens accepts distractor.
        let mut leak_rows = Vec::new();
        let [mut w0, mut w1] = make_pair_rows(0, 0, 25);
        w0.accepted_reply_tokens.push(vec![10, 11]);
        w1.accepted_reply_tokens.push(vec![8, 9]);
        leak_rows.push(w0);
        leak_rows.push(w1);
        leak_rows.extend(make_pair_rows(1, 1, 25));
        let stop_leak = evaluate_forced_copy_oracle_batch(&head, &pieces, &leak_rows, 2)
            .expect("eval ok")
            .expect_err("must halt on wrong-span leak");
        assert_eq!(
            stop_leak,
            OracleStopFailure::WrongSpanControlLeak {
                c_w: 2,
                c_n: 0,
                p_w: 1,
            }
        );
    }

    #[test]
    fn batch_oracle_evaluator_grades_opposite_role_pairs_and_selects_correct_branch() {
        let pieces = sample_pieces();
        let fold = CanonicalFoldTable::from_pieces(&pieces).expect("fold");
        let head = LawfulPointerHead::new(RouteAdmissionMode::SlotAddressSemiprime, fold, 100);

        // Build 32 pairs (64 rows) across 16 blocks (2 pairs = 4 rows per block).
        // When learned_gate_margin = 25 (< threshold 100), O1 stays blocked (0/64 rows, 0/32 pairs)
        // while O2 forces the gate open (64/64 rows, 32/32 pairs) -> Branch B!
        let mut rows_branch_b = Vec::with_capacity(64);
        for pair_idx in 0..32 {
            let block_idx = pair_idx / 2;
            rows_branch_b.extend(make_pair_rows(pair_idx, block_idx, 25));
        }
        let report_b = evaluate_forced_copy_oracle_batch(&head, &pieces, &rows_branch_b, 16)
            .expect("eval ok")
            .expect("no stop failure");
        assert_eq!(report_b.n.complete_rows, 0);
        assert_eq!(report_b.o1.complete_rows, 0);
        assert_eq!(report_b.o2.complete_rows, 64);
        assert_eq!(report_b.o2.pairs_both_complete_distinct, 32);
        assert_eq!(report_b.w.complete_rows, 0);
        assert_eq!(report_b.w.pairs_both_complete_distinct, 0);
        assert_eq!(
            report_b.branch,
            OracleDiagnosticBranch::BranchBGateMarginBottleneck
        );

        // When learned_gate_margin = 250 (>= threshold 100) and candidates are empty (addressing bottleneck),
        // N scores 0/64 while O1 supplies the span and clears both +32 rows and +16 pairs -> Branch A!
        let mut rows_branch_a = Vec::with_capacity(64);
        for pair_idx in 0..32 {
            let block_idx = pair_idx / 2;
            rows_branch_a.extend(make_pair_rows(pair_idx, block_idx, 250));
        }
        let report_a = evaluate_forced_copy_oracle_batch(&head, &pieces, &rows_branch_a, 16)
            .expect("eval ok")
            .expect("no stop failure");
        assert_eq!(report_a.n.complete_rows, 0);
        assert_eq!(report_a.o1.complete_rows, 64);
        assert_eq!(report_a.o1.pairs_both_complete_distinct, 32);
        assert_eq!(
            report_a.branch,
            OracleDiagnosticBranch::BranchAAddressingBottleneck
        );
    }
}

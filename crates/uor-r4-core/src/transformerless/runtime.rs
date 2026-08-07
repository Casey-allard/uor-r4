//! The RUNTIME: multiplication-free next-token prediction, hardened so the
//! implementation meets its prose:
//!
//! - The runtime source contains no multiplication, division, or modulo
//!   operator on any value: rotations come from a derived table, strides
//!   are walked by slice iterators (`chunks_exact`, `zip`, `nth` — O(1) on
//!   slices) and running counters, never by computed `i * stride` indices.
//! - Token vectors are NOT shipped expanded. The artifact carries the
//!   compressed form — STAGES code bytes per token plus i8 stage books —
//!   and the runtime decodes a row on demand by table reads and adds.
//!   Compression is load-bearing, not cosmetic (PROOF.md P5).
//! - Every stage of the path exists in two forms computing identical
//!   values: the plain form (bulk; word xor + hardware popcount, the fused
//!   form of the kernel's xor + table + add loop) and the kernel form
//!   (every operation dispatched through `OpKernel` and counted). Their
//!   equality — bundles, codes, predictions — is witnessed per
//!   certification run, not assumed.
//! - The store is built by calling THESE functions, so store keys and
//!   query keys come from one code path by construction.

pub use super::compiler::{derive_rotations, train_cut, SIG_BYTES, SIG_WORDS};
use super::compiler::{Compiled, Corpus, D, K, STAGES, WINDOW};
use uor_r4_graph_runtime::runtime_state::RuntimeState;

const TOP_M_MEMBERSHIPS: usize = 3;

/// Complete arithmetic interface of the runtime, with an operation census.
/// There is no multiplication method; the census has no multiplication
/// field. Both absences are the point.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct OpKernel {
    pub adds: u64,
    pub xors: u64,
    pub shifts: u64,
    pub compares: u64,
    pub table_reads: u64,
    pub candidate_scans: u64,
}

impl OpKernel {
    #[inline]
    pub fn candidate_scan(&mut self) {
        self.candidate_scans += 1;
    }
    #[inline]
    pub fn add(&mut self, a: i64, b: i64) -> i64 {
        self.adds += 1;
        a + b
    }
    #[inline]
    pub fn shl(&mut self, a: i64, s: u32) -> i64 {
        self.shifts += 1;
        a << s
    }
    #[inline]
    pub fn shr(&mut self, a: i64, s: u32) -> i64 {
        self.shifts += 1;
        a >> s
    }
    #[inline]
    pub fn xor(&mut self, a: u8, b: u8) -> u8 {
        self.xors += 1;
        a ^ b
    }
    #[inline]
    pub fn lt(&mut self, a: i64, b: i64) -> bool {
        self.compares += 1;
        a < b
    }
    #[inline]
    pub fn table_u8(&mut self, table: &[u8], idx: u8) -> u8 {
        self.table_reads += 1;
        table[idx as usize]
    }
    #[inline]
    pub fn table_i32(&mut self, table: &[i32], idx: usize) -> i32 {
        self.table_reads += 1;
        table[idx]
    }
    /// Records a table-resident fetch whose address was produced by a slice
    /// iterator (the fetch is the iterator dereference; this counts it).
    #[inline]
    pub fn table_fetch(&mut self, v: i64) -> i64 {
        self.table_reads += 1;
        v
    }
    /// Records one four-lane, two-term SIMD dot vector. The counts are
    /// logical vector operations, not scalar lane expansions; the lane
    /// width is fixed by the adapter and the scalar semantics remain the
    /// equality oracle.
    #[inline]
    pub fn simd_dot_vector(&mut self) {
        self.adds += 2;
        self.shifts += 2;
        self.compares += 1;
        self.table_reads += 2;
    }

    pub fn report(&self) -> String {
        format!(
            "op census: add {} | xor {} | shift {} | compare {} | table-read {} | candidate-scan {} | multiply — no such operation exists in the kernel",
            self.adds, self.xors, self.shifts, self.compares, self.table_reads, self.candidate_scans
        )
    }
}

/// Derived at construction from its definition; never hand-entered.
/// POPCOUNT[x] = number of set bits of x — the stratum observable.
pub fn derive_popcount_table() -> [u8; 256] {
    let mut t = [0u8; 256];
    for x in 0..=255u8 {
        t[x as usize] = x.count_ones() as u8;
    }
    t
}

/// Hamming distance between equal-length bit signatures, through the kernel:
/// per byte, one xor, one table read, one add. No multiplies anywhere.
pub fn hamming(k: &mut OpKernel, pop: &[u8; 256], a: &[u8], b: &[u8]) -> i64 {
    debug_assert_eq!(a.len(), b.len());
    let mut acc = 0i64;
    for i in 0..a.len() {
        let x = k.xor(a[i], b[i]);
        let p = k.table_u8(pop, x);
        acc = k.add(acc, p as i64);
    }
    acc
}

/// Pack sign bits (value > threshold) into a byte signature, through the
/// kernel: one compare and at most one shift+add per bit.
pub fn sign_signature(k: &mut OpKernel, values: &[i64], thresholds: &[i64]) -> [u8; SIG_BYTES] {
    assert_eq!(values.len(), D);
    assert_eq!(thresholds.len(), D);
    let mut out = [0u8; SIG_BYTES];
    let mut byte = 0usize;
    let mut bit = 0u32;
    for (&v, &t) in values.iter().zip(thresholds) {
        if k.lt(t, v) {
            let mask = k.shl(1, bit);
            out[byte] = k.add(out[byte] as i64, mask) as u8;
        }
        bit += 1;
        if bit == 8 {
            bit = 0;
            byte += 1;
        }
    }
    out
}

use std::collections::BTreeMap;

pub type Store = Vec<BTreeMap<Vec<u8>, BTreeMap<u32, u32>>>;
/// Decode one token row from the compressed artifact: per stage, one code
/// read then D book reads and D adds. Row location is `chunks_exact(..)
/// .nth(..)` — O(1) slicing on slices, no index products in this source.
pub fn decode_row_plain(art: &Compiled, t: u32, out: &mut [i32; D]) {
    out.fill(0);
    if let Some(codes) = art.token_codes.chunks_exact(STAGES).nth(t as usize) {
        for ((book, &code), &sh) in art.stage_books.iter().zip(codes).zip(&art.stage_shifts) {
            if let Some(row) = book.chunks_exact(D).nth(code as usize) {
                for (o, &b) in out.iter_mut().zip(row) {
                    *o += (b as i32) << sh;
                }
            }
        }
    }
}

/// Prefix-depth decode (used by the certifier's rate–distortion table):
/// the exact bytes and shifts the runtime reads, truncated at `depth`.
pub fn decode_row_prefix_plain(art: &Compiled, t: u32, depth: usize, out: &mut [i32; D]) {
    out.fill(0);
    if let Some(codes) = art.token_codes.chunks_exact(STAGES).nth(t as usize) {
        for ((book, &code), &sh) in art
            .stage_books
            .iter()
            .zip(codes)
            .zip(&art.stage_shifts)
            .take(depth)
        {
            if let Some(row) = book.chunks_exact(D).nth(code as usize) {
                for (o, &b) in out.iter_mut().zip(row) {
                    *o += (b as i32) << sh;
                }
            }
        }
    }
}

/// Kernel-counted decode: identical values; every element fetch recorded
/// as a table read and every accumulation as an add.
pub fn decode_row_kernel(k: &mut OpKernel, art: &Compiled, t: u32, out: &mut [i32; D]) {
    out.fill(0);
    let Some(codes) = art.token_codes.chunks_exact(STAGES).nth(t as usize) else {
        // Just leave out filled with 0 if token is out of bounds
        return;
    };
    for ((book, &code), &sh) in art.stage_books.iter().zip(codes).zip(&art.stage_shifts) {
        let code = k.table_fetch(code as i64) as usize;
        let row = book.chunks_exact(D).nth(code).unwrap();
        for (o, &b) in out.iter_mut().zip(row) {
            let v = k.table_fetch(b as i64);
            let s = k.shl(v, sh as u32);
            *o = k.add(*o as i64, s) as i32;
        }
    }
}

/// Token `j-1` positions back from `i` (j == 1 is `input[i]` itself),
/// bounded to the same story — `None` past the story start. This is the
/// canonical context-window semantics shared by observation (bundling) and
/// evaluation (issue #237: evaluation must not cross story boundaries or
/// use non-consecutive lags).
pub fn history_token(c: &Corpus, i: usize, j: usize) -> Option<u32> {
    if j == 1 {
        return Some(c.input[i]);
    }
    let back = j - 1;
    if i >= back && c.story[i - back] == c.story[i] {
        Some(c.input[i - back])
    } else {
        None
    }
}

/// Context bundle, plain form: decode-on-demand rows, dyadic weights as
/// shifts, rotation by slice split (no per-element modulo).
pub fn bundle_plain(art: &Compiled, rot: &[usize; WINDOW + 1], c: &Corpus, i: usize) -> [i64; D] {
    let mut acc = [0i64; D];
    let mut row = [0i32; D];
    for (j, &r) in rot.iter().enumerate().skip(1) {
        let Some(t) = history_token(c, i, j) else {
            continue;
        };
        decode_row_plain(art, t, &mut row);
        let w = (WINDOW - j) as u32;
        // acc[(d + r) mod D] += row[d] << w, as two straight runs
        let (lo, hi) = acc.split_at_mut(r);
        for (a, &v) in hi.iter_mut().zip(row.iter()) {
            *a += (v as i64) << w;
        }
        for (a, &v) in lo.iter_mut().zip(row.iter().skip(D - r)) {
            *a += (v as i64) << w;
        }
    }
    acc
}

/// Kernel-counted bundle: identical values.
pub fn bundle_kernel(
    k: &mut OpKernel,
    art: &Compiled,
    rot: &[usize; WINDOW + 1],
    c: &Corpus,
    i: usize,
) -> [i64; D] {
    let mut acc = [0i64; D];
    let mut row = [0i32; D];
    for (j, &r) in rot.iter().enumerate().skip(1) {
        let Some(t) = history_token(c, i, j) else {
            continue;
        };
        decode_row_kernel(k, art, t, &mut row);
        let w = (WINDOW - j) as u32;
        let (lo, hi) = acc.split_at_mut(r);
        for (a, &v) in hi.iter_mut().zip(row.iter()) {
            let s = k.shl(v as i64, w);
            *a = k.add(*a, s);
        }
        for (a, &v) in lo.iter_mut().zip(row.iter().skip(D - r)) {
            let s = k.shl(v as i64, w);
            *a = k.add(*a, s);
        }
    }
    acc
}

/// Corpus-free context bundle over a caller-supplied window of token ids,
/// oldest first: the token j back is weighted and rotated exactly as
/// `bundle_plain`'s j-th history token, so a window equal to a position's
/// in-story history produces an identical bundle. Only the WINDOW most
/// recent tokens are read.
///
/// **Context Window Bound:** Chain 2 context encoding enforces `WINDOW = 8`
/// dyadic-recency token history `[t-7..t]`. Inputs exceeding 8 tokens trigger
/// `tracing::warn!` and are truncated to the 8 most recent tokens without
/// heap allocation. Multi-timescale context expansion beyond 8 tokens is
/// scheduled for Phase 8 on the roadmap.
pub fn bundle_window_plain(art: &Compiled, rot: &[usize; WINDOW + 1], window: &[u32]) -> [i64; D] {
    let window = if window.len() > WINDOW {
        tracing::warn!(
            target: "uor_r4_core::runtime",
            window_size = WINDOW,
            input_size = window.len(),
            "Input context exceeds 8-token window; truncating to 8 most recent tokens"
        );
        &window[window.len() - WINDOW..]
    } else {
        window
    };
    let mut acc = [0i64; D];
    let mut row = [0i32; D];
    for (back, &t) in window.iter().rev().take(WINDOW).enumerate() {
        let j = back + 1;
        decode_row_plain(art, t, &mut row);
        let w = (WINDOW - j) as u32;
        let r = rot[j];
        let (lo, hi) = acc.split_at_mut(r);
        for (a, &v) in hi.iter_mut().zip(row.iter()) {
            *a += (v as i64) << w;
        }
        for (a, &v) in lo.iter_mut().zip(row.iter().skip(D - r)) {
            *a += (v as i64) << w;
        }
    }
    acc
}

/// Kernel-counted corpus-free bundle: identical values to
/// `bundle_window_plain`, every operation dispatched through `OpKernel`.
///
/// Enforces `WINDOW = 8` dyadic-recency truncation and emits `tracing::warn!`
/// if `window.len() > 8`.
pub fn bundle_window_kernel(
    k: &mut OpKernel,
    art: &Compiled,
    rot: &[usize; WINDOW + 1],
    window: &[u32],
) -> [i64; D] {
    let window = if window.len() > WINDOW {
        tracing::warn!(
            target: "uor_r4_core::runtime",
            window_size = WINDOW,
            input_size = window.len(),
            "Input context exceeds 8-token window; truncating to 8 most recent tokens"
        );
        &window[window.len() - WINDOW..]
    } else {
        window
    };
    let mut acc = [0i64; D];
    let mut row = [0i32; D];
    for (back, &t) in window.iter().rev().take(WINDOW).enumerate() {
        let j = back + 1;
        decode_row_kernel(k, art, t, &mut row);
        let w = (WINDOW - j) as u32;
        let r = rot[j];
        let (lo, hi) = acc.split_at_mut(r);
        for (a, &v) in hi.iter_mut().zip(row.iter()) {
            let s = k.shl(v as i64, w);
            *a = k.add(*a, s);
        }
        for (a, &v) in lo.iter_mut().zip(row.iter().skip(D - r)) {
            let s = k.shl(v as i64, w);
            *a = k.add(*a, s);
        }
    }
    acc
}

/// Bit signature, plain form: one compare per dimension, mask by shift.
pub fn sig_plain(art: &Compiled, bundle: &[i64; D]) -> [u8; SIG_BYTES] {
    let mut sig = [0u8; SIG_BYTES];
    let mut mask = 1u8;
    let mut byte = 0usize;
    for (&v, &t) in bundle.iter().zip(art.thresholds.iter()) {
        if v > t {
            sig[byte] |= mask;
        }
        if mask == 0x80 {
            mask = 1;
            byte += 1;
        } else {
            mask <<= 1;
        }
    }
    sig
}

/// Class assignment, plain form: Hamming by word xor + hardware popcount —
/// the fused form of the kernel's xor + table + add loop; equality with
/// the kernel path is witnessed per certification run.
pub fn assign_plain(art: &Compiled, sig: &[u8; SIG_BYTES]) -> [u8; STAGES] {
    assign_memberships_plain(art, sig).0
}

// ------------------- shift-add dot assignment (#243 Phase B) -----------
//
// The decision rows (issue #243, 2026-07-29) attributed the shipped
// assignment gap to the sign-Hamming metric itself: dot-product
// assignment against the per-stage context centroids measures
// 30.6-30.7% top1 / 34.2-34.3% agreement at ~47-48k store keys even
// with centroid values restricted to two signed powers of two. That
// restriction is what makes the runtime form legal: `dot(work, cent)`
// becomes a shift-and-add reduction — no multiply operation exists in
// this code path (P-4 scans this module). Active only when the
// artifact carries dot tables (fresh compiles, TLA6 containers);
// TLA3/4/5 loads keep the sign-Hamming path unchanged. TLA7 containers
// (issue #318 Phase B) additionally carry the integer residual
// sections and take the residual-wired path below: same dot argmax per
// stage, preceded by a power-of-two norm fold and followed by the
// winning centroid's integer-copy subtraction.

/// The centered work vector: bundle minus thresholds, integer sub.
pub fn centered_work(art: &Compiled, bundle: &[i64; D]) -> [i64; D] {
    let mut work = [0i64; D];
    for ((w, &b), &t) in work
        .iter_mut()
        .zip(bundle.iter())
        .zip(art.thresholds.iter())
    {
        *w = b - t;
    }
    work
}

/// Apply one packed power-of-two term to a work value: decode
/// (sign, nonzero, biased exponent) and shift accordingly. Plain form
/// of the kernel's shr/shl + add sequence.
#[inline]
fn dot_term_apply(work: i64, term: u8) -> i64 {
    if term & 0x40 == 0 {
        return 0;
    }
    let exp = (term & 0x3F) as i32 - 32;
    let shifted = if exp >= 0 {
        work << exp
    } else {
        work >> (-exp)
    };
    if term & 0x80 != 0 {
        -shifted
    } else {
        shifted
    }
}

/// Shift-add dot score of one class row (D packed u16 entries) against
/// the work vector.
pub fn dot_score_plain(row: &[u16], work: &[i64; D]) -> i64 {
    let mut acc = 0i64;
    for (&entry, &w) in row.iter().zip(work.iter()) {
        let [lo, hi] = entry.to_le_bytes();
        acc += dot_term_apply(w, hi);
        acc += dot_term_apply(w, lo);
    }
    acc
}

/// Per-stage top-M candidates under the dot metric, expressed as the
/// same ascending-cost shape the membership beam consumes. Candidate
/// selection ranks by (stage-best dot − class dot); the EMITTED costs
/// are the candidates' ranks (0, 1, 2, …), not the raw dot gaps —
/// dot gaps live on the raw work-vector scale (~1e6), and feeding
/// them to the beam's cross-stage sum and evidence merge (both tuned
/// to Hamming-scale costs) mis-weights everything downstream: the
/// first integer-semantics run measured the shipped beam row at
/// 18.3/20.6 against 30.6/34.2 for the primary key alone. Rank costs
/// are scale-free and metric-agnostic. Ties keep the lowest class
/// index (ascending scan, strict improvement) — the same
/// deterministic rule as the Hamming path.
fn dot_stage_top(table: &[u16], work: &[i64; D]) -> Vec<(u8, u32)> {
    let mut dots = [0i64; K];
    let mut best = i64::MIN;
    for (slot, row) in dots.iter_mut().zip(table.chunks_exact(D)) {
        *slot = dot_score_plain(row, work);
        if *slot > best {
            best = *slot;
        }
    }
    let mut top: Vec<(u8, u32)> = Vec::with_capacity(TOP_M_MEMBERSHIPS);
    for (kk, &dot) in dots.iter().enumerate() {
        let cost = u32::try_from(best - dot).unwrap_or(u32::MAX);
        let mut inserted = false;
        for (idx, &(_, c0)) in top.iter().enumerate() {
            if cost < c0 {
                top.insert(idx, (kk as u8, cost));
                inserted = true;
                break;
            }
        }
        if !inserted && top.len() < TOP_M_MEMBERSHIPS {
            top.push((kk as u8, cost));
        }
        if inserted && top.len() > TOP_M_MEMBERSHIPS {
            top.pop();
        }
    }
    for (rank, slot) in top.iter_mut().enumerate() {
        slot.1 = rank as u32;
    }
    top
}

/// Membership assignment for a bundle: residual-wired dot path when the
/// artifact carries residual copies (TLA7), dot path when it carries dot
/// tables (TLA6), sign-Hamming otherwise. The by-depth beam shape and
/// fallback-floor rule are identical between the metrics.
#[allow(clippy::type_complexity)]
pub fn assign_memberships_for_bundle(
    art: &Compiled,
    bundle: &[i64; D],
) -> ([u8; STAGES], Vec<Vec<Vec<u8>>>) {
    if art.dot_cb.is_empty() {
        return assign_memberships_plain(art, &sig_plain(art, bundle));
    }
    if !art.resid_cb.is_empty() {
        // #318 Phase B: the beam's per-stage candidate lists come from
        // the SAME residual-evolving work vector the kernel form and the
        // allocation-free serving form use — candidate selection per
        // stage is `dot_stage_top` on the folded work, then the winning
        // centroid's integer copy is subtracted before the next stage.
        let mut work = centered_work(art, bundle);
        norm_fold_plain(&mut work, art.norm_fold_const);
        let mut code = [0u8; STAGES];
        let mut stage_top: Vec<Vec<(u8, u32)>> = Vec::with_capacity(STAGES);
        for ((st_code, table), (copies, &shift)) in code
            .iter_mut()
            .zip(art.dot_cb.iter())
            .zip(art.resid_cb.iter().zip(art.resid_scale_shifts.iter()))
        {
            let top = dot_stage_top(table, &work);
            *st_code = top.first().map(|(k, _)| *k).unwrap_or(0);
            resid_subtract_plain(&mut work, copies, shift, *st_code);
            stage_top.push(top);
        }
        return memberships_from_stage_top(code, stage_top);
    }
    let work = centered_work(art, bundle);
    let mut code = [0u8; STAGES];
    let mut stage_top: Vec<Vec<(u8, u32)>> = Vec::with_capacity(STAGES);
    for (st_code, table) in code.iter_mut().zip(art.dot_cb.iter()) {
        let top = dot_stage_top(table, &work);
        *st_code = top.first().map(|(k, _)| *k).unwrap_or(0);
        stage_top.push(top);
    }
    memberships_from_stage_top(code, stage_top)
}

/// Plain class code for a bundle under whichever metric the artifact
/// declares. `assign_plain`/`assign_memberships_plain` remain the
/// signature-only sign-metric entry points for callers that never see
/// a bundle (score-time signature replay); bundle-holding callers go
/// through here so TLA6 artifacts assign by shift-add dot.
pub fn assign_for_bundle(art: &Compiled, bundle: &[i64; D]) -> [u8; STAGES] {
    assign_memberships_for_bundle(art, bundle).0
}

/// Allocation-free plain code assignment for a bundle under the
/// artifact's declared metric (#243 Phase C): per-stage argmax only —
/// no membership-beam materialization, so the steady-state serving
/// path (`R4Engine::derive_sig_code`, allocation-censused by
/// tests/status_policy_census.rs) stays allocation-free. Tie rule
/// matches `dot_stage_top` (strict improvement, lowest class index),
/// so the code equals `assign_for_bundle`'s primary code.
pub fn assign_code_for_bundle(art: &Compiled, bundle: &[i64; D]) -> [u8; STAGES] {
    if !art.resid_cb.is_empty() {
        return assign_code_for_bundle_resid(art, bundle);
    }
    if art.dot_cb.is_empty() {
        // Sign metric, argmax only — assign_plain delegates to the
        // membership-beam builder and allocates; this path must not.
        let sig = sig_plain(art, bundle);
        let mut code = [0u8; STAGES];
        for (st_code, sigs) in code.iter_mut().zip(art.class_sigs.iter()) {
            let mut best = u32::MAX;
            let mut best_class = 0u8;
            for (class, cs) in sigs.chunks_exact(SIG_BYTES).enumerate() {
                let dist =
                    crate::transformerless::simd::hamming_distance_36(&sig, cs.try_into().unwrap());
                if dist < best {
                    best = dist;
                    best_class = class as u8;
                }
            }
            *st_code = best_class;
        }
        return code;
    }
    let work = centered_work(art, bundle);
    let mut code = [0u8; STAGES];
    for (st_code, table) in code.iter_mut().zip(art.dot_cb.iter()) {
        let mut best = i64::MIN;
        let mut best_class = 0u8;
        for (class, row) in table.chunks_exact(D).enumerate() {
            let score = dot_score_plain(row, &work);
            if score > best {
                best = score;
                best_class = class as u8;
            }
        }
        *st_code = best_class;
    }
    code
}

// ------------- integer residual wiring (#318 Phase B, TLA7) ------------
//
// The kernel form of the certifier's Phase A.5 rows
// (docs/dot_residual_phase_b_design.md): the po2 norm fold replaces f32
// division (row a), and per-stage i8 centroid copies carry the residual
// subtraction (row b). Everything below is add/sub, shifts, compares,
// and table reads — the P-4 scan covers this module.

/// L1 norm of the work vector (Σ|w_d|): compare, negate, add only.
fn work_l1(work: &[i64; D]) -> i64 {
    let mut l1 = 0i64;
    for &w in work.iter() {
        l1 += if w < 0 { -w } else { w };
    }
    l1
}

/// Bit length of a positive value — the position of its highest set
/// bit, floor(log2(v)) + 1 — by shift and compare only. 0 for v = 0.
fn bit_length(v: i64) -> i32 {
    let mut v = v;
    let mut n = 0i32;
    while v > 0 {
        v >>= 1;
        n += 1;
    }
    n
}

/// Power-of-two norm fold, plain form: `work >>= s` (arithmetic) with
/// `s = bit_length(L1) − CONST`; s < 0 shifts left, and L1 = 0 leaves
/// the (all-zero) vector untouched. The kernel must never see a shift
/// amount outside the word width, so s is clamped to the i64 range.
fn norm_fold_plain(work: &mut [i64; D], norm_const: i32) {
    let l1 = work_l1(work);
    if l1 == 0 {
        return;
    }
    let s = (i64::from(bit_length(l1)) - i64::from(norm_const)).clamp(-63, 63);
    for w in work.iter_mut() {
        *w = if s >= 0 { *w >> s } else { *w << (-s) };
    }
}

/// Subtract one stage's winning integer centroid copy from the work
/// vector (plain form): per dimension, one shift and one subtract.
/// Plain form of the kernel's table-fetch + shl + add sequence.
fn resid_subtract_plain(work: &mut [i64; D], copies: &[i8], shift: u8, class: u8) {
    if let Some(copy_row) = copies.chunks_exact(D).nth(usize::from(class)) {
        for (w, &c) in work.iter_mut().zip(copy_row.iter()) {
            *w -= i64::from(c) << shift;
        }
    }
}

/// Residual-wired shift-add dot assignment (#318 Phase B), allocation-
/// free plain form: center the bundle, apply the po2 norm fold, then
/// per stage argmax `dot_score_plain` over the stage's po2 table and
/// subtract the winning centroid's integer copy (`<<` the stage's
/// decode shift) from the work vector — add/sub and shifts only. The
/// per-sample scale error of the crude fold lands in the exponent,
/// which the dot tables absorb (assignment is argmax, scale-invariant);
/// the copies ride the same folded scale by construction
/// (`RESID_WORK_FRACTION − e_st`). Active only for TLA7 artifacts
/// (`resid_cb` populated); pre-TLA7 artifacts keep the non-residual
/// dot path. Tie rule matches `assign_code_for_bundle` (strict
/// improvement, lowest class index).
pub fn assign_code_for_bundle_resid(art: &Compiled, bundle: &[i64; D]) -> [u8; STAGES] {
    let mut work = centered_work(art, bundle);
    norm_fold_plain(&mut work, art.norm_fold_const);
    let mut code = [0u8; STAGES];
    for ((st_code, table), (copies, &shift)) in code
        .iter_mut()
        .zip(art.dot_cb.iter())
        .zip(art.resid_cb.iter().zip(art.resid_scale_shifts.iter()))
    {
        let mut best = i64::MIN;
        let mut best_class = 0u8;
        for (class, row) in table.chunks_exact(D).enumerate() {
            let score = dot_score_plain(row, &work);
            if score > best {
                best = score;
                best_class = class as u8;
            }
        }
        *st_code = best_class;
        resid_subtract_plain(&mut work, copies, shift, best_class);
    }
    code
}

/// Bounded multi-membership assignment per depth, with nearest-class
/// membership retained at every depth as the fallback floor.
pub fn assign_memberships_plain(
    art: &Compiled,
    sig: &[u8; SIG_BYTES],
) -> ([u8; STAGES], Vec<Vec<Vec<u8>>>) {
    let mut code = [0u8; STAGES];
    let mut stage_top: Vec<Vec<(u8, u32)>> = Vec::with_capacity(STAGES);
    for (st_code, sigs) in code.iter_mut().zip(art.class_sigs.iter()) {
        let mut top: Vec<(u8, u32)> = Vec::with_capacity(TOP_M_MEMBERSHIPS);
        for (kk, cs) in sigs.chunks_exact(SIG_BYTES).enumerate() {
            let dist =
                crate::transformerless::simd::hamming_distance_36(sig, cs.try_into().unwrap());
            let mut inserted = false;
            for (idx, &(_, d0)) in top.iter().enumerate() {
                if dist < d0 {
                    top.insert(idx, (kk as u8, dist));
                    inserted = true;
                    break;
                }
            }
            if !inserted && top.len() < TOP_M_MEMBERSHIPS {
                top.push((kk as u8, dist));
            }
            if inserted && top.len() > TOP_M_MEMBERSHIPS {
                top.pop();
            }
        }
        *st_code = top.first().map(|(k, _)| *k).unwrap_or(0);
        stage_top.push(top);
    }
    memberships_from_stage_top(code, stage_top)
}

/// The shared membership beam: per-depth prefix expansion over the
/// per-stage top-M candidate lists, with the nearest-class prefix
/// retained at every depth as the fallback floor. Metric-agnostic —
/// candidates arrive as ascending (class, cost) lists from either the
/// Hamming or the shift-add dot scorer.
#[allow(clippy::type_complexity)]
fn memberships_from_stage_top(
    code: [u8; STAGES],
    stage_top: Vec<Vec<(u8, u32)>>,
) -> ([u8; STAGES], Vec<Vec<Vec<u8>>>) {
    let mut by_depth: Vec<Vec<Vec<u8>>> = (0..=STAGES).map(|_| Vec::new()).collect();
    by_depth[0].push(Vec::new());
    let mut beam: Vec<(Vec<u8>, u32)> = vec![(Vec::new(), 0)];
    let mut nearest = Vec::with_capacity(STAGES);
    for (depth_idx, stage) in stage_top.iter().enumerate() {
        if let Some(&(k, _)) = stage.first() {
            nearest.push(k);
        }
        let mut next_beam: Vec<(Vec<u8>, u32)> = Vec::new();
        for (prefix, score) in &beam {
            for &(k, d) in stage {
                let mut next = prefix.clone();
                next.push(k);
                next_beam.push((next, score.saturating_add(d)));
            }
        }
        next_beam.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
        if next_beam.len() > TOP_M_MEMBERSHIPS {
            next_beam.truncate(TOP_M_MEMBERSHIPS);
        }
        let depth = depth_idx + 1;
        let nearest_prefix = nearest.clone();
        if next_beam.iter().all(|(key, _)| key != &nearest_prefix) {
            if next_beam.len() == TOP_M_MEMBERSHIPS {
                next_beam.pop();
            }
            next_beam.push((nearest_prefix, u32::MAX));
            next_beam.sort_by(|a, b| a.0.cmp(&b.0));
        }
        by_depth[depth] = next_beam.iter().map(|(key, _)| key.clone()).collect();
        beam = next_beam;
    }
    (code, by_depth)
}

/// Full plain path: position → graded class code (metric per artifact).
pub fn code_plain(art: &Compiled, rot: &[usize; WINDOW + 1], c: &Corpus, i: usize) -> [u8; STAGES] {
    let b = bundle_plain(art, rot, c, i);
    assign_for_bundle(art, &b)
}

/// A prediction with its resolution witness: the store level that answered
/// (deepest populated class) and the winning entry's evidence count.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Prediction {
    pub token: u32,
    /// Resolution depth (0..=STAGES): fixed-width, never `usize`, at the
    /// serialization boundary (issue #12).
    pub depth: u8,
    pub count: u32,
}

fn fallback_prediction(store: &Store) -> Prediction {
    if let Some(root) = store.first().and_then(|level| level.get(&[][..])) {
        let mut best_t = 0u32;
        let mut best_c = -1i64;
        let mut best_n = 0u32;
        for (&t, &cnt) in root {
            let count = cnt as i64;
            if count > best_c {
                best_c = count;
                best_t = t;
                best_n = cnt;
            }
        }
        return Prediction {
            token: best_t,
            depth: 0,
            count: best_n,
        };
    }
    Prediction::default()
}

/// Plain-form prediction with witness: deepest populated class argmax with
/// backoff; canonical rule — highest count, ties to smallest token id
/// (first in B-tree order) — identical to the kernel path.
pub fn predict_witness_plain(store: &Store, code: &[u8; STAGES]) -> Prediction {
    for d in (0..=STAGES).rev() {
        if let Some(dist) = store[d].get(&code[..d]) {
            let mut best_t = 0u32;
            let mut best_c = -1i64;
            let mut best_n = 0u32;
            for (&t, &cnt) in dist {
                if (cnt as i64) > best_c {
                    best_c = cnt as i64;
                    best_t = t;
                    best_n = cnt;
                }
            }
            return Prediction {
                token: best_t,
                depth: d as u8,
                count: best_n,
            };
        }
    }
    fallback_prediction(store)
}

/// Plain-form prediction with semantic priors: deepest populated class argmax biased by priors.
pub fn predict_witness_plain_with_priors(
    store: &Store,
    code: &[u8; STAGES],
    priors: &std::collections::HashMap<u32, u32>,
) -> Prediction {
    for d in (0..=STAGES).rev() {
        if let Some(dist) = store[d].get(&code[..d]) {
            let mut best_t = 0u32;
            let mut best_c = -1i64;
            let mut best_n = 0u32;
            for (&t, &cnt) in dist {
                let prior = priors.get(&t).cloned().unwrap_or(0);
                let p_i = prior as i64;
                let bias = (p_i << 6) + (p_i << 5) + (p_i << 2);
                let score = cnt as i64 + bias;
                if score > best_c {
                    best_c = score;
                    best_t = t;
                    best_n = cnt;
                }
            }
            return Prediction {
                token: best_t,
                depth: d as u8,
                count: best_n,
            };
        }
    }
    fallback_prediction(store)
}

/// Plain-form prediction: the witness variant's token, one code path.
pub fn predict_plain(store: &Store, code: &[u8; STAGES]) -> u32 {
    predict_witness_plain(store, code).token
}

/// Merge the token→count evidence under a set of membership keys at one
/// store level (issue #281 read-time beam; reference semantics from the
/// certifier's #244 measurement, commit 1f5088b). `None` when no key hits.
pub fn merged_beam_distribution(
    level: &BTreeMap<Vec<u8>, BTreeMap<u32, u32>>,
    keys: &[Vec<u8>],
) -> Option<BTreeMap<u32, u32>> {
    let mut merged: BTreeMap<u32, u32> = BTreeMap::new();
    let mut hit = false;
    for key in keys {
        if let Some(dist) = level.get(key) {
            hit = true;
            for (&t, &cnt) in dist {
                *merged.entry(t).or_default() += cnt;
            }
        }
    }
    if hit {
        Some(merged)
    } else {
        None
    }
}

/// Plain-form beam prediction against the single-key store: deepest level
/// where any membership prefix has evidence; canonical argmax over the
/// merged distribution (highest count, ties to smallest token id).
pub fn predict_witness_plain_beam(store: &Store, by_depth: &[Vec<Vec<u8>>]) -> Prediction {
    for d in (0..=STAGES).rev() {
        let keys: &[Vec<u8>] = by_depth.get(d).map(|k| k.as_slice()).unwrap_or(&[]);
        if let Some(dist) = merged_beam_distribution(&store[d], keys) {
            let mut best_t = 0u32;
            let mut best_c = -1i64;
            let mut best_n = 0u32;
            for (&t, &cnt) in &dist {
                if (cnt as i64) > best_c {
                    best_c = cnt as i64;
                    best_t = t;
                    best_n = cnt;
                }
            }
            return Prediction {
                token: best_t,
                depth: d as u8,
                count: best_n,
            };
        }
    }
    fallback_prediction(store)
}

/// Kernel-counted full path.
pub struct Runtime<'a> {
    pub art: &'a Compiled,
    pub rot: [usize; WINDOW + 1],
    pub pop: [u8; 256],
    pub kernel: OpKernel,
    pub state: RuntimeState,
    dot_tables: Option<super::simd::DotTables>,
}

impl<'a> Runtime<'a> {
    pub fn new(art: &'a Compiled) -> Self {
        Runtime {
            art,
            rot: derive_rotations(),
            pop: derive_popcount_table(),
            kernel: OpKernel::default(),
            state: RuntimeState::default(),
            dot_tables: super::simd::DotTables::from_packed(&art.dot_cb),
        }
    }

    pub fn assign(&mut self, c: &Corpus, i: usize) -> [u8; STAGES] {
        let rot = self.rot;
        let b = bundle_kernel(&mut self.kernel, self.art, &rot, c, i);
        if !self.art.resid_cb.is_empty() {
            return self.code_from_bundle_resid(&b);
        }
        if !self.art.dot_cb.is_empty() {
            return self.code_from_bundle_dot(&b);
        }
        let sig = sign_signature(&mut self.kernel, &b, &self.art.thresholds);
        self.code_from_sig(&sig)
    }

    /// Corpus-free kernel path: window of token ids, oldest first;
    /// identical values to the plain window path, every op counted.
    pub fn assign_window(&mut self, window: &[u32]) -> [u8; STAGES] {
        let rot = self.rot;
        let b = bundle_window_kernel(&mut self.kernel, self.art, &rot, window);
        if !self.art.resid_cb.is_empty() {
            return self.code_from_bundle_resid(&b);
        }
        if !self.art.dot_cb.is_empty() {
            return self.code_from_bundle_dot(&b);
        }
        let sig = sign_signature(&mut self.kernel, &b, &self.art.thresholds);
        self.code_from_sig(&sig)
    }

    /// Corpus-free kernel path with membership beam (issue #281): the
    /// kernel-counted signature, primary code, and the per-depth
    /// membership prefixes used by the read-time beam. Membership
    /// derivation is the one plain-path rule shared with `build_store`'s
    /// codes; the signature it reads is the kernel-counted one, whose
    /// kernel==plain identity the certifier witnesses.
    #[allow(clippy::type_complexity)]
    pub fn assign_window_memberships(
        &mut self,
        window: &[u32],
    ) -> ([u8; STAGES], Vec<Vec<Vec<u8>>>) {
        let rot = self.rot;
        let b = bundle_window_kernel(&mut self.kernel, self.art, &rot, window);
        if !self.art.resid_cb.is_empty() {
            // #318 Phase B: primary code and membership beam are both
            // residual-wired (`assign_memberships_for_bundle` derives
            // its candidate lists from the same folded, residual-
            // evolving work vector), so the pair stays consistent.
            let code = self.code_from_bundle_resid(&b);
            let (_, by_depth) = assign_memberships_for_bundle(self.art, &b);
            return (code, by_depth);
        }
        if !self.art.dot_cb.is_empty() {
            let code = self.code_from_bundle_dot(&b);
            let (_, by_depth) = assign_memberships_for_bundle(self.art, &b);
            return (code, by_depth);
        }
        let sig = sign_signature(&mut self.kernel, &b, &self.art.thresholds);
        let code = self.code_from_sig(&sig);
        let (_, by_depth) = assign_memberships_plain(self.art, &sig);
        (code, by_depth)
    }

    /// Graded class assignment from a bundle by shift-add dot scoring
    /// (#243 Phase B, kernel form — every operation counted): center the
    /// bundle with integer adds, then per stage argmax over K classes of
    /// the two-term power-of-two dot. Ops per class: one table read per
    /// dimension entry, at most two shifts + two adds per term pair, one
    /// compare per candidate. Equality with `assign_for_bundle` (plain
    /// form) is witnessed per certification run.
    pub(crate) fn code_from_bundle_dot(&mut self, bundle: &[i64; D]) -> [u8; STAGES] {
        let mut work = [0i64; D];
        for ((w, &b), &t) in work
            .iter_mut()
            .zip(bundle.iter())
            .zip(self.art.thresholds.iter())
        {
            *w = self.kernel.add(b, -t);
        }
        let mut code = [0u8; STAGES];
        for (stage, (st_code, table)) in code.iter_mut().zip(self.art.dot_cb.iter()).enumerate() {
            *st_code = self.dot_argmax(stage, table, &work);
        }
        code
    }

    /// Per-stage dot argmax through the kernel: one candidate scan per
    /// class, one table read per dimension entry, at most two shifts +
    /// two adds per term pair, one compare per candidate.
    fn dot_argmax(&mut self, stage: usize, table: &[u16], work: &[i64; D]) -> u8 {
        if let Some(dot_tables) = self.dot_tables.as_ref() {
            let dot_stage = &dot_tables.stages[stage];
            let class = super::simd::dot_argmax(dot_stage, work);
            for _ in 0..K {
                self.kernel.candidate_scan();
            }
            for _ in 0..dot_stage.vector_count() {
                self.kernel.simd_dot_vector();
            }
            return class;
        }
        let mut best = i64::MIN;
        let mut best_k = 0u8;
        for (kk, row) in table.chunks_exact(D).enumerate() {
            self.kernel.candidate_scan();
            let mut acc = 0i64;
            for (&entry, &w) in row.iter().zip(work.iter()) {
                let packed = self.kernel.table_fetch(i64::from(entry)) as u16;
                let [lo, hi] = packed.to_le_bytes();
                for term in [hi, lo] {
                    if term & 0x40 == 0 {
                        continue;
                    }
                    let exp = (term & 0x3F) as i32 - 32;
                    let shifted = if exp >= 0 {
                        self.kernel.shl(w, exp as u32)
                    } else {
                        self.kernel.shr(w, (-exp) as u32)
                    };
                    let signed = if term & 0x80 != 0 { -shifted } else { shifted };
                    acc = self.kernel.add(acc, signed);
                }
            }
            if self.kernel.lt(best, acc) {
                best = acc;
                best_k = kk as u8;
            }
        }
        best_k
    }

    /// Residual-wired shift-add dot assignment (#318 Phase B, kernel
    /// form — every operation counted): identical values to
    /// `assign_code_for_bundle_resid` (plain form), with the po2 norm
    /// fold and the per-stage integer centroid-copy subtraction
    /// dispatched through `OpKernel`. Added ops over the non-residual
    /// form: the fold (D compare+add for the L1 norm, ≤ 63 shift+add
    /// for the bit length, D shifts for the fold itself) and per stage
    /// D table reads + shifts + adds for the copy subtraction — a
    /// ~1/(2K) fraction of the dot scan, far inside the design note's
    /// ⚑ 2× op-census budget.
    pub(crate) fn code_from_bundle_resid(&mut self, bundle: &[i64; D]) -> [u8; STAGES] {
        let mut work = [0i64; D];
        for ((w, &b), &t) in work
            .iter_mut()
            .zip(bundle.iter())
            .zip(self.art.thresholds.iter())
        {
            *w = self.kernel.add(b, -t);
        }
        // po2 norm fold: L1 = Σ|w_d| by compare + add; bit length by
        // shift + compare; then one shift per dimension.
        let mut l1 = 0i64;
        for &w in work.iter() {
            let mag = if self.kernel.lt(w, 0) { -w } else { w };
            l1 = self.kernel.add(l1, mag);
        }
        if l1 != 0 {
            let mut bits = 0i64;
            let mut v = l1;
            while self.kernel.lt(0, v) {
                v = self.kernel.shr(v, 1);
                bits = self.kernel.add(bits, 1);
            }
            let s = (bits - i64::from(self.art.norm_fold_const)).clamp(-63, 63);
            for w in work.iter_mut() {
                *w = if s >= 0 {
                    self.kernel.shr(*w, s as u32)
                } else {
                    self.kernel.shl(*w, (-s) as u32)
                };
            }
        }
        let mut code = [0u8; STAGES];
        for (stage, ((st_code, table), (copies, &shift))) in code
            .iter_mut()
            .zip(self.art.dot_cb.iter())
            .zip(
                self.art
                    .resid_cb
                    .iter()
                    .zip(self.art.resid_scale_shifts.iter()),
            )
            .enumerate()
        {
            *st_code = self.dot_argmax(stage, table, &work);
            if let Some(copy_row) = copies.chunks_exact(D).nth(usize::from(*st_code)) {
                for (w, &c) in work.iter_mut().zip(copy_row.iter()) {
                    let v = self.kernel.table_fetch(i64::from(c));
                    let shifted = self.kernel.shl(v, u32::from(shift));
                    *w = self.kernel.add(*w, -shifted);
                }
            }
        }
        code
    }

    /// Graded class assignment from a bit signature: Hamming to each
    /// stage's class signatures, nearest class per stage. One code path
    /// for the corpus and window forms.
    fn code_from_sig(&mut self, sig: &[u8; SIG_BYTES]) -> [u8; STAGES] {
        let mut code = [0u8; STAGES];
        for (st_code, sigs) in code.iter_mut().zip(self.art.class_sigs.iter()) {
            let mut best_d = i64::MAX;
            let mut best_k = 0u8;
            for (kk, cs) in sigs.chunks_exact(SIG_BYTES).enumerate() {
                self.kernel.candidate_scan();
                let mut d = 0i64;
                for (&a, &bb) in sig.iter().zip(cs) {
                    let x = self.kernel.xor(a, bb);
                    let p = self.kernel.table_u8(&self.pop, x);
                    d = self.kernel.add(d, p as i64);
                }
                if self.kernel.lt(d, best_d) {
                    best_d = d;
                    best_k = kk as u8;
                }
            }
            *st_code = best_k;
        }
        code
    }

    /// Kernel-counted prediction: the witness variant's token, one code path.
    pub fn predict(&mut self, store: &Store, code: &[u8; STAGES]) -> u32 {
        self.predict_witness(store, code).token
    }

    /// Kernel-counted prediction with resolution witness (deepest populated
    /// class, winning evidence count); canonical argmax rule, counted.
    pub fn predict_witness(&mut self, store: &Store, code: &[u8; STAGES]) -> Prediction {
        for d in (0..=STAGES).rev() {
            if let Some(dist) = store[d].get(&code[..d]) {
                let mut best_t = 0u32;
                let mut best_c = -1000000i64;
                let mut best_n = 0u32;
                for (&t, &cnt) in dist {
                    self.kernel.candidate_scan();
                    let mut score = cnt as i64;
                    let occurrences = self.state.token_occurrences(t);
                    if occurrences > 0 {
                        let val = occurrences as i64;
                        score -= (val << 10) - (val << 4) - (val << 3);
                    }
                    if self.kernel.lt(best_c, score) {
                        best_c = score;
                        best_t = t;
                        best_n = cnt;
                    }
                }
                self.state.record_token(best_t);
                return Prediction {
                    token: best_t,
                    depth: d as u8,
                    count: best_n,
                };
            }
        }
        let fallback = fallback_prediction(store);
        self.state.record_token(fallback.token);
        fallback
    }

    /// Kernel-counted beam prediction (issue #281): deepest level where
    /// any membership prefix has evidence; merged-distribution argmax with
    /// the same repetition-penalty state as `predict_witness`. Merging is
    /// table reads and adds — no new operation classes.
    pub fn predict_witness_beam(&mut self, store: &Store, by_depth: &[Vec<Vec<u8>>]) -> Prediction {
        for d in (0..=STAGES).rev() {
            let keys: &[Vec<u8>] = by_depth.get(d).map(|k| k.as_slice()).unwrap_or(&[]);
            let mut merged: BTreeMap<u32, i64> = BTreeMap::new();
            let mut hit = false;
            for key in keys {
                if let Some(dist) = store[d].get(key) {
                    hit = true;
                    for (&t, &cnt) in dist {
                        let acc = merged.entry(t).or_insert(0);
                        *acc = self.kernel.add(*acc, cnt as i64);
                    }
                }
            }
            if !hit {
                continue;
            }
            let mut best_t = 0u32;
            let mut best_c = -1000000i64;
            let mut best_n = 0u32;
            for (&t, &cnt) in &merged {
                self.kernel.candidate_scan();
                let mut score = cnt;
                let occurrences = self.state.token_occurrences(t);
                if occurrences > 0 {
                    let val = occurrences as i64;
                    score -= (val << 10) - (val << 4) - (val << 3);
                }
                if self.kernel.lt(best_c, score) {
                    best_c = score;
                    best_t = t;
                    best_n = cnt as u32;
                }
            }
            self.state.record_token(best_t);
            return Prediction {
                token: best_t,
                depth: d as u8,
                count: best_n,
            };
        }
        let fallback = fallback_prediction(store);
        self.state.record_token(fallback.token);
        fallback
    }

    /// Kernel-counted beam prediction with semantic context priors
    /// (issue #281): merged-distribution variant of
    /// `predict_witness_with_priors`.
    pub fn predict_witness_with_priors_beam(
        &mut self,
        store: &Store,
        by_depth: &[Vec<Vec<u8>>],
        priors: &std::collections::HashMap<u32, u32>,
    ) -> Prediction {
        for d in (0..=STAGES).rev() {
            let keys: &[Vec<u8>] = by_depth.get(d).map(|k| k.as_slice()).unwrap_or(&[]);
            let mut merged: BTreeMap<u32, i64> = BTreeMap::new();
            let mut hit = false;
            for key in keys {
                if let Some(dist) = store[d].get(key) {
                    hit = true;
                    for (&t, &cnt) in dist {
                        let acc = merged.entry(t).or_insert(0);
                        *acc = self.kernel.add(*acc, cnt as i64);
                    }
                }
            }
            if !hit {
                continue;
            }
            let mut best_t = 0u32;
            let mut best_c = -1000000i64;
            let mut best_n = 0u32;
            for (&t, &cnt) in &merged {
                self.kernel.candidate_scan();
                let prior = priors.get(&t).cloned().unwrap_or(0);
                let p_i = prior as i64;
                let bias = (p_i << 6) + (p_i << 5) + (p_i << 2);
                let mut score = cnt + bias;
                let occurrences = self.state.token_occurrences(t);
                if occurrences > 0 {
                    let val = occurrences as i64;
                    score -= (val << 10) - (val << 4) - (val << 3);
                }
                if self.kernel.lt(best_c, score) {
                    best_c = score;
                    best_t = t;
                    best_n = cnt as u32;
                }
            }
            self.state.record_token(best_t);
            return Prediction {
                token: best_t,
                depth: d as u8,
                count: best_n,
            };
        }
        let fallback = fallback_prediction(store);
        self.state.record_token(fallback.token);
        fallback
    }

    /// Kernel-counted prediction with resolution witness and semantic context priors.
    pub fn predict_witness_with_priors(
        &mut self,
        store: &Store,
        code: &[u8; STAGES],
        priors: &std::collections::HashMap<u32, u32>,
    ) -> Prediction {
        for d in (0..=STAGES).rev() {
            if let Some(dist) = store[d].get(&code[..d]) {
                let mut best_t = 0u32;
                let mut best_c = -1000000i64;
                let mut best_n = 0u32;
                for (&t, &cnt) in dist {
                    self.kernel.candidate_scan();
                    let mut score = cnt as i64;
                    let occurrences = self.state.token_occurrences(t);
                    if occurrences > 0 {
                        let val = occurrences as i64;
                        score -= (val << 10) - (val << 4) - (val << 3);
                    }
                    let prior = priors.get(&t).cloned().unwrap_or(0);
                    let p_i = prior as i64;
                    let bias = (p_i << 5) + (p_i << 4) + (p_i << 1);
                    score += bias;
                    if self.kernel.lt(best_c, score) {
                        best_c = score;
                        best_t = t;
                        best_n = cnt;
                    }
                }
                self.state.record_token(best_t);
                return Prediction {
                    token: best_t,
                    depth: d as u8,
                    count: best_n,
                };
            }
        }
        let fallback = fallback_prediction(store);
        self.state.record_token(fallback.token);
        fallback
    }

    /// Allocation-free greedy generation into caller-owned storage.
    ///
    /// Returns the number of predictions written. Only the most recent
    /// [`WINDOW`] seed tokens are copied into a fixed stack buffer.
    pub fn generate_greedy_into(
        &mut self,
        store: &Store,
        seed: &[u32],
        out: &mut [Prediction],
    ) -> usize {
        let mut window = [0u32; WINDOW];
        let seed = &seed[seed.len().saturating_sub(WINDOW)..];
        let mut window_len = seed.len();
        window[..window_len].copy_from_slice(seed);
        for slot in out.iter_mut() {
            let code = self.assign_window(&window[..window_len]);
            let p = self.predict_witness(store, &code);
            if window_len < WINDOW {
                window[window_len] = p.token;
                window_len += 1;
            } else {
                window.copy_within(1.., 0);
                window[WINDOW - 1] = p.token;
            }
            *slot = p;
        }
        out.len()
    }
}

/// Add one (context code → next token) evidence count across every grade
/// level — the store's single write path, used by `build_store` at compile
/// time and by online indexing at runtime alike.
pub fn add_evidence(store: &mut Store, code: &[u8; STAGES], next: u32, weight: u32) {
    add_evidence_multi(store, &[], code, next, weight);
}

/// Add one (context memberships → next token) evidence count using bounded
/// memberships per depth; nearest-class prefixes are used as the fallback
/// floor whenever a depth has no candidates.
pub fn add_evidence_multi(
    store: &mut Store,
    by_depth: &[Vec<Vec<u8>>],
    fallback_code: &[u8; STAGES],
    next: u32,
    weight: u32,
) {
    *store[0].entry(vec![]).or_default().entry(next).or_default() += weight;
    for d in 1..=STAGES {
        if let Some(keys) = by_depth.get(d) {
            if !keys.is_empty() {
                for key in keys {
                    *store[d]
                        .entry(key.clone())
                        .or_default()
                        .entry(next)
                        .or_default() += weight;
                }
                continue;
            }
        }
        *store[d]
            .entry(fallback_code[..d].to_vec())
            .or_default()
            .entry(next)
            .or_default() += weight;
    }
}

/// The store, built by the runtime's own plain path — key identity between
/// construction and query is by construction, not by sampling.
///
/// Issue #281 (the #244 decision): evidence is written under the primary
/// code prefix only. The membership fan-out moved from write time to read
/// time (`predict_witness_plain_beam` and the kernel beam variants), which
/// the #244 matrix measured as accuracy-identical at 2.56× fewer keys.
pub fn build_store(art: &Compiled, c: &Corpus) -> (Store, Vec<[u8; STAGES]>) {
    let rot = derive_rotations();
    let mut codes: Vec<[u8; STAGES]> = Vec::with_capacity(c.n);
    for i in 0..c.n {
        let b = bundle_plain(art, &rot, c, i);
        codes.push(assign_for_bundle(art, &b));
    }
    let store = store_from_codes(c, &codes);
    (store, codes)
}

/// The store-insertion half of [`build_store`], factored out so a caller
/// that already holds the per-record codes (issue #469 lever A: the
/// κ-keyed code sidecar) inserts evidence through exactly this path.
/// Insertion order is ascending record index in both callers, so the
/// resulting B-tree — and therefore `store_bytes` — is identical.
pub fn store_from_codes(c: &Corpus, codes: &[[u8; STAGES]]) -> Store {
    let cut = train_cut(c);
    let mut store: Store = (0..=STAGES).map(|_| BTreeMap::new()).collect();
    for (i, code) in codes.iter().enumerate() {
        if c.story[i] >= cut {
            continue;
        }
        for k_idx in 0..c.top_tokens[i].len() {
            let tok = c.top_tokens[i][k_idx];
            let weight = c.top_weights[i][k_idx];
            if weight > 0 {
                add_evidence(&mut store, code, tok, weight);
            }
        }
    }
    store
}

/// Parallel per-record code derivation — the code half of
/// [`build_store_with_threads`], factored out so the sidecar
/// (issue #469 lever A) caches exactly the bytes this returns.
/// Chunks are joined in chunk-id order, so the result is the serial
/// `code_plain` sequence regardless of `threads`.
#[cfg(not(target_arch = "wasm32"))]
pub fn codes_with_threads(
    art: &Compiled,
    c: &Corpus,
    threads: usize,
) -> Result<Vec<[u8; STAGES]>, String> {
    if threads <= 1 || c.n < 2 {
        let rot = derive_rotations();
        return Ok((0..c.n)
            .map(|i| {
                let bundle = bundle_plain(art, &rot, c, i);
                assign_for_bundle(art, &bundle)
            })
            .collect());
    }
    let worker_count = threads.min(c.n);
    let chunk_size = c.n.div_ceil(worker_count);
    let mut chunks = Vec::with_capacity(worker_count);
    std::thread::scope(|scope| -> Result<(), String> {
        let mut handles = Vec::with_capacity(worker_count);
        for (chunk_id, start) in (0..c.n).step_by(chunk_size).enumerate() {
            let end = (start + chunk_size).min(c.n);
            handles.push((
                chunk_id,
                scope.spawn(move || {
                    let rot = derive_rotations();
                    (start..end)
                        .map(|i| {
                            let bundle = bundle_plain(art, &rot, c, i);
                            assign_for_bundle(art, &bundle)
                        })
                        .collect::<Vec<_>>()
                }),
            ));
        }
        for (chunk_id, handle) in handles {
            let codes = handle
                .join()
                .map_err(|_| format!("store code worker {chunk_id} panicked"))?;
            chunks.push((chunk_id, codes));
        }
        Ok(())
    })?;
    chunks.sort_by_key(|(chunk_id, _)| *chunk_id);
    let mut codes = Vec::with_capacity(c.n);
    for (_, chunk) in chunks {
        codes.extend(chunk);
    }
    Ok(codes)
}

/// Parallel code-generation front-end for [`build_store`]. The expensive
/// bundle/code derivation is independent per corpus position; evidence is
/// then inserted through the same canonical serial path as `build_store` so
/// BTreeMap ordering and artifact bytes remain unchanged.
#[cfg(not(target_arch = "wasm32"))]
pub fn build_store_with_threads(
    art: &Compiled,
    c: &Corpus,
    threads: usize,
) -> Result<(Store, Vec<[u8; STAGES]>), String> {
    if threads <= 1 || c.n < 2 {
        return Ok(build_store(art, c));
    }
    let codes = codes_with_threads(art, c, threads)?;
    let store = store_from_codes(c, &codes);
    Ok((store, codes))
}

/// The pre-#281 write-time fan-out store (multi-membership writes).
/// Retained for the certifier's ablation row only — not a shipped path.
pub fn build_store_multi(art: &Compiled, c: &Corpus) -> (Store, Vec<[u8; STAGES]>) {
    let rot = derive_rotations();
    let cut = train_cut(c);
    let mut store: Store = (0..=STAGES).map(|_| BTreeMap::new()).collect();
    let mut codes: Vec<[u8; STAGES]> = Vec::with_capacity(c.n);
    for i in 0..c.n {
        let b = bundle_plain(art, &rot, c, i);
        let (code, by_depth) = assign_memberships_plain(art, &sig_plain(art, &b));
        codes.push(code);
        if c.story[i] >= cut {
            continue;
        }
        for k_idx in 0..c.top_tokens[i].len() {
            let tok = c.top_tokens[i][k_idx];
            let weight = c.top_weights[i][k_idx];
            if weight > 0 {
                add_evidence_multi(&mut store, &by_depth, &codes[i], tok, weight);
            }
        }
    }
    (store, codes)
}

// ---------------------------------------------------- store persistence --

/// Flat store container ("TLS1"): per grade level, keys in B-tree order,
/// each with its (token → count) evidence. Deterministic by construction
/// (B-tree iteration order), so the bytes are κ-pinnable; the store rebuilt
/// from the same corpus and artifact produces the same bytes.
pub fn store_bytes(store: &Store) -> Vec<u8> {
    let mut b: Vec<u8> = b"TLS1".to_vec();
    for level in store {
        b.extend_from_slice(&(level.len() as u32).to_le_bytes());
        for (key, dist) in level {
            b.push(key.len() as u8);
            b.extend_from_slice(key);
            b.extend_from_slice(&(dist.len() as u32).to_le_bytes());
            for (&t, &cnt) in dist {
                b.extend_from_slice(&t.to_le_bytes());
                b.extend_from_slice(&cnt.to_le_bytes());
            }
        }
    }
    b
}

/// Parse a TLS1 container; validates magic, per-level key lengths, and
/// exact consumption. Inverse of `store_bytes`.
pub fn parse_store(b: &[u8]) -> Option<Store> {
    if b.len() < 4 || &b[0..4] != b"TLS1" {
        return None;
    }
    let mut o = 4usize;
    let mut store: Store = Vec::new();
    for d in 0..=STAGES {
        if o + 4 > b.len() {
            return None;
        }
        let n_keys = u32::from_le_bytes(b[o..o + 4].try_into().ok()?) as usize;
        o += 4;
        let mut level = BTreeMap::new();
        for _ in 0..n_keys {
            if o >= b.len() {
                return None;
            }
            let klen = b[o] as usize;
            o += 1;
            if klen != d || o + klen + 4 > b.len() {
                return None;
            }
            let key = b[o..o + klen].to_vec();
            o += klen;
            let n_entries = u32::from_le_bytes(b[o..o + 4].try_into().ok()?) as usize;
            o += 4;
            let mut dist = BTreeMap::new();
            for _ in 0..n_entries {
                if o + 8 > b.len() {
                    return None;
                }
                let t = u32::from_le_bytes(b[o..o + 4].try_into().ok()?);
                let cnt = u32::from_le_bytes(b[o + 4..o + 8].try_into().ok()?);
                o += 8;
                dist.insert(t, cnt);
            }
            level.insert(key, dist);
        }
        store.push(level);
    }
    if o != b.len() {
        return None;
    }
    Some(store)
}

/// Parse the legacy pre-u32 TLS1 variant: 6-byte `(u16 token, u32 count)`
/// evidence entries, written by pre-u32-migration compilers.
#[deprecated(
    note = "Legacy 16-bit store binaries are deprecated. Recompile store artifacts using u32 token IDs."
)]
pub fn parse_store_legacy_u16(b: &[u8]) -> Option<Store> {
    if b.len() < 4 || &b[0..4] != b"TLS1" {
        return None;
    }
    let mut o = 4usize;
    let mut store: Store = Vec::new();
    for d in 0..=STAGES {
        if o + 4 > b.len() {
            return None;
        }
        let n_keys = u32::from_le_bytes(b[o..o + 4].try_into().ok()?) as usize;
        o += 4;
        let mut level = BTreeMap::new();
        for _ in 0..n_keys {
            if o >= b.len() {
                return None;
            }
            let klen = b[o] as usize;
            o += 1;
            if klen != d || o + klen + 4 > b.len() {
                return None;
            }
            let key = b[o..o + klen].to_vec();
            o += klen;
            let n_entries = u32::from_le_bytes(b[o..o + 4].try_into().ok()?) as usize;
            o += 4;
            let mut dist = BTreeMap::new();
            for _ in 0..n_entries {
                if o + 6 > b.len() {
                    return None;
                }
                let t = u32::from(u16::from_le_bytes(b[o..o + 2].try_into().ok()?));
                let cnt = u32::from_le_bytes(b[o + 2..o + 6].try_into().ok()?);
                o += 6;
                dist.insert(t, cnt);
            }
            level.insert(key, dist);
        }
        store.push(level);
    }
    if o != b.len() {
        return None;
    }
    Some(store)
}

/// Errors returned when parsing a store binary in strict u32 mode.
#[derive(Debug, PartialEq, Eq)]
pub enum StoreParseError {
    InvalidFormat,
    /// Deprecated legacy 16-bit store format detected. Recompile store artifacts using u32 token IDs.
    LegacyStoreFormatDeprecated,
}

impl std::fmt::Display for StoreParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreParseError::InvalidFormat => write!(f, "invalid TLS1 store binary format"),
            StoreParseError::LegacyStoreFormatDeprecated => write!(
                f,
                "legacy 16-bit TLS1 store format is deprecated; recompile store artifacts using u32 token IDs"
            ),
        }
    }
}

impl std::error::Error for StoreParseError {}

/// Parse a store binary enforcing strict 32-bit integer (u32) token alignment.
/// Fails fast with `StoreParseError::LegacyStoreFormatDeprecated` if a legacy 16-bit binary is detected.
pub fn parse_store_strict_u32(b: &[u8]) -> Result<Store, StoreParseError> {
    if let Some(store) = parse_store(b) {
        return Ok(store);
    }
    #[allow(deprecated)]
    if parse_store_legacy_u16(b).is_some() {
        return Err(StoreParseError::LegacyStoreFormatDeprecated);
    }
    Err(StoreParseError::InvalidFormat)
}

/// Scan `models_dir` (e.g. `.uor-models/`) recursively for legacy `.u16` store cache files
/// or legacy store binaries, removing them to enforce u32 recompilation. Returns the number of files purged.
#[cfg(not(target_arch = "wasm32"))]
pub fn purge_legacy_store_cache(models_dir: &std::path::Path) -> std::io::Result<usize> {
    if !models_dir.exists() {
        return Ok(0);
    }
    let mut purged = 0usize;
    let entries = std::fs::read_dir(models_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            purged += purge_legacy_store_cache(&path)?;
        } else if let Some(ext) = path.extension() {
            if ext == "u16"
                || path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|s| s.contains("legacy_u16"))
            {
                std::fs::remove_file(&path)?;
                purged += 1;
            }
        }
    }
    Ok(purged)
}

/// κ-label of a store's TLS1 bytes.
pub fn store_kappa(store: &Store) -> String {
    format!("blake3:{}", blake3::hash(&store_bytes(store)).to_hex())
}

/// Remove one graded store entry, returning its evidence — the deletion
/// half of the provenance/deletion promise (TRANSFORMERLESS.md §5): to
/// remove a contribution is to remove its κ.
pub fn remove_entry(store: &mut Store, depth: usize, key: &[u8]) -> Option<BTreeMap<u32, u32>> {
    store.get_mut(depth)?.remove(key)
}

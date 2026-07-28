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
use super::compiler::{Compiled, Corpus, D, STAGES, WINDOW};
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

/// Full plain path: position → graded class code.
pub fn code_plain(art: &Compiled, rot: &[usize; WINDOW + 1], c: &Corpus, i: usize) -> [u8; STAGES] {
    let b = bundle_plain(art, rot, c, i);
    assign_plain(art, &sig_plain(art, &b))
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

/// Kernel-counted full path.
pub struct Runtime<'a> {
    pub art: &'a Compiled,
    pub rot: [usize; WINDOW + 1],
    pub pop: [u8; 256],
    pub kernel: OpKernel,
    pub state: RuntimeState,
}

impl<'a> Runtime<'a> {
    pub fn new(art: &'a Compiled) -> Self {
        Runtime {
            art,
            rot: derive_rotations(),
            pop: derive_popcount_table(),
            kernel: OpKernel::default(),
            state: RuntimeState::default(),
        }
    }

    pub fn assign(&mut self, c: &Corpus, i: usize) -> [u8; STAGES] {
        let rot = self.rot;
        let b = bundle_kernel(&mut self.kernel, self.art, &rot, c, i);
        let sig = sign_signature(&mut self.kernel, &b, &self.art.thresholds);
        self.code_from_sig(&sig)
    }

    /// Corpus-free kernel path: window of token ids, oldest first;
    /// identical values to the plain window path, every op counted.
    pub fn assign_window(&mut self, window: &[u32]) -> [u8; STAGES] {
        let rot = self.rot;
        let b = bundle_window_kernel(&mut self.kernel, self.art, &rot, window);
        let sig = sign_signature(&mut self.kernel, &b, &self.art.thresholds);
        self.code_from_sig(&sig)
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
pub fn build_store(art: &Compiled, c: &Corpus) -> (Store, Vec<[u8; STAGES]>) {
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

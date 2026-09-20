//! Matched geometric query read of the ordered older-prefix state.
//!
//! Three arms add a bounded residual to a frozen parent's integer logits. Every arm uses the same
//! parameter shapes, the same fixed scales, the same two reader-row accesses and the same residual
//! magnitude bound; only the *arrangement* of the two accesses differs.
//!
//! ```text
//! Q: (sum_j W[v,j] * (R[q*b, j]      + R[e, j])) << 7
//! S: (sum_j W[v,j] * (R[q, j]        + R[b, j])) << 7
//! L: (sum_j W[v,j] * (R[q_tail*b, j] + R[e, j])) << 7
//! ```
//!
//! `A[V,8]` is the chronological write map, `B[V,8]` the current-token query map, `R` a 120x16
//! ternary reader and `W` a `V x 16` ternary output. `q` is the ordered right product of `A`
//! actions over the older prefix (at most 62 tokens, excluding the local pair); `b = gamma[B(x_i)]`;
//! `q_tail = gamma[A(x_{i-1})] * gamma[A(x_i)]`. Positions `i < 2` carry a zero residual in **all**
//! arms.
//!
//! At fixed parameters `r_Q - r_S = 128 * W(R[q*b] - R[q] - R[b] + R[e])`, which vanishes at `q = e`
//! or `b = e`. `S` therefore already has an active query channel; a win must beat `E`, `S` **and**
//! `L` and respond to an older-prefix intervention.
//!
//! Direct product is the selected transport. This is a query-conditioned *read*; it is not a
//! selective write, not a query-conditioned recurrent update and not a transformer replacement.
//! One 120-state register holds at most `log2(120)` bits, and right multiplication by a fixed
//! element only permutes it.
#![forbid(unsafe_code)]

use super::group_table::{GROUP_ORDER, ROW_STRIDE};
use super::lowbit::TernaryLinear;
use super::prefix_artifact::{hex_lower, ExactGroupTable, PREFIX_ARTIFACT_MAGIC};
use super::prefix_state::{Palette, ParentCache, PALETTE_SIZE};
use super::prior_learning::{PriorCore, BIAS_CODE_MAX, PRIOR_CLAMP};

/// Reader hidden width, shared by every arm.
pub const READER_WIDTH: usize = 16;
/// Reader row shift for this design (`h = Rcode << 4`).
pub const READER_SHIFT: u32 = 4;
/// Output row shift for this design (`residual = acc << 3`).
pub const OUTPUT_SHIFT: u32 = 3;
/// Declared residual shift: `READER_SHIFT + OUTPUT_SHIFT = 7`, a left shift and not a multiplier.
pub const RESIDUAL_SHIFT: u32 = READER_SHIFT + OUTPUT_SHIFT;
/// Context bound; the maximum older-prefix length is `CONTEXT - 2`.
pub const CONTEXT: usize = 64;
/// Maximum older-prefix tokens.
pub const MAX_OLDER: usize = 62;
/// Largest absolute residual score per vocabulary row: `2 units * 16 coords * 2^7 = 4096`.
pub const RESIDUAL_MAX_ABS: i64 = (2 * READER_WIDTH as i64) << RESIDUAL_SHIFT;

/// Reader/output Adam learning rate.
pub const RW_LR: f32 = 0.03;
/// Action/query-logit Adam learning rate.
pub const AB_LR: f32 = 0.003;
/// Adam betas and epsilon for every block.
pub const BETA1: f32 = 0.9;
pub const BETA2: f32 = 0.999;
pub const ADAM_EPS: f32 = 1e-8;
/// Ternary threshold: `|v| >= 0.5` maps to `+-1`.
pub const TERNA_TIE: f32 = 0.5;
/// Planned total updates per arm (warm-up included).
pub const TOTAL_UPDATES: usize = 512;
/// Updates 1..=warm-up train only R/W; A/B stay frozen.
pub const WARMUP_UPDATES: usize = 64;

pub const QUERY_ARTIFACT_MAGIC: &[u8; 4] = b"CPX3";
pub const QUERY_ARTIFACT_VERSION: u32 = 3;
pub const QUERY_CHECKPOINT_MAGIC: &[u8; 4] = b"CPQK";
pub const QUERY_CHECKPOINT_VERSION: u32 = 1;
/// The all-zero tokenizer field. It is a placeholder, never a tokenizer identity.
pub const ZERO_DIGEST: [u8; 32] = [0u8; 32];

const CODE_ZERO: u8 = 0;
const CODE_POS: u8 = 1;
const CODE_NEG: u8 = 2;

// ---------------------------------------------------------------------------
// Arm
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum QueryArm {
    /// Query-conditioned older-state read: `R[q*b] + R[e]`.
    Q = 0,
    /// Separable older-state and query read: `R[q] + R[b]`.
    S = 1,
    /// Local-only matched read: `R[q_tail*b] + R[e]`.
    L = 2,
}

impl QueryArm {
    pub fn name(self) -> &'static str {
        match self {
            QueryArm::Q => "query_conditioned_older_read",
            QueryArm::S => "separable_older_query_read",
            QueryArm::L => "local_only_read",
        }
    }
    pub fn from_u8(x: u8) -> Result<Self, String> {
        match x {
            0 => Ok(QueryArm::Q),
            1 => Ok(QueryArm::S),
            2 => Ok(QueryArm::L),
            _ => Err("unknown query-read arm".into()),
        }
    }
    /// Whether the read consumes the older prefix (Q and S) rather than only the local pair (L).
    pub fn uses_older_prefix(self) -> bool {
        !matches!(self, QueryArm::L)
    }
    /// The reader rows are `(q*b, e)` / `(q, b)` / `(q_tail*b, e)`; Q and L have the identity as
    /// their constant second row, S does not.
    pub fn has_constant_row(self) -> bool {
        !matches!(self, QueryArm::S)
    }
}

// ---------------------------------------------------------------------------
// Fixed-scale ternary construction from explicit codes
// ---------------------------------------------------------------------------

/// Build a `TernaryLinear` from explicit ternary codes at one declared fixed shift.
///
/// Used for the authored witnesses and for reloading, where the per-row scale is a declared
/// artifact property rather than something derived from the magnitudes.
pub fn ternary_from_codes(
    codes: &[i32],
    rows: usize,
    cols: usize,
    shift: u32,
) -> Result<TernaryLinear, String> {
    if codes.len() != rows * cols {
        return Err(format!("code count {} != {rows}x{cols}", codes.len()));
    }
    let mut packed = vec![0u8; (rows * cols).div_ceil(4)];
    for (flat, &c) in codes.iter().enumerate() {
        let code = match c {
            0 => CODE_ZERO,
            1 => CODE_POS,
            -1 => CODE_NEG,
            other => return Err(format!("non-ternary code {other}")),
        };
        packed[flat >> 2] |= code << (2 * (flat & 3) as u32);
    }
    TernaryLinear::from_packed(packed, vec![shift; rows], rows, cols)
}

#[cfg(test)]
thread_local! {
    /// Test-only compose-call counter, so a fixture can prove how many group products a path
    /// actually performs instead of asserting on the shape of a loop.
    pub(crate) static COMPOSE_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Reset the test-only compose counter on the current thread.
#[cfg(test)]
pub(crate) fn reset_compose_calls() {
    COMPOSE_CALLS.with(|c| c.set(0));
}

/// Read the test-only compose counter on the current thread.
#[cfg(test)]
pub(crate) fn compose_calls() -> usize {
    COMPOSE_CALLS.with(|c| c.get())
}

/// `h = a * b` in the bound exact table, both as historical state indices.
#[inline]
fn compose(t: &ExactGroupTable, a: usize, b: usize) -> usize {
    #[cfg(test)]
    COMPOSE_CALLS.with(|c| c.set(c.get() + 1));
    t.product[a * ROW_STRIDE + b] as usize
}

// ---------------------------------------------------------------------------
// The hard core
// ---------------------------------------------------------------------------

/// One prediction's read path: the two reader rows and the chronological A-chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadPath {
    pub first: usize,
    pub second: usize,
    /// Token indices of the chain, in chronological order.
    pub chain: Vec<usize>,
    /// `states[0]` is the identity; `states[k+1] = states[k] * gamma[A(tokens[chain[k]])]`.
    pub states: Vec<usize>,
}

/// Executable hard artifact: frozen parent, the two action maps, the reader/output codes and the
/// bound exact group table.
#[derive(Debug, Clone)]
pub struct QueryHard {
    pub parent: PriorCore,
    pub reader: TernaryLinear,
    pub wg: TernaryLinear,
    /// Hard write-action palette slots, one byte per token.
    pub a_codes: Vec<u8>,
    /// Hard query palette slots, one byte per token.
    pub b_codes: Vec<u8>,
    pub palette: Palette,
    pub arm: QueryArm,
    pub table: ExactGroupTable,
    pub parent_file_digest: [u8; 32],
    pub tokenizer_digest: [u8; 32],
}

impl QueryHard {
    /// Public construction. Requires a real (non-placeholder) tokenizer identity.
    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        parent: PriorCore,
        reader: TernaryLinear,
        wg: TernaryLinear,
        a_codes: Vec<u8>,
        b_codes: Vec<u8>,
        palette: Palette,
        arm: QueryArm,
        table: ExactGroupTable,
        parent_file_digest: [u8; 32],
        tokenizer_digest: [u8; 32],
    ) -> Result<Self, String> {
        Self::assemble(
            parent,
            reader,
            wg,
            a_codes,
            b_codes,
            palette,
            arm,
            table,
            parent_file_digest,
            tokenizer_digest,
            false,
        )
    }

    /// Validate shapes, fixed shifts, codes, table laws and the combined overflow envelope.
    ///
    /// `allow_zero_tokenizer` is set only by the restricted legacy import, whose bytes the caller
    /// must already have matched against a pinned hash.
    #[allow(clippy::too_many_arguments)]
    fn assemble(
        parent: PriorCore,
        reader: TernaryLinear,
        wg: TernaryLinear,
        a_codes: Vec<u8>,
        b_codes: Vec<u8>,
        palette: Palette,
        arm: QueryArm,
        table: ExactGroupTable,
        parent_file_digest: [u8; 32],
        tokenizer_digest: [u8; 32],
        allow_zero_tokenizer: bool,
    ) -> Result<Self, String> {
        let v = parent.cfg.vocab;
        if reader.rows != GROUP_ORDER || reader.cols != READER_WIDTH {
            return Err(format!(
                "reader shape {}x{} != {GROUP_ORDER}x{READER_WIDTH}",
                reader.rows, reader.cols
            ));
        }
        if wg.rows != v || wg.cols != READER_WIDTH {
            return Err(format!(
                "output shape {}x{} != {v}x{READER_WIDTH}",
                wg.rows, wg.cols
            ));
        }
        if a_codes.len() != v || b_codes.len() != v {
            return Err("action/query code count != vocab".into());
        }
        if a_codes
            .iter()
            .chain(b_codes.iter())
            .any(|&c| c as usize >= PALETTE_SIZE)
        {
            return Err("action or query code outside the palette".into());
        }
        for r in 0..GROUP_ORDER {
            if reader.shift(r) != READER_SHIFT {
                return Err(format!(
                    "reader row {r} shift {} != {READER_SHIFT}",
                    reader.shift(r)
                ));
            }
        }
        for r in 0..v {
            if wg.shift(r) != OUTPUT_SHIFT {
                return Err(format!(
                    "output row {r} shift {} != {OUTPUT_SHIFT}",
                    wg.shift(r)
                ));
            }
        }
        if table.identity != palette.identity {
            return Err("palette identity must match the bound table identity".into());
        }
        if tokenizer_digest == ZERO_DIGEST && !allow_zero_tokenizer {
            return Err(
                "query artifact tokenizer digest is the all-zero placeholder; production export requires the real raw 32-byte tokenizer identity"
                    .into(),
            );
        }
        let core = Self {
            parent,
            reader,
            wg,
            a_codes,
            b_codes,
            palette,
            arm,
            table,
            parent_file_digest,
            tokenizer_digest,
        };
        core.validate_envelope()?;
        Ok(core)
    }

    /// The parent's declared maximum absolute score from its own envelope declaration.
    pub fn parent_declared_max_abs(&self) -> i64 {
        let bias = (BIAS_CODE_MAX as i64) << self.parent.cfg.bias_scale_bits;
        let h_max = if self.parent.cfg.norm_bits == 0 {
            PRIOR_CLAMP as i64
        } else {
            1i64 << self.parent.cfg.norm_bits
        };
        let acc = self.parent.cfg.dv as i64 * h_max;
        let mut w_max = 0i64;
        for r in 0..self.parent.cfg.vocab {
            let bound = acc
                .checked_shl(self.parent.w_o.shift(r))
                .unwrap_or(i64::MAX / 8);
            w_max = w_max.max(bound);
        }
        bias + w_max
    }

    /// Parent bound plus residual bound must stay far from `i32::MAX`.
    pub fn validate_envelope(&self) -> Result<(), String> {
        let total = self
            .parent_declared_max_abs()
            .saturating_add(RESIDUAL_MAX_ABS);
        if total > (i32::MAX as i64) / 2 {
            return Err(format!(
                "combined parent+residual envelope {total} exceeds half of i32::MAX"
            ));
        }
        Ok(())
    }

    #[inline]
    pub fn write_state(&self, token: u32) -> usize {
        let v = self.parent.cfg.vocab;
        self.palette.elements[self.a_codes[(token as usize).min(v - 1)] as usize] as usize
    }

    #[inline]
    pub fn query_state(&self, token: u32) -> usize {
        let v = self.parent.cfg.vocab;
        self.palette.elements[self.b_codes[(token as usize).min(v - 1)] as usize] as usize
    }

    /// Ordered right product of `A` actions over `[max(0,i-63), i-1)`, or `None` when empty.
    pub fn older_state(&self, tokens: &[u32], i: usize) -> Option<usize> {
        if i < 2 {
            return None;
        }
        let start = i.saturating_sub(CONTEXT - 1);
        let end = i - 1;
        if end <= start {
            return None;
        }
        let mut q = self.table.identity as usize;
        for &tok in &tokens[start..end] {
            q = compose(&self.table, q, self.write_state(tok));
        }
        Some(q)
    }

    /// `gamma[A(x_{i-1})] * gamma[A(x_i)]`.
    pub fn tail_state(&self, tokens: &[u32], i: usize) -> Option<usize> {
        if i < 2 || i >= tokens.len() {
            return None;
        }
        let mut q = self.table.identity as usize;
        for &tok in &tokens[i - 1..=i] {
            q = compose(&self.table, q, self.write_state(tok));
        }
        Some(q)
    }

    /// The two reader rows for a prediction, or `None` where the residual is defined to be zero.
    pub fn read_path(&self, tokens: &[u32], i: usize) -> Option<ReadPath> {
        let older = self.older_state(tokens, i)?;
        self.read_path_from_older(tokens, i, older)
    }

    /// As [`Self::read_path`], but with the older-prefix product supplied (used for exact donors).
    pub fn read_path_from_older(&self, tokens: &[u32], i: usize, older: usize) -> Option<ReadPath> {
        if i < 2 || i >= tokens.len() {
            return None;
        }
        let e = self.table.identity as usize;
        let chain: Vec<usize> = match self.arm {
            QueryArm::L => vec![i - 1, i],
            _ => {
                let start = i.saturating_sub(CONTEXT - 1);
                (start..i - 1).collect()
            }
        };
        if chain.is_empty() {
            return None;
        }
        let mut states = Vec::with_capacity(chain.len() + 1);
        states.push(e);
        for &j in &chain {
            let last = *states.last().unwrap();
            states.push(compose(&self.table, last, self.write_state(tokens[j])));
        }
        let terminal = match self.arm {
            QueryArm::L => *states.last().unwrap(),
            _ => {
                debug_assert_eq!(
                    *states.last().unwrap(),
                    older,
                    "the rebuilt chain state must equal the supplied older product"
                );
                older
            }
        };
        let b = self.query_state(tokens[i]);
        let (first, second) = match self.arm {
            QueryArm::S => (terminal, b),
            _ => (compose(&self.table, terminal, b), e),
        };
        Some(ReadPath {
            first,
            second,
            chain,
            states,
        })
    }

    /// Validation for the public replay boundary: token IDs must be real vocabulary entries and
    /// no silent clamping substitutes another token.
    pub fn validate_tokens(&self, tokens: &[u32]) -> Result<(), String> {
        let v = self.parent.cfg.vocab;
        for (k, &t) in tokens.iter().enumerate() {
            if (t as usize) >= v {
                return Err(format!("token {t} at position {k} is outside 0..{v}"));
            }
        }
        Ok(())
    }

    /// **Inference row selection.** Folds the older prefix at most once, builds no training chain
    /// and returns `None` exactly where the residual is defined to be zero. Q/S fold history; L
    /// touches only its local pair.
    pub fn inference_rows(&self, tokens: &[u32], i: usize) -> Option<(usize, usize)> {
        if i < 2 || i >= tokens.len() {
            return None;
        }
        let e = self.table.identity as usize;
        let b = self.query_state(tokens[i]);
        match self.arm {
            QueryArm::L => {
                let mut q = e;
                q = compose(&self.table, q, self.write_state(tokens[i - 1]));
                q = compose(&self.table, q, self.write_state(tokens[i]));
                Some((compose(&self.table, q, b), e))
            }
            QueryArm::Q => {
                let q = self.older_state(tokens, i)?;
                Some((compose(&self.table, q, b), e))
            }
            QueryArm::S => {
                let q = self.older_state(tokens, i)?;
                Some((q, b))
            }
        }
    }

    /// **Inference row selection with a supplied older state** (the exact-donor path). Consumes the
    /// validated state without folding history again and without rebuilding a recipient chain. The
    /// local-only arm ignores the supplied state, as it must.
    pub fn rows_from_older(
        &self,
        tokens: &[u32],
        i: usize,
        older: usize,
    ) -> Option<(usize, usize)> {
        if i < 2 || i >= tokens.len() {
            return None;
        }
        let e = self.table.identity as usize;
        match self.arm {
            QueryArm::L => self.inference_rows(tokens, i),
            QueryArm::S => Some((older, self.query_state(tokens[i]))),
            QueryArm::Q => Some((compose(&self.table, older, self.query_state(tokens[i])), e)),
        }
    }

    /// Checked inference rows: rejects out-of-vocabulary tokens instead of clamping them.
    pub fn inference_rows_checked(
        &self,
        tokens: &[u32],
        i: usize,
    ) -> Result<Option<(usize, usize)>, String> {
        self.validate_tokens(tokens)?;
        Ok(self.inference_rows(tokens, i))
    }

    /// The single-row residual `u(s) = (sum_j Wcode[v,j] * Rcode[s,j]) << 7`: one reader row's
    /// contribution. `residual_scores(a, b)` equals `u(a) + u(b)` elementwise.
    pub fn row_scores(&self, s: usize) -> Vec<i32> {
        let mut h = vec![0i32; READER_WIDTH];
        for (j, slot) in h.iter_mut().enumerate() {
            *slot = self.reader.weight(s, j) << self.reader.shift(s);
        }
        self.wg.forward_i32(&h)
    }

    /// `(sum_j Wcode[v,j] * (Rcode[first,j] + Rcode[second,j])) << 7`.
    pub fn residual_scores(&self, first: usize, second: usize) -> Vec<i32> {
        let mut h = vec![0i32; READER_WIDTH];
        for (j, slot) in h.iter_mut().enumerate() {
            *slot = (self.reader.weight(first, j) + self.reader.weight(second, j))
                << self.reader.shift(first);
        }
        debug_assert_eq!(READER_SHIFT + OUTPUT_SHIFT, RESIDUAL_SHIFT);
        self.wg.forward_i32(&h)
    }

    /// Frozen parent scores plus the optional residual.
    pub fn int_logits(&self, prev: usize, cur: usize, rows: Option<(usize, usize)>) -> Vec<i32> {
        let mut z = self.parent.int_logits(prev, cur, true);
        if let Some((first, second)) = rows {
            for (a, b) in z.iter_mut().zip(self.residual_scores(first, second).iter()) {
                *a += *b;
            }
        }
        z
    }

    /// Greedy continuation, lowest-ID ties, no decoder change.
    pub fn generate(&self, prompt: &[u32], n_new: usize) -> Vec<u32> {
        let mut toks = prompt.to_vec();
        let v = self.parent.cfg.vocab;
        for _ in 0..n_new {
            let i = toks.len() - 1;
            let prev = if i == 0 {
                self.parent.cfg.pad_row()
            } else {
                (toks[i - 1] as usize).min(v - 1)
            };
            let cur = (toks[i] as usize).min(v - 1);
            let rows = self.inference_rows(&toks, i);
            let z = self.int_logits(prev, cur, rows);
            let mut best = 0usize;
            for r in 1..z.len() {
                if z[r] > z[best] {
                    best = r;
                }
            }
            toks.push(best as u32);
        }
        toks[prompt.len()..].to_vec()
    }

    /// As [`Self::generate`], with the public boundary validated: a non-empty prompt whose tokens are
    /// real vocabulary entries, and no silent clamping.
    pub fn generate_checked(&self, prompt: &[u32], n_new: usize) -> Result<Vec<u32>, String> {
        if prompt.is_empty() {
            return Err("prompt must not be empty".into());
        }
        self.validate_tokens(prompt)?;
        Ok(self.generate(prompt, n_new))
    }

    pub fn weight_bytes(&self) -> usize {
        self.reader.weight_bytes()
            + self.wg.weight_bytes()
            + self.a_codes.len().div_ceil(2)
            + self.b_codes.len().div_ceil(2)
    }

    /// Versioned `CPX3` export. Old `CPX2` bytes are never reinterpreted: the magic and version
    /// differ, and loader reuses the shared prefix magic only to reject it by name.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(QUERY_ARTIFACT_MAGIC);
        o.extend_from_slice(&QUERY_ARTIFACT_VERSION.to_le_bytes());
        o.extend_from_slice(&(self.parent.cfg.vocab as u32).to_le_bytes());
        o.extend_from_slice(&(self.parent.cfg.dv as u32).to_le_bytes());
        o.extend_from_slice(&(READER_WIDTH as u32).to_le_bytes());
        o.extend_from_slice(&READER_SHIFT.to_le_bytes());
        o.extend_from_slice(&OUTPUT_SHIFT.to_le_bytes());
        o.extend_from_slice(&(CONTEXT as u32).to_le_bytes());
        o.extend_from_slice(&self.parent.cfg.f_bits.to_le_bytes());
        o.extend_from_slice(&self.parent.cfg.bias_scale_bits.to_le_bytes());
        o.push(self.arm as u8);
        o.extend_from_slice(&self.parent_file_digest);
        o.extend_from_slice(&self.tokenizer_digest);
        o.extend_from_slice(&self.palette.elements);
        o.push(self.palette.identity);
        o.push(self.table.identity);
        o.extend_from_slice(&self.table.product);
        o.extend_from_slice(&self.table.inverse);
        o.extend_from_slice(&self.table.historical_to_sorted);
        o.extend_from_slice(&(self.reader.rows as u32).to_le_bytes());
        o.extend_from_slice(&(self.reader.cols as u32).to_le_bytes());
        for r in 0..self.reader.rows {
            o.extend_from_slice(&self.reader.shift(r).to_le_bytes());
        }
        o.extend_from_slice(self.reader.packed());
        o.extend_from_slice(&(self.wg.rows as u32).to_le_bytes());
        o.extend_from_slice(&(self.wg.cols as u32).to_le_bytes());
        for r in 0..self.wg.rows {
            o.extend_from_slice(&self.wg.shift(r).to_le_bytes());
        }
        o.extend_from_slice(self.wg.packed());
        for codes in [&self.a_codes, &self.b_codes] {
            let v = self.parent.cfg.vocab;
            let mut packed = vec![0u8; v.div_ceil(2)];
            for (t, &code) in codes.iter().enumerate() {
                packed[t >> 1] |= (code & 0xF) << (4 * (t & 1) as u32);
            }
            o.extend_from_slice(&packed);
        }
        o
    }

    /// Validated production load. Rejects a wrong magic/version, any per-row shift other than the
    /// declared fixed ones, out-of-palette codes, non-bijective table rows, a palette/table identity
    /// mismatch, a parent mismatch, a tokenizer mismatch, a placeholder tokenizer field, a bad
    /// envelope and trailing bytes.
    pub fn from_bytes(
        bytes: &[u8],
        parent: &PriorCore,
        expected_parent_digest: &[u8; 32],
        expected_tokenizer_digest: &[u8; 32],
    ) -> Result<Self, String> {
        Self::from_bytes_inner(
            bytes,
            parent,
            expected_parent_digest,
            Some(expected_tokenizer_digest),
        )
    }

    /// Restricted import for the hash-pinned legacy artifacts whose tokenizer field is the all-zero
    /// placeholder. The caller must have matched `bytes` against a known digest first; this path
    /// never becomes a production loader and never relaxes any other check.
    pub fn import_legacy(
        bytes: &[u8],
        parent: &PriorCore,
        expected_parent_digest: &[u8; 32],
    ) -> Result<Self, String> {
        Self::from_bytes_inner(bytes, parent, expected_parent_digest, None)
    }

    fn from_bytes_inner(
        bytes: &[u8],
        parent: &PriorCore,
        expected_parent_digest: &[u8; 32],
        expected_tokenizer_digest: Option<&[u8; 32]>,
    ) -> Result<Self, String> {
        let mut c = 0usize;
        let take = |c: &mut usize, n: usize| -> Result<&[u8], String> {
            let end = c.checked_add(n).ok_or("size overflow")?;
            if end > bytes.len() {
                return Err("truncated query artifact".into());
            }
            let s = &bytes[*c..end];
            *c = end;
            Ok(s)
        };
        let magic = take(&mut c, 4)?;
        if magic == PREFIX_ARTIFACT_MAGIC {
            return Err("this is a CPX2 prefix artifact, not a CPX3 query artifact".into());
        }
        if magic != QUERY_ARTIFACT_MAGIC {
            return Err("bad query artifact magic".into());
        }
        let u32_at = |c: &mut usize| -> Result<u32, String> {
            Ok(u32::from_le_bytes(take(c, 4)?.try_into().unwrap()))
        };
        if u32_at(&mut c)? != QUERY_ARTIFACT_VERSION {
            return Err("unsupported query artifact version".into());
        }
        let vocab = u32_at(&mut c)? as usize;
        let dv = u32_at(&mut c)? as usize;
        let reader_width = u32_at(&mut c)? as usize;
        let reader_shift = u32_at(&mut c)?;
        let output_shift = u32_at(&mut c)?;
        let context = u32_at(&mut c)? as usize;
        let f_bits = u32_at(&mut c)?;
        let bias_scale_bits = u32_at(&mut c)?;
        let arm = QueryArm::from_u8(take(&mut c, 1)?[0])?;
        let mut parent_file_digest = [0u8; 32];
        parent_file_digest.copy_from_slice(take(&mut c, 32)?);
        let mut tokenizer_digest = [0u8; 32];
        tokenizer_digest.copy_from_slice(take(&mut c, 32)?);
        let mut elements = [0u8; PALETTE_SIZE];
        elements.copy_from_slice(take(&mut c, PALETTE_SIZE)?);
        let palette_identity = take(&mut c, 1)?[0];
        let table_identity = take(&mut c, 1)?[0];
        let product = take(&mut c, GROUP_ORDER * ROW_STRIDE)?.to_vec();
        let inverse = take(&mut c, GROUP_ORDER)?.to_vec();
        let mapping = take(&mut c, GROUP_ORDER)?.to_vec();

        if vocab != parent.cfg.vocab
            || dv != parent.cfg.dv
            || reader_width != READER_WIDTH
            || reader_shift != READER_SHIFT
            || output_shift != OUTPUT_SHIFT
            || context != CONTEXT
            || f_bits != parent.cfg.f_bits
            || bias_scale_bits != parent.cfg.bias_scale_bits
        {
            return Err("query artifact configuration differs from the supplied parent".into());
        }
        if &parent_file_digest != expected_parent_digest {
            return Err("query artifact parent digest differs".into());
        }
        for a in 0..GROUP_ORDER {
            let mut row = [false; GROUP_ORDER];
            for b in 0..GROUP_ORDER {
                let p = product[a * ROW_STRIDE + b] as usize;
                if p >= GROUP_ORDER || row[p] {
                    return Err(format!("exact product table row {a} is not a bijection"));
                }
                row[p] = true;
            }
        }
        let e = table_identity as usize;
        if e >= GROUP_ORDER {
            return Err("bound table identity out of range".into());
        }
        for a in 0..GROUP_ORDER {
            if product[a * ROW_STRIDE + e] as usize != a
                || product[e * ROW_STRIDE + a] as usize != a
            {
                return Err("identity law failed in the bound table".into());
            }
            let inv = inverse[a] as usize;
            if inv >= GROUP_ORDER || product[a * ROW_STRIDE + inv] as usize != e {
                return Err("inverse law failed in the bound table".into());
            }
        }
        let mut seen = [false; GROUP_ORDER];
        for &m in mapping.iter() {
            if (m as usize) >= GROUP_ORDER || seen[m as usize] {
                return Err("root mapping is not a bijection".into());
            }
            seen[m as usize] = true;
        }

        let rr = u32_at(&mut c)? as usize;
        let rc = u32_at(&mut c)? as usize;
        if rr != GROUP_ORDER || rc != READER_WIDTH {
            return Err("reader shape mismatch".into());
        }
        let mut r_shift = Vec::with_capacity(rr);
        for _ in 0..rr {
            let s = u32_at(&mut c)?;
            if s != READER_SHIFT {
                return Err("reader shift differs from the declared fixed scale".into());
            }
            r_shift.push(s);
        }
        let r_packed = take(&mut c, (rr * rc).div_ceil(4))?.to_vec();
        let reader = TernaryLinear::from_packed(r_packed, r_shift, rr, rc)?;

        let or_ = u32_at(&mut c)? as usize;
        let oc = u32_at(&mut c)? as usize;
        if or_ != vocab || oc != READER_WIDTH {
            return Err("output head shape mismatch".into());
        }
        let mut o_shift = Vec::with_capacity(or_);
        for _ in 0..or_ {
            let s = u32_at(&mut c)?;
            if s != OUTPUT_SHIFT {
                return Err("output shift differs from the declared fixed scale".into());
            }
            o_shift.push(s);
        }
        let o_packed = take(&mut c, (or_ * oc).div_ceil(4))?.to_vec();
        let wg = TernaryLinear::from_packed(o_packed, o_shift, or_, oc)?;

        let mut codes: Vec<Vec<u8>> = Vec::new();
        for _ in 0..2 {
            let packed = take(&mut c, vocab.div_ceil(2))?;
            let mut out = Vec::with_capacity(vocab);
            for t in 0..vocab {
                let code = (packed[t >> 1] >> (4 * (t & 1) as u32)) & 0xF;
                if (code as usize) >= PALETTE_SIZE {
                    return Err(format!(
                        "palette code {code} for token {t} is outside the palette"
                    ));
                }
                out.push(code);
            }
            codes.push(out);
        }
        if c != bytes.len() {
            return Err(format!(
                "{} trailing bytes in the query artifact",
                bytes.len() - c
            ));
        }
        for &element in elements.iter() {
            if (element as usize) >= GROUP_ORDER {
                return Err("palette element outside the group".into());
            }
        }
        if palette_identity != table_identity {
            return Err("palette identity must match the bound table identity".into());
        }
        let b_codes = codes.pop().unwrap();
        let a_codes = codes.pop().unwrap();
        if let Some(exp) = expected_tokenizer_digest {
            if tokenizer_digest != *exp {
                return Err(format!(
                    "query artifact tokenizer digest {} != expected {}",
                    hex_lower(&tokenizer_digest),
                    hex_lower(exp)
                ));
            }
        }
        Self::assemble(
            parent.clone(),
            reader,
            wg,
            a_codes,
            b_codes,
            Palette {
                elements,
                identity: palette_identity,
            },
            arm,
            ExactGroupTable {
                product: product.into_boxed_slice(),
                identity: table_identity,
                inverse: inverse.into_boxed_slice(),
                historical_to_sorted: mapping.into_boxed_slice(),
                max_mismatches_vs_floating: 0,
            },
            parent_file_digest,
            tokenizer_digest,
            expected_tokenizer_digest.is_none(),
        )
    }
}

// ---------------------------------------------------------------------------
// Offline trainer
// ---------------------------------------------------------------------------

fn xorshift(st: &mut u64) -> u64 {
    *st ^= *st << 13;
    *st ^= *st >> 7;
    *st ^= *st << 17;
    *st
}

fn seed_state(seed: u64, tag: u64) -> u64 {
    let mut z = seed
        .wrapping_add(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(tag.wrapping_mul(0xBF58_476D_1CE4_E5B9));
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn argmax8(a: &[f32]) -> usize {
    let mut best = 0usize;
    for k in 1..PALETTE_SIZE {
        if a[k] > a[best] {
            best = k;
        }
    }
    best
}

fn softmax8(a: &[f32]) -> [f32; PALETTE_SIZE] {
    let mut max = a[0];
    for &x in &a[1..PALETTE_SIZE] {
        if x > max {
            max = x;
        }
    }
    let mut out = [0.0f32; PALETTE_SIZE];
    let mut sum = 0.0f32;
    for k in 0..PALETTE_SIZE {
        let e = (a[k] - max).exp();
        out[k] = e;
        sum += e;
    }
    for x in out.iter_mut() {
        *x /= sum;
    }
    out
}

fn softmax_scaled(z: &[i32], scale: f64) -> (Vec<f64>, f64) {
    let mut max = f64::NEG_INFINITY;
    for &v in z {
        let s = v as f64 * scale;
        if s > max {
            max = s;
        }
    }
    let mut p = Vec::with_capacity(z.len());
    let mut sum = 0.0f64;
    for &v in z {
        let e = (v as f64 * scale - max).exp();
        p.push(e);
        sum += e;
    }
    for x in p.iter_mut() {
        *x /= sum;
    }
    (p, max + sum.ln())
}

fn l2(x: &[f32]) -> f64 {
    x.iter()
        .map(|v| (*v as f64) * (*v as f64))
        .sum::<f64>()
        .sqrt()
}

/// One Adam step with bias correction at the update's own age, then the master clamp to `[-1, 1]`.
fn adam_step(master: &mut [f32], m: &mut [f32], vv: &mut [f32], grad: &[f32], lr: f32, age: u64) {
    assert_eq!(master.len(), grad.len());
    let age = age.min(100_000) as i32;
    let bc1 = 1.0 - BETA1.powi(age);
    let bc2 = 1.0 - BETA2.powi(age);
    for i in 0..master.len() {
        let g = grad[i];
        m[i] = BETA1 * m[i] + (1.0 - BETA1) * g;
        vv[i] = BETA2 * vv[i] + (1.0 - BETA2) * g * g;
        let mhat = m[i] / bc1;
        let vhat = vv[i] / bc2;
        master[i] -= lr * mhat / (vhat.sqrt() + ADAM_EPS);
        master[i] = master[i].clamp(-1.0, 1.0);
    }
}

/// Offline trainer for one arm. The frozen parent is never mutated.
#[derive(Clone, Debug)]
pub struct QueryTrainer {
    pub parent: PriorCore,
    pub palette: Palette,
    pub table: ExactGroupTable,
    pub arm: QueryArm,
    pub seed: u64,
    pub batch: usize,
    pub updates: usize,
    pub warmup: usize,
    /// `V x 8` write-action logits.
    pub a_master: Vec<f32>,
    /// `V x 8` query logits.
    pub b_master: Vec<f32>,
    /// `GROUP_ORDER x 16` reader masters.
    pub r_master: Vec<f32>,
    /// `V x 16` output masters.
    pub w_master: Vec<f32>,
    r_m: Vec<f32>,
    r_v: Vec<f32>,
    w_m: Vec<f32>,
    w_v: Vec<f32>,
    a_m: Vec<f32>,
    a_v: Vec<f32>,
    b_m: Vec<f32>,
    b_v: Vec<f32>,
    pub rw_updates: u64,
    pub ab_updates: u64,
    pub pass: u32,
    /// Diagnostics: last batch's pre-clip gradient norms.
    pub last_grad_norm_rw: f64,
    pub last_grad_norm_ab: f64,
    pub last_hard_a_changes: usize,
    pub last_hard_b_changes: usize,
    pub parent_sha256: [u8; 32],
    /// Raw 32-byte tokenizer identity bound into every exported artifact.
    pub tokenizer_digest: [u8; 32],
    pub data_identity: [u8; 32],
    initial_a_codes: Vec<u8>,
    initial_b_codes: Vec<u8>,
}

/// Raw gradients and loss accumulated over one batch, before averaging or clipping.
#[derive(Clone, Debug)]
pub struct BatchGrads {
    pub gw: Vec<f32>,
    pub gr: Vec<f32>,
    pub da: Vec<f32>,
    pub db: Vec<f32>,
    pub loss_sum: f64,
    pub scored: usize,
}

impl QueryTrainer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        parent: PriorCore,
        palette: Palette,
        table: ExactGroupTable,
        arm: QueryArm,
        seed: u64,
        batch: usize,
        updates: usize,
        warmup: usize,
        parent_sha256: [u8; 32],
        tokenizer_digest: [u8; 32],
    ) -> Result<Self, String> {
        if tokenizer_digest == ZERO_DIGEST {
            return Err(
                "a query trainer requires the real raw 32-byte tokenizer identity, not the all-zero placeholder"
                    .into(),
            );
        }
        let v = parent.cfg.vocab;
        let identity_slot = palette
            .elements
            .iter()
            .position(|&e| e == table.identity)
            .ok_or("the palette does not contain the bound table identity")?;
        let mut st = seed ^ 0x5DEE_CE66_D1CE_B00D;
        // Reader masters seeded +/- 1: nonzero from the start, so every output row gets a gradient.
        let r_master: Vec<f32> = (0..GROUP_ORDER * READER_WIDTH)
            .map(|_| {
                if xorshift(&mut st) & 1 == 0 {
                    1.0
                } else {
                    -1.0
                }
            })
            .collect();
        // Output masters exactly zero: step 0 must reproduce the frozen parent exactly.
        let w_master = vec![0.0f32; v * READER_WIDTH];
        // A: one seeded preferred palette slot per token at 0.1. B: the identity slot for every
        // token at 0.1. Both are frozen during warm-up.
        let mut a_master = vec![0.0f32; v * PALETTE_SIZE];
        let mut b_master = vec![0.0f32; v * PALETTE_SIZE];
        for t in 0..v {
            let id = (seed_state(seed, t as u64) % PALETTE_SIZE as u64) as usize;
            a_master[t * PALETTE_SIZE + id] = 0.1;
            b_master[t * PALETTE_SIZE + identity_slot] = 0.1;
        }
        let initial_a_codes = (0..v)
            .map(|t| argmax8(&a_master[t * PALETTE_SIZE..(t + 1) * PALETTE_SIZE]) as u8)
            .collect();
        let initial_b_codes = (0..v)
            .map(|t| argmax8(&b_master[t * PALETTE_SIZE..(t + 1) * PALETTE_SIZE]) as u8)
            .collect();
        Ok(Self {
            parent,
            palette,
            table,
            arm,
            seed,
            batch,
            updates,
            warmup,
            r_m: vec![0.0; r_master.len()],
            r_v: vec![0.0; r_master.len()],
            w_m: vec![0.0; w_master.len()],
            w_v: vec![0.0; w_master.len()],
            a_m: vec![0.0; a_master.len()],
            a_v: vec![0.0; a_master.len()],
            b_m: vec![0.0; b_master.len()],
            b_v: vec![0.0; b_master.len()],
            r_master,
            w_master,
            a_master,
            b_master,
            rw_updates: 0,
            ab_updates: 0,
            pass: 0,
            last_grad_norm_rw: 0.0,
            last_grad_norm_ab: 0.0,
            last_hard_a_changes: 0,
            last_hard_b_changes: 0,
            parent_sha256,
            tokenizer_digest,
            data_identity: [0u8; 32],
            initial_a_codes,
            initial_b_codes,
        })
    }

    /// The identity-preferred initial query slot; exposed for the learned-query fixture.
    pub fn identity_slot(&self) -> u8 {
        self.palette
            .elements
            .iter()
            .position(|&e| e == self.table.identity)
            .expect("validated palette") as u8
    }

    /// The seeded initial write-action codes, for hard-change accounting.
    pub fn initial_a_codes(&self) -> &[u8] {
        &self.initial_a_codes
    }

    /// The identity-preferred initial query codes.
    pub fn initial_b_codes(&self) -> &[u8] {
        &self.initial_b_codes
    }

    pub fn hard_a_codes(&self) -> Vec<u8> {
        let v = self.parent.cfg.vocab;
        (0..v)
            .map(|t| argmax8(&self.a_master[t * PALETTE_SIZE..(t + 1) * PALETTE_SIZE]) as u8)
            .collect()
    }

    pub fn hard_b_codes(&self) -> Vec<u8> {
        let v = self.parent.cfg.vocab;
        (0..v)
            .map(|t| argmax8(&self.b_master[t * PALETTE_SIZE..(t + 1) * PALETTE_SIZE]) as u8)
            .collect()
    }

    /// Build the served hard artifact from the current masters.
    pub fn hard_core(&self) -> Result<QueryHard, String> {
        let v = self.parent.cfg.vocab;
        let (rp, rs) = pack_fixed(&self.r_master, GROUP_ORDER, READER_WIDTH, READER_SHIFT);
        let (wp, ws) = pack_fixed(&self.w_master, v, READER_WIDTH, OUTPUT_SHIFT);
        QueryHard::from_parts(
            self.parent.clone(),
            TernaryLinear::from_packed(rp, rs, GROUP_ORDER, READER_WIDTH)?,
            TernaryLinear::from_packed(wp, ws, v, READER_WIDTH)?,
            self.hard_a_codes(),
            self.hard_b_codes(),
            self.palette.clone(),
            self.arm,
            self.table.clone(),
            self.parent_sha256,
            self.tokenizer_digest,
        )
    }

    /// Raw accumulated gradients over one batch, before averaging, clipping or any optimizer step.
    ///
    /// Exposed separately from [`Self::apply_grads`] so the declared derivatives can be checked
    /// against finite differences of the continuous surrogate instead of being asserted.
    pub fn compute_grads(
        &self,
        windows: &[Vec<u32>],
        cache: &mut ParentCache,
    ) -> Result<BatchGrads, String> {
        let v = self.parent.cfg.vocab;
        let core = self.hard_core()?;
        let mut gw = vec![0.0f32; v * READER_WIDTH];
        let mut gr = vec![0.0f32; GROUP_ORDER * READER_WIDTH];
        let mut da = vec![0.0f32; v * PALETTE_SIZE];
        let mut db = vec![0.0f32; v * PALETTE_SIZE];
        let mut loss = 0.0f64;
        let mut scored = 0usize;

        // Hard codes are constant inside the batch; hoist them out of the occurrence loop.
        let r_codes: Vec<i32> = (0..GROUP_ORDER * READER_WIDTH)
            .map(|k| core.reader.weight(k / READER_WIDTH, k % READER_WIDTH))
            .collect();
        let w_codes: Vec<i32> = (0..v * READER_WIDTH)
            .map(|k| core.wg.weight(k / READER_WIDTH, k % READER_WIDTH))
            .collect();

        let scale = (-(core.parent.cfg.f_bits as f64)).exp2();
        let inv_ln2 = 1.0 / std::f64::consts::LN_2;

        for w in windows {
            for i in 0..w.len().saturating_sub(1) {
                let prev = if i == 0 {
                    core.parent.cfg.pad_row()
                } else {
                    (w[i - 1] as usize).min(v - 1)
                };
                let cur = (w[i] as usize).min(v - 1);
                let target = w[i + 1];
                let t = (target as usize).min(v - 1);
                let zp = cache.z_for(&core.parent, prev, cur).to_vec();

                let path = core.read_path(w, i);
                let z_full = match &path {
                    Some(p) => {
                        let res = core.residual_scores(p.first, p.second);
                        let mut z = zp.clone();
                        for (a, b) in z.iter_mut().zip(res.iter()) {
                            *a += *b;
                        }
                        z
                    }
                    None => zp.clone(),
                };
                let (pv, lse) = softmax_scaled(&z_full, scale);
                loss += (lse - z_full[t] as f64 * scale) * inv_ln2;
                scored += 1;

                let path = match path {
                    Some(p) => p,
                    None => continue,
                };

                // d_v = (softmax_v - 1[v=target]) * 2^-F / ln 2
                let mut d = vec![0.0f64; v];
                for (r, slot) in d.iter_mut().enumerate() {
                    *slot = (pv[r] - if r == t { 1.0 } else { 0.0 }) * scale * inv_ln2;
                }

                let first = path.first;
                let second = path.second;
                let mut g = [0.0f32; READER_WIDTH];
                for r in 0..v {
                    let dv = d[r] as f32;
                    if dv == 0.0 {
                        continue;
                    }
                    let scaled = 128.0 * dv;
                    for j in 0..READER_WIDTH {
                        let wc = w_codes[r * READER_WIDTH + j];
                        if wc != 0 {
                            g[j] += dv * wc as f32;
                        }
                        let rc =
                            r_codes[first * READER_WIDTH + j] + r_codes[second * READER_WIDTH + j];
                        if rc != 0 {
                            gw[r * READER_WIDTH + j] += scaled * rc as f32;
                        }
                    }
                }
                for x in g.iter_mut() {
                    *x *= 128.0;
                }

                // Both selected reader rows receive the full reader adjoint.
                for j in 0..READER_WIDTH {
                    gr[first * READER_WIDTH + j] += g[j];
                    gr[second * READER_WIDTH + j] += g[j];
                }

                // Terminal-state adjoint and the query categorical credit.
                let b = core.query_state(w[i]);
                let terminal = *path.states.last().unwrap();
                let mut u = vec![0.0f32; GROUP_ORDER];
                for (s, us) in u.iter_mut().enumerate() {
                    let h = if self.arm == QueryArm::S {
                        s
                    } else {
                        compose(&core.table, s, b)
                    };
                    let mut acc = 0.0f32;
                    for j in 0..READER_WIDTH {
                        let rc = r_codes[h * READER_WIDTH + j];
                        if rc != 0 {
                            acc += g[j] * rc as f32;
                        }
                    }
                    *us = acc;
                }
                for k in 0..PALETTE_SIZE {
                    let gamma = core.palette.elements[k] as usize;
                    let h = if self.arm == QueryArm::S {
                        gamma
                    } else {
                        compose(&core.table, terminal, gamma)
                    };
                    let mut acc = 0.0f32;
                    for j in 0..READER_WIDTH {
                        let rc = r_codes[h * READER_WIDTH + j];
                        if rc != 0 {
                            acc += g[j] * rc as f32;
                        }
                    }
                    db[cur * PALETTE_SIZE + k] += acc;
                }

                // Reverse-chronological A-chain derivative with the ordered right-product
                // convention: `states[k+1] = states[k] * gamma_k`, credit[k] = u[states[k]*gamma_k],
                // and the state adjoint moves back as `next[s] = u[s * gamma_t]`.
                let mut uu = u;
                let mut next = vec![0.0f32; GROUP_ORDER];
                for tstep in (0..path.chain.len()).rev() {
                    let j = path.chain[tstep];
                    let tok = (w[j] as usize).min(v - 1);
                    let k_t = core.a_codes[tok] as usize;
                    let q_before = path.states[tstep];
                    let gamma_t = core.palette.elements[k_t] as usize;
                    for k in 0..PALETTE_SIZE {
                        let h = compose(&core.table, q_before, core.palette.elements[k] as usize);
                        da[tok * PALETTE_SIZE + k] += uu[h];
                    }
                    for s in 0..GROUP_ORDER {
                        next[s] = uu[core.table.product[s * ROW_STRIDE + gamma_t] as usize];
                    }
                    std::mem::swap(&mut uu, &mut next);
                }
            }
        }

        if scored == 0 {
            return Err("empty batch".into());
        }
        Ok(BatchGrads {
            gw,
            gr,
            da,
            db,
            loss_sum: loss,
            scored,
        })
    }

    /// Average by the actual scored-target count, clip, apply the A/B softmax Jacobians exactly
    /// once and take one Adam step on each enabled block. Returns the mean bits per target.
    ///
    /// `apply_a` and `apply_b` are separate so a fixture can hold the authored write map fixed and
    /// train the query map alone; the production schedule enables both together.
    pub fn apply_grads(
        &mut self,
        mut g: BatchGrads,
        apply_rw: bool,
        apply_a: bool,
        apply_b: bool,
    ) -> f64 {
        let inv = 1.0 / g.scored as f64;
        let inv32 = inv as f32;
        for x in
            g.gw.iter_mut()
                .chain(g.gr.iter_mut())
                .chain(g.da.iter_mut())
                .chain(g.db.iter_mut())
        {
            *x *= inv32;
        }

        // Global R/W gradient block clipped at 1.
        self.last_grad_norm_rw = l2(&g.gr).hypot(l2(&g.gw));
        if self.last_grad_norm_rw > 1.0 {
            let f = (1.0 / self.last_grad_norm_rw) as f32;
            for x in g.gw.iter_mut().chain(g.gr.iter_mut()) {
                *x *= f;
            }
        }

        if apply_a || apply_b {
            // Temperature-1 softmax Jacobian on each enabled map, applied exactly once per batch.
            // A/B moments and ages are untouched while the corresponding switch is false.
            let v = self.parent.cfg.vocab;
            let mut enabled: Vec<(&Vec<f32>, &mut Vec<f32>)> = Vec::new();
            if apply_a {
                enabled.push((&self.a_master, &mut g.da));
            }
            if apply_b {
                enabled.push((&self.b_master, &mut g.db));
            }
            for (logits, d) in enabled.into_iter() {
                for tok in 0..v {
                    let row = &logits[tok * PALETTE_SIZE..(tok + 1) * PALETTE_SIZE];
                    let pi = softmax8(row);
                    let mut dot = 0.0f32;
                    for k in 0..PALETTE_SIZE {
                        dot += pi[k] * d[tok * PALETTE_SIZE + k];
                    }
                    for k in 0..PALETTE_SIZE {
                        d[tok * PALETTE_SIZE + k] = pi[k] * (d[tok * PALETTE_SIZE + k] - dot);
                    }
                }
            }
            // Post-Jacobian A/B block clipped at 1, over exactly the enabled maps.
            self.last_grad_norm_ab = if apply_a && apply_b {
                l2(&g.da).hypot(l2(&g.db))
            } else if apply_a {
                l2(&g.da)
            } else {
                l2(&g.db)
            };
            if self.last_grad_norm_ab > 1.0 {
                let f = (1.0 / self.last_grad_norm_ab) as f32;
                for x in g.da.iter_mut() {
                    *x *= if apply_a { f } else { 0.0 };
                }
                for x in g.db.iter_mut() {
                    *x *= if apply_b { f } else { 0.0 };
                }
            } else if !apply_a || !apply_b {
                // The disabled map must not move at all.
                if !apply_a {
                    for x in g.da.iter_mut() {
                        *x = 0.0;
                    }
                }
                if !apply_b {
                    for x in g.db.iter_mut() {
                        *x = 0.0;
                    }
                }
            }
        }

        if apply_rw {
            let next_rw = self.rw_updates + 1;
            adam_step(
                &mut self.r_master,
                &mut self.r_m,
                &mut self.r_v,
                &g.gr,
                RW_LR,
                next_rw,
            );
            adam_step(
                &mut self.w_master,
                &mut self.w_m,
                &mut self.w_v,
                &g.gw,
                RW_LR,
                next_rw,
            );
            self.rw_updates = next_rw;
        }
        if apply_a || apply_b {
            let before_a = self.hard_a_codes();
            let before_b = self.hard_b_codes();
            let next_ab = self.ab_updates + 1;
            if apply_a {
                adam_step(
                    &mut self.a_master,
                    &mut self.a_m,
                    &mut self.a_v,
                    &g.da,
                    AB_LR,
                    next_ab,
                );
            }
            if apply_b {
                adam_step(
                    &mut self.b_master,
                    &mut self.b_m,
                    &mut self.b_v,
                    &g.db,
                    AB_LR,
                    next_ab,
                );
            }
            self.ab_updates = next_ab;
            self.last_hard_a_changes = before_a
                .iter()
                .zip(self.hard_a_codes().iter())
                .filter(|(x, y)| x != y)
                .count();
            self.last_hard_b_changes = before_b
                .iter()
                .zip(self.hard_b_codes().iter())
                .filter(|(x, y)| x != y)
                .count();
        }
        g.loss_sum * inv
    }

    /// One hard-forward training batch, returning the mean uncapped log-sum-exp bits per target.
    pub fn train_batch(
        &mut self,
        windows: &[Vec<u32>],
        cache: &mut ParentCache,
        apply_ab: bool,
    ) -> Result<f64, String> {
        self.train_batch_flags(windows, cache, true, apply_ab)
    }

    /// As [`Self::train_batch`], with explicit switches for the R/W and A/B blocks so a fixture can
    /// train the query map alone.
    pub fn train_batch_flags(
        &mut self,
        windows: &[Vec<u32>],
        cache: &mut ParentCache,
        apply_rw: bool,
        apply_ab: bool,
    ) -> Result<f64, String> {
        self.train_batch_blocks(windows, cache, apply_rw, apply_ab, apply_ab)
    }

    /// As [`Self::train_batch_flags`], with A and B separately switchable.
    pub fn train_batch_blocks(
        &mut self,
        windows: &[Vec<u32>],
        cache: &mut ParentCache,
        apply_rw: bool,
        apply_a: bool,
        apply_b: bool,
    ) -> Result<f64, String> {
        let g = self.compute_grads(windows, cache)?;
        Ok(self.apply_grads(g, apply_rw, apply_a, apply_b))
    }

    pub fn state_bytes(&self) -> usize {
        (self.r_master.len() + self.w_master.len() + self.a_master.len() + self.b_master.len()) * 4
            + (self.r_m.len()
                + self.r_v.len()
                + self.w_m.len()
                + self.w_v.len()
                + self.a_m.len()
                + self.a_v.len()
                + self.b_m.len()
                + self.b_v.len())
                * 4
    }

    /// Complete `CPQK` checkpoint: masters, moments, optimizer ages, schedule position, phase,
    /// palette/table identity and the parent binding.
    pub fn checkpoint_bytes(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(QUERY_CHECKPOINT_MAGIC);
        o.extend_from_slice(&QUERY_CHECKPOINT_VERSION.to_le_bytes());
        o.push(self.arm as u8);
        o.extend_from_slice(&self.seed.to_le_bytes());
        o.extend_from_slice(&(self.batch as u32).to_le_bytes());
        o.extend_from_slice(&(self.updates as u32).to_le_bytes());
        o.extend_from_slice(&(self.warmup as u32).to_le_bytes());
        o.extend_from_slice(&self.rw_updates.to_le_bytes());
        o.extend_from_slice(&self.ab_updates.to_le_bytes());
        o.extend_from_slice(&(self.pass as u32).to_le_bytes());
        o.extend_from_slice(&(self.parent.cfg.vocab as u32).to_le_bytes());
        o.extend_from_slice(&(self.parent.cfg.dv as u32).to_le_bytes());
        o.extend_from_slice(&self.parent_sha256);
        o.extend_from_slice(&self.data_identity);
        o.extend_from_slice(&self.palette.elements);
        o.push(self.palette.identity);
        o.push(self.table.identity);
        for a in [
            &self.r_master,
            &self.w_master,
            &self.a_master,
            &self.b_master,
            &self.r_m,
            &self.r_v,
            &self.w_m,
            &self.w_v,
            &self.a_m,
            &self.a_v,
            &self.b_m,
            &self.b_v,
        ] {
            o.extend_from_slice(&(a.len() as u32).to_le_bytes());
            for v in a {
                o.extend_from_slice(&v.to_le_bytes());
            }
        }
        o
    }

    /// Resume from a complete checkpoint. Every identity is validated; an arm mismatch is fatal.
    pub fn resume_from(&mut self, bytes: &[u8]) -> Result<(), String> {
        let mut c = 0usize;
        let take = |c: &mut usize, n: usize| -> Result<&[u8], String> {
            let end = c.checked_add(n).ok_or("size overflow")?;
            if end > bytes.len() {
                return Err("truncated query checkpoint".into());
            }
            let s = &bytes[*c..end];
            *c = end;
            Ok(s)
        };
        if take(&mut c, 4)? != QUERY_CHECKPOINT_MAGIC {
            return Err("bad query checkpoint magic".into());
        }
        let u32_at = |c: &mut usize| -> Result<u32, String> {
            Ok(u32::from_le_bytes(take(c, 4)?.try_into().unwrap()))
        };
        let u64_at = |c: &mut usize| -> Result<u64, String> {
            Ok(u64::from_le_bytes(take(c, 8)?.try_into().unwrap()))
        };
        if u32_at(&mut c)? != QUERY_CHECKPOINT_VERSION {
            return Err("unsupported query checkpoint version".into());
        }
        let arm = QueryArm::from_u8(take(&mut c, 1)?[0])?;
        let seed = u64_at(&mut c)?;
        let batch = u32_at(&mut c)? as usize;
        let updates = u32_at(&mut c)? as usize;
        let warmup = u32_at(&mut c)? as usize;
        let rw_updates = u64_at(&mut c)?;
        let ab_updates = u64_at(&mut c)?;
        let pass = u32_at(&mut c)?;
        let vocab = u32_at(&mut c)? as usize;
        let dv = u32_at(&mut c)? as usize;
        let mut parent_sha = [0u8; 32];
        parent_sha.copy_from_slice(take(&mut c, 32)?);
        let mut data_identity = [0u8; 32];
        data_identity.copy_from_slice(take(&mut c, 32)?);
        let mut elements = [0u8; PALETTE_SIZE];
        elements.copy_from_slice(take(&mut c, PALETTE_SIZE)?);
        let palette_identity = take(&mut c, 1)?[0];
        let table_identity = take(&mut c, 1)?[0];

        if arm != self.arm {
            return Err("query checkpoint arm differs from the trainer".into());
        }
        if vocab != self.parent.cfg.vocab || dv != self.parent.cfg.dv {
            return Err("query checkpoint dimensions differ from the parent".into());
        }
        if seed != self.seed
            || batch != self.batch
            || updates != self.updates
            || warmup != self.warmup
        {
            return Err("query checkpoint schedule differs from the trainer".into());
        }
        if parent_sha != self.parent_sha256 {
            return Err("query checkpoint parent binding differs".into());
        }
        if elements != self.palette.elements
            || palette_identity != self.palette.identity
            || table_identity != self.table.identity
        {
            return Err("query checkpoint palette/table binding differs".into());
        }

        let mut arrays: Vec<Vec<f32>> = Vec::new();
        for _ in 0..12 {
            let n = u32_at(&mut c)? as usize;
            let mut v = Vec::with_capacity(n);
            for _ in 0..n {
                v.push(f32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()));
            }
            arrays.push(v);
        }
        if c != bytes.len() {
            return Err("trailing bytes in the query checkpoint".into());
        }
        let expected = [
            self.r_master.len(),
            self.w_master.len(),
            self.a_master.len(),
            self.b_master.len(),
            self.r_m.len(),
            self.r_v.len(),
            self.w_m.len(),
            self.w_v.len(),
            self.a_m.len(),
            self.a_v.len(),
            self.b_m.len(),
            self.b_v.len(),
        ];
        for (k, want) in expected.iter().enumerate() {
            if arrays[k].len() != *want {
                return Err(format!("query checkpoint array {k} length mismatch"));
            }
        }
        let mut it = arrays.into_iter();
        self.r_master = it.next().unwrap();
        self.w_master = it.next().unwrap();
        self.a_master = it.next().unwrap();
        self.b_master = it.next().unwrap();
        self.r_m = it.next().unwrap();
        self.r_v = it.next().unwrap();
        self.w_m = it.next().unwrap();
        self.w_v = it.next().unwrap();
        self.a_m = it.next().unwrap();
        self.a_v = it.next().unwrap();
        self.b_m = it.next().unwrap();
        self.b_v = it.next().unwrap();
        self.rw_updates = rw_updates;
        self.ab_updates = ab_updates;
        self.pass = pass;
        self.data_identity = data_identity;
        Ok(())
    }
}

/// Ternarize a row-major master with the declared `|v| >= 0.5` rule and pack two bits per weight,
/// at a fixed uniform shift.
pub fn pack_fixed(master: &[f32], rows: usize, cols: usize, shift: u32) -> (Vec<u8>, Vec<u32>) {
    assert_eq!(master.len(), rows * cols);
    let mut packed = vec![0u8; (rows * cols).div_ceil(4)];
    for (flat, &v) in master.iter().enumerate() {
        let code = if v >= TERNA_TIE {
            CODE_POS
        } else if v <= -TERNA_TIE {
            CODE_NEG
        } else {
            CODE_ZERO
        };
        packed[flat >> 2] |= code << (2 * (flat & 3) as u32);
    }
    (packed, vec![shift; rows])
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_geometric::learner::prefix_artifact::parent_hash_convention;
    use crate::native_geometric::learner::prefix_state::palette;
    use crate::native_geometric::learner::prior_learning::{bias_codes_from_counts, Config, RADIX};

    /// A constant parent: no context heads at all, so only the frozen bias and the residual score.
    fn constant_parent(vocab: usize, bias: Vec<i8>) -> PriorCore {
        let cfg = Config {
            vocab,
            dv: 128,
            norm_bits: 6,
            f_bits: 10,
            bias_scale_bits: 10,
        };
        cfg.validate().expect("fixture config");
        PriorCore {
            cfg,
            elements: (0..vocab).map(|t| (t % RADIX) as u16).collect(),
            e_old: ternary_from_codes(&vec![0i32; (vocab + 1) * 128], vocab + 1, 128, 0).unwrap(),
            e_new: ternary_from_codes(&vec![0i32; vocab * 128], vocab, 128, 0).unwrap(),
            w_o: ternary_from_codes(&vec![0i32; vocab * 128], vocab, 128, 0).unwrap(),
            bias_codes: bias,
        }
    }

    /// The unigram parent from the recorded real-text pipeline, for shape-correct trainer tests.
    fn uniform_parent(vocab: usize) -> PriorCore {
        let counts = vec![1u64; vocab];
        let bias = bias_codes_from_counts(&counts, vocab as u64, vocab);
        constant_parent(vocab, bias)
    }

    fn fixture_digest() -> [u8; 32] {
        parent_hash_convention(b"query-read-fixture").0
    }

    /// A non-placeholder tokenizer identity for fixtures.
    fn fixture_tokenizer_digest() -> [u8; 32] {
        parent_hash_convention(b"query-read-fixture-tokenizer").0
    }

    const WINDOWS: [[u32; 4]; 4] = [[0, 2, 0, 1], [0, 2, 1, 0], [1, 2, 0, 0], [1, 2, 1, 1]];
    const TARGETS: [u32; 4] = [1, 0, 0, 1];

    /// The authored witness: frozen A/R/W with an authored or identity-preferred B.
    fn authored_full(arm: QueryArm, authored_b: bool) -> (QueryTrainer, Palette, ExactGroupTable) {
        let parent = constant_parent(4, vec![0, -1, -7, -7]);
        let table = ExactGroupTable::build().expect("exact table");
        let pal = palette().clone();
        let mut tr = QueryTrainer::new(
            parent,
            pal.clone(),
            table.clone(),
            arm,
            13,
            4,
            512,
            64,
            fixture_digest(),
            fixture_tokenizer_digest(),
        )
        .expect("trainer");
        let e = table.identity as usize;
        let a = pal.elements[1] as usize;
        let b = pal.elements[2] as usize;
        let ab = compose(&table, a, b);
        assert_ne!(a, e);
        assert_ne!(b, e);
        assert_ne!(ab, e);
        assert_ne!(
            compose(&table, a, b),
            compose(&table, b, a),
            "a and b must not commute"
        );

        let mut r = vec![0.0f32; GROUP_ORDER * READER_WIDTH];
        for j in 0..8 {
            r[e * READER_WIDTH + j] = 1.0;
            r[ab * READER_WIDTH + j] = 1.0;
            r[a * READER_WIDTH + j] = -1.0;
            r[b * READER_WIDTH + j] = -1.0;
        }
        tr.r_master = r;
        let mut w = vec![0.0f32; 4 * READER_WIDTH];
        for j in 0..8 {
            w[READER_WIDTH + j] = 1.0;
        }
        tr.w_master = w;
        // A(0) = e, A(1) = a, separator/unused tokens carry the identity slot.
        let e_slot = tr.identity_slot() as usize;
        let mut a_logits = vec![0.0f32; 4 * PALETTE_SIZE];
        a_logits[e_slot] = 0.1;
        a_logits[PALETTE_SIZE + 1] = 0.1;
        a_logits[2 * PALETTE_SIZE + e_slot] = 0.1;
        a_logits[3 * PALETTE_SIZE + e_slot] = 0.1;
        tr.a_master = a_logits;
        if authored_b {
            // B(0) = e, B(1) = gamma[2]; the remaining tokens carry the identity slot. The identity
            // preference is removed from token 1 so the authored code is unambiguous: leaving both
            // slots at 0.1 would resolve by lowest-ID tie-break to the identity.
            let mut b_logits = vec![0.0f32; 4 * PALETTE_SIZE];
            for t in 0..4 {
                b_logits[t * PALETTE_SIZE + e_slot] = 0.1;
            }
            b_logits[PALETTE_SIZE + e_slot] = 0.0;
            b_logits[PALETTE_SIZE + 2] = 0.1;
            tr.b_master = b_logits;
        }
        tr.initial_a_codes = tr.hard_a_codes();
        tr.initial_b_codes = tr.hard_b_codes();
        (tr, pal, table)
    }

    /// The authored constructive witness (B(0) = e, B(1) = gamma[2]).
    fn authored(arm: QueryArm) -> (QueryTrainer, Palette, ExactGroupTable) {
        authored_full(arm, true)
    }

    /// The same frozen A/R/W with B re-initialised identity-preferred for every token.
    fn authored_learnable(arm: QueryArm) -> (QueryTrainer, Palette, ExactGroupTable) {
        authored_full(arm, false)
    }

    fn windows() -> Vec<Vec<u32>> {
        WINDOWS.iter().map(|w| w.to_vec()).collect()
    }

    /// Score only `i = 2` of each authored window and return `(prediction, class1-minus-class0)`.
    fn score_fixture(hard: &QueryHard) -> Vec<(usize, i32)> {
        WINDOWS
            .iter()
            .map(|w| {
                let toks = w.to_vec();
                let rows = hard.read_path(&toks, 2);
                let prev = toks[1] as usize;
                let cur = toks[2] as usize;
                let z = hard.int_logits(prev, cur, rows.map(|p| (p.first, p.second)));
                let mut best = 0usize;
                for r in 1..z.len() {
                    if z[r] > z[best] {
                        best = r;
                    }
                }
                (best, z[1] - z[0])
            })
            .collect()
    }

    // -- representation and masking -----------------------------------------

    #[test]
    fn palette_slot_and_group_identity_are_resolved_through_the_bound_mapping() {
        let pal = palette();
        let t = ExactGroupTable::build().unwrap();
        let slot = pal
            .elements
            .iter()
            .position(|&x| x == t.identity)
            .expect("the palette contains the identity");
        assert_eq!(slot, 0, "the palette places the identity first");
        // The identity group ID is a historical state index, not a palette slot.
        assert_eq!(t.identity, 1, "historically the identity has group ID 1");
        assert_ne!(
            pal.elements[0] as usize,
            0usize.max(t.identity as usize) - 1
        );
        assert_eq!(pal.elements[slot], t.identity);
    }

    #[test]
    fn older_prefix_excludes_the_local_pair_and_masks_early_positions() {
        let (tr, _, _) = authored(QueryArm::Q);
        let core = tr.hard_core().unwrap();
        let toks: Vec<u32> = vec![3, 3, 1, 2, 0, 3, 2, 1];
        assert!(core.older_state(&toks, 0).is_none());
        assert!(core.older_state(&toks, 1).is_none());
        // i = 2 sees exactly tokens[0]; the local pair (tokens[1], tokens[2]) is excluded.
        assert_eq!(core.older_state(&toks, 2), Some(core.write_state(3)));
        let a3 = core.write_state(3);
        assert_eq!(
            core.older_state(&toks, 3),
            Some(compose(&core.table, a3, core.write_state(3)))
        );
        // A 200-token window still contributes at most 62 older tokens.
        let long: Vec<u32> = (0..200u32).map(|x| x % 4).collect();
        let chain = core.read_path(&long, 150).unwrap().chain;
        assert_eq!(chain.len(), MAX_OLDER);
        assert_eq!(chain.first(), Some(&(150 - 63)));
        assert!(chain.iter().all(|j| *j < 149));
    }

    #[test]
    fn step_zero_reproduces_the_frozen_parent_exactly() {
        for arm in [QueryArm::Q, QueryArm::S, QueryArm::L] {
            let tr = QueryTrainer::new(
                uniform_parent(16),
                palette().clone(),
                ExactGroupTable::build().unwrap(),
                arm,
                13,
                4,
                512,
                64,
                fixture_digest(),
                fixture_tokenizer_digest(),
            )
            .unwrap();
            let core = tr.hard_core().unwrap();
            let toks: Vec<u32> = vec![1, 5, 9, 13, 2, 7, 11, 3];
            for i in 2..toks.len() - 1 {
                let rows = core.read_path(&toks, i).map(|p| (p.first, p.second));
                let z = core.int_logits(toks[i - 1] as usize, toks[i] as usize, rows);
                let want = core
                    .parent
                    .int_logits(toks[i - 1] as usize, toks[i] as usize, true);
                assert_eq!(
                    z, want,
                    "{arm:?} at i={i}: W=0 must reproduce the parent exactly"
                );
            }
            assert!(tr.hard_core().unwrap().wg.packed().iter().all(|b| *b == 0));
        }
    }

    #[test]
    fn future_tokens_cannot_change_a_prediction() {
        let (tr, _, _) = authored(QueryArm::Q);
        let core = tr.hard_core().unwrap();
        let a: Vec<u32> = vec![1, 2, 0, 1, 3, 2, 0];
        let mut b = a.clone();
        b.extend_from_slice(&[3, 3, 1, 0, 2]);
        for i in 2..a.len() - 1 {
            let ra = core.read_path(&a, i);
            let rb = core.read_path(&b, i);
            assert_eq!(ra, rb, "i={i}");
        }
        // And a changed *future* token leaves the earlier prediction identical.
        let i = 3;
        let z0 = core.int_logits(
            a[i - 1] as usize,
            a[i] as usize,
            core.read_path(&a, i).map(|p| (p.first, p.second)),
        );
        let mut a2 = a.clone();
        a2[i + 1] = 0;
        let z1 = core.int_logits(
            a2[i - 1] as usize,
            a2[i] as usize,
            core.read_path(&a2, i).map(|p| (p.first, p.second)),
        );
        assert_eq!(z0, z1);
    }

    #[test]
    fn local_arm_is_invariant_under_an_older_prefix_change() {
        let (tr, _, _) = authored(QueryArm::L);
        let core = tr.hard_core().unwrap();
        let base: Vec<u32> = vec![0, 1, 2, 3, 0, 1, 2, 3];
        let changed: Vec<u32> = vec![3, 2, 1, 0, 3, 2, 1, 0];
        for i in 4..base.len() - 1 {
            // Positions 4.. share the local pair under this construction only after index 4.
            if base[i - 1..=i] != changed[i - 1..=i] {
                continue;
            }
            assert_eq!(
                core.read_path(&base, i),
                core.read_path(&changed, i),
                "i={i}"
            );
        }
        // The stronger statement: whatever the older prefix, only the pair matters for L.
        let a: Vec<u32> = vec![1, 1, 1, 1, 3, 0];
        let b: Vec<u32> = vec![2, 2, 2, 2, 3, 0];
        assert_eq!(a[4..=5], b[4..=5]);
        assert_eq!(core.read_path(&a, 5), core.read_path(&b, 5));
        let pa = core.read_path(&a, 5).unwrap();
        assert_eq!(pa.chain, vec![4, 5]);
    }

    #[test]
    fn q_and_s_agree_exactly_through_warmup() {
        // The real trainer's declared initialisation: A one seeded slot per token, B the identity
        // slot for every token, at 0.1 versus 0.
        let mut q = QueryTrainer::new(
            uniform_parent(16),
            palette().clone(),
            ExactGroupTable::build().unwrap(),
            QueryArm::Q,
            13,
            4,
            512,
            64,
            fixture_digest(),
            fixture_tokenizer_digest(),
        )
        .unwrap();
        let mut s = q.clone();
        s.arm = QueryArm::S;
        let wins: Vec<Vec<u32>> = vec![vec![1, 5, 9, 13], vec![2, 7, 11, 3]];
        let a_before = q.hard_a_codes();
        let b_before = q.hard_b_codes();
        let mut cq = ParentCache::default();
        let mut cs = ParentCache::default();
        for _ in 0..64 {
            let lq = q.train_batch(&wins, &mut cq, false).unwrap();
            let ls = s.train_batch(&wins, &mut cs, false).unwrap();
            assert!(
                (lq - ls).abs() < 1e-12,
                "Q/S warm-up loss must agree: {lq} vs {ls}"
            );
        }
        assert_eq!(q.r_master, s.r_master, "Q/S R/W warm-up must be identical");
        assert_eq!(q.w_master, s.w_master);
        assert_eq!(q.rw_updates, 64);
        assert_eq!(q.ab_updates, 0);
        assert_eq!(
            q.hard_a_codes(),
            a_before,
            "A must be frozen through warm-up"
        );
        assert_eq!(
            q.hard_b_codes(),
            b_before,
            "B must be frozen through warm-up"
        );
        let id_slot = q.identity_slot();
        assert!(
            q.hard_b_codes().iter().all(|c| *c == id_slot),
            "identity-preferred B starts at the identity slot for every token"
        );
    }

    // -- derivatives ---------------------------------------------------------

    /// A continuous multilinear surrogate with the hard anchors held fixed.
    struct Surrogate {
        zp: Vec<f64>,
        r: Vec<f64>,
        w: Vec<f64>,
        first: usize,
        second: usize,
        target: usize,
        scale: f64,
    }

    impl Surrogate {
        fn z(&self, v: usize) -> f64 {
            let mut s = 0.0;
            for j in 0..READER_WIDTH {
                s += self.w[v * READER_WIDTH + j]
                    * (self.r[self.first * READER_WIDTH + j]
                        + self.r[self.second * READER_WIDTH + j]);
            }
            self.zp[v] + 128.0 * s
        }
        fn loss(&self) -> f64 {
            let v = self.zp.len();
            let mut max = f64::NEG_INFINITY;
            for r in 0..v {
                max = max.max(self.z(r) * self.scale);
            }
            let mut sum = 0.0;
            for r in 0..v {
                sum += (self.z(r) * self.scale - max).exp();
            }
            (max + sum.ln() - self.z(self.target) * self.scale) / std::f64::consts::LN_2
        }
        fn d(&self) -> Vec<f64> {
            let v = self.zp.len();
            let mut max = f64::NEG_INFINITY;
            for r in 0..v {
                max = max.max(self.z(r) * self.scale);
            }
            let mut p = vec![0.0; v];
            let mut sum = 0.0;
            for r in 0..v {
                p[r] = (self.z(r) * self.scale - max).exp();
                sum += p[r];
            }
            (0..v)
                .map(|r| {
                    (p[r] / sum - if r == self.target { 1.0 } else { 0.0 }) * self.scale
                        / std::f64::consts::LN_2
                })
                .collect()
        }
    }

    fn surrogate_for(tr: &QueryTrainer, toks: &[u32]) -> Surrogate {
        let core = tr.hard_core().unwrap();
        let i = 2;
        let path = core.read_path(toks, i).expect("path");
        let zp = core
            .parent
            .int_logits(toks[i - 1] as usize, toks[i] as usize, true);
        Surrogate {
            zp: zp.iter().map(|x| *x as f64).collect(),
            r: tr.r_master.iter().map(|x| *x as f64).collect(),
            w: tr.w_master.iter().map(|x| *x as f64).collect(),
            first: path.first,
            second: path.second,
            target: toks[3] as usize,
            scale: (-(core.parent.cfg.f_bits as f64)).exp2(),
        }
    }

    #[test]
    fn reader_and_output_gradients_match_a_continuous_surrogate() {
        let (tr, _, _) = authored(QueryArm::Q);
        // Window 3 has q = a and b = gamma[2], so q*b = a*b is not the identity row.
        let toks: Vec<u32> = WINDOWS[3].to_vec();
        let s = surrogate_for(&tr, &toks);
        assert_ne!(
            s.first, s.second,
            "the two reader rows must be distinct here"
        );
        let d = s.d();

        // Analytic output gradient: grad_W[v,j] = 128 * d_v * (R[first,j] + R[second,j]).
        let mut want_w = vec![0.0f64; s.w.len()];
        for v in 0..s.zp.len() {
            for j in 0..READER_WIDTH {
                want_w[v * READER_WIDTH + j] = 128.0
                    * d[v]
                    * (s.r[s.first * READER_WIDTH + j] + s.r[s.second * READER_WIDTH + j]);
            }
        }
        // Analytic reader gradient: g_j = 128 * sum_v d_v * W[v,j], applied to BOTH rows.
        let mut g = vec![0.0f64; READER_WIDTH];
        for v in 0..s.zp.len() {
            for j in 0..READER_WIDTH {
                g[j] += 128.0 * d[v] * s.w[v * READER_WIDTH + j];
            }
        }

        // Finite differences of the explicitly defined continuous surrogate.
        let eps = 1e-6;
        {
            let (v, j) = (1usize, 0usize);
            let mut plus = Surrogate {
                zp: s.zp.clone(),
                r: s.r.clone(),
                w: s.w.clone(),
                first: s.first,
                second: s.second,
                target: s.target,
                scale: s.scale,
            };
            plus.w[v * READER_WIDTH + j] += eps;
            let lp = plus.loss();
            plus.w[v * READER_WIDTH + j] -= 2.0 * eps;
            let lm = plus.loss();
            let fd_w = (lp - lm) / (2.0 * eps);
            assert!(
                (fd_w - want_w[v * READER_WIDTH + j]).abs() < 1e-7,
                "output FD {fd_w} vs analytic {}",
                want_w[v * READER_WIDTH + j]
            );
        }
        {
            let j = 2usize;
            let mut plus = Surrogate {
                zp: s.zp.clone(),
                r: s.r.clone(),
                w: s.w.clone(),
                first: s.first,
                second: s.second,
                target: s.target,
                scale: s.scale,
            };
            plus.r[s.first * READER_WIDTH + j] += eps;
            let lp = plus.loss();
            plus.r[s.first * READER_WIDTH + j] -= 2.0 * eps;
            let lm = plus.loss();
            let fd = (lp - lm) / (2.0 * eps);
            assert!(
                (fd - g[j]).abs() < 1e-7,
                "reader FD {fd} vs analytic {}",
                g[j]
            );
        }

        // The implementation agrees with the analytic expressions on the same occurrence.
        let mut cache = ParentCache::default();
        let raw = tr.compute_grads(&vec![toks.clone()], &mut cache).unwrap();
        assert_eq!(
            raw.scored, 3,
            "the other two positions are scored but carry no read"
        );
        for v in 0..s.zp.len() {
            for j in 0..READER_WIDTH {
                let got = raw.gw[v * READER_WIDTH + j] as f64;
                assert!(
                    (got - want_w[v * READER_WIDTH + j]).abs() < 1e-4,
                    "grad_W[{v},{j}]: implementation {got} vs analytic {}",
                    want_w[v * READER_WIDTH + j]
                );
            }
        }
        for j in 0..READER_WIDTH {
            let f = raw.gr[s.first * READER_WIDTH + j] as f64;
            let t = raw.gr[s.second * READER_WIDTH + j] as f64;
            assert!(
                (f - g[j]).abs() < 1e-4,
                "grad_R[first,{j}]: {f} vs {}",
                g[j]
            );
            assert!(
                (t - g[j]).abs() < 1e-4,
                "grad_R[second,{j}]: {t} vs {}",
                g[j]
            );
        }
    }

    #[test]
    fn categorical_credit_matches_the_state_relaxation_and_distinguishes_a_from_b() {
        let (tr, pal, table) = authored(QueryArm::Q);
        let core = tr.hard_core().unwrap();
        let toks: Vec<u32> = WINDOWS[3].to_vec();
        let s = surrogate_for(&tr, &toks);
        let path = core.read_path(&toks, 2).unwrap();
        assert_eq!(
            path.chain,
            vec![0],
            "the older prefix of i=2 is token 0 alone"
        );
        let b = core.query_state(toks[2]);
        let d = s.d();
        // g is the reader adjoint, exactly as the trainer computes it.
        let mut g = vec![0.0f64; READER_WIDTH];
        for v in 0..s.zp.len() {
            for j in 0..READER_WIDTH {
                g[j] += 128.0 * d[v] * s.w[v * READER_WIDTH + j];
            }
        }
        // Terminal-state adjoint u(s) = g . R[s*b].
        let u = |state: usize| -> f64 {
            let h = compose(&table, state, b);
            (0..READER_WIDTH)
                .map(|j| g[j] * s.r[h * READER_WIDTH + j])
                .sum()
        };
        // Query credit for slot k is the read of the transported terminal, `g . R[terminal*gamma[k]]`.
        // This is deliberately not `u(terminal*gamma[k])`; the two agree only at `b = e`.
        let credit = |k: usize| -> f64 {
            let h = compose(
                &table,
                path.states.last().copied().unwrap(),
                pal.elements[k] as usize,
            );
            (0..READER_WIDTH)
                .map(|j| g[j] * s.r[h * READER_WIDTH + j])
                .sum()
        };
        let mut cache = ParentCache::default();
        let raw = tr.compute_grads(&vec![toks.clone()], &mut cache).unwrap();
        assert_eq!(raw.scored, 3);
        for k in 0..PALETTE_SIZE {
            let want = credit(k);
            let got = raw.db[toks[2] as usize * PALETTE_SIZE + k] as f64;
            assert!(
                (got - want).abs() < 1e-4,
                "query credit k={k}: {got} vs {want}"
            );
        }
        // A-chain credit: the single older token's write slot is credited at u(identity*gamma[k]).
        for k in 0..PALETTE_SIZE {
            let want = u(pal.elements[k] as usize);
            let got = raw.da[toks[0] as usize * PALETTE_SIZE + k] as f64;
            assert!((got - want).abs() < 1e-4, "A credit k={k}: {got} vs {want}");
        }
        // A and B gradients are different objects and both are nonzero here.
        assert!(raw.da.iter().any(|x| *x != 0.0), "A credit must be live");
        assert!(raw.db.iter().any(|x| *x != 0.0), "B credit must be live");
        let a_part: Vec<f32> = raw.da.clone();
        let b_part: Vec<f32> = raw.db.clone();
        assert_ne!(a_part, b_part);
    }

    /// The Q-S algebra at identical parameters, and the two-coincident-row credit rule.
    ///
    /// `r_Q - r_S = 128 * W(R[q*b] - R[q] - R[b] + R[e])` is the architectural distinction, so it is
    /// checked as an identity on the bound codes rather than as "some numbers differ".
    #[test]
    fn mixed_difference_is_the_declared_algebra_and_coincident_rows_receive_two_credits() {
        let (tr, _pal, table) = authored(QueryArm::Q);
        let core = tr.hard_core().unwrap();
        let e = table.identity as usize;
        // Window 3: q = A(1) = a and b = gamma[2], so q*b = a*b.
        let toks: Vec<u32> = WINDOWS[3].to_vec();
        let path = core.read_path(&toks, 2).unwrap();
        let a_state = core.write_state(1);
        let b_state = core.query_state(1);
        assert_eq!(path.first, compose(&table, a_state, b_state));
        assert_ne!(
            path.first, path.second,
            "window 3 keeps the two rows distinct"
        );
        let joint = core.residual_scores(path.first, path.second);
        let sep = core.residual_scores(a_state, b_state);
        let mut h = vec![0i32; READER_WIDTH];
        for j in 0..READER_WIDTH {
            h[j] = (core.reader.weight(path.first, j)
                - core.reader.weight(a_state, j)
                - core.reader.weight(b_state, j)
                + core.reader.weight(e, j))
                << core.reader.shift(path.first);
        }
        let expected = core.wg.forward_i32(&h);
        for v in 0..joint.len() {
            assert_eq!(joint[v] - sep[v], expected[v], "row {v}");
        }
        assert!(joint.iter().any(|x| *x != 0), "the joint read must be live");
        assert_ne!(joint, sep, "the joint and separable reads must differ");

        // Coincident selected rows: window [0,2,0,1] has q = e and b = e, so first == second == e.
        let co: Vec<u32> = WINDOWS[0].to_vec();
        let co_path = core.read_path(&co, 2).unwrap();
        assert_eq!(co_path.first, e);
        assert_eq!(co_path.second, e);
        let raw = tr
            .compute_grads(&vec![co.clone()], &mut ParentCache::default())
            .unwrap();
        let want = hard_reader_adjoint(&core, &co, 2, TARGETS[0]);
        for j in 0..READER_WIDTH {
            let got = raw.gr[e * READER_WIDTH + j] as f64;
            assert!(
                (got - 2.0 * want[j]).abs() < 1e-3 * want[j].abs().max(1.0),
                "coincident rows must receive two credits at j={j}: {got} vs {}",
                2.0 * want[j]
            );
        }
    }

    /// The reader adjoint `g_j = 128 * sum_v d_v * W[v,j]`, recomputed from the hard integer forward
    /// rather than read out of the trainer.
    fn hard_reader_adjoint(core: &QueryHard, toks: &[u32], i: usize, target: u32) -> Vec<f64> {
        let rows = core.read_path(toks, i).map(|p| (p.first, p.second));
        let z = core.int_logits(toks[i - 1] as usize, toks[i] as usize, rows);
        let scale = (-(core.parent.cfg.f_bits as f64)).exp2();
        let mut max = f64::NEG_INFINITY;
        for &v in &z {
            max = max.max(v as f64 * scale);
        }
        let mut p: Vec<f64> = z.iter().map(|v| (*v as f64 * scale - max).exp()).collect();
        let s: f64 = p.iter().sum();
        for x in p.iter_mut() {
            *x /= s;
        }
        let t = target as usize;
        let inv_ln2 = 1.0 / std::f64::consts::LN_2;
        let d: Vec<f64> = (0..z.len())
            .map(|v| (p[v] - if v == t { 1.0 } else { 0.0 }) * scale * inv_ln2)
            .collect();
        (0..READER_WIDTH)
            .map(|j| {
                (0..z.len())
                    .map(|v| 128.0 * d[v] * core.wg.weight(v, j) as f64)
                    .sum()
            })
            .collect()
    }

    // -- artifact and checkpoint --------------------------------------------

    #[test]
    fn artifact_round_trips_and_rejects_malformed_input() {
        let (tr, _, _) = authored(QueryArm::Q);
        let parent = tr.parent.clone();
        let digest = fixture_digest();
        let tok = fixture_tokenizer_digest();
        let hard = tr.hard_core().unwrap();
        let bytes = hard.to_bytes();
        assert_eq!(&bytes[..4], QUERY_ARTIFACT_MAGIC);
        let reload = QueryHard::from_bytes(&bytes, &parent, &digest, &tok).expect("round trip");
        assert_eq!(reload.a_codes, hard.a_codes);
        assert_eq!(reload.b_codes, hard.b_codes);
        assert_eq!(reload.arm, QueryArm::Q);
        assert_eq!(reload.table.product, hard.table.product);
        assert_eq!(score_fixture(&reload), score_fixture(&hard));

        // A CPX2 prefix artifact is rejected by name, never reinterpreted.
        let mut cpx2 = PREFIX_ARTIFACT_MAGIC.to_vec();
        cpx2.extend_from_slice(&2u32.to_le_bytes());
        cpx2.resize(bytes.len(), 0);
        assert!(QueryHard::from_bytes(&cpx2, &parent, &digest, &tok).is_err());
        assert!(QueryHard::from_bytes(&bytes, &parent, &[9u8; 32], &tok).is_err());
        assert!(QueryHard::from_bytes(&bytes[..bytes.len() - 1], &parent, &digest, &tok).is_err());
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(QueryHard::from_bytes(&trailing, &parent, &digest, &tok).is_err());
        // A corrupted output-row shift (the old loader's per-row freedom) is rejected.
        let mut bad = bytes.clone();
        let n = bad.len();
        bad[n - 1] = 0xFF;
        assert!(QueryHard::from_bytes(&bad, &parent, &digest, &tok).is_err());
    }

    #[test]
    fn checkpoint_round_trips_and_resume_rejects_a_different_arm() {
        let (mut q, _, _) = authored(QueryArm::Q);
        let (s, _, _) = authored(QueryArm::S);
        let wins = windows();
        let mut cache = ParentCache::default();
        for _ in 0..3 {
            q.train_batch(&wins, &mut cache, false).unwrap();
        }
        let bytes = q.checkpoint_bytes();
        let mut clone = s.clone();
        assert!(
            clone.resume_from(&bytes).is_err(),
            "arm mismatch must be fatal"
        );
        let mut resumed = q.clone();
        resumed.r_master[0] = 7.0;
        resumed.w_master[3] = -7.0;
        resumed.rw_updates = 999;
        resumed.resume_from(&bytes).expect("resume");
        assert_eq!(resumed.r_master, q.r_master);
        assert_eq!(resumed.w_master, q.w_master);
        assert_eq!(resumed.rw_updates, q.rw_updates);
        assert_eq!(resumed.ab_updates, q.ab_updates);
        assert_eq!(resumed.checkpoint_bytes(), bytes);
    }

    // -- representability and learned-query credit ---------------------------

    #[test]
    fn constructive_query_interaction_witness_after_export_and_reload() {
        let (tr, _, _) = authored(QueryArm::Q);
        let parent = tr.parent.clone();
        let digest = fixture_digest();
        let tok = fixture_tokenizer_digest();
        let hard = tr.hard_core().unwrap();
        let reload = QueryHard::from_bytes(&hard.to_bytes(), &parent, &digest, &tok).unwrap();
        let scored = score_fixture(&reload);
        let diffs: Vec<i32> = scored.iter().map(|(_, d)| *d).collect();
        assert_eq!(
            diffs,
            vec![1024, -1024, -1024, 1024],
            "the strict checkerboard must be +,-,-,+"
        );
        for (k, (pred, _)) in scored.iter().enumerate() {
            assert_eq!(
                *pred as u32, TARGETS[k],
                "window {k} must predict its target"
            );
        }
        // D00 + D11 == D01 + D10 is impossible for these strict signs under any separable binary
        // log-odds function, which is the representability claim.
        assert_ne!(diffs[0] + diffs[3], diffs[1] + diffs[2]);
    }

    #[test]
    fn learned_query_credit_fixture_passes_and_survives_interventions() {
        let (mut tr, _, _) = authored_learnable(QueryArm::Q);
        let wins = windows();
        let mut cache = ParentCache::default();
        let base = score_fixture(&tr.hard_core().unwrap());
        assert_eq!(
            base.iter().filter(|(p, _)| *p as u32 == 0).count() >= 2,
            true
        );
        let correct_before = base
            .iter()
            .enumerate()
            .filter(|(k, (p, _))| *p as u32 == TARGETS[*k])
            .count();
        assert_eq!(correct_before, 2, "the identity-preferred B starts at 2/4");

        let mut losses = Vec::new();
        for _ in 0..128 {
            losses.push(
                tr.train_batch_blocks(&wins, &mut cache, false, false, true)
                    .unwrap(),
            );
        }
        assert!(
            losses.last().unwrap() <= &losses[0],
            "training only B must not increase the selected loss"
        );
        assert_eq!(
            tr.rw_updates, 0,
            "R/W must be untouched in the B-only fixture"
        );
        assert_eq!(tr.ab_updates, 128);

        let learned = tr.hard_core().unwrap();
        let scored = score_fixture(&learned);
        let correct: Vec<usize> = scored
            .iter()
            .enumerate()
            .filter(|(k, (p, _))| *p as u32 == TARGETS[*k])
            .map(|(k, _)| k)
            .collect();
        assert_eq!(correct.len(), 4, "the learned B must reach 4/4: {scored:?}");

        // Swap the source token; query and target unchanged.
        let swapped_source: Vec<Vec<u32>> = vec![
            vec![1, 2, 0, 1],
            vec![1, 2, 1, 0],
            vec![0, 2, 0, 0],
            vec![0, 2, 1, 1],
        ];
        let n = swapped_source
            .iter()
            .enumerate()
            .filter(|(k, w)| {
                let rows = learned.read_path(w, 2);
                let z = learned.int_logits(
                    w[1] as usize,
                    w[2] as usize,
                    rows.map(|p| (p.first, p.second)),
                );
                argmax_full(&z) as u32 == TARGETS[*k]
            })
            .count();
        assert_eq!(
            n, 0,
            "swapping the source must destroy all four predictions"
        );

        // Swap the query token; source and target unchanged.
        let swapped_query: Vec<Vec<u32>> = vec![
            vec![0, 2, 1, 1],
            vec![0, 2, 0, 0],
            vec![1, 2, 1, 1],
            vec![1, 2, 0, 0],
        ];
        let m = swapped_query
            .iter()
            .enumerate()
            .filter(|(k, w)| {
                let rows = learned.read_path(w, 2);
                let z = learned.int_logits(
                    w[1] as usize,
                    w[2] as usize,
                    rows.map(|p| (p.first, p.second)),
                );
                argmax_full(&z) as u32 == TARGETS[*k]
            })
            .count();
        assert_eq!(m, 0, "swapping the query must destroy all four predictions");

        // Read disabled: the constant parent, correct on exactly the two class-0 targets.
        let disabled = WINDOWS
            .iter()
            .enumerate()
            .filter(|(k, w)| {
                let z = learned.int_logits(w[1] as usize, w[2] as usize, None);
                argmax_full(&z) as u32 == TARGETS[*k]
            })
            .count();
        assert_eq!(
            disabled, 2,
            "read-disabled output must be the constant parent, 2/4"
        );
    }

    // -- tokenizer binding, legacy import and the inference seam -------------

    /// Production export/load requires and round-trips the real raw tokenizer identity.
    #[test]
    fn tokenizer_identity_round_trips_and_a_mismatch_is_rejected() {
        let (tr, _, _) = authored(QueryArm::S);
        let parent = tr.parent.clone();
        let digest = fixture_digest();
        let tok = fixture_tokenizer_digest();
        assert_ne!(tok, ZERO_DIGEST);
        let hard = tr.hard_core().unwrap();
        assert_eq!(
            hard.tokenizer_digest, tok,
            "the trainer digest must reach the artifact"
        );
        let bytes = hard.to_bytes();
        assert_eq!(&bytes[73..105], &tok, "raw digest sits at offsets 73..105");
        let reload = QueryHard::from_bytes(&bytes, &parent, &digest, &tok).expect("round trip");
        assert_eq!(reload.tokenizer_digest, tok);
        // A wrong expected tokenizer identity is rejected rather than adopted.
        let mut other = tok;
        other[0] ^= 0xFF;
        assert!(QueryHard::from_bytes(&bytes, &parent, &digest, &other).is_err());
        // The public constructor refuses the placeholder.
        let err = QueryHard::from_parts(
            parent.clone(),
            hard.reader.clone(),
            hard.wg.clone(),
            hard.a_codes.clone(),
            hard.b_codes.clone(),
            hard.palette.clone(),
            hard.arm,
            hard.table.clone(),
            digest,
            ZERO_DIGEST,
        )
        .unwrap_err();
        assert!(err.contains("placeholder"), "unexpected error: {err}");
    }

    /// The hash-pinned legacy zero-digest files load only through the restricted import, and the
    /// repair re-export changes exactly the 32 identity bytes.
    #[test]
    fn legacy_zero_digest_imports_only_through_the_restricted_path_and_repairs_cleanly() {
        let (tr, _, _) = authored(QueryArm::S);
        let parent = tr.parent.clone();
        let digest = fixture_digest();
        let tok = fixture_tokenizer_digest();
        let hard = tr.hard_core().unwrap();
        let good = hard.to_bytes();
        let mut legacy = good.clone();
        for b in legacy[73..105].iter_mut() {
            *b = 0;
        }
        // Production load refuses the placeholder; the restricted import accepts it.
        assert!(QueryHard::from_bytes(&legacy, &parent, &digest, &tok).is_err());
        let imported = QueryHard::import_legacy(&legacy, &parent, &digest).expect("legacy import");
        assert_eq!(imported.tokenizer_digest, ZERO_DIGEST);
        assert_eq!(imported.a_codes, hard.a_codes);
        assert_eq!(imported.b_codes, hard.b_codes);
        assert_eq!(imported.table.product, hard.table.product);
        // Re-export with the real identity: same length, differences only in 73..105.
        let repaired = QueryHard::from_parts(
            parent.clone(),
            imported.reader.clone(),
            imported.wg.clone(),
            imported.a_codes.clone(),
            imported.b_codes.clone(),
            imported.palette.clone(),
            imported.arm,
            imported.table.clone(),
            digest,
            tok,
        )
        .expect("repair");
        let fixed = repaired.to_bytes();
        assert_eq!(fixed.len(), legacy.len());
        let diff: Vec<usize> = (0..fixed.len())
            .filter(|&k| fixed[k] != legacy[k])
            .collect();
        assert!(
            diff.iter().all(|k| (73..105).contains(k)),
            "repair changed bytes outside the tokenizer field: {diff:?}"
        );
        assert_eq!(
            diff.len(),
            32,
            "the repair must change exactly the identity field"
        );
        assert_eq!(&fixed[73..105], &tok);
        let reload = QueryHard::from_bytes(&fixed, &parent, &digest, &tok).expect("repaired load");
        // Every numerical field is unchanged by the metadata repair.
        for (k, (a, b)) in reload
            .int_logits(1, 2, reload.inference_rows(&[1, 2, 3, 0, 1, 2], 3))
            .iter()
            .zip(
                imported
                    .int_logits(1, 2, imported.inference_rows(&[1, 2, 3, 0, 1, 2], 3))
                    .iter(),
            )
            .enumerate()
        {
            assert_eq!(a, b, "logit {k} moved under the metadata repair");
        }
    }

    /// The inference seam folds the older prefix at most once, and the local arm not at all.
    #[test]
    fn inference_seam_folds_history_at_most_once_and_never_for_the_local_arm() {
        let w: Vec<u32> = (0..64u32).map(|k| k % 4).collect();
        let i = 63usize;
        // S: one 62-token fold, no query transport.
        let (tr_s, _, _) = authored(QueryArm::S);
        let s_core = tr_s.hard_core().unwrap();
        reset_compose_calls();
        let _ = s_core.inference_rows(&w, i);
        let s_inference = compose_calls();
        reset_compose_calls();
        let _ = s_core.read_path(&w, i);
        let s_training_path = compose_calls();
        // Q: the same fold plus one query product.
        let (tr_q, _, _) = authored(QueryArm::Q);
        let q_core = tr_q.hard_core().unwrap();
        reset_compose_calls();
        let _ = q_core.inference_rows(&w, i);
        let q_inference = compose_calls();
        // L: two local products plus one query product, and no history fold at all.
        let (tr_l, _, _) = authored(QueryArm::L);
        let l_core = tr_l.hard_core().unwrap();
        reset_compose_calls();
        let _ = l_core.inference_rows(&w, i);
        let l_inference = compose_calls();
        reset_compose_calls();
        let _ = l_core.read_path(&w, i);
        let l_training_path = compose_calls();

        assert_eq!(s_inference, 62, "S folds 62 older tokens once");
        assert_eq!(
            s_training_path, 124,
            "the training path folded history twice"
        );
        assert_eq!(
            q_inference, 63,
            "Q folds 62 older tokens once plus its query product"
        );
        assert_eq!(
            l_inference, 3,
            "L performs only its two local products and one query product"
        );
        assert_eq!(
            l_training_path, 65,
            "the training path folded 62 useless history tokens for L"
        );
        assert!(
            l_inference * 20 < l_training_path,
            "the local seam must be far cheaper than the training chain path"
        );
    }

    /// The inference seam reproduces the training path's rows exactly for every arm.
    #[test]
    fn inference_rows_equal_the_training_path_rows_on_every_position() {
        for arm in [QueryArm::Q, QueryArm::S, QueryArm::L] {
            let (tr, _, _) = authored(arm);
            let core = tr.hard_core().unwrap();
            for w in [
                vec![0u32, 2, 0, 1],
                vec![1u32, 2, 1, 1, 0, 2, 3, 1],
                (0..64u32).map(|k| (k * 7) % 4).collect::<Vec<u32>>(),
            ] {
                for i in 0..w.len() {
                    let a = core.inference_rows(&w, i);
                    let b = core.read_path(&w, i).map(|p| (p.first, p.second));
                    assert_eq!(a, b, "{arm:?} i={i} len={}", w.len());
                }
            }
        }
    }

    /// The public replay boundary rejects invalid tokens and empty prompts instead of clamping.
    #[test]
    fn checked_input_rejects_out_of_vocabulary_tokens_and_empty_prompts() {
        let (tr, _, _) = authored(QueryArm::Q);
        let core = tr.hard_core().unwrap();
        let v = core.parent.cfg.vocab;
        assert!(core.validate_tokens(&[0, 1, 2, 3]).is_ok());
        assert!(core.validate_tokens(&[0, v as u32]).is_err());
        assert!(core.validate_tokens(&[u32::MAX]).is_err());
        assert!(core.inference_rows_checked(&[0, 1, 2, 3], 3).is_ok());
        assert!(core
            .inference_rows_checked(&[0, 1, v as u32, 3], 3)
            .is_err());
        assert!(core.generate_checked(&[], 4).is_err());
        assert!(core.generate_checked(&[0, 1, v as u32], 4).is_err());
        let out = core
            .generate_checked(&[0, 2, 0, 1], 4)
            .expect("valid prompt");
        assert_eq!(out.len(), 4);
        // The unchecked path still clamps, for already-validated internal panels only.
        assert!(core.validate_tokens(&out).is_ok());
    }

    fn argmax_full(z: &[i32]) -> usize {
        let mut best = 0usize;
        for r in 1..z.len() {
            if z[r] > z[best] {
                best = r;
            }
        }
        best
    }
}

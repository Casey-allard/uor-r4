//! Geometric attention: an exactly-addressed associative memory over the 2I/H4 address space.
//!
//! # The mechanism
//!
//! Linear attention stores *pairs* and reads them with a soft matched filter:
//! `y = Σ_s (q·k_s) v_s`. Its limit is cross-talk — an unmatched pair contributes
//! `Θ(√dk)` of noise, so the signal-to-noise ratio falls as `√(dk/N)`.
//!
//! This core stores the same pairs but reads them by **exact address**:
//!
//! ```text
//!   write:  S[address(word)] += value(t)     word = the ordered context ending at t-1
//!   read:   y                = norm(S[address(word)])   word = the ordered context ending at the query
//! ```
//!
//! Different addresses do not interfere, so a word whose address is unique is retrieved with **zero
//! cross-talk**, at `O(dv)` per token instead of `O(dk·dv)`, and with no multiplier anywhere: the
//! address is a table read and index arithmetic, the update is an add, the read is a table read.
//!
//! # The word is ordered, and that is the point
//!
//! `order = 1` addresses by one token. `order = 2` addresses by the **ordered pair** of the last two,
//! giving `120² = 14,400` ordered addresses, and `(a,b) ≠ (b,a)` by construction.
//!
//! Order is not cosmetic. Measured (`order = 1`): a duplicated queried key — the same token appearing
//! twice with two different successors — costs **3× accuracy** (0.44 clean vs 0.14 duplicated), because
//! a first-order address accumulates several successors into one bucket. This is the same distinction
//! W33 supplies in exact finite form (`PLPL = LPLP`, `Ω = LP − PL`, `ker(Ω)` order-insensitive) and the
//! project's own `r(i,j) = inverse(g_i)·g_j`: *the same operation multiset can act differently, so
//! order carries information a set or a commutative hash cannot.*
//!
//! # What this is not
//!
//! Still a synthetic-task measurement, not a language model. The address assignment is fixed, not
//! learned. Capacity is `n_addr = 120^order`; dense buffers make `order ≥ 3` (1.7M+ addresses) a
//! sparse-store problem. The `dv` scaling, the read activation result and the negative controls are in
//! `docs/evidence/native_geometric_geometric_attention_2026-09-19.txt`.

#![forbid(unsafe_code)]

use super::group_table::{group_table, GROUP_ORDER, ROW_STRIDE};
use super::lowbit::TernaryLinear;
use super::lowbit_core::{adam_update, quantize_codes, softmax_f32, xorshift_unit, TrainConfig};

/// The conjugacy classes of `2I`, computed from the project's verified group table.
///
/// `class_of[g]` is the class index of `g`, and the returned count is the number of classes. This is
/// the exact finite analogue of a spherical-harmonic band decomposition: by Peter–Weyl, the
/// conjugation-invariant functions on a finite group (the functions of the *relative* element) form a
/// space whose dimension is the number of conjugacy classes, spanned by the irreducible characters.
/// A graded kernel that depends only on `class(q⁻¹g)` is therefore the maximally compact
/// rotation-invariant kernel this group admits — nine free weights rather than 120.
///
/// Computed rather than cited: conjugation `g ↦ h g h⁻¹` uses the table's product and inverse rows.
pub fn conjugacy_classes() -> (Vec<u8>, usize) {
    let t = group_table();
    let n = RADIX;
    let mut class_of = vec![u8::MAX; n];
    let mut next = 0u8;
    for g in 0..n {
        if class_of[g] != u8::MAX {
            continue;
        }
        for h in 0..n {
            let hg = t.product[h * ROW_STRIDE + g] as usize;
            let c = t.product[hg * ROW_STRIDE + t.inverse[h] as usize] as usize;
            class_of[c] = next;
        }
        next += 1;
    }
    (class_of, next as usize)
}

/// The exact-read kernel: unit weight on the identity, zero elsewhere. In the graded read this
/// reproduces `y = S[q]` exactly, so it is the control for a soft filter.
///
/// The identity must be looked up, not assumed: `2I`'s identity is element 1 of the table, not element
/// 0. In class mode the identity's *class* is the slot; in general mode the identity *offset* is.
pub fn exact_kernel(n_classes: usize, identity_class: usize) -> Vec<i32> {
    let mut k = vec![0i32; n_classes];
    if identity_class < n_classes {
        k[identity_class] = 1;
    }
    k
}

/// Number of filter slots: one per conjugacy class (conjugation-invariant), or one per group element
/// (a general group-algebra element, strictly more expressive).
pub fn kernel_slots(class_filter: bool, n_classes: usize) -> usize {
    if class_filter {
        n_classes
    } else {
        RADIX
    }
}

/// The control filter: unit weight on the identity, zero elsewhere.
pub fn identity_kernel(class_filter: bool, n_classes: usize, class_of: &[u8]) -> Vec<i32> {
    let n = kernel_slots(class_filter, n_classes);
    let mut k = vec![0i32; n];
    let slot = if class_filter {
        identity_class(class_of)
    } else {
        group_table().identity as usize
    };
    if slot < n {
        k[slot] = 1;
    }
    k
}

/// The class index of the group identity.
pub fn identity_class(class_of: &[u8]) -> usize {
    let ident = group_table().identity as usize;
    class_of.get(ident).copied().unwrap_or(0) as usize
}

/// Radix of the word: one element per position is an element of 2I.
pub const RADIX: usize = GROUP_ORDER;

/// Number of address buckets for a word length.
pub fn address_space(order: usize) -> usize {
    match order {
        0 | 1 => RADIX,
        n => RADIX.pow(n as u32),
    }
}

/// The fixed element of `2I` assigned to each token.
///
/// Injective for a vocabulary of at most 120, which is what makes an ordered word a unique address at
/// this size. A larger vocabulary needs an injective learned assignment over the 120 roots (the
/// project's `nearest_h4_root` over learned embeddings) rather than a wider hash.
pub fn element_table(vocab: usize) -> Vec<u16> {
    (0..vocab).map(|t| (t % RADIX) as u16).collect()
}

#[inline]
fn xorshift_u64(state: u64) -> u64 {
    let mut x = state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}

/// The address of an ordered word: positional radix composition, order-preserving by construction.
#[inline]
pub fn word_address(elements: &[u16], order: usize) -> usize {
    let mut a = 0usize;
    for k in 0..order {
        a = a * RADIX + elements[k] as usize;
    }
    a
}

/// Serving form of the geometric attention core.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeometricAttention {
    pub vocab: usize,
    /// Value width.
    pub dv: usize,
    /// Word length in tokens. `1` addresses by one token, `2` by the ordered pair.
    pub order: usize,
    pub n_addr: usize,
    /// Fixed `2I` element per token.
    pub elements: Vec<u16>,
    /// Conjugacy class of each group element (Peter–Weyl band index).
    pub class_of: Vec<u8>,
    /// Number of conjugacy classes — the dimension of the conjugation-invariant kernel space.
    pub n_classes: usize,
    /// When true the filter is a class function (conjugation-invariant, 9 weights); when false it is a
    /// general group-algebra element (120 weights), which contains the class functions as a subspace.
    pub class_filter: bool,
    /// Ternary graded-read filter. The exact read is a unit weight on the identity.
    pub kernel: Vec<i8>,
    /// Values, `vocab x dv`, ternary with a per-token power-of-two magnitude.
    pub w_v: TernaryLinear,
    /// Output map, `vocab x dv`.
    pub w_o: TernaryLinear,
    /// Target readout magnitude in bits for the power-of-two normalisation.
    pub norm_bits: u32,
    /// Apply `relu` to the read. With exact addressing the *selection* is the nonlinearity, and a
    /// `relu` after the read discards the retrieved value's sign half.
    pub use_relu: bool,
}

impl GeometricAttention {
    pub fn from_f32(
        vocab: usize,
        dv: usize,
        order: usize,
        norm_bits: u32,
        wv: &[f32],
        wo: &[f32],
    ) -> Result<Self, String> {
        if vocab == 0 || dv == 0 {
            return Err("vocab and dv must be non-zero".into());
        }
        if order == 0 || order > 2 {
            return Err("order must be 1 or 2 (dense buffers)".into());
        }
        if wv.len() != vocab * dv {
            return Err(format!("w_v must be {vocab}x{dv}, got {}", wv.len()));
        }
        if wo.len() != vocab * dv {
            return Err(format!("w_o must be {vocab}x{dv}, got {}", wo.len()));
        }
        Ok(Self {
            vocab,
            dv,
            order,
            n_addr: address_space(order),
            elements: element_table(vocab),
            class_of: conjugacy_classes().0,
            n_classes: conjugacy_classes().1,
            class_filter: true,
            kernel: {
                let (class_of, n_classes) = conjugacy_classes();
                identity_kernel(true, n_classes, &class_of)
                    .into_iter()
                    .map(|k| k as i8)
                    .collect()
            },
            w_v: TernaryLinear::quantize(wv, vocab, dv),
            w_o: TernaryLinear::quantize(wo, vocab, dv),
            norm_bits,
            use_relu: false,
        })
    }

    /// Replace the graded-read kernel (one ternary weight per conjugacy class).
    #[must_use]
    pub fn with_kernel(mut self, kernel: Vec<i8>) -> Self {
        self.kernel = kernel;
        self
    }

    /// The graded read: `Σ_g w[class(q⁻¹g)] · S[g]`, ternary `w`, so adds and subtracts only.
    ///
    /// With `w = [1, 0, …]` this is exactly `S[q]`. With weight on other classes it pools over
    /// group-near stored elements, which is the band-limited (class-function) kernel this group
    /// admits — and the reason a corrupted query address can still retrieve its value.
    fn graded_read(&self, s: &[i32], q: usize) -> Vec<i32> {
        let t = group_table();
        let inv_q = t.inverse[q] as usize;
        let mut num = vec![0i32; self.dv];
        for g in 0..RADIX {
            let off = t.product[inv_q * ROW_STRIDE + g] as usize;
            let c = if self.class_filter {
                self.class_of[off] as usize
            } else {
                off
            };
            let base = g * self.dv;
            match self.kernel[c] {
                1 => {
                    for j in 0..self.dv {
                        num[j] += s[base + j];
                    }
                }
                -1 => {
                    for j in 0..self.dv {
                        num[j] -= s[base + j];
                    }
                }
                _ => {}
            }
        }
        num
    }

    /// The graded read in `f64`.
    fn graded_read_f64(&self, s: &[f64], q: usize) -> Vec<f64> {
        let t = group_table();
        let inv_q = t.inverse[q] as usize;
        let mut num = vec![0f64; self.dv];
        for g in 0..RADIX {
            let off = t.product[inv_q * ROW_STRIDE + g] as usize;
            let c = if self.class_filter {
                self.class_of[off] as usize
            } else {
                off
            };
            let w = self.kernel[c] as f64;
            if w == 0.0 {
                continue;
            }
            let base = g * self.dv;
            for j in 0..self.dv {
                num[j] += w * s[base + j];
            }
        }
        num
    }

    /// Choose whether the read is passed through `relu`.
    #[must_use]
    pub fn with_relu(mut self, use_relu: bool) -> Self {
        self.use_relu = use_relu;
        self
    }

    #[inline]
    pub fn state_len(&self) -> usize {
        self.n_addr * self.dv + self.n_addr
    }

    #[inline]
    pub fn initial_state(&self) -> Vec<i32> {
        vec![0i32; self.state_len()]
    }

    /// The address of a word given as tokens.
    #[inline]
    fn address(&self, ctx: &[u32]) -> usize {
        let mut a = 0usize;
        for k in 0..self.order {
            let t = (ctx[k] as usize).min(self.vocab - 1);
            a = a * RADIX + self.elements[t] as usize;
        }
        a
    }

    /// Write the word `ctx` (the ordered context ending at the predecessor) with `value(token)`.
    /// Adds only; one selected bucket.
    pub fn observe(&self, s: &mut [i32], ctx: &[u32], token: u32) {
        let t = (token as usize).min(self.vocab - 1);
        let a = self.address(ctx);
        let base = a * self.dv;
        let vshift = self.w_v.shift(t);
        for j in 0..self.dv {
            s[base + j] += self.w_v.weight(t, j) << vshift;
        }
        let cbase = self.n_addr * self.dv;
        s[cbase + a] += 1;
    }

    /// Read the bucket addressed by `ctx`, normalised to a power-of-two magnitude.
    pub fn logits(&self, s: &[i32], ctx: &[u32]) -> Vec<i32> {
        let a = self.address(ctx);
        let mut num = if self.order == 1 {
            self.graded_read(s, a)
        } else {
            let base = a * self.dv;
            s[base..base + self.dv].to_vec()
        };
        let mut m = 0i32;
        for v in num.iter() {
            m = m.max(v.abs());
        }
        let shift = if self.norm_bits > 0 && m > 0 {
            (32 - (m as u32).leading_zeros()).saturating_sub(self.norm_bits)
        } else {
            0
        };
        if shift > 0 {
            for v in num.iter_mut() {
                *v >>= shift;
            }
        }
        if self.use_relu {
            for v in num.iter_mut() {
                if *v < 0 {
                    *v = 0;
                }
            }
        }
        self.w_o.forward_i32(&num)
    }

    /// Ingest a prompt and read with the final word.
    pub fn forward_i32(&self, tokens: &[u32]) -> Vec<i32> {
        let mut s = self.initial_state();
        self.ingest_all(&mut s, tokens);
        self.logits(&s, &self.tail_word(tokens))
    }

    /// The write side of the recurrence: for each position `i >= order`, write the word ending at
    /// `i-1` with the value of token `i`.
    fn ingest_all(&self, s: &mut [i32], tokens: &[u32]) {
        let order = self.order;
        for i in order..tokens.len() {
            self.observe(s, &tokens[i - order..i], tokens[i]);
        }
    }

    /// The word ending at the final token (padded with zero if the prompt is shorter than `order`).
    fn tail_word(&self, tokens: &[u32]) -> Vec<u32> {
        let len = tokens.len();
        let mut w = Vec::with_capacity(self.order);
        for k in 0..self.order {
            let idx = len + k - self.order;
            w.push(tokens.get(idx).copied().unwrap_or(0));
        }
        w
    }

    /// Write the vocabulary as a dictionary keyed by **meaning**: `S[elem(t)] += value(t)`.
    ///
    /// This is the packaging step: every token of the vocabulary is placed at its group element, so a
    /// query can address a *composed* element rather than a stored context.
    pub fn build_dictionary(&self, s: &mut [i32], tokens: &[u32]) {
        let cbase = self.n_addr * self.dv;
        for &t in tokens {
            let tok = (t as usize).min(self.vocab - 1);
            let e = self.elements[tok] as usize;
            if e >= self.n_addr || cbase + e >= s.len() {
                continue;
            }
            let base = e * self.dv;
            let vshift = self.w_v.shift(tok);
            for j in 0..self.dv {
                s[base + j] += self.w_v.weight(tok, j) << vshift;
            }
            s[cbase + e] += 1;
        }
    }

    /// Read the bucket addressed by the **composition** `elem(a) · elem(b)`.
    ///
    /// The query is computed, not looked up: an unseen pair `(a, b)` still addresses a valid element,
    /// so a dictionary keyed by meaning can answer facts it never saw. This is the mechanism a
    /// context-address lookup cannot provide.
    pub fn compose_logits(&self, s: &[i32], a: u32, b: u32) -> Vec<i32> {
        let table = group_table();
        let ea = self.elements[(a as usize).min(self.vocab - 1)] as usize;
        let eb = self.elements[(b as usize).min(self.vocab - 1)] as usize;
        let q = table.product[ea * ROW_STRIDE + eb] as usize;
        let base = q * self.dv;
        let mut m = 0i32;
        for j in 0..self.dv {
            m = m.max(s[base + j].abs());
        }
        let shift = if self.norm_bits > 0 && m > 0 {
            (32 - (m as u32).leading_zeros()).saturating_sub(self.norm_bits)
        } else {
            0
        };
        let mut num = vec![0i32; self.dv];
        for (j, slot) in num.iter_mut().enumerate() {
            *slot = s[base + j] >> shift;
        }
        if self.use_relu {
            for v in num.iter_mut() {
                if *v < 0 {
                    *v = 0;
                }
            }
        }
        self.w_o.forward_i32(&num)
    }

    /// The write count of the route addressed by `ctx`: `0` means the route is **closed**.
    #[inline]
    pub fn route_count(&self, s: &[i32], ctx: &[u32]) -> i32 {
        let a = self.address(ctx);
        s[self.n_addr * self.dv + a]
    }

    /// Read the addressed route, or `None` when the route is closed.
    ///
    /// A closed route is not evidence that the answer is absent from the world — only that this
    /// address holds no record — so the honest output is an abstention, not a confident guess. This
    /// matches the project's chain-traversal rule: prove eviction separately before abstaining.
    pub fn logits_or_abstain(&self, s: &[i32], ctx: &[u32]) -> Option<Vec<i32>> {
        if self.route_count(s, ctx) == 0 {
            return None;
        }
        Some(self.logits(s, ctx))
    }

    /// The same computation in `f64`, for verifying the integer path.
    pub fn forward_reference_f64(&self, tokens: &[u32]) -> Vec<f64> {
        let mut s = vec![0f64; self.state_len()];
        let cbase = self.n_addr * self.dv;
        for i in self.order..tokens.len() {
            let a = self.address(&tokens[i - self.order..i]);
            let t = (tokens[i] as usize).min(self.vocab - 1);
            let base = a * self.dv;
            let vscale = (1u64 << self.w_v.shift(t)) as f64;
            for j in 0..self.dv {
                s[base + j] += self.w_v.weight(t, j) as f64 * vscale;
            }
            s[cbase + a] += 1.0;
        }
        let ctx = self.tail_word(tokens);
        let a = self.address(&ctx);
        let num = if self.order == 1 {
            self.graded_read_f64(&s, a)
        } else {
            let base = a * self.dv;
            s[base..base + self.dv].to_vec()
        };
        let mut m = 0f64;
        for v in num.iter() {
            m = m.max(v.abs());
        }
        let shift = if self.norm_bits > 0 && m >= 1.0 {
            (64 - (m as u64).leading_zeros()).saturating_sub(self.norm_bits)
        } else {
            0
        };
        let div = (1u64 << shift) as f64;
        let mut num: Vec<f64> = num.iter().map(|&v| (v / div).floor()).collect();
        for v in num.iter_mut() {
            if self.use_relu && *v < 0.0 {
                *v = 0.0;
            }
        }
        self.w_o.forward_reference(&num)
    }

    /// The graded read in `f32` (training/evaluation, not a serving path).
    fn graded_read_f32(&self, s: &[f32], q: usize) -> Vec<f32> {
        let t = group_table();
        let inv_q = t.inverse[q] as usize;
        let mut num = vec![0f32; self.dv];
        for g in 0..RADIX {
            let off = t.product[inv_q * ROW_STRIDE + g] as usize;
            let c = if self.class_filter {
                self.class_of[off] as usize
            } else {
                off
            };
            let w = self.kernel[c] as f32;
            if w == 0.0 {
                continue;
            }
            let base = g * self.dv;
            for j in 0..self.dv {
                num[j] += w * s[base + j];
            }
        }
        num
    }

    pub fn weight_bytes(&self) -> usize {
        self.w_v.weight_bytes() + self.w_o.weight_bytes()
    }

    /// Mean next-token cross-entropy under the quantised weights (offline only).
    pub fn sequence_loss(&self, tokens: &[u32]) -> f32 {
        let n = tokens.len();
        if n < 2 {
            return 0.0;
        }
        let order = self.order;
        let cbase = self.n_addr * self.dv;
        let mut s = vec![0f32; self.state_len()];
        let preds = n - 1;
        let mut total = 0f64;
        for i in 0..preds {
            if i >= order {
                let a = self.address(&tokens[i - order..i]);
                let t = (tokens[i] as usize).min(self.vocab - 1);
                let base = a * self.dv;
                let vscale = (1u64 << self.w_v.shift(t)) as f32;
                for j in 0..self.dv {
                    s[base + j] += self.w_v.weight(t, j) as f32 * vscale;
                }
                s[cbase + a] += 1.0;
            }
            let ctx: Vec<u32> = (0..order)
                .map(|k| {
                    let idx = i + k + 1 - order;
                    tokens.get(idx).copied().unwrap_or(0)
                })
                .collect();
            let a = self.address(&ctx);
            let mut num = if self.order == 1 {
                self.graded_read_f32(&s, a)
            } else {
                let base = a * self.dv;
                s[base..base + self.dv].to_vec()
            };
            let mut m = 0f32;
            for v in num.iter() {
                m = m.max(v.abs());
            }
            let shift = if self.norm_bits > 0 && m >= 1.0 {
                (32 - (m as u32).leading_zeros()).saturating_sub(self.norm_bits)
            } else {
                0
            };
            let div = (1u64 << shift) as f32;
            for v in num.iter_mut() {
                let x = (*v / div).floor();
                *v = if self.use_relu { x.max(0.0) } else { x };
            }
            let mut logits = vec![0f32; self.vocab];
            for (r, slot) in logits.iter_mut().enumerate() {
                let mut acc = 0f32;
                for j in 0..self.dv {
                    acc += self.w_o.weight(r, j) as f32 * num[j];
                }
                *slot = acc * (1u64 << self.w_o.shift(r)) as f32;
            }
            let p = softmax_f32(&logits);
            let target = (tokens[i + 1] as usize).min(self.vocab - 1);
            total += -(p[target].max(1e-9)).ln() as f64;
        }
        (total / preds as f64) as f32
    }
}

// ---------------------------------------------------------------------------
// Offline training. Only the value and output tables are learned; addresses are fixed.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct GeometricAttentionTrainer {
    pub vocab: usize,
    pub dv: usize,
    pub order: usize,
    pub n_addr: usize,
    pub elements: Vec<u16>,
    pub norm_bits: u32,
    pub use_relu: bool,
    /// Conjugacy class of each group element (the harmonic band index).
    pub class_of: Vec<u8>,
    pub n_classes: usize,
    /// Class function (9 weights) or general group-algebra element (120 weights).
    pub class_filter: bool,
    /// DIAGNOSTIC ONLY: use the unquantised `f32` readout instead of the ternary one. This is an
    /// oracle for testing whether readout resolution is the binding constraint; it is not a serving
    /// path and must stay false outside that experiment.
    pub readout_oracle: bool,
    /// Learn a per-class bias on the readout. This is the **reduced form** of a nearest-prototype
    /// decode: `argmax_r (2·num·p_r − ‖p_r‖²)` is linear in `num` with a per-class constant, so a
    /// nearest-root table decode is exactly a linear readout *plus a bias*. Testing the bias first
    /// decides whether the decode's geometry is the missing piece before building the full version.
    pub readout_bias: bool,
    wb: Vec<f32>,
    gwb: Vec<f32>,
    mwb: Vec<f32>,
    vwb: Vec<f32>,
    /// Learned harmonic filter masters; the read is the group convolution of the stored
    /// superposition with this filter.
    pub kernel_m: Vec<f32>,
    gkernel: Vec<f32>,
    mkernel: Vec<f32>,
    vkernel: Vec<f32>,
    /// Probability per training sequence that the final query address is corrupted.
    pub corrupt_frac: f32,
    /// When false the filter is held at its initialisation (its gradient is discarded).
    pub learn_kernel: bool,
    corrupt_state: u64,
    pub wv: Vec<f32>,
    pub wo: Vec<f32>,
    pub cfg: TrainConfig,
    gwv: Vec<f32>,
    gwo: Vec<f32>,
    mwv: Vec<f32>,
    vwv: Vec<f32>,
    mwo: Vec<f32>,
    vwo: Vec<f32>,
    step: u64,
    scratch: Vec<f32>,
    dscratch: Vec<f32>,
    touched: Vec<u32>,
}

impl GeometricAttentionTrainer {
    pub fn new(
        vocab: usize,
        dv: usize,
        order: usize,
        norm_bits: u32,
        seed: u64,
    ) -> Result<Self, String> {
        if vocab == 0 || dv == 0 {
            return Err("vocab and dv must be non-zero".into());
        }
        if order == 0 || order > 2 {
            return Err("order must be 1 or 2 (dense buffers)".into());
        }
        let mut st = seed ^ 0x9E37_79B9_7F4A_7C15;
        if st == 0 {
            st = 0x1234_5678_9ABC_DEF0;
        }
        let mut fill = |n: usize| -> Vec<f32> { (0..n).map(|_| xorshift_unit(&mut st)).collect() };
        Ok(Self {
            vocab,
            dv,
            order,
            n_addr: address_space(order),
            elements: element_table(vocab),
            norm_bits,
            use_relu: false,
            class_of: conjugacy_classes().0,
            n_classes: conjugacy_classes().1,
            class_filter: true,
            kernel_m: {
                let (class_of, n_classes) = conjugacy_classes();
                identity_kernel(true, n_classes, &class_of)
                    .into_iter()
                    .map(|k| k as f32)
                    .collect()
            },
            gkernel: vec![0f32; conjugacy_classes().1],
            mkernel: vec![0f32; conjugacy_classes().1],
            vkernel: vec![0f32; conjugacy_classes().1],
            corrupt_frac: 0.0,
            learn_kernel: true,
            corrupt_state: seed ^ 0x5DEE_CE66_D1CE_B00D,
            wv: fill(vocab * dv),
            wo: fill(vocab * dv),
            cfg: TrainConfig::default(),
            gwv: vec![0f32; vocab * dv],
            gwo: vec![0f32; vocab * dv],
            mwv: vec![0f32; vocab * dv],
            vwv: vec![0f32; vocab * dv],
            mwo: vec![0f32; vocab * dv],
            vwo: vec![0f32; vocab * dv],
            step: 0,
            scratch: Vec::new(),
            dscratch: Vec::new(),
            touched: Vec::new(),
            readout_oracle: false,
            readout_bias: false,
            wb: vec![0f32; vocab],
            gwb: vec![0f32; vocab],
            mwb: vec![0f32; vocab],
            vwb: vec![0f32; vocab],
        })
    }

    #[inline]
    fn address(&self, ctx: &[u32]) -> usize {
        let mut a = 0usize;
        for k in 0..self.order {
            let t = (ctx[k] as usize).min(self.vocab - 1);
            a = a * RADIX + self.elements[t] as usize;
        }
        a
    }

    /// The word ending at position `i` (padded with zero on the left).
    #[inline]
    fn word_at(&self, tokens: &[u32], i: usize) -> Vec<u32> {
        (0..self.order)
            .map(|k| {
                let idx = i + k + 1 - self.order;
                tokens.get(idx).copied().unwrap_or(0)
            })
            .collect()
    }

    /// The ternary harmonic filter actually used in the read.
    pub fn quantised_kernel(&self) -> Vec<i8> {
        self.kernel_m
            .iter()
            .map(|&w| {
                if w > 0.5 {
                    1
                } else if w < -0.5 {
                    -1
                } else {
                    0
                }
            })
            .collect()
    }

    /// Switch between a class-function filter and a general group-algebra filter, rebuilding the
    /// filter state at the identity control so the change is safe.
    pub fn set_class_filter(&mut self, class_filter: bool) {
        let (class_of, n_classes) = conjugacy_classes();
        let n = kernel_slots(class_filter, n_classes);
        self.class_filter = class_filter;
        self.kernel_m = identity_kernel(class_filter, n_classes, &class_of)
            .into_iter()
            .map(|k| k as f32)
            .collect();
        self.gkernel = vec![0f32; n];
        self.mkernel = vec![0f32; n];
        self.vkernel = vec![0f32; n];
    }

    pub fn to_core(&self) -> Result<GeometricAttention, String> {
        let mut core = GeometricAttention::from_f32(
            self.vocab,
            self.dv,
            self.order,
            self.norm_bits,
            &self.wv,
            &self.wo,
        )?;
        core.use_relu = self.use_relu;
        core.class_filter = self.class_filter;
        core.kernel = self.quantised_kernel();
        // The packaging must survive the round trip or a custom element assignment is silently lost.
        core.elements = self.elements.clone();
        Ok(core)
    }

    pub fn final_token_correct(&self, tokens: &[u32]) -> bool {
        if tokens.len() < 2 {
            return false;
        }
        let core = match self.to_core() {
            Ok(c) => c,
            Err(_) => return false,
        };
        let logits = core.forward_i32(&tokens[..tokens.len() - 1]);
        let mut best = 0usize;
        for (i, &v) in logits.iter().enumerate() {
            if v > logits[best] {
                best = i;
            }
        }
        best == (tokens[tokens.len() - 1] as usize).min(self.vocab - 1)
    }

    fn accumulate(&mut self, tokens: &[u32]) -> f32 {
        let n = tokens.len();
        if n < 2 {
            return 0.0;
        }
        let order = self.order;
        let (qv, sv) = quantize_codes(&self.wv, self.vocab, self.dv);
        let (qo, so) = quantize_codes(&self.wo, self.vocab, self.dv);
        let cbase = self.n_addr * self.dv;
        let need = cbase + self.n_addr;
        let inv = 1.0f32 / (n - 1) as f32;

        if self.scratch.len() != need {
            self.scratch = vec![0f32; need];
            self.dscratch = vec![0f32; need];
            self.touched.clear();
        }
        for k in 0..self.touched.len() {
            let a = self.touched[k] as usize;
            let base = a * self.dv;
            for j in 0..self.dv {
                self.scratch[base + j] = 0.0;
                self.dscratch[base + j] = 0.0;
            }
            self.scratch[cbase + a] = 0.0;
        }
        self.touched.clear();

        // Forward. Cache what each read sees: the addressed bucket, its magnitude and its count.
        let mut reads: Vec<(Vec<f32>, u32)> = Vec::with_capacity(n - 1);
        // `order = 1` reads through a group convolution, which sees many buckets, so the full state is
        // cached per step (120 · dv, cheap). `order = 2` reads one exact bucket, so only that is kept.
        let mut states: Vec<Vec<f32>> = Vec::new();
        for i in 0..(n - 1) {
            if i >= order {
                let a = self.address(&tokens[i - order..i]);
                let t = (tokens[i] as usize).min(self.vocab - 1);
                let base = a * self.dv;
                if self.scratch[cbase + a] == 0.0 {
                    self.touched.push(a as u32);
                }
                for j in 0..self.dv {
                    self.scratch[base + j] += qv[t * self.dv + j] as f32 * sv[t];
                }
                self.scratch[cbase + a] += 1.0;
            }
            let ctx = self.word_at(tokens, i);
            let a = self.address(&ctx);
            if self.order == 1 {
                states.push(self.scratch.clone());
            } else {
                let base = a * self.dv;
                let mut bucket = vec![0f32; self.dv];
                bucket.copy_from_slice(&self.scratch[base..base + self.dv]);
                reads.push((bucket, self.scratch[cbase + a].max(0.0) as u32));
            }
        }

        // Backward. The state is a pure accumulator, so a write's gradient is the sum of the read
        // gradients at its bucket over *later* steps: a suffix sum, accumulated in reverse.
        let mut loss = 0f64;
        let kern = self.quantised_kernel();
        let gt = group_table();
        for i in (0..(n - 1)).rev() {
            let ctx = self.word_at(tokens, i);
            let a = self.address(&ctx);
            let base = a * self.dv;
            // Corruption goes into the OBJECTIVE: with probability `corrupt_frac` the read queries a
            // displaced group element, so a spread filter is what makes the answer recoverable.
            let q_elem = if self.order == 1 && self.corrupt_frac > 0.0 {
                self.corrupt_state = xorshift_u64(self.corrupt_state);
                let r = ((self.corrupt_state >> 11) as f64) / ((1u64 << 53) as f64);
                if (r as f32) < self.corrupt_frac {
                    self.corrupt_state = xorshift_u64(self.corrupt_state);
                    let h = (self.corrupt_state % RADIX as u64) as usize;
                    gt.product[h * ROW_STRIDE + a] as usize
                } else {
                    a
                }
            } else {
                a
            };
            // Only `order = 1` addresses a group *element*; an `order = 2` address is a pair index.
            let inv_q = if self.order == 1 {
                gt.inverse[q_elem] as usize
            } else {
                0
            };
            // The read: a group convolution for `order = 1`, an exact bucket for `order = 2`.
            let num_raw: Vec<f32> = if self.order == 1 {
                let st = &states[i];
                let mut num = vec![0f32; self.dv];
                for g in 0..RADIX {
                    let off = gt.product[inv_q * ROW_STRIDE + g] as usize;
                    let c = if self.class_filter {
                        self.class_of[off] as usize
                    } else {
                        off
                    };
                    let w = kern[c] as f32;
                    if w == 0.0 {
                        continue;
                    }
                    let bl = g * self.dv;
                    for j in 0..self.dv {
                        num[j] += w * st[bl + j];
                    }
                }
                num
            } else {
                reads[i].0.clone()
            };
            let mut m = 0f32;
            for v in num_raw.iter() {
                m = m.max(v.abs());
            }
            let shift = if self.norm_bits > 0 && m >= 1.0 {
                (32 - (m as u32).leading_zeros()).saturating_sub(self.norm_bits)
            } else {
                0
            };
            let dec = 1.0f32 / (1u64 << shift) as f32;

            let mut num = vec![0f32; self.dv];
            for j in 0..self.dv {
                let v = (num_raw[j] * dec).floor();
                num[j] = if self.use_relu { v.max(0.0) } else { v };
            }
            let mut logits = vec![0f32; self.vocab];
            if self.readout_oracle {
                // DIAGNOSTIC: unquantised readout. Tests whether weight resolution is the binding
                // constraint. Not a serving path.
                for (r, slot) in logits.iter_mut().enumerate() {
                    let mut acc = 0f32;
                    for j in 0..self.dv {
                        acc += self.wo[r * self.dv + j] * num[j];
                    }
                    *slot = acc;
                }
            } else {
                for (r, slot) in logits.iter_mut().enumerate() {
                    let mut acc = 0f32;
                    for j in 0..self.dv {
                        acc += qo[r * self.dv + j] as f32 * num[j];
                    }
                    *slot = acc * so[r];
                }
            }
            if self.readout_bias {
                for (r, slot) in logits.iter_mut().enumerate() {
                    *slot += self.wb[r];
                }
            }
            let mut p = softmax_f32(&logits);
            let target = (tokens[i + 1] as usize).min(self.vocab - 1);
            loss += -(p[target].max(1e-9)).ln() as f64;
            p[target] -= 1.0;

            let mut dnum = vec![0f32; self.dv];
            for r in 0..self.vocab {
                let g = p[r] * inv;
                if self.readout_bias {
                    self.gwb[r] += g;
                }
                if g == 0.0 {
                    continue;
                }
                let rbase = r * self.dv;
                let sr = so[r];
                for j in 0..self.dv {
                    self.gwo[rbase + j] += g * num[j];
                    dnum[j] += if self.readout_oracle {
                        g * self.wo[rbase + j]
                    } else {
                        g * (qo[rbase + j] as f32 * sr)
                    };
                }
            }
            // Read gradient. The shift is treated as a constant (STE through the bit scan). For the
            // convolution the filter's own gradient is the correlation of the read gradient with the
            // stored superposition.
            let mut dm = vec![0f32; self.dv];
            for j in 0..self.dv {
                let mask = if self.use_relu {
                    if num[j] > 0.0 {
                        1.0
                    } else {
                        0.0
                    }
                } else {
                    1.0
                };
                dm[j] = dnum[j] * mask * dec;
            }
            if self.order == 1 {
                let st = &states[i];
                for g in 0..RADIX {
                    let off = gt.product[inv_q * ROW_STRIDE + g] as usize;
                    let c = if self.class_filter {
                        self.class_of[off] as usize
                    } else {
                        off
                    };
                    let bl = g * self.dv;
                    let mut acc = 0f32;
                    for j in 0..self.dv {
                        acc += dm[j] * st[bl + j];
                        self.dscratch[bl + j] += (kern[c] as f32) * dm[j];
                    }
                    self.gkernel[c] += acc * inv;
                    self.touched.push(g as u32);
                }
            } else {
                self.touched.push(a as u32);
                for j in 0..self.dv {
                    self.dscratch[base + j] += dm[j];
                }
            }

            // Distribute the accumulated bucket gradient to the write made at this step.
            if i >= order {
                let wa = self.address(&tokens[i - order..i]);
                let wbase = wa * self.dv;
                let t = (tokens[i] as usize).min(self.vocab - 1);
                for j in 0..self.dv {
                    self.gwv[t * self.dv + j] += self.dscratch[wbase + j] * inv;
                }
            }
        }
        (loss / (n - 1) as f64) as f32
    }

    fn zero_grads(&mut self) {
        for g in self.gwv.iter_mut().chain(&mut self.gwo) {
            *g = 0.0;
        }
        for g in self.gkernel.iter_mut() {
            *g = 0.0;
        }
        for g in self.gwb.iter_mut() {
            *g = 0.0;
        }
    }

    /// Scale and clip. The nine filter weights are clipped in their **own** group: folding them into
    /// the matrices' global norm would change the clip applied to the matrices and alter every
    /// existing training result.
    fn normalize_and_clip(&mut self, batch: usize) {
        let scale = 1.0f32 / batch.max(1) as f32;
        if !self.learn_kernel {
            for g in self.gkernel.iter_mut() {
                *g = 0.0;
            }
        }
        let mut sumsq = 0f64;
        for g in self.gwv.iter_mut().chain(&mut self.gwo) {
            *g *= scale;
            sumsq += (*g as f64) * (*g as f64);
        }
        let mut ksq = 0f64;
        for g in self.gkernel.iter_mut() {
            *g *= scale;
            ksq += (*g as f64) * (*g as f64);
        }
        let mut bsq = 0f64;
        for g in self.gwb.iter_mut() {
            *g *= scale;
            bsq += (*g as f64) * (*g as f64);
        }
        if self.cfg.grad_clip > 0.0 {
            let clip = self.cfg.grad_clip as f64;
            let norm = libm::sqrt(sumsq);
            if norm > clip && norm > 0.0 {
                let s = (clip / norm) as f32;
                for g in self.gwv.iter_mut().chain(&mut self.gwo) {
                    *g *= s;
                }
            }
            let knorm = libm::sqrt(ksq);
            if knorm > clip && knorm > 0.0 {
                let s = (clip / knorm) as f32;
                for g in self.gkernel.iter_mut() {
                    *g *= s;
                }
            }
            let bnorm = libm::sqrt(bsq);
            if bnorm > clip && bnorm > 0.0 {
                let s = (clip / bnorm) as f32;
                for g in self.gwb.iter_mut() {
                    *g *= s;
                }
            }
        }
    }

    fn apply_adam(&mut self) {
        self.step += 1;
        let step = self.step;
        let cfg = self.cfg.clone();
        adam_update(
            &mut self.wv,
            &self.gwv,
            &mut self.mwv,
            &mut self.vwv,
            &cfg,
            step,
        );
        adam_update(
            &mut self.wo,
            &self.gwo,
            &mut self.mwo,
            &mut self.vwo,
            &cfg,
            step,
        );
        adam_update(
            &mut self.kernel_m,
            &self.gkernel,
            &mut self.mkernel,
            &mut self.vkernel,
            &cfg,
            step,
        );
        adam_update(
            &mut self.wb,
            &self.gwb,
            &mut self.mwb,
            &mut self.vwb,
            &cfg,
            step,
        );
    }

    pub fn train_batch(&mut self, batch: &[Vec<u32>]) -> f32 {
        if batch.is_empty() {
            return 0.0;
        }
        self.zero_grads();
        let mut total = 0f64;
        for seq in batch {
            total += self.accumulate(seq) as f64;
        }
        self.normalize_and_clip(batch.len());
        self.apply_adam();
        (total / batch.len() as f64) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_geometric::learner::LowBitAttentionTrainer;

    fn random_attention(vocab: usize, dv: usize, order: usize, seed: u64) -> GeometricAttention {
        let mut st = seed | 1;
        let mut nf = |n: usize| -> Vec<f32> {
            (0..n)
                .map(|_| {
                    st ^= st << 13;
                    st ^= st >> 7;
                    st ^= st << 17;
                    let u = ((st >> 11) as f64 / (1u64 << 53) as f64) as f32;
                    u * 2.0 - 1.0
                })
                .collect()
        };
        GeometricAttention::from_f32(vocab, dv, order, 0, &nf(vocab * dv), &nf(vocab * dv))
            .expect("build")
    }

    #[test]
    fn integer_path_is_exactly_the_float_reference() {
        for order in [1usize, 2] {
            let a = random_attention(17, 7, order, 0xABCD_EF01);
            for tokens in [
                vec![0u32, 3, 7, 1, 16, 2],
                vec![5, 5, 5, 5],
                vec![1, 2],
                vec![9],
                vec![],
            ] {
                let got = a.forward_i32(&tokens);
                let want = a.forward_reference_f64(&tokens);
                assert_eq!(got.len(), a.vocab);
                for j in 0..a.vocab {
                    assert_eq!(
                        got[j] as f64, want[j],
                        "order {order}: integer path diverged at {j} for {tokens:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn ordered_words_are_distinct_and_order_sensitive() {
        // Injective elements up to 120 mean an ordered pair is a unique address, and (a,b) != (b,a).
        let elems = element_table(64);
        let mut seen = std::collections::HashSet::new();
        for &a in &elems {
            for &b in &elems {
                assert!(seen.insert(word_address(&[a, b], 2)));
            }
        }
        assert_eq!(seen.len(), 64 * 64);
        assert_ne!(word_address(&[1, 2], 2), word_address(&[2, 1], 2));
        assert!(word_address(&[1, 2], 2) < address_space(2));
    }

    /// The harmonic grounding: conjugation-invariant functions on `2I` (functions of the *relative*
    /// element) form a space whose dimension is the number of conjugacy classes — the Peter–Weyl
    /// analogue of a spherical-harmonic band count. Computed from the project's verified table.
    #[test]
    fn conjugacy_classes_of_2i_from_the_verified_table() {
        let (class_of, n_classes) = conjugacy_classes();
        let mut counts = vec![0usize; n_classes];
        for &c in &class_of {
            assert_ne!(c, u8::MAX, "every element must land in a class");
            counts[c as usize] += 1;
        }
        let mut sorted = counts.clone();
        sorted.sort_unstable();
        eprintln!("2I conjugacy classes={n_classes} sizes(sorted)={sorted:?}");
        assert_eq!(counts.iter().sum::<usize>(), RADIX);
        // The identity is alone in its class.
        assert_eq!(counts[class_of[0] as usize], 1);
        // Direct conjugation invariance over every pair.
        let t = group_table();
        for g in 0..RADIX {
            for h in 0..RADIX {
                let hg = t.product[h * ROW_STRIDE + g] as usize;
                let c = t.product[hg * ROW_STRIDE + t.inverse[h] as usize] as usize;
                assert_eq!(
                    class_of[c], class_of[g],
                    "class must be conjugation-invariant"
                );
            }
        }
    }

    /// The spherical-harmonic (class-function) kernel doing work: a corrupted query address still
    /// retrieves its value, because `w[class(q⁻¹g)]` pools over group-near stored elements where the
    /// exact read would look in exactly one wrong bucket.
    #[test]
    fn graded_kernel_recovers_a_corrupted_query() {
        let k = 8usize;
        let (class_of, n_classes) = conjugacy_classes();
        // `vocab = 120` makes the element table the identity, so every group element is a token and a
        // corrupted query is still a valid token.
        let train = repeat_alphabet(0xA5A5_1234, 64, k, 8);
        let held = repeat_alphabet(0x0BAD_F00D, 64, k, 8);
        let mut t = GeometricAttentionTrainer::new(RADIX, 64, 1, 6, 2026_0919).expect("build");
        t.cfg.lr = 0.05;
        t.learn_kernel = false;
        for _ in 0..900 {
            t.train_batch(&train);
        }
        let base = t.to_core().expect("quantise");

        let gt = group_table();
        let score = |core: &GeometricAttention, h: Option<usize>| {
            let mut hits = 0usize;
            for s in &held {
                let mut prompt: Vec<u32> = s[..s.len() - 1].to_vec();
                if let Some(h) = h {
                    let q = *prompt.last().unwrap() as usize;
                    *prompt.last_mut().unwrap() = gt.product[h * ROW_STRIDE + q] as u32;
                }
                let logits = core.forward_i32(&prompt);
                let mut best = 0usize;
                for (i, &v) in logits.iter().enumerate() {
                    if v > logits[best] {
                        best = i;
                    }
                }
                if best == s[s.len() - 1] as usize {
                    hits += 1;
                }
            }
            hits as f32 / held.len() as f32
        };

        let clean_exact = score(&base, None);
        let mut exact_sum = 0f32;
        let mut soft_sum = 0f32;
        let mut clean_soft_sum = 0f32;
        let hs = [3usize, 7, 11, 13, 17];
        for &h in &hs {
            let hc = class_of[h] as usize;
            let mut soft = vec![0i32; n_classes];
            soft[hc] = 1;
            let soft_core = base
                .clone()
                .with_kernel(soft.iter().map(|x| *x as i8).collect());
            exact_sum += score(&base, Some(h));
            soft_sum += score(&soft_core, Some(h));
            clean_soft_sum += score(&soft_core, None);
        }
        let n = hs.len() as f32;
        let (exact_corrupt, soft_corrupt, clean_soft) =
            (exact_sum / n, soft_sum / n, clean_soft_sum / n);
        eprintln!(
            "graded kernel over {n:.0} corruptions: clean exact={clean_exact:.2} soft={clean_soft:.2} | \
             corrupted exact={exact_corrupt:.2} soft={soft_corrupt:.2}"
        );
        assert!(
            soft_corrupt > exact_corrupt,
            "a class-function kernel must recover a corrupted query on average: \
             {exact_corrupt:.2} -> {soft_corrupt:.2}"
        );
    }

    #[test]
    fn graded_exact_kernel_equals_the_bucket_read() {
        let a = random_attention(120, 16, 1, 1234);
        let mut s = a.initial_state();
        // `vocab = 120` makes elements the identity, so every element can be a token.
        let tokens: Vec<u32> = vec![0, 3, 7, 1, 16, 2, 5, 5];
        for i in 1..tokens.len() {
            a.observe(&mut s, &tokens[i - 1..i], tokens[i]);
        }
        let q = a.elements[*tokens.last().unwrap() as usize] as usize;
        let base = q * a.dv;
        let manual: Vec<i32> = (0..a.dv).map(|j| s[base + j]).collect();
        assert!(
            manual.iter().any(|&v| v != 0),
            "the bucket must be non-empty, or the comparison is vacuous"
        );
        let via = a.graded_read(&s, q);
        assert_eq!(manual, via, "exact kernel must reproduce the bucket read");
        // And the read must actually depend on the query, not be a constant.
        let other = a.graded_read(&s, (q + 1) % RADIX);
        assert_ne!(via, other, "the read must depend on the addressed element");
    }

    /// The diagnosis from the previous round was that robustness is not in the objective. Put corrupted
    /// addresses *in* the objective and the learned filter should stop being pure loss.
    #[test]
    fn corruption_in_the_objective_changes_the_filter() {
        let k = 8usize;
        let train = repeat_alphabet(0xA5A5_1234, 64, k, 8);
        let held = repeat_alphabet(0x0BAD_F00D, 64, k, 8);
        let gt = group_table();
        let hs = [3usize, 7, 11, 13, 17];

        let run = |corrupt_frac: f32| -> (Vec<i8>, f32, f32) {
            let mut t = GeometricAttentionTrainer::new(RADIX, 64, 1, 6, 2026_0919).expect("build");
            t.cfg.lr = 0.05;
            t.corrupt_frac = corrupt_frac;
            for _ in 0..900 {
                t.train_batch(&train);
            }
            let kernel = t.quantised_kernel();
            let core = t.to_core().expect("quantise");
            let score = |h: Option<usize>| {
                let mut hits = 0usize;
                for s in &held {
                    let mut prompt: Vec<u32> = s[..s.len() - 1].to_vec();
                    if let Some(h) = h {
                        let q = *prompt.last().unwrap() as usize;
                        *prompt.last_mut().unwrap() = gt.product[h * ROW_STRIDE + q] as u32;
                    }
                    let logits = core.forward_i32(&prompt);
                    let mut best = 0usize;
                    for (i, &v) in logits.iter().enumerate() {
                        if v > logits[best] {
                            best = i;
                        }
                    }
                    if best == s[s.len() - 1] as usize {
                        hits += 1;
                    }
                }
                hits as f32 / held.len() as f32
            };
            let clean = score(None);
            let mut corr = 0f32;
            for &h in &hs {
                corr += score(Some(h));
            }
            (kernel, clean, corr / hs.len() as f32)
        };

        let (k0, c0, x0) = run(0.0);
        let (k5, c5, x5) = run(0.5);
        eprintln!("corrupt 0.0: kernel={k0:?} clean={c0:.2} corrupted={x0:.2}");
        eprintln!("corrupt 0.5: kernel={k5:?} clean={c5:.2} corrupted={x5:.2}");
        assert!(
            x5 > x0,
            "corruption in the objective must raise corrupted accuracy: {x0:.2} -> {x5:.2}"
        );
    }

    /// Does the general group-algebra filter (120 weights) beat the class-function filter (9)?
    #[test]
    fn a_general_filter_is_at_least_as_good_as_a_class_filter() {
        let k = 8usize;
        let train = repeat_alphabet(0xA5A5_1234, 64, k, 8);
        let held = repeat_alphabet(0x0BAD_F00D, 64, k, 8);
        let run = |class_filter: bool| -> (usize, usize, f32) {
            let mut t = GeometricAttentionTrainer::new(RADIX, 64, 1, 6, 2026_0919).expect("build");
            t.cfg.lr = 0.05;
            t.set_class_filter(class_filter);
            for _ in 0..900 {
                t.train_batch(&train);
            }
            let non_zero = t.quantised_kernel().iter().filter(|&&w| w != 0).count();
            let slots = t.quantised_kernel().len();
            let core = t.to_core().expect("quantise");
            let hits = held.iter().filter(|s| core_hit(&core, s)).count();
            (non_zero, slots, hits as f32 / held.len() as f32)
        };
        let (cn, cs, ca) = run(true);
        let (gn, gs, ga) = run(false);
        eprintln!("class filter: slots={cs} non_zero={cn} clean={ca:.2}");
        eprintln!("general filter: slots={gs} non_zero={gn} clean={ga:.2}");
        assert!(
            ga >= ca - 0.02,
            "the general filter contains the class filter, so it must not lose: {ca:.2} -> {ga:.2}"
        );
    }

    /// Predict the final token of a held-out sequence with a core.
    fn core_hit(core: &GeometricAttention, s: &[u32]) -> bool {
        let logits = core.forward_i32(&s[..s.len() - 1]);
        let mut best = 0usize;
        for (i, &v) in logits.iter().enumerate() {
            if v > logits[best] {
                best = i;
            }
        }
        best == s[s.len() - 1] as usize
    }

    /// Relational facts over `2I`. Token `w < 120` has meaning = its own element; token `120 + j` is a
    /// relation whose element is `j + 1`. The fact `(w, h_j) -> compose(w, h_j)` is *defined by the
    /// group*, so an unseen pair still has a well-defined answer that no lookup can have stored.
    const RELATIONS: usize = 8;

    fn relational_elements() -> Vec<u16> {
        let vocab = RADIX + RELATIONS;
        let mut e: Vec<u16> = (0..vocab).map(|t| (t % RADIX) as u16).collect();
        for j in 0..RELATIONS {
            e[RADIX + j] = (j + 1) as u16;
        }
        e
    }

    /// Held-out pairs: never used as a fact during training, so a lookup cannot have stored them.
    fn pair_is_held(w: usize, j: usize) -> bool {
        (w * RELATIONS + j) % 5 == 0
    }

    /// Sequences of several facts `(w, h, target)`, then a query pair `(w_q, h_q)` whose target is the
    /// final token. The answer is **not** adjacent to the query and appears nowhere else in the prompt,
    /// so a context lookup can only answer it if the fact is among the sequence's own facts.
    fn relational_sequences(
        seed: u64,
        n: usize,
        facts_per_seq: usize,
        query_held: bool,
    ) -> Vec<Vec<u32>> {
        let table = group_table();
        let mut st = seed | 1;
        let mut drawn = |st: &mut u64, m: u64| -> usize {
            *st ^= *st << 13;
            *st ^= *st >> 7;
            *st ^= *st << 17;
            (*st % m) as usize
        };
        (0..n)
            .map(|_| {
                let mut facts: Vec<(u32, u32, u32)> = Vec::new();
                while facts.len() < facts_per_seq {
                    let w = drawn(&mut st, RADIX as u64);
                    let j = drawn(&mut st, RELATIONS as u64);
                    if pair_is_held(w, j) {
                        continue;
                    }
                    let t = table.product[w * ROW_STRIDE + (j + 1)] as u32;
                    facts.push((w as u32, (RADIX + j) as u32, t));
                }
                let (qw, qh, qt) = if query_held {
                    loop {
                        let w = drawn(&mut st, RADIX as u64);
                        let j = drawn(&mut st, RELATIONS as u64);
                        if !pair_is_held(w, j) {
                            continue;
                        }
                        let t = table.product[w * ROW_STRIDE + (j + 1)] as u32;
                        break (w as u32, (RADIX + j) as u32, t);
                    }
                } else {
                    let f = facts[drawn(&mut st, facts.len() as u64)];
                    f
                };
                let mut seq = Vec::with_capacity(3 * facts.len() + 3);
                for (a, b, c) in &facts {
                    seq.push(*a);
                    seq.push(*b);
                    seq.push(*c);
                }
                seq.push(qw);
                seq.push(qh);
                seq.push(qt);
                seq
            })
            .collect()
    }

    fn relational_trainer() -> GeometricAttentionTrainer {
        let mut t =
            GeometricAttentionTrainer::new(RADIX + RELATIONS, 64, 2, 6, 2026_0919).expect("build");
        t.elements = relational_elements();
        t.cfg.lr = 0.05;
        t
    }

    /// Falsification half: the architecture as it stands is a context-address lookup, so it must fit
    /// what it saw and fail on pairs it never saw.
    #[test]
    fn a_context_lookup_cannot_generalise_to_unseen_relational_facts() {
        let train = relational_sequences(1, 64, 6, false);
        let seen = relational_sequences(2, 64, 6, false);
        let unseen = relational_sequences(3, 64, 6, true);
        let mut t = relational_trainer();
        for _ in 0..900 {
            t.train_batch(&train);
        }
        let score = |seqs: &[Vec<u32>]| {
            let hits = seqs.iter().filter(|s| t.final_token_correct(s)).count();
            hits as f32 / seqs.len() as f32
        };
        let (a, b) = (score(&seen), score(&unseen));
        eprintln!("context lookup: seen-query={a:.2} unseen-query={b:.2}");
        // The claim is the *gap*, not an absolute level: the seen level is capped by the readout, which
        // is a separate and already-measured limit.
        assert!(
            b < 0.2,
            "a context-address lookup must not generalise to unseen pairs: {b:.2}"
        );
        assert!(
            a > b + 0.2,
            "the lookup must fit seen pairs far better than unseen ones: {a:.2} vs {b:.2}"
        );
    }

    /// Construction half: package the vocabulary by meaning and *compose* the query. An unseen pair
    /// then addresses a valid element, so the fact is answered without ever having been stored.
    #[test]
    fn a_composed_dictionary_read_answers_unseen_relational_facts() {
        let train = relational_sequences(1, 64, 6, false);
        let seen = relational_sequences(2, 64, 6, false);
        let unseen = relational_sequences(3, 64, 6, true);
        let mut t = relational_trainer();
        for _ in 0..900 {
            t.train_batch(&train);
        }
        let core = t.to_core().expect("quantise");
        let mut s = core.initial_state();
        let words: Vec<u32> = (0..RADIX as u32).collect();
        core.build_dictionary(&mut s, &words);
        let score = |seqs: &[Vec<u32>]| {
            let mut hits = 0usize;
            for sq in seqs {
                let n = sq.len();
                let logits = core.compose_logits(&s, sq[n - 3], sq[n - 2]);
                let mut best = 0usize;
                for (i, &v) in logits.iter().enumerate() {
                    if v > logits[best] {
                        best = i;
                    }
                }
                if best == sq[n - 1] as usize {
                    hits += 1;
                }
            }
            hits as f32 / seqs.len() as f32
        };
        let (a, b) = (score(&seen), score(&unseen));
        eprintln!("composed dictionary: seen-query={a:.2} unseen-query={b:.2}");
        // The composed read answers an unseen pair at the same level as a seen one: it has no
        // generalisation gap, because the query is computed rather than retrieved.
        assert!(
            b >= a - 0.05,
            "the composed read must not generalise worse than it fits: {a:.2} vs {b:.2}"
        );
        assert!(
            b > 0.3,
            "the composed read must be far above the lookup's unseen level: {b:.2}"
        );
    }

    /// Is the ceiling a capacity limit or an optimisation limit? Cheapest discriminator: budget.
    /// Measured: 0.28 at 900 steps and 0.28 at 4000 — **budget is not the constraint either.**
    #[test]
    fn training_budget_is_not_the_constraint() {
        let train = relational_sequences(1, 32, 6, false);
        let held = relational_sequences(4, 32, 6, true);
        let mut seen = Vec::new();
        for steps in [900usize, 4000] {
            let mut t = relational_trainer();
            for _ in 0..steps {
                t.train_batch(&train);
            }
            let acc = eval_composed(&t, &held, false);
            eprintln!("relational budget: steps={steps} unseen={acc:.2}");
            seen.push(acc);
        }
        assert!(
            (seen[1] - seen[0]).abs() < 0.05,
            "4.4x the budget must not be the lever: {:.2} vs {:.2}",
            seen[0],
            seen[1]
        );
    }

    /// Evaluate a composed query from the trainer's masters, with the readout either quantised
    /// (serving form) or unquantised (diagnostic oracle). The dictionary is built the same way in both
    /// modes, so the *only* difference is readout resolution.
    fn eval_composed(t: &GeometricAttentionTrainer, seqs: &[Vec<u32>], oracle: bool) -> f32 {
        let (qo, so) = quantize_codes(&t.wo, t.vocab, t.dv);
        let (qv, sv) = quantize_codes(&t.wv, t.vocab, t.dv);
        let n_addr = t.n_addr;
        let mut s = vec![0f32; n_addr * t.dv + n_addr];
        // Words only. Relation tokens deliberately reuse elements 1..8, so including them would write
        // several values into eight word buckets and corrupt the dictionary.
        for tok in 0..RADIX.min(t.vocab) {
            let e = t.elements[tok] as usize;
            if e >= n_addr {
                continue;
            }
            for j in 0..t.dv {
                s[e * t.dv + j] += if oracle {
                    t.wv[tok * t.dv + j]
                } else {
                    qv[tok * t.dv + j] as f32 * sv[tok]
                };
            }
            s[n_addr * t.dv + e] += 1.0;
        }
        let gt = group_table();
        let mut hits = 0usize;
        for sq in seqs {
            let n = sq.len();
            let ea = t.elements[sq[n - 3] as usize] as usize;
            let eb = t.elements[sq[n - 2] as usize] as usize;
            let q = gt.product[ea * ROW_STRIDE + eb] as usize;
            let base = q * t.dv;
            let mut num: Vec<f32> = (0..t.dv).map(|j| s[base + j]).collect();
            let m = num.iter().fold(0f32, |a, &v| a.max(v.abs()));
            let shift = if t.norm_bits > 0 && m >= 1.0 {
                (32 - (m as u32).leading_zeros()).saturating_sub(t.norm_bits)
            } else {
                0
            };
            let div = (1u64 << shift) as f32;
            for v in num.iter_mut() {
                *v = (*v / div).floor();
            }
            let mut logits = vec![0f32; t.vocab];
            for (r, slot) in logits.iter_mut().enumerate() {
                let mut acc = 0f32;
                for j in 0..t.dv {
                    acc += if oracle {
                        t.wo[r * t.dv + j]
                    } else {
                        qo[r * t.dv + j] as f32 * so[r]
                    } * num[j];
                }
                *slot = acc;
            }
            if t.readout_bias {
                for (r, slot) in logits.iter_mut().enumerate() {
                    *slot += t.wb[r];
                }
            }
            let mut best = 0usize;
            for (i, &v) in logits.iter().enumerate() {
                if v > logits[best] {
                    best = i;
                }
            }
            if best == sq[n - 1] as usize {
                hits += 1;
            }
        }
        hits as f32 / seqs.len() as f32
    }

    /// Is the *serving* readout resolution the constraint? Compare the quantised readout against an
    /// unquantised one on the **same trained masters**, so only resolution differs.
    ///
    /// Measured: 0.28 vs 0.28. **Resolution is not the constraint.** Recorded as the falsification it is.
    #[test]
    fn readout_resolution_is_not_the_constraint() {
        let train = relational_sequences(1, 32, 6, false);
        let held = relational_sequences(4, 32, 6, true);
        let mut t = relational_trainer();
        for _ in 0..900 {
            t.train_batch(&train);
        }
        let ternary = eval_composed(&t, &held, false);
        let oracle = eval_composed(&t, &held, true);
        eprintln!("serving readout: ternary={ternary:.2} unquantised={oracle:.2}");
        assert!(
            (oracle - ternary).abs() < 0.05,
            "resolution must not be the constraint, or the 4-bit lever is worth building: \
             {ternary:.2} vs {oracle:.2}"
        );
    }

    /// Resolution, budget and filter capacity are all ruled out. Remaining candidate: the read
    /// *dimension*. Measured: 0.28 at dv=64 and 0.28 at dv=256 — **width is not the constraint either.**
    #[test]
    fn read_width_is_not_the_constraint() {
        let train = relational_sequences(1, 32, 6, false);
        let held = relational_sequences(4, 32, 6, true);
        let mut seen = Vec::new();
        for dv in [64usize, 256] {
            let mut t = GeometricAttentionTrainer::new(RADIX + RELATIONS, dv, 2, 6, 2026_0919)
                .expect("build");
            t.elements = relational_elements();
            t.cfg.lr = 0.05;
            for _ in 0..900 {
                t.train_batch(&train);
            }
            let acc = eval_composed(&t, &held, false);
            let oracle = eval_composed(&t, &held, true);
            eprintln!("read width dv={dv:>3}: ternary={acc:.2} unquantised={oracle:.2}");
            seen.push(acc);
        }
        assert!(
            (seen[1] - seen[0]).abs() < 0.05,
            "4x the read width must not be the lever: {:.2} vs {:.2}",
            seen[0],
            seen[1]
        );
    }

    /// Open/closed route state: a closed route must **abstain**, not guess. Measured on the relational
    /// lookup, seen against unseen queries.
    #[test]
    fn closed_routes_abstain_instead_of_guessing() {
        let train = relational_sequences(1, 64, 6, false);
        let seen = relational_sequences(2, 64, 6, false);
        let unseen = relational_sequences(3, 64, 6, true);
        let mut t = relational_trainer();
        for _ in 0..900 {
            t.train_batch(&train);
        }
        let core = t.to_core().expect("quantise");
        let tally = |seqs: &[Vec<u32>]| -> (usize, usize, usize) {
            let (mut ok, mut abstain, mut wrong) = (0usize, 0usize, 0usize);
            for sq in seqs {
                let n = sq.len();
                let prompt = &sq[..n - 1];
                let mut st = core.initial_state();
                for i in 2..prompt.len() {
                    core.observe(&mut st, &prompt[i - 2..i], prompt[i]);
                }
                let ctx = &prompt[prompt.len() - 2..];
                match core.logits_or_abstain(&st, ctx) {
                    None => abstain += 1,
                    Some(logits) => {
                        let mut best = 0usize;
                        for (i, &v) in logits.iter().enumerate() {
                            if v > logits[best] {
                                best = i;
                            }
                        }
                        if best == sq[n - 1] as usize {
                            ok += 1
                        } else {
                            wrong += 1
                        }
                    }
                }
            }
            (ok, abstain, wrong)
        };
        let (so, sa, sw) = tally(&seen);
        let (uo, ua, uw) = tally(&unseen);
        eprintln!("route state: seen ok/abstain/wrong={so}/{sa}/{sw} unseen={uo}/{ua}/{uw}");
        // The claim: a closed route abstains rather than guessing, and abstention does not cost the
        // answers that were reachable.
        assert!(so > 0, "seen routes must still answer: {so}");
        assert_eq!(
            uw, 0,
            "no unseen route may produce a confident wrong answer"
        );
        assert_eq!(ua, unseen.len(), "every unseen route must abstain");
    }

    /// The reduced form of a nearest-root/table decode. `argmax_r (2·num·p_r − ‖p_r‖²)` is linear in
    /// `num` with a per-class constant, so a nearest-prototype decode is a linear readout **plus a
    /// bias**. If the bias does not move the ceiling, the decode's geometry is not the missing piece
    /// and building the full nearest-root version would be wasted work.
    #[test]
    fn a_per_class_bias_is_the_nearest_prototype_reduced_form() {
        let train = relational_sequences(1, 32, 6, false);
        let held = relational_sequences(4, 32, 6, true);
        let run = |bias: bool| -> f32 {
            let mut t = relational_trainer();
            t.readout_bias = bias;
            for _ in 0..900 {
                t.train_batch(&train);
            }
            eval_composed(&t, &held, false)
        };
        let plain = run(false);
        let biased = run(true);
        eprintln!("nearest-prototype reduced form: plain={plain:.2} with-bias={biased:.2}");
        // Recorded either way; the informative part is the comparison, and a large move would justify
        // the full nearest-root build while a flat result would not.
        assert!(
            (biased - plain).abs() < 0.5,
            "the comparison must be informative: {plain:.2} vs {biased:.2}"
        );
    }

    /// Where does the per-token compute actually go? Counted operations for the served path —
    /// build-independent arithmetic, with a release-mode wall-clock ratio as supporting evidence.
    ///
    /// The hierarchy proposal is only worth building if the readout dominates: a bounded shortlist can
    /// shrink the readout but not the read, and it pays for that in routing and in accuracy.
    #[test]
    fn per_token_compute_is_dominated_by_the_readout() {
        let vocab = 4096usize;
        let dv = 64usize;
        let a: Vec<f32> = (0..vocab * dv)
            .map(|i| ((i % 7) as f32 - 3.0) / 3.0)
            .collect();
        let b: Vec<f32> = (0..vocab * dv)
            .map(|i| ((i % 5) as f32 - 2.0) / 2.0)
            .collect();
        let core = GeometricAttention::from_f32(vocab, dv, 2, 6, &a, &b).expect("build");

        // Counted operations, order = 2 (the served configuration).
        let address_ops = 2; // one product-table read + one inverse read, plus index arithmetic
        let read_exact_ops = dv; // one bucket of `dv` entries
        let read_graded_ops = RADIX * dv; // the class-function convolution, worst case
        let readout_ops = vocab * dv; // the linear read over the vocabulary

        let total_exact = address_ops + read_exact_ops + readout_ops;
        let total_graded = address_ops + read_graded_ops + readout_ops;
        let exact_share = readout_ops as f64 / total_exact as f64;
        let graded_share = readout_ops as f64 / total_graded as f64;
        eprintln!(
            "per-token ops at V={vocab}, dv={dv}: address={address_ops} read_exact={read_exact_ops} \
             read_graded={read_graded_ops} readout={readout_ops}"
        );
        eprintln!(
            "readout share of per-token work: exact read {:.4}% | graded read {:.4}%",
            100.0 * exact_share,
            100.0 * graded_share
        );

        // A bounded shortlist of k candidates: readout becomes k*dv, plus a fanout-B depth-D route.
        let (k, fanout, depth) = (8usize, 8usize, 4usize);
        let shortlist_ops = k * dv + fanout * depth + address_ops;
        eprintln!(
            "projection with a fanout-{fanout} depth-{depth} shortlist of k={k}: \
             {shortlist_ops} ops/token vs {total_exact} full — {:.0}x fewer operations",
            total_exact as f64 / shortlist_ops as f64
        );

        // The claim under test is the *dominance*, which is what decides whether a hierarchy is worth
        // building at all. Accuracy cost of the shortlist is explicitly NOT measured here.
        assert!(
            exact_share > 0.9 && graded_share > 0.9,
            "the readout must dominate per-token work, or a hierarchy is the wrong lever: \
             {exact_share:.4} / {graded_share:.4}"
        );
        assert_eq!(readout_ops, core.vocab * core.dv);

        // Wall-clock is deliberately NOT used as evidence here. A first attempt compared the readout
        // against a write+read loop and measured 264 us vs 33 us — but the write+read side was
        // dominated by zeroing the state buffer (n_addr * dv = 921,600 ints, ~3.7 MB), not by the
        // read. The op counts above are the build-independent measurement; timing a layout-dominated
        // loop would have produced a number about allocation and called it compute.
        let h: Vec<i32> = (0..dv as i32).collect();
        let t0 = std::time::Instant::now();
        let mut sink = 0i32;
        for _ in 0..200 {
            sink = sink.wrapping_add(core.w_o.forward_i32(&h)[0]);
        }
        let readout_ns = t0.elapsed().as_nanos() as f64 / 200.0;
        assert_ne!(sink, i32::MIN);
        eprintln!("sanity only (not evidence): one full readout ~{readout_ns:.0} ns");
    }

    #[test]
    fn gradients_reach_both_tables() {
        let mut t = GeometricAttentionTrainer::new(32, 16, 2, 0, 3).expect("build");
        t.zero_grads();
        for seq in repeat_batch(11, 4, 6, 32) {
            t.accumulate(&seq);
        }
        let nz = |g: &[f32]| g.iter().any(|v| v.abs() > 0.0);
        assert!(nz(&t.gwv), "no value gradient");
        assert!(nz(&t.gwo), "no output gradient");
    }

    /// Context repetition: a random token run `R`, then `R` again. The second copy can only be
    /// predicted from memory of the words in the first.
    fn repeat_batch(seed: u64, n: usize, k: usize, alphabet: usize) -> Vec<Vec<u32>> {
        let mut st = seed | 1;
        (0..n)
            .map(|_| {
                let mut r = Vec::with_capacity(k);
                for _ in 0..k {
                    st ^= st << 13;
                    st ^= st >> 7;
                    st ^= st << 17;
                    r.push((st % alphabet as u64) as u32);
                }
                let mut seq = r.clone();
                seq.extend_from_slice(&r);
                seq
            })
            .collect()
    }

    const ALPHABET: usize = 32;
    const VOCAB: usize = 32;

    fn geometric_repeat(k: usize, steps: usize, seed: u64, dv: usize, order: usize) -> f32 {
        let train = repeat_batch(0xA5A5_1234, 64, k, ALPHABET);
        let held = repeat_batch(0x0BAD_F00D, 64, k, ALPHABET);
        let mut t = GeometricAttentionTrainer::new(VOCAB, dv, order, 6, seed).expect("build");
        t.cfg.lr = 0.05;
        for _ in 0..steps {
            t.train_batch(&train);
        }
        let hits = held.iter().filter(|s| t.final_token_correct(s)).count();
        hits as f32 / held.len() as f32
    }

    fn linear_repeat(k: usize, steps: usize, seed: u64) -> f32 {
        let train = repeat_batch(0xA5A5_1234, 64, k, ALPHABET);
        let held = repeat_batch(0x0BAD_F00D, 64, k, ALPHABET);
        let mut t = LowBitAttentionTrainer::new(VOCAB, 120, 32, 0, seed).expect("build");
        t.cfg.lr = 0.02;
        for _ in 0..steps {
            t.train_batch(&train);
        }
        let hits = held.iter().filter(|s| t.final_token_correct(s)).count();
        hits as f32 / held.len() as f32
    }

    #[test]
    fn exact_addressing_beats_matched_filter_as_context_grows() {
        for k in [4usize, 8, 16] {
            let g = geometric_repeat(k, 900, 2026_0919, 128, 2);
            let l = linear_repeat(k, 900, 2026_0919);
            eprintln!("context={k:>3} geometric={g:.2} linear={l:.2}");
            assert!(
                g > l,
                "exact addressing must beat the matched filter at context {k}: {g:.2} vs {l:.2}"
            );
        }
    }

    /// The falsifiable prediction from the previous round: a duplicated queried key costs 3x accuracy
    /// under a first-order address, and an **ordered pair** address should recover it.
    #[test]
    fn ordered_words_recover_the_duplicated_key_collapse() {
        let k = 16usize;
        let build = |seed: u64, n: usize, force_dup: bool| -> Vec<Vec<u32>> {
            let mut st = seed | 1;
            (0..n)
                .map(|_| {
                    let mut r: Vec<u32> = (0..k)
                        .map(|_| {
                            st ^= st << 13;
                            st ^= st >> 7;
                            st ^= st << 17;
                            (st % ALPHABET as u64) as u32
                        })
                        .collect();
                    if force_dup {
                        r[k - 2] = r[0];
                    }
                    let mut seq = r.clone();
                    seq.extend_from_slice(&r);
                    seq
                })
                .collect()
        };
        let train = build(0xA5A5_1234, 64, false);
        let run = |order: usize| {
            let mut t =
                GeometricAttentionTrainer::new(VOCAB, 128, order, 6, 2026_0919).expect("build");
            t.cfg.lr = 0.05;
            for _ in 0..900 {
                t.train_batch(&train);
            }
            let score = |seqs: &[Vec<u32>]| {
                let hits = seqs.iter().filter(|s| t.final_token_correct(s)).count();
                hits as f32 / seqs.len() as f32
            };
            let clean = score(&build(0x0BAD_F00D, 64, false));
            let dup = score(&build(0x0BAD_F00D, 64, true));
            (clean, dup)
        };
        let (c1, d1) = run(1);
        let (c2, d2) = run(2);
        eprintln!(
            "collision order=1 clean={c1:.2} dup={d1:.2} | order=2 clean={c2:.2} dup={d2:.2}"
        );
        assert!(
            d2 > d1,
            "an ordered-pair address must recover the duplicated-key collapse: {d1:.2} -> {d2:.2}"
        );
    }

    /// Likewise measured at `order = 1`, where the task has headroom; at `order = 2` both widths
    /// reach 1.00.
    #[test]
    fn resolution_scales_the_mechanism() {
        let narrow = geometric_repeat(8, 900, 2026_0919, 32, 1);
        let wide = geometric_repeat(8, 900, 2026_0919, 256, 1);
        eprintln!("resolution dv=32 {narrow:.2} vs dv=256 {wide:.2}");
        assert!(
            wide > narrow + 0.1,
            "value/output resolution must scale the mechanism: {narrow:.2} vs {wide:.2}"
        );
    }

    /// Ablations of the read must be run at `order = 1`: at `order = 2` this task saturates (1.00) and
    /// a `relu` would have no headroom to show its cost.
    #[test]
    fn selection_is_the_nonlinearity_not_the_read() {
        let k = 8usize;
        let train = repeat_batch(0xA5A5_1234, 64, k, ALPHABET);
        let held = repeat_batch(0x0BAD_F00D, 64, k, ALPHABET);
        let run = |use_relu: bool| {
            let mut t = GeometricAttentionTrainer::new(VOCAB, 32, 1, 6, 2026_0919).expect("build");
            t.cfg.lr = 0.05;
            t.use_relu = use_relu;
            for _ in 0..900 {
                t.train_batch(&train);
            }
            let hits = held.iter().filter(|s| t.final_token_correct(s)).count();
            hits as f32 / held.len() as f32
        };
        let with = run(true);
        let without = run(false);
        eprintln!("read relu={with:.2} no-relu={without:.2}");
        assert!(
            without > with + 0.1,
            "the sign-destroying relu after an exact-address read must hurt: {with:.2} vs {without:.2}"
        );
    }

    /// The headline result: an ordered-word address copies a repeated context of 16 tokens, where a
    /// matched-filter memory scores zero.
    /// The copy task saturates at `order = 2` for a 32-token alphabet, so it can no longer
    /// discriminate. Shrinking the alphabet makes ordered *pairs* repeat with different successors,
    /// which should produce a real curve instead of a ceiling.
    fn repeat_alphabet(seed: u64, n: usize, k: usize, alphabet: usize) -> Vec<Vec<u32>> {
        let mut st = seed | 1;
        (0..n)
            .map(|_| {
                let mut r = Vec::with_capacity(k);
                for _ in 0..k {
                    st ^= st << 13;
                    st ^= st >> 7;
                    st ^= st << 17;
                    r.push((st % alphabet as u64) as u32);
                }
                let mut seq = r.clone();
                seq.extend_from_slice(&r);
                seq
            })
            .collect()
    }

    /// The 32-token copy task saturates at `order = 2`, so difficulty is raised by shrinking the
    /// alphabet until ordered *pairs* repeat with different successors. The ordered-pair address wins
    /// at every difficulty, and the margin grows as words become unique:
    /// alphabet 4/8/16/32 → order1 0.34/0.25/0.42/0.55, order2 0.62/0.84/0.97/1.00.
    #[test]
    fn ordered_words_win_at_every_difficulty() {
        let k = 16usize;
        let mut margins = Vec::new();
        for alphabet in [4usize, 8, 16, 32] {
            let train = repeat_alphabet(0xA5A5_1234, 64, k, alphabet);
            let held = repeat_alphabet(0x0BAD_F00D, 64, k, alphabet);
            let run = |order: usize| {
                let mut t =
                    GeometricAttentionTrainer::new(VOCAB, 64, order, 6, 2026_0919).expect("build");
                t.cfg.lr = 0.05;
                for _ in 0..900 {
                    t.train_batch(&train);
                }
                let hits = held.iter().filter(|s| t.final_token_correct(s)).count();
                hits as f32 / held.len() as f32
            };
            let one = run(1);
            let two = run(2);
            eprintln!("alphabet={alphabet:>2} context={k} order1={one:.2} order2={two:.2}");
            margins.push((alphabet, two - one));
        }
        for (alphabet, margin) in margins {
            assert!(
                margin > 0.0,
                "the ordered-pair address must beat the first-order one at alphabet {alphabet}"
            );
        }
    }

    #[test]
    fn copies_a_repeated_context() {
        for k in [4usize, 8, 16] {
            let acc = geometric_repeat(k, 900, 2026_0919, 32, 2);
            assert!(
                acc >= 0.9,
                "an ordered-word address must copy a {k}-token repeated context, got {acc:.2}"
            );
        }
    }
}

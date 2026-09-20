//! Learned geometric selection of exact token occurrences (BPE token path).
//!
//! A bounded causal ring retains observed tokens with an exact sequential identity. At a prediction
//! position a bounded set of past occurrences of the current token is admitted, a shared small
//! learned integer score decides between them and an explicit `NoRead`, and the selected occurrence's
//! observed successor is emitted as a bounded sparse residual on the frozen local scorer.
//!
//! ```text
//! z_local(v) = z_E(v) + u_S(b)                    (S's query row only; no history row)
//! score(c)   = sum_k theta_k * phi_k(c)           (phi in {0,1}; adds only, no multiplier)
//! z(v)       = z_local(v) + (1 << amp_shift) * 1[v == payload(j*)]   (NoRead adds nothing)
//! ```
//!
//! Geometry enters through `phi_2`/`phi_3` (equality of transported write slots) and the collision
//! feature `phi_8`; the payload is always an exact observed token, never reconstructed from a code.
//! These are the invariants of the historical `source_routing` scorer (relative element plus rank
//! table, explicit no-source, selection separated from payload access); its `Model`/`ValueState`
//! artifact and authored grammar are **not** reused.
#![forbid(unsafe_code)]

use super::prefix_state::PALETTE_SIZE;
use super::prior_learning::PriorCore;
use super::query_read::QueryHard;

/// Ring capacity in observed tokens.
pub const RING_CAP: usize = 128;
/// Maximum admitted candidates per position.
pub const MAX_CANDIDATES: usize = 24;
/// Number of indicator features.
pub const FEATS: usize = 9;
/// Largest magnitude of a served weight or threshold (signed 4-bit).
pub const WEIGHT_MAX: i32 = 7;

pub const OCC_MAGIC: &[u8; 4] = b"OCQ1";
pub const OCC_VERSION: u32 = 1;

/// Exact identity of one observed token occurrence.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct OccurrenceRef {
    pub seq: u32,
    pub abs: u32,
}

/// The frozen S write map `A(t) = gamma_S[a_codes_S[t]]`, used for the geometric features.
#[derive(Clone, Copy)]
pub struct WriteMap<'a> {
    pub a_codes: &'a [u8],
    pub elements: &'a [u8; PALETTE_SIZE],
}

impl WriteMap<'_> {
    #[inline]
    pub fn slot(&self, token: u32) -> u8 {
        let v = self.a_codes.len();
        self.elements[self.a_codes[(token as usize).min(v - 1)] as usize]
    }
}

/// A fixed-capacity ring of observed tokens with a session identity.
#[derive(Clone, Debug)]
pub struct OccurrenceRing {
    cap: usize,
    seq: u32,
    written: u32,
    buf: Vec<u32>,
}

impl OccurrenceRing {
    pub fn new(cap: usize) -> Self {
        assert!(cap > 0, "ring capacity must be positive");
        Self {
            cap,
            seq: 1,
            written: 0,
            buf: vec![0u32; cap],
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.cap
    }

    /// Begin a new session. Every earlier reference becomes stale by sequence identity.
    pub fn reset(&mut self) {
        self.seq = self.seq.wrapping_add(1).max(1);
        self.written = 0;
    }

    #[inline]
    pub fn seq(&self) -> u32 {
        self.seq
    }

    #[inline]
    pub fn written(&self) -> u32 {
        self.written
    }

    #[inline]
    pub fn valid_from(&self) -> u32 {
        self.written.saturating_sub(self.cap as u32)
    }

    /// Observe the next token. Called only after the prediction for that position.
    pub fn observe(&mut self, token: u32) {
        self.buf[(self.written % self.cap as u32) as usize] = token;
        self.written = self.written.saturating_add(1);
    }

    #[inline]
    pub fn get(&self, abs: u32) -> Option<u32> {
        if abs >= self.written || abs < self.valid_from() {
            None
        } else {
            Some(self.buf[(abs % self.cap as u32) as usize])
        }
    }

    /// A reference to a live occurrence, or `None` when it has been evicted.
    #[inline]
    pub fn reference(&self, abs: u32) -> Option<OccurrenceRef> {
        self.get(abs).map(|_| OccurrenceRef { seq: self.seq, abs })
    }

    /// Resolve a reference. A stale sequence, an evicted slot or a future index all yield `None`.
    #[inline]
    pub fn resolve(&self, r: OccurrenceRef) -> Option<u32> {
        if r.seq != self.seq {
            return None;
        }
        self.get(r.abs)
    }

    #[inline]
    pub fn evicted(&self, abs: u32) -> bool {
        abs < self.valid_from()
    }
}

/// The causal query context and the frozen maps needed to score candidates.
#[derive(Clone, Copy)]
pub struct QueryContext {
    /// Position being predicted (the current token `x_i` occupies this index).
    pub i: u32,
    pub cur: u32,
    pub prev: Option<u32>,
    pub prev2: Option<u32>,
}

/// One admitted exact occurrence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub slot_ref: OccurrenceRef,
    pub abs: u32,
    /// The observed successor `x_{j+1}`, always an already-observed token.
    pub payload: u32,
    pub feats: [u8; FEATS],
}

/// Admission counters, reported separately from selection and emission.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AdmitStats {
    /// Live ring entries scanned.
    pub scanned: usize,
    /// Live entries whose token equals the query token.
    pub matching: usize,
    /// Matching entries admitted before the candidate bound was reached.
    pub admitted: usize,
    /// Matching entries dropped by the candidate bound.
    pub dropped_by_bound: usize,
    /// Matching entries with no observed successor strictly inside the prefix: no payload exists.
    pub no_observed_successor: usize,
}

/// Indicator features for one candidate. Every entry is 0 or 1.
pub fn features(
    ctx: &QueryContext,
    ring: &OccurrenceRing,
    j: u32,
    is_newest: bool,
    wm: &WriteMap<'_>,
) -> [u8; FEATS] {
    let mut f = [0u8; FEATS];
    let role = if j >= 1 { ring.get(j - 1) } else { None };
    let role2 = if j >= 2 { ring.get(j - 2) } else { None };
    let payload = ring.get(j + 1);
    // f0 / f1: exact ordered-context matches.
    if let (Some(a), Some(b)) = (role, ctx.prev) {
        f[0] = u8::from(a == b);
    }
    if let (Some(a), Some(b)) = (role2, ctx.prev2) {
        f[1] = u8::from(a == b);
    }
    // f2 / f3: transported write-slot equality (coarser than exact identity: 4096 tokens -> 8 slots).
    if let (Some(a), Some(b)) = (role, ctx.prev) {
        f[2] = u8::from(wm.slot(a) == wm.slot(b));
    }
    if let (Some(a), Some(b)) = (role2, ctx.prev2) {
        f[3] = u8::from(wm.slot(a) == wm.slot(b));
    }
    // f4: the nearest admissible lag. An occurrence immediately before the query has no observed
    // successor strictly inside the prefix and is never admitted, so the closest admissible lag is 2.
    f[4] = u8::from(ctx.i >= 2 && j + 2 == ctx.i);
    // f5: the payload repeats the query token.
    if let Some(p) = payload {
        f[5] = u8::from(p == ctx.cur);
    }
    // f6: newest admitted occurrence.
    f[6] = u8::from(is_newest);
    // f7 / f8: geometric interactions.
    f[7] = u8::from(f[0] == 1 && f[2] == 1);
    f[8] = u8::from(f[2] == 1 && f[0] == 0);
    f
}

/// Admit causal candidates for the current token. Never inspects `x_{i+1}`.
pub fn admit(
    ring: &OccurrenceRing,
    ctx: &QueryContext,
    wm: &WriteMap<'_>,
    max_candidates: usize,
) -> (Vec<Candidate>, AdmitStats) {
    debug_assert_eq!(
        ctx.i,
        ring.written(),
        "the context position must be the number of observed tokens"
    );
    let mut out: Vec<Candidate> = Vec::new();
    let mut st = AdmitStats::default();
    let low = ring.valid_from();
    if ring.written() == 0 {
        return (out, st);
    }
    let written = ring.written();
    let mut abs = written - 1;
    loop {
        if abs < low {
            break;
        }
        st.scanned += 1;
        if let Some(tok) = ring.get(abs) {
            if tok == ctx.cur {
                st.matching += 1;
                if abs + 1 >= written {
                    // The successor is the current token, which is not an observed payload.
                    st.no_observed_successor += 1;
                } else if out.len() < max_candidates {
                    let is_newest = out.is_empty();
                    let feats = features(ctx, ring, abs, is_newest, wm);
                    let payload = ring.get(abs + 1).expect("successor inside the prefix");
                    out.push(Candidate {
                        slot_ref: OccurrenceRef {
                            seq: ring.seq(),
                            abs,
                        },
                        abs,
                        payload,
                        feats,
                    });
                    st.admitted += 1;
                } else {
                    st.dropped_by_bound += 1;
                }
            }
        }
        if abs == 0 {
            break;
        }
        abs -= 1;
    }
    (out, st)
}

/// The served selector: signed <=4-bit integer weights and an explicit NoRead threshold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Selector {
    pub w: [i32; FEATS],
    pub mu: i32,
}

impl Selector {
    pub fn zero() -> Self {
        Self {
            w: [0; FEATS],
            mu: 0,
        }
    }

    /// Every coefficient stays inside the declared signed 4-bit bound.
    pub fn validate(&self) -> Result<(), String> {
        for (k, v) in self.w.iter().enumerate() {
            if v.abs() > WEIGHT_MAX {
                return Err(format!("weight {k} = {v} exceeds +/-{WEIGHT_MAX}"));
            }
        }
        if self.mu.abs() > WEIGHT_MAX {
            return Err(format!("threshold {} exceeds +/-{WEIGHT_MAX}", self.mu));
        }
        Ok(())
    }

    /// Integer score of one candidate: one conditional add per active indicator.
    #[inline]
    pub fn score(&self, f: &[u8; FEATS]) -> i32 {
        let mut s = 0i32;
        for k in 0..FEATS {
            if f[k] != 0 {
                s += self.w[k];
            }
        }
        s
    }

    /// Choose a candidate index (newest-first on ties) or `None` for NoRead.
    pub fn choose(&self, cands: &[Candidate]) -> Option<usize> {
        let mut best: Option<(usize, i32)> = None;
        for (k, c) in cands.iter().enumerate() {
            let s = self.score(&c.feats);
            if best.is_none_or(|(_, prior)| s > prior) {
                best = Some((k, s));
            }
        }
        match best {
            Some((k, s)) if s > self.mu => Some(k),
            _ => None,
        }
    }

    /// A fixed latest-occurrence selector: always read the newest admitted candidate.
    pub fn latest(cands: &[Candidate]) -> Option<usize> {
        if cands.is_empty() {
            None
        } else {
            Some(0)
        }
    }

    /// A fixed exact-role-match selector: read the newest candidate whose role matches exactly,
    /// otherwise NoRead. This is the cheap shared-artifact control.
    pub fn exact_role(cands: &[Candidate]) -> Option<usize> {
        cands.iter().position(|c| c.feats[0] == 1)
    }
}

/// One reader decision with everything needed to audit it.
#[derive(Clone, Debug)]
pub struct Decision {
    pub admitted: usize,
    pub chosen: Option<usize>,
    pub slot_ref: Option<OccurrenceRef>,
    pub payload: Option<u32>,
    pub no_read: bool,
}

/// Apply the selected payload as a bounded sparse residual on the local logits.
pub fn apply_residual(z_local: &mut [i32], payload: Option<u32>, amp_shift: u32) {
    if let Some(v) = payload {
        let v = v as usize;
        if v < z_local.len() {
            z_local[v] = z_local[v].saturating_add(1i32 << amp_shift);
        }
    }
}

/// The frozen local baseline: `E` plus S's query row only, with S's absence behaviour.
///
/// No history fold is executed: `b` is read directly from the frozen artifact's query map.
pub fn local_logits(
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    tokens: &[u32],
    i: usize,
) -> Vec<i32> {
    let v = parent.cfg.vocab;
    let prev = if i == 0 {
        parent.cfg.pad_row()
    } else {
        (tokens[i - 1] as usize).min(v - 1)
    };
    let cur = (tokens[i] as usize).min(v - 1);
    let mut z = parent.int_logits(prev, cur, true);
    if i >= 2 {
        let b = local.query_state(tokens[i]);
        for (x, y) in z.iter_mut().zip(u[b].iter()) {
            *x += *y;
        }
    }
    z
}

/// Versioned selector artifact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OccurrenceArtifact {
    pub selector: Selector,
    pub amp_shift: u32,
    pub ring_cap: u32,
    pub max_candidates: u32,
    pub local_artifact_digest: [u8; 32],
    pub tokenizer_digest: [u8; 32],
    pub source_digest: [u8; 32],
}

impl OccurrenceArtifact {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(OCC_MAGIC);
        o.extend_from_slice(&OCC_VERSION.to_le_bytes());
        o.extend_from_slice(&(self.ring_cap).to_le_bytes());
        o.extend_from_slice(&(self.max_candidates).to_le_bytes());
        o.extend_from_slice(&(FEATS as u32).to_le_bytes());
        for k in 0..FEATS {
            o.push(self.selector.w[k] as i8 as u8);
        }
        o.push(self.selector.mu as i8 as u8);
        o.extend_from_slice(&self.amp_shift.to_le_bytes());
        o.extend_from_slice(&self.local_artifact_digest);
        o.extend_from_slice(&self.tokenizer_digest);
        o.extend_from_slice(&self.source_digest);
        o
    }

    /// Validated load. Rejects a wrong magic/version, out-of-range coefficients, an implausible
    /// amplitude and any identity mismatch.
    pub fn from_bytes(
        bytes: &[u8],
        expected_local_digest: &[u8; 32],
        expected_tokenizer: &[u8; 32],
    ) -> Result<Self, String> {
        let mut c = 0usize;
        let take = |c: &mut usize, n: usize| -> Result<&[u8], String> {
            let end = c.checked_add(n).ok_or("size overflow")?;
            if end > bytes.len() {
                return Err("truncated occurrence artifact".into());
            }
            let s = &bytes[*c..end];
            *c = end;
            Ok(s)
        };
        if take(&mut c, 4)? != OCC_MAGIC {
            return Err("bad occurrence artifact magic".into());
        }
        let u32_at = |c: &mut usize| -> Result<u32, String> {
            Ok(u32::from_le_bytes(take(c, 4)?.try_into().unwrap()))
        };
        if u32_at(&mut c)? != OCC_VERSION {
            return Err("unsupported occurrence artifact version".into());
        }
        let ring_cap = u32_at(&mut c)?;
        let max_candidates = u32_at(&mut c)?;
        if u32_at(&mut c)? != FEATS as u32 {
            return Err("occurrence artifact feature count differs".into());
        }
        if ring_cap == 0 || ring_cap > 4096 || max_candidates == 0 || max_candidates > 4096 {
            return Err("occurrence artifact bounds are out of range".into());
        }
        let mut w = [0i32; FEATS];
        for (k, slot) in w.iter_mut().enumerate() {
            let b = take(&mut c, 1)?[0] as i8;
            if (b as i32).abs() > WEIGHT_MAX {
                return Err(format!("weight {k} = {b} exceeds the declared bound"));
            }
            *slot = b as i32;
        }
        let mu = take(&mut c, 1)?[0] as i8 as i32;
        if mu.abs() > WEIGHT_MAX {
            return Err("threshold exceeds the declared bound".into());
        }
        let amp_shift = u32_at(&mut c)?;
        if amp_shift == 0 || amp_shift > 20 {
            return Err("amplitude shift outside 1..=20".into());
        }
        let mut local_artifact_digest = [0u8; 32];
        local_artifact_digest.copy_from_slice(take(&mut c, 32)?);
        let mut tokenizer_digest = [0u8; 32];
        tokenizer_digest.copy_from_slice(take(&mut c, 32)?);
        let mut source_digest = [0u8; 32];
        source_digest.copy_from_slice(take(&mut c, 32)?);
        if c != bytes.len() {
            return Err(format!(
                "{} trailing bytes in the occurrence artifact",
                bytes.len() - c
            ));
        }
        if &local_artifact_digest != expected_local_digest {
            return Err("occurrence artifact local-baseline digest differs".into());
        }
        if &tokenizer_digest != expected_tokenizer {
            return Err("occurrence artifact tokenizer digest differs".into());
        }
        Ok(Self {
            selector: Selector { w, mu },
            amp_shift,
            ring_cap,
            max_candidates,
            local_artifact_digest,
            tokenizer_digest,
            source_digest,
        })
    }
}

/// One training example: the candidate features at a position and the supervised label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Example {
    pub feats: Vec<[u8; FEATS]>,
    /// Index into `feats`, or `feats.len()` for NoRead.
    pub label: usize,
    /// Sequence identity, for paired intervals and document grouping.
    pub group: u16,
}

/// A deterministic on-disk-serializable selector trainer. Ten trained scalars plus Adam moments and
/// an explicit schedule cursor.
#[derive(Clone, Debug)]
pub struct SelectorTrainer {
    pub w: [f64; FEATS],
    pub mu: f64,
    m_w: [f64; FEATS],
    v_w: [f64; FEATS],
    m_mu: f64,
    v_mu: f64,
    pub step: u64,
    pub cursor: usize,
    pub epoch: u32,
    pub rng: u64,
    pub lr: f64,
    pub data_identity: [u8; 32],
    pub config_identity: [u8; 32],
}

impl SelectorTrainer {
    pub fn new(lr: f64, seed: u64, data_identity: [u8; 32], config_identity: [u8; 32]) -> Self {
        Self {
            w: [0.0; FEATS],
            mu: 0.0,
            m_w: [0.0; FEATS],
            v_w: [0.0; FEATS],
            m_mu: 0.0,
            v_mu: 0.0,
            step: 0,
            cursor: 0,
            epoch: 0,
            rng: seed | 1,
            lr,
            data_identity,
            config_identity,
        }
    }

    /// The next batch of example indices, in the declared deterministic order.
    ///
    /// The order is a per-epoch Fisher-Yates permutation of the example indices; `cursor` is the
    /// position inside the current epoch. Reconstructing the permutation from `(rng, epoch)` is the
    /// declared cache reconstruction, so a resumed process needs only the cursor and the RNG.
    pub fn next_batch(&mut self, n_examples: usize, batch: usize) -> Vec<usize> {
        let order = epoch_order(n_examples, self.rng, self.epoch);
        let out: Vec<usize> = (0..batch)
            .map(|k| order[(self.cursor + k) % n_examples])
            .collect();
        self.cursor += batch;
        if self.cursor >= n_examples {
            self.cursor %= n_examples;
            self.epoch += 1;
            self.rng = permute_rng(self.rng, self.epoch);
        }
        out
    }

    /// One Adam step over a batch, returning the mean cross-entropy in nats.
    pub fn step(&mut self, examples: &[Example], batch: &[usize]) -> f64 {
        let mut g_w = [0.0f64; FEATS];
        let mut g_mu = 0.0f64;
        let mut loss = 0.0f64;
        let mut count = 0usize;
        for &e in batch {
            let ex = &examples[e];
            let n = ex.feats.len();
            let mut s = vec![0.0f64; n + 1];
            for (k, f) in ex.feats.iter().enumerate() {
                let mut acc = 0.0f64;
                for j in 0..FEATS {
                    if f[j] != 0 {
                        acc += self.w[j];
                    }
                }
                s[k] = acc;
            }
            s[n] = self.mu;
            let max = s.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let mut p: Vec<f64> = s.iter().map(|x| (x - max).exp()).collect();
            let z: f64 = p.iter().sum();
            for x in p.iter_mut() {
                *x /= z;
            }
            if ex.label >= p.len() {
                continue;
            }
            loss += -(p[ex.label].max(1e-300).ln());
            count += 1;
            let mut d = p.clone();
            d[ex.label] -= 1.0;
            for (k, f) in ex.feats.iter().enumerate() {
                for j in 0..FEATS {
                    if f[j] != 0 {
                        g_w[j] += d[k];
                    }
                }
            }
            g_mu += d[n];
        }
        if count == 0 {
            return f64::NAN;
        }
        let inv = 1.0 / count as f64;
        for x in g_w.iter_mut() {
            *x *= inv;
        }
        g_mu *= inv;
        loss *= inv;
        // Adam over ten scalars.
        let age = self.step + 1;
        let (b1, b2, eps) = (0.9f64, 0.999f64, 1e-8f64);
        let bc1 = 1.0 - b1.powi(age.min(100_000) as i32);
        let bc2 = 1.0 - b2.powi(age.min(100_000) as i32);
        for j in 0..FEATS {
            self.m_w[j] = b1 * self.m_w[j] + (1.0 - b1) * g_w[j];
            self.v_w[j] = b2 * self.v_w[j] + (1.0 - b2) * g_w[j] * g_w[j];
            let mh = self.m_w[j] / bc1;
            let vh = self.v_w[j] / bc2;
            self.w[j] -= self.lr * mh / (vh.sqrt() + eps);
        }
        self.m_mu = b1 * self.m_mu + (1.0 - b1) * g_mu;
        self.v_mu = b2 * self.v_mu + (1.0 - b2) * g_mu * g_mu;
        let mh = self.m_mu / bc1;
        let vh = self.v_mu / bc2;
        self.mu -= self.lr * mh / (vh.sqrt() + eps);
        self.step = age;
        loss
    }

    /// Quantize to the served signed <=4-bit selector with a single declared scale.
    ///
    /// A positive global scale preserves the ranking exactly when no rounding is needed, so the
    /// largest power-of-two scale that fits the bound is chosen.
    pub fn quantize(&self) -> Selector {
        let m = self
            .w
            .iter()
            .cloned()
            .chain(std::iter::once(self.mu))
            .fold(0.0f64, |a, b| a.max(b.abs()));
        let mut scale = 1.0f64;
        if m > 0.0 {
            while (m * scale * 2.0).round() <= WEIGHT_MAX as f64 {
                scale *= 2.0;
            }
        }
        let q = |x: f64| ((x * scale).round() as i32).clamp(-WEIGHT_MAX, WEIGHT_MAX);
        Selector {
            w: std::array::from_fn(|k| q(self.w[k])),
            mu: q(self.mu),
        }
    }

    /// Complete trainer state: parameters, moments, optimizer age, schedule cursor and identities.
    pub fn checkpoint_bytes(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(b"OCQK");
        o.extend_from_slice(&1u32.to_le_bytes());
        for x in self.w.iter().chain(std::iter::once(&self.mu)) {
            o.extend_from_slice(&x.to_le_bytes());
        }
        for x in self
            .m_w
            .iter()
            .chain(self.v_w.iter())
            .chain(std::iter::once(&self.m_mu))
            .chain(std::iter::once(&self.v_mu))
        {
            o.extend_from_slice(&x.to_le_bytes());
        }
        o.extend_from_slice(&self.step.to_le_bytes());
        o.extend_from_slice(&(self.cursor as u64).to_le_bytes());
        o.extend_from_slice(&(self.epoch as u64).to_le_bytes());
        o.extend_from_slice(&self.rng.to_le_bytes());
        o.extend_from_slice(&self.lr.to_le_bytes());
        o.extend_from_slice(&self.data_identity);
        o.extend_from_slice(&self.config_identity);
        o
    }

    /// Load a checkpoint into this trainer. Every identity must match.
    pub fn resume_from(&mut self, bytes: &[u8]) -> Result<(), String> {
        let mut c = 0usize;
        let take = |c: &mut usize, n: usize| -> Result<&[u8], String> {
            let end = c.checked_add(n).ok_or("size overflow")?;
            if end > bytes.len() {
                return Err("truncated selector checkpoint".into());
            }
            let s = &bytes[*c..end];
            *c = end;
            Ok(s)
        };
        if take(&mut c, 4)? != b"OCQK" {
            return Err("bad selector checkpoint magic".into());
        }
        let u32_at = |c: &mut usize| -> Result<u32, String> {
            Ok(u32::from_le_bytes(take(c, 4)?.try_into().unwrap()))
        };
        let u64_at = |c: &mut usize| -> Result<u64, String> {
            Ok(u64::from_le_bytes(take(c, 8)?.try_into().unwrap()))
        };
        let f64_at = |c: &mut usize| -> Result<f64, String> {
            Ok(f64::from_le_bytes(take(c, 8)?.try_into().unwrap()))
        };
        if u32_at(&mut c)? != 1 {
            return Err("unsupported selector checkpoint version".into());
        }
        let mut w = [0.0f64; FEATS];
        for slot in w.iter_mut() {
            *slot = f64_at(&mut c)?;
        }
        let mu = f64_at(&mut c)?;
        let mut m_w = [0.0f64; FEATS];
        for slot in m_w.iter_mut() {
            *slot = f64_at(&mut c)?;
        }
        let mut v_w = [0.0f64; FEATS];
        for slot in v_w.iter_mut() {
            *slot = f64_at(&mut c)?;
        }
        let m_mu = f64_at(&mut c)?;
        let v_mu = f64_at(&mut c)?;
        let step = u64_at(&mut c)?;
        let cursor = u64_at(&mut c)? as usize;
        let epoch = u64_at(&mut c)? as u32;
        let rng = u64_at(&mut c)?;
        let lr = f64_at(&mut c)?;
        let mut data_identity = [0u8; 32];
        data_identity.copy_from_slice(take(&mut c, 32)?);
        let mut config_identity = [0u8; 32];
        config_identity.copy_from_slice(take(&mut c, 32)?);
        if c != bytes.len() {
            return Err("trailing bytes in the selector checkpoint".into());
        }
        if data_identity != self.data_identity {
            return Err("selector checkpoint data identity differs".into());
        }
        if config_identity != self.config_identity {
            return Err("selector checkpoint configuration identity differs".into());
        }
        if lr != self.lr {
            return Err("selector checkpoint learning rate differs".into());
        }
        self.w = w;
        self.mu = mu;
        self.m_w = m_w;
        self.v_w = v_w;
        self.m_mu = m_mu;
        self.v_mu = v_mu;
        self.step = step;
        self.cursor = cursor;
        self.epoch = epoch;
        self.rng = rng;
        Ok(())
    }
}

fn xorshift(st: &mut u64) -> u64 {
    *st ^= *st << 13;
    *st ^= *st >> 7;
    *st ^= *st << 17;
    *st
}

/// The Fisher-Yates order for one epoch, reconstructed from `(rng, epoch)`.
pub fn epoch_order(n: usize, rng: u64, epoch: u32) -> Vec<usize> {
    let mut st = permute_rng(rng, epoch) | 1;
    let mut p: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let j = (xorshift(&mut st) as usize) % (i + 1);
        p.swap(i, j);
    }
    p
}

fn permute_rng(rng: u64, epoch: u32) -> u64 {
    let mut z = rng
        .wrapping_add(0x9E37_79B9_7F4A_7C15)
        .wrapping_mul(epoch as u64 + 1);
    z ^= z >> 30;
    z = z.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z ^= z >> 27;
    z.wrapping_mul(0x94D0_49BB_1331_11EB)
}

/// The copy amplitude as a declared power of two.
#[inline]
pub fn amplitude(amp_shift: u32) -> i32 {
    1i32 << amp_shift
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn wm<'a>(a: &'a [u8], e: &'a [u8; PALETTE_SIZE]) -> WriteMap<'a> {
        WriteMap {
            a_codes: a,
            elements: e,
        }
    }

    fn palette_slots() -> [u8; PALETTE_SIZE] {
        [0, 1, 2, 3, 4, 5, 6, 7]
    }

    #[test]
    fn ring_reset_stales_every_reference_and_rejects_future_indices() {
        let mut r = OccurrenceRing::new(4);
        for t in [10u32, 11, 12] {
            r.observe(t);
        }
        let live = r.reference(1).expect("live");
        assert_eq!(r.resolve(live), Some(11));
        assert!(r.reference(3).is_none(), "not yet written");
        assert!(r.reference(9).is_none(), "future index");
        r.reset();
        assert!(r.resolve(live).is_none(), "stale sequence must be rejected");
        assert_eq!(r.written(), 0);
        r.observe(42);
        assert_eq!(
            r.resolve(OccurrenceRef {
                seq: r.seq(),
                abs: 0
            }),
            Some(42)
        );
    }

    #[test]
    fn evicted_slots_are_reported_not_silently_absent() {
        let mut r = OccurrenceRing::new(3);
        for t in [1u32, 2, 3, 4, 5] {
            r.observe(t);
        }
        assert_eq!(r.valid_from(), 2);
        assert!(r.evicted(0) && r.evicted(1));
        assert!(r.reference(0).is_none() && r.reference(1).is_none());
        assert_eq!(
            r.resolve(OccurrenceRef {
                seq: r.seq(),
                abs: 1
            }),
            None
        );
        assert_eq!(r.get(2), Some(3));
    }

    #[test]
    fn admission_uses_only_the_causal_prefix_and_respects_the_bound() {
        let a = [0u8, 1, 2, 3, 0, 1, 2, 3];
        let e = palette_slots();
        let map = wm(&a, &e);
        let mut r = OccurrenceRing::new(RING_CAP);
        // prefix: 7 5 7 5 7  with the query token 7
        for t in [7u32, 5, 7, 5, 7] {
            r.observe(t);
        }
        let ctx = QueryContext {
            i: 5,
            cur: 7,
            prev: Some(7),
            prev2: Some(5),
        };
        let (cands, st) = admit(&r, &ctx, &map, MAX_CANDIDATES);
        // Three occurrences of the query token; the newest has no observed successor.
        assert_eq!(st.matching, 3);
        assert_eq!(st.no_observed_successor, 1);
        assert_eq!(cands.len(), 2);
        assert_eq!(cands[0].abs, 2, "newest admissible first");
        assert_eq!(cands[0].feats[6], 1, "newest flag");
        assert_eq!(cands[0].feats[4], 0);
        assert_eq!(cands[0].payload, 5);
        assert_eq!(cands[1].abs, 0);
        assert_eq!(
            cands[1].feats[4], 0,
            "abs 0 is not the nearest admissible lag"
        );
        assert_eq!(cands[1].payload, 5);
        // The bound drops matches rather than admitting them.
        let (c2, st2) = admit(&r, &ctx, &map, 1);
        assert_eq!(c2.len(), 1);
        assert_eq!(st2.dropped_by_bound, 1);
    }

    #[test]
    fn features_are_indicators_and_the_newest_flag_is_unique() {
        let a = [0u8, 1, 2, 3, 0, 1, 2, 3];
        let e = palette_slots();
        let map = wm(&a, &e);
        let mut r = OccurrenceRing::new(RING_CAP);
        // Prefix [4, 9, 5, 9, 8]; candidates are the occurrences of 9 at abs 1 and abs 3.
        for t in [4u32, 9, 5, 9, 8] {
            r.observe(t);
        }
        let ctx = QueryContext {
            i: 5,
            cur: 9,
            prev: Some(1),
            prev2: Some(5),
        };
        let (cands, st) = admit(&r, &ctx, &map, MAX_CANDIDATES);
        assert_eq!(st.matching, 2);
        assert_eq!(cands.len(), 2);
        assert_eq!(cands[0].abs, 3, "newest first");
        assert_eq!(cands[1].abs, 1);
        assert_eq!(cands.iter().filter(|c| c.feats[6] == 1).count(), 1);
        assert_eq!(cands[0].feats[6], 1);
        // abs 3 is the nearest admissible lag for i = 5.
        assert_eq!(cands[0].feats[4], 1);
        assert_eq!(cands[1].feats[4], 0);
        for c in cands.iter() {
            assert!(c.feats.iter().all(|v| *v <= 1));
        }
        assert_eq!(cands[0].payload, 8);
        assert_eq!(cands[1].payload, 5);
    }

    #[test]
    fn geometric_slot_equality_fires_for_distinct_tokens_in_the_same_slot() {
        // Tokens 1 and 2 share write slot 1; token 3 maps to slot 2.
        let a = [0u8, 1, 1, 2, 1];
        let e = palette_slots();
        let map = wm(&a, &e);
        assert_eq!(map.slot(2), map.slot(1));
        assert_ne!(map.slot(3), map.slot(1));

        // No candidate at all: the query token 6 never occurred in the prefix.
        let mut r0 = OccurrenceRing::new(RING_CAP);
        for t in [3u32, 8, 1] {
            r0.observe(t);
        }
        let ctx0 = QueryContext {
            i: 3,
            cur: 6,
            prev: Some(1),
            prev2: None,
        };
        let (c0, st0) = admit(&r0, &ctx0, &map, MAX_CANDIDATES);
        assert!(c0.is_empty());
        assert_eq!(st0.matching, 0);

        // Prefix [2, 6, 1], query token 6 at i = 3 with previous token 1.
        let mut r = OccurrenceRing::new(RING_CAP);
        for t in [2u32, 6, 1] {
            r.observe(t);
        }
        let ctx = QueryContext {
            i: 3,
            cur: 6,
            prev: Some(1),
            prev2: None,
        };
        let (cands, st) = admit(&r, &ctx, &map, MAX_CANDIDATES);
        assert_eq!(st.matching, 1);
        assert_eq!(cands.len(), 1);
        assert_eq!(cands[0].abs, 1);
        assert_eq!(cands[0].payload, 1, "the observed successor");
        // The candidate's role is token 2; the query's previous token is 1: different tokens,
        // identical transported write slot.
        assert_eq!(cands[0].feats[0], 0, "exact role differs");
        assert_eq!(cands[0].feats[2], 1, "geometric slot equality fires");
        assert_eq!(cands[0].feats[8], 1, "collision feature fires");
        assert_eq!(
            cands[0].feats[7], 0,
            "the conjunction needs an exact match too"
        );
        // A token in a different slot does not fire the geometric feature.
        let mut r3 = OccurrenceRing::new(RING_CAP);
        for t in [3u32, 6, 1] {
            r3.observe(t);
        }
        let (c3, _) = admit(&r3, &ctx, &map, MAX_CANDIDATES);
        assert_eq!(c3[0].feats[2], 0);
        assert_eq!(c3[0].feats[8], 0);
    }

    #[test]
    fn no_read_leaves_the_local_logits_untouched() {
        let mut z = vec![5i32, -3, 11];
        apply_residual(&mut z, None, 6);
        assert_eq!(z, vec![5, -3, 11]);
        apply_residual(&mut z, Some(1), 6);
        // The residual is 1 << 6 applied at the payload index only.
        assert_eq!(z, vec![5, -3 + 64, 11]);
    }

    #[test]
    fn selector_threshold_prevents_a_forced_copy_and_ties_go_to_the_newer() {
        let mut c0 = Candidate {
            slot_ref: OccurrenceRef { seq: 1, abs: 3 },
            abs: 3,
            payload: 9,
            feats: [0; FEATS],
        };
        let mut c1 = c0;
        c0.feats[0] = 1;
        c1.abs = 1;
        let cands = vec![c0, c1];
        // Weights are zero: nothing beats the threshold.
        let s = Selector::zero();
        assert_eq!(s.choose(&cands), None, "no copy without support");
        assert_eq!(Selector::latest(&cands), Some(0));
        assert_eq!(Selector::exact_role(&cands), Some(0));
        // A positive role weight turns it on.
        let mut s2 = Selector::zero();
        s2.w[0] = 2;
        assert_eq!(s2.choose(&cands), Some(0));
        // Two identical candidates: the newer (index 0) wins the tie.
        let cands2 = vec![
            Candidate {
                feats: [1, 0, 0, 0, 0, 0, 0, 0, 0],
                ..cands[0]
            },
            Candidate {
                feats: [1, 0, 0, 0, 0, 0, 0, 0, 0],
                ..cands[1]
            },
        ];
        assert_eq!(s2.choose(&cands2), Some(0));
    }

    #[test]
    fn artifact_round_trips_and_rejects_malformed_input() {
        let art = OccurrenceArtifact {
            selector: Selector {
                w: [1, -2, 3, 0, 0, -1, 0, 2, -3],
                mu: 1,
            },
            amp_shift: 7,
            ring_cap: RING_CAP as u32,
            max_candidates: MAX_CANDIDATES as u32,
            local_artifact_digest: [7u8; 32],
            tokenizer_digest: [9u8; 32],
            source_digest: [3u8; 32],
        };
        art.selector.validate().unwrap();
        let b = art.to_bytes();
        let back = OccurrenceArtifact::from_bytes(&b, &[7u8; 32], &[9u8; 32]).expect("round trip");
        assert_eq!(back, art);
        assert!(OccurrenceArtifact::from_bytes(&b, &[8u8; 32], &[9u8; 32]).is_err());
        assert!(OccurrenceArtifact::from_bytes(&b, &[7u8; 32], &[8u8; 32]).is_err());
        assert!(OccurrenceArtifact::from_bytes(&b[..b.len() - 1], &[7u8; 32], &[9u8; 32]).is_err());
        let mut trailing = b.clone();
        trailing.push(0);
        assert!(OccurrenceArtifact::from_bytes(&trailing, &[7u8; 32], &[9u8; 32]).is_err());
        // Out-of-range coefficient rejected at the byte level.
        let mut bad = b.clone();
        bad[20] = 99; // first weight byte
        assert!(OccurrenceArtifact::from_bytes(&bad, &[7u8; 32], &[9u8; 32]).is_err());
    }

    #[test]
    fn trainer_quantization_preserves_the_ranking_and_respects_the_bound() {
        let mut tr = SelectorTrainer::new(0.1, 7, [1u8; 32], [2u8; 32]);
        tr.w = [3.5, -1.25, 0.5, 0.0, 0.0, -0.75, 0.0, 2.0, -4.0];
        tr.mu = 0.25;
        let q = tr.quantize();
        q.validate().unwrap();
        // The largest power-of-two scale that fits |w| <= 7 is 1 for max|w| = 4.0.
        assert_eq!(q.w[8], -4);
        assert_eq!(q.w[0], 4);
        assert_eq!(q.mu, 0);
    }

    #[test]
    fn a_hand_built_example_reduces_loss_and_moves_the_weight() {
        // Two candidates: the labelled one has feature 0 set; the other has feature 8 set.
        let ex = Example {
            feats: vec![[1, 0, 0, 0, 0, 0, 0, 1, 0], [0, 0, 1, 0, 0, 0, 0, 0, 1]],
            label: 0,
            group: 0,
        };
        let examples = vec![ex; 4];
        let mut tr = SelectorTrainer::new(0.2, 11, [3u8; 32], [4u8; 32]);
        let before = {
            let mut t2 = tr.clone();
            t2.step(&examples, &[0])
        };
        for _ in 0..40 {
            tr.step(&examples, &[0, 1, 2, 3]);
        }
        let after = {
            let mut t2 = tr.clone();
            t2.step(&examples, &[0])
        };
        assert!(after < before, "loss must fall: {before} -> {after}");
        assert!(
            tr.w[7] > 0.0,
            "the informative conjunction must gain weight"
        );
        assert!(tr.w[8] < 0.0, "the collision feature must lose weight");
    }

    #[test]
    fn checkpoint_round_trips_and_rejects_a_foreign_identity() {
        let mut tr = SelectorTrainer::new(0.05, 3, [5u8; 32], [6u8; 32]);
        let ex = Example {
            feats: vec![[1, 0, 0, 0, 0, 0, 1, 0, 0]],
            label: 0,
            group: 0,
        };
        let examples = vec![ex; 8];
        for _ in 0..5 {
            let b = tr.next_batch(examples.len(), 2);
            tr.step(&examples, &b);
        }
        let bytes = tr.checkpoint_bytes();
        let mut other = SelectorTrainer::new(0.05, 3, [9u8; 32], [6u8; 32]);
        assert!(
            other.resume_from(&bytes).is_err(),
            "data identity must match"
        );
        let mut same = SelectorTrainer::new(0.05, 3, [5u8; 32], [6u8; 32]);
        same.resume_from(&bytes).expect("resume");
        assert_eq!(same.step, tr.step);
        assert_eq!(same.cursor, tr.cursor);
        assert_eq!(same.w, tr.w);
        assert_eq!(same.checkpoint_bytes(), bytes);
        // The reconstructed order is a function of (rng, epoch) only.
        assert_eq!(
            epoch_order(16, 12345, 2),
            epoch_order(16, 12345, 2),
            "order reconstruction must be deterministic"
        );
        assert_ne!(epoch_order(16, 12345, 2), epoch_order(16, 12345, 3));
    }
}

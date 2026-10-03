//! Prototype: delta-rule associative memory vs a fixed-size gated-linear state
//! on masked associative recall with a same-key-vocabulary distractor.
//!
//! Self-contained (std only, no unsafe, no external crate) so it can be built
//! with a single `rustc` invocation against the owner checkout's Rust toolchain
//! without touching the workspace or its shared build cache.
//!
//! Build:
//!   rustc -O -C target-cpu=native -o /tmp/drm prototypes/delta_rule_memory.rs
//! Run:
//!   OMP_NUM_THREADS=2 /tmp/drm --arm delta --n 32 --steps 3000 --seed 1
//!
//! Task ("MQAR + distractor"): a sequence of `n` (key, value) pairs, then one
//! extra pair whose key repeats, then the query key of one of the *first* `n`
//! pairs. The model must emit that pair's value; emitting the repeated key's
//! value is the measured failure mode ("binds the relation, emits the
//! distractor's value"). Supervision is sparse: one cross-entropy term on the
//! final value token, exactly the regime of the reported distance-curriculum
//! result.

#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::io::Write;

// ---------------------------------------------------------------- rng

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }
    #[inline]
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    #[inline]
    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % (n as u64)) as usize
    }
    #[inline]
    fn f32(&mut self) -> f32 {
        ((self.next_u64() >> 40) as f32) / ((1u64 << 24) as f32)
    }
    /// Standard normal via Box-Muller (training-time only; never served).
    fn normal(&mut self) -> f32 {
        let u1 = self.f32().max(1e-7);
        let u2 = self.f32();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos()
    }
}

// ---------------------------------------------------------------- config

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    /// Gated linear attention: a state *vector* of `d` scalars, the analogue of
    /// the existing 288-scalar-per-layer quaternion recurrence.
    Gla,
    /// Delta rule: a state *matrix* `d x d`, erase-then-write.
    Delta,
}

const N_KEYS: usize = 48;
const N_VALUES: usize = 48;
const KEY0: usize = 0;
const VAL0: usize = N_KEYS;
const VOCAB: usize = N_KEYS + N_VALUES;

#[derive(Clone, Copy)]
struct Cfg {
    d: usize,
    arm: Arm,
    n_pairs: usize,
    batch: usize,
    steps: usize,
    lr: f32,
    curriculum: bool,
    eval_every: usize,
    eval_batches: usize,
    init_curriculum: usize,
}

// ---------------------------------------------------------------- params

struct Params {
    emb: Vec<f32>,    // VOCAB x d
    g_gate: Vec<f32>, // d x d
    g_bias: Vec<f32>, // d
    g_beta: Vec<f32>, // d   (per-lane write gate pre-activation)
    w_q: Vec<f32>,    // d x d   (query projection, key vocab side)
    w_k: Vec<f32>,    // d x d   (key projection)
    w_v: Vec<f32>,    // d x d   (value projection)
    beta_raw: f32,
    q_scale: f32,
    unemb: Vec<f32>, // N_VALUES x d  (value-token classifier only)
}

impl Params {
    fn new(cfg: &Cfg, rng: &mut Rng) -> Self {
        let d = cfg.d;
        let scale = (1.0 / d as f32).sqrt();
        let mut mat = |rows: usize, cols: usize, rng: &mut Rng| -> Vec<f32> {
            (0..rows * cols).map(|_| rng.normal() * scale).collect()
        };
        Params {
            emb: (0..VOCAB * d).map(|_| rng.normal() * 0.6).collect(),
            g_gate: mat(d, d, rng),
            g_bias: vec![0.0; d],
            g_beta: vec![0.0; d],
            w_q: mat(d, d, rng),
            w_k: mat(d, d, rng),
            w_v: mat(d, d, rng),
            beta_raw: 0.0,
            q_scale: 1.0,
            unemb: mat(N_VALUES, d, rng),
        }
    }

    fn flat(&self) -> Vec<f32> {
        let mut v = Vec::new();
        v.extend_from_slice(&self.emb);
        v.extend_from_slice(&self.g_gate);
        v.extend_from_slice(&self.g_bias);
        v.extend_from_slice(&self.g_beta);
        v.extend_from_slice(&self.w_q);
        v.extend_from_slice(&self.w_k);
        v.extend_from_slice(&self.w_v);
        v.push(self.beta_raw);
        v.push(self.q_scale);
        v.extend_from_slice(&self.unemb);
        v
    }
    fn unflat(&mut self, v: &[f32]) {
        let d = self.g_bias.len();
        let mut o = 0;
        macro_rules! take {
            ($dst:expr, $n:expr) => {{
                let n = $n;
                $dst.copy_from_slice(&v[o..o + n]);
                o += n;
            }};
        }
        take!(self.emb, VOCAB * d);
        take!(self.g_gate, d * d);
        take!(self.g_bias, d);
        take!(self.g_beta, d);
        take!(self.w_q, d * d);
        take!(self.w_k, d * d);
        take!(self.w_v, d * d);
        self.beta_raw = v[o];
        o += 1;
        self.q_scale = v[o];
        o += 1;
        take!(self.unemb, N_VALUES * d);
    }
}

struct Grads {
    emb: Vec<f32>,
    g_gate: Vec<f32>,
    g_bias: Vec<f32>,
    g_beta: Vec<f32>,
    w_q: Vec<f32>,
    w_k: Vec<f32>,
    w_v: Vec<f32>,
    beta_raw: f32,
    q_scale: f32,
    unemb: Vec<f32>,
}

impl Grads {
    fn zeros(p: &Params) -> Self {
        Grads {
            emb: vec![0.0; p.emb.len()],
            g_gate: vec![0.0; p.g_gate.len()],
            g_bias: vec![0.0; p.g_bias.len()],
            g_beta: vec![0.0; p.g_beta.len()],
            w_q: vec![0.0; p.w_q.len()],
            w_k: vec![0.0; p.w_k.len()],
            w_v: vec![0.0; p.w_v.len()],
            beta_raw: 0.0,
            q_scale: 0.0,
            unemb: vec![0.0; p.unemb.len()],
        }
    }
    fn flat(&self) -> Vec<f32> {
        let mut v = Vec::new();
        v.extend_from_slice(&self.emb);
        v.extend_from_slice(&self.g_gate);
        v.extend_from_slice(&self.g_bias);
        v.extend_from_slice(&self.g_beta);
        v.extend_from_slice(&self.w_q);
        v.extend_from_slice(&self.w_k);
        v.extend_from_slice(&self.w_v);
        v.push(self.beta_raw);
        v.push(self.q_scale);
        v.extend_from_slice(&self.unemb);
        v
    }
}

// ---------------------------------------------------------------- task

struct Batch {
    tokens: Vec<Vec<usize>>, // batch x len, last token is the query
    target: Vec<usize>,      // batch, the value token
    distractor: Vec<usize>,  // batch, the repeated key's value token
    cursor: Vec<usize>,      // batch, index in tokens of the query token
}

/// `n` distinct keys with values, one extra pair whose key repeats an earlier
/// key with a different value, then one of the first `n` keys as the query.
fn sample(cfg: &Cfg, rng: &mut Rng) -> Batch {
    let n = cfg.n_pairs;
    // n pairs, then one extra (repeated-key) pair, then the query key.
    let len = 2 * n + 3;
    let mut tokens = Vec::with_capacity(cfg.batch);
    let mut target = Vec::with_capacity(cfg.batch);
    let mut distractor = Vec::with_capacity(cfg.batch);
    let mut cursor = Vec::with_capacity(cfg.batch);
    for _ in 0..cfg.batch {
        let mut used = HashSet::new();
        let mut keys = Vec::with_capacity(n);
        let mut vals = Vec::with_capacity(n);
        while keys.len() < n {
            let k = KEY0 + rng.below(N_KEYS);
            if used.insert(k) {
                keys.push(k);
                vals.push(VAL0 + rng.below(N_VALUES));
            }
        }
        // The queried pair must have n >= 2 so that at least one *other* pair
        // exists to carry the conflicting value.
        assert!(n >= 2, "the task needs n >= 2 pairs");
        assert!(
            n <= N_KEYS,
            "the task needs n <= N_KEYS ({N_KEYS}) distinct keys, got {n}"
        );
        let t = rng.below(n); // which pair is queried
        // The extra pair repeats an earlier key with a value that differs from
        // the queried pair's value: emitting it is the measured failure mode.
        let d_idx = (t + 1 + rng.below(n - 1)) % n;
        let d_key = keys[d_idx];
        let mut d_val = VAL0 + rng.below(N_VALUES);
        while d_val == vals[t] {
            d_val = VAL0 + rng.below(N_VALUES);
        }
        let mut row = Vec::with_capacity(len);
        for i in 0..n {
            row.push(keys[i]);
            row.push(vals[i]);
        }
        row.push(d_key);
        row.push(d_val);
        row.push(keys[t]);
        debug_assert_eq!(row.len(), len);
        tokens.push(row);
        target.push(vals[t]);
        distractor.push(d_val);
        cursor.push(len - 1);
    }
    Batch { tokens, target, distractor, cursor }
}

/// Zero-parameter oracle: nearest previous key token by embedding cosine, copy
/// its value. This is the "external exact store" upper bound on the same task.
fn exact_store_hits(cfg: &Cfg, p: &Params, batch: &Batch) -> (usize, usize) {
    let d = cfg.d;
    let mut hits = 0;
    let mut distractor_hits = 0;
    for b in 0..batch.tokens.len() {
        let row = &batch.tokens[b];
        let q = &p.emb[row[row.len() - 1] * d..row[row.len() - 1] * d + d];
        let mut best = f32::NEG_INFINITY;
        let mut best_val = 0usize;
        let mut i = 0;
        while i + 1 < row.len() - 1 {
            let k = &p.emb[row[i] * d..row[i] * d + d];
            let mut dot = 0.0;
            for j in 0..d {
                dot += q[j] * k[j];
            }
            if dot > best {
                best = dot;
                best_val = row[i + 1];
            }
            i += 2;
        }
        if best_val == batch.target[b] {
            hits += 1;
        }
        if best_val == batch.distractor[b] {
            distractor_hits += 1;
        }
    }
    (hits, distractor_hits)
}

// ---------------------------------------------------------------- model

struct Forward {
    logits: Vec<f32>,       // batch x N_VALUES
    probs: Vec<f32>,        // batch x N_VALUES
    loss: f32,
    /// Per-position saved state for backward.
    per_pos: Vec<Pos>,      // batch x len
}

#[derive(Clone)]
struct Pos {
    z: Vec<f32>,    // mixer output d
    a: f32,         // softmax prob of the target value token
    /// d, GLA only: the read/write gate at this position.
    gate: Vec<f32>,
    kvec: Vec<f32>, // d, delta only (unit key)
    knorm: f32,     // delta only
    state: Vec<f32>, // saved state *before* this position's update
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

fn forward(cfg: &Cfg, p: &Params, batch: &Batch) -> Forward {
    let d = cfg.d;
    let b = batch.tokens.len();
    let len = batch.tokens[0].len();
    debug_assert!(batch.tokens.iter().all(|r| r.len() == len));
    let mut per_pos = Vec::with_capacity(b * len);
    let mut logits = vec![0.0f32; b * N_VALUES];
    let mut probs = vec![0.0f32; b * N_VALUES];
    let mut loss = 0.0f32;

    let beta = sigmoid(p.beta_raw);
    let state_dim = match cfg.arm {
        Arm::Gla => d,
        Arm::Delta => d * d,
    };
    let state_len = state_dim;

    for bi in 0..b {
        let row = &batch.tokens[bi];
        let mut state = vec![0.0f32; state_len];
        let mut prev: Option<usize> = None;
        for (t, &tok) in row.iter().enumerate() {
            // x = embedding of the *previous* token: the mixer sees the stream
            // so far and writes the pair that just completed.
            let x: Vec<f32> = match prev {
                Some(pt) => p.emb[pt * d..pt * d + d].to_vec(),
                None => vec![0.0; d],
            };
            let state_before = state.clone();
            let mut z = vec![0.0f32; d];
            let mut gate = vec![0.0f32; d];
            let mut kvec: Vec<f32> = Vec::new();
            let mut knorm = 0.0f32;

            match cfg.arm {
                Arm::Gla => {
                    // g = sigmoid(W_g x + b), beta = sigmoid(w_b . x)
                    for i in 0..d {
                        let mut acc = p.g_bias[i];
                        let r = &p.g_gate[i * d..i * d + d];
                        for j in 0..d {
                            acc += r[j] * x[j];
                        }
                        gate[i] = sigmoid(acc);
                    }
                    let mut bwrite = 0.0f32;
                    for i in 0..d {
                        bwrite += p.g_beta[i] * x[i];
                    }
                    let bwrite = sigmoid(bwrite);
                    // v = W_v x
                    let mut v = vec![0.0f32; d];
                    for i in 0..d {
                        let r = &p.w_v[i * d..i * d + d];
                        let mut acc = 0.0;
                        for j in 0..d {
                            acc += r[j] * x[j];
                        }
                        v[i] = acc;
                    }
                    // read with k = W_q x, then write
                    let mut k = vec![0.0f32; d];
                    for i in 0..d {
                        let r = &p.w_q[i * d..i * d + d];
                        let mut acc = 0.0;
                        for j in 0..d {
                            acc += r[j] * x[j];
                        }
                        k[i] = acc;
                    }
                    for i in 0..d {
                        z[i] = p.q_scale * k[i] * state[i];
                    }
                    for i in 0..d {
                        state[i] = gate[i] * state[i] + bwrite * v[i];
                    }
                    per_pos.push(Pos { z, a: 0.0, gate, kvec, knorm, state: state_before });
                }
                Arm::Delta => {
                    // k = normalize(W_k x), q = W_q x, v = W_v x
                    let mut k = vec![0.0f32; d];
                    let mut q = vec![0.0f32; d];
                    let mut v = vec![0.0f32; d];
                    for i in 0..d {
                        let rk = &p.w_k[i * d..i * d + d];
                        let rq = &p.w_q[i * d..i * d + d];
                        let rv = &p.w_v[i * d..i * d + d];
                        let (mut ak, mut aq, mut av) = (0.0f32, 0.0f32, 0.0f32);
                        for j in 0..d {
                            ak += rk[j] * x[j];
                            aq += rq[j] * x[j];
                            av += rv[j] * x[j];
                        }
                        k[i] = ak;
                        q[i] = aq;
                        v[i] = av;
                    }
                    let mut nrm = 0.0f32;
                    for i in 0..d {
                        nrm += k[i] * k[i];
                    }
                    nrm = nrm.sqrt().max(1e-6);
                    let k_raw = k.clone();
                    for i in 0..d {
                        k[i] /= nrm;
                    }
                    // z = q_scale * S^T q  with S row-major d x d, S[i][j]
                    for i in 0..d {
                        let si = &state[i * d..i * d + d];
                        let mut acc = 0.0f32;
                        for j in 0..d {
                            acc += si[j] * q[j];
                        }
                        z[i] = p.q_scale * acc;
                    }
                    // erase then write
                    let mut sk = vec![0.0f32; d];
                    for j in 0..d {
                        let mut acc = 0.0f32;
                        for i in 0..d {
                            acc += state[i * d + j] * k[i];
                        }
                        sk[j] = acc;
                    }
                    for i in 0..d {
                        let si = i * d;
                        for j in 0..d {
                            state[si + j] += beta * (v[i] - sk[j]) * k[j];
                        }
                    }
                    // Store the pre-normalization key: the backward applies the
                    // normalization adjoint itself.
                    kvec = k_raw;
                    knorm = nrm;
                    per_pos.push(Pos { z, a: 0.0, gate: Vec::new(), kvec, knorm, state: state_before });
                }
            }
            prev = Some(tok);
        }
        // Query-position readout: the last position's z mixed through W_v^T
        // (shared) is not needed; use the final state's read directly.
        let last = &per_pos[bi * len + len - 1];
        let zlast = &last.z;
        let mut mx = f32::NEG_INFINITY;
        for c in 0..N_VALUES {
            let r = &p.unemb[c * d..c * d + d];
            let mut acc = 0.0;
            for j in 0..d {
                acc += r[j] * zlast[j];
            }
            logits[bi * N_VALUES + c] = acc;
            if acc > mx {
                mx = acc;
            }
        }
        let mut sum = 0.0f32;
        for c in 0..N_VALUES {
            let e = (logits[bi * N_VALUES + c] - mx).exp();
            probs[bi * N_VALUES + c] = e;
            sum += e;
        }
        let tg = batch.target[bi] - VAL0;
        for c in 0..N_VALUES {
            probs[bi * N_VALUES + c] /= sum;
        }
        let a = probs[bi * N_VALUES + tg].max(1e-9);
        loss += -a.ln();
        per_pos[bi * len + len - 1].a = a;
    }
    Forward { logits, probs, loss: loss / b as f32, per_pos }
}

fn backward(cfg: &Cfg, p: &Params, batch: &Batch, fwd: &Forward, g: &mut Grads) -> f32 {
    let d = cfg.d;
    let b = batch.tokens.len();
    let len = batch.tokens[0].len();
    let inv_b = 1.0 / b as f32;
    let beta = sigmoid(p.beta_raw);
    let state_dim = match cfg.arm {
        Arm::Gla => d,
        Arm::Delta => d * d,
    };
    let mut total = 0.0f32;

    for bi in 0..b {
        let row = &batch.tokens[bi];
        let tg = batch.target[bi] - VAL0;
        let a = fwd.per_pos[bi * len + len - 1].a;
        total += -a.ln() * inv_b;

        // dL/dlogits at the query position, then dL/dz there.
        let mut dlogit = vec![0.0f32; N_VALUES];
        for c in 0..N_VALUES {
            let pr = fwd.probs[bi * N_VALUES + c];
            let y = if c == tg { 1.0 } else { 0.0 };
            dlogit[c] = (pr - y) * inv_b;
        }
        let zq = &fwd.per_pos[bi * len + len - 1].z;
        let mut dz = vec![0.0f32; d];
        for c in 0..N_VALUES {
            let dl = dlogit[c];
            if dl == 0.0 {
                continue;
            }
            let r = &p.unemb[c * d..c * d + d];
            for j in 0..d {
                dz[j] += dl * r[j];
            }
            for j in 0..d {
                g.unemb[c * d + j] += dl * zq[j];
            }
        }

        // ---- Phase A: reverse scan. `ds[t]` = dL/dS_t, the gradient of the state
        // *after* the update at position t. Phase B at position t therefore
        // consumes ds[t-1] (the state entering t); ds[-1] means 0. ----
        let mut ds: Vec<Vec<f32>> = vec![vec![0.0f32; state_dim]; len + 1];
        match cfg.arm {
            Arm::Delta => {
                // The query reads the state entering the final position. Its
                // adjoint deposits q_scale * outer(dz, q) there.
                let mut ds_last = vec![0.0f32; d * d];
                for i in 0..d {
                    if dz[i] == 0.0 {
                        continue;
                    }
                    let coef = p.q_scale * dz[i];
                    for j in 0..d {
                        ds_last[i * d + j] += coef * x_of(p, row, len - 1, d)[j];
                    }
                }
                ds[len - 1] = ds_last;
            }
            Arm::Gla => {
                // z_i = q_scale * k_i * h_i, so dL/dh_i = dz_i * q_scale * k_i.
                let k = fwd_proj(&p.w_q, &x_of(p, row, len - 1, d), d);
                for i in 0..d {
                    ds[len - 1][i] = dz[i] * p.q_scale * k[i];
                }
            }
        }
        // Propagate ds[t+1] back through the update performed at t.
        for t in (1..len).rev() {
            let carried = ds[t].clone();
            let mut into = vec![0.0f32; state_dim];
            match cfg.arm {
                Arm::Gla => {
                    // h_t = gate_i h_{t-1,i} + bwrite v_i
                    let pos = &fwd.per_pos[bi * len + t];
                    for i in 0..d {
                        into[i] = carried[i] * pos.gate[i];
                    }
                }
                Arm::Delta => {
                    // S_t = S_{t-1}(I - beta k k^T) + beta v k^T: column j of the
                    // carried gradient is scaled by (1 - beta k_j^2).
                    let pos = &fwd.per_pos[bi * len + t];
                    let k: Vec<f32> = pos.kvec.iter().map(|v| v / pos.knorm).collect();
                    into.copy_from_slice(&carried);
                    for j in 0..d {
                        let mut col = 0.0f32;
                        for l in 0..d {
                            col += carried[l * d + j];
                        }
                        let coef = -beta * col * k[j];
                        for i in 0..d {
                            into[i * d + j] += coef;
                        }
                    }
                }
            }
            ds[t - 1] = into;
        }

        // ---- Phase B: forward sweep with ds[t+1] available. ----
        for t in 0..len {
            let pos = &fwd.per_pos[bi * len + t];
            let x = x_of(p, row, t, d);
            let zero_dz = vec![0.0f32; d];
            let dz_t: &[f32] = if t == len - 1 { &dz } else { &zero_dz };
            let incoming: Vec<f32> = if t == 0 { vec![0.0f32; state_dim] } else { ds[t - 1].clone() };
            let mut dx = vec![0.0f32; d];

            match cfg.arm {
                Arm::Gla => {
                    let h = &pos.state;
                    let k = fwd_proj(&p.w_q, &x, d);
                    let mut dstate_t = incoming.clone();
                    for i in 0..d {
                        dstate_t[i] += dz_t[i] * p.q_scale * k[i];
                        let dk = dz_t[i] * p.q_scale * h[i];
                        let r = &p.w_q[i * d..i * d + d];
                        for j in 0..d {
                            g.w_q[i * d + j] += dk * x[j];
                            dx[j] += dk * r[j];
                        }
                        g.q_scale += dz_t[i] * k[i] * h[i];
                    }
                    let mut bpre = 0.0f32;
                    for i in 0..d {
                        bpre += p.g_beta[i] * x[i];
                    }
                    let bwrite = sigmoid(bpre);
                    let v = fwd_proj(&p.w_v, &x, d);
                    let mut d_bwrite = 0.0f32;
                    for i in 0..d {
                        let dh = dstate_t[i];
                        d_bwrite += dh * v[i];
                        let dpre = dh * h[i] * pos.gate[i] * (1.0 - pos.gate[i]);
                        let r = &p.g_gate[i * d..i * d + d];
                        for j in 0..d {
                            g.g_gate[i * d + j] += dpre * x[j];
                            dx[j] += dpre * r[j];
                        }
                        g.g_bias[i] += dpre;
                        let dv = d_bwrite * bwrite;
                        let rv = &p.w_v[i * d..i * d + d];
                        for j in 0..d {
                            g.w_v[i * d + j] += dv * x[j];
                            dx[j] += dv * rv[j];
                        }
                    }
                    let dbpre = d_bwrite * bwrite * (1.0 - bwrite);
                    for i in 0..d {
                        g.g_beta[i] += dbpre * x[i];
                        dx[i] += dbpre * p.g_beta[i];
                    }
                }
                Arm::Delta => {
                    let s = &pos.state; // S_{t-1}
                    let nrm = pos.knorm;
                    let k: Vec<f32> = pos.kvec.iter().map(|v| v / nrm).collect();
                    let q = fwd_proj(&p.w_q, &x, d);
                    let v = fwd_proj(&p.w_v, &x, d);
                    // dL/dS_t, including this position's read if it is the query.
                    let mut dstate_t = incoming.clone();
                    for i in 0..d {
                        let dzt = dz_t[i];
                        if dzt == 0.0 {
                            continue;
                        }
                        let mut r = 0.0f32;
                        for j in 0..d {
                            r += s[i * d + j] * q[j];
                        }
                        g.q_scale += dzt * r;
                        let coef = p.q_scale * dzt;
                        for j in 0..d {
                            dstate_t[i * d + j] += coef * q[j];
                        }
                    }
                    let mut dq = vec![0.0f32; d];
                    for j in 0..d {
                        let mut acc = 0.0f32;
                        for i in 0..d {
                            acc += s[i * d + j] * dz_t[i];
                        }
                        dq[j] = p.q_scale * acc;
                    }
                    for i in 0..d {
                        let rq = &p.w_q[i * d..i * d + d];
                        for j in 0..d {
                            g.w_q[i * d + j] += dq[i] * x[j];
                            dx[j] += dq[i] * rq[j];
                        }
                    }
                    let mut sk = vec![0.0f32; d];
                    for j in 0..d {
                        let mut acc = 0.0f32;
                        for i in 0..d {
                            acc += s[i * d + j] * k[i];
                        }
                        sk[j] = acc;
                    }
                    let delta: Vec<f32> = (0..d).map(|j| v[j] - sk[j]).collect();
                    // The update writes delta_j * k_j into column j, so the
                    // adjoints contract *columns* of dL/dS_t.
                    let mut gk = vec![0.0f32; d];
                    let mut gdel = vec![0.0f32; d];
                    for j in 0..d {
                        let mut col = 0.0f32;
                        for l in 0..d {
                            col += dstate_t[l * d + j];
                        }
                        gk[j] = col * k[j];
                        gdel[j] = col * delta[j];
                    }
                    let mut dbeta = 0.0f32;
                    for j in 0..d {
                        dbeta += gdel[j] * k[j];
                    }
                    g.beta_raw += dbeta * beta * (1.0 - beta);
                    let dv: Vec<f32> = (0..d).map(|j| beta * gk[j]).collect();
                    let mut dk = vec![0.0f32; d];
                    for j in 0..d {
                        let mut acc = 0.0f32;
                        for l in 0..d {
                            acc += gdel[l] * s[l * d + j];
                        }
                        dk[j] = beta * (gk[j] - acc);
                    }
                    for i in 0..d {
                        let rv = &p.w_v[i * d..i * d + d];
                        for j in 0..d {
                            g.w_v[i * d + j] += dv[i] * x[j];
                            dx[j] += dv[i] * rv[j];
                        }
                    }
                    let mut dot = 0.0f32;
                    for i in 0..d {
                        dot += dk[i] * k[i];
                    }
                    for i in 0..d {
                        let draw = (dk[i] - k[i] * dot) / nrm;
                        let rk = &p.w_k[i * d..i * d + d];
                        for j in 0..d {
                            g.w_k[i * d + j] += draw * x[j];
                            dx[j] += draw * rk[j];
                        }
                    }
                }
            }
            if t > 0 {
                for j in 0..d {
                    g.emb[row[t - 1] * d + j] += dx[j];
                }
            }
        }
    }
    total
}

/// The mixer input at position `t`: the embedding of the previous token.
fn x_of(p: &Params, row: &[usize], t: usize, d: usize) -> Vec<f32> {
    if t == 0 {
        vec![0.0; d]
    } else {
        p.emb[row[t - 1] * d..row[t - 1] * d + d].to_vec()
    }
}

/// W x.
fn fwd_proj(w: &[f32], x: &[f32], d: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; d];
    for i in 0..d {
        let r = &w[i * d..i * d + d];
        let mut acc = 0.0f32;
        for j in 0..d {
            acc += r[j] * x[j];
        }
        out[i] = acc;
    }
    out
}

// ---------------------------------------------------------------- adam

struct Adam {
    m: Vec<f32>,
    v: Vec<f32>,
    t: u32,
}

impl Adam {
    fn new(n: usize) -> Self {
        Adam { m: vec![0.0; n], v: vec![0.0; n], t: 0 }
    }
    fn step(&mut self, p: &mut [f32], g: &[f32], lr: f32) {
        self.t += 1;
        let b1 = 0.9f32;
        let b2 = 0.999f32;
        let c1 = 1.0 - b1.powi(self.t as i32);
        let c2 = 1.0 - b2.powi(self.t as i32);
        for i in 0..p.len() {
            self.m[i] = b1 * self.m[i] + (1.0 - b1) * g[i];
            self.v[i] = b2 * self.v[i] + (1.0 - b2) * g[i] * g[i];
            let mh = self.m[i] / c1;
            let vh = self.v[i] / c2;
            p[i] -= lr * mh / (vh.sqrt() + 1e-8);
        }
    }
}

// ---------------------------------------------------------------- evaluate

fn evaluate(cfg: &Cfg, p: &Params, rng: &mut Rng, n_pairs: usize) -> (f32, f32, f32) {
    let eval_cfg = Cfg { n_pairs, batch: cfg.batch, ..*cfg };
    let mut hits = 0usize;
    let mut distract = 0usize;
    let mut loss = 0.0f32;
    let mut total = 0usize;
    for _ in 0..cfg.eval_batches {
        let batch = sample(&eval_cfg, rng);
        let fwd = forward(&eval_cfg, p, &batch);
        loss += fwd.loss * batch.tokens.len() as f32;
        total += batch.tokens.len();
        for bi in 0..batch.tokens.len() {
            let mut best = 0usize;
            let mut bv = f32::NEG_INFINITY;
            for c in 0..N_VALUES {
                let l = fwd.logits[bi * N_VALUES + c];
                if l > bv {
                    bv = l;
                    best = c;
                }
            }
            let pred = best + VAL0;
            if pred == batch.target[bi] {
                hits += 1;
            }
            if pred == batch.distractor[bi] {
                distract += 1;
            }
        }
    }
    (
        loss / total as f32,
        hits as f32 / total as f32,
        distract as f32 / total as f32,
    )
}

// ---------------------------------------------------------------- main

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut get = |name: &str, default: &str| -> String {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1).cloned())
            .unwrap_or_else(|| default.to_string())
    };
    let arm = match get("--arm", "delta").as_str() {
        "gla" => Arm::Gla,
        "delta" => Arm::Delta,
        other => panic!("unknown arm {other}"),
    };
    if args.iter().any(|a| a == "--gradcheck") {
        grad_check(arm);
        return;
    }
    let cfg = Cfg {
        d: get("--d", "64").parse().unwrap(),
        arm,
        n_pairs: get("--n", "32").parse().unwrap(),
        batch: get("--batch", "32").parse().unwrap(),
        steps: get("--steps", "3000").parse().unwrap(),
        lr: get("--lr", "0.003").parse().unwrap(),
        curriculum: get("--curriculum", "0") == "1",
        eval_every: get("--eval-every", "250").parse().unwrap(),
        eval_batches: get("--eval-batches", "4").parse().unwrap(),
        init_curriculum: get("--curriculum-start", "4").parse().unwrap(),
    };
    let seed: u64 = get("--seed", "1").parse().unwrap();
    let mut rng = Rng::new(seed);
    let mut p = Params::new(&cfg, &mut rng);
    let nparams = p.flat().len();
    println!(
        "# arm={:?} d={} n={} batch={} steps={} lr={} curriculum={} seed={} params={}",
        cfg.arm, cfg.d, cfg.n_pairs, cfg.batch, cfg.steps, cfg.lr, cfg.curriculum, seed, nparams
    );

    let (oracle_hits, oracle_distract) = {
        let batch = sample(&cfg, &mut rng);
        let (h, dd) = exact_store_hits(&cfg, &p, &batch);
        (h as f32 / batch.tokens.len() as f32, dd as f32 / batch.tokens.len() as f32)
    };
    println!("# oracle(exact store, 0 learned params): hits={oracle_hits:.4} distractor={oracle_distract:.4}");

    let flush = || {
        let _ = std::io::stdout().flush();
    };
    let mut adam = Adam::new(nparams);
    let start = std::time::Instant::now();
    for step in 0..cfg.steps {
        let cur_n = if cfg.curriculum {
            let frac = (step as f32 / cfg.steps as f32 * 2.0).min(1.0);
            let lo = cfg.init_curriculum;
            (lo as f32 + (cfg.n_pairs as f32 - lo as f32) * frac).round() as usize
        } else {
            cfg.n_pairs
        };
        let mut train_cfg = cfg;
        train_cfg.n_pairs = cur_n;
        let batch = sample(&train_cfg, &mut rng);
        let fwd = forward(&train_cfg, &p, &batch);
        let mut g = Grads::zeros(&p);
        backward(&train_cfg, &p, &batch, &fwd, &mut g);
        let mut flat = p.flat();
        let gf = g.flat();
        adam.step(&mut flat, &gf, cfg.lr);
        p.unflat(&flat);
        if (step + 1) % cfg.eval_every == 0 || step == 0 {
            let (l, h, dd) = evaluate(&cfg, &p, &mut rng, cfg.n_pairs);
            println!(
                "step {:5} n={:3} loss {:.4} hits {:.4} distractor {:.4} elapsed {:.1}s",
                step + 1,
                cur_n,
                l,
                h,
                dd,
                start.elapsed().as_secs_f32()
            );
            flush();
        }
    }
    let (l, h, dd) = evaluate(&cfg, &p, &mut rng, cfg.n_pairs);
    println!("FINAL arm={:?} n={} loss={l:.4} hits={h:.4} distractor={dd:.4}", cfg.arm, cfg.n_pairs);
    flush();
}

// ---------------------------------------------------------------- grad check

/// Central finite differences against the analytic backward, on a tiny config.
/// Parameters are sampled across every block, and `eps` is scaled to the loss so
/// the check is not dominated by float roundoff.
fn grad_check(arm: Arm) {
    let tiny: usize = std::env::args()
        .position(|a| a == "--tiny")
        .map(|_| 1)
        .unwrap_or(0);
    let cfg = Cfg {
        d: if tiny == 1 { 4 } else { 8 },
        arm,
        n_pairs: if tiny == 1 { 2 } else { 3 },
        batch: if tiny == 1 { 1 } else { 2 },
        steps: 0,
        lr: 0.0,
        curriculum: false,
        eval_every: 0,
        eval_batches: 0,
        init_curriculum: 0,
    };
    let mut rng = Rng::new(7);
    let mut p = Params::new(&cfg, &mut rng);
    let batch = sample(&cfg, &mut rng);
    let fwd = forward(&cfg, &p, &batch);
    let mut g = Grads::zeros(&p);
    backward(&cfg, &p, &batch, &fwd, &mut g);
    let analytic = g.flat();
    let base = p.flat();
    let d = cfg.d;
    let eps = if tiny == 1 { 1e-3f32 } else { 3e-3f32 };

    // One representative index per block, plus spread within the big blocks.
    let mut idx: Vec<usize> = Vec::new();
    let mut o = 0usize;
    let mut push_block = |o: &mut usize, n: usize, k: usize, idx: &mut Vec<usize>| {
        for s in 0..k.min(n) {
            idx.push(*o + s * (n / k.max(1)).max(1));
        }
        *o += n;
    };
    let counts = [(VOCAB * d, 8), (d * d, 6), (d, 3), (d, 3), (d * d, 6), (d * d, 6), (d * d, 6)];
    for (n, k) in counts {
        push_block(&mut o, n, k, &mut idx);
    }
    idx.push(o); // beta_raw
    idx.push(o + 1); // q_scale
    push_block(&mut o, N_VALUES * d, 4, &mut idx);

    let mut worst = 0.0f32;
    let mut num_bad = 0usize;
    let mut checked = 0usize;
    if tiny == 1 {
        idx = (0..base.len()).step_by(16).collect();
    }
    for &i in &idx {
        if i >= base.len() {
            continue;
        }
        let mut hi = base.clone();
        hi[i] += eps;
        p.unflat(&hi);
        let lp = forward(&cfg, &p, &batch).loss;
        let mut lo = base.clone();
        lo[i] -= eps;
        p.unflat(&lo);
        let lm = forward(&cfg, &p, &batch).loss;
        p.unflat(&base);
        let num = (lp - lm) / (2.0 * eps);
        let an = analytic[i];
        let den = num.abs().max(an.abs()).max(1e-3);
        let rel = (num - an).abs() / den;
        checked += 1;
        if rel > worst {
            worst = rel;
        }
        if rel > 5e-2 {
            num_bad += 1;
            if num_bad <= 5 {
                println!("  MISMATCH i={i} analytic={an:.6e} numeric={num:.6e} rel={rel:.4}");
            }
        }
    }
    println!(
        "GRADCHECK arm={arm:?} checked={checked} params={} worst_rel={worst:.5} mismatches={num_bad}",
        base.len()
    );
}

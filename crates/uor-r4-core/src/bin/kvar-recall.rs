//! KVAR — keyed variable-lag retrieval with rebinding (synthetic harness).
//!
//! Executes the frozen pre-registration `docs/integration/kvar-plan-2026-09-24.md`
//! (M1 Part B′). It is a **self-contained** synthetic panel plus controls plus
//! three trainable arms sharing one model struct with two boolean flags:
//!
//! - `(a)` `GATE=false, STORE=false` — current recurrence, no gate.
//! - `(b)` `GATE=true,  STORE=false` — plus a hard binary gate whose input is the
//!   joint (current token × state) read `cg[x_t] + dot(ag[x_t], h)`.
//! - `(c)` `GATE=true,  STORE=true`  — plus a token-addressed overwrite store
//!   (`Z[k] ← onehot(v)` on write; hard table read at the query key).
//!
//! Controls, training-free: `C` tuned order-2 `(prev,cur)→next` counts (must be
//! near chance `1/64` on held-out queries) and `(g)` a hand-coded overwrite table
//! (must be near-perfect). Arms `(d)`, `(e)`, `(f)`, `(h)` are `NOT_RUN` at this
//! reduced scope and are recorded as such.
//!
//! # D0-b / served-kernel boundary
//!
//! Training is floating point (permitted). The **served** evaluation is integer/
//! select only: dense linear maps use ternary weights and add/subtract/zero;
//! embedding and bias lookups use i8 values with power-of-two shifts. Gates are
//! hard compares/selects, the store is a table write/read with explicit row
//! validity, and the output is `argmax`. The declared numerical kernel in
//! `qmat_t`, `qdot_t` and `serve_scores` has no multiplier or floating-point
//! arithmetic. Softmax/float is used only in training and offline measurement.
//!
//! # Report
//!
//! One sealed root: `claim` → write → `seal` → `verify`.

#![forbid(unsafe_code)]

use serde_json::{json, Value};
use std::path::PathBuf;
use uor_r4_core::report_output::{claim, seal, verify};

#[path = "kvar_relative_energy.rs"]
mod kvar_relative_energy;

/// Fixed panel vocabulary: `CONTENT` content tokens, then `BIND`, then `QUERY`.
const CONTENT: usize = 64;
const D: usize = 64;
const OUT: usize = 64;
const BIND: u8 = 64;
const QUERY: u8 = 65;
const VOCAB: usize = 66;
const NT: usize = 3;

/// Panel cells (frozen).
const KS: [usize; 2] = [4, 8];
const LAGS: [usize; 3] = [4, 16, 64];

/// Training / evaluation budget (frozen before the run).
const TRAIN_EPISODES_PER_CELL: usize = 100;
const HELD_EPISODES_PER_CELL: usize = 34;
const TRAIN_STEPS: usize = 1500;
const BATCH: usize = 32;
const LR: f32 = 3e-3;
const CLIP: f32 = 1.0;
const BOOTSTRAP: usize = 2000;
const ACT_CAP: f32 = 8.0;
const STORE_SHIFT: i32 = 8;

// RNG (xorshift64*) — deterministic, seed-disjoint panel.

#[derive(Clone)]
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed ^ 0x9E37_79B9_7F4A_7C15 | 1)
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        (self.next_u64() % n as u64) as usize
    }
    fn unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }
}

/// Standard-normal via Box–Muller, for small weight init.
fn gauss(rng: &mut Rng) -> f32 {
    let u1 = (rng.unit() + 1e-7).min(1.0);
    let u2 = rng.unit();
    (-2.0 * u1.ln()).sqrt() * (std::f32::consts::TAU * u2).cos()
}

// Panel generator

/// One episode. `inputs` are the tokens the model reads; `target` is the value
/// the model must produce at the query. `lag` is the frozen number of tokens
/// strictly between the query key's final binding value and the `QUERY` marker.
#[derive(Clone)]
struct Episode {
    inputs: Vec<u8>,
    target: u8,
    k: usize,
    lag: usize,
    key: u8,
    seed: u64,
}

impl Episode {
    #[cfg(test)]
    fn bindings(&self) -> Vec<(u8, u8, usize)> {
        let mut out = Vec::new();
        let n = self.inputs.len();
        let mut t = 0;
        while t + 2 < n {
            if self.inputs[t] == BIND {
                out.push((self.inputs[t + 1], self.inputs[t + 2], t + 2));
                t += 3;
            } else {
                t += 1;
            }
        }
        out
    }
}

fn fresh_content(rng: &mut Rng, avoid: u8) -> u8 {
    loop {
        let v = rng.below(CONTENT) as u8;
        if v != avoid {
            return v;
        }
    }
}

fn shuffle<T>(v: &mut [T], rng: &mut Rng) {
    for i in (1..v.len()).rev() {
        let j = rng.below(i + 1);
        v.swap(i, j);
    }
}

/// Generate one KVAR episode. Guarantees: `K` distinct keys, every key bound at
/// least twice with consecutive values different (so it is rebound at least once
/// to a different value); the query key's final binding is the last `kq` binding;
/// exactly `lag` tokens (none a `kq` binding) follow it before `QUERY`.
fn gen_episode(seed: u64, k: usize, lag: usize) -> Episode {
    let mut rng = Rng::new(seed);
    // K distinct keys.
    let mut keys: Vec<u8> = Vec::new();
    while keys.len() < k {
        let t = rng.below(CONTENT) as u8;
        if !keys.contains(&t) {
            keys.push(t);
        }
    }
    let kq = keys[rng.below(k)];
    let mut last_val = [255u8; CONTENT];
    let mut prefix: Vec<(u8, u8)> = Vec::new();
    for &key in keys.iter().filter(|&&x| x != kq) {
        for _ in 0..2 {
            let v = fresh_content(&mut rng, last_val[key as usize]);
            last_val[key as usize] = v;
            prefix.push((key, v));
        }
    }
    let nb = 2 + rng.below(2); // kq bound 2 or 3 times
    let mut kq_vals = Vec::with_capacity(nb);
    for _ in 0..nb {
        let v = fresh_content(&mut rng, last_val[kq as usize]);
        last_val[kq as usize] = v;
        kq_vals.push(v);
    }
    let final_v = *kq_vals.last().expect("nb >= 2");
    for &v in kq_vals.iter().take(nb - 1) {
        prefix.push((kq, v));
    }
    shuffle(&mut prefix, &mut rng);
    let mut tokens: Vec<u8> = Vec::new();
    for (key, val) in prefix {
        for _ in 0..rng.below(2) {
            tokens.push(rng.below(CONTENT) as u8);
        }
        tokens.push(BIND);
        tokens.push(key);
        tokens.push(val);
    }
    // kq's final binding, immediately before the controlled tail.
    tokens.push(BIND);
    tokens.push(kq);
    tokens.push(final_v);
    // Tail of exactly `lag` tokens, containing no kq binding.
    let mut tail = 0usize;
    while tail < lag {
        let rem = lag - tail;
        if rem >= 3 && rng.below(100) < 40 {
            let key = keys[rng.below(k)];
            if key == kq {
                tokens.push(rng.below(CONTENT) as u8);
                tail += 1;
                continue;
            }
            let v = fresh_content(&mut rng, last_val[key as usize]);
            last_val[key as usize] = v;
            tokens.push(BIND);
            tokens.push(key);
            tokens.push(v);
            tail += 3;
        } else {
            tokens.push(rng.below(CONTENT) as u8);
            tail += 1;
        }
    }
    tokens.push(QUERY);
    tokens.push(kq);
    Episode {
        inputs: tokens,
        target: final_v,
        k,
        lag,
        key: kq,
        seed,
    }
}

fn seed_for(split: u64, k: usize, lag: usize, i: usize) -> u64 {
    // Disjoint seeds per split / cell / episode; design and held-out never collide.
    split
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add((k as u64) << 40)
        .wrapping_add((lag as u64) << 24)
        .wrapping_add(i as u64)
        .wrapping_add(0x1234_5678_9ABC_DEF0)
}

// Control C — tuned order-2 (prev,cur)->next counts (must be near chance).

struct Count2 {
    counts: Vec<u32>,
    vocab: usize,
    out: usize,
    alpha: f32,
}

impl Count2 {
    fn new(vocab: usize, out: usize) -> Self {
        Count2 {
            counts: vec![0u32; vocab * vocab * out],
            vocab,
            out,
            alpha: 0.5,
        }
    }
    fn train(&mut self, eps: &[Episode]) {
        for ep in eps {
            let xs = &ep.inputs;
            for t in 2..xs.len() {
                let a = xs[t - 2] as usize;
                let b = xs[t - 1] as usize;
                let c = xs[t];
                if c < self.out as u8 {
                    self.counts[(a * self.vocab + b) * self.out + c as usize] += 1;
                }
            }
        }
    }
    fn scores(&self, a: u8, b: u8) -> Vec<f32> {
        let base = (a as usize * self.vocab + b as usize) * self.out;
        (0..self.out)
            .map(|v| (self.counts[base + v] as f32 + self.alpha).ln())
            .collect()
    }
    fn predict(&self, ep: &Episode) -> usize {
        let n = ep.inputs.len();
        let a = ep.inputs[n - 2];
        let b = ep.inputs[n - 1];
        let s = self.scores(a, b);
        argmax(&s)
    }
}

fn argmax(s: &[f32]) -> usize {
    let mut best = 0usize;
    let mut bv = f32::NEG_INFINITY;
    for (i, &v) in s.iter().enumerate() {
        if v > bv {
            bv = v;
            best = i;
        }
    }
    best
}

fn argmax_i32(s: &[i32]) -> usize {
    let mut best = 0usize;
    for i in 1..s.len() {
        if s[i] > s[best] {
            best = i;
        }
    }
    best
}

// Control (g) — hand-coded overwrite table key -> last value (no training).

fn overwrite_predict(ep: &Episode) -> usize {
    let mut table = [255u8; CONTENT];
    let xs = &ep.inputs;
    let n = xs.len();
    let mut t = 0usize;
    while t + 1 < n {
        if xs[t] == BIND && t + 2 < n && (xs[t + 1] as usize) < CONTENT {
            if (xs[t + 2] as usize) < CONTENT {
                table[xs[t + 1] as usize] = xs[t + 2];
            }
            t += 3;
        } else if xs[t] == QUERY && (xs[t + 1] as usize) < CONTENT {
            let k = xs[t + 1] as usize;
            return if table[k] == 255 {
                0
            } else {
                table[k] as usize
            };
        } else {
            t += 1;
        }
    }
    0
}

// Model. One struct, two flags. Flat parameter vector + layout so Adam and the
// finite-difference gradient check operate on one contiguous array.

#[derive(Clone, Copy)]
struct Layout {
    emb: usize,
    wh: usize,
    wf: usize,
    bh: usize,
    wo: usize,
    ag: usize,
    cg: usize,
    bw: usize,
    cw: usize,
    br: usize,
    cr: usize,
    beta: usize,
    total: usize,
}

fn take(o: &mut usize, n: usize) -> usize {
    let s = *o;
    *o += n;
    s
}

fn layout(d: usize, out: usize, vocab: usize) -> Layout {
    let mut o = 0usize;
    let emb = take(&mut o, vocab * d);
    let wh = take(&mut o, d * d);
    let wf = take(&mut o, d * NT);
    let bh = take(&mut o, d);
    let wo = take(&mut o, out * d);
    let ag = take(&mut o, vocab * d);
    let cg = take(&mut o, vocab);
    let bw = take(&mut o, vocab * d);
    let cw = take(&mut o, vocab);
    let br = take(&mut o, vocab * d);
    let cr = take(&mut o, vocab);
    let beta = take(&mut o, 1);
    Layout {
        emb,
        wh,
        wf,
        bh,
        wo,
        ag,
        cg,
        bw,
        cw,
        br,
        cr,
        beta,
        total: o,
    }
}

#[derive(Clone, Copy)]
struct Dims {
    d: usize,
    out: usize,
    vocab: usize,
    gate: bool,
    store: bool,
}

fn act(x: f32) -> f32 {
    x.clamp(-ACT_CAP, ACT_CAP)
}
fn act_deriv(x: f32) -> f32 {
    if x.abs() < ACT_CAP {
        1.0
    } else {
        0.0
    }
}
fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

/// Token type one-hot: content / BIND / QUERY.
fn type_onehot(x: u8, out: usize) -> [f32; NT] {
    if x as usize == out {
        [0.0, 1.0, 0.0]
    } else if x as usize == out + 1 {
        [0.0, 0.0, 1.0]
    } else {
        [1.0, 0.0, 0.0]
    }
}

#[derive(Clone)]
struct Tap {
    n: usize,
    xs: Vec<u8>,
    hs: Vec<f32>,
    zs: Vec<f32>,
    upds: Vec<f32>,
    gss: Vec<f32>,
    gprobs: Vec<f32>,
    wss: Vec<f32>,
    wprobs: Vec<f32>,
    rss: Vec<f32>,
    rprobs: Vec<f32>,
    wrote: Vec<u8>,
    addrs: Vec<usize>,
    vals: Vec<u8>,
    zbef: Vec<f32>,
    read: Vec<u8>,
    raddrs: Vec<usize>,
    zread: Vec<f32>,
    mems: Vec<f32>,
    scores: Vec<f32>,
    target: usize,
    hard_select: bool,
}

fn softmax_ce(scores: &[f32], target: usize) -> (f32, Vec<f32>) {
    let mx = scores.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let mut exps: Vec<f32> = scores.iter().map(|&s| (s - mx).exp()).collect();
    let sum: f32 = exps.iter().sum();
    if !sum.is_finite() || sum <= 0.0 {
        let n = scores.len().max(1);
        return ((n as f32).ln(), vec![1.0 / n as f32; n]);
    }
    for e in exps.iter_mut() {
        *e /= sum;
    }
    let loss = -(exps[target].max(1e-12)).ln();
    (loss, exps)
}

/// Soft forward pass, recording every intermediate the backward pass needs.
fn forward(p: &[f32], lay: &Layout, dims: Dims, ep: &Episode) -> (f32, Tap) {
    forward_with_readout(p, lay, dims, ep, false)
}

/// Hard forward with a sigmoid straight-through surrogate for the three gates.
/// The store and terminal read are the actual select operations during training.
fn forward_with_readout(
    p: &[f32],
    lay: &Layout,
    dims: Dims,
    ep: &Episode,
    hard_select: bool,
) -> (f32, Tap) {
    let d = dims.d;
    let out = dims.out;
    let n = ep.inputs.len();
    let xs = ep.inputs.clone();
    let mut hs = vec![0f32; (n + 1) * d];
    let mut zs = vec![0f32; n * d];
    let mut upds = vec![0f32; n * d];
    let mut gss = vec![0f32; n];
    let mut gprobs = vec![0f32; n];
    let mut wss = vec![0f32; n];
    let mut wprobs = vec![0f32; n];
    let mut rss = vec![0f32; n];
    let mut rprobs = vec![0f32; n];
    let mut wrote = vec![0u8; n];
    let mut addrs = vec![0usize; n];
    let mut vals = vec![0u8; n];
    let mut zbef = vec![0f32; n * out];
    let mut read = vec![0u8; n];
    let mut raddrs = vec![0usize; n];
    let mut zread = vec![0f32; n * out];
    let mut mems = vec![0f32; n * out];
    let mut z = vec![0f32; out * out];
    let mut h = vec![0f32; d];
    let mut hp = vec![0f32; d];
    let mut zi = vec![0f32; d];
    let mut upd = vec![0f32; d];
    for t in 0..n {
        hp.copy_from_slice(&h);
        let x = xs[t];
        // z = wh@hp + emb[x] + wf@type(x) + bh
        zi.copy_from_slice(&p[lay.bh..lay.bh + d]);
        let emb_off = lay.emb + x as usize * d;
        for j in 0..d {
            zi[j] += p[emb_off + j];
        }
        let ty = type_onehot(x, out);
        for j in 0..d {
            for (ti, &tval) in ty.iter().enumerate() {
                zi[j] += p[lay.wf + j * NT + ti] * tval;
            }
        }
        for r in 0..d {
            let row = lay.wh + r * d;
            let mut acc = 0f32;
            for c in 0..d {
                acc += p[row + c] * hp[c];
            }
            zi[r] += acc;
        }
        for j in 0..d {
            upd[j] = act(zi[j]);
        }
        zs[t * d..(t + 1) * d].copy_from_slice(&zi);
        upds[t * d..(t + 1) * d].copy_from_slice(&upd);
        if dims.gate {
            let ag_off = lay.ag + x as usize * d;
            let mut gl = p[lay.cg + x as usize];
            for j in 0..d {
                gl += p[ag_off + j] * hp[j];
            }
            let gs = sigmoid(gl);
            gprobs[t] = gs;
            let gs = if hard_select { f32::from(gl > 0.0) } else { gs };
            gss[t] = gs;
            for j in 0..d {
                h[j] = gs * hp[j] + (1.0 - gs) * upd[j];
            }
        } else {
            h.copy_from_slice(&upd);
        }
        if dims.store {
            if (x as usize) < out && t >= 1 && (xs[t - 1] as usize) < out {
                let key = xs[t - 1] as usize;
                let bw_off = lay.bw + x as usize * d;
                let mut wgl = p[lay.cw + x as usize];
                for j in 0..d {
                    wgl += p[bw_off + j] * hp[j];
                }
                let ws = sigmoid(wgl);
                wprobs[t] = ws;
                let ws = if hard_select {
                    f32::from(wgl > 0.0)
                } else {
                    ws
                };
                wss[t] = ws;
                wrote[t] = 1;
                addrs[t] = key;
                vals[t] = x;
                let row = key * out;
                for v in 0..out {
                    zbef[t * out + v] = z[row + v];
                }
                for v in 0..out {
                    let target_v = if v == x as usize { 1.0 } else { 0.0 };
                    z[row + v] = (1.0 - ws) * z[row + v] + ws * target_v;
                }
            }
            if (x as usize) < out {
                let key = x as usize;
                let br_off = lay.br + x as usize * d;
                let mut rgl = p[lay.cr + x as usize];
                for j in 0..d {
                    rgl += p[br_off + j] * hp[j];
                }
                let rs = sigmoid(rgl);
                rprobs[t] = rs;
                let rs = if hard_select {
                    f32::from(rgl > 0.0)
                } else {
                    rs
                };
                rss[t] = rs;
                read[t] = 1;
                raddrs[t] = key;
                for v in 0..out {
                    zread[t * out + v] = z[key * out + v];
                    mems[t * out + v] = rs * z[key * out + v];
                }
            }
        }
        hs[(t + 1) * d..(t + 2) * d].copy_from_slice(&h);
    }
    // scores[v] = wo[v] . h_last + beta * mem_last[v]
    let hlast = &hs[n * d..(n + 1) * d];
    let mut scores = vec![0f32; out];
    for v in 0..out {
        let row = lay.wo + v * d;
        let mut acc = 0f32;
        for j in 0..d {
            acc += p[row + j] * hlast[j];
        }
        scores[v] = acc;
    }
    if dims.store && hard_select {
        let mem = &mems[(n - 1) * out..n * out];
        if mem.iter().any(|&v| v > 0.0) {
            scores[argmax(mem)] += (1 << STORE_SHIFT) as f32;
        }
    } else if dims.store {
        let beta = p[lay.beta];
        for v in 0..out {
            scores[v] += beta * mems[(n - 1) * out + v];
        }
    }
    let (loss, _) = softmax_ce(&scores, ep.target as usize);
    let tap = Tap {
        n,
        xs,
        hs,
        zs,
        upds,
        gss,
        gprobs,
        wss,
        wprobs,
        rss,
        rprobs,
        wrote,
        addrs,
        vals,
        zbef,
        read,
        raddrs,
        zread,
        mems,
        scores,
        target: ep.target as usize,
        hard_select,
    };
    (loss, tap)
}

/// Reverse-mode BPTT for the soft forward pass. Accumulates into `grad` (caller
/// zeroes it); returns the loss.
fn backward(
    p: &[f32],
    lay: &Layout,
    dims: Dims,
    tap: &Tap,
    grad: &mut [f32],
    unscaled_read_grad: bool,
) -> f32 {
    let d = dims.d;
    let out = dims.out;
    let n = tap.n;
    let (loss, mut dscores) = softmax_ce(&tap.scores, tap.target);
    dscores[tap.target] -= 1.0;
    let hlast = &tap.hs[n * d..(n + 1) * d];
    for v in 0..out {
        let dv = dscores[v];
        let row = lay.wo + v * d;
        for j in 0..d {
            grad[row + j] += dv * hlast[j];
        }
    }
    let mut dh = vec![0f32; d];
    for v in 0..out {
        let dv = dscores[v];
        let row = lay.wo + v * d;
        for j in 0..d {
            dh[j] += dv * p[row + j];
        }
    }
    let mut dmem = vec![0f32; out];
    if dims.store {
        // Identity straight-through derivative of the selected one-hot memory
        // row. The forward loss sees the exact hard readout; the bounded
        // surrogate slope only affects optimization, never the served result.
        let beta = if tap.hard_select { 8.0 } else { p[lay.beta] };
        for v in 0..out {
            if !tap.hard_select {
                grad[lay.beta] += dscores[v] * tap.mems[(n - 1) * out + v];
            }
            dmem[v] = beta * dscores[v];
        }
    }
    let mut dz_store = vec![0f32; out * out];
    let mut dhp = vec![0f32; d];
    let mut dupd = vec![0f32; d];
    let mut dz = vec![0f32; d];
    let zero_mem = vec![0f32; out];
    for t in (0..n).rev() {
        for v in dhp.iter_mut() {
            *v = 0.0;
        }
        let x = tap.xs[t];
        let xc = x as usize;
        let hp = &tap.hs[t * d..(t + 1) * d];
        let upd = &tap.upds[t * d..(t + 1) * d];
        let dm: &[f32] = if t == n - 1 { &dmem } else { &zero_mem };
        // The forward read sees the store after this step's write. Reverse
        // order therefore accumulates read sensitivity before undoing write.
        let mut drgl = 0f32;
        if dims.store && tap.read[t] != 0 {
            let k = tap.raddrs[t];
            let r = tap.rss[t];
            let zread = &tap.zread[t * out..(t + 1) * out];
            for v in 0..out {
                dz_store[k * out + v] += r * dm[v];
                drgl += zread[v] * dm[v];
            }
        }
        let mut dwgl = 0f32;
        if dims.store && tap.wrote[t] != 0 {
            let k = tap.addrs[t];
            let val = tap.vals[t] as usize;
            let zb = &tap.zbef[t * out..(t + 1) * out];
            let mut dw = dz_store[k * out + val];
            for v in 0..out {
                dw -= dz_store[k * out + v] * zb[v];
            }
            let keep = 1.0 - tap.wss[t];
            for v in 0..out {
                dz_store[k * out + v] *= keep;
            }
            dwgl = dw * tap.wprobs[t] * (1.0 - tap.wprobs[t]);
        }
        if dims.gate {
            let gs = tap.gss[t];
            let mut dg = 0f32;
            for j in 0..d {
                dg += dh[j] * (hp[j] - upd[j]);
            }
            let gp = tap.gprobs[t];
            let dgl = dg * gp * (1.0 - gp);
            let agrow = &p[lay.ag + xc * d..lay.ag + xc * d + d];
            for j in 0..d {
                dhp[j] += gs * dh[j];
                dupd[j] = (1.0 - gs) * dh[j];
                dhp[j] += dgl * agrow[j];
                grad[lay.ag + xc * d + j] += dgl * hp[j];
            }
            grad[lay.cg + xc] += dgl;
        } else {
            dupd.copy_from_slice(&dh);
        }
        if dims.store {
            let brow = &p[lay.br + xc * d..lay.br + xc * d + d];
            let bwrow = &p[lay.bw + xc * d..lay.bw + xc * d + d];
            // Optional straight-through read-gate surrogate used by the
            // historical KVAR run. The exact sigmoid derivative is the
            // default and is covered by the finite-difference tests.
            let read_factor = if unscaled_read_grad {
                1.0
            } else {
                tap.rprobs[t] * (1.0 - tap.rprobs[t])
            };
            for j in 0..d {
                dhp[j] += drgl * read_factor * brow[j];
                dhp[j] += dwgl * bwrow[j];
                grad[lay.br + xc * d + j] += drgl * read_factor * hp[j];
                grad[lay.bw + xc * d + j] += dwgl * hp[j];
            }
            grad[lay.cr + xc] += drgl * read_factor;
            grad[lay.cw + xc] += dwgl;
        }
        let zrow = &tap.zs[t * d..(t + 1) * d];
        for j in 0..d {
            dz[j] = dupd[j] * act_deriv(zrow[j]);
        }
        for r in 0..d {
            let dzz = dz[r];
            if dzz != 0.0 {
                let row = lay.wh + r * d;
                for c in 0..d {
                    grad[row + c] += dzz * hp[c];
                }
            }
        }
        for c in 0..d {
            let mut acc = 0f32;
            for r in 0..d {
                acc += p[lay.wh + r * d + c] * dz[r];
            }
            dhp[c] += acc;
        }
        for j in 0..d {
            grad[lay.emb + xc * d + j] += dz[j];
            grad[lay.bh + j] += dz[j];
        }
        let ty = type_onehot(x, out);
        for j in 0..d {
            for (ti, &tv) in ty.iter().enumerate() {
                grad[lay.wf + j * NT + ti] += dz[j] * tv;
            }
        }
        dh.copy_from_slice(&dhp);
    }
    loss
}

struct Adam {
    m: Vec<f32>,
    v: Vec<f32>,
    t: u32,
}

impl Adam {
    fn new(n: usize) -> Self {
        Adam {
            m: vec![0f32; n],
            v: vec![0f32; n],
            t: 0,
        }
    }
    fn step(&mut self, p: &mut [f32], g: &[f32], lr: f32) {
        self.t += 1;
        let b1 = 0.9f32;
        let b2 = 0.999f32;
        let eps = 1e-8f32;
        let bc1 = 1.0 - b1.powi(self.t as i32);
        let bc2 = 1.0 - b2.powi(self.t as i32);
        for i in 0..p.len() {
            self.m[i] = b1 * self.m[i] + (1.0 - b1) * g[i];
            self.v[i] = b2 * self.v[i] + (1.0 - b2) * g[i] * g[i];
            let mh = self.m[i] / bc1;
            let vh = self.v[i] / bc2;
            p[i] -= lr * mh / (vh.sqrt() + eps);
        }
    }
}

fn grad_norm(g: &[f32]) -> f32 {
    let mut s = 0f32;
    for &x in g {
        s += x * x;
    }
    s.sqrt()
}

fn init_params(lay: &Layout, dims: Dims, seed: u64) -> Vec<f32> {
    let mut rng = Rng::new(seed ^ 0xABCD_1234_5678_9999);
    let mut p = vec![0f32; lay.total];
    let scale = 0.02f32;
    let blocks: [(usize, usize); 7] = [
        (lay.emb, dims.vocab * dims.d),
        (lay.wh, dims.d * dims.d),
        (lay.wf, dims.d * NT),
        (lay.wo, dims.out * dims.d),
        (lay.ag, dims.vocab * dims.d),
        (lay.bw, dims.vocab * dims.d),
        (lay.br, dims.vocab * dims.d),
    ];
    for (off, n) in blocks {
        for i in 0..n {
            p[off + i] = gauss(&mut rng) * scale;
        }
    }
    p
}

fn quant_i8(vals: &[f32]) -> (Vec<i8>, i32) {
    let mx = vals.iter().fold(0f32, |m, &v| m.max(v.abs()));
    if mx <= 0.0 {
        return (vec![0i8; vals.len()], 0);
    }
    let sh = mx.log2().floor() as i32;
    let scale = 2f32.powi(sh);
    let q = vals
        .iter()
        .map(|&v| (v / scale).round().clamp(-127.0, 127.0) as i8)
        .collect();
    (q, sh)
}

fn quant_tern(vals: &[f32]) -> Vec<i8> {
    vals.iter()
        .map(|&v| {
            if v > 0.0 {
                1i8
            } else if v < 0.0 {
                -1i8
            } else {
                0i8
            }
        })
        .collect()
}

#[derive(serde::Serialize, serde::Deserialize)]
struct QParams {
    wh: Vec<i8>,
    wf: Vec<i8>,
    wo: Vec<i8>,
    ag: Vec<i8>,
    bw: Vec<i8>,
    br: Vec<i8>,
    emb: Vec<i8>,
    emb_sh: i32,
    bh: Vec<i8>,
    bh_sh: i32,
    cg: Vec<i8>,
    cg_sh: i32,
    cw: Vec<i8>,
    cw_sh: i32,
    cr: Vec<i8>,
    cr_sh: i32,
    beta_sh: i32,
    beta_neg: bool,
}

fn served_quantize(p: &[f32], lay: &Layout, dims: Dims) -> QParams {
    let d = dims.d;
    let out = dims.out;
    let (emb, emb_sh) = quant_i8(&p[lay.emb..lay.emb + dims.vocab * d]);
    let (bh, bh_sh) = quant_i8(&p[lay.bh..lay.bh + d]);
    let (cg, cg_sh) = quant_i8(&p[lay.cg..lay.cg + dims.vocab]);
    let (cw, cw_sh) = quant_i8(&p[lay.cw..lay.cw + dims.vocab]);
    let (cr, cr_sh) = quant_i8(&p[lay.cr..lay.cr + dims.vocab]);
    let beta = p[lay.beta];
    let beta_neg = beta < 0.0;
    let beta_sh = if beta.abs() <= 0.0 {
        0
    } else {
        (beta.abs().log2().round() as i32).clamp(0, 20)
    };
    QParams {
        wh: quant_tern(&p[lay.wh..lay.wh + d * d]),
        wf: quant_tern(&p[lay.wf..lay.wf + d * NT]),
        wo: quant_tern(&p[lay.wo..lay.wo + out * d]),
        ag: quant_tern(&p[lay.ag..lay.ag + dims.vocab * d]),
        bw: quant_tern(&p[lay.bw..lay.bw + dims.vocab * d]),
        br: quant_tern(&p[lay.br..lay.br + dims.vocab * d]),
        emb,
        emb_sh,
        bh,
        bh_sh,
        cg,
        cg_sh,
        cw,
        cw_sh,
        cr,
        cr_sh,
        beta_sh,
        beta_neg,
    }
}

#[inline]
fn shl(x: i32, sh: i32) -> i32 {
    if sh >= 0 {
        x << sh
    } else {
        x >> (-sh)
    }
}

#[inline]
fn tern_mul(t: i8, x: i32) -> i32 {
    match t {
        1 => x,
        -1 => -x,
        _ => 0,
    }
}

/// Multiplier-free integer matvec over a ternary matrix: selects add/subtract/zero.
fn qmat_t(w: &[i8], rows: usize, cols: usize, x: &[i32]) -> Vec<i32> {
    let mut y = vec![0i32; rows];
    for r in 0..rows {
        let row = &w[r * cols..r * cols + cols];
        let mut acc = 0i32;
        for c in 0..cols {
            acc += tern_mul(row[c], x[c]);
        }
        y[r] = acc;
    }
    y
}

/// Multiplier-free integer dot over a ternary row.
fn qdot_t(w: &[i8], x: &[i32]) -> i32 {
    let mut acc = 0i32;
    for (i, &wi) in w.iter().enumerate() {
        acc += tern_mul(wi, x[i]);
    }
    acc
}

fn type_i32(x: u8, out: usize) -> [i32; NT] {
    if x as usize == out {
        [0, 1, 0]
    } else if x as usize == out + 1 {
        [0, 0, 1]
    } else {
        [1, 0, 0]
    }
}

/// Served integer/select forward: no float, no multiply. Returns (argmax, scores).
/// `select_readout` is a diagnostic variant for store arms only: when the read gate
/// fired at the final step, the store row is the answer instead of an additive term.
fn serve_scores(q: &QParams, dims: Dims, ep: &Episode, select_readout: bool) -> (usize, Vec<i32>) {
    let (pred, scores, _) = serve_scores_trace(q, dims, ep, select_readout);
    (pred, scores)
}

#[derive(Clone)]
struct TerminalTrace {
    query: u8,
    value: u8,
    valid: bool,
    gate_logit: i32,
    base_scores: Vec<i32>,
    target: usize,
}

fn serve_scores_trace(
    q: &QParams,
    dims: Dims,
    ep: &Episode,
    select_readout: bool,
) -> (usize, Vec<i32>, TerminalTrace) {
    let d = dims.d;
    let out = dims.out;
    let n = ep.inputs.len();
    let mut h = vec![0i32; d];
    let mut ztab = vec![0i32; out * out];
    let mut valid = vec![false; out];
    let mut mem_last = vec![0i32; out];
    let mut last_read = false;
    let mut terminal_gate_logit = 0i32;
    let mut terminal_value = 0u8;
    let mut terminal_valid = false;
    for t in 0..n {
        let hp = h.clone();
        let x = ep.inputs[t] as usize;
        let mut z = qmat_t(&q.wh, d, d, &hp);
        for j in 0..d {
            z[j] += shl(q.emb[x * d + j] as i32, q.emb_sh) + shl(q.bh[j] as i32, q.bh_sh);
        }
        let tv = type_i32(ep.inputs[t], out);
        let wfz = qmat_t(&q.wf, d, NT, &tv);
        for j in 0..d {
            z[j] += wfz[j];
        }
        let upd: Vec<i32> = z.iter().map(|&v| v.clamp(-8, 8)).collect();
        if dims.gate {
            let mut gl = shl(q.cg[x] as i32, q.cg_sh);
            gl += qdot_t(&q.ag[x * d..x * d + d], &hp);
            h = if gl > 0 { hp.clone() } else { upd };
        } else {
            h = upd;
        }
        if dims.store {
            if x < out && t >= 1 && (ep.inputs[t - 1] as usize) < out {
                let key = ep.inputs[t - 1] as usize;
                let mut wgl = shl(q.cw[x] as i32, q.cw_sh);
                wgl += qdot_t(&q.bw[x * d..x * d + d], &hp);
                if wgl > 0 {
                    for v in 0..out {
                        ztab[key * out + v] = if v == x { 1 } else { 0 };
                    }
                    valid[key] = true;
                }
            }
            let mut mem = vec![0i32; out];
            last_read = false;
            if x < out {
                let mut rgl = shl(q.cr[x] as i32, q.cr_sh);
                rgl += qdot_t(&q.br[x * d..x * d + d], &hp);
                if t + 1 == n {
                    terminal_gate_logit = rgl;
                    terminal_valid = valid[x];
                    if valid[x] {
                        terminal_value = argmax_i32(&ztab[x * out..(x + 1) * out]) as u8;
                    }
                }
                if rgl > 0 && valid[x] {
                    last_read = true;
                    for v in 0..out {
                        mem[v] = ztab[x * out + v];
                    }
                }
            }
            mem_last = mem;
        }
    }
    let mut scores = vec![0i32; out];
    for v in 0..out {
        scores[v] = qdot_t(&q.wo[v * d..v * d + d], &h);
    }
    let trace = TerminalTrace {
        query: ep.inputs[n - 1],
        value: terminal_value,
        valid: terminal_valid,
        gate_logit: terminal_gate_logit,
        base_scores: scores.clone(),
        target: ep.target as usize,
    };
    if dims.store {
        if select_readout {
            if last_read {
                let am = argmax_i32(&mem_last);
                scores[am] += 1 << STORE_SHIFT;
            }
        } else {
            for v in 0..out {
                let m = shl(mem_last[v], q.beta_sh);
                scores[v] += if q.beta_neg { -m } else { m };
            }
        }
    }
    let mut best = 0usize;
    for v in 1..out {
        if scores[v] > scores[best] {
            best = v;
        }
    }
    (best, scores, trace)
}

fn bits_at(scores: &[f32], target: usize, temp: f32) -> f64 {
    let mx = scores.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let mut sum = 0f64;
    for &s in scores {
        sum += (((s - mx) / temp) as f64).exp();
    }
    let p = (((scores[target] - mx) / temp) as f64).exp() / sum;
    -(p.max(1e-12)).log2()
}

fn fit_temp(score_rows: &[Vec<f32>], targets: &[usize]) -> f32 {
    let grid = [
        0.25f32, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0,
    ];
    let mut best_t = 1.0f32;
    let mut best_bits = f64::INFINITY;
    for &t in &grid {
        let mut s = 0f64;
        for (row, &tg) in score_rows.iter().zip(targets) {
            s += bits_at(row, tg, t);
        }
        if s < best_bits {
            best_bits = s;
            best_t = t;
        }
    }
    best_t
}

fn train_arm(
    dims: Dims,
    lay: &Layout,
    seed: u64,
    train: &[Episode],
    steps: usize,
    batch: usize,
    hard_train: bool,
    hard_warmup: usize,
    hard_aux_weight: f32,
    unscaled_read_grad: bool,
) -> Vec<f32> {
    let mut rng = Rng::new(seed ^ 0xFEED_FACE_CAFE_BEEF);
    let mut p = init_params(lay, dims, seed);
    let mut adam = Adam::new(lay.total);
    let mut grad = vec![0f32; lay.total];
    let mut soft_grad = vec![0f32; lay.total];
    let mut hard_grad = vec![0f32; lay.total];
    for step in 0..steps {
        for g in grad.iter_mut() {
            *g = 0.0;
        }
        for _b in 0..batch {
            let ep = &train[rng.below(train.len())];
            if hard_aux_weight > 0.0 && dims.store {
                soft_grad.fill(0.0);
                hard_grad.fill(0.0);
                let (_, soft_tap) = forward(&p, lay, dims, ep);
                backward(&p, lay, dims, &soft_tap, &mut soft_grad, unscaled_read_grad);
                let (_, hard_tap) = forward_with_readout(&p, lay, dims, ep, true);
                backward(&p, lay, dims, &hard_tap, &mut hard_grad, unscaled_read_grad);
                let scale = grad_norm(&soft_grad) / grad_norm(&hard_grad).max(1e-6);
                for i in 0..grad.len() {
                    grad[i] += (1.0 - hard_aux_weight) * soft_grad[i]
                        + hard_aux_weight * scale * hard_grad[i];
                }
            } else {
                let (_loss, tap) =
                    forward_with_readout(&p, lay, dims, ep, hard_train && step >= hard_warmup);
                backward(&p, lay, dims, &tap, &mut grad, unscaled_read_grad);
            }
        }
        let inv = 1.0 / batch as f32;
        for g in grad.iter_mut() {
            *g *= inv;
        }
        let nrm = grad_norm(&grad);
        if nrm > CLIP {
            let s = CLIP / nrm;
            for g in grad.iter_mut() {
                *g *= s;
            }
        }
        adam.step(&mut p, &grad, LR);
    }
    p
}

fn float_predict(p: &[f32], lay: &Layout, dims: Dims, ep: &Episode) -> (usize, Vec<f32>) {
    let (_l, tap) = forward(p, lay, dims, ep);
    let mut best = 0usize;
    for v in 1..dims.out {
        if tap.scores[v] > tap.scores[best] {
            best = v;
        }
    }
    (best, tap.scores)
}

fn bootstrap_mean(vals: &[f64], resamples: usize, rng: &mut Rng) -> (f64, f64) {
    if vals.is_empty() {
        return (f64::NAN, f64::NAN);
    }
    let n = vals.len();
    let mut means = Vec::with_capacity(resamples);
    for _ in 0..resamples {
        let mut s = 0f64;
        for _ in 0..n {
            s += vals[rng.below(n)];
        }
        means.push(s / n as f64);
    }
    means.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    (
        means[(resamples as f64 * 0.025) as usize],
        means[((resamples as f64 * 0.975) as usize).min(resamples - 1)],
    )
}

fn bootstrap_paired(a: &[f64], b: &[f64], resamples: usize, rng: &mut Rng) -> (f64, f64) {
    if a.is_empty() {
        return (f64::NAN, f64::NAN);
    }
    let n = a.len();
    let mut means = Vec::with_capacity(resamples);
    for _ in 0..resamples {
        let mut s = 0f64;
        for _ in 0..n {
            let i = rng.below(n);
            s += a[i] - b[i];
        }
        means.push(s / n as f64);
    }
    means.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    (
        means[(resamples as f64 * 0.025) as usize],
        means[((resamples as f64 * 0.975) as usize).min(resamples - 1)],
    )
}

fn mean(v: &[f64]) -> f64 {
    if v.is_empty() {
        f64::NAN
    } else {
        v.iter().sum::<f64>() / v.len() as f64
    }
}

fn d5_report(q: &QParams, dims: Dims) -> Value {
    let d = dims.d;
    let out = dims.out;
    let nz = |v: &[i8]| v.iter().filter(|&&x| x != 0).count();
    let emb_row_slots = d;
    let emb_row_nz = nz(&q.emb[..d]);
    let wh_slots = d * d;
    let wh_nz = nz(&q.wh);
    let wf_slots = d * NT;
    let wf_nz = nz(&q.wf);
    let bh_slots = d;
    let wo_slots = out * d;
    let wo_nz = nz(&q.wo);
    let mut slots = emb_row_slots + wh_slots + wf_slots + bh_slots + wo_slots;
    let mut selects = emb_row_nz + wh_nz + wf_nz + wo_nz;
    let mut extra = json!({});
    if dims.gate {
        let row_slots = d + 1;
        let row_nz = nz(&q.ag[..d]) + 1;
        slots += row_slots;
        selects += row_nz;
        extra["gate_row_slots"] = json!(row_slots);
        extra["gate_row_nonzero"] = json!(row_nz);
    }
    if dims.store {
        let w_row_nz = nz(&q.bw[..d]) + 1;
        let r_row_nz = nz(&q.br[..d]) + 1;
        slots += 2 * (d + 1) + out;
        selects += w_row_nz + r_row_nz + out;
        extra["write_gate_row_slots"] = json!(d + 1);
        extra["read_gate_row_slots"] = json!(d + 1);
        extra["store_read_row_slots"] = json!(out);
        extra["write_gate_row_nonzero"] = json!(w_row_nz);
        extra["read_gate_row_nonzero"] = json!(r_row_nz);
    }
    json!({
        "arm": if dims.store { "c" } else if dims.gate { "b" } else { "a" },
        "per_token_slots_inspected": slots,
        "per_token_nonzero_selects": selects,
        "dense_detail": {
            "wh_slots": wh_slots, "wh_nonzero": wh_nz,
            "wo_slots": wo_slots, "wo_nonzero": wo_nz,
            "wf_slots": wf_slots, "wf_nonzero": wf_nz,
            "emb_row_slots": emb_row_slots, "emb_row_nonzero": emb_row_nz,
            "bh_slots": bh_slots,
        },
        "extra": extra,
        "note": "table rows (emb/gate/store read) are reads; ternary/int4 maps execute as add/sub/shift selects with no multiply",
    })
}

fn panel_cells() -> Vec<(usize, usize)> {
    let mut v = Vec::new();
    for &k in KS.iter() {
        for &l in LAGS.iter() {
            v.push((k, l));
        }
    }
    v
}

struct Config {
    root: PathBuf,
    steps: usize,
    train_per_cell: usize,
    held_per_cell: usize,
    seeds: Vec<u64>,
    hard_train: bool,
    hard_warmup: usize,
    hard_aux_weight: f32,
    unscaled_read_grad: bool,
    held_seed_group: u64,
    selected_arms: Vec<String>,
}

fn parse_args() -> Result<Config, String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return Err("usage: kvar-recall NEW_REPORT_ROOT [--steps N] [--train-per-cell N] [--held-per-cell N] [--seeds A,B] [--hard-train] [--hard-warmup N] [--hard-aux-weight W] [--unscaled-read-grad] [--held-seed-group N] [--arms a,b,c] [--quick]".into());
    }
    let mut cfg = Config {
        root: PathBuf::from(&args[1]),
        steps: TRAIN_STEPS,
        train_per_cell: TRAIN_EPISODES_PER_CELL,
        held_per_cell: HELD_EPISODES_PER_CELL,
        seeds: vec![1, 2],
        hard_train: false,
        hard_warmup: 0,
        hard_aux_weight: 0.0,
        unscaled_read_grad: false,
        held_seed_group: 2,
        selected_arms: vec!["a".into(), "b".into(), "c".into()],
    };
    let mut i = 2usize;
    while i < args.len() {
        match args[i].as_str() {
            "--steps" => {
                i += 1;
                cfg.steps = args
                    .get(i)
                    .ok_or("--steps needs a value")?
                    .parse()
                    .map_err(|e| format!("--steps: {e}"))?;
            }
            "--train-per-cell" => {
                i += 1;
                cfg.train_per_cell = args
                    .get(i)
                    .ok_or("--train-per-cell needs a value")?
                    .parse()
                    .map_err(|e| format!("--train-per-cell: {e}"))?;
            }
            "--held-per-cell" => {
                i += 1;
                cfg.held_per_cell = args
                    .get(i)
                    .ok_or("--held-per-cell needs a value")?
                    .parse()
                    .map_err(|e| format!("--held-per-cell: {e}"))?;
            }
            "--seeds" => {
                i += 1;
                let s = args.get(i).ok_or("--seeds needs a value")?;
                cfg.seeds = s
                    .split(',')
                    .map(|x| x.trim().parse::<u64>().map_err(|e| format!("--seeds: {e}")))
                    .collect::<Result<Vec<_>, _>>()?;
            }
            "--hard-train" => cfg.hard_train = true,
            "--hard-warmup" => {
                i += 1;
                cfg.hard_warmup = args
                    .get(i)
                    .ok_or("--hard-warmup needs a value")?
                    .parse()
                    .map_err(|e| format!("--hard-warmup: {e}"))?;
            }
            "--hard-aux-weight" => {
                i += 1;
                cfg.hard_aux_weight = args
                    .get(i)
                    .ok_or("--hard-aux-weight needs a value")?
                    .parse()
                    .map_err(|e| format!("--hard-aux-weight: {e}"))?;
            }
            "--unscaled-read-grad" => cfg.unscaled_read_grad = true,
            "--held-seed-group" => {
                i += 1;
                cfg.held_seed_group = args
                    .get(i)
                    .ok_or("--held-seed-group needs a value")?
                    .parse()
                    .map_err(|e| format!("--held-seed-group: {e}"))?;
                if cfg.held_seed_group == 1 {
                    return Err("held seed group must differ from training group 1".into());
                }
            }
            "--arms" => {
                i += 1;
                cfg.selected_arms = args
                    .get(i)
                    .ok_or("--arms needs a value")?
                    .split(',')
                    .map(str::to_owned)
                    .collect();
                if cfg.selected_arms.is_empty()
                    || cfg
                        .selected_arms
                        .iter()
                        .any(|a| !["a", "b", "c"].contains(&a.as_str()))
                {
                    return Err("--arms must be a nonempty comma-separated subset of a,b,c".into());
                }
            }
            "--quick" => {
                cfg.steps = 60;
                cfg.train_per_cell = 8;
                cfg.held_per_cell = 8;
                cfg.seeds = vec![1];
            }
            other => return Err(format!("unknown argument {other}")),
        }
        i += 1;
    }
    if cfg.hard_warmup > cfg.steps || (cfg.hard_warmup > 0 && !cfg.hard_train) {
        return Err("--hard-warmup requires --hard-train and must not exceed --steps".into());
    }
    if !(0.0..=1.0).contains(&cfg.hard_aux_weight) || (cfg.hard_aux_weight > 0.0 && cfg.hard_train)
    {
        return Err(
            "--hard-aux-weight must be in [0,1] and cannot combine with --hard-train".into(),
        );
    }
    Ok(cfg)
}

struct ArmConfig {
    name: &'static str,
    gate: bool,
    store: bool,
}

fn run() -> Result<(), String> {
    let cfg = parse_args()?;
    claim(&cfg.root).map_err(|e| e.to_string())?;
    let dims_full = Dims {
        d: D,
        out: OUT,
        vocab: VOCAB,
        gate: false,
        store: false,
    };
    let lay = layout(D, OUT, VOCAB);
    let cells = panel_cells();
    let mut train: Vec<Episode> = Vec::new();
    let mut held: Vec<Episode> = Vec::new();
    let mut held_cell: Vec<usize> = Vec::new();
    for (ci, &(k, l)) in cells.iter().enumerate() {
        for i in 0..cfg.train_per_cell {
            train.push(gen_episode(seed_for(1, k, l, i), k, l));
        }
        for i in 0..cfg.held_per_cell {
            held.push(gen_episode(seed_for(cfg.held_seed_group, k, l, i), k, l));
            held_cell.push(ci);
        }
    }
    let mut ctrl = Count2::new(VOCAB, OUT);
    ctrl.train(&train);
    let c_pred: Vec<usize> = held.iter().map(|e| ctrl.predict(e)).collect();
    let g_pred: Vec<usize> = held.iter().map(overwrite_predict).collect();
    let targets: Vec<usize> = held.iter().map(|e| e.target as usize).collect();
    let c_acc = c_pred
        .iter()
        .zip(&targets)
        .filter(|(p, t)| *p == *t)
        .count() as f64
        / held.len() as f64;
    let g_acc = g_pred
        .iter()
        .zip(&targets)
        .filter(|(p, t)| *p == *t)
        .count() as f64
        / held.len() as f64;
    let c_design: Vec<Vec<f32>> = train
        .iter()
        .map(|e| {
            let n = e.inputs.len();
            ctrl.scores(e.inputs[n - 2], e.inputs[n - 1])
        })
        .collect();
    let c_train_targets: Vec<usize> = train.iter().map(|e| e.target as usize).collect();
    let c_temp = fit_temp(&c_design, &c_train_targets);
    let c_held_scores: Vec<Vec<f32>> = held
        .iter()
        .map(|e| {
            let n = e.inputs.len();
            ctrl.scores(e.inputs[n - 2], e.inputs[n - 1])
        })
        .collect();
    let c_bits: Vec<f64> = c_held_scores
        .iter()
        .zip(&targets)
        .map(|(s, &t)| bits_at(s, t, c_temp))
        .collect();
    let g_bits: Vec<f64> = g_pred
        .iter()
        .zip(&targets)
        .map(|(p, t)| if p == t { 0.01 } else { 12.0 })
        .collect();

    let arms = [
        ArmConfig {
            name: "a",
            gate: false,
            store: false,
        },
        ArmConfig {
            name: "b",
            gate: true,
            store: false,
        },
        ArmConfig {
            name: "c",
            gate: true,
            store: true,
        },
    ];
    let mut rng = Rng::new(0x5EED_1234_5678_9ABC);
    let mut arm_json: Vec<Value> = Vec::new();
    let mut d5_json: Vec<Value> = Vec::new();
    let mut param_json: Vec<Value> = Vec::new();
    let mut prediction_json: Vec<Value> = Vec::new();
    let mut summary: Vec<(String, f64, f64, f64, f64, f64)> = Vec::new();
    for ac in arms.iter() {
        if !cfg.selected_arms.iter().any(|a| a == ac.name) {
            continue;
        }
        let dims = Dims {
            gate: ac.gate,
            store: ac.store,
            ..dims_full
        };
        for &seed in cfg.seeds.iter() {
            let p = train_arm(
                dims,
                &lay,
                seed,
                &train,
                cfg.steps,
                BATCH,
                cfg.hard_train,
                cfg.hard_warmup,
                cfg.hard_aux_weight,
                cfg.unscaled_read_grad,
            );
            let q = served_quantize(&p, &lay, dims);
            let mut d5 = d5_report(&q, dims);
            d5["seed"] = json!(seed);
            d5_json.push(d5);
            let design_rows: Vec<Vec<f32>> = train
                .iter()
                .map(|e| {
                    let (_a, s) = serve_scores(&q, dims, e, true);
                    s.iter().map(|&v| v as f32).collect()
                })
                .collect();
            let temp = fit_temp(&design_rows, &c_train_targets);
            let mut correct: Vec<f64> = Vec::with_capacity(held.len());
            let mut bits: Vec<f64> = Vec::with_capacity(held.len());
            let mut float_ok: Vec<f64> = Vec::with_capacity(held.len());
            let mut hard_float_ok: Vec<f64> = Vec::with_capacity(held.len());
            let mut additive_ok: Vec<f64> = Vec::with_capacity(held.len());
            for (episode_index, e) in held.iter().enumerate() {
                let (pred, sc) = serve_scores(&q, dims, e, true);
                correct.push(if pred == e.target as usize { 1.0 } else { 0.0 });
                let scf: Vec<f32> = sc.iter().map(|&v| v as f32).collect();
                bits.push(bits_at(&scf, e.target as usize, temp));
                let (fp, _) = float_predict(&p, &lay, dims, e);
                float_ok.push(if fp == e.target as usize { 1.0 } else { 0.0 });
                let (_, hard_tap) = forward_with_readout(&p, &lay, dims, e, true);
                let hard_float_pred = argmax(&hard_tap.scores);
                hard_float_ok.push(if hard_float_pred == e.target as usize {
                    1.0
                } else {
                    0.0
                });
                prediction_json.push(json!({
                    "arm": ac.name, "seed": seed, "episode_index": episode_index,
                    "episode_seed": e.seed, "k": e.k, "lag": e.lag, "key": e.key,
                    "target": e.target, "served_prediction": pred,
                    "served_scores": sc, "hard_float_prediction": hard_float_pred,
                    "count_prediction": c_pred[episode_index],
                    "overwrite_prediction": g_pred[episode_index],
                }));
                if dims.store {
                    let (ap, _) = serve_scores(&q, dims, e, false);
                    additive_ok.push(if ap == e.target as usize { 1.0 } else { 0.0 });
                }
            }
            let acc = mean(&correct);
            let (acc_lo, acc_hi) = bootstrap_mean(&correct, BOOTSTRAP, &mut rng);
            let mb = mean(&bits);
            let (d_lo, d_hi) = bootstrap_paired(&bits, &c_bits, BOOTSTRAP, &mut rng);
            let float_acc = mean(&float_ok);
            let mut per_cell: Vec<Value> = Vec::new();
            for (ci, &(k, l)) in cells.iter().enumerate() {
                let idx: Vec<usize> = (0..held.len()).filter(|&i| held_cell[i] == ci).collect();
                let c: Vec<f64> = idx.iter().map(|&i| correct[i]).collect();
                let b: Vec<f64> = idx.iter().map(|&i| bits[i]).collect();
                let cb: Vec<f64> = idx.iter().map(|&i| c_bits[i]).collect();
                let fo: Vec<f64> = idx.iter().map(|&i| float_ok[i]).collect();
                let ao: Vec<f64> = if dims.store {
                    idx.iter().map(|&i| additive_ok[i]).collect()
                } else {
                    Vec::new()
                };
                per_cell.push(json!({
                    "k": k, "lag": l,
                    "episodes": idx.len(),
                    "accuracy": mean(&c),
                    "bits_per_query": mean(&b),
                    "count_control_accuracy": idx.iter().filter(|&&i| c_pred[i] == targets[i]).count() as f64 / idx.len() as f64,
                    "bits_vs_count": mean(&b) - mean(&cb),
                    "float_diagnostic_accuracy": mean(&fo),
                    "additive_diagnostic_accuracy": if dims.store { json!(mean(&ao)) } else { Value::Null },
                    "served_accuracy_ci_upper": acc_hi,
                }));
            }
            summary.push((
                format!("{}-s{seed}", ac.name),
                acc,
                mb,
                mean(&c_bits) - mb,
                acc_lo,
                float_acc,
            ));
            arm_json.push(json!({
                "arm": ac.name,
                "seed": seed,
                "gate": ac.gate,
                "store": ac.store,
                "readout": "hard_select",
                "steps": cfg.steps,
                "batch": BATCH,
                "hard_train": cfg.hard_train,
                "hard_warmup": cfg.hard_warmup,
                "hard_aux_weight": cfg.hard_aux_weight,
                "unscaled_read_grad": cfg.unscaled_read_grad,
                "accuracy": acc,
                "accuracy_ci95": [acc_lo, acc_hi],
                "bits_per_query": mb,
                "bits_vs_count": mean(&c_bits) - mb,
                "bits_vs_count_ci95": [-d_hi, -d_lo],
                "calibration_temperature": temp,
                "float_diagnostic_accuracy": float_acc,
                "hard_float_diagnostic_accuracy": mean(&hard_float_ok),
                "additive_diagnostic_accuracy": if dims.store { json!(mean(&additive_ok)) } else { Value::Null },
                "per_cell": per_cell,
            }));
            param_json.push(json!({"arm": ac.name, "seed": seed, "quantized_parameters": q}));
            if dims.store {
                println!(
                    "  diag {}-s{seed} additive-readout acc={:.4}",
                    ac.name,
                    mean(&additive_ok)
                );
            }
        }
    }

    let g_solves = g_acc >= 0.99;
    let chance = 1.0 / OUT as f64;
    let c_at_chance = (c_acc - chance).abs() <= 0.02;
    let mut beats_chance_2bits = false;
    for a in arm_json.iter() {
        let acc_lo = a["accuracy_ci95"][0].as_f64().unwrap_or(0.0);
        let vs = a["bits_vs_count"].as_f64().unwrap_or(0.0);
        if acc_lo > chance && vs >= 2.0 {
            beats_chance_2bits = true;
        }
    }
    let best_b = arm_json
        .iter()
        .filter(|a| a["arm"] == "b")
        .map(|a| a["bits_per_query"].as_f64().unwrap_or(f64::INFINITY))
        .fold(f64::INFINITY, f64::min);
    let best_c = arm_json
        .iter()
        .filter(|a| a["arm"] == "c")
        .map(|a| a["bits_per_query"].as_f64().unwrap_or(f64::INFINITY))
        .fold(f64::INFINITY, f64::min);
    let write_lever = best_b.is_finite() && best_c.is_finite() && best_c < best_b;
    let served_at_chance = arm_json.iter().all(|a| {
        let lo = a["accuracy_ci95"][0].as_f64().unwrap_or(0.0);
        let hi = a["accuracy_ci95"][1].as_f64().unwrap_or(0.0);
        let pt = a["accuracy"].as_f64().unwrap_or(0.0);
        lo <= chance && hi >= chance && pt <= 3.0 * chance
    });
    let max_float_diag = arm_json
        .iter()
        .map(|a| a["float_diagnostic_accuracy"].as_f64().unwrap_or(0.0))
        .fold(0.0f64, f64::max);
    let realization_gap = served_at_chance && max_float_diag >= 0.10;
    let invalid = !g_solves || !c_at_chance;
    let full_arms = ["a", "b", "c"]
        .iter()
        .all(|name| cfg.selected_arms.iter().any(|selected| selected == name));
    let mut not_run = vec![
        "(d) width".to_owned(),
        "(e) width+gate".to_owned(),
        "(f) ordinary matched control".to_owned(),
        "(h) geometric parameterisation".to_owned(),
        "equal-work stateful n-gram control".to_owned(),
    ];
    for ac in arms.iter() {
        if !cfg.selected_arms.iter().any(|a| a == ac.name) {
            not_run.push(format!("({}) trainable arm", ac.name));
        }
    }
    let branch = if invalid {
        "INVALID_PANEL_DEFECT"
    } else if !full_arms {
        "PARTIAL_ARMS_NOT_ADJUDICABLE"
    } else if beats_chance_2bits && write_lever {
        "ACCEPT_MEMORY_MECHANISM"
    } else if g_solves && served_at_chance {
        "REJECT_CLASS_TERMINAL_ESCALATE"
    } else {
        "NO_ACCEPT_PARTIAL_OR_NULL"
    };

    let receipt = json!({
        "schema": "uor-r4.kvar-receipt/2",
        "base_commit": "552d847d",
        "plan": "docs/integration/kvar-plan-2026-09-24.md",
        "panel": {
            "content_vocab": CONTENT, "keys": KS, "lags": LAGS,
            "cells": cells.iter().map(|&(k,l)| format!("K{k}_L{l}")).collect::<Vec<_>>(),
            "train_episodes_per_cell": cfg.train_per_cell,
            "held_episodes_per_cell": cfg.held_per_cell,
            "train_episodes_total": train.len(),
            "held_episodes_total": held.len(),
            "design_seed_group": 1, "held_seed_group": cfg.held_seed_group,
            "query": "answer = query key's most recent value; lag = tokens between its final binding and QUERY",
        },
        "controls": {
            "count_order2_accuracy": c_acc,
            "count_order2_at_chance": c_at_chance,
            "count_temperature": c_temp,
            "count_bits_per_query": mean(&c_bits),
            "overwrite_table_accuracy": g_acc,
            "overwrite_table_bits_per_query": mean(&g_bits),
        },
        "arms": arm_json,
        "d5": d5_json,
        "decision": {
            "chance_accuracy": chance,
            "primary_readout": "hard_select",
            "partial_arm_run": !full_arms,
            "hard_train": cfg.hard_train,
            "hard_warmup": cfg.hard_warmup,
            "hard_aux_weight": cfg.hard_aux_weight,
            "unscaled_read_grad": cfg.unscaled_read_grad,
            "store_shift": STORE_SHIFT,
            "g_solves_panel": g_solves,
            "panel_valid": !invalid,
            "beats_chance_and_count_2bits": beats_chance_2bits,
            "store_write_lever_c_beats_b": write_lever,
            "served_all_arms_at_chance": served_at_chance,
            "max_float_diagnostic_accuracy": max_float_diag,
            "realization_gap_localized": realization_gap,
            "readout_note": "Primary served read is the plan-compliant hard select: scores = wo@h, plus 1<<STORE_SHIFT at argmax(mem) when the store read gate fired. The additive beta*mem+wo@h form is retained only as a labelled diagnostic; it was an earlier deviation from plan §5.1 and is NOT the evaluated artifact.",
            "geometry_gate_h": "NOT_RUN",
            "branch": branch,
        },
        "not_run": not_run,
        "selected_arms": cfg.selected_arms,
        "controls_note": "(b) gate/no-store is the model-family ordinary gated-recurrence control; the plan's (f) GRU is NOT_RUN.",
        "scope": "Synthetic KVAR panel only. No language, capability, reasoning, energy or geometric-advantage claim.",
    });
    let write = |name: &str, v: &Value| -> Result<(), String> {
        let bytes = serde_json::to_vec_pretty(v).map_err(|e| e.to_string())?;
        std::fs::write(cfg.root.join(name), bytes).map_err(|e| format!("{name}: {e}"))
    };
    write("receipt.json", &receipt)?;
    write("parameters.json", &json!({"arms": param_json}))?;
    write("predictions.json", &json!({"held_rows": prediction_json}))?;
    let panel = json!({
        "cells": cells.iter().map(|&(k,l)| json!({"k":k,"lag":l})).collect::<Vec<_>>(),
        "train": train.iter().map(|e| json!({"seed": e.seed, "k": e.k, "lag": e.lag, "key": e.key, "target": e.target, "inputs": e.inputs})).collect::<Vec<_>>(),
        "held": held.iter().map(|e| json!({"seed": e.seed, "k": e.k, "lag": e.lag, "key": e.key, "target": e.target, "inputs": e.inputs})).collect::<Vec<_>>(),
    });
    write("panel.json", &panel)?;
    let boot = json!({
        "resamples": BOOTSTRAP,
        "summary": summary.iter().map(|(n, acc, bits, vs, lo, fa)| json!({
            "arm_seed": n, "accuracy": acc, "bits_per_query": bits,
            "bits_better_than_count": vs, "accuracy_ci_lower": lo, "float_diagnostic_accuracy": fa,
        })).collect::<Vec<_>>(),
    });
    write("bootstrap.json", &boot)?;
    seal(&cfg.root).map_err(|e| e.to_string())?;
    let unlisted = verify(&cfg.root).map_err(|e| e.to_string())?;
    if !unlisted.is_empty() {
        return Err(format!("sealed report has unlisted files: {unlisted:?}"));
    }
    println!("KVAR root={}", cfg.root.display());
    println!(
        "panel train={} held={} cells={}",
        train.len(),
        held.len(),
        cells.len()
    );
    println!(
        "control C accuracy={c_acc:.4} (chance {chance:.4}, at_chance={c_at_chance}) bits={:.4}",
        mean(&c_bits)
    );
    println!("control (g) accuracy={g_acc:.4} bits={:.4}", mean(&g_bits));
    for (n, acc, bits, vs, lo, fa) in summary.iter() {
        println!("arm {n}: acc={acc:.4} ci_lo={lo:.4} bits={bits:.4} bits_better_than_C={vs:.4} float_diag_acc={fa:.4}");
    }
    println!("branch={branch}");
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(2).map(String::as_str) == Some("--relative-energy") {
        if let Err(e) = kvar_relative_energy::run(&args) {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
        return;
    }
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zero_q(dims: Dims) -> QParams {
        let d = dims.d;
        let out = dims.out;
        let v = dims.vocab;
        QParams {
            wh: vec![0; d * d],
            wf: vec![0; d * NT],
            wo: vec![0; out * d],
            ag: vec![0; v * d],
            bw: vec![0; v * d],
            br: vec![0; v * d],
            emb: vec![0; v * d],
            emb_sh: 0,
            bh: vec![0; d],
            bh_sh: 0,
            cg: vec![0; v],
            cg_sh: 0,
            cw: vec![0; v],
            cw_sh: 0,
            cr: vec![0; v],
            cr_sh: 0,
            beta_sh: 8,
            beta_neg: false,
        }
    }

    fn tiny_dims(gate: bool, store: bool) -> Dims {
        Dims {
            d: 4,
            out: 4,
            vocab: 6,
            gate,
            store,
        }
    }

    fn tiny_episode() -> Episode {
        Episode {
            inputs: vec![4, 0, 1, 4, 2, 3, 4, 0, 2, 1, 5, 0],
            target: 2,
            k: 2,
            lag: 1,
            key: 0,
            seed: 0,
        }
    }

    fn check_grads(gate: bool, store: bool) {
        let dims = tiny_dims(gate, store);
        let lay = layout(dims.d, dims.out, dims.vocab);
        let mut p = init_params(&lay, dims, 7);
        let ep = tiny_episode();
        let mut grad = vec![0f32; lay.total];
        let (_l, tap) = forward(&p, &lay, dims, &ep);
        backward(&p, &lay, dims, &tap, &mut grad, false);
        let eps = 2e-3f32;
        let mut checked = 0usize;
        let mut significant = 0usize;
        for i in 0..lay.total {
            let orig = p[i];
            p[i] = orig + eps;
            let lp = forward(&p, &lay, dims, &ep).0;
            p[i] = orig - eps;
            let lm = forward(&p, &lay, dims, &ep).0;
            p[i] = orig;
            let fd = (lp - lm) / (2.0 * eps);
            let a = grad[i];
            let absdiff = (a - fd).abs();
            let scale = a.abs().max(fd.abs());
            let ok = if scale > 1e-3 {
                significant += 1;
                absdiff / scale < 5e-2
            } else {
                absdiff < 2e-4
            };
            assert!(
                ok,
                "gate={gate} store={store} idx={i} analytic={a} fd={fd} absdiff={absdiff}"
            );
            checked += 1;
        }
        assert!(checked > 20, "checked {checked}");
        assert!(
            significant > 8,
            "significant gradients checked: {significant}"
        );
    }

    #[test]
    fn gradient_check_no_gate_no_store() {
        check_grads(false, false);
    }

    #[test]
    fn gradient_check_gate_only() {
        check_grads(true, false);
    }

    #[test]
    fn gradient_check_gate_and_store() {
        check_grads(true, true);
    }

    #[test]
    fn generator_rebinds_and_answer_is_most_recent() {
        for &k in KS.iter() {
            for &lag in LAGS.iter() {
                for i in 0..40usize {
                    let ep = gen_episode(seed_for(1, k, lag, i), k, lag);
                    let binds = ep.bindings();
                    let mut keys: Vec<u8> = binds.iter().map(|b| b.0).collect();
                    keys.sort_unstable();
                    keys.dedup();
                    assert_eq!(keys.len(), k, "distinct key count");
                    for key in keys {
                        let vals: Vec<u8> =
                            binds.iter().filter(|b| b.0 == key).map(|b| b.1).collect();
                        assert!(vals.len() >= 2, "key {key} bound < 2 times");
                        assert!(
                            vals.iter().any(|&v| v != vals[0]),
                            "key {key} never rebound to a different value"
                        );
                    }
                    let qi = ep
                        .inputs
                        .iter()
                        .position(|&t| t == QUERY)
                        .expect("QUERY present");
                    assert_eq!(ep.inputs[qi + 1], ep.key, "key follows QUERY");
                    let key_binds: Vec<(u8, u8, usize)> =
                        binds.iter().cloned().filter(|b| b.0 == ep.key).collect();
                    let last = *key_binds.last().expect("query key bound");
                    assert_eq!(last.1, ep.target, "target is the most recent value");
                    assert_eq!(qi - last.2 - 1, lag, "lag between binding and QUERY");
                    assert!(
                        key_binds.iter().all(|b| b.2 <= last.2),
                        "no query-key binding after the answer binding"
                    );
                }
            }
        }
    }

    #[test]
    fn overwrite_control_is_near_perfect() {
        let mut ok = 0usize;
        let mut tot = 0usize;
        for &k in KS.iter() {
            for &lag in LAGS.iter() {
                for i in 0..40usize {
                    let ep = gen_episode(seed_for(2, k, lag, i), k, lag);
                    if overwrite_predict(&ep) == ep.target as usize {
                        ok += 1;
                    }
                    tot += 1;
                }
            }
        }
        assert_eq!(ok, tot, "overwrite table control {ok}/{tot}");
    }

    #[test]
    fn count_control_is_at_chance() {
        let mut train = Vec::new();
        let mut held = Vec::new();
        for &k in KS.iter() {
            for &lag in LAGS.iter() {
                for i in 0..60usize {
                    train.push(gen_episode(seed_for(11, k, lag, i), k, lag));
                }
                for i in 0..60usize {
                    held.push(gen_episode(seed_for(12, k, lag, i), k, lag));
                }
            }
        }
        let mut c = Count2::new(VOCAB, OUT);
        c.train(&train);
        let ok = held
            .iter()
            .filter(|e| c.predict(e) == e.target as usize)
            .count();
        let acc = ok as f64 / held.len() as f64;
        assert!(acc < 0.08, "count control accuracy {acc} above chance band");
    }

    #[test]
    fn qmat_matches_integer_reference() {
        let mut rng = Rng::new(99);
        let rows = 5;
        let cols = 7;
        let w: Vec<i8> = (0..rows * cols).map(|_| (rng.below(3) as i8) - 1).collect();
        let x: Vec<i32> = (0..cols).map(|_| rng.below(21) as i32 - 10).collect();
        let y = qmat_t(&w, rows, cols, &x);
        for r in 0..rows {
            let mut acc = 0i32;
            for c in 0..cols {
                acc += w[r * cols + c] as i32 * x[c];
            }
            assert_eq!(y[r], acc, "row {r}");
        }
    }

    #[test]
    fn served_store_reads_most_recent_value() {
        let dims = tiny_dims(true, true);
        let out = dims.out;
        let mut q = zero_q(dims);
        for x in 0..out {
            q.cw[x] = 100;
            q.cr[x] = 100;
        }
        let (pred, _scores) = serve_scores(&q, dims, &tiny_episode(), true);
        assert_eq!(
            pred, 2,
            "served overwrite store returns the most recent value"
        );
    }

    #[test]
    fn empty_read_cannot_inject_token_zero() {
        let dims = tiny_dims(true, true);
        let mut q = zero_q(dims);
        q.cr[1] = 100;
        q.bh[0] = 1;
        q.wo[3 * dims.d] = 1;
        let ep = Episode {
            inputs: vec![5, 1],
            target: 3,
            k: 1,
            lag: 0,
            key: 1,
            seed: 0,
        };
        let (pred, scores) = serve_scores(&q, dims, &ep, true);
        assert_eq!(pred, 3);
        assert_eq!(scores[0], 0);
    }

    #[test]
    fn hard_training_forward_selects_latest_written_value() {
        let dims = tiny_dims(true, true);
        let lay = layout(dims.d, dims.out, dims.vocab);
        let mut p = vec![0.0; lay.total];
        for x in 0..dims.out {
            p[lay.cw + x] = 1.0;
            p[lay.cr + x] = 1.0;
        }
        let (_, tap) = forward_with_readout(&p, &lay, dims, &tiny_episode(), true);
        assert_eq!(argmax(&tap.scores), 2);
        assert!(tap.scores[2] >= (1 << STORE_SHIFT) as f32);
    }
}

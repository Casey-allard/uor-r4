//! Finite-policy feasibility: counterfactual action outcomes and a bounded constrained solve.
//!
//! This answers the one question the fitted-table evidence cannot: does *some* deterministic action
//! table over the existing causal observations minimise complete-stream natural-text loss while
//! preserving present-query emission and absence behaviour? The module provides
//!
//!   * [`position_action_outcomes`]: the four declared actions' counterfactual outcomes at one
//!     position, taken from the **actual integer logits** and cross-checked against the ideal
//!     score-space identity `log(1 + p*(exp(a)-1)) - a*1[payload == target]`;
//!   * a multiple-choice linear program ([`FeasProblem`]) solved by a Lagrangian lower bound plus a
//!     bounded branch-and-bound, reporting a provable bound and an incumbent; and
//!   * an **expected-manifest** artifact loader that verifies byte hashes, fit-input identity and the
//!     configured feature contract before it returns a predictor.
#![forbid(unsafe_code)]

use std::path::Path;
use std::time::Instant;

use super::relational::{
    action_loss, PolicyConfig, RelationalArtifact, RelationalSelector, ACTS, AMP_SHIFTS,
};

// ---------------------------------------------------------------------------
// Actual-integer-path outcomes
// ---------------------------------------------------------------------------

/// Actual integer-path cross-entropy of `target` in **bits** from integer logits at `f_bits` scale.
/// The single `ln` is offline analysis; no served path calls this.
pub fn ce_bits_actual(z: &[i32], target: u32, f_bits: u32) -> f64 {
    let scale = (-(f_bits as f64)).exp2();
    let t = (target as usize).min(z.len().saturating_sub(1));
    let mut max = f64::NEG_INFINITY;
    for &v in z {
        max = max.max(v as f64 * scale);
    }
    let mut sum = 0.0f64;
    for &v in z {
        sum += (v as f64 * scale - max).exp();
    }
    (max + sum.ln() - z[t] as f64 * scale) / std::f64::consts::LN_2
}

/// First argmax under the served integer tie rule (`>` keeps the lowest index).
pub fn argmax_actual(z: &[i32]) -> usize {
    let mut b = 0usize;
    for r in 1..z.len() {
        if z[r] > z[b] {
            b = r;
        }
    }
    b
}

/// Local softmax probability of one token from integer logits at `f_bits` scale.
pub fn prob_of_int(z: &[i32], token: u32, f_bits: u32) -> f64 {
    let scale = (-(f_bits as f64)).exp2();
    let mut max = f64::NEG_INFINITY;
    for &v in z {
        max = max.max(v as f64 * scale);
    }
    let mut sum = 0.0f64;
    let mut want = 0.0f64;
    for (k, &v) in z.iter().enumerate() {
        let e = (v as f64 * scale - max).exp();
        sum += e;
        if k == token as usize {
            want = e;
        }
    }
    want / sum
}

/// One action's counterfactual outcome at one position, from the **actual integer logits**.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ActionOutcome {
    /// Change in next-token CE in bits relative to the pre-boost local logits.
    pub delta_bits: f64,
    /// Emitted token under the served integer tie rule.
    pub emitted: u32,
    /// Whether the emitted token equals the target.
    pub correct: bool,
    /// Read indicator (`action > 0`).
    pub read: bool,
}

/// The `ACTS + 1` declared actions' outcomes at one position: index `0` is NoRead (delta exactly 0
/// by construction). The second return value is the maximum absolute residual between the actual
/// integer delta and the ideal score-space identity in bits, for the caller's per-position assertion.
pub fn position_action_outcomes(
    z_local: &[i32],
    payload: u32,
    target: u32,
    f_bits: u32,
) -> ([ActionOutcome; ACTS + 1], f64) {
    let base = ce_bits_actual(z_local, target, f_bits);
    let p = prob_of_int(z_local, payload, f_bits);
    let mut out = [ActionOutcome::default(); ACTS + 1];
    let mut max_resid = 0.0f64;
    for a in 0..=ACTS {
        let (shift, read) = if a == 0 {
            (0u32, false)
        } else {
            (AMP_SHIFTS[a - 1], true)
        };
        let mut z = z_local.to_vec();
        if read {
            let pi = payload as usize;
            if pi < z.len() {
                z[pi] = z[pi].saturating_add(1i32 << shift);
            }
        }
        let delta_bits = ce_bits_actual(&z, target, f_bits) - base;
        let emitted = argmax_actual(&z) as u32;
        out[a] = ActionOutcome {
            delta_bits,
            emitted,
            correct: emitted == target,
            read,
        };
        // Saturation can make the effective boost smaller than the nominal action. Compare the
        // identity against exactly the integer change served, widening before subtraction.
        let boost = if read {
            z.get(payload as usize)
                .zip(z_local.get(payload as usize))
                .map_or(0.0, |(after, before)| {
                    (i64::from(*after) - i64::from(*before)) as f64 * (-(f_bits as f64)).exp2()
                })
        } else {
            0.0
        };
        let ideal_bits = action_loss(p, boost, payload == target) / std::f64::consts::LN_2;
        max_resid = max_resid.max((delta_bits - ideal_bits).abs());
    }
    (out, max_resid)
}

// ---------------------------------------------------------------------------
// Constrained multiple-choice solve
// ---------------------------------------------------------------------------

/// Signed sense of one declared behavioral constraint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConSense {
    AtLeast,
    AtMost,
}

/// One linear behavioral constraint over the action table: `sum coeff[b][a] x[b,a] (>=|<=) rhs`.
#[derive(Clone, Debug)]
pub struct ConSpec {
    pub name: &'static str,
    pub coeff: Vec<[f64; ACTS + 1]>,
    pub sense: ConSense,
    pub rhs: f64,
    pub tol: f64,
}

/// The finite multiple-choice program: one action per bucket, minimising `obj`.
#[derive(Clone, Debug, Default)]
pub struct FeasProblem {
    /// Objective contribution per `(bucket, action)`; the solver minimises the sum.
    pub obj: Vec<[f64; ACTS + 1]>,
    pub cons: Vec<ConSpec>,
    /// `Some(a)` pins the declared support/fallback opcode for that bucket.
    pub fixed: Vec<Option<usize>>,
    /// Weighted support per bucket (reported only).
    pub support: Vec<f64>,
}

#[derive(Clone, Debug)]
pub struct FeasSolution {
    pub actions: Vec<usize>,
    pub obj: f64,
    pub feasible: bool,
    /// A provable lower bound on the constrained minimum objective.
    pub lower_bound: f64,
    /// `OPTIMAL` | `FEASIBLE_SUBOPTIMAL` | `INFEASIBLE` | `UNRESOLVED`.
    pub status: &'static str,
    pub nodes: u64,
    /// Search completed without hitting a node/time cap.
    pub exhaustive: bool,
    pub seconds: f64,
}

fn normalize(p: &FeasProblem) -> (Vec<Vec<[f64; ACTS + 1]>>, Vec<f64>) {
    let m = p.cons.len();
    let nb = p.obj.len();
    let mut c = vec![vec![[0.0f64; ACTS + 1]; nb]; m];
    let mut r = vec![0.0f64; m];
    for j in 0..m {
        let s = match p.cons[j].sense {
            ConSense::AtMost => 1.0,
            ConSense::AtLeast => -1.0,
        };
        for b in 0..nb {
            for a in 0..=ACTS {
                c[j][b][a] = s * p.cons[j].coeff[b][a];
            }
        }
        // Every search and bound must use the same tolerated feasible set. A lower bound
        // formed from the stricter, unrelaxed RHS can exceed a tolerance-feasible optimum.
        r[j] = s * p.cons[j].rhs + p.cons[j].tol;
    }
    (c, r)
}

fn lb_for(p: &FeasProblem, c: &[Vec<[f64; ACTS + 1]>], r: &[f64], lam: &[f64]) -> f64 {
    let mut tot = 0.0f64;
    for b in 0..p.obj.len() {
        let mut best = f64::INFINITY;
        for a in 0..=ACTS {
            if let Some(f) = p.fixed[b] {
                if a != f {
                    continue;
                }
            }
            let mut v = p.obj[b][a];
            for j in 0..lam.len() {
                v += lam[j] * c[j][b][a];
            }
            if v < best {
                best = v;
            }
        }
        tot += best;
    }
    for j in 0..lam.len() {
        tot -= lam[j] * r[j];
    }
    tot
}

/// Coordinate ascent on the Lagrangian multipliers. Every evaluation is a valid lower bound, so the
/// returned value is a sound bound on the constrained minimum regardless of convergence.
fn best_lambda(
    p: &FeasProblem,
    c: &[Vec<[f64; ACTS + 1]>],
    r: &[f64],
    passes: usize,
) -> (Vec<f64>, f64) {
    let m = r.len();
    let mut lam = vec![0.0f64; m];
    let mut best = lb_for(p, c, r, &lam);
    let mut spread = 0.0f64;
    for b in 0..p.obj.len() {
        let mut mn = f64::INFINITY;
        let mut mx = f64::NEG_INFINITY;
        for a in 0..=ACTS {
            if let Some(f) = p.fixed[b] {
                if a != f {
                    continue;
                }
            }
            mn = mn.min(p.obj[b][a]);
            mx = mx.max(p.obj[b][a]);
        }
        spread = spread.max(mx - mn);
    }
    for _ in 0..passes {
        let mut improved = false;
        for j in 0..m {
            if r[j].abs() <= 1e-12 && c[j].iter().all(|row| row.iter().all(|v| v.abs() <= 1e-12)) {
                continue;
            }
            let mut hi = 0.0f64;
            for b in 0..p.obj.len() {
                for a in 0..=ACTS {
                    hi = hi.max(c[j][b][a].abs());
                }
            }
            if hi <= 1e-12 {
                continue;
            }
            let scale = (spread.abs() + 1.0) / hi * 4.0;
            let saved = lam[j];
            let mut lbest = best;
            let mut bl = saved;
            for k in 0..=64 {
                lam[j] = scale * (k as f64) / 64.0;
                let v = lb_for(p, c, r, &lam);
                if v > lbest + 1e-12 {
                    lbest = v;
                    bl = lam[j];
                }
            }
            let lo = (bl - scale / 64.0).max(0.0);
            let hi2 = bl + scale / 64.0;
            for k in 0..=32 {
                lam[j] = lo + (hi2 - lo) * (k as f64) / 32.0;
                let v = lb_for(p, c, r, &lam);
                if v > lbest + 1e-12 {
                    lbest = v;
                    bl = lam[j];
                }
            }
            lam[j] = bl;
            if lbest > best + 1e-9 {
                best = lbest;
                improved = true;
            } else {
                lam[j] = saved;
            }
        }
        if !improved {
            break;
        }
    }
    (lam, best)
}

fn obj_of(p: &FeasProblem, x: &[usize]) -> f64 {
    let mut o = 0.0f64;
    for b in 0..x.len() {
        o += p.obj[b][x[b]];
    }
    o
}

fn norm_c_of(c: &[Vec<[f64; ACTS + 1]>], j: usize, x: &[usize]) -> f64 {
    let mut v = 0.0f64;
    for b in 0..x.len() {
        v += c[j][b][x[b]];
    }
    v
}

/// Realized value of constraint `j` (in original units) for a table.
pub fn realized(p: &FeasProblem, x: &[usize], j: usize) -> f64 {
    let mut v = 0.0f64;
    for b in 0..x.len() {
        v += p.cons[j].coeff[b][x[b]];
    }
    v
}

fn slack_ok(c: &[Vec<[f64; ACTS + 1]>], r: &[f64], x: &[usize]) -> bool {
    for j in 0..r.len() {
        if norm_c_of(c, j, x) > r[j] {
            return false;
        }
    }
    true
}

fn violation(c: &[Vec<[f64; ACTS + 1]>], r: &[f64], x: &[usize]) -> f64 {
    let mut v = 0.0f64;
    for j in 0..r.len() {
        let over = norm_c_of(c, j, x) - r[j];
        if over > 0.0 {
            v += over / (r[j].abs() + 1.0);
        }
    }
    v
}

/// Greedy repair from a per-bucket objective-minimising start. Deterministic.
fn greedy_incumbent(
    p: &FeasProblem,
    c: &[Vec<[f64; ACTS + 1]>],
    r: &[f64],
    seed: usize,
) -> Vec<usize> {
    let nb = p.obj.len();
    let mut x = vec![0usize; nb];
    for b in 0..nb {
        x[b] = match p.fixed[b] {
            Some(f) => f,
            None => {
                let mut ba = 0usize;
                for a in 1..=ACTS {
                    if p.obj[b][a] < p.obj[b][ba] - 1e-12 {
                        ba = a;
                    }
                }
                ba
            }
        };
    }
    if seed == 1 {
        // Seed every free bucket toward the currently most-violated constraint's best action.
        let mut worst = 0usize;
        let mut wv = 0.0f64;
        for j in 0..r.len() {
            let over = norm_c_of(c, j, &x) - r[j];
            if over > wv {
                wv = over;
                worst = j;
            }
        }
        for b in 0..nb {
            if p.fixed[b].is_some() {
                continue;
            }
            let mut ba = x[b];
            for a in 0..=ACTS {
                if c[worst][b][a] < c[worst][b][ba] - 1e-12 {
                    ba = a;
                }
            }
            x[b] = ba;
        }
    }
    let cap = nb * (ACTS + 1) * 12 + 128;
    for _ in 0..cap {
        if violation(c, r, &x) <= 0.0 {
            break;
        }
        let mut best_move: Option<(usize, usize, f64)> = None;
        let base_v = violation(c, r, &x);
        for b in 0..nb {
            if p.fixed[b].is_some() {
                continue;
            }
            let cur = x[b];
            for a in 0..=ACTS {
                if a == cur {
                    continue;
                }
                let saved = x[b];
                x[b] = a;
                let v = violation(c, r, &x);
                x[b] = saved;
                if v >= base_v {
                    continue;
                }
                let dobj = (p.obj[b][a] - p.obj[b][cur]).max(0.0);
                let score = (base_v - v) / (1.0 + dobj);
                if best_move.map_or(true, |(_, _, s)| score > s) {
                    best_move = Some((b, a, score));
                }
            }
        }
        match best_move {
            Some((b, a, _)) => x[b] = a,
            None => break,
        }
    }
    x
}

struct Bnb<'a> {
    p: &'a FeasProblem,
    c: Vec<Vec<[f64; ACTS + 1]>>,
    r: Vec<f64>,
    free: Vec<usize>,
    lam: Vec<f64>,
    suf_c: Vec<Vec<f64>>,
    best: Option<(Vec<usize>, f64)>,
    nodes: u64,
    node_cap: u64,
    start: Instant,
    time_cap: f64,
    exhaustive: bool,
}

impl<'a> Bnb<'a> {
    fn dfs(&mut self, depth: usize, partial_obj: f64, partial_c: &[f64], x: &mut [usize]) {
        self.nodes += 1;
        if self.nodes > self.node_cap || self.start.elapsed().as_secs_f64() > self.time_cap {
            self.exhaustive = false;
            return;
        }
        if depth == self.free.len() {
            if slack_ok(&self.c, &self.r, x) {
                let o = obj_of(self.p, x);
                if self.best.as_ref().is_none_or(|(_, bo)| o < *bo - 1e-12) {
                    self.best = Some((x.to_vec(), o));
                }
            }
            return;
        }
        // Optimistic feasibility: can the remaining buckets still satisfy every constraint?
        for j in 0..self.r.len() {
            if partial_c[j] + self.suf_c[j][depth] > self.r[j] + 1e-9 {
                return;
            }
        }
        // Objective bound from the Lagrangian on the remaining buckets.
        let mut lb = partial_obj;
        for d in depth..self.free.len() {
            let b = self.free[d];
            let mut bv = f64::INFINITY;
            for a in 0..=ACTS {
                let mut v = self.p.obj[b][a];
                for j in 0..self.lam.len() {
                    v += self.lam[j] * self.c[j][b][a];
                }
                if v < bv {
                    bv = v;
                }
            }
            lb += bv;
        }
        for j in 0..self.lam.len() {
            lb -= self.lam[j] * (self.r[j] - partial_c[j]);
        }
        if let Some((_, bo)) = self.best.as_ref() {
            if lb >= *bo - 1e-12 {
                return;
            }
        }
        let b = self.free[depth];
        let mut order: Vec<usize> = (0..=ACTS).collect();
        order.sort_by(|&a1, &a2| {
            self.p.obj[b][a1]
                .partial_cmp(&self.p.obj[b][a2])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for a in order {
            x[b] = a;
            let mut pc = partial_c.to_vec();
            for j in 0..self.r.len() {
                pc[j] += self.c[j][b][a];
            }
            self.dfs(depth + 1, partial_obj + self.p.obj[b][a], &pc, x);
            if !self.exhaustive {
                return;
            }
        }
    }
}

/// Solve the finite multiple-choice program within a node and wall-time cap.
pub fn solve_feasible(p: &FeasProblem, time_cap_s: f64, node_cap: u64) -> FeasSolution {
    let start = Instant::now();
    let nb = p.obj.len();
    let (c, r) = normalize(p);
    let (lam, lower_bound) = best_lambda(p, &c, &r, 4);
    let free: Vec<usize> = (0..nb).filter(|b| p.fixed[*b].is_none()).collect();
    let nf = free.len();
    // Suffix minima for optimistic feasibility and objective bounds.
    let mut suf_c = vec![vec![0.0f64; nf + 1]; r.len()];
    for d in (0..nf).rev() {
        let b = free[d];
        for j in 0..r.len() {
            let mut mc = f64::INFINITY;
            for a in 0..=ACTS {
                mc = mc.min(c[j][b][a]);
            }
            suf_c[j][d] = suf_c[j][d + 1] + mc;
        }
    }
    // Seed with deterministic greedy incumbents.
    let mut best: Option<(Vec<usize>, f64)> = None;
    for seed in 0..2 {
        let x = greedy_incumbent(p, &c, &r, seed);
        if slack_ok(&c, &r, &x) {
            let o = obj_of(p, &x);
            if best.as_ref().is_none_or(|(_, bo)| o < *bo - 1e-12) {
                best = Some((x, o));
            }
        }
    }
    let mut x0 = vec![0usize; nb];
    for b in 0..nb {
        x0[b] = p.fixed[b].unwrap_or(0);
    }
    let mut pc = vec![0.0f64; r.len()];
    let mut fixed_obj = 0.0f64;
    for b in 0..nb {
        if p.fixed[b].is_some() {
            fixed_obj += p.obj[b][x0[b]];
            for j in 0..r.len() {
                pc[j] += c[j][b][x0[b]];
            }
        }
    }
    let mut bnb = Bnb {
        p,
        c,
        r,
        free,
        lam,
        suf_c,
        best,
        nodes: 0,
        node_cap,
        start,
        time_cap: time_cap_s,
        exhaustive: true,
    };
    // Fixed buckets contribute to every full objective, including each subtree bound.
    // Omitting a negative fixed cost raises the bound and can prune the true optimum.
    bnb.dfs(0, fixed_obj, &pc, &mut x0);
    let feasible = bnb.best.is_some();
    let (actions, obj) = match bnb.best {
        Some((x, o)) => (x, o),
        None => (x0, f64::INFINITY),
    };
    let status = match (feasible, bnb.exhaustive) {
        (true, true) => "OPTIMAL",
        (true, false) => "FEASIBLE_SUBOPTIMAL",
        (false, true) => "INFEASIBLE",
        (false, false) => "UNRESOLVED",
    };
    FeasSolution {
        actions,
        obj,
        feasible,
        lower_bound,
        status,
        nodes: bnb.nodes,
        exhaustive: bnb.exhaustive,
        seconds: start.elapsed().as_secs_f64(),
    }
}

// ---------------------------------------------------------------------------
// Expected-manifest artifact loader
// ---------------------------------------------------------------------------

/// Expected content of one exported selector artifact. The byte hash binds the exact bytes; the
/// fit-input digest binds the data/configuration the policy was fitted from.
#[derive(Clone, Debug)]
pub struct ExpectedArtifact {
    pub name: String,
    pub bytes_sha256: String,
    pub data_digest: [u8; 32],
}

fn expected_sha256_hex(b: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(b);
    let d = h.finalize();
    let mut s = String::with_capacity(64);
    for x in d {
        s.push_str(&format!("{x:02x}"));
    }
    s
}

/// SHA-256 of a byte slice as lowercase hex. Exposed so the caller can build the expected manifest.
pub fn artifact_sha256(bytes: &[u8]) -> String {
    expected_sha256_hex(bytes)
}

/// Resolve one artifact through the **expected manifest** and the existing verified reader. Order:
/// on-disk byte hash, structural load, fit-input identity, configured feature contract, vocabulary.
/// Any drift is a hard error *before* a predictor exists. It never hashes its own output, so the
/// binding stays acyclic.
pub fn load_expected_artifact(
    root: &Path,
    exp: &ExpectedArtifact,
    expect_local: &[u8; 32],
    expect_tokenizer: &[u8; 32],
    cfg: &PolicyConfig,
    vocab: usize,
) -> Result<RelationalSelector, String> {
    let path = root.join("artifacts").join(format!("{}.rlr2", exp.name));
    let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let got = expected_sha256_hex(&bytes);
    if got != exp.bytes_sha256 {
        return Err(format!(
            "artifact {} byte hash {got} != expected {}",
            exp.name, exp.bytes_sha256
        ));
    }
    let art = RelationalArtifact::from_bytes(&bytes, expect_local, expect_tokenizer)?;
    if art.data_digest != exp.data_digest {
        return Err(format!(
            "artifact {} fit-input digest differs from the expected manifest",
            exp.name
        ));
    }
    art.selector.verify_policy_contract(cfg)?;
    art.selector.validate_for_vocab(vocab)?;
    Ok(art.selector)
}

/// Realized outcome for a served `(payload, strength)` choice from the actual integer logits.
/// `strength = None` is NoRead. Used to score the frozen parent/comparator on the same populations.
pub fn realized_outcome(
    z_local: &[i32],
    payload: Option<u32>,
    strength: Option<usize>,
    target: u32,
    f_bits: u32,
) -> ActionOutcome {
    let base = ce_bits_actual(z_local, target, f_bits);
    let mut z = z_local.to_vec();
    let read = payload.is_some() && strength.is_some();
    if let (Some(p), Some(st)) = (payload, strength) {
        let pi = p as usize;
        if pi < z.len() {
            z[pi] = z[pi].saturating_add(1i32 << AMP_SHIFTS[st.min(ACTS - 1)]);
        }
    }
    let delta_bits = ce_bits_actual(&z, target, f_bits) - base;
    let emitted = argmax_actual(&z) as u32;
    ActionOutcome {
        delta_bits,
        emitted,
        correct: emitted == target,
        read,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Independent exhaustive truth for small programs: evaluate original constraint senses and
    // tolerances, without reusing normalization, feasibility checks or bounds from the solver.
    fn exhaustive_optimum(p: &FeasProblem) -> Option<(Vec<usize>, f64)> {
        let mut best: Option<(Vec<usize>, f64)> = None;
        for encoded in 0..(ACTS + 1).pow(p.obj.len() as u32) {
            let mut digits = encoded;
            let mut actions = Vec::new();
            for _ in 0..p.obj.len() {
                actions.push(digits % (ACTS + 1));
                digits /= ACTS + 1;
            }
            if p.fixed
                .iter()
                .zip(&actions)
                .any(|(fixed, action)| fixed.is_some_and(|f| f != *action))
            {
                continue;
            }
            let satisfies = p.cons.iter().all(|con| {
                let value: f64 = actions
                    .iter()
                    .enumerate()
                    .map(|(b, a)| con.coeff[b][*a])
                    .sum();
                match con.sense {
                    ConSense::AtMost => value <= con.rhs + con.tol,
                    ConSense::AtLeast => value >= con.rhs - con.tol,
                }
            });
            if !satisfies {
                continue;
            }
            let objective: f64 = actions.iter().enumerate().map(|(b, a)| p.obj[b][*a]).sum();
            if best.as_ref().is_none_or(|(_, value)| objective < *value) {
                best = Some((actions, objective));
            }
        }
        best
    }

    fn assert_matches_exhaustive(p: &FeasProblem) -> FeasSolution {
        let truth = exhaustive_optimum(p);
        let solved = solve_feasible(p, 5.0, 100_000);
        assert!(solved.exhaustive, "small search must finish: {solved:?}");
        match truth {
            Some((_, objective)) => {
                assert!(solved.feasible, "missed a feasible table: {solved:?}");
                assert_eq!(solved.status, "OPTIMAL");
                assert!(
                    (solved.obj - objective).abs() < 1e-9,
                    "solver {} differs from exhaustive {objective}",
                    solved.obj
                );
                assert!(
                    solved.lower_bound <= objective + 1e-9,
                    "bound {} exceeds exhaustive {objective}",
                    solved.lower_bound
                );
            }
            None => {
                assert!(!solved.feasible, "invented a feasible table: {solved:?}");
                assert_eq!(solved.status, "INFEASIBLE");
            }
        }
        solved
    }

    #[test]
    fn fixed_objective_is_included_in_every_subtree_bound() {
        for fixed_cost in [-100.0, 0.0, 100.0] {
            // Greedy repair first buys two units for one, then three more for three: cost four.
            // The true optimum buys only three units: cost three. A negative fixed contribution
            // must not raise the subtree bound and prune that improvement at the root.
            let p = FeasProblem {
                obj: vec![
                    [0.0, 3.0, 3.0, 3.0],
                    [0.0, 1.0, 1.0, 1.0],
                    [900.0, 900.0, fixed_cost, 900.0],
                ],
                cons: vec![ConSpec {
                    name: "preserved_units",
                    coeff: vec![[0.0, 3.0, 3.0, 3.0], [0.0, 2.0, 2.0, 2.0], [0.0; 4]],
                    sense: ConSense::AtLeast,
                    rhs: 3.0,
                    tol: 0.0,
                }],
                fixed: vec![None, None, Some(2)],
                support: vec![100.0, 100.0, 0.0],
            };
            let solved = assert_matches_exhaustive(&p);
            assert_eq!(solved.obj, fixed_cost + 3.0);
            assert_eq!(solved.actions[1], 0);
            assert_eq!(solved.actions[2], 2);
        }
    }

    #[test]
    fn bounds_and_acceptance_share_the_tolerated_feasible_set() {
        for sense in [ConSense::AtMost, ConSense::AtLeast] {
            let sign = if sense == ConSense::AtMost { 1.0 } else { -1.0 };
            let mut p = FeasProblem {
                obj: vec![[0.0, -1.0, -2.0, -2.5]],
                cons: vec![ConSpec {
                    name: "tolerance_boundary",
                    coeff: vec![[0.0, sign, 1.25 * sign, 1.5 * sign]],
                    sense,
                    rhs: sign,
                    tol: 0.25,
                }],
                fixed: vec![None],
                support: vec![100.0],
            };
            // Action two is feasible exactly at tolerance; action three exceeds it. A dual bound
            // using the strict RHS would incorrectly exclude action two from its feasible set.
            let solved = assert_matches_exhaustive(&p);
            assert_eq!(solved.actions, vec![2]);
            p.cons[0].tol = 0.0;
            assert_eq!(assert_matches_exhaustive(&p).actions, vec![1]);
            p.fixed[0] = Some(2);
            assert_matches_exhaustive(&p);
            p.cons[0].tol = 0.25;
            assert_eq!(assert_matches_exhaustive(&p).actions, vec![2]);
        }
    }

    #[test]
    fn integer_outcomes_match_the_ideal_identity() {
        // A small vocabulary with a clear payload/target structure at f_bits = 10.
        let z: Vec<i32> = vec![0, 3072, -512, 1024, -2048, 256];
        for &(payload, target) in &[(1u32, 1u32), (1, 3), (3, 1), (5, 0)] {
            let (out, resid) = position_action_outcomes(&z, payload, target, 10);
            assert_eq!(out[0].delta_bits, 0.0, "NoRead must not change the loss");
            assert!(!out[0].read);
            // The ideal score-space identity reproduces the integer path up to rounding.
            assert!(resid < 1e-9, "residual {resid} too large");
            for a in 1..=ACTS {
                assert!(out[a].read);
                assert_eq!(out[a].correct, out[a].emitted == target);
            }
        }
    }

    #[test]
    fn saturated_boost_uses_its_effective_integer_change() {
        let z = vec![i32::MAX - 32, i32::MAX - 96];
        for target in [0u32, 1u32] {
            let (out, residual) = position_action_outcomes(&z, 0, target, 10);
            assert!(residual < 1e-9, "saturated identity residual {residual}");
            // Every positive action saturates at MAX, so all three have the same effective
            // 32/1024-nat boost, regardless of their nominal 64/1024, 1 or 8 nats.
            for action in 1..=ACTS {
                assert_eq!(out[action], out[1]);
                assert_eq!(out[action].emitted, 0);
                let ideal = action_loss(prob_of_int(&z, 0, 10), 32.0 / 1024.0, target == 0)
                    / std::f64::consts::LN_2;
                assert!((out[action].delta_bits - ideal).abs() < 1e-9);
            }
        }
        let already_maximal = vec![i32::MAX, i32::MAX - 64];
        let (out, residual) = position_action_outcomes(&already_maximal, 0, 1, 10);
        assert!(residual < 1e-9);
        for action in 1..=ACTS {
            assert_eq!(out[action].delta_bits, 0.0);
            assert!(
                out[action].read,
                "a zero effective boost remains a read action"
            );
        }
    }

    #[test]
    fn solver_finds_the_text_minimum_under_a_preservation_floor() {
        // Two free buckets, four actions. Bucket 0 is the only source of present-correct emissions.
        let mut p = FeasProblem {
            obj: vec![[0.0, -1.0, -0.5, 0.2], [0.0, 0.1, 0.05, -0.4]],
            cons: vec![ConSpec {
                name: "present_correct",
                coeff: vec![[0.0, 0.0, 1.0, 4.0], [0.0, 0.0, 0.0, 0.0]],
                sense: ConSense::AtLeast,
                rhs: 3.0,
                tol: 1e-9,
            }],
            fixed: vec![None, None],
            support: vec![100.0, 100.0],
        };
        let s = solve_feasible(&p, 5.0, 5_000_000);
        assert!(s.feasible, "a feasible table exists: {:?}", s);
        assert!(realized(&p, &s.actions, 0) >= 3.0 - 1e-9);
        // Bucket 0 must read strongly to satisfy the floor; bucket 1 takes its text optimum.
        assert_eq!(s.actions[0], 3);
        assert_eq!(s.actions[1], 3);
        assert!(
            s.lower_bound <= s.obj + 1e-9,
            "bound must not exceed the optimum"
        );

        // Raising the floor beyond what any table can reach proves a fallback obstruction.
        p.cons[0].rhs = 100.0;
        let s2 = solve_feasible(&p, 5.0, 5_000_000);
        assert!(!s2.feasible);
        assert_eq!(s2.status, "INFEASIBLE");
    }
}

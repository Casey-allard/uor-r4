#!/usr/bin/env python3
"""run_router_invariant_state_probe_v1.py

INVARIANT STATE / EIGENSTATE LAW VALIDATION v1

Tests whether the system is better described by an invariant/eigenstate
state law than by a routing law.

BACKGROUND (locked):
  Fixed operator 0 and unrestricted random routing both achieve 1.0000 at D=24.
  Therefore correctness cannot be carried by the routing path itself.
  Candidate: the tau representation occupies an invariant class under
  operator application.

  Two forms tested:
    Fixed-point:  R_n σ ≈ σ          (no change under operation)
    Eigenstate:   R_n σ ≈ λ_n σ      (scaled but direction-preserving)

MEASUREMENTS PER ROUTING VARIANT {exact_k1, fixed_op_0, unrestricted_rand}:

  1. Step-to-step angular alignment: cos_sim(dir(τ_t), dir(τ_{t-1}))
     → high = angular fixed-point holds

  2. Step-to-step scalar eigenvalue estimate:
       λ_t = (τ_t · τ_{t-1}) / ||τ_{t-1}||²  (optimal scalar for τ_t ≈ λ τ_{t-1})

  3. Fixed-point residual:  ||τ_t - τ_{t-1}|| / ||τ_{t-1}||

  4. Eigenstate residual: ||τ_t - λ_t τ_{t-1}|| / ||τ_{t-1}||
     → if this << fixed-point residual: eigenstate is the better model

  5. Alignment of τ_t across routing variants (same initial state, different routes):
     cross_align(t) = cos_sim(τ_t^{k1}, τ_t^{fixed})  and  (τ_t^{k1}, τ_t^{rand})
     → if high: invariant class is routing-independent

  6. Radial part magnitude trajectory: mean ||τ_t[8:]||

  7. Alignment of each τ_t with the final τ_{D-1}: convergence indicator

NOTE on architecture:
  For hard geometric inference (all variants here):
    τ_t[:8] = exact TN entry for chosen op (unit vector — guaranteed)
    τ_t[8:] = ones(B, 4)  (hard geom always sets magnitude to 1.0)
  Therefore radial part is IDENTICALLY 1.0; the interesting dynamics
  are purely in the angular (direction) component.
  Step t=0 is special: τ_0 += W_tok_inject[x0] AFTER the geometric update.

VARIANTS ON SAME INITIAL CONDITIONS (same sids, x0 per batch):
  exact_k1, fixed_op_0, unrestricted_rand
"""
from __future__ import annotations

import csv
import datetime
import math
import sys
import time
from pathlib import Path
from typing import Dict, List, Tuple

import torch
import torch.nn as nn
import torch.nn.functional as F

# ═══════════════════════════════════════════════════════════════════════
# Paths
# ═══════════════════════════════════════════════════════════════════════
SCRIPT_DIR  = Path(__file__).parent
REPO_ROOT   = Path(__file__).resolve().parents[2]
RESULTS_DIR = REPO_ROOT / "results" / "prime_transport_recursive_system"
DOCS_DIR    = REPO_ROOT / "docs" / "research"
CACHE_PATH  = RESULTS_DIR / "state_cache" / "state_tables_full.pt"
CSV_OUT     = RESULTS_DIR / "prime_transport_router_invariant_state_probe_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_router_invariant_state_probe_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Locked config
# ═══════════════════════════════════════════════════════════════════════
GLOBAL_SEED   = 42
D             = 24
D_HIDDEN      = 32
BATCH_SIZE    = 512       # larger for stable statistics
VOCAB         = 4
D_EMB         = 4
D_TAU_ANG     = 8
N_PHASE_PAIRS = 4
D_TAU_HYB     = D_TAU_ANG + N_PHASE_PAIRS   # 12
D_IN_HYB      = D_EMB + D_TAU_HYB            # 16
N_OPS         = 6
LR            = 0.02
TEMP_START    = 2.0
TEMP_END      = 0.1
CLIP_GRAD     = 1.0
EVAL_EVERY    = 500
N_EVAL        = 512
SOLVE_ACC     = 0.999
PHASE_BLOCKS  = [(0, 2, 2), (2, 7, 5), (7, 9, 2), (9, 21, 12)]
INIT_SCALE    = 0.05
B0_INIT       = 2.0
MAX_STEPS     = 3_000
N_MEASURE_BATCHES = 16    # 16 × 512 = 8192 samples for statistics
FAILURE_TH    = 0.99

try:
    sys.path.insert(0, str(SCRIPT_DIR))
    from thread_policy import select_threads as _st
    _st(BATCH_SIZE, D_IN_HYB, D_HIDDEN)
except Exception:
    pass


# ═══════════════════════════════════════════════════════════════════════
# Angular conversion + hybrid tables (locked)
# ═══════════════════════════════════════════════════════════════════════
def convert_onehot_to_angular(onehot):
    shape = onehot.shape[:-1]
    out = torch.zeros(*shape, D_TAU_ANG)
    ai = 0
    for s, e, m in PHASE_BLOCKS:
        k = onehot[..., s:e].argmax(dim=-1).float()
        angle = 2.0 * math.pi * k / float(m)
        out[..., ai]     = torch.cos(angle)
        out[..., ai + 1] = torch.sin(angle)
        ai += 2
    return out


def prepare_hybrid_tables(TN_oh, tau0_oh, TR, pool_ids):
    TN_ang   = convert_onehot_to_angular(TN_oh)
    tau0_ang = convert_onehot_to_angular(tau0_oh)
    tau0_hyb = torch.cat([tau0_ang, torch.ones(tau0_ang.shape[0], N_PHASE_PAIRS)], dim=1)
    return TN_ang, TR, tau0_hyb, pool_ids


# ═══════════════════════════════════════════════════════════════════════
# Model (locked baseline)
# ═══════════════════════════════════════════════════════════════════════
class RouterAngularHybrid(nn.Module):
    def __init__(self, TN_ang, TR, tau0_hyb, pool_ids,
                 d_hidden=D_HIDDEN, d_context=D,
                 b0_init=B0_INIT, init_scale=INIT_SCALE, seed=GLOBAL_SEED):
        super().__init__()
        dh = d_hidden; dha = max(8, dh // 4)
        self.register_buffer("TN", TN_ang)
        self.register_buffer("TR", TR)
        self.register_buffer("tau0_table", tau0_hyb)
        self.register_buffer("pool_ids", pool_ids)
        m = torch.zeros(1, d_context); m[0, 0] = 1.0
        self.register_buffer("pos0_mask", m)
        self.b_pos0 = nn.Parameter(torch.tensor(b0_init))
        gen = torch.Generator().manual_seed(seed)
        def rp(*sh): return nn.Parameter(torch.empty(*sh).normal_(0, init_scale, generator=gen))
        def zp(*sh): return nn.Parameter(torch.zeros(*sh))
        self.W_emb        = rp(VOCAB, D_EMB)
        self.W1           = rp(D_IN_HYB, dh);    self.b1 = zp(dh)
        self.W2           = rp(dh, N_OPS);        self.b2 = zp(N_OPS)
        self.W_attn       = rp(dha, D_TAU_HYB);  self.b_attn = zp(dha); self.v_attn = rp(dha)
        self.W_pred       = rp(D_TAU_HYB, VOCAB); self.b_pred = zp(VOCAB)
        self.W_tok_inject = rp(VOCAB, D_TAU_HYB)

    def forward(self, state_ids, tokens, x0, temp):
        B = state_ids.shape[0]
        tau_prev = self.tau0_table[state_ids]
        soft_taus: List[torch.Tensor] = []
        for t in range(D):
            tn   = self.TN[state_ids]
            embs = self.W_emb[tokens[:, t]]
            h    = torch.tanh(torch.cat([embs, tau_prev], dim=1) @ self.W1 + self.b1)
            logits = h @ self.W2 + self.b2
            if self.training:
                u = torch.rand_like(logits).clamp_(1e-20, 1.0)
                w = torch.softmax((logits - torch.log(-torch.log(u))) / temp, dim=1)
            else:
                w = torch.softmax(logits / 0.05, dim=1)
            base  = torch.einsum("bi,bij->bj", w, tn)
            pairs = base.view(B, N_PHASE_PAIRS, 2)
            mag   = (pairs * pairs).sum(dim=2).sqrt()
            dirn  = (pairs / mag.clamp(min=1e-8).unsqueeze(2)).view(B, D_TAU_ANG)
            hybrid = torch.cat([dirn, mag], dim=1)
            tau_prev = (hybrid + self.W_tok_inject[x0]) if t == 0 else hybrid
            soft_taus.append(tau_prev)
            state_ids = self.TR[state_ids].gather(
                1, torch.argmax(w, dim=1).unsqueeze(1)).squeeze(1)
        st    = torch.stack(soft_taus, dim=1)
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = (h_a * self.v_attn).sum(dim=-1) + self.pos0_mask * self.b_pos0
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred


# ═══════════════════════════════════════════════════════════════════════
# Utilities
# ═══════════════════════════════════════════════════════════════════════
def make_batch(pool_ids, rng):
    idx  = torch.randint(pool_ids.shape[0], (BATCH_SIZE,), generator=rng)
    sids = pool_ids[idx]
    x0   = torch.randint(VOCAB, (BATCH_SIZE,), generator=rng)
    toks = torch.randint(VOCAB, (BATCH_SIZE, D), generator=rng)
    toks[:, 0] = x0
    return sids, toks, x0


def train_model(TN_ang, TR, tau0_hyb, pool_ids):
    model = RouterAngularHybrid(TN_ang, TR, tau0_hyb, pool_ids)
    opt   = torch.optim.SGD(model.parameters(), lr=LR)
    rng_t = torch.Generator().manual_seed(GLOBAL_SEED + 100)
    model.train()
    t0 = time.perf_counter()
    solve_step = None; final_acc = 0.0
    for step in range(1, MAX_STEPS + 1):
        frac   = step / max(MAX_STEPS - 1, 1)
        temp   = float(TEMP_START * (TEMP_END / TEMP_START) ** frac)
        sids, toks, x0 = make_batch(pool_ids, rng_t)
        logits = model(sids, toks, x0, temp)
        loss   = F.cross_entropy(logits, x0)
        opt.zero_grad(); loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), CLIP_GRAD)
        opt.step()
        if step % EVAL_EVERY == 0:
            acc = _eval_acc(model, pool_ids)
            if acc >= SOLVE_ACC and solve_step is None: solve_step = step
            final_acc = acc
            print(f"    step={step:5d}  acc={acc:.4f}")
    wall = time.perf_counter() - t0
    print(f"  Done: acc={final_acc:.4f}  solve={solve_step}  sps={MAX_STEPS/wall:.1f}")
    model.eval()
    return model, final_acc, solve_step


def _eval_acc(model, pool_ids):
    model.eval()
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 200)
    correct = 0
    with torch.no_grad():
        for _ in range(N_EVAL // BATCH_SIZE):
            sids, toks, x0 = make_batch(pool_ids, rng)
            correct += (model(sids, toks, x0, 0.05).argmax(1) == x0).sum().item()
    model.train()
    return correct / N_EVAL


# ═══════════════════════════════════════════════════════════════════════
# Hard geometric forward — collect per-step tau trajectory
# ═══════════════════════════════════════════════════════════════════════
def _hard_geom_taus(model, sids, x0, variant: str, rng=None) -> Tuple[List[torch.Tensor], torch.Tensor]:
    """
    Run one batch through the hard geometric inference under the given routing variant.
    Returns:
      taus: List[Tensor(B, 12)] — tau at each step t=0..D-1 (AFTER tok_inject on t=0)
      pred_logits: Tensor(B, VOCAB)
    """
    B = sids.shape[0]
    tau_prev  = model.tau0_table[sids]
    sids_loop = sids.clone()
    taus: List[torch.Tensor] = []

    for t in range(D):
        tn      = model.TN[sids_loop]                          # (B, 6, 8)
        cur_dir = tau_prev[:, :D_TAU_ANG]                      # (B, 8)
        ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)      # (B, 6)

        if variant == "exact_k1":
            chosen = ang_sim.argmax(dim=1)
        elif variant == "fixed_op_0":
            chosen = torch.zeros(B, dtype=torch.long)
        elif variant == "unrestricted_rand":
            chosen = torch.randint(N_OPS, (B,), generator=rng)
        else:
            raise ValueError(f"Unknown variant: {variant}")

        best_ang = tn.gather(
            1, chosen.view(B, 1, 1).expand(B, 1, D_TAU_ANG)
        ).squeeze(1)                                            # (B, 8) unit vector
        hybrid   = torch.cat([best_ang, torch.ones(B, N_PHASE_PAIRS)], dim=1)  # (B, 12)
        tau_cur  = (hybrid + model.W_tok_inject[x0]) if t == 0 else hybrid
        taus.append(tau_cur.clone())
        tau_prev  = tau_cur
        sids_loop = model.TR[sids_loop].gather(1, chosen.unsqueeze(1)).squeeze(1)

    # Prediction
    st    = torch.stack(taus, dim=1)                           # (B, D, 12)
    h_a   = torch.tanh(st @ model.W_attn.t() + model.b_attn)
    sc    = (h_a * model.v_attn).sum(dim=-1) + model.pos0_mask * model.b_pos0
    alpha = torch.softmax(sc, dim=1)
    pred  = torch.einsum("bd,bdt->bt", alpha, st) @ model.W_pred + model.b_pred
    return taus, pred


# ═══════════════════════════════════════════════════════════════════════
# Invariant statistics computation
# ═══════════════════════════════════════════════════════════════════════
def _cosine_sim_batch(a: torch.Tensor, b: torch.Tensor) -> torch.Tensor:
    """Batch cosine similarity. a, b: (B, d)."""
    na = a.norm(dim=1, keepdim=True).clamp(min=1e-8)
    nb = b.norm(dim=1, keepdim=True).clamp(min=1e-8)
    return (a / na * (b / nb)).sum(dim=1)   # (B,)


def compute_invariant_stats(taus_prev: torch.Tensor, taus_curr: torch.Tensor) -> Dict:
    """
    Given previous and current tau tensors (B, 12), compute invariant statistics.

    Angular part: taus[:, :8]  — 4 concatenated 2D unit pairs (norm = 2, NOT 1)
    Radial part:  taus[:, 8:]  — magnitude per phase pair (always 1.0 for hard geom)

    NOTE: tau[:8] = [cos θ₁, sin θ₁, cos θ₂, sin θ₂, cos θ₃, sin θ₃, cos θ₄, sin θ₄]
    Each pair is unit length, so the 8D vector has norm = sqrt(4) = 2.
    Cosine similarity must normalise by this norm (= 2) explicitly.

    Returns dict of mean scalars over the batch.
    """
    B = taus_prev.shape[0]

    # Angular alignment: normalised cosine similarity on the 8D directional part
    ang_prev = taus_prev[:, :D_TAU_ANG]                       # (B, 8)
    ang_curr = taus_curr[:, :D_TAU_ANG]
    # Proper cosine similarity (normalise each vector)
    ang_align = _cosine_sim_batch(ang_prev, ang_curr)          # (B,), range [-1, 1]

    # Radial part analysis
    rad_prev = taus_prev[:, D_TAU_ANG:]                        # (B, 4)
    rad_curr = taus_curr[:, D_TAU_ANG:]
    rad_norm_prev = rad_prev.norm(dim=1).clamp(min=1e-8)       # (B,)
    rad_norm_curr = rad_curr.norm(dim=1)
    radial_scale  = rad_norm_curr / rad_norm_prev               # candidate λ for radial part

    # Full tau fixed-point residual: ||τ_t - τ_{t-1}|| / ||τ_{t-1}||
    norm_prev = taus_prev.norm(dim=1).clamp(min=1e-8)
    fp_residual = (taus_curr - taus_prev).norm(dim=1) / norm_prev  # (B,)

    # Eigenstate lambda estimate: λ = (τ_t · τ_{t-1}) / ||τ_{t-1}||²
    dot_product = (taus_curr * taus_prev).sum(dim=1)            # (B,)
    lambda_est  = dot_product / (norm_prev ** 2)                # (B,)

    # Eigenstate residual: ||τ_t - λ τ_{t-1}|| / ||τ_{t-1}||
    scaled_prev = taus_prev * lambda_est.unsqueeze(1)
    es_residual = (taus_curr - scaled_prev).norm(dim=1) / norm_prev  # (B,)

    # Full cosine similarity (whole tau vector)
    full_cosim = _cosine_sim_batch(taus_prev, taus_curr)        # (B,)

    return {
        "angular_alignment":      float(ang_align.mean()),
        "radial_scale_lambda":    float(radial_scale.mean()),
        "full_cosine_sim":        float(full_cosim.mean()),
        "fp_residual":            float(fp_residual.mean()),
        "lambda_est":             float(lambda_est.mean()),
        "lambda_est_std":         float(lambda_est.std()),
        "es_residual":            float(es_residual.mean()),
        "rad_norm_curr":          float(rad_norm_curr.mean()),
    }


# ═══════════════════════════════════════════════════════════════════════
# Main measurement loop
# ═══════════════════════════════════════════════════════════════════════
def measure_variant(model, pool_ids, variant: str) -> Tuple[List[Dict], float]:
    """
    Collect per-step invariant statistics for one routing variant.
    Returns (step_stats_list, accuracy).
    """
    # Per-step accumulators
    accum: List[Dict] = [{
        "angular_alignment": 0.0,
        "radial_scale_lambda": 0.0,
        "full_cosine_sim": 0.0,
        "fp_residual": 0.0,
        "lambda_est": 0.0,
        "lambda_est_std": 0.0,
        "es_residual": 0.0,
        "rad_norm_curr": 0.0,
        "tau_norm": 0.0,              # ||τ_t||
        "align_to_tau0": 0.0,         # cos_sim(τ_t[:8], τ_0[:8]) — stability to initial state
        "align_to_tauD": 0.0,         # will fill in second pass
    } for _ in range(D)]

    correct = 0
    total   = 0
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 888)

    with torch.no_grad():
        for batch_i in range(N_MEASURE_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, rng)
            taus, pred = _hard_geom_taus(model, sids, x0, variant, rng=rng)
            correct += (pred.argmax(1) == x0).sum().item()
            total   += BATCH_SIZE

            tau_stack = torch.stack(taus, dim=1)      # (B, D, 12)
            tau0_dir  = taus[0][:, :D_TAU_ANG]        # (B, 8) — reference direction at t=0
            tauD_dir  = taus[-1][:, :D_TAU_ANG]       # (B, 8) — reference direction at t=D-1

            for t in range(D):
                tau_t = taus[t]
                accum[t]["tau_norm"]      += float(tau_t.norm(dim=1).mean())
                accum[t]["rad_norm_curr"] += float(tau_t[:, D_TAU_ANG:].norm(dim=1).mean())

                # Align to tau_0 direction (angular stability)
                dir_t = tau_t[:, :D_TAU_ANG]
                accum[t]["align_to_tau0"] += float(_cosine_sim_batch(dir_t, tau0_dir).mean())
                accum[t]["align_to_tauD"] += float(_cosine_sim_batch(dir_t, tauD_dir).mean())

                if t > 0:
                    stats = compute_invariant_stats(taus[t-1], tau_t)
                    for k, v in stats.items():
                        if k in accum[t]:
                            accum[t][k] += v

    # Average over batches
    step_stats = []
    for t in range(D):
        row = {k: v / N_MEASURE_BATCHES for k, v in accum[t].items()}
        row["step"] = t
        row["variant"] = variant
        step_stats.append(row)

    accuracy = round(correct / total, 4)
    return step_stats, accuracy


def measure_cross_variant_alignment(model, pool_ids) -> List[Dict]:
    """
    Run exact_k1, fixed_op_0, unrestricted_rand on the SAME initial batch.
    Measure cross-variant tau alignment at each step.
    """
    results = []
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 5555)

    # Per-step accumulators
    align_k1_fixed = [0.0] * D
    align_k1_rand  = [0.0] * D
    align_fixed_rand = [0.0] * D

    with torch.no_grad():
        for _ in range(N_MEASURE_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, rng)
            rng_r = torch.Generator().manual_seed(int(rng.initial_seed()) + 1)

            taus_k1,    _ = _hard_geom_taus(model, sids, x0, "exact_k1")
            taus_fixed, _ = _hard_geom_taus(model, sids, x0, "fixed_op_0")
            taus_rand,  _ = _hard_geom_taus(model, sids, x0, "unrestricted_rand", rng=rng_r)

            for t in range(D):
                dir_k1    = taus_k1[t][:, :D_TAU_ANG]
                dir_fixed = taus_fixed[t][:, :D_TAU_ANG]
                dir_rand  = taus_rand[t][:, :D_TAU_ANG]

                # Proper cosine similarity (normalise each 8D vector)
                align_k1_fixed[t]   += float(_cosine_sim_batch(dir_k1, dir_fixed).mean())
                align_k1_rand[t]    += float(_cosine_sim_batch(dir_k1, dir_rand).mean())
                align_fixed_rand[t] += float(_cosine_sim_batch(dir_fixed, dir_rand).mean())

    for t in range(D):
        results.append({
            "step":               t,
            "align_k1_fixed":     round(align_k1_fixed[t] / N_MEASURE_BATCHES, 4),
            "align_k1_rand":      round(align_k1_rand[t] / N_MEASURE_BATCHES, 4),
            "align_fixed_rand":   round(align_fixed_rand[t] / N_MEASURE_BATCHES, 4),
        })
    return results


# ═══════════════════════════════════════════════════════════════════════
# Report helpers
# ═══════════════════════════════════════════════════════════════════════
def _summarize_variant(step_stats: List[Dict], label: str) -> Dict:
    """Compute summary statistics averaged over steps 1..D-1 (skip step 0 for transitions)."""
    rows = [s for s in step_stats if s["step"] >= 1]
    def mean(k): return round(sum(r[k] for r in rows) / len(rows), 4)
    return {
        "variant": label,
        "mean_angular_alignment":  mean("angular_alignment"),
        "mean_fp_residual":        mean("fp_residual"),
        "mean_es_residual":        mean("es_residual"),
        "mean_lambda_est":         mean("lambda_est"),
        "mean_lambda_std":         mean("lambda_est_std"),
        "mean_align_to_tau0":      mean("align_to_tau0"),
        "mean_align_to_tauD":      mean("align_to_tauD"),
    }


def _fp_better(summary: Dict) -> bool:
    """True if fixed-point residual is meaningfully lower than eigenstate residual."""
    return summary["mean_fp_residual"] < summary["mean_es_residual"] * 0.8


def _es_better(summary: Dict) -> bool:
    """True if eigenstate explains more than fixed-point (es_residual < fp_residual)."""
    return summary["mean_es_residual"] < summary["mean_fp_residual"] * 0.8


# ═══════════════════════════════════════════════════════════════════════
# Markdown writer
# ═══════════════════════════════════════════════════════════════════════
def _write_markdown(summaries, cross_align_rows, variant_accs, step_stats_all, conclusion):
    ts = datetime.datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ")
    L: List[str] = []
    L.append("# Prime Transport Router: Invariant State / Eigenstate Law Validation v1\n\n")
    L.append(f"**Generated:** {ts}  \n")
    L.append(f"**Config:** D={D}, D_HIDDEN={D_HIDDEN}, B_measure={N_MEASURE_BATCHES*BATCH_SIZE}, CPU  \n\n")

    L.append("---\n\n")
    L.append("## Setup\n\n")
    L.append("Three routing variants run on identical initial conditions (same sids, x0):\n\n")
    L.append("| variant | routing law |\n|---|---|\n")
    L.append("| exact_k1 | argmax angular similarity |\n")
    L.append("| fixed_op_0 | always operator 0 |\n")
    L.append("| unrestricted_rand | uniform random over all 6 ops |\n\n")
    L.append("**Note on architecture:**  \n")
    L.append("For all hard-geometric variants, τ[:8] = 4 concatenated 2D unit pairs "
             "(cos θᵢ, sin θᵢ) — each pair is a 2D unit vector, so the full 8D vector "
             "has norm = 2.0, NOT 1.0. All angular alignments are proper cosine similarities "
             "(normalised by vector norm). τ[8:] = ones (magnitude identically 1.0 by construction). "
             "Therefore the radial λ is trivially 1.0 and all dynamics are in the angular "
             "(S¹×S¹×S¹×S¹) component.  \n\n")
    L.append("**Definitions:**  \n")
    L.append("- `angular_alignment(t)` = dot(τ_t[:8], τ_{t-1}[:8])  (unit vectors → = cosine sim)  \n")
    L.append("- `lambda_est(t)` = (τ_t · τ_{t-1}) / ||τ_{t-1}||²  (optimal eigenstate scalar)  \n")
    L.append("- `fp_residual(t)` = ||τ_t - τ_{t-1}|| / ||τ_{t-1}||  (fixed-point error)  \n")
    L.append("- `es_residual(t)` = ||τ_t - λ_t τ_{t-1}|| / ||τ_{t-1}||  (eigenstate error)  \n\n")

    L.append("---\n\n")
    L.append("## Per-Variant Summary (averaged over steps 1..D-1)\n\n")
    L.append("| variant | acc | ang_align | fp_residual | es_residual | λ_mean | λ_std | align_to_τ₀ |\n")
    L.append("|---------|-----|-----------|-------------|-------------|--------|-------|-------------|\n")
    for s, acc in zip(summaries, variant_accs):
        L.append(f"| {s['variant']} | {acc:.4f} "
                 f"| {s['mean_angular_alignment']:.4f} "
                 f"| {s['mean_fp_residual']:.4f} "
                 f"| {s['mean_es_residual']:.4f} "
                 f"| {s['mean_lambda_est']:.4f} "
                 f"| {s['mean_lambda_std']:.4f} "
                 f"| {s['mean_align_to_tau0']:.4f} |\n")
    L.append("\n")

    L.append("---\n\n")
    L.append("## Cross-Variant Angular Alignment (same initial state)\n\n")
    L.append("Measures whether different routing laws converge to the same tau direction.  \n\n")
    L.append("| step | k1–fixed | k1–rand | fixed–rand |\n")
    L.append("|------|----------|---------|------------|\n")
    for r in cross_align_rows[::4]:  # every 4th step for readability
        L.append(f"| {r['step']:2d} | {r['align_k1_fixed']:.4f} "
                 f"| {r['align_k1_rand']:.4f} "
                 f"| {r['align_fixed_rand']:.4f} |\n")
    L.append("\n")

    # Summarise cross-variant
    mean_kf  = round(sum(r["align_k1_fixed"] for r in cross_align_rows) / D, 4)
    mean_kr  = round(sum(r["align_k1_rand"] for r in cross_align_rows) / D, 4)
    mean_fr  = round(sum(r["align_fixed_rand"] for r in cross_align_rows) / D, 4)
    L.append(f"Mean cross-variant alignment: k1–fixed={mean_kf:.4f}  "
             f"k1–rand={mean_kr:.4f}  fixed–rand={mean_fr:.4f}  \n\n")
    if min(mean_kf, mean_kr, mean_fr) > 0.8:
        L.append("→ High cross-variant alignment: different routing laws produce "
                 "strongly correlated tau directions. The angular invariant class is "
                 "routing-independent.  \n\n")
    elif min(mean_kf, mean_kr, mean_fr) > 0.3:
        L.append("→ Moderate cross-variant alignment: some tau structure is shared "
                 "across routing laws, but paths are not identical.  \n\n")
    else:
        L.append("→ Low cross-variant alignment: routing laws produce independent tau "
                 "trajectories. If accuracy is still 1.0, the invariant must be "
                 "at the prediction level (attention mechanism), not tau level.  \n\n")

    L.append("---\n\n")
    L.append("## Fixed-Point vs Eigenstate Comparison\n\n")
    for s in summaries:
        L.append(f"**[{s['variant']}]**  \n")
        L.append(f"- angular_alignment = {s['mean_angular_alignment']:.4f} "
                 f"({'≈1 (near fixed-point)' if s['mean_angular_alignment'] > 0.9 else 'below 0.9'})  \n")
        L.append(f"- fp_residual = {s['mean_fp_residual']:.4f}  "
                 f"es_residual = {s['mean_es_residual']:.4f}  \n")
        if _fp_better(s):
            L.append("  → Fixed-point is better than eigenstate model (fp < es).  \n")
        elif _es_better(s):
            L.append("  → Eigenstate model (λ scaling) reduces residual vs fixed-point.  \n")
        else:
            L.append("  → Fixed-point and eigenstate models are equivalent (within 20%).  \n")
        L.append(f"- λ_mean = {s['mean_lambda_est']:.4f} ± {s['mean_lambda_std']:.4f}  \n")
        if abs(s["mean_lambda_est"] - 1.0) < 0.05 and s["mean_lambda_std"] < 0.1:
            L.append("  → λ ≈ 1.0 (consistent with fixed-point law)  \n")
        elif s["mean_lambda_est"] > 0.5:
            L.append(f"  → λ ≈ {s['mean_lambda_est']:.3f} (eigenstate scaling; not a fixed-point)  \n")
        else:
            L.append(f"  → λ = {s['mean_lambda_est']:.3f} (weak eigenstate; states decorrelate)  \n")
        L.append(f"- align_to_τ₀ = {s['mean_align_to_tau0']:.4f} "
                 f"(angular coherence with initial state τ₀)  \n\n")

    L.append("---\n\n")
    L.append("## Explicit Answers\n\n")
    L.append("**1. Is a fixed-point model plausible?**  \n")
    fp_plaus = all(s["mean_angular_alignment"] > 0.8 for s in summaries)
    if fp_plaus:
        ang_mean = round(sum(s["mean_angular_alignment"] for s in summaries)/len(summaries), 4)
        L.append(f"Partially. Mean angular alignment across all variants = {ang_mean:.4f}. "
                 f"Direction is largely preserved step-to-step, but full tau vectors change "
                 f"(fp_residual > 0), so the full-vector fixed-point does not hold exactly.  \n\n")
    else:
        L.append("No. Angular alignment is too low for a fixed-point model.  \n\n")

    L.append("**2. Is an eigenstate model more plausible?**  \n")
    es_wins = sum(1 for s in summaries if _es_better(s))
    if es_wins > 0:
        L.append(f"Yes for {es_wins}/{len(summaries)} variants: eigenstate residual is "
                 f"meaningfully lower than fixed-point residual.  \n\n")
    else:
        ang_mean = round(sum(s["mean_angular_alignment"] for s in summaries)/len(summaries), 4)
        lam_mean = round(sum(s["mean_lambda_est"] for s in summaries)/len(summaries), 4)
        L.append(f"Only weakly: λ_mean ≈ {lam_mean:.4f}, angular_alignment ≈ {ang_mean:.4f}. "
                 f"The eigenstate scalar is near 1.0, so both models are nearly equivalent.  \n")
        L.append(f"The system is consistent with a UNIT EIGENSTATE (λ≈1, angular fixed-point)  \n"
                 f"but the full tau vector does change between steps.  \n\n")

    L.append("**3. What is the minimum testable invariant law supported by current data?**  \n")
    L.append(f"See conclusion line below.  \n\n")

    L.append("---\n\n")
    L.append("## Honesty Section\n\n")
    L.append("**What is supported:**  \n")
    L.append("- Angular direction is partially preserved step-to-step (alignment > 0).  \n")
    L.append("- The prediction is determined at τ₀ (position-0 attention bias), not by the "
             "full trajectory — confirmed by fixed_op_0 and random routing both achieving 1.0000.  \n")
    L.append("- The radial magnitude is IDENTICALLY 1.0 for all hard-geometric variants "
             "(by construction), so no radial eigenvalue exists to estimate.  \n\n")
    L.append("**What is not yet supported:**  \n")
    L.append("- A true eigenstate in the algebraic sense requires R_n σ = λ_n σ for a "
             "linear operator R_n. The discrete state transitions TR are not linear operators "
             "on the continuous tau space, so this form cannot be verified from transition data alone.  \n")
    L.append("- Whether the angular invariant class is shared across all routing laws depends "
             "on the cross-variant tau alignment. If alignment is low, correctness is carried "
             "by the attention mechanism selecting τ₀, not by any geometric invariant of the path.  \n\n")
    L.append("**What next measurement would most cleanly distinguish the forms:**  \n")
    L.append("- Measure alignment of τ₀ (post tok_inject) across routing variants on the same "
             "initial state. If all routing laws yield identical τ₀, the answer is fixed by "
             "τ₀ alone and the routing path is irrelevant from step 1 onward.  \n")
    L.append("- Test whether setting τ_t = τ₀ for all t > 0 (pure τ₀ predictor) yields "
             "1.0000 accuracy — that would confirm the attention mechanism collapses to a "
             "fixed-point at position 0 and the subsequent trajectory carries no information.  \n\n")

    L.append("---\n\n")
    L.append(f"## BEST CURRENT STATE LAW: {conclusion}\n\n")

    with open(MD_OUT, "w") as f:
        f.writelines(L)


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main():
    print("=" * 60)
    print("INVARIANT STATE / EIGENSTATE LAW VALIDATION v1")
    print("=" * 60)

    print("\nLoading cache ...")
    t0 = time.perf_counter()
    data = torch.load(CACHE_PATH, weights_only=False)
    print(f"  {data['TN_oh'].shape[0]:,} states in {time.perf_counter()-t0:.3f}s")
    TN_ang, TR, tau0_hyb, pool_ids = prepare_hybrid_tables(
        data["TN_oh"], data["tau0_oh"], data["TR"], data["pool_ids"]
    )

    print("\nTraining D=24 model ...")
    model, train_acc, solve_step = train_model(TN_ang, TR, tau0_hyb, pool_ids)

    VARIANTS = ["exact_k1", "fixed_op_0", "unrestricted_rand"]

    print(f"\n{'='*60}")
    print(f"MEASURING INVARIANT STATISTICS  ({N_MEASURE_BATCHES}×{BATCH_SIZE}={N_MEASURE_BATCHES*BATCH_SIZE} samples)")
    print(f"{'='*60}")

    all_step_stats: Dict[str, List[Dict]] = {}
    summaries = []
    variant_accs = []

    for variant in VARIANTS:
        print(f"\n  Variant: {variant}")
        t_v = time.perf_counter()
        step_stats, accuracy = measure_variant(model, pool_ids, variant)
        wall_v = time.perf_counter() - t_v
        all_step_stats[variant] = step_stats

        s = _summarize_variant(step_stats, variant)
        summaries.append(s)
        variant_accs.append(accuracy)

        print(f"    acc={accuracy:.4f}  wall={wall_v:.1f}s")
        print(f"    ang_align={s['mean_angular_alignment']:.4f}  "
              f"fp_err={s['mean_fp_residual']:.4f}  "
              f"es_err={s['mean_es_residual']:.4f}  "
              f"λ={s['mean_lambda_est']:.4f}±{s['mean_lambda_std']:.4f}  "
              f"align_τ₀={s['mean_align_to_tau0']:.4f}")

    print(f"\n  Cross-variant alignment (same initial state) ...")
    t_c = time.perf_counter()
    cross_align_rows = measure_cross_variant_alignment(model, pool_ids)
    print(f"    wall={time.perf_counter()-t_c:.1f}s")
    mean_kf = round(sum(r["align_k1_fixed"] for r in cross_align_rows) / D, 4)
    mean_kr = round(sum(r["align_k1_rand"] for r in cross_align_rows) / D, 4)
    mean_fr = round(sum(r["align_fixed_rand"] for r in cross_align_rows) / D, 4)
    print(f"    k1–fixed={mean_kf:.4f}  k1–rand={mean_kr:.4f}  fixed–rand={mean_fr:.4f}")

    # Determine conclusion
    ang_mean = sum(s["mean_angular_alignment"] for s in summaries) / len(summaries)
    lam_mean = sum(s["mean_lambda_est"] for s in summaries) / len(summaries)
    es_wins  = sum(1 for s in summaries if _es_better(s))
    cross_high = min(mean_kf, mean_kr, mean_fr) > 0.7

    if abs(lam_mean - 1.0) < 0.05 and ang_mean > 0.8:
        if cross_high:
            conclusion = "UNIT-EIGENSTATE ANGULAR FIXED-POINT (λ≈1, routing-invariant)"
        else:
            conclusion = "UNIT-EIGENSTATE ANGULAR FIXED-POINT (λ≈1, per-variant; paths diverge)"
    elif es_wins >= 2:
        conclusion = "EIGENSTATE (λ≠1 scaling; eigenstate model outperforms fixed-point)"
    elif ang_mean > 0.7:
        conclusion = "PARTIAL ANGULAR FIXED-POINT (direction partially preserved; full state changes)"
    else:
        conclusion = "INCONCLUSIVE (low alignment; correctness carried by attention at τ₀, not trajectory invariant)"

    print(f"\nBEST CURRENT STATE LAW: {conclusion}")

    # CSV output — per-step rows for all variants + cross-variant
    fieldnames = [
        "variant", "step",
        "angular_alignment", "radial_scale_lambda", "full_cosine_sim",
        "fp_residual", "lambda_est", "lambda_est_std", "es_residual",
        "rad_norm_curr", "tau_norm", "align_to_tau0", "align_to_tauD",
        "cross_align_k1_fixed", "cross_align_k1_rand", "cross_align_fixed_rand",
        "accuracy", "note",
    ]
    csv_rows = []
    # Build cross-align lookup
    cross_lookup = {r["step"]: r for r in cross_align_rows}

    for variant, step_stats in all_step_stats.items():
        acc = variant_accs[VARIANTS.index(variant)]
        for t_row in step_stats:
            cr = cross_lookup.get(t_row["step"], {})
            row = {
                "variant":               variant,
                "step":                  t_row["step"],
                "angular_alignment":     round(t_row["angular_alignment"], 5),
                "radial_scale_lambda":   round(t_row["radial_scale_lambda"], 5),
                "full_cosine_sim":       round(t_row["full_cosine_sim"], 5),
                "fp_residual":           round(t_row["fp_residual"], 5),
                "lambda_est":            round(t_row["lambda_est"], 5),
                "lambda_est_std":        round(t_row["lambda_est_std"], 5),
                "es_residual":           round(t_row["es_residual"], 5),
                "rad_norm_curr":         round(t_row["rad_norm_curr"], 5),
                "tau_norm":              round(t_row["tau_norm"], 5),
                "align_to_tau0":         round(t_row["align_to_tau0"], 5),
                "align_to_tauD":         round(t_row["align_to_tauD"], 5),
                "cross_align_k1_fixed":  cr.get("align_k1_fixed", ""),
                "cross_align_k1_rand":   cr.get("align_k1_rand", ""),
                "cross_align_fixed_rand":cr.get("align_fixed_rand", ""),
                "accuracy":              acc,
                "note":                  f"D={D} hard_geom_{variant}",
            }
            csv_rows.append(row)

    with open(CSV_OUT, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(csv_rows)
    print(f"\nCSV written: {CSV_OUT}  ({len(csv_rows)} rows)")

    _write_markdown(summaries, cross_align_rows, variant_accs, all_step_stats, conclusion)
    print(f"Markdown written: {MD_OUT}")


if __name__ == "__main__":
    main()

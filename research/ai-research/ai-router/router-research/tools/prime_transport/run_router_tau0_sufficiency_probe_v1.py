#!/usr/bin/env python3
"""run_router_tau0_sufficiency_probe_v1.py

TAU0 SUFFICIENCY PROBE v1

Determines whether τ₀ alone carries the prediction, and whether
τ_t for t>0 contribute any measurable information at D=24.

LOCKED CONTEXT:
  - Routing choice is unnecessary (fixed_op_0 and random both achieve 1.0)
  - Fixed-point/eigenstate trajectory laws not supported (cross-variant tau alignment ≈ 0)
  - Best interpretation: correctness lives in τ₀ (position-0 state, set before routing)
  - τ₀ = TN_entry + W_tok_inject[x0], then attended via b_pos0 bias

PROCEDURE:
  1. Train D=24 model (standard soft scaffold)
  2. Collect full tau trajectory via exact_k1 hard routing
  3. Apply READOUT ABLATIONS on the collected trajectory (no routing changes):

  ABLATIONS ON READOUT (attention + prediction layer):
    reference      — full trajectory, full attention (baseline)
    tau0_direct    — pred = τ₀ @ W_pred + b_pred (bypass attention entirely)
    tau0_repeated  — replace all τ_t (t>0) with τ₀; run full attention
    no_tau0        — mask position 0 (force alpha[0]=0); predict from t≥1 only
    early_2        — use only positions 0,1; mask rest to -∞
    late_half      — use only positions D//2..D-1; mask first half
    random_t_gt_0  — replace τ_t (t>0) with random tau table entries; τ₀ intact

  Also run: ablations on SOFT MLP trajectory (same readout ablations, soft routing)

  MEASURE PER VARIANT:
    - accuracy, delta_vs_reference, runtime
    - mean alpha[0] (attention weight on position 0)
    - ablated_component description

ALL VARIANTS ARE INFERENCE-ONLY. No retraining.
"""
from __future__ import annotations

import csv
import datetime
import math
import sys
import time
from pathlib import Path
from typing import Dict, List, Optional, Tuple

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
CSV_OUT     = RESULTS_DIR / "prime_transport_router_tau0_sufficiency_probe_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_router_tau0_sufficiency_probe_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Locked config
# ═══════════════════════════════════════════════════════════════════════
GLOBAL_SEED   = 42
D             = 24
D_HIDDEN      = 32
BATCH_SIZE    = 512
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
N_EVAL        = 2048
SOLVE_ACC     = 0.999
PHASE_BLOCKS  = [(0, 2, 2), (2, 7, 5), (7, 9, 2), (9, 21, 12)]
INIT_SCALE    = 0.05
B0_INIT       = 2.0
MAX_STEPS     = 3_000
N_MEASURE_BATCHES = N_EVAL // BATCH_SIZE    # 4 batches × 512 = 2048

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
                 b0_init=B0_INIT, init_scale=INIT_SCALE, seed=GLOBAL_SEED):
        super().__init__()
        dh = D_HIDDEN; dha = max(8, dh // 4)
        self.register_buffer("TN", TN_ang)
        self.register_buffer("TR", TR)
        self.register_buffer("tau0_table", tau0_hyb)
        self.register_buffer("pool_ids", pool_ids)
        m = torch.zeros(1, D); m[0, 0] = 1.0
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

    def readout(self, st: torch.Tensor) -> Tuple[torch.Tensor, torch.Tensor]:
        """Apply attention readout to a (B, D, 12) tau stack. Returns pred, alpha."""
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = (h_a * self.v_attn).sum(dim=-1) + self.pos0_mask * self.b_pos0
        alpha = torch.softmax(sc, dim=1)
        pred  = torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred
        return pred, alpha

    def readout_masked(self, st: torch.Tensor, mask: torch.Tensor) -> Tuple[torch.Tensor, torch.Tensor]:
        """
        Readout with additive mask on attention scores before softmax.
        mask: (1, D) or (B, D) — add to scores (use -1e9 to force alpha=0 at positions).
        """
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = (h_a * self.v_attn).sum(dim=-1) + self.pos0_mask * self.b_pos0 + mask
        alpha = torch.softmax(sc, dim=1)
        pred  = torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred
        return pred, alpha


# ═══════════════════════════════════════════════════════════════════════
# Training + basic eval
# ═══════════════════════════════════════════════════════════════════════
def make_batch(pool_ids, rng):
    idx  = torch.randint(pool_ids.shape[0], (BATCH_SIZE,), generator=rng)
    sids = pool_ids[idx]
    x0   = torch.randint(VOCAB, (BATCH_SIZE,), generator=rng)
    toks = torch.randint(VOCAB, (BATCH_SIZE, D), generator=rng)
    toks[:, 0] = x0
    return sids, toks, x0


def _eval_acc(model, pool_ids):
    model.eval()
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 200)
    correct = 0
    with torch.no_grad():
        for _ in range(N_MEASURE_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, rng)
            correct += (model(sids, toks, x0, 0.05).argmax(1) == x0).sum().item()
    model.train()
    return correct / N_EVAL


def train_model(TN_ang, TR, tau0_hyb, pool_ids):
    model = RouterAngularHybrid(TN_ang, TR, tau0_hyb, pool_ids)
    opt   = torch.optim.SGD(model.parameters(), lr=LR)
    rng_t = torch.Generator().manual_seed(GLOBAL_SEED + 100)
    model.train()
    t0 = time.perf_counter(); solve_step = None; final_acc = 0.0
    for step in range(1, MAX_STEPS + 1):
        frac = step / max(MAX_STEPS - 1, 1)
        temp = float(TEMP_START * (TEMP_END / TEMP_START) ** frac)
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


# ═══════════════════════════════════════════════════════════════════════
# Trajectory collection (hard geom or soft)
# ═══════════════════════════════════════════════════════════════════════
def collect_trajectories(model, pool_ids, routing: str = "exact_k1") -> Tuple[
    List[torch.Tensor],   # list of (B,12) tau stacks per batch
    List[torch.Tensor],   # list of (B,) x0 per batch
]:
    """Collect full tau trajectories for N_EVAL examples under the given routing."""
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 888)
    all_st: List[torch.Tensor] = []
    all_x0: List[torch.Tensor] = []
    B = BATCH_SIZE

    with torch.no_grad():
        for _ in range(N_MEASURE_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, rng)
            tau_prev  = model.tau0_table[sids]
            sids_loop = sids.clone()
            taus: List[torch.Tensor] = []

            for t in range(D):
                tn = model.TN[sids_loop]

                if routing == "exact_k1":
                    cur_dir = tau_prev[:, :D_TAU_ANG]
                    ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)
                    chosen  = ang_sim.argmax(dim=1)
                elif routing == "soft_mlp":
                    embs = model.W_emb[toks[:, t]]
                    h    = torch.tanh(torch.cat([embs, tau_prev], dim=1) @ model.W1 + model.b1)
                    w    = torch.softmax((h @ model.W2 + model.b2) / 0.05, dim=1)
                    # For soft: compute hybrid tau from blend
                    base  = torch.einsum("bi,bij->bj", w, tn)
                    pairs = base.view(B, N_PHASE_PAIRS, 2)
                    mag   = (pairs * pairs).sum(dim=2).sqrt()
                    dirn  = (pairs / mag.clamp(min=1e-8).unsqueeze(2)).view(B, D_TAU_ANG)
                    hybrid = torch.cat([dirn, mag], dim=1)
                    tau_prev = (hybrid + model.W_tok_inject[x0]) if t == 0 else hybrid
                    taus.append(tau_prev.clone())
                    sids_loop = model.TR[sids_loop].gather(
                        1, w.argmax(dim=1).unsqueeze(1)).squeeze(1)
                    continue  # skip the hard-geom path below

                # Hard geom path (exact_k1)
                best_ang = tn.gather(
                    1, chosen.view(B,1,1).expand(B,1,D_TAU_ANG)
                ).squeeze(1)
                hybrid   = torch.cat([best_ang, torch.ones(B, N_PHASE_PAIRS)], dim=1)
                tau_prev = (hybrid + model.W_tok_inject[x0]) if t == 0 else hybrid
                taus.append(tau_prev.clone())
                sids_loop = model.TR[sids_loop].gather(1, chosen.unsqueeze(1)).squeeze(1)

            all_st.append(torch.stack(taus, dim=1))   # (B, D, 12)
            all_x0.append(x0)

    return all_st, all_x0


# ═══════════════════════════════════════════════════════════════════════
# Readout ablations
# ═══════════════════════════════════════════════════════════════════════
def apply_ablation(model, all_st, all_x0, variant: str,
                   rand_taus: Optional[torch.Tensor] = None) -> Tuple[float, float, float]:
    """
    Apply a readout ablation to pre-collected trajectories.

    Returns (accuracy, mean_alpha0, runtime_seconds)

    Variants:
      reference        — standard readout, no modification
      tau0_direct      — pred = τ₀ @ W_pred + b_pred (bypass attention)
      tau0_repeated    — st[:, t>0, :] = st[:, 0:1, :].expand; standard readout
      no_tau0          — mask position 0 with -1e9 in attention scores
      early_2          — mask positions 2..D-1 with -1e9
      late_half        — mask positions 0..D//2-1 with -1e9
      random_t_gt_0    — replace st[:, 1:, :] with random taus; standard readout
    """
    t_start = time.perf_counter()
    correct = 0
    total   = 0
    alpha0_sum = 0.0

    NEG_INF = -1e9

    with torch.no_grad():
        for st, x0 in zip(all_st, all_x0):
            B = st.shape[0]

            if variant == "reference":
                pred, alpha = model.readout(st)
                alpha0_sum += alpha[:, 0].mean().item()

            elif variant == "tau0_direct":
                # Bypass attention: directly read from τ₀
                tau0 = st[:, 0, :]                                # (B, 12)
                pred = tau0 @ model.W_pred + model.b_pred         # (B, VOCAB)
                alpha0_sum += 1.0  # by definition position 0 only

            elif variant == "tau0_repeated":
                # Replace all t>0 positions with τ₀
                st_abl = st.clone()
                st_abl[:, 1:, :] = st[:, 0:1, :].expand(B, D-1, D_TAU_HYB)
                pred, alpha = model.readout(st_abl)
                alpha0_sum += alpha[:, 0].mean().item()

            elif variant == "no_tau0":
                # Mask out position 0 from attention
                mask = torch.zeros(1, D)
                mask[0, 0] = NEG_INF
                pred, alpha = model.readout_masked(st, mask)
                alpha0_sum += alpha[:, 0].mean().item()

            elif variant == "early_2":
                # Use only positions 0 and 1
                mask = torch.zeros(1, D)
                mask[0, 2:] = NEG_INF
                pred, alpha = model.readout_masked(st, mask)
                alpha0_sum += alpha[:, 0].mean().item()

            elif variant == "late_half":
                # Use only positions D//2 .. D-1
                mask = torch.zeros(1, D)
                mask[0, :D//2] = NEG_INF
                pred, alpha = model.readout_masked(st, mask)
                alpha0_sum += 0.0  # pos0 is masked

            elif variant == "random_t_gt_0":
                # Replace t>0 taus with random taus from the pool
                st_abl = st.clone()
                if rand_taus is not None:
                    # rand_taus: (B, D-1, 12) pre-sampled
                    st_abl[:, 1:, :] = rand_taus[:B, :, :]
                pred, alpha = model.readout(st_abl)
                alpha0_sum += alpha[:, 0].mean().item()

            else:
                raise ValueError(f"Unknown ablation variant: {variant}")

            correct += (pred.argmax(1) == x0).sum().item()
            total   += B

    wall = time.perf_counter() - t_start
    acc = round(correct / total, 4)
    mean_alpha0 = round(alpha0_sum / N_MEASURE_BATCHES, 4)
    return acc, mean_alpha0, round(wall, 4)


# ═══════════════════════════════════════════════════════════════════════
# Markdown writer
# ═══════════════════════════════════════════════════════════════════════
def _write_markdown(rows_hard, rows_soft, ref_acc_hard, ref_acc_soft,
                    b_pos0_val, conclusion):
    ts = datetime.datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ")
    L: List[str] = []
    L.append("# Prime Transport Router: τ₀ Sufficiency Probe v1\n\n")
    L.append(f"**Generated:** {ts}  \n")
    L.append(f"**Config:** D={D}, D_HIDDEN={D_HIDDEN}, B={N_EVAL}, CPU  \n")
    L.append(f"**Trained b_pos0:** {b_pos0_val:.4f}  \n\n")

    def table(rows, ref_acc):
        L.append("| variant | accuracy | Δ_vs_ref | alpha₀ | runtime_s | ablated_component |\n")
        L.append("|---------|----------|----------|--------|-----------|-------------------|\n")
        for r in rows:
            delta = round(r["accuracy"] - ref_acc, 4)
            sign  = "+" if delta >= 0 else ""
            L.append(f"| {r['variant']} | {r['accuracy']:.4f} | {sign}{delta:.4f} "
                     f"| {r['alpha0']:.4f} | {r['runtime_s']:.3f} "
                     f"| {r['ablated_component']} |\n")
        L.append("\n")

    L.append("---\n\n")
    L.append("## Hard Geometric Routing (exact_k1) — Readout Ablations\n\n")
    table(rows_hard, ref_acc_hard)

    L.append("## Soft MLP Routing — Readout Ablations\n\n")
    table(rows_soft, ref_acc_soft)

    L.append("---\n\n")
    L.append("## Explicit Answers\n\n")

    # Extract key rows
    def get(rows, v): return next((r for r in rows if r["variant"] == v), None)

    r_tau0_direct  = get(rows_hard, "tau0_direct")
    r_tau0_rep     = get(rows_hard, "tau0_repeated")
    r_no_tau0      = get(rows_hard, "no_tau0")
    r_rand_t       = get(rows_hard, "random_t_gt_0")
    r_early        = get(rows_hard, "early_2")
    r_late         = get(rows_hard, "late_half")

    L.append("**1. Is τ₀ alone sufficient for 1.0000?**  \n")
    if r_tau0_direct and r_tau0_direct["accuracy"] >= 0.999:
        L.append(f"YES. tau0_direct (pred = τ₀ @ W_pred + b_pred, no attention) "
                 f"achieves {r_tau0_direct['accuracy']:.4f}. "
                 f"The attention mechanism is not required; τ₀ alone predicts correctly.  \n\n")
    elif r_tau0_direct and r_tau0_direct["accuracy"] >= 0.99:
        L.append(f"NEARLY. tau0_direct achieves {r_tau0_direct['accuracy']:.4f} "
                 f"(Δ={r_tau0_direct['accuracy']-ref_acc_hard:+.4f}).  \n\n")
    else:
        L.append(f"NO. tau0_direct achieves only {r_tau0_direct['accuracy']:.4f} "
                 f"(Δ={r_tau0_direct['accuracy']-ref_acc_hard:+.4f}). "
                 f"Attention over the full trajectory is required.  \n\n")

    L.append("**2. If τ₀ is repeated across all positions, does performance stay perfect?**  \n")
    if r_tau0_rep and abs(r_tau0_rep["accuracy"] - ref_acc_hard) < 0.005:
        L.append(f"YES. tau0_repeated achieves {r_tau0_rep['accuracy']:.4f} "
                 f"(Δ={r_tau0_rep['accuracy']-ref_acc_hard:+.4f}). "
                 f"Filling all positions with τ₀ does not degrade prediction.  \n\n")
    elif r_tau0_rep:
        L.append(f"PARTIALLY. tau0_repeated achieves {r_tau0_rep['accuracy']:.4f} "
                 f"(Δ={r_tau0_rep['accuracy']-ref_acc_hard:+.4f}).  \n\n")

    L.append("**3. If τ₀ is removed, does accuracy collapse?**  \n")
    if r_no_tau0 and r_no_tau0["accuracy"] < 0.5:
        L.append(f"YES — complete collapse. no_tau0 accuracy = {r_no_tau0['accuracy']:.4f} "
                 f"(chance ≈ {1/VOCAB:.2f}). Without τ₀ the model cannot predict.  \n\n")
    elif r_no_tau0 and r_no_tau0["accuracy"] < 0.99:
        L.append(f"YES — significant degradation. no_tau0 accuracy = {r_no_tau0['accuracy']:.4f} "
                 f"(Δ={r_no_tau0['accuracy']-ref_acc_hard:+.4f}).  \n\n")
    elif r_no_tau0:
        L.append(f"NO. no_tau0 still achieves {r_no_tau0['accuracy']:.4f} — "
                 f"the trajectory τ_t (t≥1) also contains sufficient information.  \n\n")

    L.append("**4. Are τ_t for t>0 contributing anything measurable at D=24?**  \n")
    if r_rand_t and abs(r_rand_t["accuracy"] - ref_acc_hard) < 0.005:
        L.append(f"NO measurable contribution. random_t_gt_0 achieves "
                 f"{r_rand_t['accuracy']:.4f} after replacing all t>0 taus with random values. "
                 f"τ₀ alone is sufficient; subsequent taus carry no necessary information.  \n\n")
    elif r_rand_t and r_rand_t["accuracy"] >= 0.99:
        L.append(f"MINIMAL. random_t_gt_0 achieves {r_rand_t['accuracy']:.4f} "
                 f"(Δ={r_rand_t['accuracy']-ref_acc_hard:+.4f}) — small degradation when "
                 f"t>0 taus are randomised. τ₀ carries almost all information.  \n\n")
    elif r_rand_t:
        L.append(f"YES. random_t_gt_0 drops to {r_rand_t['accuracy']:.4f} "
                 f"(Δ={r_rand_t['accuracy']-ref_acc_hard:+.4f}). "
                 f"τ_t for t>0 carry necessary information.  \n\n")

    L.append("---\n\n")
    L.append("## Honesty Section\n\n")

    # Determine what matters
    strong_drops = [r for r in rows_hard if r["accuracy"] < 0.99]
    no_drops     = [r for r in rows_hard if abs(r["accuracy"] - ref_acc_hard) < 0.005
                    and r["variant"] != "reference"]

    L.append("**What definitely matters:**  \n")
    if strong_drops:
        for r in strong_drops:
            L.append(f"- Ablating [{r['variant']}] → acc={r['accuracy']:.4f} "
                     f"(Δ={r['accuracy']-ref_acc_hard:+.4f}). This component is load-bearing.  \n")
    else:
        L.append("- No single ablation caused acc < 0.99. This is itself informative.  \n")
    L.append("\n")

    L.append("**What definitely does not matter:**  \n")
    if no_drops:
        for r in no_drops:
            L.append(f"- [{r['variant']}]: acc={r['accuracy']:.4f} (no degradation). "
                     f"The ablated component is not load-bearing.  \n")
    else:
        L.append("- All ablations caused some degradation.  \n")
    L.append("\n")

    L.append("**What remains ambiguous:**  \n")
    moderate = [r for r in rows_hard if 0.95 <= r["accuracy"] < 0.99]
    if moderate:
        for r in moderate:
            L.append(f"- [{r['variant']}]: acc={r['accuracy']:.4f} — partial degradation. "
                     f"This component is partially load-bearing but could be compensated.  \n")
    else:
        L.append("- No moderate failures (0.95-0.99): all ablations are either harmless or catastrophic.  \n")
    L.append("\n")

    L.append("---\n\n")
    L.append(f"## TAU0 ALONE IS: {conclusion}\n\n")

    with open(MD_OUT, "w") as f:
        f.writelines(L)


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main():
    print("=" * 60)
    print("TAU0 SUFFICIENCY PROBE v1")
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
    b_pos0_val = float(model.b_pos0.item())
    print(f"  b_pos0 (trained) = {b_pos0_val:.4f}")

    # Pre-sample random taus for the random_t_gt_0 ablation
    rng_r = torch.Generator().manual_seed(GLOBAL_SEED + 77777)
    rand_pool_idx = torch.randint(pool_ids.shape[0],
                                  (BATCH_SIZE, D - 1), generator=rng_r)
    rand_sids = pool_ids[rand_pool_idx]                          # (B, D-1)
    rand_taus = model.tau0_table[rand_sids]                      # (B, D-1, 12)

    # ── Collect trajectories ─────────────────────────────────────────────
    print("\nCollecting hard_geom (exact_k1) trajectories ...")
    all_st_hard, all_x0_hard = collect_trajectories(model, pool_ids, "exact_k1")

    print("Collecting soft_mlp trajectories ...")
    all_st_soft, all_x0_soft = collect_trajectories(model, pool_ids, "soft_mlp")

    # ── Define ablation variants ─────────────────────────────────────────
    ABLATION_SPECS = [
        ("reference",      "none (full trajectory)"),
        ("tau0_direct",    "attention bypassed; pred=τ₀@W_pred+b_pred"),
        ("tau0_repeated",  "τ_t (t>0) replaced with τ₀"),
        ("no_tau0",        "position 0 masked in attention (alpha[0]=0)"),
        ("early_2",        "positions 2..D-1 masked (only pos 0,1 used)"),
        ("late_half",      f"positions 0..{D//2-1} masked (only pos {D//2}..{D-1} used)"),
        ("random_t_gt_0",  "τ_t (t>0) replaced with random tau table entries"),
    ]

    print(f"\n{'='*60}")
    print("APPLYING READOUT ABLATIONS")
    print(f"{'='*60}")

    rows_hard: List[Dict] = []
    rows_soft: List[Dict] = []
    ref_acc_hard = 0.0
    ref_acc_soft = 0.0

    for routing_label, (all_st, all_x0) in [
        ("hard_geom", (all_st_hard, all_x0_hard)),
        ("soft_mlp",  (all_st_soft, all_x0_soft)),
    ]:
        print(f"\n  [{routing_label}]")
        rows = rows_hard if routing_label == "hard_geom" else rows_soft
        ref_acc = 0.0

        for variant, ablated_desc in ABLATION_SPECS:
            acc, alpha0, wall = apply_ablation(
                model, all_st, all_x0, variant,
                rand_taus=rand_taus if variant == "random_t_gt_0" else None
            )
            if variant == "reference":
                if routing_label == "hard_geom":
                    ref_acc_hard = acc
                else:
                    ref_acc_soft = acc
                ref_acc = acc

            delta = round(acc - ref_acc, 4)
            sign  = "+" if delta >= 0 else ""
            print(f"    [{variant:<18}] acc={acc:.4f}  Δ={sign}{delta:.4f}  "
                  f"alpha₀={alpha0:.4f}  t={wall:.3f}s")

            rows.append({
                "variant":           variant,
                "routing":           routing_label,
                "accuracy":          acc,
                "delta_vs_reference": delta,
                "alpha0":            alpha0,
                "runtime_s":         wall,
                "ablated_component": ablated_desc,
                "note":              f"D={D} {routing_label}",
            })

    # ── Summary ───────────────────────────────────────────────────────────
    print(f"\n{'='*60}")
    print("SUMMARY")
    print(f"{'='*60}")
    r_tau0d = next(r for r in rows_hard if r["variant"] == "tau0_direct")
    r_notau = next(r for r in rows_hard if r["variant"] == "no_tau0")
    r_randr = next(r for r in rows_hard if r["variant"] == "random_t_gt_0")
    r_tau0r = next(r for r in rows_hard if r["variant"] == "tau0_repeated")

    print(f"  τ₀ alone (direct):        acc={r_tau0d['accuracy']:.4f}")
    print(f"  τ₀ repeated all pos:      acc={r_tau0r['accuracy']:.4f}")
    print(f"  No τ₀ (mask pos 0):       acc={r_notau['accuracy']:.4f}")
    print(f"  Random t>0 taus:          acc={r_randr['accuracy']:.4f}")
    print(f"  Reference:                acc={ref_acc_hard:.4f}")

    # Determine conclusion
    tau0_direct_ok   = r_tau0d["accuracy"] >= 0.999
    tau0_repeated_ok = r_tau0r["accuracy"] >= 0.999
    no_tau0_bad      = r_notau["accuracy"] < 0.5
    random_t_ok      = r_randr["accuracy"] >= 0.999

    if tau0_direct_ok and random_t_ok and no_tau0_bad:
        conclusion = "SUFFICIENT"
    elif tau0_direct_ok and not no_tau0_bad:
        conclusion = "SUFFICIENT (but τ_{t>0} also carry redundant information)"
    elif r_tau0d["accuracy"] >= 0.99 and r_randr["accuracy"] >= 0.99:
        conclusion = "PARTIALLY SUFFICIENT (nearly so; small residual from trajectory)"
    else:
        conclusion = "INSUFFICIENT (trajectory τ_{t>0} contributes necessary information)"

    print(f"\nTAU0 ALONE IS: {conclusion}")

    # ── CSV ───────────────────────────────────────────────────────────────
    fieldnames = [
        "variant", "routing", "accuracy", "delta_vs_reference",
        "alpha0", "runtime_s", "ablated_component", "note",
    ]
    with open(CSV_OUT, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows_hard + rows_soft)
    print(f"\nCSV written: {CSV_OUT}")

    _write_markdown(rows_hard, rows_soft, ref_acc_hard, ref_acc_soft,
                    b_pos0_val, conclusion)
    print(f"Markdown written: {MD_OUT}")


if __name__ == "__main__":
    main()

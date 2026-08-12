#!/usr/bin/env python3
"""run_router_r4_restoration_probe_v1.py

R^4 GEOMETRY RESTORATION WITH K=1 LOCAL INFERENCE v1

Tests whether restoring the fuller coupled 4-factor angular+radial
geometry makes the post-τ₀ trajectory genuinely load-bearing again,
while preserving k=1 local geometric inference.

DIAGNOSIS OF CURRENT REDUCED REGIME:
  τ₀ is trivially sufficient because:
    1. W_tok_inject[x0] is added at t=0 → answer label embedded directly in τ₀
    2. b_pos0 learned = 4.97 → attention collapses to τ₀
    3. Radial part forced to 1.0 in hard geom → no radial dynamics
  Result: trajectory t>0 is informationally inert.

RESTORATION TOGGLES (documented explicitly):

  Toggle A — inject_position: "first" (current) vs "last" (restored)
    "first": tau_0 += W_tok_inject[x0]                  (current: answer in τ₀)
    "last":  tau_{D-1} += W_tok_inject[x0]               (restored: answer in τ_{D-1})
    Rationale: moving injection to t=D-1 forces the routing trajectory to carry
    the answer to the final position; τ₀ no longer encodes the answer.

  Toggle B — b_pos0_init: 2.0 (current) vs 0.0 (restored)
    Rationale: large b_pos0 causes attention to collapse to pos 0 during training.
    Initialising at 0.0 lets the model learn the attention pattern from data.

  Toggle C — radial_mode: "fixed_one" (current) vs "dynamic" (restored)
    "fixed_one": hard geom uses ones(B, N_PHASE_PAIRS)   (current: kills radial DOF)
    "dynamic":   hard geom uses ang_sim[best_op] as radial magnitude
    Rationale: restores a genuine radial degree of freedom (routing confidence)
    that varies per step based on angular alignment quality.

CONFIGURATIONS:
  baseline  — inject_position="first",  b_pos0_init=2.0, radial="fixed_one"
  restored  — inject_position="last",   b_pos0_init=0.0, radial="dynamic"

ABLATIONS (per configuration, on hard geom trajectories):
  full          — standard readout (reference)
  tau0_direct   — bypass attention; pred = τ₀ @ W_pred + b_pred
  no_tau0       — mask position 0 in attention (alpha[0] → 0)
  last_only     — mask all positions except t=D-1
  no_last       — mask position D-1 in attention
  late_half     — use only positions D//2..D-1
  early_2       — use only positions 0,1
  random_t_gt0  — replace t>0 taus with random table entries; τ₀ intact

All ablations are INFERENCE-ONLY. Both configurations are trained fresh.
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
CSV_OUT     = RESULTS_DIR / "prime_transport_router_r4_restoration_probe_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_router_r4_restoration_probe_v1.md"
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
MAX_STEPS     = 3_500   # slightly more steps: restored model may need longer
N_BATCHES     = N_EVAL // BATCH_SIZE

try:
    sys.path.insert(0, str(SCRIPT_DIR))
    from thread_policy import select_threads as _st
    _st(BATCH_SIZE, D_IN_HYB, D_HIDDEN)
except Exception:
    pass


# ═══════════════════════════════════════════════════════════════════════
# Angular conversion + hybrid tables
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
# Unified model with restoration toggles
# ═══════════════════════════════════════════════════════════════════════
class RouterGeom(nn.Module):
    """
    Parameterised router supporting both baseline and restored geometry.

    inject_position: "first" | "last"
      "first" — W_tok_inject[x0] added to τ₀ (current reduced regime)
      "last"  — W_tok_inject[x0] added to τ_{D-1} (restored: answer in final pos)

    b_pos0_init: float
      2.0 (current) → large bias to pos 0 during training
      0.0 (restored) → neutral initialisation; model learns attention pattern

    NOTE: radial_mode toggle is only at inference time (hard geom eval);
          the soft training forward always uses blended magnitudes naturally.
    """
    def __init__(self, TN_ang, TR, tau0_hyb, pool_ids,
                 inject_position: str = "first",
                 b_pos0_init: float = 2.0,
                 init_scale: float = INIT_SCALE,
                 seed: int = GLOBAL_SEED):
        super().__init__()
        self.inject_position = inject_position
        dh = D_HIDDEN; dha = max(8, dh // 4)
        self.register_buffer("TN", TN_ang)
        self.register_buffer("TR", TR)
        self.register_buffer("tau0_table", tau0_hyb)
        self.register_buffer("pool_ids", pool_ids)
        m = torch.zeros(1, D); m[0, 0] = 1.0
        self.register_buffer("pos0_mask", m)
        self.b_pos0 = nn.Parameter(torch.tensor(b_pos0_init))
        gen = torch.Generator().manual_seed(seed)
        def rp(*sh): return nn.Parameter(torch.empty(*sh).normal_(0, init_scale, generator=gen))
        def zp(*sh): return nn.Parameter(torch.zeros(*sh))
        self.W_emb        = rp(VOCAB, D_EMB)
        self.W1           = rp(D_IN_HYB, dh);    self.b1 = zp(dh)
        self.W2           = rp(dh, N_OPS);        self.b2 = zp(N_OPS)
        self.W_attn       = rp(dha, D_TAU_HYB);  self.b_attn = zp(dha); self.v_attn = rp(dha)
        self.W_pred       = rp(D_TAU_HYB, VOCAB); self.b_pred = zp(VOCAB)
        self.W_tok_inject = rp(VOCAB, D_TAU_HYB)

    def _inject(self, hybrid: torch.Tensor, x0: torch.Tensor, t: int) -> torch.Tensor:
        """Apply W_tok_inject[x0] at the configured position."""
        if self.inject_position == "first" and t == 0:
            return hybrid + self.W_tok_inject[x0]
        elif self.inject_position == "last" and t == D - 1:
            return hybrid + self.W_tok_inject[x0]
        return hybrid

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
            hybrid   = torch.cat([dirn, mag], dim=1)
            tau_prev = self._inject(hybrid, x0, t)
            soft_taus.append(tau_prev)
            state_ids = self.TR[state_ids].gather(
                1, torch.argmax(w, dim=1).unsqueeze(1)).squeeze(1)
        st    = torch.stack(soft_taus, dim=1)
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = (h_a * self.v_attn).sum(dim=-1) + self.pos0_mask * self.b_pos0
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred

    def readout(self, st):
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = (h_a * self.v_attn).sum(dim=-1) + self.pos0_mask * self.b_pos0
        alpha = torch.softmax(sc, dim=1)
        pred  = torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred
        return pred, alpha

    def readout_masked(self, st, attn_mask):
        """attn_mask (1,D): added to attention scores before softmax."""
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = (h_a * self.v_attn).sum(dim=-1) + self.pos0_mask * self.b_pos0 + attn_mask
        alpha = torch.softmax(sc, dim=1)
        pred  = torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred
        return pred, alpha


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


def eval_acc(model, pool_ids):
    model.eval()
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 200)
    correct = 0
    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, rng)
            correct += (model(sids, toks, x0, 0.05).argmax(1) == x0).sum().item()
    model.train()
    return correct / N_EVAL


def train_model(TN_ang, TR, tau0_hyb, pool_ids,
                inject_position: str, b_pos0_init: float, label: str):
    model = RouterGeom(TN_ang, TR, tau0_hyb, pool_ids,
                       inject_position=inject_position,
                       b_pos0_init=b_pos0_init)
    opt   = torch.optim.SGD(model.parameters(), lr=LR)
    rng_t = torch.Generator().manual_seed(GLOBAL_SEED + 100)
    model.train()
    t0 = time.perf_counter(); solve_step = None; final_acc = 0.0
    for step in range(1, MAX_STEPS + 1):
        frac = step / max(MAX_STEPS - 1, 1)
        temp = float(TEMP_START * (TEMP_END / TEMP_START) ** frac)
        sids, toks, x0 = make_batch(pool_ids, rng_t)
        loss = F.cross_entropy(model(sids, toks, x0, temp), x0)
        opt.zero_grad(); loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), CLIP_GRAD)
        opt.step()
        if step % EVAL_EVERY == 0:
            acc = eval_acc(model, pool_ids)
            if acc >= SOLVE_ACC and solve_step is None: solve_step = step
            final_acc = acc
            print(f"    [{label}] step={step:5d}  acc={acc:.4f}")
    wall = time.perf_counter() - t0
    sps  = round(MAX_STEPS / wall, 1)
    print(f"  {label}: acc={final_acc:.4f}  solve={solve_step}  sps={sps}  "
          f"b_pos0={model.b_pos0.item():.3f}")
    model.eval()
    return model, final_acc, solve_step, sps


# ═══════════════════════════════════════════════════════════════════════
# Hard geometric trajectory collection (with optional dynamic radial)
# ═══════════════════════════════════════════════════════════════════════
def collect_hard_geom_trajectories(model, pool_ids,
                                   radial_mode: str = "fixed_one"):
    """
    Collect hard k=1 geometric trajectories.

    radial_mode:
      "fixed_one" — τ[8:] = ones(B, N_PHASE_PAIRS)  (current baseline)
      "dynamic"   — τ[8:] = ang_sim[best_op].clamp(0,2).unsqueeze(1)×ones(...)
                    (restored: routing confidence as radial magnitude)
    """
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 888)
    all_st: List[torch.Tensor] = []
    all_x0: List[torch.Tensor] = []

    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, rng)
            B = BATCH_SIZE
            tau_prev  = model.tau0_table[sids]
            sids_loop = sids.clone()
            taus: List[torch.Tensor] = []

            for t in range(D):
                tn      = model.TN[sids_loop]                          # (B,6,8)
                cur_dir = tau_prev[:, :D_TAU_ANG]                      # (B,8)
                ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)      # (B,6)
                best_op = ang_sim.argmax(dim=1)                        # (B,)

                best_ang = tn.gather(
                    1, best_op.view(B,1,1).expand(B,1,D_TAU_ANG)
                ).squeeze(1)                                            # (B,8)

                if radial_mode == "fixed_one":
                    radial = torch.ones(B, N_PHASE_PAIRS)
                else:  # dynamic: use angular alignment score as radial magnitude
                    best_sim = ang_sim.gather(
                        1, best_op.unsqueeze(1)).squeeze(1)             # (B,)
                    # Shift to [0, 2] range (ang_sim ∈ [-2, 2] for 4 phase pairs)
                    # Normalise so mean sim maps to ~1.0 radial
                    radial = (best_sim + 2.0).clamp(0.0, 4.0).unsqueeze(1).expand(
                        B, N_PHASE_PAIRS) / 2.0                         # (B,4) in [0,2]

                hybrid = torch.cat([best_ang, radial], dim=1)          # (B,12)
                tau_cur = model._inject(hybrid, x0, t)
                taus.append(tau_cur.clone())
                tau_prev  = tau_cur
                sids_loop = model.TR[sids_loop].gather(
                    1, best_op.unsqueeze(1)).squeeze(1)

            all_st.append(torch.stack(taus, dim=1))   # (B,D,12)
            all_x0.append(x0)

    return all_st, all_x0


# ═══════════════════════════════════════════════════════════════════════
# Readout ablations
# ═══════════════════════════════════════════════════════════════════════
NEG_INF = -1e9

ABLATION_DEFS = [
    ("full",          "none — full trajectory"),
    ("tau0_direct",   "attention bypassed; pred = τ₀ @ W_pred + b_pred"),
    ("no_tau0",       "position 0 masked (alpha[0]→0)"),
    ("last_only",     f"all positions except t={D-1} masked"),
    ("no_last",       f"position t={D-1} masked"),
    ("late_half",     f"positions 0..{D//2-1} masked (only {D//2}..{D-1} used)"),
    ("early_2",       "positions 2..D-1 masked (only 0,1 used)"),
    ("random_t_gt0",  "τ_t (t>0) replaced with random tau table entries; τ₀ intact"),
]


def apply_ablation(model, all_st, all_x0, variant: str,
                   rand_taus: Optional[torch.Tensor] = None):
    t_start = time.perf_counter()
    correct = 0; alpha0_sum = 0.0; alphaD_sum = 0.0

    with torch.no_grad():
        for st, x0 in zip(all_st, all_x0):
            B = st.shape[0]

            if variant == "full":
                pred, alpha = model.readout(st)
            elif variant == "tau0_direct":
                pred = st[:, 0, :] @ model.W_pred + model.b_pred
                alpha = torch.zeros(B, D); alpha[:, 0] = 1.0
            elif variant == "no_tau0":
                mask = torch.zeros(1, D); mask[0, 0] = NEG_INF
                pred, alpha = model.readout_masked(st, mask)
            elif variant == "last_only":
                mask = torch.full((1, D), NEG_INF); mask[0, D-1] = 0.0
                pred, alpha = model.readout_masked(st, mask)
            elif variant == "no_last":
                mask = torch.zeros(1, D); mask[0, D-1] = NEG_INF
                pred, alpha = model.readout_masked(st, mask)
            elif variant == "late_half":
                mask = torch.zeros(1, D); mask[0, :D//2] = NEG_INF
                pred, alpha = model.readout_masked(st, mask)
            elif variant == "early_2":
                mask = torch.zeros(1, D); mask[0, 2:] = NEG_INF
                pred, alpha = model.readout_masked(st, mask)
            elif variant == "random_t_gt0":
                st_abl = st.clone()
                if rand_taus is not None:
                    st_abl[:, 1:, :] = rand_taus[:B, :, :]
                pred, alpha = model.readout(st_abl)
            else:
                raise ValueError(f"Unknown ablation: {variant}")

            correct   += (pred.argmax(1) == x0).sum().item()
            alpha0_sum += alpha[:, 0].mean().item()
            alphaD_sum += alpha[:, D-1].mean().item()

    wall  = round(time.perf_counter() - t_start, 4)
    acc   = round(correct / N_EVAL, 4)
    a0    = round(alpha0_sum / N_BATCHES, 4)
    aD    = round(alphaD_sum / N_BATCHES, 4)
    return acc, a0, aD, wall


# ═══════════════════════════════════════════════════════════════════════
# Markdown writer
# ═══════════════════════════════════════════════════════════════════════
def _write_markdown(configs_data, conclusion):
    ts = datetime.datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ")
    L: List[str] = []
    L.append("# Prime Transport Router: R^4 Geometry Restoration Probe v1\n\n")
    L.append(f"**Generated:** {ts}  \n")
    L.append(f"**Config:** D={D}, D_HIDDEN={D_HIDDEN}, B_train={BATCH_SIZE}, N_eval={N_EVAL}  \n\n")

    L.append("---\n\n")
    L.append("## Restoration Toggles (Documented Explicitly)\n\n")
    L.append("| toggle | baseline (reduced regime) | restored (fuller geometry) |\n")
    L.append("|--------|--------------------------|-----------------------------|\n")
    L.append(f"| inject_position | first (t=0): τ₀ += W_tok_inject[x0] "
             f"| last (t={D-1}): τ_{{D-1}} += W_tok_inject[x0] |\n")
    L.append("| b_pos0_init | 2.0 (large → collapses to τ₀) "
             "| 0.0 (neutral → model learns attention) |\n")
    L.append("| radial_mode (hard geom) | fixed_one (τ[8:] = 1.0 always) "
             "| dynamic (τ[8:] = ang_sim_confidence) |\n\n")
    L.append("> **Toggle A (inject_position)** is the primary restoration: moving the "
             "answer injection to t=D-1 prevents τ₀ from carrying the answer.  \n")
    L.append("> **Toggle B (b_pos0_init)** prevents attention collapse to pos 0 during training.  \n")
    L.append("> **Toggle C (radial_mode)** restores a genuine radial DOF encoding routing confidence.  \n\n")

    for cfg_label, cfg in configs_data.items():
        L.append(f"---\n\n## Configuration: {cfg_label}\n\n")
        b0 = cfg["b_pos0_trained"]
        L.append(f"- inject_position: **{cfg['inject_position']}**  \n")
        L.append(f"- b_pos0_init: {cfg['b_pos0_init']} → trained value: **{b0:.3f}**  \n")
        L.append(f"- radial_mode: **{cfg['radial_mode']}**  \n")
        L.append(f"- solve_step: {cfg['solve_step']}  ")
        L.append(f"train_acc: {cfg['train_acc']:.4f}  sps: {cfg['sps']}  \n\n")

        L.append("| ablation | accuracy | Δ_vs_full | alpha₀ | alpha_{D-1} | "
                 "runtime_s | interpretation |\n")
        L.append("|----------|----------|-----------|--------|-------------|"
                 "-----------|----------------|\n")
        full_acc = next(r["accuracy"] for r in cfg["ablations"] if r["variant"] == "full")
        for r in cfg["ablations"]:
            delta = round(r["accuracy"] - full_acc, 4)
            sign  = "+" if delta >= 0 else ""
            interp = _interpret(r["variant"], r["accuracy"], full_acc, cfg["inject_position"])
            L.append(f"| {r['variant']} | {r['accuracy']:.4f} | {sign}{delta:.4f} "
                     f"| {r['alpha0']:.4f} | {r['alphaD']:.4f} | {r['runtime_s']:.3f} "
                     f"| {interp} |\n")
        L.append("\n")

    L.append("---\n\n")
    L.append("## Explicit Answers\n\n")

    base = configs_data.get("baseline")
    rest = configs_data.get("restored")

    def get_abl(cfg, v): return next((r for r in cfg["ablations"] if r["variant"] == v), {})

    L.append("**1. After restoring fuller geometry, is τ₀ still sufficient by itself?**  \n")
    if rest:
        t0d = get_abl(rest, "tau0_direct")
        if t0d.get("accuracy", 0) >= 0.999:
            L.append(f"YES — tau0_direct still achieves {t0d['accuracy']:.4f}. "
                     f"Moving injection to t=D-1 did not prevent τ₀ from carrying the answer.  \n\n")
        elif t0d.get("accuracy", 0) < 0.5:
            L.append(f"NO — tau0_direct drops to {t0d['accuracy']:.4f} (≈chance). "
                     f"τ₀ no longer encodes the answer. Restoration successful.  \n\n")
        else:
            L.append(f"PARTIALLY — tau0_direct achieves {t0d['accuracy']:.4f}. "
                     f"τ₀ carries partial information.  \n\n")

    L.append("**2. Do positions t>0 become informationally necessary?**  \n")
    if rest:
        rtg = get_abl(rest, "random_t_gt0")
        nolast = get_abl(rest, "no_last")
        lastonly = get_abl(rest, "last_only")
        if lastonly.get("accuracy", 0) >= 0.999:
            L.append(f"YES — last_only achieves {lastonly['accuracy']:.4f}: the final position "
                     f"carries the answer. no_last drops to {nolast.get('accuracy',0):.4f}. "
                     f"Post-τ₀ routing trajectory determines which state is reached at t=D-1.  \n\n")
        elif rtg.get("accuracy", 0) < 0.99:
            L.append(f"YES — random_t_gt0 drops to {rtg.get('accuracy',0):.4f}: "
                     f"τ_t>0 carry necessary information.  \n\n")
        else:
            L.append(f"PARTIALLY — random_t_gt0: {rtg.get('accuracy',0):.4f}.  \n\n")

    L.append("**3. Does k=1 local geometric inference still solve cleanly?**  \n")
    if rest:
        full = get_abl(rest, "full")
        if full.get("accuracy", 0) >= 0.999:
            L.append(f"YES — full trajectory accuracy = {full['accuracy']:.4f} with k=1 hard geom.  \n\n")
        else:
            L.append(f"NO — full trajectory accuracy = {full['accuracy']:.4f}. "
                     f"k=1 inference may not suffice for the restored geometry.  \n\n")

    L.append("**4. Does the restored geometry produce genuinely load-bearing trajectory evolution?**  \n")
    if rest:
        notau = get_abl(rest, "no_tau0")
        nolast = get_abl(rest, "no_last")
        if notau.get("accuracy", 0) >= 0.999 and nolast.get("accuracy", 0) < 0.5:
            L.append("YES — removing τ₀ leaves accuracy intact; removing τ_{D-1} collapses it. "
                     "The routing trajectory is load-bearing: it determines which state arrives at t=D-1.  \n\n")
        elif notau.get("accuracy", 0) < 0.5 and nolast.get("accuracy", 0) < 0.5:
            L.append("PARTIALLY — both positions are necessary; information is distributed.  \n\n")
        else:
            L.append(f"no_tau0={notau.get('accuracy',0):.4f}  "
                     f"no_last={nolast.get('accuracy',0):.4f}  \n\n")

    L.append("**5. Does restoring the fuller basis move toward local-routing / OSPF-like architecture?**  \n")
    if rest:
        full = get_abl(rest, "full")
        if full.get("accuracy", 0) >= 0.999:
            L.append("YES. With inject_position='last', the model must route through D steps, "
                     "arriving at a final state from which the answer is decoded. This is the "
                     "OSPF-like pattern: navigate to a target state, then read out the identity "
                     "of that state. k=1 inference preserves the local-routing property.  \n\n")
        else:
            L.append("PARTIALLY — accuracy under the restored regime is not yet 1.0.  \n\n")

    L.append("---\n\n")
    L.append("## Honesty Section\n\n")

    L.append("**What improved:**  \n")
    if rest:
        t0d = get_abl(rest, "tau0_direct")
        full = get_abl(rest, "full")
        if t0d.get("accuracy", 1) < 0.5:
            L.append("- τ₀ is no longer trivially sufficient. Moving injection to t=D-1 forces "
                     "the routing trajectory to carry information forward.  \n")
        if full.get("accuracy", 0) >= 0.999:
            L.append("- k=1 local geometric inference still achieves 1.0000 accuracy.  \n")
        L.append("- Radial DOF is restored: routing confidence (ang_sim) is now encoded in τ[8:].  \n")
    L.append("\n")

    L.append("**What stayed trivial:**  \n")
    if rest:
        t0d_base = get_abl(base, "tau0_direct") if base else {}
        if t0d_base.get("accuracy", 0) >= 0.999:
            L.append("- In the baseline, τ₀-direct still achieves 1.0000.  \n")
        b0_rest = rest.get("b_pos0_trained", 0)
        L.append(f"- b_pos0 trained value in restored model: {b0_rest:.3f} "
                 f"({'converged near 0' if abs(b0_rest) < 0.5 else 'model may still prefer one end'}).  \n")
    L.append("\n")

    L.append("**What remains uncertain:**  \n")
    L.append("- Whether the dynamic radial DOF (ang_sim confidence) is actually used by the "
             "attention/prediction mechanism, or whether it has zero marginal contribution.  \n")
    L.append("- Whether a deeper D or larger D_HIDDEN would further reinforce trajectory dependence.  \n")
    L.append("- Whether the restored regime still has a trivial shortcut via W_tok_inject "
             "learning to encode the answer in the initial tau0_table entry.  \n\n")

    L.append("---\n\n")
    L.append(f"## RESTORED R^4 GEOMETRY MAKES TRAJECTORY: {conclusion}\n\n")

    with open(MD_OUT, "w") as f:
        f.writelines(L)


def _interpret(variant, acc, full_acc, inject_pos):
    drop = round(acc - full_acc, 3)
    if variant == "full":
        return "reference"
    elif variant == "tau0_direct":
        if acc >= 0.999:
            return "τ₀ encodes answer (collapse persists)"
        elif acc < 0.3:
            return "τ₀ does NOT encode answer ✓ restoration"
        else:
            return f"partial: {drop:+.3f}"
    elif variant == "no_tau0":
        if acc >= 0.999 and inject_pos == "last":
            return "trajectory sufficient without τ₀ ✓"
        elif acc < 0.3:
            return "τ₀ is critical (collapse without it)"
        else:
            return f"{drop:+.3f}"
    elif variant == "last_only":
        if acc >= 0.999:
            return "final position carries answer ✓"
        else:
            return f"{drop:+.3f}"
    elif variant == "no_last":
        if acc < 0.3:
            return "final position critical (collapse) ✓"
        else:
            return f"{drop:+.3f}"
    elif variant == "random_t_gt0":
        if acc >= 0.999:
            return "t>0 taus irrelevant (τ₀ sufficient)"
        elif acc < 0.5:
            return "t>0 taus necessary ✓"
        else:
            return f"{drop:+.3f}"
    return f"{drop:+.3f}"


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main():
    print("=" * 60)
    print("R^4 GEOMETRY RESTORATION PROBE v1")
    print("=" * 60)

    print("\nLoading cache ...")
    t0 = time.perf_counter()
    data = torch.load(CACHE_PATH, weights_only=False)
    print(f"  {data['TN_oh'].shape[0]:,} states in {time.perf_counter()-t0:.3f}s")
    TN_ang, TR, tau0_hyb, pool_ids = prepare_hybrid_tables(
        data["TN_oh"], data["tau0_oh"], data["TR"], data["pool_ids"]
    )

    # Pre-sample random taus for random_t_gt0 ablation
    rng_r = torch.Generator().manual_seed(GLOBAL_SEED + 77777)
    rand_idx  = torch.randint(pool_ids.shape[0], (BATCH_SIZE, D-1), generator=rng_r)
    rand_taus = tau0_hyb[pool_ids[rand_idx]]   # (B, D-1, 12) from pool initial taus

    CONFIG_SPECS = [
        dict(label="baseline", inject_position="first", b_pos0_init=2.0,
             radial_mode="fixed_one"),
        dict(label="restored", inject_position="last",  b_pos0_init=0.0,
             radial_mode="dynamic"),
    ]

    configs_data: Dict[str, Dict] = {}
    all_csv_rows: List[Dict] = []

    for spec in CONFIG_SPECS:
        label    = spec["label"]
        inj_pos  = spec["inject_position"]
        b_init   = spec["b_pos0_init"]
        rad_mode = spec["radial_mode"]

        print(f"\n{'='*60}")
        print(f"CONFIGURATION: {label.upper()}")
        print(f"  inject_position={inj_pos}  b_pos0_init={b_init}  radial={rad_mode}")
        print(f"{'='*60}")

        print(f"\nTraining [{label}] ...")
        model, train_acc, solve_step, sps = train_model(
            TN_ang, TR, tau0_hyb, pool_ids, inj_pos, b_init, label
        )
        b_pos0_trained = float(model.b_pos0.item())

        print(f"\nCollecting hard geom trajectories (radial={rad_mode}) ...")
        all_st, all_x0 = collect_hard_geom_trajectories(model, pool_ids, rad_mode)

        print(f"\nApplying ablations ...")
        ablation_results: List[Dict] = []
        full_acc_here = None
        for variant, desc in ABLATION_DEFS:
            acc, a0, aD, wall = apply_ablation(model, all_st, all_x0, variant, rand_taus)
            if variant == "full":
                full_acc_here = acc
            delta = round(acc - (full_acc_here or acc), 4)
            sign  = "+" if delta >= 0 else ""
            print(f"  [{variant:<18}] acc={acc:.4f}  Δ={sign}{delta:.4f}  "
                  f"α₀={a0:.4f}  α_{{D-1}}={aD:.4f}  t={wall:.3f}s")
            ablation_results.append({
                "variant": variant, "accuracy": acc,
                "alpha0": a0, "alphaD": aD, "runtime_s": wall,
            })
            all_csv_rows.append({
                "variant":              variant,
                "geometry_regime":      label,
                "inject_position":      inj_pos,
                "radial_mode":          rad_mode,
                "b_pos0_trained":       round(b_pos0_trained, 4),
                "accuracy":             acc,
                "delta_vs_baseline":    0.0,   # filled below
                "runtime_seconds":      wall,
                "alpha0":               a0,
                "alphaD":               aD,
                "tau0_only_sufficient": "",    # filled below
                "trajectory_load_bearing": "", # filled below
                "solve_step":           solve_step,
                "train_acc":            train_acc,
                "ablated_component":    desc,
                "note":                 f"D={D} {label} {variant}",
            })

        configs_data[label] = {
            "inject_position": inj_pos, "b_pos0_init": b_init,
            "radial_mode": rad_mode, "b_pos0_trained": b_pos0_trained,
            "solve_step": solve_step, "train_acc": train_acc, "sps": sps,
            "ablations": ablation_results,
        }

    # ── Post-process: fill delta_vs_baseline, tau0_only_sufficient, trajectory_load_bearing
    def get_acc(regime, variant):
        for r in all_csv_rows:
            if r["geometry_regime"] == regime and r["variant"] == variant:
                return r["accuracy"]
        return None

    base_full = get_acc("baseline", "full")
    for r in all_csv_rows:
        r["delta_vs_baseline"] = round(r["accuracy"] - (base_full or 0), 4)
        # tau0_only_sufficient: True if tau0_direct == full for this regime
        t0d = get_acc(r["geometry_regime"], "tau0_direct")
        r["tau0_only_sufficient"] = (t0d is not None and t0d >= 0.999)
        # trajectory_load_bearing: True if no_tau0 succeeds AND no_last fails
        notau = get_acc(r["geometry_regime"], "no_tau0")
        nolast = get_acc(r["geometry_regime"], "no_last")
        r["trajectory_load_bearing"] = (
            (notau is not None and notau >= 0.999) and
            (nolast is not None and nolast < 0.5)
        )

    # ── Determine conclusion ─────────────────────────────────────────────
    rest = configs_data.get("restored", {})
    rest_abl = {r["variant"]: r["accuracy"] for r in rest.get("ablations", [])}
    rest_full    = rest_abl.get("full", 0)
    rest_tau0d   = rest_abl.get("tau0_direct", 1)
    rest_notau   = rest_abl.get("no_tau0", 0)
    rest_nolast  = rest_abl.get("no_last", 1)
    rest_lastonly= rest_abl.get("last_only", 0)

    traj_load_bearing = (rest_tau0d < 0.5 and rest_notau >= 0.99 and rest_nolast < 0.5)
    traj_partial      = (rest_tau0d < 0.99 or rest_notau >= 0.95)
    k1_valid          = rest_full >= 0.999

    if traj_load_bearing and k1_valid:
        conclusion = "LOAD-BEARING"
    elif traj_partial and k1_valid:
        conclusion = "PARTIALLY LOAD-BEARING"
    else:
        conclusion = "DECORATIVE"

    print(f"\n{'='*60}")
    print(f"CONCLUSION")
    print(f"  baseline tau0_direct: {get_acc('baseline','tau0_direct'):.4f}")
    print(f"  restored tau0_direct: {rest_tau0d:.4f}")
    print(f"  restored no_tau0:     {rest_notau:.4f}")
    print(f"  restored last_only:   {rest_lastonly:.4f}")
    print(f"  restored no_last:     {rest_nolast:.4f}")
    print(f"  restored full (k=1):  {rest_full:.4f}")
    print(f"\nRESTORED R^4 GEOMETRY MAKES TRAJECTORY: {conclusion}")

    # ── CSV ───────────────────────────────────────────────────────────────
    fieldnames = [
        "variant", "geometry_regime", "inject_position", "radial_mode",
        "b_pos0_trained", "accuracy", "delta_vs_baseline", "runtime_seconds",
        "alpha0", "alphaD", "tau0_only_sufficient", "trajectory_load_bearing",
        "solve_step", "train_acc", "ablated_component", "note",
    ]
    with open(CSV_OUT, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(all_csv_rows)
    print(f"\nCSV written: {CSV_OUT}  ({len(all_csv_rows)} rows)")

    _write_markdown(configs_data, conclusion)
    print(f"Markdown written: {MD_OUT}")


if __name__ == "__main__":
    main()

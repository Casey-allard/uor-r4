"""run_dynamic_decoding_language_probe_v1.py

DYNAMIC DECODING LANGUAGE PROBE v1
====================================

BRANCH: dynamic_decoding_language_probe_v1

CANONICAL FILES:
  - results/prime_transport_recursive_system/radial_ratio_revalidation_probe_v1.csv
  - results/prime_transport_recursive_system/train_free_geometric_probe_v1.csv
  - tools/prime_transport/run_radial_ratio_revalidation_probe_v1.py

CONTRAST FILES (for historical comparison only):
  - results/prime_transport_recursive_system/prime_transport_router_reintegration_v6_torch.csv
  - tools/prime_transport/run_router_reintegration_v6_torch.py

═══════════════════════════════════════════════════════════════════════════════
MECHANISM LOCK (MANDATORY PRE-CODE — completed before any coding)
═══════════════════════════════════════════════════════════════════════════════

1. EXACT OLDER MECHANISM — minimal candidate for active comparison-language
   formation:

   RouterV6.forward() in run_router_reintegration_v6_torch.py.
   The one essential piece: at each step t, a soft routing weight vector w is
   computed from MLP([emb(tok_t) | tau_prev]), then:

     base = einsum("bi,bij->bj", w, TN_ang[sids])   # (B, d_ang)

   This is the comparison step: all 6 operator outcomes are simultaneously
   considered and blended in proportion to router confidence.  The result has
   direction and norm determined by BOTH current state AND current token.
   This is the "Rosetta stone" formation: tau dynamically encodes which
   operator candidates are aligned, producing input-specific radial structure
   that the static/deterministic stripped system cannot produce.

2. REQUIRED PARTS:
   - Operator angular lookup tables: TN_ang (N, 6, d_ang)  [from cache]
   - Token embedding W_emb (VOCAB → D_EMB)  [fixed random, no training]
   - Router MLP W1, W2  [fixed random, no training]
   - D-step loop with soft-blend update: base = einsum(w, TN_ang[sids])
   - Hard-argmax state transition for state tracking

3. NOT REQUIRED (must remain removed):
   - Training / backprop / gradient descent
   - Trajectory attention readout (W_attn, v_attn)
   - Prediction head (W_pred, b_pred)
   - W_tok_inject injection at t=0
   - Gumbel noise (use sharp eval-mode softmax)
   - BFS warm-up (already cached in state_tables_full.pt)

4. RADIAL QUANTITIES MEASURED:
   - full_radius:    ||tau_ang||   (12-dim angular vector, all 6 harmonic pairs)
   - h3_radius:      ||tau[:, 10:12]||   (H3 pair — class carrier)
   - h2_radius:      ||tau[:, 8:10]||
   - block3_ang_radius: ||tau[:, 6:12]||
   - ratio:          subspace_radius / full_radius

5. DIRECTIONAL QUANTITIES MEASURED:
   - h3_cosine:      dot(tau_t[:,10:12], tau_init[:,10:12]) / (||a||·||b||)
   - full_cosine:    cosine_similarity(tau_t, tau_init)
   - phase_delta:    ||tau_t - tau_{t-1}||  (step-to-step change)

6. EVIDENCE FOR dynamic decoding-language formation:
   - Radial lengths vary across samples (input-specific) AND timesteps
     (phase-specific) in CASE B but are constant in CASE A
   - CASE C: matched inputs have statistically different radial quantities than
     mismatched inputs
   - CASE D: samples where H3 class is recoverable have statistically different
     radial quantities than failure cases
   - Variation must be PREDICTIVE, not just random noise

7. EVIDENCE AGAINST:
   - Radial variation in CASE B is purely structural / operator-diversity-driven
     with no predictive power
   - CASE C shows no significant matched vs mismatched difference
   - CASE D shows no correlation between radial quantities and recovery success

═══════════════════════════════════════════════════════════════════════════════
GEOMETRY LOCK (mandatory per prompt_contract_v3.md)
═══════════════════════════════════════════════════════════════════════════════

State representation:
  tau_ang: 12-dimensional angular vector
    Block 0 (s=0,e=2,m=2,n_h=1):   h1 at [0,1]
    Block 1 (s=2,e=7,m=5,n_h=1):   h1 at [2,3]
    Block 2 (s=7,e=9,m=2,n_h=1):   h1 at [4,5]
    Block 3 (s=9,e=21,m=12,n_h=3): h1 at [6,7], h2 at [8,9], h3 at [10,11]

Task geometry:
  VOCAB=4 classes, dominant block period=12, n_h=3 harmonics.
  H3 (indices 10,11) encodes class identity (period 4 = VOCAB).
  H2 (indices 8,9) encodes k%6 (period 6).
  After apply_anchor_two_i: tau[:,10]=-sin(πk/2), tau[:,11]=cos(πk/2).

CASES:
  CASE A — stripped baseline (deterministic + eps_high=1.0)
  CASE B — uniform soft blend (equal weight to all 6 operators, no MLP)
  CASE B_rc — random-conditioned soft blend (random fixed MLP, no training)
  CASE C_matched — CASE_B_rc with token matching state class
  CASE C_mismatched — CASE_B_rc with wrong-class token
  CASE D_success — CASE_B_rc samples where H3 class survives
  CASE D_failure — CASE_B_rc samples where H3 class is lost
"""

from __future__ import annotations

import csv
import math
import time
from pathlib import Path
from typing import Dict, List, Optional, Tuple

import numpy as np
import torch
import torch.nn.functional as F

# ═══════════════════════════════════════════════════════════════════════
# Paths
# ═══════════════════════════════════════════════════════════════════════
REPO_ROOT   = Path(__file__).resolve().parents[2]
RESULTS_DIR = REPO_ROOT / "results" / "prime_transport_recursive_system"
DOCS_DIR    = REPO_ROOT / "docs" / "research"
CACHE_PATH  = RESULTS_DIR / "state_cache" / "state_tables_full.pt"
CSV_OUT     = RESULTS_DIR / "dynamic_decoding_language_probe_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_dynamic_decoding_language_probe_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Constants — identical to canonical probes
# ═══════════════════════════════════════════════════════════════════════
GLOBAL_SEED   = 42
VOCAB         = 4
BLOCKS_A      = [(0, 2, 2, 1), (2, 7, 5, 1), (7, 9, 2, 1), (9, 21, 12, 3)]
TASK_A_CYC_S  = 9
TASK_A_CYC_E  = 21
N_EVAL        = 2048
D             = 20    # number of comparison steps (same as canonical)
N_OPS         = 6     # number of operators (same as v6)

# MLP for CASE B_rc (minimal comparison router)
D_EMB         = 4     # token embedding dim (same as v6)
D_HIDDEN      = 32    # hidden dim (same as v6)
INIT_SCALE    = 0.05  # weight init scale (same as v6)

# Harmonic indices (block-3 starts at ai=6)
H1_IDX0, H1_IDX1 = 6,  7
H2_IDX0, H2_IDX1 = 8,  9
H3_IDX0, H3_IDX1 = 10, 11

TASK_VARIANTS = [
    ("original_s42", 42, [0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3], 0),
]

CSV_FIELDS = [
    "case", "variant", "timestep",
    "full_radius_mean", "full_radius_std",
    "subspace", "subspace_radius_mean", "subspace_radius_std",
    "ratio_mean", "ratio_std",
    "direction_metric_mean", "direction_metric_std",
    "match_condition", "success_condition",
    "notes",
]

# ═══════════════════════════════════════════════════════════════════════
# Geometry helpers — identical to canonical probes
# ═══════════════════════════════════════════════════════════════════════

def geom_dims(blocks) -> Tuple[int, int, int]:
    n_pairs  = sum(n_h for (_, _, _, n_h) in blocks)
    d_ang    = 2 * n_pairs
    n_blocks = len(blocks)
    return d_ang, n_pairs, n_blocks


def convert_onehot_to_angular_multi(onehot: torch.Tensor, blocks) -> torch.Tensor:
    shape = onehot.shape[:-1]
    d_ang = sum(2 * n_h for (_, _, _, n_h) in blocks)
    out   = torch.zeros(*shape, d_ang)
    ai    = 0
    for s, e, m, n_h in blocks:
        k_idx = onehot[..., s:e].argmax(dim=-1).float()
        for harm in range(1, n_h + 1):
            angle            = 2.0 * math.pi * harm * k_idx / float(m)
            out[..., ai]     = torch.cos(angle)
            out[..., ai + 1] = torch.sin(angle)
            ai += 2
    return out


def apply_anchor_two_i(tau0_ang: torch.Tensor, n_pairs: int) -> torch.Tensor:
    out = tau0_ang.clone()
    for p in range(n_pairs):
        c = tau0_ang[..., 2 * p].clone()
        s = tau0_ang[..., 2 * p + 1].clone()
        out[..., 2 * p]     = -s
        out[..., 2 * p + 1] =  c
    return out


def apply_split_transport(
        base: torch.Tensor,
        tau_prev: torch.Tensor,
        blocks,
        eps_high: float,
) -> torch.Tensor:
    ang_parts: List[torch.Tensor] = []
    mags:      List[torch.Tensor] = []
    ai = 0
    for _, _, _, n_h in blocks:
        fund = base[:, ai:ai + 2]
        mag  = fund.norm(dim=1, keepdim=True).clamp(1e-8)
        mags.append(mag)
        ang_parts.append(fund / mag)
        for h_idx in range(1, n_h):
            new_pair  = base[:, ai + h_idx * 2 : ai + h_idx * 2 + 2] / mag
            prev_pair = tau_prev[:, ai + h_idx * 2 : ai + h_idx * 2 + 2]
            blended   = (1.0 - eps_high) * new_pair + eps_high * prev_pair
            ang_parts.append(blended)
        ai += n_h * 2
    return torch.cat(ang_parts + mags, dim=1)


def prepare_tables(TN_oh, tau0_oh, TR, pool_ids, blocks):
    d_ang, n_pairs, _ = geom_dims(blocks)
    TN_ang   = convert_onehot_to_angular_multi(TN_oh, blocks)
    tau0_ang = convert_onehot_to_angular_multi(tau0_oh, blocks)
    tau0_ang = apply_anchor_two_i(tau0_ang, n_pairs)
    tau0_hyb = torch.cat(
        [tau0_ang, torch.ones(tau0_ang.shape[0], len(blocks))], dim=1
    )
    return TN_ang, TR, tau0_hyb, pool_ids


def build_class_table(tau0_oh, cyc_start, cyc_end, partition):
    idx = tau0_oh[:, cyc_start:cyc_end].argmax(dim=-1)
    lut = torch.tensor(partition, dtype=torch.long)
    return lut[idx]


# ═══════════════════════════════════════════════════════════════════════
# Identity readout helpers — train-free, geometric only
# ═══════════════════════════════════════════════════════════════════════

def h3_class(tau: torch.Tensor, partition_offset: int) -> torch.Tensor:
    """k%4 from H3 phase. Works on tau_ang (12-dim) directly."""
    angle  = torch.atan2(tau[:, H3_IDX0], tau[:, H3_IDX1])
    k_mod4 = torch.round(-angle / (math.pi / 2)).long() % VOCAB
    return (k_mod4 + partition_offset) % VOCAB


# ═══════════════════════════════════════════════════════════════════════
# Radial and directional measurement helpers
# ═══════════════════════════════════════════════════════════════════════

def _safe_cos_sim(va: torch.Tensor, vb: torch.Tensor) -> torch.Tensor:
    """Cosine similarity between rows of va and vb (handles zero-norm gracefully)."""
    na = va.norm(dim=1, keepdim=True).clamp(1e-12)
    nb = vb.norm(dim=1, keepdim=True).clamp(1e-12)
    return (va / na * vb / nb).sum(dim=1)


def measure_radii_ang(tau: torch.Tensor) -> Dict[str, torch.Tensor]:
    """
    Measure radial lengths for angular representation (12-dim).

    Returns dict of per-sample scalar tensors (B,).
    """
    full_r        = tau.norm(dim=1)
    h3_r          = tau[:, H3_IDX0:H3_IDX1+1].norm(dim=1)
    h2_r          = tau[:, H2_IDX0:H2_IDX1+1].norm(dim=1)
    block3_ang_r  = tau[:, H1_IDX0:H3_IDX1+1].norm(dim=1)
    return {
        "full":        full_r,
        "h3":          h3_r,
        "h2":          h2_r,
        "block3_ang":  block3_ang_r,
    }


# ═══════════════════════════════════════════════════════════════════════
# CASE A — stripped baseline trajectory
# ═══════════════════════════════════════════════════════════════════════

def run_case_a(
        sids: torch.Tensor,
        tau0_hyb: torch.Tensor,
        TN_ang: torch.Tensor,
        TR: torch.Tensor,
        D: int,
        d_ang: int,
        blocks,
) -> List[torch.Tensor]:
    """
    Deterministic stripped system: hard angular-similarity routing +
    apply_split_transport(eps_high=1.0).

    Returns list of tau_ang snapshots at t=0,1,...,D.
    tau_ang = first d_ang dims of tau_hyb.
    """
    B         = sids.shape[0]
    tau_prev  = tau0_hyb[sids].clone()
    sids_loop = sids.clone()
    steps = [tau_prev[:, :d_ang].clone()]

    for _ in range(D):
        tn      = TN_ang[sids_loop]                # (B, N_OPS, d_ang)
        cur_dir = tau_prev[:, :d_ang]
        ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)
        best_op = ang_sim.argmax(dim=1)
        best_ang = tn.gather(
            1, best_op.view(B, 1, 1).expand(B, 1, d_ang)).squeeze(1)
        tau_prev  = apply_split_transport(best_ang, tau_prev, blocks, 1.0)
        sids_loop = TR[sids_loop].gather(1, best_op.unsqueeze(1)).squeeze(1)
        steps.append(tau_prev[:, :d_ang].clone())

    return steps


# ═══════════════════════════════════════════════════════════════════════
# CASE B — uniform soft blend (comparison without conditioning)
# ═══════════════════════════════════════════════════════════════════════

def run_case_b_uniform(
        sids: torch.Tensor,
        tau0_ang: torch.Tensor,
        TN_ang: torch.Tensor,
        TR: torch.Tensor,
        D: int,
        d_ang: int,
) -> List[torch.Tensor]:
    """
    Minimal dynamic comparison: uniform equal weight across all N_OPS operators.

    base_t = TN_ang[sids_t].mean(dim=1)   — average of all 6 operator outcomes
    tau_{t+1} = base_t   (raw, no normalization, no eps_high blending)

    State transitions via hard argmax of angular similarity (for state tracking).
    This is the absolute minimum comparison process: no routing MLP, no token
    conditioning.  Even here, the weighted blend produces input-specific radii.

    Returns list of tau_ang snapshots at t=0,1,...,D.
    """
    B         = sids.shape[0]
    tau_prev  = tau0_ang[sids].clone()
    sids_loop = sids.clone()
    steps = [tau_prev.clone()]

    for _ in range(D):
        tn_batch  = TN_ang[sids_loop]                      # (B, N_OPS, d_ang)
        base      = tn_batch.mean(dim=1)                   # (B, d_ang)
        # State transition: hard argmax over angular similarity for tracking
        ang_sim   = torch.einsum("bi,bji->bj", tau_prev, tn_batch)
        best_op   = ang_sim.argmax(dim=1)
        sids_loop = TR[sids_loop].gather(1, best_op.unsqueeze(1)).squeeze(1)
        tau_prev  = base
        steps.append(tau_prev.clone())

    return steps


# ═══════════════════════════════════════════════════════════════════════
# CASE B_rc — random-conditioned soft blend (minimal MLP, no training)
# ═══════════════════════════════════════════════════════════════════════

class MinimalComparisonRouter:
    """
    Minimal reconstruction of the RouterV6 comparison step.

    Architecture (fixed random weights, NO training):
      emb  = W_emb[tok]                        # (B, D_EMB)
      h_in = cat([emb, tau_prev], dim=1)        # (B, D_EMB + d_ang)
      h    = tanh(h_in @ W1 + b1)              # (B, D_HIDDEN)
      logits = h @ W2 + b2                      # (B, N_OPS)
      w    = softmax(logits / 0.05)             # (B, N_OPS) — sharp, eval mode
      base = einsum("bi,bij->bj", w, TN_ang)   # (B, d_ang)

    This is the narrowest slice of RouterV6 that still performs:
      - Input-dependent comparison (w depends on tok + tau_prev)
      - Multi-step state updating (each step uses updated tau_prev)
      - Production of a non-unit-norm blended angular vector
    """
    def __init__(self, d_ang: int, seed: int = GLOBAL_SEED):
        gen = torch.Generator().manual_seed(seed)
        D_IN = D_EMB + d_ang

        def rp(*shape: int) -> torch.Tensor:
            return torch.empty(*shape).normal_(0.0, INIT_SCALE, generator=gen)

        self.W_emb = rp(VOCAB, D_EMB)            # (4, 4)
        self.W1    = rp(D_IN, D_HIDDEN)          # (16, 32)
        self.b1    = torch.zeros(D_HIDDEN)
        self.W2    = rp(D_HIDDEN, N_OPS)         # (32, 6)
        self.b2    = torch.zeros(N_OPS)

    def route_weights(
        self,
        tok: torch.Tensor,    # (B,) int64
        tau_prev: torch.Tensor,  # (B, d_ang)
    ) -> torch.Tensor:        # (B, N_OPS)
        emb    = self.W_emb[tok]                          # (B, D_EMB)
        h_in   = torch.cat([emb, tau_prev], dim=1)        # (B, D_IN)
        h      = torch.tanh(h_in @ self.W1 + self.b1)    # (B, D_HIDDEN)
        logits = h @ self.W2 + self.b2                    # (B, N_OPS)
        w      = torch.softmax(logits / 0.05, dim=1)     # sharp eval mode
        return w


def run_case_b_rc(
        sids: torch.Tensor,
        tau0_ang: torch.Tensor,
        TN_ang: torch.Tensor,
        TR: torch.Tensor,
        tokens: torch.Tensor,     # (B, D) int64 — token sequence
        router: MinimalComparisonRouter,
        D: int,
        d_ang: int,
) -> List[torch.Tensor]:
    """
    Token-conditioned soft blend using fixed random MLP weights.

    base_t = einsum(w_t, TN_ang[sids_t])
    where w_t = softmax(MLP([emb(tok_t) | tau_prev]))

    Returns list of tau_ang snapshots at t=0,1,...,D.
    """
    B         = sids.shape[0]
    tau_prev  = tau0_ang[sids].clone()
    sids_loop = sids.clone()
    steps = [tau_prev.clone()]

    for t in range(D):
        tok_t    = tokens[:, t]                            # (B,)
        tn_batch = TN_ang[sids_loop]                       # (B, N_OPS, d_ang)
        w        = router.route_weights(tok_t, tau_prev)   # (B, N_OPS)
        base     = torch.einsum("bi,bij->bj", w, tn_batch) # (B, d_ang)
        # State transition: hard argmax of router weights
        best_op  = w.argmax(dim=1)
        sids_loop = TR[sids_loop].gather(1, best_op.unsqueeze(1)).squeeze(1)
        tau_prev  = base
        steps.append(tau_prev.clone())

    return steps


# ═══════════════════════════════════════════════════════════════════════
# Measurement collector
# ═══════════════════════════════════════════════════════════════════════

def collect_rows(
    case: str,
    variant: str,
    steps: List[torch.Tensor],    # t=0..D tau_ang snapshots
    tau_init: torch.Tensor,       # (B, 12) — reference for cosine
    match_condition: str,
    success_mask: Optional[torch.Tensor],  # (B,) bool or None
    success_label: str = "all",   # explicit label: "all", "success", or "failure"
) -> List[Dict]:
    """
    Build CSV rows for all timesteps and all measured subspaces.
    Each row is a per-timestep aggregate over the sample batch (mean±std).
    """
    rows: List[Dict] = []
    subspace_names = ["full", "h3", "h2", "block3_ang"]

    for t, tau_t in enumerate(steps):
        # Optionally filter to success or failure subset
        if success_mask is not None:
            tau_t     = tau_t[success_mask]
            tau_ref   = tau_init[success_mask]
            sc_label  = success_label
        else:
            tau_ref   = tau_init
            sc_label  = success_label if success_label != "all" else "all"

        radii = measure_radii_ang(tau_t)

        # Direction: H3 cosine similarity to tau_init
        h3_cos  = _safe_cos_sim(tau_t[:, H3_IDX0:H3_IDX1+1],
                                 tau_ref[:, H3_IDX0:H3_IDX1+1])
        # Full cosine similarity
        full_cos = _safe_cos_sim(tau_t, tau_ref)

        # Direction metric = mean of H3 cosine similarity (primary diagnostic)
        dir_mean = float(h3_cos.mean().item())
        dir_std  = float(h3_cos.std().item())

        full_r = radii["full"]

        for sname in subspace_names:
            sr    = radii[sname]
            ratio = (sr / full_r.clamp(1e-12))
            row = {
                "case":                   case,
                "variant":                variant,
                "timestep":               t,
                "full_radius_mean":       round(float(full_r.mean().item()), 8),
                "full_radius_std":        round(float(full_r.std().item()), 8),
                "subspace":               sname,
                "subspace_radius_mean":   round(float(sr.mean().item()), 8),
                "subspace_radius_std":    round(float(sr.std().item()), 8),
                "ratio_mean":             round(float(ratio.mean().item()), 8),
                "ratio_std":              round(float(ratio.std().item()), 8),
                "direction_metric_mean":  round(dir_mean, 8),
                "direction_metric_std":   round(dir_std, 8),
                "match_condition":        match_condition,
                "success_condition":      sc_label,
                "notes": (
                    f"subspace={sname};t={t};n_samples={tau_t.shape[0]};"
                    f"full_cos_mean={float(full_cos.mean().item()):.6f};"
                    f"h3_cos_mean={dir_mean:.6f}"
                ),
            }
            rows.append(row)

    return rows


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════

def main():
    print("=" * 72)
    print("DYNAMIC DECODING LANGUAGE PROBE v1")
    print(f"  D={D} steps, N_EVAL={N_EVAL}, VOCAB={VOCAB}")
    print("=" * 72)

    t_start = time.perf_counter()

    # ── Load state cache ─────────────────────────────────────────────
    print("\nLoading state cache ...", end=" ", flush=True)
    cache        = torch.load(str(CACHE_PATH), map_location="cpu", weights_only=True)
    TN_oh        = cache["TN_oh"]      # (N_states, N_OPS, 21)
    tau0_oh      = cache["tau0_oh"]    # (N_states, 21)
    TR           = cache["TR"]         # (N_states, N_OPS) int64
    pool_ids_raw = cache["pool_ids"]   # (4000,)
    print(f"done. N_states={TN_oh.shape[0]:,}", flush=True)

    TN_ang, TR, tau0_hyb_base, pool_ids = prepare_tables(
        TN_oh, tau0_oh, TR, pool_ids_raw, BLOCKS_A)

    d_ang, n_pairs, n_blocks = geom_dims(BLOCKS_A)
    tau0_ang_base = tau0_hyb_base[:, :d_ang]    # (N_states, 12) — angular only

    print(f"  d_ang={d_ang}, n_pairs={n_pairs}, n_blocks={n_blocks}")
    print(f"  Structural norm in stripped system: sqrt({n_pairs})={math.sqrt(n_pairs):.6f}")

    # Build class table for H3 ground truth
    partition_4   = [0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3]
    class_table_4 = build_class_table(tau0_oh, TASK_A_CYC_S, TASK_A_CYC_E, partition_4)

    # ── Minimal comparison router (fixed random weights, NO training) ──
    router = MinimalComparisonRouter(d_ang=d_ang, seed=GLOBAL_SEED)

    all_rows: List[Dict] = []

    for vname, vseed, vpartition, voffset in TASK_VARIANTS:
        print(f"\n{'─'*72}")
        print(f"Variant: {vname}  (partition_offset={voffset})")

        rng  = torch.Generator().manual_seed(vseed)
        idx  = torch.randint(pool_ids.shape[0], (N_EVAL,), generator=rng)
        sids = pool_ids[idx]

        tau_init_ang = tau0_ang_base[sids].clone()    # (B, 12) angular
        h3_gt        = h3_class(tau_init_ang, voffset)
        x0           = h3_gt.clone()                  # class = token for matched case
        x0_wrong     = (x0 + 1) % VOCAB              # deliberately mismatched token

        print(f"  N={N_EVAL}, H3 class dist: "
              f"{dict([(c, int((h3_gt==c).sum())) for c in range(4)])}")

        # ── CASE A: stripped baseline ─────────────────────────────────
        print("  Running CASE A (stripped baseline) ...", flush=True)
        steps_a = run_case_a(sids, tau0_hyb_base, TN_ang, TR, D, d_ang, BLOCKS_A)
        rows_a  = collect_rows("A_stripped_baseline", vname, steps_a,
                               tau_init_ang, "na", None, "all")
        all_rows.extend(rows_a)

        # Sanity check: radii should be constant
        fr0 = float(steps_a[0].norm(dim=1).mean().item())
        frD = float(steps_a[-1].norm(dim=1).mean().item())
        print(f"    CASE A full_radius: t=0 mean={fr0:.6f}  t={D} mean={frD:.6f}  "
              f"(expected constant ~{math.sqrt(n_pairs):.6f})")

        # ── CASE B: uniform soft blend ────────────────────────────────
        print("  Running CASE B (uniform soft blend) ...", flush=True)
        steps_bu = run_case_b_uniform(sids, tau0_ang_base, TN_ang, TR, D, d_ang)
        rows_bu  = collect_rows("B_uniform_blend", vname, steps_bu,
                                tau_init_ang, "na", None, "all")
        all_rows.extend(rows_bu)

        fr0_bu = float(steps_bu[0].norm(dim=1).mean().item())
        frD_bu = float(steps_bu[-1].norm(dim=1).mean().item())
        fr_std_bu = float(steps_bu[-1].norm(dim=1).std().item())
        print(f"    CASE B full_radius: t=0 mean={fr0_bu:.6f}  t={D} mean={frD_bu:.6f}  "
              f"std={fr_std_bu:.6f}")

        # ── CASE B_rc: token-conditioned soft blend (matched tokens) ──
        # Create matched token sequences: use x0 for all D steps
        tok_matched    = x0.unsqueeze(1).expand(-1, D).clone()    # (B, D)
        tok_mismatched = x0_wrong.unsqueeze(1).expand(-1, D).clone()  # (B, D)

        print("  Running CASE B_rc matched ...", flush=True)
        steps_rc_m = run_case_b_rc(
            sids, tau0_ang_base, TN_ang, TR, tok_matched, router, D, d_ang)
        rows_rc_m  = collect_rows("Brc_conditioned", vname, steps_rc_m,
                                  tau_init_ang, "matched", None, "all")
        all_rows.extend(rows_rc_m)

        print("  Running CASE B_rc mismatched ...", flush=True)
        steps_rc_mm = run_case_b_rc(
            sids, tau0_ang_base, TN_ang, TR, tok_mismatched, router, D, d_ang)
        rows_rc_mm  = collect_rows("Brc_conditioned", vname, steps_rc_mm,
                                   tau_init_ang, "mismatched", None, "all")
        all_rows.extend(rows_rc_mm)

        # CASE C stats
        frD_rc_m  = float(steps_rc_m[-1].norm(dim=1).mean().item())
        frD_rc_mm = float(steps_rc_mm[-1].norm(dim=1).mean().item())
        std_rc_m  = float(steps_rc_m[-1].norm(dim=1).std().item())
        std_rc_mm = float(steps_rc_mm[-1].norm(dim=1).std().item())
        h3cos_m   = float(_safe_cos_sim(
            steps_rc_m[-1][:, H3_IDX0:H3_IDX1+1],
            tau_init_ang[:, H3_IDX0:H3_IDX1+1]).mean().item())
        h3cos_mm  = float(_safe_cos_sim(
            steps_rc_mm[-1][:, H3_IDX0:H3_IDX1+1],
            tau_init_ang[:, H3_IDX0:H3_IDX1+1]).mean().item())
        print(f"    CASE C matched:    full_radius mean={frD_rc_m:.6f}  std={std_rc_m:.6f}  "
              f"h3_cos={h3cos_m:.6f}")
        print(f"    CASE C mismatched: full_radius mean={frD_rc_mm:.6f}  std={std_rc_mm:.6f}  "
              f"h3_cos={h3cos_mm:.6f}")

        # ── CASE D: success vs failure within matched condition ────────
        # Success proxy: H3 phase class is still recoverable from tau_final
        tau_final_m = steps_rc_m[-1]    # (B, 12) at step D
        h3_final_m  = h3_class(tau_final_m, voffset)
        success_mask = (h3_final_m == h3_gt)
        failure_mask = ~success_mask
        n_success = int(success_mask.sum().item())
        n_failure = int(failure_mask.sum().item())
        print(f"    CASE D: success={n_success}  failure={n_failure}  "
              f"(H3 class recovery after D={D} matched steps)")

        if n_success >= 10:
            rows_ds = collect_rows("D_success", vname, steps_rc_m,
                                   tau_init_ang, "matched", success_mask, "success")
            all_rows.extend(rows_ds)

        if n_failure >= 10:
            rows_df = collect_rows("D_failure", vname, steps_rc_m,
                                   tau_init_ang, "matched", failure_mask, "failure")
            all_rows.extend(rows_df)

        # Also run D_success/failure for mismatched condition
        tau_final_mm = steps_rc_mm[-1]
        h3_final_mm  = h3_class(tau_final_mm, voffset)
        success_mm   = (h3_final_mm == h3_gt)
        failure_mm   = ~success_mm
        n_succ_mm = int(success_mm.sum().item())
        n_fail_mm = int(failure_mm.sum().item())
        print(f"    CASE D (mismatched): success={n_succ_mm}  failure={n_fail_mm}")

        if n_succ_mm >= 10:
            rows_ds_mm = collect_rows("D_success_mismatch", vname, steps_rc_mm,
                                      tau_init_ang, "mismatched", success_mm, "success")
            all_rows.extend(rows_ds_mm)

        if n_fail_mm >= 10:
            rows_df_mm = collect_rows("D_failure_mismatch", vname, steps_rc_mm,
                                      tau_init_ang, "mismatched", failure_mm, "failure")
            all_rows.extend(rows_df_mm)

        # ── Per-case summary print ─────────────────────────────────────
        # H3 class recovery rates
        h3_a_final   = h3_class(steps_a[-1], voffset)
        h3_bu_final  = h3_class(steps_bu[-1], voffset)
        acc_a  = float((h3_a_final  == h3_gt).float().mean().item())
        acc_bu = float((h3_bu_final == h3_gt).float().mean().item())
        acc_m  = float((h3_final_m  == h3_gt).float().mean().item())
        acc_mm = float((h3_final_mm == h3_gt).float().mean().item())
        print(f"\n  H3 class recovery after D={D} steps:")
        print(f"    CASE A  (stripped):    {acc_a:.4f}")
        print(f"    CASE B  (uniform):     {acc_bu:.4f}")
        print(f"    CASE B_rc matched:     {acc_m:.4f}")
        print(f"    CASE B_rc mismatched:  {acc_mm:.4f}")

        # ── Per-case radial summary ────────────────────────────────────
        print(f"\n  Full-radius statistics at final step (t={D}):")
        for case_label, steps_list in [
            ("A_stripped",      steps_a),
            ("B_uniform",       steps_bu),
            ("Brc_matched",     steps_rc_m),
            ("Brc_mismatched",  steps_rc_mm),
        ]:
            fr_final = steps_list[-1].norm(dim=1)
            print(f"    {case_label:22s}: mean={float(fr_final.mean()):.6f}  "
                  f"std={float(fr_final.std()):.6f}  "
                  f"min={float(fr_final.min()):.6f}  "
                  f"max={float(fr_final.max()):.6f}")

    elapsed = time.perf_counter() - t_start

    # ── Write CSV ─────────────────────────────────────────────────────
    with open(CSV_OUT, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=CSV_FIELDS)
        writer.writeheader()
        writer.writerows(all_rows)
    print(f"\nCSV written: {CSV_OUT.name}  ({len(all_rows)} rows)")

    # ── Compute summary stats for MD ──────────────────────────────────
    # Organize by case for quick reporting
    def case_summary(rows: List[Dict], case: str, t_val: int, sname: str) -> Dict:
        subset = [r for r in rows
                  if r["case"] == case and r["timestep"] == t_val
                  and r["subspace"] == sname
                  and r["match_condition"] in ("na", "matched", "mismatched")]
        if not subset:
            return {}
        r = subset[0]
        return {
            "full_r_mean":   r["full_radius_mean"],
            "full_r_std":    r["full_radius_std"],
            "sub_r_mean":    r["subspace_radius_mean"],
            "ratio_mean":    r["ratio_mean"],
            "ratio_std":     r["ratio_std"],
            "dir_mean":      r["direction_metric_mean"],
            "dir_std":       r["direction_metric_std"],
        }

    # ── Write MD ──────────────────────────────────────────────────────
    _write_md(all_rows, elapsed, d_ang, n_pairs)
    print(f"MD  written: {MD_OUT.name}")
    print(f"\nTotal elapsed: {elapsed:.1f}s")
    print("=" * 72)


# ═══════════════════════════════════════════════════════════════════════
# MD writer
# ═══════════════════════════════════════════════════════════════════════

def _extract_scalar(rows: List[Dict], case: str, t_val: int,
                    subspace: str, field: str,
                    match_cond: Optional[str] = None,
                    succ_cond: Optional[str] = None) -> str:
    for r in rows:
        if r["case"] != case:
            continue
        if r["timestep"] != t_val:
            continue
        if r["subspace"] != subspace:
            continue
        if match_cond is not None and r["match_condition"] != match_cond:
            continue
        if succ_cond is not None and r["success_condition"] != succ_cond:
            continue
        return str(r.get(field, "n/a"))
    return "n/a"


def _make_table_stripped_vs_dynamic(rows: List[Dict], D_val: int) -> str:
    """Table 1: stripped baseline vs dynamic comparison at final step."""
    header = (
        "| Case | full_r mean | full_r std | h3_r mean | h3_r std | "
        "ratio mean | ratio std | H3 cos mean | H3 cos std |\n"
        "|------|------------|-----------|----------|---------|"
        "----------|----------|------------|------------|\n"
    )
    cases = [
        ("A_stripped_baseline", "na", "all",   "A: Stripped baseline"),
        ("B_uniform_blend",     "na", "all",   "B: Uniform soft blend"),
        ("Brc_conditioned",     "matched",   "all", "B_rc: Cond. matched"),
        ("Brc_conditioned",     "mismatched","all", "B_rc: Cond. mismatched"),
    ]
    body = ""
    for case_id, match_cond, succ_cond, label in cases:
        fr_m  = _extract_scalar(rows, case_id, D_val, "full",  "full_radius_mean",  match_cond, succ_cond)
        fr_s  = _extract_scalar(rows, case_id, D_val, "full",  "full_radius_std",   match_cond, succ_cond)
        h3_m  = _extract_scalar(rows, case_id, D_val, "h3",    "subspace_radius_mean", match_cond, succ_cond)
        h3_s  = _extract_scalar(rows, case_id, D_val, "h3",    "subspace_radius_std",  match_cond, succ_cond)
        ra_m  = _extract_scalar(rows, case_id, D_val, "h3",    "ratio_mean",        match_cond, succ_cond)
        ra_s  = _extract_scalar(rows, case_id, D_val, "h3",    "ratio_std",         match_cond, succ_cond)
        dc_m  = _extract_scalar(rows, case_id, D_val, "h3",    "direction_metric_mean", match_cond, succ_cond)
        dc_s  = _extract_scalar(rows, case_id, D_val, "h3",    "direction_metric_std",  match_cond, succ_cond)
        body += f"| {label} | {fr_m} | {fr_s} | {h3_m} | {h3_s} | {ra_m} | {ra_s} | {dc_m} | {dc_s} |\n"
    return header + body


def _make_table_matched_vs_mismatched(rows: List[Dict], D_val: int) -> str:
    header = (
        "| Condition | full_r mean | full_r std | ratio mean | ratio std | "
        "H3 cos mean | H3 cos std |\n"
        "|-----------|------------|-----------|----------|---------|"
        "------------|------------|\n"
    )
    body = ""
    for match_cond, label in [("matched", "Matched (correct class token)"),
                               ("mismatched", "Mismatched (wrong class token)")]:
        fr_m  = _extract_scalar(rows, "Brc_conditioned", D_val, "full",  "full_radius_mean",      match_cond, "all")
        fr_s  = _extract_scalar(rows, "Brc_conditioned", D_val, "full",  "full_radius_std",       match_cond, "all")
        ra_m  = _extract_scalar(rows, "Brc_conditioned", D_val, "h3",    "ratio_mean",            match_cond, "all")
        ra_s  = _extract_scalar(rows, "Brc_conditioned", D_val, "h3",    "ratio_std",             match_cond, "all")
        dc_m  = _extract_scalar(rows, "Brc_conditioned", D_val, "h3",    "direction_metric_mean", match_cond, "all")
        dc_s  = _extract_scalar(rows, "Brc_conditioned", D_val, "h3",    "direction_metric_std",  match_cond, "all")
        body += f"| {label} | {fr_m} | {fr_s} | {ra_m} | {ra_s} | {dc_m} | {dc_s} |\n"
    return header + body


def _make_table_success_vs_failure(rows: List[Dict], D_val: int) -> str:
    header = (
        "| Condition | Subset | full_r mean | full_r std | ratio mean | "
        "H3 cos mean |\n"
        "|-----------|--------|------------|-----------|----------|"
        "------------|\n"
    )
    body = ""
    cases_to_show = [
        ("D_success",           "matched",    "success", "Matched / H3 success"),
        ("D_failure",           "matched",    "failure", "Matched / H3 failure"),
        ("D_success_mismatch",  "mismatched", "success", "Mismatched / H3 success"),
        ("D_failure_mismatch",  "mismatched", "failure", "Mismatched / H3 failure"),
    ]
    for case_id, match_cond, succ_cond, label in cases_to_show:
        fr_m = _extract_scalar(rows, case_id, D_val, "full", "full_radius_mean", match_cond, succ_cond)
        fr_s = _extract_scalar(rows, case_id, D_val, "full", "full_radius_std",  match_cond, succ_cond)
        ra_m = _extract_scalar(rows, case_id, D_val, "h3",   "ratio_mean",       match_cond, succ_cond)
        dc_m = _extract_scalar(rows, case_id, D_val, "h3",   "direction_metric_mean", match_cond, succ_cond)
        body += f"| {label} | {succ_cond} | {fr_m} | {fr_s} | {ra_m} | {dc_m} |\n"
    return header + body


def _write_md(rows: List[Dict], elapsed: float, d_ang: int, n_pairs: int):
    D_val = D   # final timestep

    # Gather H3 class recovery rates from rows (stored in notes field)
    # We stored these as printed values; recompute from direction metrics for MD
    # Get full_r std at final step for each case
    def get_std(case, match_cond, t_val=D_val, subspace="full"):
        return _extract_scalar(rows, case, t_val, subspace, "full_radius_std", match_cond, "all")

    stripped_std   = get_std("A_stripped_baseline", "na")
    uniform_std    = get_std("B_uniform_blend",     "na")
    matched_std    = get_std("Brc_conditioned",     "matched")
    mismatched_std = get_std("Brc_conditioned",     "mismatched")

    table1 = _make_table_stripped_vs_dynamic(rows, D_val)
    table2 = _make_table_matched_vs_mismatched(rows, D_val)
    table3 = _make_table_success_vs_failure(rows, D_val)

    md = f"""# Prime Transport — Dynamic Decoding Language Probe v1

**Branch:** `dynamic_decoding_language_probe_v1`

**CANONICAL SOURCES:**
- `results/prime_transport_recursive_system/radial_ratio_revalidation_probe_v1.csv`
- `results/prime_transport_recursive_system/train_free_geometric_probe_v1.csv`

**CONTRAST SOURCES (for historical comparison only):**
- `results/prime_transport_recursive_system/prime_transport_router_reintegration_v6_torch.csv`

---

## 1. Mechanism Lock Summary

### What was minimally reconstructed

The comparison loop from `RouterV6.forward()` (`run_router_reintegration_v6_torch.py`),
stripped to its bare minimum:

```
At each step t (of D total steps):
  emb    = W_emb[tok_t]                       # (B, D_EMB)  — token embedding
  h_in   = cat([emb, tau_prev], dim=1)        # (B, D_EMB + d_ang)
  h      = tanh(h_in @ W1 + b1)              # (B, D_HIDDEN)
  logits = h @ W2 + b2                        # (B, N_OPS)
  w      = softmax(logits / 0.05)             # (B, N_OPS)  — sharp, eval mode
  base   = einsum("bi,bij->bj", w, TN_ang)   # (B, d_ang)  — KEY: soft blend
  tau_{{t+1}} = base
```

This is the "active comparison-language formation" step: all 6 operator
outcomes are compared simultaneously and blended proportionally to the
router's confidence given the current state + token.  The result is a
**non-unit-norm** vector whose norm and direction are **input-specific**.

### What was NOT restored

- Training / backpropagation / optimizer
- Trajectory attention readout (W_attn, v_attn)
- Prediction head (W_pred, b_pred)
- Token injection separate from routing (W_tok_inject)
- Gumbel sampling noise (eval mode: sharp softmax)

### Weights

Fixed random initialization (INIT_SCALE={INIT_SCALE}, seed={GLOBAL_SEED}).
No training. Same scale as v6 but no parameter updates.

---

## 2. Cases Tested

| Case | Description |
|------|-------------|
| A_stripped_baseline | Deterministic hard routing + apply_split_transport(eps_high=1.0) |
| B_uniform_blend | Uniform soft blend over all 6 operators (no router MLP) |
| Brc_conditioned | Token-conditioned soft blend (random fixed MLP) |
| Brc_conditioned (matched) | Correct class token at every step |
| Brc_conditioned (mismatched) | Wrong class token (+1 mod VOCAB) at every step |
| D_success | Matched-condition samples where H3 class survives to step D |
| D_failure | Matched-condition samples where H3 class is lost by step D |
| D_success_mismatch | Mismatched-condition samples where H3 class survives |
| D_failure_mismatch | Mismatched-condition samples where H3 class is lost |

Structural prediction for stripped system:
- All (cos,sin) pairs are unit-norm → `||tau_ang||` = sqrt({n_pairs}) ≈ {math.sqrt(n_pairs):.6f}
- Radial ratios are structural constants — confirmed by radial_ratio_revalidation_probe_v1.

---

## 3. Stripped Baseline vs Dynamic Comparison Process (at step D={D_val})

{table1}

**Key diagnostic:**
- `full_radius_std` for stripped baseline:    {stripped_std}  (expected ≈ 0)
- `full_radius_std` for uniform soft blend:   {uniform_std}
- `full_radius_std` for conditioned matched:  {matched_std}
- `full_radius_std` for conditioned mismatch: {mismatched_std}

A non-zero std in the dynamic cases but not in the stripped case would confirm
that radial variation is produced by the comparison mechanism, not the geometry.

---

## 4. Matched vs Mismatched Conditions (Case B_rc at step D={D_val})

{table2}

---

## 5. Success vs Failure Trajectories (Case D, proxy = H3 class recovery)

{table3}

---

## 6. Do radial/directional quantities become informative only during active comparison-language formation?

### Test criteria

| Criterion | Required observation |
|-----------|---------------------|
| Input-specific radial variation | `full_radius_std` >> 0 in Cases B/B_rc; ≈ 0 in Case A |
| Phase-specific variation | `full_radius` changes across timesteps in Cases B/B_rc; constant in Case A |
| Match-condition discriminability | Matched vs mismatched differ in radius or H3 cosine |
| Predictive for success | Success subset has different radial signature than failure subset |

### Observed evidence

The stripped baseline (CASE A) has `full_radius_std` ≈ {stripped_std} at all timesteps,
confirming the structural constant property established in radial_ratio_revalidation_probe_v1.

The uniform soft blend (CASE B) produces `full_radius_std` = {uniform_std} at step D={D_val}.
This variation arises because the mean of 6 unit-norm vectors is not unit-norm; the degree
of constructive/destructive interference varies by state.  This is **architectural**, not
dynamic: it is fully determined by which operators are available at each state, independent
of any input token.

The token-conditioned blend (CASE B_rc) with matched tokens produces std = {matched_std}
and with mismatched tokens produces std = {mismatched_std}.  The **difference** between
these two reflects genuine input-conditioning of the comparison process.

Whether this difference is large enough to constitute a discriminative signal, and whether
it predicts H3 class recovery, is the key measurement (see Tables 3 and 4 above).

---

## 7. Final Conclusion

### Evidence summary

| Property | CASE A | CASE B | CASE B_rc | Conclusion |
|----------|--------|--------|-----------|------------|
| Radial variation across samples | None (structural const) | Present (architectural) | Present (input+arch) | B/Brc vary; A does not |
| Variation across timesteps | None | Present | Present | B/Brc evolve; A does not |
| Matched vs mismatched differ | N/A | N/A | See Table 3 | Needs table values |
| Predicts class recovery | N/A | N/A | See Table 4 | Needs table values |

### Interpretation

The soft-blend comparison mechanism does produce radial variation that is absent
in the stripped system.  The question is whether this variation is:
(a) purely structural/architectural (same regardless of input class), or
(b) genuinely dynamic (depends on alignment between token and state).

If (a): WEAK_SUPPORT at best — the variation is real but not informative beyond
        the operator-diversity structure of the state graph.

If (b): possible STRONG_SUPPORT — but only if the matched/mismatched and
        success/failure tables show significant and consistent differences.

The H3 direction metric (cosine between tau_final[10:12] and tau_init[10:12])
is the primary diagnostic.  If it differs significantly between matched and
mismatched, and between success and failure, that constitutes evidence that
the comparison process is forming an input-specific "decoding language."

DYNAMIC DECODING LANGUAGE STATUS: **SEE_BELOW**

(Replace with one of: NO_SUPPORT / WEAK_SUPPORT / STRONG_SUPPORT after reading
Tables 3 and 4.)

---

*Generated by `run_dynamic_decoding_language_probe_v1.py` in {elapsed:.1f}s.*
"""

    # Post-process: replace SEE_BELOW with actual verdict based on data
    # Read key values
    def get_val(case, t_val, subspace, field, match_cond="na", succ_cond="all"):
        v = _extract_scalar(rows, case, t_val, subspace, field, match_cond, succ_cond)
        try:
            return float(v)
        except Exception:
            return None

    stripped_full_std   = get_val("A_stripped_baseline", D_val, "full", "full_radius_std")
    dynamic_full_std    = get_val("B_uniform_blend",     D_val, "full", "full_radius_std")
    matched_h3cos_mean  = get_val("Brc_conditioned",     D_val, "h3",  "direction_metric_mean", "matched")
    mismatch_h3cos_mean = get_val("Brc_conditioned",     D_val, "h3",  "direction_metric_mean", "mismatched")
    succ_h3cos          = get_val("D_success",           D_val, "h3",  "direction_metric_mean", "matched", "success")
    fail_h3cos          = get_val("D_failure",           D_val, "h3",  "direction_metric_mean", "matched", "failure")

    # Decision logic: use full_radius difference as primary discriminator
    # (H3 cosine is tautologically linked to success/failure definition; full radius
    #  is an independent radial quantity — if it tracks success, that's genuine signal)
    if (dynamic_full_std is not None and stripped_full_std is not None and
            dynamic_full_std < 0.01):
        verdict = "NO_SUPPORT"
        reason  = ("Radial variation in dynamic cases is negligible (< 0.01) — "
                   "no meaningful difference from stripped baseline.")
    elif (matched_h3cos_mean is not None and mismatch_h3cos_mean is not None and
          succ_h3cos is not None and fail_h3cos is not None):
        # Primary test: matched vs mismatched difference in H3 cosine direction
        cos_match_diff = abs(matched_h3cos_mean - mismatch_h3cos_mean)
        # Secondary test: success vs failure full_radius difference (non-tautological)
        succ_full_r  = get_val("D_success", D_val, "full", "full_radius_mean", "matched", "success")
        fail_full_r  = get_val("D_failure", D_val, "full", "full_radius_mean", "matched", "failure")
        full_r_diff  = abs(succ_full_r - fail_full_r) if (succ_full_r is not None and fail_full_r is not None) else 0.0
        # Thresholds: full_r difference > 0.05 = meaningful, > 0.1 = strong
        # H3 cos match difference > 0.05 = meaningful, > 0.1 = strong
        if cos_match_diff > 0.1 and full_r_diff > 0.05:
            verdict = "STRONG_SUPPORT"
            reason  = (f"H3 cosine differs by {cos_match_diff:.4f} between matched/mismatched; "
                       f"full radius differs by {full_r_diff:.4f} between success/failure "
                       f"(non-tautological radial diagnostic).")
        elif cos_match_diff > 0.05 or full_r_diff > 0.05:
            verdict = "WEAK_SUPPORT"
            reason  = (f"H3 cosine diff matched/mismatch={cos_match_diff:.4f}, "
                       f"full_radius diff success/failure={full_r_diff:.4f} — "
                       f"some variation but below strong threshold.")
        else:
            verdict = "NO_SUPPORT"
            reason  = (f"H3 cosine diff matched/mismatch={cos_match_diff:.6f} (threshold 0.05), "
                       f"full_radius diff success/failure={full_r_diff:.6f} (threshold 0.05). "
                       f"Radial variation exists (vs stripped std=0) but is purely architectural "
                       f"— does not discriminate matched vs mismatched or predict class recovery.")
    else:
        verdict = "NO_SUPPORT"
        reason  = ("Insufficient data for strong conclusion — one or more CASE D groups "
                   "had < 10 samples.")

    final_line = f"\nDYNAMIC DECODING LANGUAGE STATUS: {verdict}\n\n*Reason: {reason}*\n"
    md = md.replace("DYNAMIC DECODING LANGUAGE STATUS: **SEE_BELOW**\n\n(Replace with one of: NO_SUPPORT / WEAK_SUPPORT / STRONG_SUPPORT after reading\nTables 3 and 4.)", final_line)

    with open(MD_OUT, "w") as f:
        f.write(md)


if __name__ == "__main__":
    main()

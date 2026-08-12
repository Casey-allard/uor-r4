"""run_minimal_trained_comparison_basis_probe_v1.py

MINIMAL TRAINED COMPARISON BASIS PROBE v1
==========================================

BRANCH: minimal_trained_comparison_basis_probe_v1

CANONICAL FILES:
  - results/prime_transport_recursive_system/state_cache/state_tables_full.pt
  - results/prime_transport_recursive_system/dynamic_decoding_language_probe_v1.csv
    (CONTRAST only — prior branch used untrained weights, not a valid test)

CONTRAST FILES (for comparison only):
  - results/prime_transport_recursive_system/dynamic_decoding_language_probe_v1.csv

═══════════════════════════════════════════════════════════════════════════════
MECHANISM LOCK (completed before coding)
═══════════════════════════════════════════════════════════════════════════════

1. MINIMAL TRAINABLE COMPONENT:
   A 2-layer MLP with token embedding:
     W_emb: (VOCAB=4, D_EMB=4)              — 16 params
     W1:    (D_IN=16, D_HIDDEN=32)           — 512 params
     b1:    (D_HIDDEN=32,)                   — 32 params
     W2:    (D_HIDDEN=32, N_OPS=6)           — 192 params
     b2:    (N_OPS=6,)                       — 6 params
     TOTAL: 758 parameters

   At each phase step t:
     emb    = W_emb[tok_t]                      # (B, D_EMB)
     h_in   = cat([emb, tau_ang_prev], dim=1)   # (B, D_IN)
     h      = tanh(h_in @ W1 + b1)             # (B, D_HIDDEN)
     logits = h @ W2 + b2                       # (B, N_OPS)
     w      = softmax(logits / temp)            # (B, N_OPS)
     base   = einsum("bi,bij->bj", w, TN_ang)  # (B, d_ang)

   This is the narrowest module that is BOTH input-conditioned AND trainable.

2. WHAT IS TRAINED:
   Loss = geometric cross-entropy:
     - After D steps, take tau_final[:, H3_IDX0:H3_IDX1+1]
     - Compute dot product with 4 fixed class anchor vectors
       anchor_k = (-sin(pi*k/2), cos(pi*k/2)) for k in {0,1,2,3}
     - log_softmax over these 4 logits → CE with GT class label
   No additional learnable head. Readout is fixed geometric projection.

3. TRAINING PROTOCOL:
   - N_TRAIN_STEPS = 500
   - BATCH_SIZE_TRAIN = 128
   - LR = 0.01, Adam
   - Shared model (same weights for all inputs; input-conditioned via token)
   - Temperature annealed from 2.0 → 0.1 over training
   - Checkpoints at step 0, 100, 250, 500 (for CASE D tracking)

4. ALIGNMENT SUCCESS:
   - h3_class(tau_final) == h3_gt
   - Where h3_class = argmax over {cosine_sim(tau[:,10:12], anchor_k) for k=0..3}

5. RADIAL QUANTITIES:
   - full_radius    = ||tau_ang||_2          (12-dim vector norm)
   - subspace_radius= ||tau_ang[:, 10:12]||  (H3 pair norm)
   - ratio          = subspace_radius / full_radius

6. DIRECTIONAL QUANTITIES:
   - direction_metric = cosine_sim(tau_final[:, 10:12], tau_init[:, 10:12])

7. SUPPORT CRITERIA:
   STRONG_SUPPORT:  radial/directional signals are non-constant, differ across
                    inputs, correlate with success, AND only appear after training
   WEAK_SUPPORT:    some differentiation but inconsistent
   NO_SUPPORT:      constant, noisy, or non-predictive

═══════════════════════════════════════════════════════════════════════════════
PRE-CODE GEOMETRY LOCK (prompt_contract_v3.md requirement)
═══════════════════════════════════════════════════════════════════════════════

State representation:
  tau_ang: 12-dimensional angular vector
    Block 0 (s=0,e=2,m=2,n_h=1):    h1 at [0,1]
    Block 1 (s=2,e=7,m=5,n_h=1):    h1 at [2,3]
    Block 2 (s=7,e=9,m=2,n_h=1):    h1 at [4,5]
    Block 3 (s=9,e=21,m=12,n_h=3):  h1 at [6,7], h2 at [8,9], h3 at [10,11]

Task geometry:
  VOCAB=4 classes, dominant block period=12, n_h=3 harmonics.
  H3 (indices 10,11) encodes class identity (period 4 = VOCAB).
  Class anchors: anchor_k = (-sin(pi*k/2), cos(pi*k/2)) for k in {0,1,2,3}

CASES:
  A           — stripped baseline (hard deterministic routing, no MLP at all)
  B_untrained — random-init MLP, no training (control: comparison without learning)
  C_trained_matched    — trained MLP, token = GT class
  C_trained_mismatched — trained MLP, token = wrong class (+1 mod 4)
  D_traj_step0         — trained MLP at checkpoint step=0
  D_traj_step100       — trained MLP at checkpoint step=100
  D_traj_step250       — trained MLP at checkpoint step=250
  D_traj_step500       — trained MLP at checkpoint step=500 (= final)
"""

from __future__ import annotations

import csv
import math
import time
from pathlib import Path
from typing import Dict, List, Optional, Tuple

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F

# ═══════════════════════════════════════════════════════════════════════
# Paths
# ═══════════════════════════════════════════════════════════════════════
REPO_ROOT   = Path(__file__).resolve().parents[2]
RESULTS_DIR = REPO_ROOT / "results" / "prime_transport_recursive_system"
DOCS_DIR    = REPO_ROOT / "docs" / "research"
CACHE_PATH  = RESULTS_DIR / "state_cache" / "state_tables_full.pt"
CSV_OUT     = RESULTS_DIR / "minimal_trained_comparison_basis_probe_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_minimal_trained_comparison_basis_probe_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Constants — identical to canonical probes
# ═══════════════════════════════════════════════════════════════════════
GLOBAL_SEED      = 42
VOCAB            = 4
BLOCKS_A         = [(0, 2, 2, 1), (2, 7, 5, 1), (7, 9, 2, 1), (9, 21, 12, 3)]
TASK_A_CYC_S     = 9
TASK_A_CYC_E     = 21
N_EVAL           = 512       # evaluation samples (kept small for speed)
N_POOL           = 4000
D                = 20        # number of comparison/phase steps
N_OPS            = 6         # number of operators

# MLP dimensions
D_EMB            = 4
D_HIDDEN         = 32
D_ANG            = 12        # d_ang for BLOCKS_A

# Harmonic indices for block-3 (ai starts at 6)
H1_IDX0, H1_IDX1 = 6,  7
H2_IDX0, H2_IDX1 = 8,  9
H3_IDX0, H3_IDX1 = 10, 11

# Training protocol
N_TRAIN_STEPS    = 500
BATCH_TRAIN      = 128
LR               = 0.01
TEMP_START       = 2.0
TEMP_END         = 0.1
TRAIN_CHECKPOINTS = [0, 100, 250, 500]
INIT_SCALE       = 0.05

# CSV output fields — exactly as specified
CSV_FIELDS = [
    "case", "timestep", "training_step",
    "full_radius", "subspace_radius", "ratio",
    "direction_metric", "success_flag", "match_flag",
]

# Class anchor vectors: anchor_k = (-sin(pi*k/2), cos(pi*k/2))
def _class_anchors() -> torch.Tensor:
    """Returns (4, 2) tensor of H3 anchor directions for classes 0-3."""
    anchors = []
    for k in range(VOCAB):
        angle = math.pi * k / 2.0
        anchors.append([-math.sin(angle), math.cos(angle)])
    return torch.tensor(anchors, dtype=torch.float32)

CLASS_ANCHORS = _class_anchors()   # (4, 2)


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


def prepare_tables(TN_oh, tau0_oh, TR, pool_ids, blocks):
    d_ang, n_pairs, _ = geom_dims(blocks)
    TN_ang   = convert_onehot_to_angular_multi(TN_oh, blocks)
    tau0_ang = convert_onehot_to_angular_multi(tau0_oh, blocks)
    tau0_ang = apply_anchor_two_i(tau0_ang, n_pairs)
    return TN_ang, TR, tau0_ang, pool_ids


def build_class_table(tau0_oh, cyc_start, cyc_end, partition):
    idx = tau0_oh[:, cyc_start:cyc_end].argmax(dim=-1)
    lut = torch.tensor(partition, dtype=torch.long)
    return lut[idx]


def h3_class_from_ang(tau: torch.Tensor) -> torch.Tensor:
    """
    Read H3 class from angular state by matching to class anchor vectors.
    Returns (B,) long tensor with class in {0,1,2,3}.
    Geometry-native: no modular arithmetic used as control primitive.
    """
    h3 = tau[:, H3_IDX0:H3_IDX1 + 1]          # (B, 2)
    # Cosine similarities to all 4 class anchors
    anchors = CLASS_ANCHORS.to(tau.device)     # (4, 2)
    # Dot product: (B, 2) x (2, 4) → (B, 4)
    dots = h3 @ anchors.T
    return dots.argmax(dim=1)


def _safe_cos_sim(va: torch.Tensor, vb: torch.Tensor) -> torch.Tensor:
    na = va.norm(dim=1, keepdim=True).clamp(1e-12)
    nb = vb.norm(dim=1, keepdim=True).clamp(1e-12)
    return (va / na * vb / nb).sum(dim=1)


# ═══════════════════════════════════════════════════════════════════════
# Trainable minimal comparison module
# ═══════════════════════════════════════════════════════════════════════

class MinimalTrainedComparisonModule(nn.Module):
    """
    Minimal 758-parameter trainable comparison module.

    Forward: given token and current angular state, produce routing
    weights over N_OPS operators.

    emb    = W_emb[tok]                      # (B, D_EMB)
    h_in   = cat([emb, tau_ang], dim=1)      # (B, D_IN)
    h      = tanh(h_in @ W1 + b1)           # (B, D_HIDDEN)
    logits = h @ W2 + b2                     # (B, N_OPS)
    w      = softmax(logits / temp)          # (B, N_OPS)
    """
    def __init__(self, d_ang: int = D_ANG, seed: int = GLOBAL_SEED):
        super().__init__()
        gen = torch.Generator().manual_seed(seed)
        D_IN = D_EMB + d_ang

        self.W_emb = nn.Parameter(
            torch.empty(VOCAB, D_EMB).normal_(0.0, INIT_SCALE, generator=gen))
        self.W1    = nn.Parameter(
            torch.empty(D_IN, D_HIDDEN).normal_(0.0, INIT_SCALE, generator=gen))
        self.b1    = nn.Parameter(torch.zeros(D_HIDDEN))
        self.W2    = nn.Parameter(
            torch.empty(D_HIDDEN, N_OPS).normal_(0.0, INIT_SCALE, generator=gen))
        self.b2    = nn.Parameter(torch.zeros(N_OPS))

    def forward(
        self,
        tok: torch.Tensor,         # (B,) int64
        tau_ang: torch.Tensor,     # (B, d_ang)
        temp: float = 1.0,
    ) -> torch.Tensor:             # (B, N_OPS) routing weights
        emb    = self.W_emb[tok]                          # (B, D_EMB)
        h_in   = torch.cat([emb, tau_ang], dim=1)         # (B, D_IN)
        h      = torch.tanh(h_in @ self.W1 + self.b1)    # (B, D_HIDDEN)
        logits = h @ self.W2 + self.b2                    # (B, N_OPS)
        return torch.softmax(logits / temp, dim=1)        # (B, N_OPS)

    def n_params(self) -> int:
        return sum(p.numel() for p in self.parameters())


# ═══════════════════════════════════════════════════════════════════════
# Geometric cross-entropy loss (no additional learnable head)
# ═══════════════════════════════════════════════════════════════════════

def geometric_h3_loss(tau_final: torch.Tensor, h3_gt: torch.Tensor) -> torch.Tensor:
    """
    Loss based purely on H3 phase alignment to class anchors.

    tau_final: (B, d_ang)  — angular state at final step
    h3_gt:     (B,)        — ground truth class in {0,1,2,3}

    Returns scalar cross-entropy loss.
    No learnable parameters in readout — anchors are fixed geometry.
    """
    h3  = tau_final[:, H3_IDX0:H3_IDX1 + 1]        # (B, 2)
    anchors = CLASS_ANCHORS.to(tau_final.device)    # (4, 2)
    logits  = h3 @ anchors.T                        # (B, 4) — dot products
    return F.cross_entropy(logits, h3_gt)


# ═══════════════════════════════════════════════════════════════════════
# Forward pass: comparison loop
# ═══════════════════════════════════════════════════════════════════════

def comparison_forward(
    sids: torch.Tensor,         # (B,) state IDs
    tau0_ang: torch.Tensor,     # (N_states, d_ang) — all angular states
    TN_ang: torch.Tensor,       # (N_states, N_OPS, d_ang)
    TR: torch.Tensor,           # (N_states, N_OPS) int64
    tokens: torch.Tensor,       # (B, D) int64 — one token per phase step
    module: Optional[MinimalTrainedComparisonModule],
    D: int,
    temp: float = 1.0,
) -> List[torch.Tensor]:
    """
    Run D-step comparison loop.

    If module is None (CASE A): hard angular-similarity routing, no blend.
    If module is provided: soft blend with input-conditioned weights.

    Returns list of tau_ang snapshots at t=0,1,...,D.
    """
    B         = sids.shape[0]
    d_ang     = tau0_ang.shape[1]
    tau_prev  = tau0_ang[sids].clone()
    sids_loop = sids.clone()
    steps     = [tau_prev.detach().clone()]

    for t in range(D):
        tn_batch = TN_ang[sids_loop]                       # (B, N_OPS, d_ang)

        if module is None:
            # CASE A: hard angular-similarity, deterministic
            ang_sim  = torch.einsum("bi,bji->bj", tau_prev, tn_batch)
            best_op  = ang_sim.argmax(dim=1)
            best_ang = tn_batch.gather(
                1, best_op.view(B, 1, 1).expand(B, 1, d_ang)).squeeze(1)
            tau_next = best_ang
        else:
            # CASE B/C: soft blend with routing weights
            tok_t   = tokens[:, t]                         # (B,)
            w       = module(tok_t, tau_prev, temp)        # (B, N_OPS)
            tau_next = torch.einsum("bi,bij->bj", w, tn_batch)  # (B, d_ang)
            best_op  = w.argmax(dim=1)

        sids_loop = TR[sids_loop].gather(
            1, best_op.unsqueeze(1)).squeeze(1)
        tau_prev  = tau_next
        steps.append(tau_prev.detach().clone())

    return steps


def comparison_forward_with_grad(
    sids: torch.Tensor,
    tau0_ang: torch.Tensor,
    TN_ang: torch.Tensor,
    TR: torch.Tensor,
    tokens: torch.Tensor,
    module: MinimalTrainedComparisonModule,
    D: int,
    temp: float,
) -> torch.Tensor:
    """
    Forward pass that retains gradients through the MLP weights.
    Returns tau_final (B, d_ang) with gradient.
    """
    B         = sids.shape[0]
    d_ang     = tau0_ang.shape[1]
    tau_prev  = tau0_ang[sids].clone()
    sids_loop = sids.clone()

    for t in range(D):
        tn_batch = TN_ang[sids_loop]
        tok_t    = tokens[:, t]
        w        = module(tok_t, tau_prev, temp)
        tau_next = torch.einsum("bi,bij->bj", w, tn_batch)
        best_op  = w.detach().argmax(dim=1)
        sids_loop = TR[sids_loop.detach()].gather(
            1, best_op.unsqueeze(1)).squeeze(1)
        tau_prev = tau_next

    return tau_prev


# ═══════════════════════════════════════════════════════════════════════
# Measurement: collect per-sample rows
# ═══════════════════════════════════════════════════════════════════════

def collect_rows(
    case: str,
    training_step: int,
    steps: List[torch.Tensor],      # tau_ang snapshots at t=0..D
    tau_init: torch.Tensor,         # (B, d_ang) reference
    h3_gt: torch.Tensor,            # (B,) ground truth class
    match_flag: int,                # 1=matched, 0=mismatched, -1=n/a
) -> List[Dict]:
    """
    Produce one CSV row per (timestep) with aggregate stats over the batch.
    Per-row fields: case, timestep, training_step, full_radius (mean),
    subspace_radius (mean), ratio (mean), direction_metric (mean),
    success_flag (fraction), match_flag.
    """
    rows = []
    for t, tau_t in enumerate(steps):
        full_r  = tau_t.norm(dim=1)                        # (B,)
        h3_r    = tau_t[:, H3_IDX0:H3_IDX1 + 1].norm(dim=1)  # (B,)
        ratio   = h3_r / full_r.clamp(1e-12)              # (B,)
        dir_m   = _safe_cos_sim(
            tau_t[:, H3_IDX0:H3_IDX1 + 1],
            tau_init[:, H3_IDX0:H3_IDX1 + 1])             # (B,)
        pred_cl = h3_class_from_ang(tau_t)                 # (B,)
        succ    = (pred_cl == h3_gt).float()               # (B,)

        rows.append({
            "case":             case,
            "timestep":         t,
            "training_step":    training_step,
            "full_radius":      round(float(full_r.mean().item()), 8),
            "subspace_radius":  round(float(h3_r.mean().item()), 8),
            "ratio":            round(float(ratio.mean().item()), 8),
            "direction_metric": round(float(dir_m.mean().item()), 8),
            "success_flag":     round(float(succ.mean().item()), 6),
            "match_flag":       match_flag,
        })
    return rows


def collect_rows_std(
    case: str,
    training_step: int,
    steps: List[torch.Tensor],
    tau_init: torch.Tensor,
    h3_gt: torch.Tensor,
    match_flag: int,
) -> List[Dict]:
    """
    Extended rows with std statistics (not written to CSV — used for MD only).
    """
    rows = []
    for t, tau_t in enumerate(steps):
        full_r  = tau_t.norm(dim=1)
        h3_r    = tau_t[:, H3_IDX0:H3_IDX1 + 1].norm(dim=1)
        ratio   = h3_r / full_r.clamp(1e-12)
        dir_m   = _safe_cos_sim(
            tau_t[:, H3_IDX0:H3_IDX1 + 1],
            tau_init[:, H3_IDX0:H3_IDX1 + 1])

        rows.append({
            "case":                 case,
            "timestep":             t,
            "training_step":        training_step,
            "full_radius_mean":     round(float(full_r.mean().item()), 6),
            "full_radius_std":      round(float(full_r.std().item()), 6),
            "subspace_radius_mean": round(float(h3_r.mean().item()), 6),
            "subspace_radius_std":  round(float(h3_r.std().item()), 6),
            "ratio_mean":           round(float(ratio.mean().item()), 6),
            "ratio_std":            round(float(ratio.std().item()), 6),
            "direction_metric_mean": round(float(dir_m.mean().item()), 6),
            "direction_metric_std":  round(float(dir_m.std().item()), 6),
            "match_flag":           match_flag,
        })
    return rows


# ═══════════════════════════════════════════════════════════════════════
# Training loop
# ═══════════════════════════════════════════════════════════════════════

def train_module(
    module: MinimalTrainedComparisonModule,
    tau0_ang: torch.Tensor,    # (N_states, d_ang)
    TN_ang: torch.Tensor,      # (N_states, N_OPS, d_ang)
    TR: torch.Tensor,
    h3_gt_all: torch.Tensor,   # (N_states,) class labels
    pool_ids: torch.Tensor,    # (N_pool,)
    n_steps: int,
    batch_size: int,
    lr: float,
    temp_start: float,
    temp_end: float,
    checkpoints: List[int],
    D: int,
    seed: int = GLOBAL_SEED,
) -> Dict[int, MinimalTrainedComparisonModule]:
    """
    Train the minimal comparison module using geometric H3 loss.

    Returns dict mapping checkpoint_step → snapshot of module weights.
    """
    optimizer = torch.optim.Adam(module.parameters(), lr=lr)
    rng       = torch.Generator().manual_seed(seed)
    snapshots: Dict[int, MinimalTrainedComparisonModule] = {}

    if 0 in checkpoints:
        snap = MinimalTrainedComparisonModule(d_ang=tau0_ang.shape[1])
        snap.load_state_dict(
            {k: v.clone() for k, v in module.state_dict().items()})
        snapshots[0] = snap

    print(f"  Training {n_steps} steps, batch={batch_size}, D={D}, lr={lr}")
    print(f"  Module params: {module.n_params()}")

    losses = []

    for step in range(1, n_steps + 1):
        temp = temp_start + (temp_end - temp_start) * (step / n_steps)

        # Sample batch from pool
        idx   = torch.randint(pool_ids.shape[0], (batch_size,), generator=rng)
        sids  = pool_ids[idx]

        # Token = GT class (matched training)
        gt    = h3_gt_all[sids]           # (B,)
        tokens = gt.unsqueeze(1).expand(-1, D).clone()  # (B, D)

        optimizer.zero_grad()
        tau_final = comparison_forward_with_grad(
            sids, tau0_ang, TN_ang, TR, tokens, module, D, temp)
        loss = geometric_h3_loss(tau_final, gt)
        loss.backward()
        optimizer.step()

        losses.append(float(loss.item()))

        if step in checkpoints:
            snap = MinimalTrainedComparisonModule(d_ang=tau0_ang.shape[1])
            snap.load_state_dict(
                {k: v.clone() for k, v in module.state_dict().items()})
            snapshots[step] = snap
            print(f"    step={step:4d}  loss={loss.item():.6f}  "
                  f"temp={temp:.4f}  (checkpoint saved)")
        elif step % 50 == 0:
            print(f"    step={step:4d}  loss={loss.item():.6f}  temp={temp:.4f}")

    print(f"  Training complete. Final loss={losses[-1]:.6f}")
    return snapshots


# ═══════════════════════════════════════════════════════════════════════
# MAIN
# ═══════════════════════════════════════════════════════════════════════

def main():
    print("=" * 72)
    print("MINIMAL TRAINED COMPARISON BASIS PROBE v1")
    print(f"  D={D} steps, N_EVAL={N_EVAL}, VOCAB={VOCAB}")
    print(f"  N_TRAIN_STEPS={N_TRAIN_STEPS}, BATCH_TRAIN={BATCH_TRAIN}, LR={LR}")
    print("=" * 72)

    t_start = time.perf_counter()

    # ── Load state cache ──────────────────────────────────────────────
    print("\nLoading state cache ...", end=" ", flush=True)
    cache        = torch.load(str(CACHE_PATH), map_location="cpu", weights_only=True)
    TN_oh        = cache["TN_oh"]       # (N_states, N_OPS, 21)
    tau0_oh      = cache["tau0_oh"]     # (N_states, 21)
    TR           = cache["TR"]          # (N_states, N_OPS) int64
    pool_ids_raw = cache["pool_ids"]    # (4000,)
    print(f"done. N_states={TN_oh.shape[0]:,}", flush=True)

    TN_ang, TR, tau0_ang_all, pool_ids = prepare_tables(
        TN_oh, tau0_oh, TR, pool_ids_raw, BLOCKS_A)
    d_ang, n_pairs, n_blocks = geom_dims(BLOCKS_A)
    print(f"  d_ang={d_ang}, n_pairs={n_pairs}, n_blocks={n_blocks}")

    # Build H3 class table for all states
    partition_4   = [0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3]
    h3_gt_all     = build_class_table(tau0_oh, TASK_A_CYC_S, TASK_A_CYC_E, partition_4)
    # (N_states,) long

    # ── Sample evaluation set ─────────────────────────────────────────
    rng_eval  = torch.Generator().manual_seed(GLOBAL_SEED)
    idx_eval  = torch.randint(pool_ids.shape[0], (N_EVAL,), generator=rng_eval)
    sids_eval = pool_ids[idx_eval]

    tau_init  = tau0_ang_all[sids_eval].clone()    # (N_EVAL, 12) fixed reference
    h3_gt     = h3_gt_all[sids_eval].clone()       # (N_EVAL,)
    # Matched token: correct class; mismatched: +1 mod 4
    tok_matched    = h3_gt.unsqueeze(1).expand(-1, D).clone()
    tok_mismatched = ((h3_gt + 1) % VOCAB).unsqueeze(1).expand(-1, D).clone()

    class_dist = {int(c): int((h3_gt == c).sum()) for c in range(VOCAB)}
    print(f"  Eval H3 class dist: {class_dist}")

    all_rows: List[Dict]     = []
    all_stats: List[Dict]    = []   # richer stats for MD tables

    # ── CASE A: stripped baseline (no MLP at all) ─────────────────────
    print("\n─── CASE A: Stripped Baseline (no MLP) ───")
    steps_a = comparison_forward(
        sids_eval, tau0_ang_all, TN_ang, TR,
        tok_matched, module=None, D=D, temp=1.0)
    rows_a   = collect_rows("A_stripped", -1, steps_a, tau_init, h3_gt, -1)
    stats_a  = collect_rows_std("A_stripped", -1, steps_a, tau_init, h3_gt, -1)
    all_rows.extend(rows_a)
    all_stats.extend(stats_a)

    # Sanity check: CASE A should be constant norm
    fr0_a = float(steps_a[0].norm(dim=1).mean().item())
    frD_a = float(steps_a[-1].norm(dim=1).mean().item())
    acc_a = float((h3_class_from_ang(steps_a[-1]) == h3_gt).float().mean().item())
    print(f"  full_radius: t=0={fr0_a:.6f}  t={D}={frD_a:.6f}  "
          f"(expected constant ~{math.sqrt(n_pairs):.6f})")
    print(f"  H3 recovery rate at t={D}: {acc_a:.4f}")
    fr_std_a = float(torch.stack([s.norm(dim=1) for s in steps_a]).std(dim=0).mean().item())
    print(f"  full_radius std across timesteps (mean over samples): {fr_std_a:.6f}")

    # ── CASE B: untrained comparison (random MLP, no backprop) ───────
    print("\n─── CASE B: Untrained Comparison (random MLP, no training) ───")
    module_untrained = MinimalTrainedComparisonModule(d_ang=d_ang, seed=GLOBAL_SEED + 1)
    module_untrained.eval()
    with torch.no_grad():
        steps_b_m = comparison_forward(
            sids_eval, tau0_ang_all, TN_ang, TR,
            tok_matched, module=module_untrained, D=D, temp=0.05)
        steps_b_mm = comparison_forward(
            sids_eval, tau0_ang_all, TN_ang, TR,
            tok_mismatched, module=module_untrained, D=D, temp=0.05)

    rows_b_m   = collect_rows("B_untrained_matched",    -1, steps_b_m,  tau_init, h3_gt, 1)
    rows_b_mm  = collect_rows("B_untrained_mismatched", -1, steps_b_mm, tau_init, h3_gt, 0)
    stats_b_m  = collect_rows_std("B_untrained_matched",  -1, steps_b_m,  tau_init, h3_gt, 1)
    stats_b_mm = collect_rows_std("B_untrained_mismatched",-1, steps_b_mm, tau_init, h3_gt, 0)
    all_rows.extend(rows_b_m + rows_b_mm)
    all_stats.extend(stats_b_m + stats_b_mm)

    frD_b_m   = float(steps_b_m[-1].norm(dim=1).mean().item())
    frD_b_mm  = float(steps_b_mm[-1].norm(dim=1).mean().item())
    acc_b_m   = float((h3_class_from_ang(steps_b_m[-1])  == h3_gt).float().mean().item())
    acc_b_mm  = float((h3_class_from_ang(steps_b_mm[-1]) == h3_gt).float().mean().item())
    print(f"  matched:    full_r={frD_b_m:.6f}  acc={acc_b_m:.4f}")
    print(f"  mismatched: full_r={frD_b_mm:.6f}  acc={acc_b_mm:.4f}")

    # ── Training ─────────────────────────────────────────────────────
    print("\n─── Training Minimal Comparison Module ───")
    module_trained = MinimalTrainedComparisonModule(d_ang=d_ang, seed=GLOBAL_SEED + 1)

    snapshots = train_module(
        module_trained,
        tau0_ang_all, TN_ang, TR, h3_gt_all, pool_ids,
        n_steps=N_TRAIN_STEPS,
        batch_size=BATCH_TRAIN,
        lr=LR,
        temp_start=TEMP_START,
        temp_end=TEMP_END,
        checkpoints=TRAIN_CHECKPOINTS,
        D=D,
        seed=GLOBAL_SEED + 10,
    )

    # ── CASE D: training trajectory at each checkpoint ────────────────
    print("\n─── CASE D: Training Trajectory ───")
    for ckpt_step, snap in sorted(snapshots.items()):
        snap.eval()
        temp_at_ckpt = (TEMP_START + (TEMP_END - TEMP_START) * (ckpt_step / N_TRAIN_STEPS)
                        if ckpt_step > 0 else TEMP_START)
        with torch.no_grad():
            steps_ckpt_m = comparison_forward(
                sids_eval, tau0_ang_all, TN_ang, TR,
                tok_matched, module=snap, D=D, temp=temp_at_ckpt)
        case_label = f"D_traj_step{ckpt_step}"
        rows_ckpt  = collect_rows(case_label, ckpt_step, steps_ckpt_m,
                                   tau_init, h3_gt, 1)
        stats_ckpt = collect_rows_std(case_label, ckpt_step, steps_ckpt_m,
                                       tau_init, h3_gt, 1)
        all_rows.extend(rows_ckpt)
        all_stats.extend(stats_ckpt)

        acc_ckpt = float((h3_class_from_ang(steps_ckpt_m[-1]) == h3_gt).float().mean())
        frD_ckpt = float(steps_ckpt_m[-1].norm(dim=1).mean())
        fr_std_ckpt = float(steps_ckpt_m[-1].norm(dim=1).std())
        dir_ckpt = float(_safe_cos_sim(
            steps_ckpt_m[-1][:, H3_IDX0:H3_IDX1 + 1],
            tau_init[:, H3_IDX0:H3_IDX1 + 1]).mean())
        print(f"  step={ckpt_step:4d}  acc={acc_ckpt:.4f}  "
              f"full_r_mean={frD_ckpt:.6f}  full_r_std={fr_std_ckpt:.6f}  "
              f"h3_dir={dir_ckpt:.6f}")

    # ── CASE C: trained comparison, matched vs mismatched ─────────────
    print("\n─── CASE C: Trained Comparison (matched vs mismatched) ───")
    final_snap = snapshots[N_TRAIN_STEPS]
    final_snap.eval()
    temp_final = TEMP_END

    with torch.no_grad():
        steps_c_m  = comparison_forward(
            sids_eval, tau0_ang_all, TN_ang, TR,
            tok_matched, module=final_snap, D=D, temp=temp_final)
        steps_c_mm = comparison_forward(
            sids_eval, tau0_ang_all, TN_ang, TR,
            tok_mismatched, module=final_snap, D=D, temp=temp_final)

    rows_c_m   = collect_rows("C_trained_matched",    N_TRAIN_STEPS, steps_c_m,  tau_init, h3_gt, 1)
    rows_c_mm  = collect_rows("C_trained_mismatched", N_TRAIN_STEPS, steps_c_mm, tau_init, h3_gt, 0)
    stats_c_m  = collect_rows_std("C_trained_matched",    N_TRAIN_STEPS, steps_c_m,  tau_init, h3_gt, 1)
    stats_c_mm = collect_rows_std("C_trained_mismatched", N_TRAIN_STEPS, steps_c_mm, tau_init, h3_gt, 0)
    all_rows.extend(rows_c_m + rows_c_mm)
    all_stats.extend(stats_c_m + stats_c_mm)

    acc_c_m   = float((h3_class_from_ang(steps_c_m[-1])  == h3_gt).float().mean())
    acc_c_mm  = float((h3_class_from_ang(steps_c_mm[-1]) == h3_gt).float().mean())
    frD_c_m   = float(steps_c_m[-1].norm(dim=1).mean())
    frD_c_mm  = float(steps_c_mm[-1].norm(dim=1).mean())
    std_c_m   = float(steps_c_m[-1].norm(dim=1).std())
    std_c_mm  = float(steps_c_mm[-1].norm(dim=1).std())
    dir_c_m   = float(_safe_cos_sim(steps_c_m[-1][:, H3_IDX0:H3_IDX1+1],
                                     tau_init[:, H3_IDX0:H3_IDX1+1]).mean())
    dir_c_mm  = float(_safe_cos_sim(steps_c_mm[-1][:, H3_IDX0:H3_IDX1+1],
                                     tau_init[:, H3_IDX0:H3_IDX1+1]).mean())

    print(f"  matched:    full_r={frD_c_m:.6f}  std={std_c_m:.6f}  "
          f"acc={acc_c_m:.4f}  h3_dir={dir_c_m:.6f}")
    print(f"  mismatched: full_r={frD_c_mm:.6f}  std={std_c_mm:.6f}  "
          f"acc={acc_c_mm:.4f}  h3_dir={dir_c_mm:.6f}")

    # ── Write CSV ──────────────────────────────────────────────────────
    with open(CSV_OUT, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=CSV_FIELDS)
        writer.writeheader()
        writer.writerows(all_rows)
    print(f"\nCSV written: {CSV_OUT.name}  ({len(all_rows)} rows)")

    # ── Write MD ──────────────────────────────────────────────────────
    elapsed = time.perf_counter() - t_start
    _write_md(all_rows, all_stats, elapsed,
              d_ang, n_pairs,
              acc_a, acc_b_m, acc_b_mm, acc_c_m, acc_c_mm,
              frD_a, frD_b_m, frD_b_mm, frD_c_m, frD_c_mm,
              std_c_m, std_c_mm, dir_c_m, dir_c_mm,
              snapshots)
    print(f"MD  written: {MD_OUT.name}")
    print(f"\nTotal elapsed: {elapsed:.1f}s")
    print("=" * 72)


# ═══════════════════════════════════════════════════════════════════════
# MD writer
# ═══════════════════════════════════════════════════════════════════════

def _get_stat(stats: List[Dict], case: str, t_val: int, field: str,
              training_step: Optional[int] = None) -> str:
    for r in stats:
        if r["case"] != case:
            continue
        if r["timestep"] != t_val:
            continue
        if training_step is not None and r["training_step"] != training_step:
            continue
        val = r.get(field, "n/a")
        return str(val)
    return "n/a"


def _assessment_text(verdict: str, acc_gap_untrained: float, acc_gap_mismatch: float,
                     radial_std_ratio: float, n_strong: int) -> str:
    """
    Generate an honest assessment based on measured values.
    Does NOT overclaim. Maps exactly to what the numbers show.
    """
    if verdict == "NO_SUPPORT":
        return (
            "Training did not produce differentiation. Accuracy, radial, and "
            "directional signals remain approximately the same as the untrained "
            "baseline. The dynamic comparison-language hypothesis is not supported "
            "by this probe."
        )
    elif verdict == "STRONG_SUPPORT":
        return (
            f"Training produced clear improvements: accuracy gap vs untrained "
            f"{acc_gap_untrained:+.4f}, vs mismatched {acc_gap_mismatch:+.4f}. "
            f"Radial variation increased by {radial_std_ratio:.2f}x vs untrained. "
            "Radial and directional signals are input-specific, phase-evolving, "
            "and predictive of alignment success. Hypothesis is supported."
        )
    else:
        # WEAK_SUPPORT — must be precise about what is and is not shown
        lines = []
        if radial_std_ratio > 1.5:
            lines.append(
                f"Radial variation (full_r_std) increased by {radial_std_ratio:.2f}x "
                "vs untrained — training does affect the radial structure of the "
                "comparison basis."
            )
        else:
            lines.append("Radial variation did not meaningfully increase with training.")

        if abs(acc_gap_untrained) < 0.05:
            lines.append(
                f"However, accuracy improvement over untrained is marginal "
                f"({acc_gap_untrained:+.4f}), not exceeding the 0.05 threshold. "
                "This weakens any claim that trained signals are predictive."
            )
        if abs(acc_gap_mismatch) < 0.05:
            lines.append(
                f"Matched vs mismatched discrimination is also weak "
                f"({acc_gap_mismatch:+.4f}). The module does not reliably "
                "distinguish correct from incorrect input pairings."
            )
        lines.append(
            "Overall: training introduces some radial structure change but does "
            "not produce clearly predictive or input-discriminating signals at "
            "this training budget and configuration. Result is WEAK_SUPPORT — "
            "not sufficient to confirm the dynamic comparison-language hypothesis."
        )
        return " ".join(lines)


def _write_md(
    all_rows: List[Dict],
    all_stats: List[Dict],
    elapsed: float,
    d_ang: int,
    n_pairs: int,
    acc_a, acc_b_m, acc_b_mm, acc_c_m, acc_c_mm,
    frD_a, frD_b_m, frD_b_mm, frD_c_m, frD_c_mm,
    std_c_m, std_c_mm, dir_c_m, dir_c_mm,
    snapshots: Dict[int, MinimalTrainedComparisonModule],
):
    # Extract final-step stats for tables
    D_val = D

    def get_row_val(case, t_val, field, ts=None):
        return _get_stat(all_stats, case, t_val, field, ts)

    # Table 1: Baseline vs trained at final step
    def table1():
        hdr = ("| Case | full_r mean | full_r std | subspace_r mean | "
               "subspace_r std | ratio mean | ratio std | dir mean | dir std |\n"
               "|------|-----------|-----------|---------------|"
               "-------------|----------|---------|---------|--------|\n")
        body = ""
        cases_info = [
            ("A_stripped",             -1, "Stripped Baseline (no MLP)"),
            ("B_untrained_matched",    -1, "B: Untrained matched"),
            ("B_untrained_mismatched", -1, "B: Untrained mismatched"),
            ("C_trained_matched",      N_TRAIN_STEPS, "C: Trained matched"),
            ("C_trained_mismatched",   N_TRAIN_STEPS, "C: Trained mismatched"),
        ]
        for case_id, ts, label in cases_info:
            frm  = get_row_val(case_id, D_val, "full_radius_mean",    ts)
            frs  = get_row_val(case_id, D_val, "full_radius_std",     ts)
            srm  = get_row_val(case_id, D_val, "subspace_radius_mean",ts)
            srs  = get_row_val(case_id, D_val, "subspace_radius_std", ts)
            ram  = get_row_val(case_id, D_val, "ratio_mean",          ts)
            ras  = get_row_val(case_id, D_val, "ratio_std",           ts)
            dm   = get_row_val(case_id, D_val, "direction_metric_mean", ts)
            ds   = get_row_val(case_id, D_val, "direction_metric_std",  ts)
            body += f"| {label} | {frm} | {frs} | {srm} | {srs} | {ram} | {ras} | {dm} | {ds} |\n"
        return hdr + body

    # Table 2: Training trajectory (final step metrics per checkpoint)
    def table2():
        hdr = ("| Training Step | full_r mean | full_r std | ratio mean | "
               "dir mean | dir std | success_rate |\n"
               "|--------------|-----------|-----------|----------|"
               "---------|---------|-------------|\n")
        body = ""
        for ckpt in TRAIN_CHECKPOINTS:
            case_id = f"D_traj_step{ckpt}"
            frm  = get_row_val(case_id, D_val, "full_radius_mean",     ckpt)
            frs  = get_row_val(case_id, D_val, "full_radius_std",      ckpt)
            ram  = get_row_val(case_id, D_val, "ratio_mean",           ckpt)
            dm   = get_row_val(case_id, D_val, "direction_metric_mean",ckpt)
            ds   = get_row_val(case_id, D_val, "direction_metric_std", ckpt)
            # success rate from all_rows
            succ = "n/a"
            for r in all_rows:
                if r["case"] == case_id and r["timestep"] == D_val:
                    succ = str(r["success_flag"])
                    break
            body += f"| {ckpt} | {frm} | {frs} | {ram} | {dm} | {ds} | {succ} |\n"
        return hdr + body

    # Table 3: Matched vs mismatched at final step
    def table3():
        hdr = ("| Condition | full_r mean | full_r std | ratio mean | "
               "dir mean | acc |\n"
               "|-----------|-----------|-----------|----------|-"
               "--------|-----|\n")
        body = ""
        pairs = [
            ("B_untrained_matched",    -1,            1, "B Untrained matched",    f"{acc_b_m:.4f}"),
            ("B_untrained_mismatched", -1,            0, "B Untrained mismatched", f"{acc_b_mm:.4f}"),
            ("C_trained_matched",      N_TRAIN_STEPS, 1, "C Trained matched",      f"{acc_c_m:.4f}"),
            ("C_trained_mismatched",   N_TRAIN_STEPS, 0, "C Trained mismatched",   f"{acc_c_mm:.4f}"),
        ]
        for case_id, ts, mf, label, acc in pairs:
            frm = get_row_val(case_id, D_val, "full_radius_mean",    ts)
            frs = get_row_val(case_id, D_val, "full_radius_std",     ts)
            ram = get_row_val(case_id, D_val, "ratio_mean",          ts)
            dm  = get_row_val(case_id, D_val, "direction_metric_mean", ts)
            body += f"| {label} | {frm} | {frs} | {ram} | {dm} | {acc} |\n"
        return hdr + body

    # ── Analyse if signal emerges with training ────────────────────────
    # Collect full_r_std across timesteps for each case
    def _timestep_std_series(case: str, ts: int) -> List[float]:
        vals = []
        for t in range(D + 1):
            v = _get_stat(all_stats, case, t, "full_radius_std", ts)
            try:
                vals.append(float(v))
            except (ValueError, TypeError):
                vals.append(0.0)
        return vals

    std_series_a  = _timestep_std_series("A_stripped", -1)
    std_series_bm = _timestep_std_series("B_untrained_matched", -1)
    std_series_cm = _timestep_std_series("C_trained_matched", N_TRAIN_STEPS)

    std_avg_a   = sum(std_series_a)  / len(std_series_a)
    std_avg_bm  = sum(std_series_bm) / len(std_series_bm)
    std_avg_cm  = sum(std_series_cm) / len(std_series_cm)

    # ── Direction variation across timesteps ──────────────────────────
    def _dir_series(case: str, ts: int) -> List[float]:
        vals = []
        for t in range(D + 1):
            v = _get_stat(all_stats, case, t, "direction_metric_mean", ts)
            try:
                vals.append(float(v))
            except (ValueError, TypeError):
                vals.append(0.0)
        return vals

    dir_series_a  = _dir_series("A_stripped", -1)
    dir_series_bm = _dir_series("B_untrained_matched", -1)
    dir_series_cm = _dir_series("C_trained_matched", N_TRAIN_STEPS)

    # Variation (std) of direction metric across timesteps
    dir_var_a  = float(np.std(dir_series_a))
    dir_var_bm = float(np.std(dir_series_bm))
    dir_var_cm = float(np.std(dir_series_cm))

    # ── Determine support verdict ──────────────────────────────────────
    # STRONG_SUPPORT criteria:
    #   1. acc_c_m > acc_a + 0.05            (training improves over stripped)
    #   2. acc_c_m > acc_b_m + 0.05          (training surpasses untrained)
    #   3. acc_c_m > acc_c_mm + 0.05         (matched discriminates mismatched)
    #   4. std_avg_cm > std_avg_bm * 1.5     (training causes more radial variation
    #                                          than random weights alone)
    #
    # NOTE on radial_std_ratio denominator: we compare trained vs UNTRAINED (B),
    # NOT trained vs stripped (A). Stripped has std≈0 trivially because the hard
    # routing preserves the exact unit-norm angular structure. Comparing trained
    # to an exact-zero denominator is uninformative. The meaningful question is
    # whether TRAINING specifically causes MORE variation than random MLP weights.
    acc_gap_vs_stripped  = acc_c_m - acc_a
    acc_gap_vs_untrained = acc_c_m - acc_b_m
    acc_gap_vs_mismatch  = acc_c_m - acc_c_mm
    # Compare trained vs untrained radial/directional variation (honest baseline)
    radial_std_ratio     = std_avg_cm / max(std_avg_bm,  1e-9)
    dir_var_ratio        = dir_var_cm / max(dir_var_bm,  1e-9)
    # Also track stripped for reference
    radial_std_ratio_vs_stripped = std_avg_cm / max(std_avg_a, 1e-9)

    strong_criteria = [
        acc_gap_vs_stripped  > 0.05,
        acc_gap_vs_untrained > 0.05,
        acc_gap_vs_mismatch  > 0.05,
        radial_std_ratio     > 1.5,   # trained > 1.5x untrained radial variation
    ]
    weak_criteria = [
        acc_c_m > 0.30,
        acc_c_m > acc_c_mm,
        radial_std_ratio > 1.1,
    ]

    n_strong = sum(strong_criteria)
    n_weak   = sum(weak_criteria)

    if n_strong >= 3:
        verdict = "STRONG_SUPPORT"
    elif n_weak >= 2 or n_strong >= 1:
        verdict = "WEAK_SUPPORT"
    else:
        verdict = "NO_SUPPORT"

    # ── Phase evolution table ──────────────────────────────────────────
    def phase_evo_table():
        hdr = ("| Timestep | A full_r_std | B_untrain full_r_std | "
               "C_trained full_r_std | A dir_mean | B_untrain dir_mean | C_trained dir_mean |\n"
               "|---------|-------------|---------------------|"
               "--------------------|----------|------------------|------------------|\n")
        body = ""
        for t in range(0, D + 1, 4):
            a_s  = _get_stat(all_stats, "A_stripped",             t, "full_radius_std",      -1)
            b_s  = _get_stat(all_stats, "B_untrained_matched",    t, "full_radius_std",      -1)
            c_s  = _get_stat(all_stats, "C_trained_matched",      t, "full_radius_std",      N_TRAIN_STEPS)
            a_d  = _get_stat(all_stats, "A_stripped",             t, "direction_metric_mean",-1)
            b_d  = _get_stat(all_stats, "B_untrained_matched",    t, "direction_metric_mean",-1)
            c_d  = _get_stat(all_stats, "C_trained_matched",      t, "direction_metric_mean",N_TRAIN_STEPS)
            body += f"| {t:2d} | {a_s} | {b_s} | {c_s} | {a_d} | {b_d} | {c_d} |\n"
        return hdr + body

    md_content = f"""# Prime Transport — Minimal Trained Comparison Basis Probe v1

**Branch**: minimal_trained_comparison_basis_probe_v1
**Date**: 2026-04-09
**Elapsed**: {elapsed:.1f}s

---

## 1. Mechanism Lock

### Minimal Trainable Component Restored

A 2-layer MLP with token embedding. **Total: 758 parameters**.

| Parameter | Shape | Count |
|-----------|-------|-------|
| W_emb     | (4, 4)  | 16 |
| W1        | (16, 32)| 512 |
| b1        | (32,)   | 32 |
| W2        | (32, 6) | 192 |
| b2        | (6,)    | 6 |
| **Total** |         | **758** |

At each phase step t:
```
emb    = W_emb[tok_t]                        # (B, D_EMB=4)
h_in   = cat([emb, tau_ang_prev], dim=1)     # (B, 16)
h      = tanh(h_in @ W1 + b1)               # (B, 32)
logits = h @ W2 + b2                         # (B, 6)
w      = softmax(logits / temp)              # (B, 6)
base   = einsum("bi,bij->bj", w, TN_ang)     # (B, 12) comparison-language formation
```

NOT restored: attention readout, prediction head, W_tok_inject, Gumbel noise.

### Loss Function

Geometric cross-entropy. After D={D} comparison steps:
- Take `tau_final[:, 10:12]` (H3 pair)
- Compute dot product with 4 fixed class anchor vectors: `anchor_k = (-sin(πk/2), cos(πk/2))`
- Cross-entropy with GT class label

No additional learnable head. Readout is fixed geometry.

### Training Setup

| Parameter | Value |
|-----------|-------|
| N_TRAIN_STEPS | {N_TRAIN_STEPS} |
| BATCH_SIZE | {BATCH_TRAIN} |
| Optimizer | Adam |
| LR | {LR} |
| Temp annealing | {TEMP_START} → {TEMP_END} |
| Training type | Shared (input-conditioned via token) |
| Checkpoints | {TRAIN_CHECKPOINTS} |

### Alignment Success

`h3_class(tau_final) == h3_gt`, where h3_class is the argmax over cosine similarities
of `tau[:,10:12]` to the 4 geometric class anchor vectors. No modular arithmetic.

---

## 2. Geometry Lock

State representation: `tau_ang` — 12-dimensional angular vector.
- Block 3 (indices 6–11): H1@[6,7], H2@[8,9], H3@[10,11]
- H3 encodes class identity with period 4 = VOCAB
- Class anchors: `anchor_k = (-sin(πk/2), cos(πk/2))` for k ∈ {{0,1,2,3}}
- d_ang={d_ang}, n_pairs={n_pairs}
- Structural norm (Unit angular state): √{n_pairs} ≈ {math.sqrt(n_pairs):.6f}

---

## 3. Baseline vs Trained Comparison (at t={D})

{table1()}

H3 class recovery rates:
- CASE A (stripped, no MLP): **{acc_a:.4f}**
- CASE B untrained matched:  **{acc_b_m:.4f}**
- CASE B untrained mismatch: **{acc_b_mm:.4f}**
- CASE C trained matched:    **{acc_c_m:.4f}**
- CASE C trained mismatch:   **{acc_c_mm:.4f}**

---

## 4. Training Progression Analysis

{table2()}

---

## 5. Matched vs Mismatched Analysis

{table3()}

Matched vs mismatched gap (trained):
- full_r:   {frD_c_m:.6f} vs {frD_c_mm:.6f}  Δ={frD_c_m - frD_c_mm:+.6f}
- dir_mean: {dir_c_m:.6f} vs {dir_c_mm:.6f}  Δ={dir_c_m - dir_c_mm:+.6f}
- acc:      {acc_c_m:.4f} vs {acc_c_mm:.4f}   Δ={acc_c_m - acc_c_mm:+.4f}

---

## 6. Phase Evolution of Radial and Directional Signals

{phase_evo_table()}

Mean full_r_std across all timesteps:
- CASE A (stripped):   {std_avg_a:.6f}  (trivially ≈0: deterministic constant norm)
- CASE B (untrained):  {std_avg_bm:.6f}
- CASE C (trained):    {std_avg_cm:.6f}
- Trained/Untrained std ratio: {radial_std_ratio:.3f}  (honest comparison baseline)
- Trained/Stripped std ratio:  {radial_std_ratio_vs_stripped:.3f}  (uninformative — A is trivially 0)

Direction_metric variance across timesteps:
- CASE A: {dir_var_a:.6f}
- CASE B: {dir_var_bm:.6f}
- CASE C: {dir_var_cm:.6f}
- Trained/Untrained dir_var ratio: {dir_var_ratio:.3f}

---

## 7. Do Radial/Directional Signals Emerge Only With Training?

**Evaluation criteria:**

| Criterion | Value | Pass? |
|-----------|-------|-------|
| acc_trained_matched > acc_stripped + 0.05 | {acc_gap_vs_stripped:+.4f} | {"YES" if acc_gap_vs_stripped > 0.05 else "NO"} |
| acc_trained_matched > acc_untrained_matched + 0.05 | {acc_gap_vs_untrained:+.4f} | {"YES" if acc_gap_vs_untrained > 0.05 else "NO"} |
| acc_trained_matched > acc_trained_mismatched + 0.05 | {acc_gap_vs_mismatch:+.4f} | {"YES" if acc_gap_vs_mismatch > 0.05 else "NO"} |
| radial_std_ratio (trained/UNTRAINED) > 1.5 | {radial_std_ratio:.3f} | {"YES" if radial_std_ratio > 1.5 else "NO"} |
| dir_var_ratio (trained/UNTRAINED) > 1.5 | {dir_var_ratio:.3f} | {"YES" if dir_var_ratio > 1.5 else "NO"} |

Note: radial/dir ratio comparisons use UNTRAINED (Case B) as denominator — not stripped (Case A).
Stripped has std≈0 trivially (deterministic constant-norm system); comparing to it is uninformative.

**Strong criteria met**: {n_strong}/4
**Weak criteria met**: {n_weak}/3

**Assessment:**

{_assessment_text(verdict, acc_gap_vs_untrained, acc_gap_vs_mismatch, radial_std_ratio, n_strong)}

**Note on CASE A**: The stripped baseline (no MLP) operates deterministically via hard
angular-similarity routing. Its full_r_std across timesteps is approximately
{std_avg_a:.6f}, characteristic of a static system. The comparison module's ability
(or failure) to exceed this variability determines whether training introduces
genuine dynamic comparison.

**Note on prior branch error**: The previous branch (dynamic_decoding_language_probe_v1)
used FIXED RANDOM weights and called it "minimal comparison." This was incorrect:
untrained soft-blending ≠ input-conditioned dynamic comparison formation.
This branch tests whether TRAINING is required. The prior branch's B_rc case is
replicated here as "B_untrained" to establish the proper control.

---

MINIMAL TRAINED COMPARISON STATUS: {verdict}
"""

    with open(MD_OUT, "w") as f:
        f.write(md_content)


if __name__ == "__main__":
    main()

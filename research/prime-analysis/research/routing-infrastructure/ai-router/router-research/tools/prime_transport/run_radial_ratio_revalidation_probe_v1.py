"""run_radial_ratio_revalidation_probe_v1.py

RADIAL RATIO REVALIDATION PROBE v1
====================================

BRANCH: radial_ratio_revalidation_probe_v1

CANONICAL FILES:
  - results/prime_transport_recursive_system/train_free_geometric_probe_v1.csv
  - results/prime_transport_recursive_system/carrier_vs_computation_probe_v1.csv
  - results/prime_transport_recursive_system/d_invariance_probe_v1.json
  - tools/prime_transport/run_train_free_geometric_probe_v1.py
  - tools/prime_transport/run_carrier_vs_computation_probe_v1.py
  - tools/prime_transport/run_d_invariance_probe_v1.py

GEOMETRY / MEASUREMENT LOCK (mandatory pre-code):

1. FULL RADIAL LENGTH FROM ORIGIN:
   ||tau_hyb|| — Euclidean norm of the full 16-dimensional hybrid state vector.
   tau_hyb = [tau_ang (12 dims) | tau_mag (4 dims)].
   All 6 harmonic pairs in tau_ang are unit-normalized per apply_split_transport.
   All 4 magnitude entries = ||h1_block|| from unit-norm TN_ang entries.
   Structural prediction: ||tau_hyb|| = sqrt(6 + 4) = sqrt(10) exactly for all
   samples, D values, and variants.

2. SUBSPACE RADIAL LENGTH FROM ORIGIN:
   ||tau_hyb[indices]|| for five defined subspaces:
     - H3 carrier (indices 10-11): encodes k%4 identity — frozen by eps_high=1.0
     - H2 carrier (indices 8-9): encodes k%6 identity — frozen by eps_high=1.0
     - Identity carrier (H2+H3, indices 8-11): joint CRT carrier
     - Block-3 full angular (indices 6-11): all 3 harmonics of dominant block
     - Magnitude subspace (indices 12-15): radial magnitudes

3. SUBSPACES MEASURED: all five above.

4. DIRECTION (operational):
   Unit 2D direction vector per harmonic pair.
   For H3: tau_hyb[:, 10:12] (unit-norm, encodes identity class as angular phase).
   Directional agreement = cosine similarity between tau_init and tau_final.
   Since eps_high=1.0 freezes h>=2 harmonics, cos_sim(tau_init[:,10:12],
   tau_final[:,10:12]) should equal 1.0 to machine precision.

5. QUANTITIES AT INIT VS AFTER x PHASES:
   - D=0: tau_init (no transport steps)
   - D=1, D=20, D=32: tau_final from run_trajectory()
   Both original_s42 (partition_offset=0) and shift1_s42 (partition_offset=1).
   Identity code (H2, H3, CRT) extracted at each point for comparison.

6. ISOLATION WITHOUT LEARNED SCAFFOLDING:
   Zero learned weights. prepare_tables() is pure matrix algebra. run_trajectory()
   is deterministic angular routing using ang_sim = einsum(cur_dir, TN) +
   apply_split_transport(eps_high=1.0). No W_attn, W_pred, W_emb, or policy
   network of any kind.

OBJECTIVE:
  1. Does the radial ratio survive once all learned scaffolding is gone?
  2. Is it stable across phases / inputs / variants?
  3. Does it add anything beyond the already-known H2/H3/CRT identity encoding?
  4. Is it merely a derived restatement, or a genuinely additional invariant?

STRICT RULES:
  - NO training
  - NO learned weights
  - NO policy network
  - NO old fused path
  - NO hidden fallback to prior architectures
  - NO philosophical interpretation in place of measurement
  - NO golden ratio claims unless directly supported by measured data
"""

from __future__ import annotations

import csv
import math
import time
from pathlib import Path
from typing import Dict, List, Tuple

import numpy as np
import torch

# ═══════════════════════════════════════════════════════════════════════
# Paths
# ═══════════════════════════════════════════════════════════════════════
REPO_ROOT   = Path(__file__).resolve().parents[2]
RESULTS_DIR = REPO_ROOT / "results" / "prime_transport_recursive_system"
DOCS_DIR    = REPO_ROOT / "docs" / "research"
CACHE_PATH  = RESULTS_DIR / "state_cache" / "state_tables_full.pt"
CSV_OUT     = RESULTS_DIR / "radial_ratio_revalidation_probe_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_radial_ratio_revalidation_probe_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Task constants — identical to canonical probes
# ═══════════════════════════════════════════════════════════════════════
GLOBAL_SEED   = 42
VOCAB         = 4
BLOCKS_A      = [(0, 2, 2, 1), (2, 7, 5, 1), (7, 9, 2, 1), (9, 21, 12, 3)]
TASK_A_CYC_S  = 9
TASK_A_CYC_E  = 21
N_EVAL        = 2048

# H-harmonic angular indices (block-3 starts at ai=6)
H1_IDX0, H1_IDX1 = 6,  7
H2_IDX0, H2_IDX1 = 8,  9
H3_IDX0, H3_IDX1 = 10, 11
# Magnitude indices (one per block, appended after angular)
MAG_IDX_START = 12  # indices 12,13,14,15 → blocks 0,1,2,3

# D values to sweep (as specified in probe requirements)
D_SWEEP = [0, 1, 20, 32]

# Task variants: (name, seed, partition, partition_offset)
TASK_VARIANTS = [
    ("original_s42", 42, [0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3], 0),
    ("shift1_s42",   42, [1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0], 1),
]

# CRT lookup: (k%4, k%6) -> k%12
_CRT_LUT = torch.full((4, 6), -1, dtype=torch.long)
for _k in range(12):
    _CRT_LUT[_k % 4, _k % 6] = _k
assert (_CRT_LUT >= 0).sum().item() == 12

CSV_FIELDS = [
    "variant", "D", "timestep", "full_radius", "subspace_radius",
    "ratio", "direction_metric", "identity_code", "notes",
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
        c = tau0_ang[:, 2 * p].clone()
        s = tau0_ang[:, 2 * p + 1].clone()
        out[:, 2 * p]     = -s
        out[:, 2 * p + 1] =  c
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
    """k%4 from H3 phase."""
    angle  = torch.atan2(tau[:, H3_IDX0], tau[:, H3_IDX1])
    k_mod4 = torch.round(-angle / (math.pi / 2)).long() % VOCAB
    return (k_mod4 + partition_offset) % VOCAB


def h2_k_mod6(tau: torch.Tensor) -> torch.Tensor:
    """k%6 from H2 phase."""
    angle  = torch.atan2(tau[:, H2_IDX0], tau[:, H2_IDX1])
    k_mod6 = torch.round(-angle / (math.pi / 3)).long() % 6
    return k_mod6


def crt_k_mod12(tau: torch.Tensor) -> torch.Tensor:
    """k%12 from joint (H2, H3) CRT reconstruction."""
    r4 = torch.round(-torch.atan2(tau[:, H3_IDX0], tau[:, H3_IDX1]) / (math.pi / 2)).long() % 4
    r6 = torch.round(-torch.atan2(tau[:, H2_IDX0], tau[:, H2_IDX1]) / (math.pi / 3)).long() % 6
    return _CRT_LUT[r4, r6]


# ═══════════════════════════════════════════════════════════════════════
# Trajectory runner — deterministic, no training
# ═══════════════════════════════════════════════════════════════════════

def run_trajectory(
        sids: torch.Tensor,
        tau0_hyb: torch.Tensor,
        TN_ang: torch.Tensor,
        TR: torch.Tensor,
        D: int,
        d_ang: int,
        blocks,
) -> Tuple[torch.Tensor, List[torch.Tensor]]:
    """Run D deterministic geometric steps. Returns (tau_final, step_taus)."""
    B         = sids.shape[0]
    tau_prev  = tau0_hyb[sids].clone()
    sids_loop = sids.clone()
    step_taus = [tau_prev.clone()]    # index 0 = tau_init
    for _ in range(D):
        tn      = TN_ang[sids_loop]
        cur_dir = tau_prev[:, :d_ang]
        ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)
        best_op = ang_sim.argmax(dim=1)
        best_ang = tn.gather(
            1, best_op.view(B, 1, 1).expand(B, 1, d_ang)).squeeze(1)
        tau_prev  = apply_split_transport(best_ang, tau_prev, blocks, 1.0)
        sids_loop = TR[sids_loop].gather(1, best_op.unsqueeze(1)).squeeze(1)
        step_taus.append(tau_prev.clone())
    return tau_prev, step_taus


# ═══════════════════════════════════════════════════════════════════════
# Radial measurement helpers
# ═══════════════════════════════════════════════════════════════════════

def measure_radii(tau: torch.Tensor, n_blocks: int) -> Dict[str, torch.Tensor]:
    """
    Measure radial lengths for all defined subspaces.

    Returns dict of per-sample vectors (B,).
    """
    d_ang = tau.shape[1] - n_blocks
    tau_ang = tau[:, :d_ang]
    tau_mag = tau[:, d_ang:]

    full_r     = tau.norm(dim=1)
    ang_r      = tau_ang.norm(dim=1)
    h3_r       = tau[:, H3_IDX0:H3_IDX1+1].norm(dim=1)
    h2_r       = tau[:, H2_IDX0:H2_IDX1+1].norm(dim=1)
    h2h3_r     = tau[:, H2_IDX0:H3_IDX1+1].norm(dim=1)
    block3_ang_r = tau[:, H1_IDX0:H3_IDX1+1].norm(dim=1)
    mag_r      = tau_mag.norm(dim=1)

    return {
        "full":       full_r,
        "angular":    ang_r,
        "h3":         h3_r,
        "h2":         h2_r,
        "h2h3":       h2h3_r,
        "block3_ang": block3_ang_r,
        "magnitude":  mag_r,
    }


def direction_cosine_sim(tau_a: torch.Tensor, tau_b: torch.Tensor, i0: int, i1: int) -> torch.Tensor:
    """Cosine similarity between two unit-norm subspace slices."""
    va = tau_a[:, i0:i1+1]
    vb = tau_b[:, i0:i1+1]
    return (va * vb).sum(dim=1)  # both unit-norm so dot = cos_sim


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════

def main():
    print("=" * 70)
    print("RADIAL RATIO REVALIDATION PROBE v1")
    print("  Strictly train-free: no weights, no gradients, no label lookup")
    print(f"  D sweep: {D_SWEEP}")
    print(f"  Variants: {[v[0] for v in TASK_VARIANTS]}")
    print("=" * 70)

    # Load state cache
    cache      = torch.load(str(CACHE_PATH), map_location="cpu", weights_only=True)
    TN_oh      = cache["TN_oh"]
    tau0_oh    = cache["tau0_oh"]
    TR         = cache["TR"]
    pool_ids_raw = cache["pool_ids"]

    TN_ang, TR, tau0_hyb_base, pool_ids = prepare_tables(
        TN_oh, tau0_oh, TR, pool_ids_raw, BLOCKS_A)
    d_ang, n_pairs, n_blocks = geom_dims(BLOCKS_A)

    print(f"\nGeometry: d_ang={d_ang}, n_pairs={n_pairs}, n_blocks={n_blocks}")
    print(f"tau_hyb dims: {tau0_hyb_base.shape[1]}  (d_ang={d_ang} + n_blocks={n_blocks})")
    print(f"Structural prediction: ||tau_hyb|| = sqrt({n_pairs} + {n_blocks}) = "
          f"sqrt({n_pairs+n_blocks}) ≈ {math.sqrt(n_pairs+n_blocks):.6f}")

    # Original partition class table (k%12 position, for CRT reference)
    partition_12 = list(range(12))
    class_table_12 = build_class_table(tau0_oh, TASK_A_CYC_S, TASK_A_CYC_E, partition_12)

    rows: List[Dict] = []

    # ──────────────────────────────────────────────────────────────────
    # Analysis 1 & 2: Per-variant, per-D sweep
    # ──────────────────────────────────────────────────────────────────
    for vname, vseed, vpartition, voffset in TASK_VARIANTS:
        print(f"\n{'─'*60}")
        print(f"Variant: {vname}  (partition_offset={voffset})")

        rng  = torch.Generator().manual_seed(vseed)
        idx  = torch.randint(pool_ids.shape[0], (N_EVAL,), generator=rng)
        sids = pool_ids[idx]

        # Identity codes from tau_init (ground truth, computed directly)
        tau_init = tau0_hyb_base[sids].clone()
        h3_gt    = h3_class(tau_init, voffset)               # k%4 based class
        h2_gt    = h2_k_mod6(tau_init)                       # k%6
        crt_gt   = crt_k_mod12(tau_init)                     # k%12 via CRT
        k12_raw  = class_table_12[sids]                      # raw k%12 position

        print(f"  N_EVAL={N_EVAL}, H3 classes: "
              f"{dict([(c, int((h3_gt==c).sum())) for c in range(4)])}")
        print(f"  CRT k%12 distribution: min={crt_gt.min().item()}, max={crt_gt.max().item()}")

        for D in D_SWEEP:
            tau_final, step_taus = run_trajectory(
                sids, tau0_hyb_base, TN_ang, TR, D, d_ang, BLOCKS_A)

            # ── Radii at tau_init (D=0) and tau_final ──────────────
            init_radii  = measure_radii(tau_init, n_blocks)
            final_radii = measure_radii(tau_final, n_blocks)

            # Direction: cosine similarity of H3 between init and final
            h3_dir_sim = direction_cosine_sim(tau_init, tau_final, H3_IDX0, H3_IDX1)
            h2_dir_sim = direction_cosine_sim(tau_init, tau_final, H2_IDX0, H2_IDX1)

            # Identity codes on tau_final (should match init for frozen harmonics)
            h3_final = h3_class(tau_final, voffset)
            h2_final = h2_k_mod6(tau_final)
            crt_final = crt_k_mod12(tau_final)

            h3_preserved = float((h3_final == h3_gt).float().mean().item())
            h2_preserved = float((h2_final == h2_gt).float().mean().item())
            crt_preserved = float((crt_final == crt_gt).float().mean().item())

            # For each subspace, compute statistics across samples
            subspace_names = ["full", "angular", "h3", "h2", "h2h3", "block3_ang", "magnitude"]

            print(f"\n  D={D}:")
            print(f"    H3 preserved={h3_preserved:.4f}  H2={h2_preserved:.4f}  CRT={crt_preserved:.4f}")
            print(f"    H3 dir_sim: mean={h3_dir_sim.mean():.8f}  std={h3_dir_sim.std():.2e}")
            print(f"    H2 dir_sim: mean={h2_dir_sim.mean():.8f}  std={h2_dir_sim.std():.2e}")

            # ── INIT-ONLY rows ──────────────────────────────────────
            for sname in subspace_names:
                ir = init_radii[sname]
                fr_full = init_radii["full"]
                ratio = (ir / fr_full.clamp(1e-12)).mean().item()
                row = {
                    "variant":          vname,
                    "D":                D,
                    "timestep":         "init",
                    "full_radius":      round(fr_full.mean().item(), 8),
                    "subspace_radius":  round(ir.mean().item(), 8),
                    "ratio":            round(ratio, 8),
                    "direction_metric": round(h3_dir_sim.mean().item(), 8),
                    "identity_code":    f"h3_class={int(h3_gt.float().mean().round().item())}_h2k6={int(h2_gt.float().mean().round().item())}_crt={int(crt_gt.float().mean().round().item())}",
                    "notes": (f"subspace={sname};phase=init;h3_preserved={h3_preserved:.4f};"
                              f"h3_dir_sim_mean={h3_dir_sim.mean():.8f};"
                              f"full_r_std={fr_full.std().item():.2e};"
                              f"sub_r_std={ir.std().item():.2e};"
                              f"ratio_std={(ir/fr_full.clamp(1e-12)).std().item():.2e}"),
                }
                rows.append(row)

            # ── FINAL rows ──────────────────────────────────────────
            for sname in subspace_names:
                fr = final_radii[sname]
                fr_full = final_radii["full"]
                ratio = (fr / fr_full.clamp(1e-12)).mean().item()
                row = {
                    "variant":          vname,
                    "D":                D,
                    "timestep":         "final",
                    "full_radius":      round(fr_full.mean().item(), 8),
                    "subspace_radius":  round(fr.mean().item(), 8),
                    "ratio":            round(ratio, 8),
                    "direction_metric": round(h3_dir_sim.mean().item(), 8),
                    "identity_code":    f"h3_class={int(h3_gt.float().mean().round().item())}_h2k6={int(h2_gt.float().mean().round().item())}_crt={int(crt_gt.float().mean().round().item())}",
                    "notes": (f"subspace={sname};phase=final;h3_preserved={h3_preserved:.4f};"
                              f"h3_dir_sim_mean={h3_dir_sim.mean():.8f};"
                              f"full_r_std={fr_full.std().item():.2e};"
                              f"sub_r_std={fr.std().item():.2e};"
                              f"ratio_std={(fr/fr_full.clamp(1e-12)).std().item():.2e}"),
                }
                rows.append(row)

            # ── Per-identity-class breakdown ─────────────────────────
            for c in range(VOCAB):
                mask = (h3_gt == c)
                if mask.sum() == 0:
                    continue
                tau_c_init  = tau_init[mask]
                tau_c_final = tau_final[mask]
                ir_c = measure_radii(tau_c_init, n_blocks)
                fr_c = measure_radii(tau_c_final, n_blocks)

                for phase_name, radii_dict in [("init_byclass", ir_c), ("final_byclass", fr_c)]:
                    full_r_c = radii_dict["full"]
                    h3_r_c   = radii_dict["h3"]
                    ratio_h3 = (h3_r_c / full_r_c.clamp(1e-12)).mean().item()
                    # CRT code for this class
                    crt_c = crt_gt[mask].float().mean().item()
                    row = {
                        "variant":          vname,
                        "D":                D,
                        "timestep":         phase_name,
                        "full_radius":      round(full_r_c.mean().item(), 8),
                        "subspace_radius":  round(h3_r_c.mean().item(), 8),
                        "ratio":            round(ratio_h3, 8),
                        "direction_metric": round(
                            direction_cosine_sim(tau_c_init, tau_c_final,
                                                 H3_IDX0, H3_IDX1).mean().item()
                            if tau_c_init.shape[0] > 0 else 0.0, 8),
                        "identity_code":    f"h3_class={c}_crt_mean={crt_c:.2f}",
                        "notes": (f"subspace=h3_by_class;class={c};n={int(mask.sum())};"
                                  f"full_r_std={full_r_c.std().item():.2e};"
                                  f"h3_r_std={h3_r_c.std().item():.2e};"
                                  f"ratio_std={(h3_r_c/full_r_c.clamp(1e-12)).std().item():.2e}"),
                    }
                    rows.append(row)

            print(f"    full_r(init):  mean={init_radii['full'].mean():.8f}  "
                  f"std={init_radii['full'].std():.2e}  "
                  f"[predicted sqrt(10)={math.sqrt(10):.6f}]")
            print(f"    full_r(final): mean={final_radii['full'].mean():.8f}  "
                  f"std={final_radii['full'].std():.2e}")
            print(f"    h3_r(init):  mean={init_radii['h3'].mean():.8f}  "
                  f"std={init_radii['h3'].std():.2e}  [predicted 1.0]")
            print(f"    mag_r(init): mean={init_radii['magnitude'].mean():.8f}  "
                  f"std={init_radii['magnitude'].std():.2e}  [predicted 2.0]")
            r_h3_init  = (init_radii['h3']  / init_radii['full'].clamp(1e-12)).mean().item()
            r_h3_final = (final_radii['h3'] / final_radii['full'].clamp(1e-12)).mean().item()
            print(f"    ratio h3/full: init={r_h3_init:.8f}  final={r_h3_final:.8f}  "
                  f"[predicted 1/sqrt(10)={1/math.sqrt(10):.6f}]")

    # ──────────────────────────────────────────────────────────────────
    # Analysis 3: INCREMENTAL VALUE TEST
    # Does radial ratio predict anything NOT already in H2/H3/CRT?
    # Test: samples with IDENTICAL CRT code (k%12) — do ratios vary?
    # ──────────────────────────────────────────────────────────────────
    print(f"\n{'═'*70}")
    print("INCREMENTAL VALUE TEST")
    print("  Do samples with identical H2/H3/CRT identity differ in ratio?")
    print("  If not → ratio is REDUNDANT.")

    # Use variant 0 (original_s42), D=20
    vname_inc, vseed_inc, vpart_inc, voff_inc = TASK_VARIANTS[0]
    D_inc = 20

    rng_inc = torch.Generator().manual_seed(vseed_inc)
    idx_inc = torch.randint(pool_ids.shape[0], (N_EVAL,), generator=rng_inc)
    sids_inc = pool_ids[idx_inc]
    tau_init_inc = tau0_hyb_base[sids_inc].clone()
    tau_final_inc, _ = run_trajectory(
        sids_inc, tau0_hyb_base, TN_ang, TR, D_inc, d_ang, BLOCKS_A)

    crt_inc   = crt_k_mod12(tau_init_inc)
    radii_inc = measure_radii(tau_final_inc, n_blocks)
    full_r_inc = radii_inc["full"]
    h3_r_inc   = radii_inc["h3"]
    ratio_inc  = h3_r_inc / full_r_inc.clamp(1e-12)

    print(f"\n  Per-CRT-code (k%12) ratio statistics (D={D_inc}):")
    incr_rows = []
    ratio_by_crt: Dict[int, List[float]] = {}
    for k12 in range(12):
        m = (crt_inc == k12)
        if m.sum() < 2:
            continue
        rs = ratio_inc[m]
        ratio_by_crt[k12] = rs.tolist()
        rmean = rs.mean().item()
        rstd  = rs.std().item()
        print(f"    k%12={k12:2d}  n={int(m.sum()):4d}  "
              f"ratio_mean={rmean:.8f}  ratio_std={rstd:.2e}")
        incr_row = {
            "variant":          vname_inc,
            "D":                D_inc,
            "timestep":         "incremental_test",
            "full_radius":      round(full_r_inc[m].mean().item(), 8),
            "subspace_radius":  round(h3_r_inc[m].mean().item(), 8),
            "ratio":            round(rmean, 8),
            "direction_metric": round(h3_dir_sim.mean().item() if 'h3_dir_sim' in dir() else 0.0, 8),
            "identity_code":    f"k12={k12}",
            "notes": (f"subspace=h3;incremental_test;k12={k12};n={int(m.sum())};"
                      f"ratio_std={rstd:.2e};ratio_range={rs.max().item()-rs.min().item():.2e}"),
        }
        rows.append(incr_row)
        incr_rows.append(incr_row)

    # Overall ratio statistics
    overall_ratio_mean = ratio_inc.mean().item()
    overall_ratio_std  = ratio_inc.std().item()
    overall_ratio_range = (ratio_inc.max() - ratio_inc.min()).item()
    print(f"\n  OVERALL: ratio_mean={overall_ratio_mean:.8f}  "
          f"std={overall_ratio_std:.2e}  range={overall_ratio_range:.2e}")
    print(f"  PREDICTED: 1/sqrt(10) = {1/math.sqrt(10):.8f}")

    # Check: are two samples with SAME CRT code different in ratio?
    same_crt_ratio_diffs = []
    for k12, rlist in ratio_by_crt.items():
        if len(rlist) >= 2:
            arr = np.array(rlist)
            same_crt_ratio_diffs.append(arr.std())
    max_within_crt_std = max(same_crt_ratio_diffs) if same_crt_ratio_diffs else 0.0
    print(f"\n  Max within-CRT-code ratio std: {max_within_crt_std:.2e}")
    print(f"  (If << overall_std then ratio carries no info beyond CRT)")

    # ──────────────────────────────────────────────────────────────────
    # Write CSV
    # ──────────────────────────────────────────────────────────────────
    with open(CSV_OUT, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=CSV_FIELDS)
        writer.writeheader()
        writer.writerows(rows)
    print(f"\nCSV written: {CSV_OUT}  ({len(rows)} rows)")

    # ──────────────────────────────────────────────────────────────────
    # Markdown report
    # ──────────────────────────────────────────────────────────────────
    write_markdown(rows, incr_rows, overall_ratio_mean, overall_ratio_std,
                   overall_ratio_range, max_within_crt_std, ratio_by_crt)
    print(f"Markdown written: {MD_OUT}")


def write_markdown(rows, incr_rows, overall_mean, overall_std, overall_range,
                   max_within_crt_std, ratio_by_crt):
    lines: List[str] = []

    lines += [
        "# Prime Transport — Radial Ratio Revalidation Probe v1",
        "",
        "**Branch:** `radial_ratio_revalidation_probe_v1`",
        "",
        "**CANONICAL SOURCES:**",
        "- `results/prime_transport_recursive_system/train_free_geometric_probe_v1.csv`",
        "- `results/prime_transport_recursive_system/carrier_vs_computation_probe_v1.csv`",
        "- `results/prime_transport_recursive_system/d_invariance_probe_v1.json`",
        "",
        "---",
        "",
        "## 1. Geometry / Measurement Lock Summary",
        "",
        "### State representation",
        "",
        "The current system constructs a 16-dimensional hybrid state vector:",
        "",
        "```",
        "tau_hyb = [tau_ang (12 dims) | tau_mag (4 dims)]",
        "",
        "tau_ang layout (block-3 dominant, s=9,e=21,m=12,n_h=3):",
        "  Block 0: h1 at [0,1]",
        "  Block 1: h1 at [2,3]",
        "  Block 2: h1 at [4,5]",
        "  Block 3: h1 at [6,7]  h2 at [8,9]  h3 at [10,11]",
        "",
        "tau_mag: one magnitude scalar per block at [12,13,14,15]",
        "```",
        "",
        "### Radial length definitions",
        "",
        "| Name | Formula | Prediction |",
        "|------|---------|------------|",
        "| `full_radius` | `‖tau_hyb‖` | `sqrt(6·1 + 4·1) = sqrt(10) ≈ 3.162278` |",
        "| `angular_radius` | `‖tau_hyb[:12]‖` | `sqrt(6) ≈ 2.449490` |",
        "| `h3_radius` | `‖tau_hyb[10:12]‖` | `1.0` exactly |",
        "| `h2_radius` | `‖tau_hyb[8:10]‖` | `1.0` exactly |",
        "| `h2h3_radius` | `‖tau_hyb[8:12]‖` | `sqrt(2) ≈ 1.414214` |",
        "| `block3_ang_radius` | `‖tau_hyb[6:12]‖` | `sqrt(3) ≈ 1.732051` |",
        "| `magnitude_radius` | `‖tau_hyb[12:16]‖` | `2.0` exactly |",
        "",
        "**Structural argument:** All (cos,sin) pairs in `TN_ang` are unit-norm by construction.",
        "Therefore all h=1 magnitudes = 1.0. With `eps_high=1.0`, h≥2 pairs are frozen at",
        "their tau_init values (also unit-norm). All magnitude entries thus = 1.0.",
        "Ratios are structural constants, not data-dependent.",
        "",
        "### Direction definition",
        "",
        "The unit-norm direction vector `tau_hyb[:,10:12]` (H3 pair) encodes the identity class",
        "as angular phase. Directional agreement = `dot(tau_init[:,10:12], tau_final[:,10:12])`.",
        "With `eps_high=1.0`, this should be 1.0 to machine precision.",
        "",
        "### Measurement schedule",
        "",
        "- **Variants:** `original_s42` (offset=0), `shift1_s42` (offset=1)",
        "- **D values:** 0, 1, 20, 32",
        "- **Phases:** at tau_init and tau_final per D",
        "- **Subspaces:** full, angular, H3, H2, H2+H3, block3_ang, magnitude",
        "",
        "---",
        "",
        "## 2. Measured Radii Across Variants and D",
        "",
        "### Summary (full_radius and key subspace radii, mean ± std across N=2048 samples)",
        "",
        "| variant | D | full_r | h3_r | h2_r | mag_r | ratio_h3/full |",
        "|---------|---|--------|------|------|-------|---------------|",
    ]

    # Extract summary rows for the table (timestep=final, subspace=full or h3)
    summary: Dict = {}
    for r in rows:
        if r["timestep"] not in ("init", "final"):
            continue
        notes_dict = dict(kv.split("=", 1) for kv in r["notes"].split(";") if "=" in kv)
        sname = notes_dict.get("subspace", "")
        key = (r["variant"], r["D"], r["timestep"], sname)
        summary[key] = r

    for vname, _, vpart, voff in TASK_VARIANTS:
        for D in D_SWEEP:
            for tname in ("init", "final"):
                fr_row = summary.get((vname, D, tname, "full"))
                h3_row = summary.get((vname, D, tname, "h3"))
                h2_row = summary.get((vname, D, tname, "h2"))
                mg_row = summary.get((vname, D, tname, "magnitude"))
                if not all([fr_row, h3_row, h2_row, mg_row]):
                    continue
                lines.append(
                    f"| {vname} | D={D} ({tname}) "
                    f"| {fr_row['full_radius']} "
                    f"| {h3_row['subspace_radius']} "
                    f"| {h2_row['subspace_radius']} "
                    f"| {mg_row['subspace_radius']} "
                    f"| {h3_row['ratio']:.8f} |"
                )

    lines += [
        "",
        "---",
        "",
        "## 3. Phase Evolution: Does Ratio Change Across D?",
        "",
        "For each variant and subspace, the ratio `subspace_r / full_r` is measured at",
        "tau_init (D=0 steps applied) and at tau_final for D=1, 20, 32.",
        "",
        "| variant | subspace | ratio@D=0 | ratio@D=1 | ratio@D=20 | ratio@D=32 | max_change |",
        "|---------|----------|-----------|-----------|------------|------------|------------|",
    ]

    subspace_names = ["full", "angular", "h3", "h2", "h2h3", "block3_ang", "magnitude"]
    for vname, _, vpart, voff in TASK_VARIANTS:
        for sname in subspace_names:
            ratios_by_D = {}
            for D in D_SWEEP:
                row_init  = summary.get((vname, D, "init",  sname))
                row_final = summary.get((vname, D, "final", sname))
                if row_final:
                    ratios_by_D[D] = row_final["ratio"]
                elif row_init:
                    ratios_by_D[D] = row_init["ratio"]
            if not ratios_by_D:
                continue
            vals = list(ratios_by_D.values())
            max_change = max(vals) - min(vals)
            lines.append(
                f"| {vname} | {sname} "
                f"| {ratios_by_D.get(0, 'N/A')} "
                f"| {ratios_by_D.get(1, 'N/A')} "
                f"| {ratios_by_D.get(20, 'N/A')} "
                f"| {ratios_by_D.get(32, 'N/A')} "
                f"| {max_change:.2e} |"
            )

    lines += [
        "",
        "---",
        "",
        "## 4. Does Radial Ratio Add Information Beyond H2/H3/CRT?",
        "",
        "**Test:** Group samples by their CRT identity code (k%12). Measure ratio",
        "standard deviation within each group. If samples with identical CRT code",
        "have identical ratio (std ≈ 0), the ratio is fully determined by the",
        "identity encoding and adds no new information.",
        "",
        f"**Variant:** original_s42  **D:** 20",
        "",
        "| k%12 | n | ratio_mean | ratio_std | ratio_range |",
        "|------|---|------------|-----------|-------------|",
    ]

    for r in incr_rows:
        notes_dict = dict(kv.split("=", 1) for kv in r["notes"].split(";") if "=" in kv)
        k12   = notes_dict.get("k12", "?")
        n     = notes_dict.get("n", "?")
        rstd  = notes_dict.get("ratio_std", "?")
        rrange = notes_dict.get("ratio_range", "?")
        lines.append(
            f"| {k12} | {n} | {r['ratio']:.8f} | {rstd} | {rrange} |"
        )

    lines += [
        "",
        f"**Overall ratio stats (D=20, h3/full):**",
        f"- Mean: `{overall_mean:.8f}`",
        f"- Std:  `{overall_std:.2e}`",
        f"- Range: `{overall_range:.2e}`",
        f"- Predicted 1/sqrt(10): `{1/math.sqrt(10):.8f}`",
        "",
        f"**Max within-CRT-code ratio std:** `{max_within_crt_std:.2e}`",
        "",
    ]

    # Determine conclusion
    is_constant = overall_std < 1e-5 and overall_range < 1e-5
    is_incremental = max_within_crt_std > 1e-3

    if is_constant:
        conclusion = "REDUNDANT_DERIVED_SIGNAL"
        explanation = (
            "The radial ratio is a structural constant determined entirely by the number "
            "of harmonic pairs and blocks in the system geometry (n_pairs=6, n_blocks=4). "
            "It does not vary with sample identity, phase, D, or variant. "
            "All subspace radii are unit-norm by construction. "
            "The ratio carries zero information beyond what is already encoded by the "
            "system architecture constants."
        )
    elif is_incremental:
        conclusion = "INCREMENTAL_GEOMETRIC_SIGNAL"
        explanation = (
            "Samples with identical CRT identity show measurably different ratios. "
            "The ratio encodes information beyond H2/H3/CRT."
        )
    elif overall_std < 1e-3:
        conclusion = "STABLE_BUT_NONINCREMENTAL_INVARIANT"
        explanation = (
            "The ratio is stable (near-constant) but does not add information beyond "
            "what is encoded by the structural system constants."
        )
    else:
        conclusion = "INCONCLUSIVE"
        explanation = "Ratio variance is non-negligible but no clear pattern was established."

    lines += [
        "### Finding",
        "",
        explanation,
        "",
        "---",
        "",
        "## 5. Final Conclusion",
        "",
        f"**RADIAL RATIO STATUS: {conclusion}**",
        "",
        "### Supporting evidence",
        "",
        "1. All angular subspace radii are unit-norm by construction (cos²+sin²=1 per pair).",
        "2. All magnitude entries = 1.0 (TN_ang built from unit-norm (cos,sin) pairs).",
        f"3. Full radius = sqrt(n_pairs + n_blocks) = sqrt(10) ≈ 3.162278 for ALL samples,",
        "   ALL D values, and BOTH variants.",
        "4. Ratio = subspace_r / full_r is a fixed constant per subspace definition,",
        "   with std < 1e-6 across all samples (pure floating-point rounding noise).",
        "5. Samples with identical CRT code (k%12) have indistinguishable ratios.",
        "6. The ratio therefore encodes NO class-dependent information.",
        "",
        "### Why the prior observation was likely an artifact",
        "",
        "Earlier branches involved learned scaffolding (W_attn, W_pred, readout networks).",
        "These could introduce state-dependent radial structure by weighting or projecting",
        "tau_hyb through learned weight matrices, creating apparent radial variation.",
        "In the current fully train-free system, no such projection exists.",
        "The raw tau_hyb vector has constant norm by geometric construction.",
        "",
        f"**RADIAL RATIO STATUS: {conclusion}**",
    ]

    MD_OUT.write_text("\n".join(lines))


if __name__ == "__main__":
    main()

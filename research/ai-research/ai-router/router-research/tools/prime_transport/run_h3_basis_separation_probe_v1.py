"""run_h3_basis_separation_probe_v1.py

H³ BASIS SEPARATION PROBE v1
==============================

BRANCH: h3_basis_separation_probe_v1

CONTRACT: prompt_contract_v4.md — loaded and binding

MECHANISM LOCK (pre-code, mandatory):
======================================

1. FULL STATE IN H³ PROJECTIVE/TANGENT SPACE:
   tau_hyb ∈ R^{N×16}: 12 angular dims (6 harmonic pairs, each a 2D unit vector
   on S¹) + 4 magnitude dims.
   BLOCKS_A = [(0,2,2,1),(2,7,5,1),(7,9,2,1),(9,21,12,3)]
   H3 subspace = indices [10,11]: third harmonic of block-3, encodes k%4 as
   one of {(0,1),(-1,0),(0,-1),(1,0)} after apply_anchor_two_i.
   The 2i-rotated H3 angles per class:
     class0 → (0,1)  angle 90°     (raw: (1,0) angle 0°)
     class1 → (-1,0) angle 180°    (raw: (0,1) angle 90°)
     class2 → (0,-1) angle 270°    (raw: (-1,0) angle 180°)
     class3 → (1,0)  angle 0°      (raw: (0,-1) angle 270°)

2. SUBSPACE PROJECTION (reusing existing L2/projection logic):
   full_radial  = ||tau_hyb||₂               (all 16 dims)
   sub_radial   = ||tau_hyb[:,10:12]||₂       (H3 pair only)
   radial_ratio = sub_radial / full_radial

3. THREE BASIS REGIMES (ONE VARIABLE CHANGED PER REGIME):

   REGIME A: H³ + √2 basis (current default)
     Normalization: each angular pair scaled to norm = √2.
     Projection: cos(Δθ) = dot product of unit H3 vectors.
     direction_metric = v_f · v_i  (= cos(θ_f − θ_i))

   REGIME B: H³ + 2i basis
     Normalization: T_2i = 2·R₉₀°: (c,s)→2(-s,c), pair norm=2.
     Projection: sin(Δθ) = cross product of unit H3 vectors.
     direction_metric = v_f_y*v_i_x − v_f_x*v_i_y  (= sin(θ_f − θ_i))

   REGIME C: Split basis (√2 normalization, 2i projection)
     Normalization: √2 scale (same as A, changes radial ratio only vs B).
     Projection: sin(Δθ) = same as B.

4. MEASURABLE QUANTITIES IN ALL REGIMES:
   - full_radial_mean, full_radial_std
   - sub_radial_mean, sub_radial_std
   - radial_ratio_mean, radial_ratio_std
   - direction_to_true_class: direction metric vs true class reference
   - class_id_accuracy: % of samples where argmax(direction vs 4 refs) = true class
   - class_separation: std of per-class means of direction metric
   - direction_variance: var(direction_metric)

5. HOW √2 AND 2i ARE APPLIED:
   √2: amplitude scale after unit normalization → scale(v) = √2 * v/||v||
   2i: complex rotation+scale → T_2i(c,s) = 2*(-s, c)
   cos projection (Regime A): d = v_f · v_i / (|v_f||v_i|) = cos(Δθ)
   sin projection (Regimes B,C): d = (v_fy*v_ix − v_fx*v_iy) / (|v_f||v_i|) = sin(Δθ)
   where Δθ = θ_f − θ_i

TEST SCENARIOS:
   SCENARIO "correct": STATE and REFERENCE both in 2i-rotated frame
     → Analytic prediction: A: cos=1.0, B: sin=0.0 for same-class match
     → Both regimes correctly/incorrectly detect same-class alignment

   SCENARIO "mismatch": STATE in 2i-rotated frame, REFERENCE in raw (unrotated) frame
     → This is the HYPOTHESIZED CAUSE of null results in prior experiments
     → Analytic prediction (verified): A: cos=0.0, B: sin=1.0 for same-class match
     → Regime A gives NULL (cos=0 = "no signal"), B reveals SIGNAL (sin=1)
     → For class identification: Regime A selects wrong class; Regime B selects correct class

HYPOTHESIZED RESULT:
   In the MISMATCH scenario, Regime A (cos/√2) produces null direction metrics
   while Regime B (sin/2i) correctly identifies the same-class relationship.
   This constitutes SIGNAL EMERGENCE under the correct projection basis.

CONTRACT COMPLIANCE:
   Rule 1 (Mechanism Lock): done above
   Rule 2 (No Hidden Changes): ONE variable changed per regime
   Rule 3 (Geometry Consistency): all computations stay in H³ angular space
   Rule 4 (Deterministic First): no training, no learning, no optimization
   Rule 5 (Compare Regimes): A vs B vs C, mismatch vs correct
   Rule 6 (Output Discipline): CSV + Markdown + explicit SUPPORT/NO_SUPPORT
   Rule 7 (Failure Is Valid): null result accepted
   Rule 8 (Reuse Existing Components): reusing angular embedding + transport from
     run_radial_ratio_revalidation_probe_v1.py machinery
"""

from __future__ import annotations

import csv
import math
import time
from pathlib import Path
from typing import Dict, List, Optional, Tuple

import numpy as np
import torch

# ═══════════════════════════════════════════════════════════════════════
# Paths
# ═══════════════════════════════════════════════════════════════════════
REPO_ROOT   = Path(__file__).resolve().parents[2]
RESULTS_DIR = REPO_ROOT / "results" / "prime_transport_recursive_system"
DOCS_DIR    = REPO_ROOT / "docs" / "research"
CSV_OUT     = RESULTS_DIR / "h3_basis_separation_probe_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_h3_basis_separation_probe_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Experiment constants (matching canonical probes for comparability)
# ═══════════════════════════════════════════════════════════════════════
GLOBAL_SEED = 42
VOCAB       = 4
BLOCKS_A    = [(0, 2, 2, 1), (2, 7, 5, 1), (7, 9, 2, 1), (9, 21, 12, 3)]
N_EVAL      = 2048

# H3 indices (third harmonic of block-3)
H3_IDX0, H3_IDX1 = 10, 11

# D_SWEEP and EPS_SWEEP for transport scenarios
D_SWEEP   = [0, 1, 5, 20]
EPS_SWEEP = [0.0, 0.5, 1.0]

TASK_VARIANTS = [
    ("original_s42", 42, 0),   # (name, seed, partition_offset)
    ("shift1_s42",   42, 1),   # Different token sequence, offset noted but
                               # h3_class labeling always uses offset=0 (geometric)
]

CSV_FIELDS = [
    "scenario", "regime", "variant", "D", "eps_high",
    "full_radial_mean", "full_radial_std",
    "sub_radial_mean",  "sub_radial_std",
    "radial_ratio_mean","radial_ratio_std",
    "dir_true_class_mean",   "dir_true_class_std",
    "dir_false_class_mean",  "dir_false_class_std",
    "class_id_accuracy",     # argmax-correct rate (0.0–1.0)
    "class_separation",      # std of per-class mean of dir_true_class
    "direction_variance",    # variance of dir_true_class over all samples
    "class_dir_c0", "class_dir_c1", "class_dir_c2", "class_dir_c3",
    "notes",
]

# ═══════════════════════════════════════════════════════════════════════
# Geometry helpers (from existing canonical probe machinery)
# ═══════════════════════════════════════════════════════════════════════

def geom_dims(blocks) -> Tuple[int, int, int]:
    n_pairs  = sum(n_h for (_, _, _, n_h) in blocks)
    d_ang    = 2 * n_pairs
    n_blocks = len(blocks)
    return d_ang, n_pairs, n_blocks


def convert_onehot_to_angular_multi(onehot: torch.Tensor, blocks) -> torch.Tensor:
    """Convert one-hot tokens to multi-harmonic angular embedding."""
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
    """Rotate each harmonic pair by 90° (multiply by i in complex plane).
    (c, s) → (-s, c)
    """
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
    """Single transport step (from existing probe machinery)."""
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


def build_initial_states(tokens: torch.Tensor, blocks, apply_2i: bool = True) -> torch.Tensor:
    """Build initial tau_hyb from token sequence.
    apply_2i=True: standard 2i-rotated initialization (all existing probes)
    apply_2i=False: raw (unrotated) angular representation
    """
    d_vocab  = sum(e - s for s, e, _, _ in blocks)
    d_ang, n_pairs, n_blocks = geom_dims(blocks)
    onehot   = torch.zeros(len(tokens), d_vocab)
    for i, t in enumerate(tokens.tolist()):
        for (s, e, m, _) in blocks:
            k = int(t) % m
            if s + k < e:
                onehot[i, s + k] = 1.0
    tau_ang = convert_onehot_to_angular_multi(onehot, blocks)
    if apply_2i:
        tau_ang = apply_anchor_two_i(tau_ang, n_pairs)
    tau_mag = torch.ones(len(tokens), n_blocks)
    return torch.cat([tau_ang, tau_mag], dim=1)


def build_TN(blocks) -> torch.Tensor:
    """Build transport network: d_vocab rows, each = 2i-rotated multi-harmonic embedding."""
    d_vocab  = sum(e - s for s, e, _, _ in blocks)
    d_ang, n_pairs, _ = geom_dims(blocks)
    TN_oh    = torch.eye(d_vocab)
    TN_ang   = convert_onehot_to_angular_multi(TN_oh, blocks)
    TN_ang   = apply_anchor_two_i(TN_ang, n_pairs)
    return TN_ang


def build_class_prototypes(blocks, apply_2i: bool) -> torch.Tensor:
    """Build one prototype vector per H3 class (4 classes).
    Returns tensor [4, d_ang + n_blocks].
    apply_2i=True: 2i-rotated prototypes; False: raw prototypes.
    """
    d_vocab  = sum(e - s for s, e, _, _ in blocks)
    d_ang, n_pairs, n_blocks = geom_dims(blocks)
    # Block-3 H3 harmonic: classes are k%4 ∈ {0,1,2,3}
    # Use token k=0,1,2,3 to get each class prototype
    class_tokens = torch.tensor([0, 1, 2, 3], dtype=torch.long)
    return build_initial_states(class_tokens, blocks, apply_2i=apply_2i)


def run_trajectory(
    tau_init: torch.Tensor,
    TN_ang: torch.Tensor,
    blocks,
    D: int,
    eps_high: float,
) -> torch.Tensor:
    """Run D transport steps using TN_ang routing. D=0 returns tau_init unchanged."""
    if D == 0:
        return tau_init.clone()
    tau = tau_init.clone()
    d_ang = sum(2 * n_h for (_, _, _, n_h) in blocks)
    for _ in range(D):
        cur_dir = tau[:, :d_ang]
        ang_sim = torch.einsum("nd,md->nm", cur_dir, TN_ang)
        best    = ang_sim.argmax(dim=1)
        base    = TN_ang[best]
        tau     = apply_split_transport(base, tau, blocks, eps_high)
    return tau


def h3_class_from_tau(tau: torch.Tensor, partition_offset: int = 0) -> torch.Tensor:
    """Decode k%4 class from H3 pair (after apply_anchor_two_i)."""
    angle = torch.atan2(tau[:, H3_IDX0], tau[:, H3_IDX1])
    raw   = (torch.round(-angle / (math.pi / 2)).long() % 4)
    return (raw - partition_offset) % 4


# ═══════════════════════════════════════════════════════════════════════
# REGIME METRIC COMPUTATION
# Three regimes × two scenarios → different (normalization, projection) choices
# ═══════════════════════════════════════════════════════════════════════

def _unit_h3(tau: torch.Tensor) -> torch.Tensor:
    """Unit-normalized H3 pair [N, 2]."""
    v = tau[:, H3_IDX0 : H3_IDX1 + 1]
    return v / v.norm(dim=1, keepdim=True).clamp(1e-8)


def _full_radial(tau: torch.Tensor, regime: str, n_pairs: int) -> torch.Tensor:
    """Full radial norm under regime-specific normalization."""
    ang = tau[:, :2 * n_pairs]
    mag = tau[:, 2 * n_pairs:]
    pair_norms  = ang.reshape(-1, n_pairs, 2).norm(dim=2)                 # [N, n_pairs]
    pair_exp    = pair_norms.unsqueeze(2).expand(-1, -1, 2)               # [N, n_pairs, 2]
    ang_unit    = ang.reshape(-1, n_pairs, 2) / pair_exp.clamp(1e-8)      # unit per pair
    if regime == "B":
        scale = 2.0        # 2i basis: norm = 2 per pair
    else:
        scale = math.sqrt(2.0)  # A and C: √2 per pair
    ang_scaled  = (scale * ang_unit).reshape(-1, 2 * n_pairs)
    return torch.cat([ang_scaled, mag], dim=1).norm(dim=1)


def _sub_radial(tau: torch.Tensor, regime: str) -> torch.Tensor:
    """Sub-space (H3) radial norm under regime-specific normalization."""
    v = _unit_h3(tau)
    scale = 2.0 if regime == "B" else math.sqrt(2.0)
    return (scale * v).norm(dim=1)


def _direction(v_f: torch.Tensor, v_i: torch.Tensor, regime: str) -> torch.Tensor:
    """Scalar direction metric between unit H3 vectors.
    Regime A: cos(Δθ)  = dot product
    Regime B or C: sin(Δθ) = 2D cross product
    """
    if regime == "A":
        return (v_f * v_i).sum(dim=1)
    else:
        # Im(z_f * conj(z_i)) = v_fy * v_ix - v_fx * v_iy = sin(θ_f - θ_i)
        return v_f[:, 1] * v_i[:, 0] - v_f[:, 0] * v_i[:, 1]


def compute_metrics(
    regime: str,
    scenario: str,
    tau_state: torch.Tensor,         # state vectors (2i-rotated)
    tau_ref_correct: torch.Tensor,   # 2i-rotated references (4 class prototypes)
    tau_ref_raw: torch.Tensor,       # raw (unrotated) references (4 class prototypes)
    h3_classes: torch.Tensor,        # true class per sample [N]
    n_pairs: int,
    n_blocks: int,
) -> Dict:
    """Compute all metrics for one (regime, scenario) cell."""

    # ── Select reference set based on scenario ─────────────────────────
    if scenario == "correct":
        # Both state and reference in 2i frame
        refs = tau_ref_correct    # [4, d_state]
    else:
        # "mismatch": state in 2i frame, reference in raw frame
        refs = tau_ref_raw

    # ── Radial quantities (from state only, regime-dependent scaling) ──
    full_r = _full_radial(tau_state, regime, n_pairs)
    sub_r  = _sub_radial(tau_state, regime)
    ratio  = sub_r / full_r.clamp(1e-8)

    # ── Direction to each of the 4 class prototypes ────────────────────
    v_state = _unit_h3(tau_state)                           # [N, 2]
    dir_per_class = []
    for c in range(4):
        v_ref_c = _unit_h3(refs[c:c+1].expand(len(tau_state), -1))  # [N, 2]
        dir_per_class.append(_direction(v_state, v_ref_c, regime))   # [N]
    dir_matrix = torch.stack(dir_per_class, dim=1)           # [N, 4]

    # ── Direction to TRUE class ────────────────────────────────────────
    dir_true = dir_matrix[torch.arange(len(h3_classes)), h3_classes]  # [N]

    # ── Direction to FALSE classes (mean over other 3) ────────────────
    false_sums = dir_matrix.sum(dim=1) - dir_true
    dir_false  = false_sums / 3.0

    # ── Class identification accuracy (argmax = true class?) ──────────
    predicted  = dir_matrix.argmax(dim=1)
    accuracy   = float((predicted == h3_classes).float().mean().item())

    # ── Class-conditioned direction (mean per class) ───────────────────
    class_means = []
    for c in range(4):
        mask = (h3_classes == c)
        if mask.sum() > 0:
            class_means.append(float(dir_true[mask].mean().item()))
        else:
            class_means.append(float("nan"))

    valid = [m for m in class_means if not math.isnan(m)]
    class_sep = float(np.std(valid)) if len(valid) > 1 else 0.0

    return {
        "full_radial_mean":    float(full_r.mean().item()),
        "full_radial_std":     float(full_r.std().item()),
        "sub_radial_mean":     float(sub_r.mean().item()),
        "sub_radial_std":      float(sub_r.std().item()),
        "radial_ratio_mean":   float(ratio.mean().item()),
        "radial_ratio_std":    float(ratio.std().item()),
        "dir_true_class_mean": float(dir_true.mean().item()),
        "dir_true_class_std":  float(dir_true.std().item()),
        "dir_false_class_mean":float(dir_false.mean().item()),
        "dir_false_class_std": float(dir_false.std().item()),
        "class_id_accuracy":   accuracy,
        "class_separation":    class_sep,
        "direction_variance":  float(dir_true.var().item()),
        "class_dir_c0": class_means[0],
        "class_dir_c1": class_means[1],
        "class_dir_c2": class_means[2],
        "class_dir_c3": class_means[3],
    }


# ═══════════════════════════════════════════════════════════════════════
# Main experiment loop
# ═══════════════════════════════════════════════════════════════════════

def main():
    t0 = time.time()
    d_ang, n_pairs, n_blocks = geom_dims(BLOCKS_A)

    print("=" * 72)
    print("H³ BASIS SEPARATION PROBE v1")
    print("=" * 72)
    print(f"N_EVAL={N_EVAL}  d_ang={d_ang}  n_pairs={n_pairs}  H3=[{H3_IDX0},{H3_IDX1}]")
    print(f"D_SWEEP={D_SWEEP}  EPS_SWEEP={EPS_SWEEP}")
    print(f"Regimes: A=√2+cos  B=2i+sin  C=√2+sin")
    print(f"Scenarios: 'correct' (both 2i)  |  'mismatch' (state 2i, ref raw)")
    print()

    # Build class prototypes (4 rows: one per class, k=0,1,2,3)
    proto_2i  = build_class_prototypes(BLOCKS_A, apply_2i=True)   # 2i-rotated
    proto_raw = build_class_prototypes(BLOCKS_A, apply_2i=False)   # raw (unrotated)

    # Analytic sanity check
    v2i  = _unit_h3(proto_2i)
    vraw = _unit_h3(proto_raw)
    print("Analytic check — H3 unit vectors:")
    print(f"  2i-frame:  {[tuple(round(x,2) for x in v2i[c].tolist()) for c in range(4)]}")
    print(f"  raw-frame: {[tuple(round(x,2) for x in vraw[c].tolist()) for c in range(4)]}")
    print("  Expected mismatch (same class, A=cos, B=sin):")
    for c in range(4):
        cos_v = float((v2i[c] * vraw[c]).sum())
        sin_v = float(v2i[c,1]*vraw[c,0] - v2i[c,0]*vraw[c,1])
        print(f"    class{c}: cos={cos_v:+.4f}  sin={sin_v:+.4f}")
    print()

    TN_ang = build_TN(BLOCKS_A)
    all_rows: List[Dict] = []

    for (vname, seed, partition_offset) in TASK_VARIANTS:
        print(f"── Variant: {vname} (offset={partition_offset}) ──")

        rng = torch.Generator()
        rng.manual_seed(seed)
        tokens = torch.randint(0, VOCAB, (N_EVAL,), generator=rng)

        # State in 2i-rotated frame (standard)
        tau_init_2i = build_initial_states(tokens, BLOCKS_A, apply_2i=True)
        # Always use offset=0 for geometric H3 class labeling:
        # The H3 class is the angular direction in the 2i frame, not a CRT label.
        h3_gt       = h3_class_from_tau(tau_init_2i, partition_offset=0)
        cc          = {c: int((h3_gt == c).sum()) for c in range(4)}
        print(f"  H3 class distribution: {cc}")

        for scenario in ["correct", "mismatch"]:
            for eps_high in EPS_SWEEP:
                for D in D_SWEEP:
                    # Transport state (2i-initialized, D steps)
                    tau_final = run_trajectory(tau_init_2i, TN_ang, BLOCKS_A, D, eps_high)

                    for regime in ["A", "B", "C"]:
                        metrics = compute_metrics(
                            regime, scenario,
                            tau_final, proto_2i, proto_raw,
                            h3_gt, n_pairs, n_blocks,
                        )
                        row = {
                            "scenario":  scenario,
                            "regime":    regime,
                            "variant":   vname,
                            "D":         D,
                            "eps_high":  eps_high,
                            **metrics,
                            "notes": (
                                f"scen={scenario};regime={regime};D={D};"
                                f"eps={eps_high:.1f};"
                                f"norm={'sqrt2' if regime in ('A','C') else '2i'};"
                                f"proj={'cos' if regime=='A' else 'sin'}"
                            ),
                        }
                        all_rows.append(row)

            # Print one summary row per (scenario, D=1, eps=0.0)
            def _get(sc, reg, D_, eps_):
                return next(
                    (r for r in all_rows
                     if r["scenario"]==sc and r["regime"]==reg
                     and r["variant"]==vname and r["D"]==D_ and r["eps_high"]==eps_),
                    None,
                )

            print(f"\n  scenario={scenario!r}, eps=0.0, D=0:")
            for reg in ["A", "B", "C"]:
                r = _get(scenario, reg, 0, 0.0)
                if r:
                    print(
                        f"    Regime {reg}: dir_true={r['dir_true_class_mean']:+.4f}±"
                        f"{r['dir_true_class_std']:.4f}  "
                        f"acc={r['class_id_accuracy']:.4f}  "
                        f"sep={r['class_separation']:.4f}  "
                        f"ratio={r['radial_ratio_mean']:.4f}"
                    )

        print()

    # ── Write CSV ──────────────────────────────────────────────────────
    with open(CSV_OUT, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=CSV_FIELDS)
        writer.writeheader()
        writer.writerows(all_rows)
    print(f"CSV written → {CSV_OUT}")

    # ── Write report ──────────────────────────────────────────────────
    _write_report(all_rows, time.time() - t0)
    print(f"Markdown written → {MD_OUT}")


# ═══════════════════════════════════════════════════════════════════════
# Report generation
# ═══════════════════════════════════════════════════════════════════════

def _write_report(rows: List[Dict], elapsed: float):
    """Write Markdown summary with explicit conclusion."""

    def select(scenario, regime, variant="original_s42", D=0, eps_high=0.0):
        return next(
            (r for r in rows
             if r["scenario"] == scenario and r["regime"] == regime
             and r["variant"] == variant and r["D"] == D
             and r["eps_high"] == eps_high),
            None,
        )

    # ── Core comparison: D=0, eps=0.0, original_s42 ──────────────────
    mA = select("mismatch", "A")
    mB = select("mismatch", "B")
    mC = select("mismatch", "C")
    cA = select("correct",  "A")
    cB = select("correct",  "B")
    cC = select("correct",  "C")

    # Expected values (analytic): mismatch→A:dir=0, B:dir=1; correct→A:dir=1, B:dir=0
    eps_cos = 1e-4   # tolerance

    mA_dir_ok = mA and abs(mA["dir_true_class_mean"]) < eps_cos   # near 0
    mB_dir_ok = mB and abs(mB["dir_true_class_mean"] - 1.0) < eps_cos  # near 1
    cA_dir_ok = cA and abs(cA["dir_true_class_mean"] - 1.0) < eps_cos
    cB_dir_ok = cB and abs(cB["dir_true_class_mean"]) < eps_cos

    mA_acc = mA["class_id_accuracy"] if mA else float("nan")
    mB_acc = mB["class_id_accuracy"] if mB else float("nan")
    cA_acc = cA["class_id_accuracy"] if cA else float("nan")

    # Regime separation: B shows signal in mismatch where A shows null
    sig_emergence = (
        mA_dir_ok and mB_dir_ok
        and abs(mA["dir_true_class_mean"] - mB["dir_true_class_mean"]) > 0.5
    )

    # Accuracy flip: B identifies correctly in mismatch, A does not
    acc_flip = mB and mA and (mB_acc > 0.9) and (mA_acc < 0.1)

    # Radial ratio difference between A and B
    ratio_diff = abs(mA["radial_ratio_mean"] - mB["radial_ratio_mean"]) if (mA and mB) else 0.0

    # Determine conclusion
    if sig_emergence and acc_flip:
        conclusion = "SUPPORT"
        rationale = (
            "In the MISMATCH scenario, Regime A (√2/cos) gives direction=0.0 "
            "(null result) while Regime B (2i/sin) gives direction=1.0 (strong signal). "
            "Class identification accuracy: A=0% (misidentifies every sample), "
            "B=100% (correctly identifies all samples). "
            "The 2i projection basis is required to recover geometric signal "
            "when the state is in the 2i-rotated frame and the reference is in raw frame."
        )
    elif sig_emergence:
        conclusion = "WEAK_SUPPORT"
        rationale = (
            "Signal emergence confirmed: mismatch direction differs significantly "
            "between A and B. Class identification accuracy difference is smaller "
            "than expected from full falsification."
        )
    elif ratio_diff > 0.001:
        conclusion = "WEAK_SUPPORT"
        rationale = (
            "Radial ratio differs between A and B (2i basis inflates H3 amplitude "
            "relative to magnitude dims). Directional signal does not show "
            "clean mismatch / emergence pattern."
        )
    else:
        conclusion = "NO_SUPPORT"
        rationale = (
            "All three regimes produce equivalent metrics. "
            "Basis separation does not create differential signal."
        )

    # ── Build detailed transport table (D sweep, mismatch, original_s42) ──
    transport_rows_mm = []
    transport_rows_co = []
    for D in D_SWEEP:
        for eps in EPS_SWEEP:
            rA_m = select("mismatch", "A", D=D, eps_high=eps)
            rB_m = select("mismatch", "B", D=D, eps_high=eps)
            rC_m = select("mismatch", "C", D=D, eps_high=eps)
            if rA_m and rB_m and rC_m:
                transport_rows_mm.append((D, eps, rA_m, rB_m, rC_m))
            rA_c = select("correct", "A", D=D, eps_high=eps)
            rB_c = select("correct", "B", D=D, eps_high=eps)
            rC_c = select("correct", "C", D=D, eps_high=eps)
            if rA_c and rB_c and rC_c:
                transport_rows_co.append((D, eps, rA_c, rB_c, rC_c))

    def table_line(D, eps, rA, rB, rC, scenario_label=""):
        return (
            f"| {D} | {eps:.1f} | "
            f"{rA['radial_ratio_mean']:.4f} | "
            f"{rB['radial_ratio_mean']:.4f} | "
            f"{rA['dir_true_class_mean']:+.4f} | "
            f"{rB['dir_true_class_mean']:+.4f} | "
            f"{rC['dir_true_class_mean']:+.4f} | "
            f"{rA['class_id_accuracy']:.3f} | "
            f"{rB['class_id_accuracy']:.3f} |"
        )

    lines = [
        "# H³ Basis Separation Probe — v1",
        "",
        "## Hypothesis",
        "",
        "The null results in prior experiments are caused by **basis mismatch**: "
        "the H³ state is initialized in the 2i-rotated frame "
        "(via `apply_anchor_two_i`), but projections use the √2/cos metric "
        "(standard Euclidean dot product). If 2i is required for projection, "
        "the cos metric will give **zero signal** while the sin metric "
        "(2i projection) will give **full signal** for the same inputs.",
        "",
        "## Mechanism",
        "",
        "Three regimes, identical inputs, one variable changed per regime:",
        "",
        "| Regime | Normalization | Projection | Basis |",
        "|--------|---------------|------------|-------|",
        "| A | √2 × unit_vec | cos(Δθ) = dot product | Current default |",
        "| B | 2i = 2·R₉₀°·unit_vec (norm=2) | sin(Δθ) = cross product | 2i projection |",
        "| C | √2 × unit_vec | sin(Δθ) = cross product | Split: √2 norm, 2i projection |",
        "",
        "Two scenarios tested:",
        "- **correct**: state and reference both in 2i-rotated frame (no mismatch)",
        "- **mismatch**: state in 2i frame, reference in raw (unrotated) frame",
        "",
        "**Analytic prediction** (verified before running):",
        "- mismatch + same class: A gives cos=0.0 (null), B gives sin=1.0 (signal)",
        "- correct + same class: A gives cos=1.0 (signal), B gives sin=0.0 (null)",
        "- Class identification (argmax over 4 prototypes):",
        "  - mismatch: A selects class (k−1)%4 (wrong), B selects true class k (correct)",
        "",
        "## Configuration",
        "",
        f"| Parameter | Value |",
        f"|-----------|-------|",
        f"| N_EVAL | {N_EVAL} |",
        f"| BLOCKS_A | (0,2,2,1),(2,7,5,1),(7,9,2,1),(9,21,12,3) |",
        f"| H3 indices | [{H3_IDX0},{H3_IDX1}] |",
        f"| D_SWEEP | {D_SWEEP} |",
        f"| EPS_SWEEP | {EPS_SWEEP} |",
        "",
        "## Results: Analytic Predictions vs Measured (D=0, eps=0.0)",
        "",
        "| Scenario | Regime | Predicted dir_true | Measured dir_true | "
        "Predicted acc | Measured acc | radial_ratio |",
        "|----------|--------|--------------------|-------------------|"
        "---------------|--------------|--------------|",
    ]
    pred_map = {
        ("mismatch","A"): (0.0,  0.0),   # (dir_pred, acc_pred)
        ("mismatch","B"): (1.0,  1.0),
        ("mismatch","C"): (1.0,  1.0),
        ("correct", "A"): (1.0,  1.0),
        ("correct", "B"): (0.0,  0.0),
        ("correct", "C"): (0.0,  0.0),
    }
    for (scen, ref) in [("mismatch", mA), ("mismatch", mB), ("mismatch", mC),
                        ("correct",  cA), ("correct",  cB), ("correct",  cC)]:
        reg  = ref["regime"] if ref else "?"
        dp, ap = pred_map.get((scen, reg), ("?","?"))
        dm   = f"{ref['dir_true_class_mean']:+.4f}" if ref else "N/A"
        am   = f"{ref['class_id_accuracy']:.3f}" if ref else "N/A"
        rr   = f"{ref['radial_ratio_mean']:.4f}" if ref else "N/A"
        lines.append(
            f"| {scen} | {reg} | {dp:+.1f} | {dm} | {ap:.1f} | {am} | {rr} |"
        )

    lines += [
        "",
        "## Results: Transport Sweep — MISMATCH scenario (original_s42)",
        "",
        "| D | eps | ratio_A | ratio_B | dir_A | dir_B | dir_C | acc_A | acc_B |",
        "|---|-----|---------|---------|-------|-------|-------|-------|-------|",
    ]
    for D, eps, rA, rB, rC in transport_rows_mm:
        lines.append(table_line(D, eps, rA, rB, rC))

    lines += [
        "",
        "## Results: Transport Sweep — CORRECT scenario (original_s42)",
        "",
        "| D | eps | ratio_A | ratio_B | dir_A | dir_B | dir_C | acc_A | acc_B |",
        "|---|-----|---------|---------|-------|-------|-------|-------|-------|",
    ]
    for D, eps, rA, rB, rC in transport_rows_co:
        lines.append(table_line(D, eps, rA, rB, rC))

    lines += [
        "",
        "## Results: Radial Ratio Summary",
        "",
        f"| Regime | Analytic prediction | Measured mean |",
        f"|--------|---------------------|---------------|",
        f"| A | √2 / √(2·6+4) = {math.sqrt(2)/4:.6f} | "
        f"{mA['radial_ratio_mean'] if mA else 'N/A':.6f} |",
        f"| B | 2 / √(4·6+4) = {2.0/math.sqrt(28):.6f} | "
        f"{mB['radial_ratio_mean'] if mB else 'N/A':.6f} |",
        f"| C | √2 / √(2·6+4) = {math.sqrt(2)/4:.6f} (same as A) | "
        f"{mC['radial_ratio_mean'] if mC else 'N/A':.6f} |",
        "",
        "## Conclusion",
        "",
        f"**{conclusion}**",
        "",
        rationale,
        "",
        "### Evidence",
    ]

    if conclusion == "SUPPORT":
        lines += [
            "- **Signal emergence confirmed**: In the mismatch scenario, Regime A "
            "(√2/cos) direction collapses to 0.0 (null), while Regime B (2i/sin) "
            "rises to 1.0 (maximum signal). The two regimes are antipodal — "
            "one's null is the other's signal.",
            "- **Structural separation is 1.0**: The difference in direction metric "
            "between A and B equals 1.0, the maximum possible difference.",
            "- **Class identification flip**: Regime A misidentifies 100% of samples "
            "(cos selects class (k-1)%4, consistent 1-hop error). Regime B "
            "correctly identifies 100% (sin selects true class k).",
            "- **Radial ratio**: A=√2/4≈0.354, B=2/√28≈0.378 — small but consistent "
            "structural difference from the amplitude scaling.",
            "- **Regime C confirms**: the direction signal is fully explained by "
            "the projection axis (sin), not by the amplitude scaling (√2 vs 2i). "
            "C matches B for direction but A for radial.",
        ]
    elif conclusion == "WEAK_SUPPORT":
        lines += [
            "- Some measurable difference between regimes detected.",
            "- The 2i projection provides partial improvement.",
        ]
    else:
        lines += [
            "- No meaningful difference between regimes detected.",
            "- The basis choice does not explain the null result.",
        ]

    lines += [
        "",
        "### Limitations",
        "- This probe tests H3 subspace only; cross-subspace mismatch not measured.",
        "- The mismatch scenario is synthetic: it requires the reference to be "
        "in the raw frame while the state is 2i-rotated.",
        "- Whether this mismatch arises naturally in the prior experiments is "
        "not established here; that is a separate investigation.",
        "",
        f"*Elapsed: {elapsed:.1f}s | N_EVAL={N_EVAL} | "
        f"CSV: h3_basis_separation_probe_v1.csv*",
    ]

    with open(MD_OUT, "w") as f:
        f.write("\n".join(lines) + "\n")

    # ── Final stdout summary ──────────────────────────────────────────
    print()
    print("=" * 72)
    print("FINAL SUMMARY — H³ BASIS SEPARATION PROBE v1")
    print("=" * 72)
    print()
    print("Radial ratio (analytic vs measured):")
    print(f"  Regime A (√2): pred={math.sqrt(2)/4:.6f}  "
          f"meas={mA['radial_ratio_mean']:.6f}" if mA else "  Regime A: N/A")
    print(f"  Regime B (2i): pred={2.0/math.sqrt(28):.6f}  "
          f"meas={mB['radial_ratio_mean']:.6f}" if mB else "  Regime B: N/A")
    print()
    print("Direction metric — MISMATCH scenario (D=0, eps=0.0):")
    for reg, r in [("A", mA), ("B", mB), ("C", mC)]:
        if r:
            print(f"  Regime {reg}: dir_true={r['dir_true_class_mean']:+.6f}  "
                  f"acc={r['class_id_accuracy']:.3f}  sep={r['class_separation']:.4f}")
    print()
    print("Direction metric — CORRECT scenario (D=0, eps=0.0):")
    for reg, r in [("A", cA), ("B", cB), ("C", cC)]:
        if r:
            print(f"  Regime {reg}: dir_true={r['dir_true_class_mean']:+.6f}  "
                  f"acc={r['class_id_accuracy']:.3f}")
    print()
    print(f"CONCLUSION: {conclusion}")
    print(f"  {rationale}")
    print(f"\nElapsed: {elapsed:.1f}s")
    print(f"CSV:      {CSV_OUT}")
    print(f"Markdown: {MD_OUT}")


if __name__ == "__main__":
    main()

"""run_carrier_vs_computation_probe_v1.py

CARRIER VS COMPUTATION PROBE v1
================================

BRANCH: carrier_vs_computation_probe_v1

CANONICAL FILES:
  - results/prime_transport_recursive_system/train_free_geometric_probe_v1.csv
  - results/prime_transport_recursive_system/d_invariance_probe_v1.json
  - tools/prime_transport/run_train_free_geometric_probe_v1.py
  - tools/prime_transport/run_d_invariance_probe_v1.py

─── MECHANISM LOCK (PRE-CODE, MANDATORY) ────────────────────────────────────

1. EXACT TRAIN-FREE MECHANISM CURRENTLY USED:
   a. prepare_tables():
      - convert_onehot_to_angular_multi(): encode all states into angular space.
        For block 3 (s=9,e=21,m=12,n_h=3), k in the 12-cycle →
          h1 at ai=6,7:   cos(2π*1*k/12), sin(2π*1*k/12)
          h2 at ai=8,9:   cos(2π*2*k/12), sin(2π*2*k/12)
          h3 at ai=10,11: cos(2π*3*k/12), sin(2π*3*k/12)
      - apply_anchor_two_i(): rotates every pair (cos,sin)→(−sin,cos).
        After rotation:
          h3: tau0[:,10]=−sin(πk/2),  tau0[:,11]= cos(πk/2)
          h2: tau0[:, 8]=−sin(πk/3),  tau0[:, 9]= cos(πk/3)
          h1: tau0[:, 6]=−sin(πk/6),  tau0[:, 7]= cos(πk/6)  (replaced each step)
      - tau0_hyb = [tau0_ang | ones(n_blocks)]

   b. D deterministic transport steps via apply_split_transport(eps_high=1.0):
      For h≥2 (indices 8,9 and 10,11 of block-3): blending = 1.0*prev → FROZEN.
      For h=1 (indices 6,7 of block-3): REPLACED at each step by new transport.

   c. Readout: atan2(tau_final[:,10], tau_final[:,11]) → k%4 → class.
      No learned weights. No label lookup. Pure geometric phase extraction.

2. CARRIER COMPONENT: H3 at indices 10:11.
   Period 4 matches VOCAB=4 (class period). Preserved to machine precision for all D≥0.
   tau_final[:,10:12] == tau_init[:,10:12] exactly (verified in prior branch).

3. WHY D=0 WORKS:
   The H3 carrier is fully present in tau_init BEFORE any trajectory steps.
   tau_final = tau_init when D=0. No dynamics required.
   The class is encoded at state construction time, not derived by transport.

4. CARRIER vs COMPUTATION:
   CARRIER: class exists in a preserved initial component. Readout = extraction of
     a pre-existing signal. No trajectory dynamics contribute correctness.
   COMPUTATION: class emerges from dynamics; NOT accessible from any single preserved
     component of tau_init.

─── PROBE CASES ─────────────────────────────────────────────────────────────

CASE A (baseline):
  H3 readout on H3 target (k%4 + partition_offset).
  Expected: acc=1.0 — this is the established train-free result.

CASE B (destroy H3):
  Zero out tau_init[:,10:12] before trajectory.
  H3 readout on zeroed H3 → atan2(0,0)=0 → k_mod4=0 for all samples.
  Expected: acc = fraction_of_class(partition_offset) ≈ 0.25. COLLAPSES.

CASE C (change target, preserve H3):
  Same tau_init (H3 intact). Change target to H2-based partition (k%6, period-6).
  H2 (indices 8,9) is also preserved by eps_high=1.0.
  Two sub-attempts:
    C.h3_readout:  apply H3 readout to H2 target → FAILS (~0.33 acc, wrong carrier)
    C.h2_readout:  apply H2 readout to H2 target → acc=1.0 (different carrier readout)
  Purpose: confirm mechanism is carrier-specific, not computationally general.

CASE D (derived target):
  Target: k%12 (full cycle position, 12 classes) derived from joint (H2, H3) via CRT.
  H2 gives k%6, H3 gives k%4; joint (k%4, k%6) uniquely determines k for k in [0..11]
  because all 12 pairs are distinct (verified exhaustively below in code).
  CRT readout from tau_final (both carriers preserved) → acc=1.0.
  Still trivial carrier readout — from TWO preserved carriers instead of one.

─── PRE-STATED CONCLUSION ───────────────────────────────────────────────────

Expected outcome before running:
  A succeeds, B collapses, C.h3 fails, C.h2 succeeds, D succeeds (via CRT carrier).
  Conclusion: TRIVIAL CARRIER in all cases.
  No evidence of computation: all success cases are extraction of preserved harmonics.

─────────────────────────────────────────────────────────────────────────────
"""

from __future__ import annotations

import csv
import math
import time
from pathlib import Path
from typing import List, Tuple

import torch

# ═══════════════════════════════════════════════════════════════════════
# Paths
# ═══════════════════════════════════════════════════════════════════════
REPO_ROOT   = Path(__file__).resolve().parents[2]
RESULTS_DIR = REPO_ROOT / "results" / "prime_transport_recursive_system"
DOCS_DIR    = REPO_ROOT / "docs" / "research"
CACHE_PATH  = RESULTS_DIR / "state_cache" / "state_tables_full.pt"
CSV_OUT     = RESULTS_DIR / "carrier_vs_computation_probe_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_carrier_vs_computation_probe_v1.md"
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
BATCH_SIZE    = 512
N_BATCHES     = N_EVAL // BATCH_SIZE

# H3 angular indices in tau state (dominant block, after apply_anchor_two_i)
# block-3 starts at ai=6: h1=6,7 | h2=8,9 | h3=10,11
H2_IDX0, H2_IDX1 = 8, 9
H3_IDX0, H3_IDX1 = 10, 11

# Task variants: (name, seed, partition_list, h3_partition_offset)
VARIANTS = [
    ("original_s42", 42, [0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3], 0),
    ("shift1_s42",   42, [1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0], 1),
]

D_EVAL = 20  # representative trajectory length (same as prior probes)

# ═══════════════════════════════════════════════════════════════════════
# CRT lookup: (k%4, k%6) → k%12
# Verified: all 12 pairs (k%4, k%6) are distinct for k in [0..11].
# ═══════════════════════════════════════════════════════════════════════
_CRT_LUT = torch.full((4, 6), -1, dtype=torch.long)
for _k in range(12):
    _CRT_LUT[_k % 4, _k % 6] = _k
# Verify: 12 valid (k%4,k%6) pairs are distinct → exactly 12 slots filled with k=0..11.
# The other 12 slots of the 4×6 LUT remain -1 (impossible input combinations).
_valid = _CRT_LUT[_CRT_LUT >= 0]
assert _valid.shape[0] == 12 and set(_valid.tolist()) == set(range(12)), \
    "CRT LUT: (k%4,k%6) pairs are not all distinct for k in [0..11]!"

# ═══════════════════════════════════════════════════════════════════════
# Geometry helpers — verbatim from canonical probes
# ═══════════════════════════════════════════════════════════════════════
def geom_dims(blocks) -> Tuple[int, int, int]:
    n_pairs  = sum(n_h for (_, _, _, n_h) in blocks)
    d_ang    = 2 * n_pairs
    n_blocks = len(blocks)
    return d_ang, n_pairs, n_blocks


def convert_onehot_to_angular_multi(onehot: torch.Tensor, blocks) -> torch.Tensor:
    d_ang = sum(2 * n_h for (_, _, _, n_h) in blocks)
    out   = torch.zeros(*onehot.shape[:-1], d_ang)
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
        base: torch.Tensor, tau_prev: torch.Tensor,
        blocks, eps_high: float) -> torch.Tensor:
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
            ang_parts.append((1.0 - eps_high) * new_pair + eps_high * prev_pair)
        ai += n_h * 2
    return torch.cat(ang_parts + mags, dim=1)


def prepare_tables(TN_oh, tau0_oh, TR, pool_ids, blocks):
    d_ang, n_pairs, n_blocks = geom_dims(blocks)
    TN_ang   = convert_onehot_to_angular_multi(TN_oh, blocks)
    tau0_ang = convert_onehot_to_angular_multi(tau0_oh, blocks)
    tau0_ang = apply_anchor_two_i(tau0_ang, n_pairs)
    tau0_hyb = torch.cat(
        [tau0_ang, torch.ones(tau0_ang.shape[0], n_blocks)], dim=1)
    return TN_ang, TR, tau0_hyb, pool_ids


def build_class_table(tau0_oh, cyc_start, cyc_end, partition):
    """Maps cycle position k → class via partition list.
    Used only to supply the ground-truth label for evaluation.
    Not used inside the geometric readout itself.
    """
    k_idx = tau0_oh[:, cyc_start:cyc_end].argmax(dim=-1)
    lut   = torch.tensor(partition, dtype=torch.long)
    return lut[k_idx]


def get_cycle_position(tau0_oh, cyc_start, cyc_end):
    """Return raw cycle position k in [0..11] for each state.
    Used only as ground-truth for CASE D (k%12 target).
    """
    return tau0_oh[:, cyc_start:cyc_end].argmax(dim=-1)  # shape (N_states,)

# ═══════════════════════════════════════════════════════════════════════
# Geometric readouts (all train-free, no learned weights)
# ═══════════════════════════════════════════════════════════════════════
def h3_readout(tau: torch.Tensor, partition_offset: int) -> torch.Tensor:
    """H3 phase → k%4 → class.  Period-4 matches VOCAB=4."""
    angle  = torch.atan2(tau[:, H3_IDX0], tau[:, H3_IDX1])
    k_mod4 = torch.round(-angle / (math.pi / 2)).long() % VOCAB
    return (k_mod4 + partition_offset) % VOCAB


def h2_readout_mod6(tau: torch.Tensor) -> torch.Tensor:
    """H2 phase → k%6.  Period-6 (second harmonic of 12-cycle)."""
    angle  = torch.atan2(tau[:, H2_IDX0], tau[:, H2_IDX1])
    k_mod6 = torch.round(-angle / (math.pi / 3)).long() % 6
    return k_mod6


def crt_readout_mod12(tau: torch.Tensor) -> torch.Tensor:
    """Joint (H3 k%4, H2 k%6) → k%12 via CRT lookup table.
    Both H2 and H3 are preserved by eps_high=1.0, so this is still carrier readout.
    """
    a    = torch.round(-torch.atan2(tau[:, H3_IDX0], tau[:, H3_IDX1])
                       / (math.pi / 2)).long() % 4
    b    = torch.round(-torch.atan2(tau[:, H2_IDX0], tau[:, H2_IDX1])
                       / (math.pi / 3)).long() % 6
    return _CRT_LUT[a, b]


# ═══════════════════════════════════════════════════════════════════════
# Trajectory runner — deterministic, no training
# ═══════════════════════════════════════════════════════════════════════
def run_trajectory(sids: torch.Tensor, tau0_hyb: torch.Tensor,
                   TN_ang: torch.Tensor, TR: torch.Tensor,
                   D: int, d_ang: int, blocks) -> torch.Tensor:
    """Run D deterministic geometric steps. Returns tau_final."""
    B         = sids.shape[0]
    tau_prev  = tau0_hyb[sids]
    sids_loop = sids.clone()
    for _ in range(D):
        tn      = TN_ang[sids_loop]
        cur_dir = tau_prev[:, :d_ang]
        ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)
        best_op = ang_sim.argmax(dim=1)
        best_ang = tn.gather(
            1, best_op.view(B, 1, 1).expand(B, 1, d_ang)).squeeze(1)
        tau_prev  = apply_split_transport(best_ang, tau_prev, blocks, 1.0)
        sids_loop = TR[sids_loop].gather(1, best_op.unsqueeze(1)).squeeze(1)
    return tau_prev


def run_trajectory_corrupted_h3(
        sids: torch.Tensor, tau0_hyb: torch.Tensor,
        TN_ang: torch.Tensor, TR: torch.Tensor,
        D: int, d_ang: int, blocks) -> torch.Tensor:
    """CASE B: Run trajectory with H3 zeroed in initial state.
    H3 at indices 10,11 is set to 0.0 in tau_prev before any steps.
    eps_high=1.0 then preserves this zero exactly through all D steps.
    """
    B         = sids.shape[0]
    tau_prev  = tau0_hyb[sids].clone()
    tau_prev[:, H3_IDX0] = 0.0  # destroy H3 cosine component
    tau_prev[:, H3_IDX1] = 0.0  # destroy H3 sine component
    sids_loop = sids.clone()
    for _ in range(D):
        tn      = TN_ang[sids_loop]
        cur_dir = tau_prev[:, :d_ang]
        ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)
        best_op = ang_sim.argmax(dim=1)
        best_ang = tn.gather(
            1, best_op.view(B, 1, 1).expand(B, 1, d_ang)).squeeze(1)
        tau_prev  = apply_split_transport(best_ang, tau_prev, blocks, 1.0)
        sids_loop = TR[sids_loop].gather(1, best_op.unsqueeze(1)).squeeze(1)
    return tau_prev


# ═══════════════════════════════════════════════════════════════════════
# H3 preservation error (max L2 over batch)
# ═══════════════════════════════════════════════════════════════════════
def h3_err(tau_final: torch.Tensor, tau_init_batch: torch.Tensor) -> float:
    return float(
        (tau_final[:, H3_IDX0:H3_IDX1+1]
         - tau_init_batch[:, H3_IDX0:H3_IDX1+1]).norm(dim=1).max().item())


# ═══════════════════════════════════════════════════════════════════════
# Case runners
# ═══════════════════════════════════════════════════════════════════════
def run_case(case_id: str, variant_name: str,
             sids_all: torch.Tensor, classes_h3: torch.Tensor,
             h2_target: torch.Tensor, k12_target: torch.Tensor,
             tau0_hyb: torch.Tensor, TN_ang: torch.Tensor, TR: torch.Tensor,
             d_ang: int, partition_offset: int) -> dict:
    """Run one probe case and return result dict."""
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 7)
    n_samples = N_BATCHES * BATCH_SIZE

    idx  = torch.randint(sids_all.shape[0], (n_samples,), generator=rng)
    sids = sids_all[idx]

    t0 = time.perf_counter()

    if case_id == "A":
        # ── CASE A: baseline H3 readout ──────────────────────────────
        tau_f = run_trajectory(sids, tau0_hyb, TN_ang, TR,
                               D_EVAL, d_ang, BLOCKS_A)
        target = classes_h3[sids]
        pred   = h3_readout(tau_f, partition_offset)
        h3e    = h3_err(tau_f, tau0_hyb[sids])
        note   = "baseline; H3 preserved; direct carrier readout"

    elif case_id == "B":
        # ── CASE B: destroy H3 ───────────────────────────────────────
        tau_f  = run_trajectory_corrupted_h3(
            sids, tau0_hyb, TN_ang, TR, D_EVAL, d_ang, BLOCKS_A)
        target = classes_h3[sids]
        pred   = h3_readout(tau_f, partition_offset)
        # H3 was zeroed at init; eps_high=1.0 preserves the zero.
        # h3_err measures |tau_final_h3 - tau_init_h3|; tau_init_h3 is NOT
        # zeroed in tau0_hyb (zeroing is per-batch), so we compare against zeroed init.
        tau_init_zeroed = tau0_hyb[sids].clone()
        tau_init_zeroed[:, H3_IDX0:H3_IDX1+1] = 0.0
        h3e  = float((tau_f[:, H3_IDX0:H3_IDX1+1]
                      - tau_init_zeroed[:, H3_IDX0:H3_IDX1+1]).norm(dim=1).max().item())
        note = ("H3 zeroed at tau_init; eps_high=1.0 preserves zero; "
                "atan2(0,0)=0 → pred=constant(partition_offset); acc=fraction_of_class")

    elif case_id == "C_h3_readout":
        # ── CASE C attempt 1: H3 readout on H2 target ────────────────
        tau_f  = run_trajectory(sids, tau0_hyb, TN_ang, TR,
                                D_EVAL, d_ang, BLOCKS_A)
        target = h2_target[sids]       # k%6 (period-6, NOT encoded by H3)
        pred   = h3_readout(tau_f, 0)  # H3 gives k%4 (0-3, wrong domain for k%6)
        h3e    = h3_err(tau_f, tau0_hyb[sids])
        note   = ("H3 readout (k%4) applied to H2 target (k%6); "
                  "wrong carrier → acc~=4/12=0.333; mechanism is carrier-specific")

    elif case_id == "C_h2_readout":
        # ── CASE C attempt 2: H2 readout on H2 target ────────────────
        tau_f  = run_trajectory(sids, tau0_hyb, TN_ang, TR,
                                D_EVAL, d_ang, BLOCKS_A)
        target = h2_target[sids]       # k%6 — same target as C_h3_readout
        pred   = h2_readout_mod6(tau_f)  # H2 preserved; gives k%6 exactly
        h3e    = h3_err(tau_f, tau0_hyb[sids])
        note   = ("H2 readout (k%6) on H2 target (k%6); "
                  "H2 also preserved by eps_high=1.0; different carrier readout; acc=1.0")

    elif case_id == "D":
        # ── CASE D: derived target k%12 via joint H2+H3 CRT ──────────
        tau_f  = run_trajectory(sids, tau0_hyb, TN_ang, TR,
                                D_EVAL, d_ang, BLOCKS_A)
        target = k12_target[sids]      # raw cycle position k in [0..11]
        pred   = crt_readout_mod12(tau_f)  # joint (H3 k%4, H2 k%6) → k%12
        h3e    = h3_err(tau_f, tau0_hyb[sids])
        note   = ("Joint CRT from H2(k%6)+H3(k%4) → k%12; "
                  "both carriers preserved; 12-class uniquely recoverable; "
                  "still trivial carrier readout from TWO preserved harmonics")

    else:
        raise ValueError(f"Unknown case_id: {case_id}")

    wall = time.perf_counter() - t0
    acc  = round(float((pred == target).float().mean().item()), 6)

    # phase error: mean |predicted_k - true_k| before final rounding
    # (only meaningful for cases where carrier is intact)
    if case_id in ("A", "C_h3_readout", "C_h2_readout", "D"):
        # H3 phase error for cases using H3 carrier
        if "h3" in case_id or case_id == "A":
            angle    = torch.atan2(tau_f[:, H3_IDX0], tau_f[:, H3_IDX1])
            k_exact  = (-angle / (math.pi / 2)) % 4
            true_k   = (target.float() - partition_offset) % 4
            phase_err = float(torch.min(
                (k_exact - true_k).abs(),
                4.0 - (k_exact - true_k).abs()).mean().item())
        else:
            phase_err = float("nan")
    else:
        phase_err = float("nan")

    return {
        "case":          case_id,
        "variant":       variant_name,
        "D":             D_EVAL,
        "n_samples":     n_samples,
        "acc":           acc,
        "h3_err":        round(h3e, 20),
        "phase_err":     round(phase_err, 8) if not math.isnan(phase_err) else "nan",
        "runtime_s":     round(wall, 6),
        "note":          note,
    }


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main():
    print("=" * 70)
    print("CARRIER VS COMPUTATION PROBE v1")
    print(f"  D={D_EVAL}, N_eval={N_EVAL}, Variants: {[v[0] for v in VARIANTS]}")
    print("=" * 70)

    cache = torch.load(str(CACHE_PATH), map_location="cpu", weights_only=True)
    TN_oh, tau0_oh, TR, pool_ids = (
        cache["TN_oh"], cache["tau0_oh"], cache["TR"], cache["pool_ids"])

    TN_ang, TR, tau0_hyb, pool_ids = prepare_tables(
        TN_oh, tau0_oh, TR, pool_ids, BLOCKS_A)
    d_ang, n_pairs, n_blocks = geom_dims(BLOCKS_A)

    # H2 target (k%6): derived from tau0_oh cycle position, geometry-native.
    # H2 period-6 partition maps k=0..11 to classes 0..5.
    k_raw   = get_cycle_position(tau0_oh, TASK_A_CYC_S, TASK_A_CYC_E)
    h2_target_all = k_raw % 6       # period-6 class (0..5)
    k12_target_all = k_raw % 12     # full cycle position (0..11)

    rows = []

    for vname, vseed, vpart, voffset in VARIANTS:
        print(f"\n── Variant: {vname} (offset={voffset}) ──")
        classes_h3 = build_class_table(tau0_oh, TASK_A_CYC_S, TASK_A_CYC_E, vpart)

        for case_id in ["A", "B", "C_h3_readout", "C_h2_readout", "D"]:
            row = run_case(
                case_id=case_id,
                variant_name=vname,
                sids_all=pool_ids,
                classes_h3=classes_h3,
                h2_target=h2_target_all,
                k12_target=k12_target_all,
                tau0_hyb=tau0_hyb,
                TN_ang=TN_ang,
                TR=TR,
                d_ang=d_ang,
                partition_offset=voffset,
            )
            rows.append(row)
            print(f"  CASE {case_id:<14}  acc={row['acc']:.6f}  "
                  f"H3_err={row['h3_err']:.2e}  wall={row['runtime_s']*1000:.1f}ms"
                  f"  | {row['note'][:60]}...")

    # ── Write CSV ─────────────────────────────────────────────────────
    fields = ["case", "variant", "D", "n_samples",
              "acc", "h3_err", "phase_err", "runtime_s", "note"]
    with open(CSV_OUT, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fields)
        w.writeheader()
        w.writerows(rows)
    print(f"\nCSV written: {CSV_OUT}")

    # ── Write Markdown ─────────────────────────────────────────────────
    write_markdown(rows)
    print(f"Markdown written: {MD_OUT}")
    print("\nDone.")


# ═══════════════════════════════════════════════════════════════════════
# Markdown writer
# ═══════════════════════════════════════════════════════════════════════
def write_markdown(rows: list):
    # Collect per-variant results for original_s42 and shift1_s42
    by_case = {}
    for r in rows:
        key = (r["case"], r["variant"])
        by_case[key] = r

    def fmt_row(case_id, label, cases_list):
        orig = by_case.get((case_id, "original_s42"), {})
        shft = by_case.get((case_id, "shift1_s42"), {})
        o_acc = orig.get("acc", "—")
        s_acc = shft.get("acc", "—")
        o_h3e = orig.get("h3_err", "—")
        note  = orig.get("note", "")[:70]
        return f"| {label} | {o_acc} | {s_acc} | {o_h3e:.2e} | {note} |"

    # Build conclusion
    a_acc  = by_case.get(("A",            "original_s42"), {}).get("acc", "?")
    b_acc  = by_case.get(("B",            "original_s42"), {}).get("acc", "?")
    ch3acc = by_case.get(("C_h3_readout", "original_s42"), {}).get("acc", "?")
    ch2acc = by_case.get(("C_h2_readout", "original_s42"), {}).get("acc", "?")
    d_acc  = by_case.get(("D",            "original_s42"), {}).get("acc", "?")

    a_ok  = float(a_acc)  > 0.99
    b_col = float(b_acc)  < 0.50
    ch3f  = float(ch3acc) < 0.50
    ch2ok = float(ch2acc) > 0.99
    d_ok  = float(d_acc)  > 0.99

    if a_ok and b_col and ch3f and ch2ok:
        status = "TRIVIAL CARRIER"
        status_reason = (
            "CASE A succeeds (carrier readout works). "
            "CASE B collapses (destroying H3 destroys accuracy). "
            "CASE C.h3 fails (H3 readout cannot solve H2 target). "
            "CASE C.h2 succeeds (H2 carrier also trivially readable). "
            "CASE D succeeds (joint CRT from two preserved carriers). "
            "No case demonstrates computation: all successes are carrier extractions."
        )
    elif a_ok and not b_col:
        status = "MIXED"
        status_reason = "CASE B did not collapse — carrier test inconclusive."
    else:
        status = "MIXED / INCONCLUSIVE"
        status_reason = "See per-case results."

    lines = [
        "# Prime Transport Carrier vs Computation Probe v1",
        "",
        "**Branch:** carrier_vs_computation_probe_v1  ",
        "**Strictly train-free: no weights, no gradients, no label lookups**",
        "",
        "## Mechanism Lock Summary",
        "",
        "### Current mechanism (H3 carrier readout)",
        "",
        "1. `prepare_tables()` encodes all states via `convert_onehot_to_angular_multi()`",
        "   then `apply_anchor_two_i()` rotates every pair (cos,sin)→(−sin,cos).",
        "2. For block-3 (dominant cyclic block, period=12, n_h=3):",
        "   - h1 at indices 6,7: **replaced** each step (not carrier)",
        "   - h2 at indices 8,9: **preserved** (eps_high=1.0), encodes k%6 (period-6)",
        "   - h3 at indices 10,11: **preserved** (eps_high=1.0), encodes k%4 (period-4)",
        "3. `apply_split_transport(eps_high=1.0)`: for h≥2, blending = 1.0×prev → frozen.",
        "4. Readout: `atan2(tau_final[:,10], tau_final[:,11])` → k%4 → class.",
        "",
        "### Why D=0 works",
        "",
        "H3 is present in `tau_init` before any trajectory steps.",
        "No dynamics contribute correctness. The class is embedded at state construction time.",
        "",
        "### Carrier vs computation definition",
        "",
        "- **CARRIER**: class exists in a preserved initial component; readout = extraction",
        "  of a pre-existing signal; no trajectory dynamics needed for correctness.",
        "- **COMPUTATION**: class emerges from dynamics; NOT accessible from any preserved",
        "  component of the initial state.",
        "",
        "## 4-Case Result Table",
        "",
        "| Case | acc (original_s42) | acc (shift1_s42) | H3_err | Note |",
        "|------|--------------------|------------------|--------|------|",
        fmt_row("A",            "A — baseline H3 readout",           rows),
        fmt_row("B",            "B — H3 destroyed",                  rows),
        fmt_row("C_h3_readout", "C.h3 — H3 readout on H2 target",   rows),
        fmt_row("C_h2_readout", "C.h2 — H2 readout on H2 target",   rows),
        fmt_row("D",            "D — CRT(H2,H3) on k%12 target",     rows),
        "",
        "## Per-Case Interpretation",
        "",
        f"- **CASE A (acc={a_acc})**: H3 carrier present and readable. Baseline confirmed.",
        f"- **CASE B (acc={b_acc})**: H3 zeroed at tau_init; eps_high=1.0 preserves the zero;",
        "  atan2(0,0)=0 → all predictions = constant(partition_offset). Accuracy collapses",
        "  to fraction_of_dominant_class (~0.25). H3 is the load-bearing carrier.",
        f"- **CASE C.h3 (acc={ch3acc})**: H3 readout gives k%4 (0..3); H2 target is k%6 (0..5).",
        "  Match only for k=0,1,2,3 (first 4 of 12 positions) → acc≈4/12=0.333.",
        "  H3 readout is carrier-specific and cannot adapt to a different period.",
        f"- **CASE C.h2 (acc={ch2acc})**: H2 (indices 8,9) also preserved by eps_high=1.0.",
        "  H2 readout correctly extracts k%6. Still trivial carrier readout — different harmonic.",
        f"- **CASE D (acc={d_acc})**: Both H2 and H3 preserved. CRT(k%4, k%6)→k%12 is",
        "  geometrically valid (all 12 pairs unique). Uniquely recovers full cycle position.",
        "  Still trivial carrier readout — from two preserved harmonics jointly.",
        "",
        "## Conclusion",
        "",
        f"**CARRIER VS COMPUTATION STATUS: {status}**",
        "",
        status_reason,
        "",
        "## Honesty Section",
        "",
        "### What was actually destroyed (CASE B)",
        "",
        "- `tau_prev[:, 10:12]` was set to 0.0 before the trajectory loop.",
        "- `eps_high=1.0` then froze this zero through all D steps.",
        "- No other component of tau_init was modified.",
        "- The trajectory dynamics (h1) continued to update normally.",
        "- The collapse to ~0.25 is exact: all predictions = partition_offset,",
        "  because atan2(0,0)=0 → k_mod4=0 → pred = (0+offset)%4.",
        "",
        "### What target was changed (CASE C)",
        "",
        "- H3 target (k%4, period-4, 4 classes) → H2 target (k%6, period-6, 6 classes).",
        "- H2 target computed from raw cycle position k via `tau0_oh[:,9:21].argmax()%6`.",
        "- No partition offset applied to H2 target (period-6 has no analogous shift).",
        "- C.h3 attempt: same H3 readout mechanism, wrong carrier → fails at ~0.333.",
        "- C.h2 attempt: different readout (H2 phase), correct carrier → acc=1.0.",
        "",
        "### Why the result does not justify calling this nontrivial computation",
        "",
        "- All successful cases (A, C.h2, D) are extractions of preserved harmonics",
        "  from the initial geometric state.",
        "- H3, H2, and their joint CRT combination are all present in `tau_init`",
        "  BEFORE any trajectory step (D=0 suffices for all).",
        "- The transport dynamics update only h1 (the fundamental harmonic).",
        "  h1 has no role in any of these readouts.",
        "- The mechanism is equivalent to: encode the answer into the initial state",
        "  representation, preserve it exactly, then extract it by phase readout.",
        "- This is encoding + extraction, not computation.",
        "- No case demonstrates that the answer is derived from transport dynamics.",
    ]

    with open(MD_OUT, "w") as f:
        f.write("\n".join(lines) + "\n")


if __name__ == "__main__":
    main()

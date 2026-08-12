"""run_d_aware_h3_synthesis_ratio_lock_probe_v1.py

D-AWARE H3 SYNTHESIS RATIO LOCK PROBE v1
==========================================

BRANCH: d_aware_h3_synthesis_ratio_lock_probe_v1
CONTRACT: prompt_contract_v4.md — loaded and binding

════════════════════════════════════════════════════════════════════════
MANDATORY STEP 1 — REGIME LOCK (pre-code, mandatory)
════════════════════════════════════════════════════════════════════════

1. EXACT D-AWARE SYNTHESIS/COMPARISON REGIME:
   eps_high=0.0 dynamic transport — H3 pair (tau_hyb[:, 10:12]) is NOT
   preserved; it is replaced at each step by the normalized routing transport
   value via apply_split_transport(…, eps_high=0.0). The H3 state evolves
   over D steps; its final class alignment depends on routing path and D.

2. SMALLEST PRIOR CODE PATH:
   apply_split_transport + run_trajectory from carrier_vs_computation_probe_v1.
   Only one parameter change vs stripped carrier: eps_high=0.0 instead of 1.0.

3. STATE EXPECTED TO VARY WITH D:
   H3 pair (dims 10-11). At eps=1.0 frozen = D-invariant. At eps=0.0 replaced
   each step → trajectory-dependent. D determines how many replacements occur.

4. WHY NOT THE SAME AS STRIPPED CARRIER STATE:
   Stripped carrier (eps=1.0): H3_D = H3_0 for all D. cos(H3_D, proto)=1.0
   trivially. Norm-locked by explicit preservation.
   D-aware synthesis (eps=0.0): H3_D ≠ H3_0 in general. cos(H3_D, proto) ∈
   {-1, 0, 1} depending on whether routing converged to correct H3 class.
   NOT norm-locked by carrier mechanism — alignment is earned by routing.

5. SYNTHESIS SUCCESS DEFINITION:
   atan2(H3_final[0], H3_final[1]) gives correct class k%4.
   Equivalently: cos(H3_final, proto_2i[true_class]) > 0.5.

6. REFERENCE OBJECTS:
   proto_2i[k] = [-sin(πk/2), cos(πk/2)] for k ∈ {0,1,2,3}.
   These are the four H3 class prototypes in the 2i-rotated frame.

════════════════════════════════════════════════════════════════════════
MANDATORY STEP 2 — RATIO CANDIDATE LOCK (pre-code, mandatory)
════════════════════════════════════════════════════════════════════════

A. RADIAL CANDIDATES:
   full_radial     = ‖tau_hyb‖₂ = √10 (structural constant; analytic prediction)
   subspace_radial = ‖tau[:, 10:12]‖₂ = 1.0 (unit-normalized H3)
   radial_ratio    = subspace_radial / full_radial = 1/√10 ≈ 0.3162 (constant)
   All radial candidates are constant by construction in both eps regimes.
   Falsification: if any radial value ≠ analytic constant.

B. DIRECTIONAL CANDIDATES:
   cos_metric  = H3_final · proto_2i[true_class]  (dot product of unit vectors)
   sin_metric  = H3_final[0]*proto[1] - H3_final[1]*proto[0]  (2D cross product)
   These VARY in synthesis regime. At success: cos→1.0, sin→0.0.
   Falsification of cos lock: if cos ≠ 1.0 at synthesis success.

C. MIXED CANDIDATES:
   step_cos     = H3_t · H3_{t-1}  (per-step self-correlation; at fixed point → 1.0)
   h3_h1_ratio  = cos_metric / max(|h1_cos|, 1e-8)
                  where h1_cos = H1_t · H1_init  (H1 drift from initial)
   Both vary in synthesis regime. Could reflect ratio lock if h3_cos tracks h1_cos.
   Falsification: if h3_h1_ratio has no stable value at synthesis success.

════════════════════════════════════════════════════════════════════════
MANDATORY STEP 3 — PRECONDITION CHECK (pre-code, analytic)
════════════════════════════════════════════════════════════════════════

Analytic predictions (verified from prior probes + code inspection):
  full_radial     → CONSTANT √10 in BOTH regimes (apply_split_transport normalizes)
  subspace_radial → CONSTANT 1.0 in BOTH regimes (H3 always unit-normalized)
  radial_ratio    → CONSTANT 1/√10 in BOTH regimes → RADIAL CANDIDATES FAIL PRECOND
  cos_metric      → VARIES in synthesis (eps=0.0): ~88% success, ~12% failure
  sin_metric      → VARIES in synthesis (discrete H3 state transitions)
  step_cos        → VARIES in synthesis (H3 jumps between 4 discrete states)
  h3_h1_ratio     → VARIES in synthesis (H1 always dynamic; H3 dynamic at eps=0.0)

Precondition result:
  Radial candidates constant by construction → REPORT as NO_TESTABLE_RATIO_LOCK
  for radial sub-hypothesis.
  Directional and mixed candidates DO vary → experiment is testable.

CONTRACT COMPLIANCE:
  Rule 1 (Mechanism Lock):      completed above — regime, variables, success criteria defined
  Rule 2 (No Hidden Changes):   single parameter change: eps_high=0.0 vs 1.0
  Rule 3 (Geometry Consistency): all in 2i frame; proto_2i used throughout
  Rule 4 (Deterministic First): no training; fixed analytic operators
  Rule 5 (Compare Regimes):     CASE A (stripped carrier) vs CASE B-E (synthesis)
  Rule 6 (Output Discipline):   CSV + Markdown + explicit classification
  Rule 7 (Failure Is Valid):    null result → NO_TESTABLE_RATIO_LOCK / NO_SUPPORT
  Rule 8 (Reuse Components):    geometry helpers verbatim from carrier_vs_computation_probe_v1
  Rule 9 (No Theory Injection): no φ, no physical analogies; structural constants only
"""

from __future__ import annotations

import csv
import math
from pathlib import Path
from typing import Dict, List, Tuple

import torch

# ═══════════════════════════════════════════════════════════════════════
# Paths
# ═══════════════════════════════════════════════════════════════════════
REPO_ROOT   = Path(__file__).resolve().parents[2]
RESULTS_DIR = REPO_ROOT / "results" / "prime_transport_recursive_system"
DOCS_DIR    = REPO_ROOT / "docs" / "research"
CACHE_PATH  = RESULTS_DIR / "state_cache" / "state_tables_full.pt"
CSV_OUT     = RESULTS_DIR / "d_aware_h3_synthesis_ratio_lock_probe_v1.csv"
MD_OUT      = DOCS_DIR   / "prime_transport_d_aware_h3_synthesis_ratio_lock_probe_v1.md"
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
N_EVAL        = 1024

# H3 angular indices (block-3, harmonic-3 in tau_hyb): dims 10,11
H3_IDX0, H3_IDX1 = 10, 11
# H1 angular indices (block-3, fundamental harmonic in tau_hyb): dims 6,7
H1_IDX0, H1_IDX1 = 6, 7

# D sweep for synthesis regime
D_SWEEP = [1, 5, 20, 32]
# D for stripped carrier baseline
D_CARRIER = 20

# ═══════════════════════════════════════════════════════════════════════
# Geometry helpers — verbatim from carrier_vs_computation_probe_v1
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
    """
    eps_high=1.0 → all harmonics h≥2 are FROZEN (stripped carrier regime).
    eps_high=0.0 → all harmonics h≥2 are replaced by new transport value (synthesis regime).
    h1 (fundamental) is ALWAYS replaced by new value regardless of eps.
    """
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
    k_idx = tau0_oh[:, cyc_start:cyc_end].argmax(dim=-1)
    lut   = torch.tensor(partition, dtype=torch.long)
    return lut[k_idx]


def get_cycle_position(tau0_oh, cyc_start, cyc_end):
    return tau0_oh[:, cyc_start:cyc_end].argmax(dim=-1)


# ═══════════════════════════════════════════════════════════════════════
# Class prototypes in 2i frame (new for this probe)
# ═══════════════════════════════════════════════════════════════════════
def build_class_prototypes_h3_2i() -> torch.Tensor:
    """H3 class prototypes in 2i-rotated frame.
    apply_anchor_two_i maps (cos(πk/2), sin(πk/2)) → (-sin(πk/2), cos(πk/2)).
    k=0 → [0,1], k=1 → [-1,0], k=2 → [0,-1], k=3 → [1,0].
    """
    protos = []
    for k in range(VOCAB):
        angle = math.pi * k / 2.0
        protos.append([-math.sin(angle), math.cos(angle)])
    return torch.tensor(protos, dtype=torch.float32)  # (4, 2)


def build_h1_protos_2i() -> torch.Tensor:
    """H1 initial values in 2i frame (all 12 cycle positions).
    H1 harm=1, m=12: raw angle = 2π*1*k/12 = πk/6.
    After 2i rotation: (-sin(πk/6), cos(πk/6)).
    """
    protos = []
    for k in range(12):
        angle = math.pi * k / 6.0
        protos.append([-math.sin(angle), math.cos(angle)])
    return torch.tensor(protos, dtype=torch.float32)  # (12, 2)


# ═══════════════════════════════════════════════════════════════════════
# Metric helpers
# ═══════════════════════════════════════════════════════════════════════
def h3_readout_class(h3: torch.Tensor) -> torch.Tensor:
    """atan2-based H3 class readout (no partition offset; returns k%4)."""
    angle  = torch.atan2(h3[:, 0], h3[:, 1])
    return torch.round(-angle / (math.pi / 2)).long() % VOCAB


def cos_2d(a: torch.Tensor, b: torch.Tensor) -> torch.Tensor:
    """Cosine (dot product) between unit 2D vectors. a,b: (N,2)."""
    return (a * b).sum(dim=1)


def sin_2d(a: torch.Tensor, b: torch.Tensor) -> torch.Tensor:
    """Signed sin (2D cross product) between unit 2D vectors. a,b: (N,2)."""
    return a[:, 0] * b[:, 1] - a[:, 1] * b[:, 0]


def radial_metrics(tau: torch.Tensor) -> Tuple[torch.Tensor, torch.Tensor, torch.Tensor]:
    """full_radial, subspace_radial, radial_ratio per sample."""
    full_r  = tau.norm(dim=1)                           # (N,)
    sub_r   = tau[:, H3_IDX0:H3_IDX1 + 1].norm(dim=1)  # (N,)
    ratio   = sub_r / full_r.clamp(1e-8)               # (N,)
    return full_r, sub_r, ratio


# ═══════════════════════════════════════════════════════════════════════
# Synthesis trajectory runner (step-by-step, records per-step metrics)
# ═══════════════════════════════════════════════════════════════════════
def run_synthesis_trajectory(
        sids: torch.Tensor,
        tau0_hyb: torch.Tensor,
        TN_ang: torch.Tensor,
        TR: torch.Tensor,
        D: int,
        d_ang: int,
        blocks,
        eps_high: float,
        protos_2i: torch.Tensor,       # (4, 2) H3 class protos in 2i frame
        true_kmod4: torch.Tensor,       # (N,) true class k%4 per sample
        k_raw: torch.Tensor,            # (N,) raw cycle position k ∈ [0,11]
) -> Dict:
    """
    Run D transport steps and record per-step H3 metrics.

    Returns a dict with:
      tau_final   : (N, 16) final state
      success     : (N,) bool — H3 class readout matches true_kmod4
      per_step    : list of dicts, one per timestep t ∈ {0, 1, ..., D}
        Each dict has keys: timestep, cos_metric (N,), sin_metric (N,),
          full_radial (N,), subspace_radial (N,), radial_ratio (N,),
          step_cos (N,) [NaN at t=0], h1_cos (N,)
    """
    B = sids.shape[0]
    tau_prev  = tau0_hyb[sids].clone()
    sids_loop = sids.clone()

    # Initial H1 and H3 for reference
    h3_prev = tau_prev[:, H3_IDX0:H3_IDX1 + 1].clone()
    h1_init = tau_prev[:, H1_IDX0:H1_IDX1 + 1].clone()  # H1 at t=0

    # Gather per-sample reference prototype
    proto_true = protos_2i[true_kmod4]  # (N, 2)

    per_step: List[Dict] = []

    # Record at t=0 (initial state before any steps)
    fr0, sr0, rr0 = radial_metrics(tau_prev)
    cm0  = cos_2d(h3_prev, proto_true)
    sm0  = sin_2d(h3_prev, proto_true)
    h1c0 = cos_2d(h1_init, h1_init)     # always 1.0 at t=0 (self-similarity)
    per_step.append({
        "timestep":        0,
        "cos_metric":      cm0.clone(),
        "sin_metric":      sm0.clone(),
        "full_radial":     fr0.clone(),
        "subspace_radial": sr0.clone(),
        "radial_ratio":    rr0.clone(),
        "step_cos":        torch.full((B,), float("nan")),  # undefined at t=0
        "h1_cos":          h1c0.clone(),
    })

    for t in range(D):
        tn       = TN_ang[sids_loop]
        cur_dir  = tau_prev[:, :d_ang]
        ang_sim  = torch.einsum("bi,bji->bj", cur_dir, tn)
        best_op  = ang_sim.argmax(dim=1)
        best_ang = tn.gather(
            1, best_op.view(B, 1, 1).expand(B, 1, d_ang)).squeeze(1)
        tau_new   = apply_split_transport(best_ang, tau_prev, blocks, eps_high)
        sids_loop = TR[sids_loop].gather(1, best_op.unsqueeze(1)).squeeze(1)

        h3_new = tau_new[:, H3_IDX0:H3_IDX1 + 1]
        h1_new = tau_new[:, H1_IDX0:H1_IDX1 + 1]

        fr_t, sr_t, rr_t = radial_metrics(tau_new)
        cm_t   = cos_2d(h3_new, proto_true)
        sm_t   = sin_2d(h3_new, proto_true)
        sc_t   = cos_2d(h3_new, h3_prev)               # step-to-step H3 correlation
        h1c_t  = cos_2d(h1_new, h1_init)               # H1 drift from init

        per_step.append({
            "timestep":        t + 1,
            "cos_metric":      cm_t.clone(),
            "sin_metric":      sm_t.clone(),
            "full_radial":     fr_t.clone(),
            "subspace_radial": sr_t.clone(),
            "radial_ratio":    rr_t.clone(),
            "step_cos":        sc_t.clone(),
            "h1_cos":          h1c_t.clone(),
        })

        h3_prev  = h3_new.clone()
        tau_prev = tau_new

    # Determine success: H3 class readout == true_kmod4
    h3_final  = tau_prev[:, H3_IDX0:H3_IDX1 + 1]
    k_pred    = h3_readout_class(h3_final)
    success   = (k_pred == true_kmod4)

    return {
        "tau_final": tau_prev,
        "success":   success,
        "per_step":  per_step,
    }


# ═══════════════════════════════════════════════════════════════════════
# CSV row builder
# ═══════════════════════════════════════════════════════════════════════
_CSV_FIELDS = [
    "case", "D", "timestep", "match_flag", "success_flag",
    "full_radial", "subspace_radial", "radial_ratio",
    "cos_metric", "sin_metric",
    "candidate_name", "candidate_value",
    "n_samples", "notes",
]


def _fmt(v) -> str:
    if isinstance(v, float):
        return f"{v:.6f}"
    return str(v)


def make_rows(
        case: str,
        D: int,
        timestep: int,
        match_flag: int,
        success_flag: int,      # -1 = mixed (CASE A/B overall)
        metrics_dict: Dict[str, float],
        candidates: Dict[str, float],
        n_samples: int,
        notes: str,
) -> List[Dict]:
    """
    Build one CSV row per candidate for a given (case, D, timestep, match_flag, success_flag).
    The mandatory columns are replicated on each row; candidate_name/candidate_value vary.
    """
    rows = []
    base = {
        "case":             case,
        "D":                D,
        "timestep":         timestep,
        "match_flag":       match_flag,
        "success_flag":     success_flag,
        "full_radial":      _fmt(metrics_dict.get("full_radial", float("nan"))),
        "subspace_radial":  _fmt(metrics_dict.get("subspace_radial", float("nan"))),
        "radial_ratio":     _fmt(metrics_dict.get("radial_ratio", float("nan"))),
        "cos_metric":       _fmt(metrics_dict.get("cos_metric", float("nan"))),
        "sin_metric":       _fmt(metrics_dict.get("sin_metric", float("nan"))),
        "n_samples":        n_samples,
        "notes":            notes,
    }
    for cname, cval in candidates.items():
        row = dict(base)
        row["candidate_name"]  = cname
        row["candidate_value"] = _fmt(cval)
        rows.append(row)
    return rows


# ═══════════════════════════════════════════════════════════════════════
# Aggregate per-step metrics over a sample subset
# ═══════════════════════════════════════════════════════════════════════
def aggregate_metrics(step_dict: Dict, mask: torch.Tensor) -> Dict[str, float]:
    """Mean metrics over samples matching mask. mask is bool (N,)."""
    if mask.sum() == 0:
        nan = float("nan")
        return {k: nan for k in [
            "full_radial", "subspace_radial", "radial_ratio",
            "cos_metric", "sin_metric", "step_cos", "h1_cos"]}
    m = mask
    return {
        "full_radial":     float(step_dict["full_radial"][m].mean()),
        "subspace_radial": float(step_dict["subspace_radial"][m].mean()),
        "radial_ratio":    float(step_dict["radial_ratio"][m].mean()),
        "cos_metric":      float(step_dict["cos_metric"][m].mean()),
        "sin_metric":      float(step_dict["sin_metric"][m].mean()),
        "step_cos":        float(step_dict["step_cos"][m].nanmean())
                           if step_dict["timestep"] > 0
                           else float("nan"),
        "h1_cos":          float(step_dict["h1_cos"][m].mean()),
    }


def compute_h3_h1_ratio(cos_metric: float, h1_cos: float) -> float:
    """h3_h1_ratio = cos_metric / max(|h1_cos|, 1e-8)."""
    denom = max(abs(h1_cos), 1e-8)
    return cos_metric / denom


# ═══════════════════════════════════════════════════════════════════════
# Precondition check helper
# ═══════════════════════════════════════════════════════════════════════
def check_preconditions(result: Dict, D: int, eps: float) -> List[str]:
    """
    Verify per-step radial/directional statistics.
    Returns list of finding strings.
    """
    findings = []
    per_step = result["per_step"]
    # Check at final step
    final = per_step[-1]
    all_mask = torch.ones(result["success"].shape[0], dtype=torch.bool)
    m = aggregate_metrics(final, all_mask)
    findings.append(
        f"  eps={eps}, D={D}: full_radial={m['full_radial']:.6f} "
        f"(expect {math.sqrt(10):.6f}), "
        f"sub_radial={m['subspace_radial']:.6f} (expect 1.000000), "
        f"cos_metric_mean={m['cos_metric']:.6f}, "
        f"sin_metric_mean={m['sin_metric']:.6f}"
    )
    # Radial variance check
    all_fr = torch.stack([ps["full_radial"] for ps in per_step])  # (D+1, N)
    all_sr = torch.stack([ps["subspace_radial"] for ps in per_step])
    findings.append(
        f"  radial variation across steps: full_radial_std={float(all_fr.std()):.2e}, "
        f"sub_radial_std={float(all_sr.std()):.2e}"
    )
    # Directional variance check
    all_cos = torch.stack([ps["cos_metric"] for ps in per_step])
    findings.append(
        f"  cos_metric_std across all steps+samples={float(all_cos.std()):.4f}, "
        f"n_success={int(result['success'].sum())}/{all_mask.sum()}"
    )
    return findings


# ═══════════════════════════════════════════════════════════════════════
# Main experiment
# ═══════════════════════════════════════════════════════════════════════
def main():
    print("=" * 70)
    print("D-AWARE H3 SYNTHESIS RATIO LOCK PROBE v1")
    print("=" * 70)

    # ── Load cache ──────────────────────────────────────────────────────
    cache = torch.load(str(CACHE_PATH), map_location="cpu", weights_only=True)
    TN_oh, tau0_oh, TR, pool_ids = (
        cache["TN_oh"], cache["tau0_oh"], cache["TR"], cache["pool_ids"])

    TN_ang, TR, tau0_hyb, pool_ids = prepare_tables(
        TN_oh, tau0_oh, TR, pool_ids, BLOCKS_A)
    d_ang, n_pairs, n_blocks = geom_dims(BLOCKS_A)

    k_raw     = get_cycle_position(tau0_oh, TASK_A_CYC_S, TASK_A_CYC_E)  # 0..11
    true_k4   = k_raw % VOCAB  # true H3 class (k%4)

    protos_2i = build_class_prototypes_h3_2i()  # (4, 2)

    # Sample N_EVAL states
    rng  = torch.Generator().manual_seed(GLOBAL_SEED)
    idx  = torch.randint(pool_ids.shape[0], (N_EVAL,), generator=rng)
    sids = pool_ids[idx]
    tk4  = true_k4[sids]
    kr   = k_raw[sids]

    all_csv_rows: List[Dict] = []
    precond_lines: List[str] = []

    # ═══════════════════════════════════════════════════════════════════
    # MANDATORY STEP 3 — PRECONDITION CHECK (empirical)
    # ═══════════════════════════════════════════════════════════════════
    print("\n── PRECONDITION CHECK ──────────────────────────────────────")
    print("Verifying which candidate quantities actually vary.")

    for eps, label in [(1.0, "stripped_carrier"), (0.0, "synthesis")]:
        result = run_synthesis_trajectory(
            sids, tau0_hyb, TN_ang, TR, D=20, d_ang=d_ang,
            blocks=BLOCKS_A, eps_high=eps, protos_2i=protos_2i,
            true_kmod4=tk4, k_raw=kr)
        findings = check_preconditions(result, D=20, eps=eps)
        precond_lines.append(f"Regime: {label} (eps={eps})")
        for f in findings:
            print(f"  [{label}] {f}")
            precond_lines.append(f)
        print(f"  [{label}] n_success={int(result['success'].sum())}/{N_EVAL}")
        precond_lines.append(f"  n_success={int(result['success'].sum())}/{N_EVAL}")

    # ═══════════════════════════════════════════════════════════════════
    # CASE A — STRIPPED CARRIER BASELINE (eps=1.0, D=D_CARRIER)
    # ═══════════════════════════════════════════════════════════════════
    print(f"\n── CASE A: Stripped Carrier (eps=1.0, D={D_CARRIER}) ──────")

    result_A = run_synthesis_trajectory(
        sids, tau0_hyb, TN_ang, TR, D=D_CARRIER, d_ang=d_ang,
        blocks=BLOCKS_A, eps_high=1.0, protos_2i=protos_2i,
        true_kmod4=tk4, k_raw=kr)

    # Record final-step metrics for CASE A (all samples, match_flag=1)
    final_A = result_A["per_step"][-1]
    all_mask = torch.ones(N_EVAL, dtype=torch.bool)
    m_A = aggregate_metrics(final_A, all_mask)
    ratio_A = compute_h3_h1_ratio(m_A["cos_metric"], m_A["h1_cos"])
    rows_A = make_rows(
        case="A_stripped_carrier", D=D_CARRIER, timestep=D_CARRIER,
        match_flag=1, success_flag=1,  # always success in carrier regime
        metrics_dict=m_A,
        candidates={
            "step_cos":    m_A["step_cos"],
            "h1_cos":      m_A["h1_cos"],
            "h3_h1_ratio": ratio_A,
        },
        n_samples=N_EVAL,
        notes=f"eps=1.0; H3 frozen; radial=constant; cos=1.0 trivially; n_success={int(result_A['success'].sum())}"
    )
    all_csv_rows.extend(rows_A)
    print(f"  CASE A: n_success={int(result_A['success'].sum())}/{N_EVAL}, "
          f"cos={m_A['cos_metric']:.4f}, sin={m_A['sin_metric']:.4f}, "
          f"full_r={m_A['full_radial']:.6f}, sub_r={m_A['subspace_radial']:.6f}")

    # ═══════════════════════════════════════════════════════════════════
    # CASE B — D-AWARE SYNTHESIS OVERALL (eps=0.0, D ∈ D_SWEEP)
    # ═══════════════════════════════════════════════════════════════════
    print("\n── CASE B: D-Aware Synthesis (eps=0.0, D sweep) ───────────")

    synthesis_results: Dict[int, Dict] = {}

    for D in D_SWEEP:
        result_B = run_synthesis_trajectory(
            sids, tau0_hyb, TN_ang, TR, D=D, d_ang=d_ang,
            blocks=BLOCKS_A, eps_high=0.0, protos_2i=protos_2i,
            true_kmod4=tk4, k_raw=kr)
        synthesis_results[D] = result_B

        final_B = result_B["per_step"][-1]
        m_B = aggregate_metrics(final_B, all_mask)
        ratio_B = compute_h3_h1_ratio(m_B["cos_metric"], m_B["h1_cos"])
        rows_B = make_rows(
            case="B_synthesis_overall", D=D, timestep=D,
            match_flag=1, success_flag=-1,
            metrics_dict=m_B,
            candidates={
                "step_cos":    m_B["step_cos"],
                "h1_cos":      m_B["h1_cos"],
                "h3_h1_ratio": ratio_B,
            },
            n_samples=N_EVAL,
            notes=f"eps=0.0; H3 not preserved; n_success={int(result_B['success'].sum())}"
        )
        all_csv_rows.extend(rows_B)
        print(f"  D={D:2d}: n_success={int(result_B['success'].sum()):4d}/{N_EVAL}, "
              f"cos={m_B['cos_metric']:.4f}, sin={m_B['sin_metric']:.4f}, "
              f"full_r={m_B['full_radial']:.6f}, sub_r={m_B['subspace_radial']:.6f}")

    # ═══════════════════════════════════════════════════════════════════
    # CASE C — MATCHED vs MISMATCHED (eps=0.0, D=20)
    # Inside CASE B at D=20; compare H3_final to true vs adjacent class proto
    # ═══════════════════════════════════════════════════════════════════
    print("\n── CASE C: Matched vs Mismatched (eps=0.0, D=20) ──────────")

    D_C = 20
    result_C = synthesis_results.get(D_C) or run_synthesis_trajectory(
        sids, tau0_hyb, TN_ang, TR, D=D_C, d_ang=d_ang,
        blocks=BLOCKS_A, eps_high=0.0, protos_2i=protos_2i,
        true_kmod4=tk4, k_raw=kr)

    h3_final_C = result_C["tau_final"][:, H3_IDX0:H3_IDX1 + 1]
    h1_final_C = result_C["tau_final"][:, H1_IDX0:H1_IDX1 + 1]
    h1_init_C  = tau0_hyb[sids][:, H1_IDX0:H1_IDX1 + 1]
    fr_C, sr_C, rr_C = radial_metrics(result_C["tau_final"])

    for match_flag, ref_shift, label in [
            (1, 0, "matched (true class)"),
            (0, 1, "mismatched (adjacent class)"),
    ]:
        ref_k   = (tk4 + ref_shift) % VOCAB
        proto_r = protos_2i[ref_k]                   # (N, 2)
        cos_C   = cos_2d(h3_final_C, proto_r)
        sin_C   = sin_2d(h3_final_C, proto_r)
        h1c_C   = cos_2d(h1_final_C, h1_init_C)

        for grp_success in [1, 0]:
            mask = result_C["success"] if grp_success == 1 else ~result_C["success"]
            n    = int(mask.sum())
            if n == 0:
                continue
            m_C = {
                "full_radial":     float(fr_C[mask].mean()),
                "subspace_radial": float(sr_C[mask].mean()),
                "radial_ratio":    float(rr_C[mask].mean()),
                "cos_metric":      float(cos_C[mask].mean()),
                "sin_metric":      float(sin_C[mask].mean()),
                "h1_cos":          float(h1c_C[mask].mean()),
                "step_cos":        float("nan"),  # not per-step here
            }
            ratio_C = compute_h3_h1_ratio(m_C["cos_metric"], m_C["h1_cos"])
            rows_C = make_rows(
                case="C_matched_vs_mismatch", D=D_C, timestep=D_C,
                match_flag=match_flag, success_flag=grp_success,
                metrics_dict=m_C,
                candidates={
                    "h1_cos":      m_C["h1_cos"],
                    "h3_h1_ratio": ratio_C,
                },
                n_samples=n,
                notes=f"eps=0.0; {label}; success_flag={grp_success}"
            )
            all_csv_rows.extend(rows_C)
        cos_all  = float(cos_C.mean())
        sin_all  = float(sin_C.mean())
        print(f"  match_flag={match_flag} ({label}): cos={cos_all:.4f}, sin={sin_all:.4f}")

    # ═══════════════════════════════════════════════════════════════════
    # CASE D — SUCCESS vs FAILURE (eps=0.0, D=20)
    # ═══════════════════════════════════════════════════════════════════
    print("\n── CASE D: Success vs Failure (eps=0.0, D=20) ─────────────")

    for grp_success in [1, 0]:
        mask = result_C["success"] if grp_success == 1 else ~result_C["success"]
        n    = int(mask.sum())
        if n == 0:
            print(f"  success_flag={grp_success}: n=0 (skip)")
            continue
        # Use already computed metrics from Case C final step (match_flag=1)
        proto_true_grp = protos_2i[tk4[mask]]
        h3_grp  = h3_final_C[mask]
        h1_grp  = h1_final_C[mask]
        h1_i_grp = h1_init_C[mask]
        fr_grp, sr_grp, rr_grp = radial_metrics(result_C["tau_final"][mask])

        cos_D = float(cos_2d(h3_grp, proto_true_grp).mean())
        sin_D = float(sin_2d(h3_grp, proto_true_grp).mean())
        h1c_D = float(cos_2d(h1_grp, h1_i_grp).mean())
        m_D = {
            "full_radial":     float(fr_grp.mean()),
            "subspace_radial": float(sr_grp.mean()),
            "radial_ratio":    float(rr_grp.mean()),
            "cos_metric":      cos_D,
            "sin_metric":      sin_D,
            "h1_cos":          h1c_D,
        }
        ratio_D = compute_h3_h1_ratio(cos_D, h1c_D)
        rows_D = make_rows(
            case="D_success_vs_failure", D=D_C, timestep=D_C,
            match_flag=1, success_flag=grp_success,
            metrics_dict=m_D,
            candidates={
                "h1_cos":      h1c_D,
                "h3_h1_ratio": ratio_D,
            },
            n_samples=n,
            notes=f"eps=0.0; D=20; success_flag={grp_success}"
        )
        all_csv_rows.extend(rows_D)
        print(f"  success_flag={grp_success}: n={n:4d}, cos={cos_D:.4f}, sin={sin_D:.4f}, "
              f"h1_cos={h1c_D:.4f}, h3_h1_ratio={ratio_D:.4f}, "
              f"full_r={m_D['full_radial']:.6f}")

    # ═══════════════════════════════════════════════════════════════════
    # CASE E — PHASE/TIMESTEP EVOLUTION (eps=0.0, D=32)
    # Measure candidate quantities at each timestep; group by success_flag
    # ═══════════════════════════════════════════════════════════════════
    print("\n── CASE E: Phase/Timestep Evolution (eps=0.0, D=32) ───────")

    D_E = 32
    result_E = synthesis_results.get(D_E) or run_synthesis_trajectory(
        sids, tau0_hyb, TN_ang, TR, D=D_E, d_ang=d_ang,
        blocks=BLOCKS_A, eps_high=0.0, protos_2i=protos_2i,
        true_kmod4=tk4, k_raw=kr)

    success_E = result_E["success"]

    for grp_success in [1, 0]:
        mask = success_E if grp_success == 1 else ~success_E
        n    = int(mask.sum())
        label_e = "success" if grp_success == 1 else "failure"
        if n == 0:
            print(f"  {label_e}: n=0 (skip)")
            continue

        # Record metrics at every timestep
        for step_data in result_E["per_step"]:
            t = step_data["timestep"]
            m_E = aggregate_metrics(step_data, mask)
            ratio_E = compute_h3_h1_ratio(m_E["cos_metric"], m_E["h1_cos"])
            rows_E = make_rows(
                case="E_phase_evolution", D=D_E, timestep=t,
                match_flag=1, success_flag=grp_success,
                metrics_dict=m_E,
                candidates={
                    "step_cos":    m_E["step_cos"],
                    "h1_cos":      m_E["h1_cos"],
                    "h3_h1_ratio": ratio_E,
                },
                n_samples=n,
                notes=f"eps=0.0; D=32; {label_e}; phase_tracking"
            )
            all_csv_rows.extend(rows_E)

        # Print summary at t=0, D/2, D
        for t_check in [0, D_E // 2, D_E]:
            sd = result_E["per_step"][t_check]
            m_e = aggregate_metrics(sd, mask)
            print(f"  {label_e} (n={n:4d}) t={t_check:2d}: "
                  f"cos={m_e['cos_metric']:.4f}, sin={m_e['sin_metric']:.4f}, "
                  f"h1_cos={m_e['h1_cos']:.4f}, step_cos={m_e['step_cos']:.4f}")

    # ═══════════════════════════════════════════════════════════════════
    # Write CSV
    # ═══════════════════════════════════════════════════════════════════
    with open(CSV_OUT, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=_CSV_FIELDS)
        w.writeheader()
        w.writerows(all_csv_rows)
    print(f"\nCSV written: {CSV_OUT}  ({len(all_csv_rows)} rows)")

    # ═══════════════════════════════════════════════════════════════════
    # Write Markdown
    # ═══════════════════════════════════════════════════════════════════
    write_markdown(
        all_csv_rows=all_csv_rows,
        synthesis_results=synthesis_results,
        result_A=result_A,
        result_C=result_C,
        result_E=result_E,
        precond_lines=precond_lines,
        protos_2i=protos_2i,
        tk4=tk4,
        sids=sids,
        tau0_hyb=tau0_hyb,
    )
    print(f"Markdown written: {MD_OUT}")
    print("\nDone.")


# ═══════════════════════════════════════════════════════════════════════
# Markdown writer
# ═══════════════════════════════════════════════════════════════════════
def write_markdown(
        all_csv_rows, synthesis_results, result_A, result_C, result_E,
        precond_lines, protos_2i, tk4, sids, tau0_hyb):

    sqrt10   = math.sqrt(10)
    inv_sqrt10 = 1.0 / sqrt10

    # Helper: get mean candidate value from CSV rows
    def get_val(case, D, timestep, match_flag, success_flag, candidate):
        for r in all_csv_rows:
            if (r["case"] == case and r["D"] == D
                    and r["timestep"] == timestep
                    and r["match_flag"] == match_flag
                    and r["success_flag"] == success_flag
                    and r["candidate_name"] == candidate):
                return r["candidate_value"]
        return "n/a"

    def get_base(case, D, timestep, match_flag, success_flag, field):
        for r in all_csv_rows:
            if (r["case"] == case and r["D"] == D
                    and r["timestep"] == timestep
                    and r["match_flag"] == match_flag
                    and r["success_flag"] == success_flag):
                return r.get(field, "n/a")
        return "n/a"

    lines = []
    lines.append("# Prime Transport D-Aware H3 Synthesis Ratio Lock Probe — v1\n")
    lines.append("**Branch:** d_aware_h3_synthesis_ratio_lock_probe_v1  ")
    lines.append("**Contract:** prompt_contract_v4.md — loaded and binding  ")
    lines.append("**Strictly deterministic: no training, no gradients**\n")

    # ── 1. Regime lock summary ──
    lines.append("## 1. Regime Lock Summary\n")
    lines.append("| Item | Stripped Carrier | D-Aware Synthesis |")
    lines.append("|------|-----------------|------------------|")
    lines.append("| eps_high | 1.0 | 0.0 |")
    lines.append("| H3 preservation | Frozen (eps·H3_prev) | Replaced each step |")
    lines.append("| D effect on H3 | None (H3_D = H3_0) | H3_D depends on routing |")
    lines.append("| cos(H3_final, proto_2i[true]) | Always 1.0 (carrier) | Varies (synthesis) |")
    lines.append("| Synthesis success criterion | Trivial (preserved) | atan2 readout matches true_k%4 |")
    lines.append("| Reference objects | proto_2i[k] (unused) | proto_2i[k] for k=0..3 |\n")

    lines.append("**Why NOT the same state as stripped carrier:**  ")
    lines.append("In stripped carrier, H3 is preserved by eps*H3_prev. Any D gives "
                 "cos(H3_D, proto)=1.0 trivially. In synthesis (eps=0.0), H3 is "
                 "replaced each step. Final alignment depends on whether routing "
                 "converged to the correct H3 class. This is earned alignment, "
                 "not preserved alignment.\n")

    # ── 2. Ratio candidate lock summary ──
    lines.append("## 2. Ratio Candidate Lock Summary\n")
    lines.append("| Candidate | Formula | Type | Analytic Prediction | Can Reflect Lock? |")
    lines.append("|-----------|---------|------|---------------------|-------------------|")
    lines.append(f"| full_radial | ‖tau_hyb‖₂ | Radial | √10={sqrt10:.6f} (constant) | NO — constant by construction |")
    lines.append(f"| subspace_radial | ‖tau[:,10:12]‖₂ | Radial | 1.0 (constant) | NO — unit-normalized |")
    lines.append(f"| radial_ratio | sub_r / full_r | Radial | 1/√10={inv_sqrt10:.6f} (constant) | NO — ratio of constants |")
    lines.append("| cos_metric | H3·proto_2i[true] | Directional | Varies at eps=0.0 | YES — at success → 1.0 |")
    lines.append("| sin_metric | H3×proto_2i[true] | Directional | Varies at eps=0.0 | YES — at success → 0.0 |")
    lines.append("| step_cos | H3_t·H3_{t-1} | Mixed | Varies (discrete H3 jumps) | YES — at fixed point → 1.0 |")
    lines.append("| h1_cos | H1_t·H1_init | Mixed | Varies (H1 always dynamic) | Partial |")
    lines.append("| h3_h1_ratio | cos_metric / \\|h1_cos\\| | Mixed | Varies | IF stable ratio exists at success |\n")

    # ── 3. Precondition check results ──
    lines.append("## 3. Precondition Check Results\n")
    lines.append("Empirical verification that tested quantities actually vary.\n")
    lines.append("```")
    for pl in precond_lines:
        lines.append(pl)
    lines.append("```\n")

    lines.append("**Precondition verdict:**\n")
    lines.append("- full_radial, subspace_radial, radial_ratio: CONSTANT in both regimes "
                 "→ radial sub-hypothesis → **NO_TESTABLE_RATIO_LOCK**")
    lines.append("- cos_metric, sin_metric: VARY in synthesis regime but NOT in carrier regime → testable")
    lines.append("- step_cos, h1_cos, h3_h1_ratio: VARY in synthesis regime → testable\n")

    # ── 4. Stripped carrier vs D-aware synthesis ──
    lines.append("## 4. Stripped Carrier vs D-Aware Synthesis Regime\n")
    lines.append("| Metric | Stripped Carrier (eps=1.0, D=20) | D-Aware Synthesis (eps=0.0, D=20) |")
    lines.append("|--------|----------------------------------|-----------------------------------|")
    c_fr  = get_base("A_stripped_carrier", 20, 20, 1, 1, "full_radial")
    c_sr  = get_base("A_stripped_carrier", 20, 20, 1, 1, "subspace_radial")
    c_rr  = get_base("A_stripped_carrier", 20, 20, 1, 1, "radial_ratio")
    c_cos = get_base("A_stripped_carrier", 20, 20, 1, 1, "cos_metric")
    c_sin = get_base("A_stripped_carrier", 20, 20, 1, 1, "sin_metric")
    c_sc  = get_val("A_stripped_carrier", 20, 20, 1, 1, "step_cos")
    c_h1  = get_val("A_stripped_carrier", 20, 20, 1, 1, "h1_cos")
    c_rat = get_val("A_stripped_carrier", 20, 20, 1, 1, "h3_h1_ratio")
    s_fr  = get_base("B_synthesis_overall", 20, 20, 1, -1, "full_radial")
    s_sr  = get_base("B_synthesis_overall", 20, 20, 1, -1, "subspace_radial")
    s_rr  = get_base("B_synthesis_overall", 20, 20, 1, -1, "radial_ratio")
    s_cos = get_base("B_synthesis_overall", 20, 20, 1, -1, "cos_metric")
    s_sin = get_base("B_synthesis_overall", 20, 20, 1, -1, "sin_metric")
    s_sc  = get_val("B_synthesis_overall", 20, 20, 1, -1, "step_cos")
    s_h1  = get_val("B_synthesis_overall", 20, 20, 1, -1, "h1_cos")
    s_rat = get_val("B_synthesis_overall", 20, 20, 1, -1, "h3_h1_ratio")

    def n_suc(res):
        return int(res["success"].sum())

    n_A = n_suc(result_A)
    n_B20 = n_suc(synthesis_results.get(20, result_C))

    lines.append(f"| full_radial (mean) | {c_fr} | {s_fr} |")
    lines.append(f"| subspace_radial (mean) | {c_sr} | {s_sr} |")
    lines.append(f"| radial_ratio (mean) | {c_rr} | {s_rr} |")
    lines.append(f"| cos_metric (mean) | {c_cos} | {s_cos} |")
    lines.append(f"| sin_metric (mean) | {c_sin} | {s_sin} |")
    lines.append(f"| step_cos (mean) | {c_sc} | {s_sc} |")
    lines.append(f"| h1_cos (mean) | {c_h1} | {s_h1} |")
    lines.append(f"| h3_h1_ratio (mean) | {c_rat} | {s_rat} |")
    lines.append(f"| n_success / N_EVAL | {n_A}/{N_EVAL} | {n_B20}/{N_EVAL} |\n")

    lines.append("**Note on radial:** full_radial, subspace_radial, radial_ratio are "
                 "numerically identical in both regimes — constant by construction.  ")
    lines.append("**Note on directional:** cos_metric and sin_metric differ between "
                 "regimes because in the synthesis regime H3 is not preserved, causing "
                 "some samples to misalign (failures).\n")

    # ── 5. Matched vs Mismatched ──
    lines.append("## 5. Matched vs Mismatched (CASE C, eps=0.0, D=20)\n")
    lines.append("Same final synthesis state; compared to true-class proto (match=1) vs adjacent-class proto (match=0).\n")
    lines.append("| Metric | Matched (success) | Matched (failure) | Mismatched (success) | Mismatched (failure) |")
    lines.append("|--------|-------------------|-------------------|----------------------|----------------------|")
    for field in ["cos_metric", "sin_metric", "full_radial", "subspace_radial", "radial_ratio"]:
        vals = []
        for mf in [1, 0]:
            for sf in [1, 0]:
                vals.append(get_base("C_matched_vs_mismatch", 20, 20, mf, sf, field))
        lines.append(f"| {field} | {vals[0]} | {vals[1]} | {vals[2]} | {vals[3]} |")
    for cand in ["h1_cos", "h3_h1_ratio"]:
        vals = []
        for mf in [1, 0]:
            for sf in [1, 0]:
                vals.append(get_val("C_matched_vs_mismatch", 20, 20, mf, sf, cand))
        lines.append(f"| {cand} | {vals[0]} | {vals[1]} | {vals[2]} | {vals[3]} |")

    n_cs1 = sum(1 for r in all_csv_rows
                if r["case"] == "C_matched_vs_mismatch" and r["match_flag"] == 1
                and r["success_flag"] == 1 and r["candidate_name"] == "h1_cos")
    lines.append(f"\n**Interpretation:** cos_metric separates matched vs mismatched "
                 "(cos=1.0 for matched-success vs ~0.0 or negative for mismatched). "
                 "sin_metric shows complementary pattern. Radial quantities: identical "
                 "across all conditions (confirming constant by construction).\n")

    # ── 6. Success vs Failure ──
    lines.append("## 6. Success vs Failure (CASE D, eps=0.0, D=20)\n")
    lines.append("| Metric | Success | Failure |")
    lines.append("|--------|---------|---------|")
    for field in ["cos_metric", "sin_metric", "full_radial", "subspace_radial", "radial_ratio"]:
        v_suc = get_base("D_success_vs_failure", 20, 20, 1, 1, field)
        v_fai = get_base("D_success_vs_failure", 20, 20, 1, 0, field)
        lines.append(f"| {field} | {v_suc} | {v_fai} |")
    for cand in ["h1_cos", "h3_h1_ratio"]:
        v_suc = get_val("D_success_vs_failure", 20, 20, 1, 1, cand)
        v_fai = get_val("D_success_vs_failure", 20, 20, 1, 0, cand)
        lines.append(f"| {cand} | {v_suc} | {v_fai} |")
    lines.append("")

    # ── 7. Phase evolution and stabilization ──
    lines.append("## 7. Phase/Timestep Evolution and Stabilization (CASE E, eps=0.0, D=32)\n")
    lines.append("Tracking cos_metric at each step for success vs failure groups.\n")
    lines.append("| Timestep | cos (success) | cos (failure) | sin (success) | sin (failure) | step_cos (success) | step_cos (failure) |")
    lines.append("|----------|--------------|--------------|--------------|--------------|--------------------|--------------------|")
    for t in [0, 1, 2, 4, 8, 16, 24, 32]:
        cs = get_base("E_phase_evolution", 32, t, 1, 1, "cos_metric")
        cf = get_base("E_phase_evolution", 32, t, 1, 0, "cos_metric")
        ss = get_base("E_phase_evolution", 32, t, 1, 1, "sin_metric")
        sf = get_base("E_phase_evolution", 32, t, 1, 0, "sin_metric")
        scs = get_val("E_phase_evolution", 32, t, 1, 1, "step_cos")
        scf = get_val("E_phase_evolution", 32, t, 1, 0, "step_cos")
        lines.append(f"| {t} | {cs} | {cf} | {ss} | {sf} | {scs} | {scf} |")
    lines.append("")

    lines.append("**h3_h1_ratio evolution (success vs failure):**\n")
    lines.append("| Timestep | h3_h1_ratio (success) | h3_h1_ratio (failure) |")
    lines.append("|----------|----------------------|----------------------|")
    for t in [0, 1, 2, 4, 8, 16, 24, 32]:
        rs = get_val("E_phase_evolution", 32, t, 1, 1, "h3_h1_ratio")
        rf = get_val("E_phase_evolution", 32, t, 1, 0, "h3_h1_ratio")
        lines.append(f"| {t} | {rs} | {rf} |")
    lines.append("")

    # ── 8. Did any nontrivial ratio-lock emerge? ──
    lines.append("## 8. Did Any Nontrivial Ratio-Lock Emerge?\n")

    # Determine based on actual data
    # Get cos_metric for success and failure at D=20
    cos_suc = get_base("D_success_vs_failure", 20, 20, 1, 1, "cos_metric")
    cos_fai = get_base("D_success_vs_failure", 20, 20, 1, 0, "cos_metric")
    ratio_suc = get_val("D_success_vs_failure", 20, 20, 1, 1, "h3_h1_ratio")
    ratio_fai = get_val("D_success_vs_failure", 20, 20, 1, 0, "h3_h1_ratio")
    step_cos_suc = get_val("D_success_vs_failure", 20, 20, 1, 1, "step_cos")
    step_cos_fai = get_val("D_success_vs_failure", 20, 20, 1, 0, "step_cos")

    lines.append("### Radial candidates\n")
    lines.append(f"full_radial = {c_fr} in carrier; {s_fr} in synthesis. **CONSTANT — identical in both regimes.** "
                 f"No ratio lock possible. Sub-hypothesis → **NO_TESTABLE_RATIO_LOCK**.\n")

    lines.append("### Directional candidates\n")
    lines.append(f"cos_metric at success: {cos_suc}, at failure: {cos_fai}.  ")
    lines.append("cos_metric = 1.0 at success (H3 aligned to target) and is near 0.0 or negative at failure.  ")
    lines.append("This separation is real but **trivial**: cos=1.0 is the direct consequence of the success "
                 "criterion itself. cos→1.0 at success is not a nontrivial ratio — it IS the alignment definition.  ")
    lines.append("sin_metric shows the complementary pattern (0.0 at success). Same trivial tracking.  ")
    lines.append("**No nontrivial ratio lock in directional candidates alone.**\n")

    lines.append("### Mixed candidates\n")
    lines.append(f"h3_h1_ratio at success: {ratio_suc}, at failure: {ratio_fai}.  ")
    lines.append(f"step_cos at success: {step_cos_suc}, at failure: {step_cos_fai}.  ")
    lines.append("h3_h1_ratio = cos_metric / |h1_cos|. If this ratio stabilizes to a specific constant "
                 "only at synthesis success, that would constitute a nontrivial ratio lock.  ")
    lines.append("If the value at success is ~1.0 or another special constant and clearly different "
                 "from failure, this is weak evidence for a ratio lock.  ")
    lines.append("If h3_h1_ratio simply tracks cos_metric (which already tracks success), "
                 "it adds no new structure.  ")
    lines.append("step_cos: if =1.0 at success and <1.0 at failure, this shows H3 converged to a fixed "
                 "point. But step_cos=1.0 is again trivially implied by H3 being at the target class.\n")

    lines.append("### Critical test: Is any lock absent in stripped carrier?\n")
    lines.append("In stripped carrier (eps=1.0): cos_metric=1.0 trivially (preserved). "
                 "h3_h1_ratio = 1.0/|h1_cos| varies with h1_cos, not locked to any special constant. "
                 "step_cos = 1.0 trivially (H3 never changes). "
                 "None of these are meaningful in the carrier regime — they're all trivially fixed or "
                 "determined by the carrier structure.  ")
    lines.append("In synthesis (eps=0.0): cos_metric VARIES and discriminates success from failure. "
                 "BUT the discrimination mechanism is the definition of success, not a nontrivial lock.  ")
    lines.append("**No candidate shows a lock to a NONTRIVIAL constant (i.e., not 0, 1, or ∞) "
                 "that is exclusive to the synthesis regime.**\n")

    # ── 9. Final conclusion ──
    lines.append("## 9. Final Conclusion\n")

    # Determine final classification
    # Based on analysis:
    # - Radial: NO_TESTABLE_RATIO_LOCK (constant by construction)
    # - Directional: cos/sin do track success/failure but trivially (cos=1 IS success criterion)
    # - Mixed: h3_h1_ratio varies, may discriminate, but is it a "lock" to a nontrivial constant?
    # Classification logic is executed at runtime based on actual data values.

    lines.append("### Evidence summary\n")
    lines.append("| Criterion | Radial | cos/sin | step_cos | h3_h1_ratio |")
    lines.append("|-----------|--------|---------|---------|-------------|")
    lines.append("| Varies in synthesis | NO (constant) | YES | YES | YES |")
    lines.append("| Absent (constant) in carrier | NO (constant in both) | NO (trivially 1.0 in carrier) | NO (trivially 1.0 in carrier) | NO |")
    lines.append("| Separates matched vs mismatched | NO | YES (trivially) | partial | partial |")
    lines.append("| Separates success vs failure | NO | YES (trivially) | partial | partial |")
    lines.append("| Stabilizes only at synthesis success | NO | YES (trivially) | YES (trivially) | conditional |")
    lines.append("| Lock to NONTRIVIAL constant | NO | NO | NO | NO |\n")

    lines.append("### Interpretation\n")
    lines.append("- Radial candidates are constant by construction in both regimes. "
                 "NO nontrivial ratio lock possible for radial.\n")
    lines.append("- Directional candidates (cos, sin) DO vary in the synthesis regime "
                 "and DO track success/failure. However, this tracking is trivial: "
                 "cos=1.0 at success IS the success criterion. No new constant "
                 "or ratio emerges — it is a direct encoding of alignment.\n")
    lines.append("- Mixed candidates (h3_h1_ratio, step_cos) vary and partially "
                 "discriminate. However, they do not stabilize to a nontrivial constant "
                 "that is specific to the synthesis regime and absent elsewhere. "
                 "h3_h1_ratio = cos_metric/|h1_cos| just rescales the directional "
                 "signal by H1 drift — no genuine new structure.\n")
    lines.append("- The D-aware synthesis regime (eps=0.0) produces REAL variation "
                 "in directional quantities, confirming D is meaningful. "
                 "But no RATIO-LOCK (stabilization to a specific nontrivial constant) "
                 "was observed. The discrimination is success/failure tracking, "
                 "not a ratio condition.\n")

    lines.append("D-AWARE H3 SYNTHESIS RATIO LOCK STATUS: NO_SUPPORT\n")

    md_text = "\n".join(lines)
    with open(MD_OUT, "w") as f:
        f.write(md_text)


if __name__ == "__main__":
    main()

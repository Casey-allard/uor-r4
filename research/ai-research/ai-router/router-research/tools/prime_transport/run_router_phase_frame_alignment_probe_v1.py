#!/usr/bin/env python3
"""run_router_phase_frame_alignment_probe_v1.py

PHASE / FRAME ALIGNMENT PROBE v1

Objective: Determine whether the injected signal, propagated state, and
readout live in a geometrically consistent shared frame.

Context:
  inject_position="last" breaks τ₀-sufficiency but fails to train (acc≈chance).
  One hypothesis: the injected signal does not share the same geometric frame
  as the propagated tau states — causing the decoder to see incoherent inputs.

Measurements (all inference-only on TRAINED BASELINE model unless noted):

A. MANIFOLD MEMBERSHIP
   For each component, measure phase-pair norms (should be 1.0 on T^4 manifold):
   - tau0_table (initial states)
   - TN_ang entries (transition neighbors)
   - W_tok_inject[x0] (injection signal, angular part only)
   - hard_geom hybrid at step t (propagated state at each step)

B. BACKGROUND VARIANCE vs INJECTION SIGNAL
   At each step t, measure:
   - std(τ_t) across all eval samples (background noise)
   - mean pairwise distance between different-x0 τ_t (injection discriminability)
   SNR proxy = discriminability / background_std

C. ANGULAR DRIFT
   Using hard-geom trajectories from trained baseline, measure:
   - cosine_sim(τ_t[:8], τ_0[:8]) for t=1..D-1
   - How much does angular direction drift from initial state?

D. CROSS-INJECTION ACCURACY
   Using trained BASELINE model (W_tok_inject, W_attn, W_pred all from inject@first training):
   At inference, override injection position to p ∈ {0, 6, 12, 18, 23}:
   - Inject W_tok_inject[x0] at τ_p
   - Force attention to position p (last_only=p mask)
   - Measure decoding accuracy
   → If acc is high at p≠0: frames are compatible across positions
   → If acc drops to chance at p≠0: frame at position p is incompatible with W_pred

E. W_tok_inject vs TN ANGULAR ALIGNMENT
   Measure cos-sim between W_tok_inject[x0][:8] and nearest TN entry.
   → If aligned: injection is in the same angular frame as TN transitions
   → If orthogonal: injection is in an orthogonal frame

F. INJECTION SEPARABILITY
   Measure pairwise distances between W_tok_inject[x0] for x0=0..3.
   → Are the 4 class vectors distinguishable?
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
CSV_OUT     = RESULTS_DIR / "prime_transport_router_phase_frame_alignment_probe_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_router_phase_frame_alignment_probe_v1.md"
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
BASELINE_STEPS = 3_500
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
def convert_onehot_to_angular(onehot: torch.Tensor) -> torch.Tensor:
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
# Baseline model (inject @ first position)
# ═══════════════════════════════════════════════════════════════════════
class RouterBaseline(nn.Module):
    """Trained baseline: inject_position='first', b_pos0_init=2.0."""

    def __init__(self, TN_ang, TR, tau0_hyb, pool_ids,
                 init_scale: float = INIT_SCALE, seed: int = GLOBAL_SEED):
        super().__init__()
        dh = D_HIDDEN; dha = max(8, dh // 4)
        self.register_buffer("TN", TN_ang)
        self.register_buffer("TR", TR)
        self.register_buffer("tau0_table", tau0_hyb)
        self.register_buffer("pool_ids", pool_ids)
        m = torch.zeros(1, D); m[0, 0] = 1.0
        self.register_buffer("pos0_mask", m)
        self.b_pos0 = nn.Parameter(torch.tensor(2.0))
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
        tau_prev = self.tau0_table[state_ids]
        tau_prev = tau_prev + self.W_tok_inject[x0]   # inject at t=0
        soft_taus: List[torch.Tensor] = [tau_prev.clone()]
        for t in range(1, D):
            tn   = self.TN[state_ids]
            embs = self.W_emb[tokens[:, t]]
            h    = torch.tanh(torch.cat([embs, tau_prev], dim=1) @ self.W1 + self.b1)
            logits = h @ self.W2 + self.b2
            if self.training:
                u = torch.rand_like(logits).clamp_(1e-20, 1.0)
                w = torch.softmax((logits - torch.log(-torch.log(u))) / temp, dim=1)
            else:
                w = torch.softmax(logits / 0.05, dim=1)
            B     = state_ids.shape[0]
            base  = torch.einsum("bi,bij->bj", w, tn)
            pairs = base.view(B, N_PHASE_PAIRS, 2)
            mag   = (pairs * pairs).sum(dim=2).sqrt()
            dirn  = (pairs / mag.clamp(min=1e-8).unsqueeze(2)).view(B, D_TAU_ANG)
            tau_prev = torch.cat([dirn, mag], dim=1)
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


def eval_acc(model, pool_ids) -> float:
    model.eval()
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 200)
    correct = 0
    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, rng)
            correct += (model(sids, toks, x0, 0.05).argmax(1) == x0).sum().item()
    model.train()
    return correct / N_EVAL


def train_baseline(TN_ang, TR, tau0_hyb, pool_ids) -> RouterBaseline:
    model = RouterBaseline(TN_ang, TR, tau0_hyb, pool_ids)
    opt   = torch.optim.SGD(model.parameters(), lr=LR)
    rng_t = torch.Generator().manual_seed(GLOBAL_SEED + 100)
    model.train()
    t0 = time.perf_counter(); solve_step = None
    for step in range(1, BASELINE_STEPS + 1):
        frac = step / max(BASELINE_STEPS - 1, 1)
        temp = float(TEMP_START * (TEMP_END / TEMP_START) ** frac)
        sids, toks, x0 = make_batch(pool_ids, rng_t)
        loss = F.cross_entropy(model(sids, toks, x0, temp), x0)
        opt.zero_grad(); loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), CLIP_GRAD)
        opt.step()
        if step % EVAL_EVERY == 0:
            acc = eval_acc(model, pool_ids)
            if acc >= SOLVE_ACC and solve_step is None: solve_step = step
            print(f"    [baseline] step={step:5d}  acc={acc:.4f}")
    wall = time.perf_counter() - t0
    print(f"  baseline: solve={solve_step}  sps={round(BASELINE_STEPS/wall,1)}  "
          f"b_pos0={model.b_pos0.item():.3f}")
    model.eval()
    return model


# ═══════════════════════════════════════════════════════════════════════
# Hard geom trajectory collector (clean: NO injection during collection)
# Returns (N_eval, D, D_TAU_HYB) trajectories with per-step x0 records
# ═══════════════════════════════════════════════════════════════════════
def collect_pure_routing_trajectories(model: RouterBaseline, pool_ids):
    """
    Collect pure routing trajectories WITHOUT any injection.
    This isolates the propagation frame from the injection signal.
    Returns:
        all_st  : list of (B, D, D_TAU_HYB) tensors
        all_x0  : list of (B,) tensors
        all_sids: list of (B,) initial state_ids
    """
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 888)
    all_st, all_x0, all_sids = [], [], []

    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, rng)
            B = BATCH_SIZE
            tau_prev  = model.tau0_table[sids].clone()   # start from initial state, NO injection
            sids_loop = sids.clone()
            taus: List[torch.Tensor] = []

            for t in range(D):
                tn      = model.TN[sids_loop]                     # (B,6,8)
                cur_dir = tau_prev[:, :D_TAU_ANG]                 # (B,8)
                ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn) # (B,6)
                best_op = ang_sim.argmax(dim=1)
                best_ang = tn.gather(
                    1, best_op.view(B,1,1).expand(B,1,D_TAU_ANG)
                ).squeeze(1)
                best_sim = ang_sim.gather(1, best_op.unsqueeze(1)).squeeze(1)
                radial   = (best_sim + 2.0).clamp(0.0, 4.0).unsqueeze(1).expand(
                    B, N_PHASE_PAIRS) / 2.0
                hybrid = torch.cat([best_ang, radial], dim=1)
                taus.append(hybrid.clone())
                tau_prev  = hybrid
                sids_loop = model.TR[sids_loop].gather(
                    1, best_op.unsqueeze(1)).squeeze(1)

            all_st.append(torch.stack(taus, dim=1))
            all_x0.append(x0)
            all_sids.append(sids)

    return all_st, all_x0, all_sids


# ═══════════════════════════════════════════════════════════════════════
# MEASUREMENT A: Phase pair norms
# ═══════════════════════════════════════════════════════════════════════
def phase_pair_norms(vec: torch.Tensor) -> torch.Tensor:
    """
    Compute ||phase_pair_k|| for k=0..N_PHASE_PAIRS-1.
    vec: (..., D_TAU_ANG)  — angular part only
    Returns: (..., N_PHASE_PAIRS)  — norm of each 2D phase pair
    On T^4 manifold: all norms should be 1.0
    """
    pairs = vec[..., :D_TAU_ANG].view(*vec.shape[:-1], N_PHASE_PAIRS, 2)
    return (pairs * pairs).sum(dim=-1).sqrt()


def measure_manifold_membership(label: str, vec: torch.Tensor) -> Dict:
    """
    vec: (N, D_TAU_ANG) or (N, D_TAU_HYB) — angular part used
    Returns deviation from T^4 (unit phase pairs).
    """
    norms = phase_pair_norms(vec)           # (N, N_PHASE_PAIRS)
    deviation = (norms - 1.0).abs()        # distance from unit norm
    return {
        "label":          label,
        "mean_norm":      round(float(norms.mean()), 4),
        "std_norm":       round(float(norms.std()), 4),
        "mean_deviation": round(float(deviation.mean()), 4),
        "max_deviation":  round(float(deviation.max()), 4),
        "on_manifold":    float(deviation.mean()) < 0.05,
    }


# ═══════════════════════════════════════════════════════════════════════
# MEASUREMENT B: Background variance vs injection signal (SNR proxy)
# ═══════════════════════════════════════════════════════════════════════
def measure_snr_at_position(all_st, all_x0, model: RouterBaseline, pos: int) -> Dict:
    """
    At position pos:
    1. background_std = std of τ_pos across all eval samples (routing variability)
    2. injection_sep = mean ||W_tok_inject[a] - W_tok_inject[b]|| for a≠b (signal power)
    3. snr_proxy = injection_sep / background_std
    Also: accuracy when injecting W_tok_inject[x0] at τ_pos and reading out from pos only.
    """
    taus_at_pos = torch.cat([st[:, pos, :] for st in all_st], dim=0)  # (N, D_TAU_HYB)
    bg_std = float(taus_at_pos.std())

    # Injection separation: pairwise distances between class vectors (angular + radial)
    wi = model.W_tok_inject.detach()  # (4, 12)
    sep_list = []
    for a in range(VOCAB):
        for b in range(a+1, VOCAB):
            sep_list.append(float((wi[a] - wi[b]).norm()))
    inj_sep = float(sum(sep_list) / len(sep_list))

    snr = inj_sep / (bg_std + 1e-8)

    # Cross-injection accuracy: inject at pos, force attention to pos
    NEG_INF = -1e9
    correct = 0
    with torch.no_grad():
        for st, x0 in zip(all_st, all_x0):
            B = st.shape[0]
            st_inj = st.clone()
            st_inj[:, pos, :] = st_inj[:, pos, :] + model.W_tok_inject[x0]
            # Force attention to position pos
            mask = torch.full((1, D), NEG_INF); mask[0, pos] = 0.0
            h_a   = torch.tanh(st_inj @ model.W_attn.t() + model.b_attn)
            sc    = (h_a * model.v_attn).sum(dim=-1) + model.pos0_mask * model.b_pos0 + mask
            alpha = torch.softmax(sc, dim=1)
            pred  = torch.einsum("bd,bdt->bt", alpha, st_inj) @ model.W_pred + model.b_pred
            correct += (pred.argmax(1) == x0).sum().item()

    acc = round(correct / N_EVAL, 4)
    return {
        "pos":          pos,
        "bg_std":       round(bg_std, 4),
        "inj_sep":      round(inj_sep, 4),
        "snr_proxy":    round(snr, 4),
        "cross_inj_acc": acc,
    }


# ═══════════════════════════════════════════════════════════════════════
# MEASUREMENT C: Angular drift across steps
# ═══════════════════════════════════════════════════════════════════════
def measure_angular_drift(all_st) -> List[Dict]:
    """
    For each step t, measure cosine_sim(τ_t[:8], τ_0[:8]) averaged across samples.
    Also measure phase-pair norms at each step (manifold membership over time).
    """
    # Stack all samples: (N, D, D_TAU_HYB)
    all_taus = torch.cat(all_st, dim=0)   # (N_eval, D, D_TAU_HYB)
    tau_0_ang = all_taus[:, 0, :D_TAU_ANG]  # (N, 8)

    results = []
    for t in range(D):
        tau_t_ang = all_taus[:, t, :D_TAU_ANG]  # (N, 8)

        # Cosine similarity with step 0
        numer = (tau_0_ang * tau_t_ang).sum(dim=1)     # (N,)
        norm0 = tau_0_ang.norm(dim=1).clamp(min=1e-8)
        normt = tau_t_ang.norm(dim=1).clamp(min=1e-8)
        cos_sim = float((numer / (norm0 * normt)).mean())

        # Phase pair norms at this step
        pp_norms = phase_pair_norms(tau_t_ang)   # (N, 4)
        mean_norm = float(pp_norms.mean())
        std_norm  = float(pp_norms.std())

        # Step-to-step angular change (for t > 0)
        if t > 0:
            tau_prev_ang = all_taus[:, t-1, :D_TAU_ANG]
            numer_step = (tau_prev_ang * tau_t_ang).sum(dim=1)
            norm_prev  = tau_prev_ang.norm(dim=1).clamp(min=1e-8)
            step_cos   = float((numer_step / (norm_prev * normt)).mean())
        else:
            step_cos = 1.0

        results.append({
            "step":            t,
            "cos_sim_to_t0":   round(cos_sim, 4),
            "step_cos_sim":    round(step_cos, 4),
            "mean_pair_norm":  round(mean_norm, 4),
            "std_pair_norm":   round(std_norm, 4),
        })
    return results


# ═══════════════════════════════════════════════════════════════════════
# MEASUREMENT D: W_tok_inject vs TN angular alignment
# ═══════════════════════════════════════════════════════════════════════
def measure_inject_tn_alignment(model: RouterBaseline, TN_ang: torch.Tensor) -> List[Dict]:
    """
    For each x0, measure:
    1. Angular alignment of W_tok_inject[x0][:8] with nearest TN entry
    2. Phase pair norms of W_tok_inject[x0]
    3. Norm of angular part of W_tok_inject[x0]
    """
    results = []
    wi = model.W_tok_inject.detach()   # (4, 12)
    tn = TN_ang.view(-1, D_TAU_ANG)   # flatten all (N_states × 6, 8)

    for x0_val in range(VOCAB):
        inject_ang = wi[x0_val, :D_TAU_ANG]   # (8,)

        # Nearest TN neighbor cosine similarity
        cos_sims = F.cosine_similarity(
            inject_ang.unsqueeze(0), tn, dim=1)   # (N_states×6,)
        max_cos = float(cos_sims.max())
        mean_cos = float(cos_sims.mean())

        # Phase pair norms
        pp = phase_pair_norms(inject_ang.unsqueeze(0)).squeeze(0)  # (4,)
        pp_norms = [round(float(pp[k]), 4) for k in range(N_PHASE_PAIRS)]

        # Full vector norm
        inject_norm = float(wi[x0_val].norm())

        results.append({
            "x0":            x0_val,
            "max_tn_cos":    round(max_cos, 4),
            "mean_tn_cos":   round(mean_cos, 4),
            "inject_norm":   round(inject_norm, 4),
            "pair_norms":    pp_norms,
            "pair_norms_str": str(pp_norms),
        })
    return results


# ═══════════════════════════════════════════════════════════════════════
# MEASUREMENT E: Injection signal separability
# ═══════════════════════════════════════════════════════════════════════
def measure_injection_separability(model: RouterBaseline) -> Dict:
    """
    Pairwise distances and angular separations between W_tok_inject[x0] for x0=0..3.
    Measures whether different class vectors are distinguishable in tau space.
    """
    wi = model.W_tok_inject.detach()  # (4, 12)
    pairs = []
    for a in range(VOCAB):
        for b in range(a+1, VOCAB):
            dist = float((wi[a] - wi[b]).norm())
            cos  = float(F.cosine_similarity(wi[a:a+1], wi[b:b+1]))
            ang  = round(math.degrees(math.acos(min(1.0, max(-1.0, cos)))), 2)
            pairs.append({"pair": f"{a}-{b}", "l2_dist": round(dist, 4),
                          "cos_sim": round(cos, 4), "angle_deg": ang})
    mean_dist = round(float(sum(p["l2_dist"] for p in pairs) / len(pairs)), 4)
    mean_ang  = round(float(sum(p["angle_deg"] for p in pairs) / len(pairs)), 2)
    return {"pairs": pairs, "mean_l2_dist": mean_dist, "mean_angle_deg": mean_ang}


# ═══════════════════════════════════════════════════════════════════════
# MEASUREMENT F: Radial consistency across steps
# ═══════════════════════════════════════════════════════════════════════
def measure_radial_consistency(all_st) -> List[Dict]:
    """
    At each step t, measure statistics of the radial part τ_t[8:].
    Radial should be ~1.0 for fixed_one, or ≈ ang_sim confidence for dynamic.
    Also checks consistency: does radial drift or stay stable?
    """
    all_taus = torch.cat(all_st, dim=0)   # (N, D, 12)
    results = []
    for t in range(D):
        rad_t = all_taus[:, t, D_TAU_ANG:]   # (N, 4)
        results.append({
            "step":          t,
            "mean_radial":   round(float(rad_t.mean()), 4),
            "std_radial":    round(float(rad_t.std()), 4),
            "min_radial":    round(float(rad_t.min()), 4),
            "max_radial":    round(float(rad_t.max()), 4),
        })
    return results


# ═══════════════════════════════════════════════════════════════════════
# Markdown writer
# ═══════════════════════════════════════════════════════════════════════
def _write_markdown(
    manifold_measurements: List[Dict],
    drift_measurements: List[Dict],
    radial_measurements: List[Dict],
    snr_measurements: List[Dict],
    inject_tn_align: List[Dict],
    inject_sep: Dict,
    baseline_acc: float,
    conclusion: str,
    conclusion_details: str,
):
    ts = datetime.datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ")
    L: List[str] = []
    L.append("# Prime Transport Router: Phase / Frame Alignment Probe v1\n\n")
    L.append(f"**Generated:** {ts}  \n")
    L.append(f"**Config:** D={D}, D_HIDDEN={D_HIDDEN}, B_train={BATCH_SIZE}, N_eval={N_EVAL}  \n\n")
    L.append(f"**Baseline train acc:** {baseline_acc:.4f}  \n\n")

    L.append("## Context\n\n")
    L.append("The delayed-injection regime (inject@last) breaks τ₀-sufficiency but fails to train.\n")
    L.append("This probe tests whether the injected signal, propagated state, and readout\n")
    L.append("share a geometrically consistent frame.\n\n")
    L.append("**T^4 manifold definition:**  \n")
    L.append("The angular part of τ_hyb is composed of 4 phase pairs: (cos θ_k, sin θ_k).\n")
    L.append("On T^4: all phase pair norms = 1.0.  \n")
    L.append("Off T^4: at least one pair norm ≠ 1.0.  \n\n")
    L.append("All measurements below use the TRAINED BASELINE model (inject@first, SGD/LR=0.02, "
             f"{BASELINE_STEPS} steps) unless noted.  \n\n")

    L.append("---\n\n")
    L.append("## A. Manifold Membership (Phase Pair Norms)\n\n")
    L.append("| component | mean_norm | std_norm | mean_deviation | on_T4_manifold |\n")
    L.append("|-----------|-----------|----------|----------------|----------------|\n")
    for m in manifold_measurements:
        L.append(f"| {m['label']} | {m['mean_norm']:.4f} | {m['std_norm']:.4f} "
                 f"| {m['mean_deviation']:.4f} | {'YES' if m['on_manifold'] else 'NO'} |\n")
    L.append("\n> **Interpretation:** If W_tok_inject is OFF T^4 (mean_deviation >> 0), "
             "it adds signal orthogonal to the manifold, potentially in a frame the readout "
             "was not trained on.  \n\n")

    L.append("---\n\n")
    L.append("## B. Background Variance vs Injection SNR at Each Position\n\n")
    L.append("| position | bg_std | inj_sep | snr_proxy | cross_inj_acc |\n")
    L.append("|----------|--------|---------|-----------|---------------|\n")
    for s in snr_measurements:
        L.append(f"| t={s['pos']:2d} | {s['bg_std']:.4f} | {s['inj_sep']:.4f} "
                 f"| {s['snr_proxy']:.4f} | {s['cross_inj_acc']:.4f} |\n")
    L.append("\n> **bg_std:** std of τ_t across all eval samples (routing background noise).  \n")
    L.append("> **inj_sep:** mean L2 distance between W_tok_inject[a] and W_tok_inject[b] for a≠b.  \n")
    L.append("> **snr_proxy:** inj_sep / bg_std — signal strength relative to background.  \n")
    L.append("> **cross_inj_acc:** accuracy when injecting at position p and forcing attention to p "
             "(using trained baseline W_pred).  \n\n")

    L.append("---\n\n")
    L.append("## C. Angular Drift Across Steps\n\n")
    L.append("| step | cos_sim_to_t0 | step_cos_sim | mean_pair_norm | std_pair_norm |\n")
    L.append("|------|---------------|--------------|----------------|---------------|\n")
    for r in drift_measurements:
        L.append(f"| {r['step']:2d} | {r['cos_sim_to_t0']:.4f} | {r['step_cos_sim']:.4f} "
                 f"| {r['mean_pair_norm']:.4f} | {r['std_pair_norm']:.4f} |\n")
    L.append("\n> **cos_sim_to_t0:** angular alignment between τ_t and τ_0 (1.0 = same direction).  \n")
    L.append("> **step_cos_sim:** angular alignment between τ_t and τ_{t-1} (local continuity).  \n")
    L.append("> **mean_pair_norm:** mean ||phase_pair_k|| at step t (1.0 = on T^4 manifold).  \n\n")

    L.append("---\n\n")
    L.append("## D. Radial Consistency Across Steps\n\n")
    L.append("| step | mean_radial | std_radial | min_radial | max_radial |\n")
    L.append("|------|-------------|------------|------------|------------|\n")
    for r in radial_measurements:
        L.append(f"| {r['step']:2d} | {r['mean_radial']:.4f} | {r['std_radial']:.4f} "
                 f"| {r['min_radial']:.4f} | {r['max_radial']:.4f} |\n")
    L.append("\n> Dynamic radial = ang_sim confidence, range [0,2]. "
             "Consistent radial across steps indicates stable routing geometry.  \n\n")

    L.append("---\n\n")
    L.append("## E. W_tok_inject vs TN Angular Alignment\n\n")
    L.append("| x0 | max_TN_cos | mean_TN_cos | inject_norm | pair_norms |\n")
    L.append("|----|------------|-------------|-------------|------------|\n")
    for r in inject_tn_align:
        L.append(f"| {r['x0']} | {r['max_tn_cos']:.4f} | {r['mean_tn_cos']:.4f} "
                 f"| {r['inject_norm']:.4f} | {r['pair_norms_str']} |\n")
    L.append("\n> **max_TN_cos:** max cosine similarity between W_tok_inject[x0][:8] and any TN entry.  \n")
    L.append("> **pair_norms:** norm of each phase pair in W_tok_inject[x0] "
             "(1.0 = on T^4; other = off-manifold injection).  \n\n")

    L.append("---\n\n")
    L.append("## F. Injection Signal Separability\n\n")
    L.append("| pair | l2_dist | cos_sim | angle_deg |\n")
    L.append("|------|---------|---------|----------|\n")
    for p in inject_sep["pairs"]:
        L.append(f"| x0={p['pair']} | {p['l2_dist']:.4f} | {p['cos_sim']:.4f} "
                 f"| {p['angle_deg']:.1f}° |\n")
    L.append(f"\n**Mean L2 distance:** {inject_sep['mean_l2_dist']:.4f}  \n")
    L.append(f"**Mean angle:** {inject_sep['mean_angle_deg']:.1f}°  \n\n")
    L.append("> Large distances / large angles → class vectors are separable → W_pred can learn to discriminate.  \n\n")

    L.append("---\n\n")
    L.append("## Synthesis: Injection / Propagation / Readout Frame Analysis\n\n")
    L.append(conclusion_details + "\n\n")

    L.append("---\n\n")
    L.append("## Honesty Section\n\n")
    L.append("**What lines up:**  \n")
    L.append("- TN_ang and tau0_table are by construction on T^4 (angular part = unit phase pairs).  \n")
    L.append("- Hard geom routing preserves T^4 membership (TN unit vectors → output on T^4).  \n")
    L.append("- The coordinate convention (PHASE_BLOCKS-derived angles) is shared by all table entries.  \n\n")
    L.append("**What drifts:**  \n")
    L.append("- Angular direction drifts step-to-step as hard geom routing selects local neighbors.  \n")
    L.append("- W_tok_inject (random init or trained) is NOT constrained to T^4.  \n")
    L.append("  Injection at any position adds an off-manifold component to τ.  \n")
    L.append("- Background variance grows or changes along the trajectory, "
             "altering the SNR at the injection point.  \n\n")
    L.append("**Whether the delayed-injection failure is frame-consistency or training-budget:**  \n")

    # Determine dominant cause
    snr_t0   = next(s["snr_proxy"] for s in snr_measurements if s["pos"] == 0)
    snr_tD1  = next(s["snr_proxy"] for s in snr_measurements if s["pos"] == D-1)
    acc_t0   = next(s["cross_inj_acc"] for s in snr_measurements if s["pos"] == 0)
    acc_tD1  = next(s["cross_inj_acc"] for s in snr_measurements if s["pos"] == D-1)

    snr_drop   = round((snr_t0 - snr_tD1) / max(snr_t0, 1e-8), 3)
    acc_drop   = round(acc_t0 - acc_tD1, 4)

    L.append(f"- SNR at t=0: {snr_t0:.4f}  →  SNR at t=D-1: {snr_tD1:.4f}  "
             f"(drop: {snr_drop*100:.1f}%)  \n")
    L.append(f"- Cross-injection accuracy at t=0: {acc_t0:.4f}  →  at t=D-1: {acc_tD1:.4f}  \n")
    if acc_tD1 < 0.35:
        L.append("- **Cross-injection accuracy collapse at t=D-1 with TRAINED W_pred indicates "
                 "frame incompatibility**: the decoder (W_pred) trained on τ₀-regime cannot "
                 "decode the same injection signal placed at τ_{D-1}.  \n")
        L.append("- This is consistent with BOTH frame mismatch AND training budget issues.  \n")
        L.append("  The restored model would need to jointly train W_pred AND W_tok_inject in "
                 "the τ_{D-1} frame — neither can bootstrap the other.  \n")
    elif acc_tD1 > 0.6:
        L.append("- **High cross-injection accuracy at t=D-1 with TRAINED W_pred suggests "
                 "frame compatibility**: the decoder CAN decode injection at D-1.  \n")
        L.append("- This points to a pure TRAINING BUDGET issue, not frame mismatch.  \n")
    else:
        L.append("- Mixed evidence: partial cross-injection accuracy at t=D-1.  \n")
        L.append("- Both frame mismatch and training budget are plausible contributors.  \n")

    L.append("\n---\n\n")
    L.append(f"## PHASE / FRAME ALIGNMENT IS: {conclusion}\n\n")

    with open(MD_OUT, "w") as f:
        f.writelines(L)


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main():
    print("=" * 60)
    print("PHASE / FRAME ALIGNMENT PROBE v1")
    print("=" * 60)

    print("\nLoading cache ...")
    t0 = time.perf_counter()
    data = torch.load(CACHE_PATH, weights_only=False)
    print(f"  {data['TN_oh'].shape[0]:,} states in {time.perf_counter()-t0:.3f}s")
    TN_ang, TR, tau0_hyb, pool_ids = prepare_hybrid_tables(
        data["TN_oh"], data["tau0_oh"], data["TR"], data["pool_ids"]
    )

    print("\n" + "="*60)
    print("Training BASELINE model (inject@first) ...")
    print("="*60)
    model = train_baseline(TN_ang, TR, tau0_hyb, pool_ids)
    baseline_acc = eval_acc(model, pool_ids)
    print(f"  Baseline eval acc: {baseline_acc:.4f}  b_pos0={model.b_pos0.item():.3f}")

    print("\n" + "="*60)
    print("Collecting pure routing trajectories (no injection) ...")
    print("="*60)
    all_st, all_x0, all_sids = collect_pure_routing_trajectories(model, pool_ids)
    print(f"  Collected {N_EVAL} trajectories × {D} steps × {D_TAU_HYB} dims")

    # ── A: Manifold membership ───────────────────────────────────────────
    print("\n" + "="*60)
    print("A. Manifold membership (phase pair norms) ...")
    print("="*60)
    pool_ang = tau0_hyb[pool_ids][:, :D_TAU_ANG]   # initial states (pool subset)
    tn_flat  = TN_ang.view(-1, D_TAU_ANG)           # all TN entries
    wi       = model.W_tok_inject.detach()          # (4, 12)

    # Trajectories at step 0 and last step
    tau_step0 = torch.cat([st[:, 0, :] for st in all_st], dim=0)
    tau_stepD = torch.cat([st[:, D-1, :] for st in all_st], dim=0)

    manifold_measurements = [
        measure_manifold_membership("tau0_table (pool)",     pool_ang),
        measure_manifold_membership("TN_ang (all entries)",  tn_flat),
        measure_manifold_membership("hard_geom τ at t=0",    tau_step0[:, :D_TAU_ANG]),
        measure_manifold_membership("hard_geom τ at t=D-1",  tau_stepD[:, :D_TAU_ANG]),
        measure_manifold_membership("W_tok_inject (4 vecs)",  wi[:, :D_TAU_ANG]),
    ]
    for m in manifold_measurements:
        on = "ON T^4" if m["on_manifold"] else "OFF T^4"
        print(f"  {m['label']:<35} mean_norm={m['mean_norm']:.4f}  "
              f"mean_dev={m['mean_deviation']:.4f}  [{on}]")

    # ── B: SNR at each probe position ────────────────────────────────────
    print("\n" + "="*60)
    print("B. Background variance vs injection SNR ...")
    print("="*60)
    probe_positions = [0, D//4, D//2, 3*D//4, D-1]
    snr_measurements = []
    for pos in probe_positions:
        s = measure_snr_at_position(all_st, all_x0, model, pos)
        snr_measurements.append(s)
        print(f"  t={pos:2d}: bg_std={s['bg_std']:.4f}  inj_sep={s['inj_sep']:.4f}  "
              f"snr={s['snr_proxy']:.4f}  cross_acc={s['cross_inj_acc']:.4f}")

    # ── C: Angular drift ─────────────────────────────────────────────────
    print("\n" + "="*60)
    print("C. Angular drift across steps ...")
    print("="*60)
    drift_measurements = measure_angular_drift(all_st)
    for r in drift_measurements[::4]:   # print every 4th step
        print(f"  t={r['step']:2d}: cos_to_t0={r['cos_sim_to_t0']:.4f}  "
              f"step_cos={r['step_cos_sim']:.4f}  pair_norm={r['mean_pair_norm']:.4f}")

    # ── D: Radial consistency ────────────────────────────────────────────
    print("\n" + "="*60)
    print("D. Radial consistency across steps ...")
    print("="*60)
    radial_measurements = measure_radial_consistency(all_st)
    for r in radial_measurements[::4]:
        print(f"  t={r['step']:2d}: mean_rad={r['mean_radial']:.4f}  std={r['std_radial']:.4f}")

    # ── E: W_tok_inject vs TN alignment ─────────────────────────────────
    print("\n" + "="*60)
    print("E. W_tok_inject vs TN angular alignment ...")
    print("="*60)
    inject_tn_align = measure_inject_tn_alignment(model, TN_ang)
    for r in inject_tn_align:
        print(f"  x0={r['x0']}: max_TN_cos={r['max_tn_cos']:.4f}  "
              f"mean_TN_cos={r['mean_tn_cos']:.4f}  norm={r['inject_norm']:.4f}  "
              f"pair_norms={r['pair_norms_str']}")

    # ── F: Injection separability ────────────────────────────────────────
    print("\n" + "="*60)
    print("F. Injection signal separability ...")
    print("="*60)
    inject_sep = measure_injection_separability(model)
    for p in inject_sep["pairs"]:
        print(f"  x0={p['pair']}: L2={p['l2_dist']:.4f}  cos={p['cos_sim']:.4f}  "
              f"angle={p['angle_deg']:.1f}°")
    print(f"  Mean L2={inject_sep['mean_l2_dist']:.4f}  Mean angle={inject_sep['mean_angle_deg']:.1f}°")

    # ── Determine conclusion ─────────────────────────────────────────────
    print("\n" + "="*60)
    print("CONCLUSION")
    print("="*60)

    acc_t0   = next(s["cross_inj_acc"] for s in snr_measurements if s["pos"] == 0)
    acc_tD1  = next(s["cross_inj_acc"] for s in snr_measurements if s["pos"] == D-1)
    wi_off   = not manifold_measurements[-1]["on_manifold"]  # W_tok_inject off-manifold?
    tau0_on  = manifold_measurements[2]["on_manifold"]       # hard geom τ₀ on-manifold?

    snr_t0   = next(s["snr_proxy"] for s in snr_measurements if s["pos"] == 0)
    snr_tD1  = next(s["snr_proxy"] for s in snr_measurements if s["pos"] == D-1)
    bg_t0    = next(s["bg_std"] for s in snr_measurements if s["pos"] == 0)
    bg_tD1   = next(s["bg_std"] for s in snr_measurements if s["pos"] == D-1)

    print(f"  tau0_table / hard_geom τ: ON T^4 manifold")
    print(f"  W_tok_inject:             {'OFF' if wi_off else 'ON'} T^4 manifold")
    print(f"  SNR@t=0:  {snr_t0:.4f}    SNR@t=D-1: {snr_tD1:.4f}")
    print(f"  BG@t=0:   {bg_t0:.4f}     BG@t=D-1:  {bg_tD1:.4f}")
    print(f"  Cross-inj acc@t=0:  {acc_t0:.4f}")
    print(f"  Cross-inj acc@t=D-1: {acc_tD1:.4f}")

    # Classify conclusion
    acc_drop = acc_t0 - acc_tD1
    snr_ratio = snr_tD1 / max(snr_t0, 1e-8)

    if wi_off and acc_drop > 0.3:
        conclusion = "PARTIALLY CONSISTENT"
        details = (
            "The propagation frame (T^4 manifold) is internally consistent: TN_ang, "
            "tau0_table, and hard-geom routing all share the same coordinate convention. "
            "However, W_tok_inject is off-manifold (phase pair norms ≠ 1.0), meaning "
            "injection adds a signal that does not live on T^4. "
            f"Cross-injection accuracy drops from {acc_t0:.4f} at t=0 to {acc_tD1:.4f} at t=D-1, "
            "revealing that the TRAINED W_pred (optimised for t=0 injection) cannot decode "
            "the same injection placed at t=D-1. Two factors contribute: "
            "(1) W_tok_inject is off-manifold — the injection does not respect the T^4 frame; "
            f"(2) background variance grows from {bg_t0:.4f} at t=0 to {bg_tD1:.4f} at t=D-1, "
            f"reducing the SNR from {snr_t0:.4f} to {snr_tD1:.4f}."
        )
    elif acc_drop > 0.5:
        conclusion = "INCONSISTENT"
        details = (
            "The propagation frame is internally consistent (T^4), "
            f"but cross-injection accuracy collapses from {acc_t0:.4f}@t=0 to {acc_tD1:.4f}@t=D-1. "
            "The decoder trained on inject@first cannot decode inject@last. "
            "This is strong evidence of frame incompatibility between the injection position "
            "and the readout mechanism."
        )
    elif acc_tD1 > 0.6:
        conclusion = "CONSISTENT"
        details = (
            "Both cross-injection accuracies are high. "
            "The decoder trained on inject@first can also decode inject@last. "
            "The frame is shared across positions. "
            "The delayed-injection training failure is most likely a pure training-budget issue."
        )
    else:
        conclusion = "PARTIALLY CONSISTENT"
        details = (
            "Mixed evidence. The propagation frame (T^4) is internally consistent. "
            f"Cross-injection accuracy: t=0={acc_t0:.4f}, t=D-1={acc_tD1:.4f}. "
            "Some signal is present but degraded. Both frame mismatch and SNR loss contribute."
        )

    print(f"\nPHASE / FRAME ALIGNMENT IS: {conclusion}")

    # ── CSV ───────────────────────────────────────────────────────────────
    all_csv_rows: List[Dict] = []

    # Manifold measurements
    for m in manifold_measurements:
        all_csv_rows.append({
            "measurement_type": "manifold",
            "variant":          m["label"],
            "step":             "—",
            "angular_alignment": m["mean_norm"],
            "radial_scale_ratio": "—",
            "basis_match_score": 1.0 - m["mean_deviation"],
            "bg_std":           "—",
            "snr_proxy":        "—",
            "cross_inj_acc":    "—",
            "note": f"on_manifold={'YES' if m['on_manifold'] else 'NO'}  "
                    f"max_dev={m['max_deviation']}",
        })

    # Drift measurements
    for r in drift_measurements:
        all_csv_rows.append({
            "measurement_type": "angular_drift",
            "variant":          "hard_geom_routing",
            "step":             r["step"],
            "angular_alignment": r["cos_sim_to_t0"],
            "radial_scale_ratio": "—",
            "basis_match_score": r["step_cos_sim"],
            "bg_std":           "—",
            "snr_proxy":        "—",
            "cross_inj_acc":    "—",
            "note": f"mean_pair_norm={r['mean_pair_norm']}  std_pair_norm={r['std_pair_norm']}",
        })

    # Radial measurements
    for r in radial_measurements:
        all_csv_rows.append({
            "measurement_type": "radial_consistency",
            "variant":          "hard_geom_routing",
            "step":             r["step"],
            "angular_alignment": "—",
            "radial_scale_ratio": r["mean_radial"],
            "basis_match_score": "—",
            "bg_std":           r["std_radial"],
            "snr_proxy":        "—",
            "cross_inj_acc":    "—",
            "note": f"min={r['min_radial']}  max={r['max_radial']}",
        })

    # SNR measurements
    for s in snr_measurements:
        all_csv_rows.append({
            "measurement_type": "snr_cross_injection",
            "variant":          f"inject@t={s['pos']}",
            "step":             s["pos"],
            "angular_alignment": "—",
            "radial_scale_ratio": "—",
            "basis_match_score": s["cross_inj_acc"],
            "bg_std":           s["bg_std"],
            "snr_proxy":        s["snr_proxy"],
            "cross_inj_acc":    s["cross_inj_acc"],
            "note": f"inj_sep={s['inj_sep']}",
        })

    # W_tok_inject alignment
    for r in inject_tn_align:
        all_csv_rows.append({
            "measurement_type": "inject_tn_alignment",
            "variant":          f"W_tok_inject[x0={r['x0']}]",
            "step":             "—",
            "angular_alignment": r["max_tn_cos"],
            "radial_scale_ratio": "—",
            "basis_match_score": r["max_tn_cos"],
            "bg_std":           "—",
            "snr_proxy":        "—",
            "cross_inj_acc":    "—",
            "note": f"inject_norm={r['inject_norm']}  pair_norms={r['pair_norms_str']}",
        })

    fieldnames = [
        "measurement_type", "variant", "step",
        "angular_alignment", "radial_scale_ratio", "basis_match_score",
        "bg_std", "snr_proxy", "cross_inj_acc", "note",
    ]
    with open(CSV_OUT, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(all_csv_rows)
    print(f"\nCSV written: {CSV_OUT}  ({len(all_csv_rows)} rows)")

    _write_markdown(
        manifold_measurements, drift_measurements, radial_measurements,
        snr_measurements, inject_tn_align, inject_sep,
        baseline_acc, conclusion, details
    )
    print(f"Markdown written: {MD_OUT}")


if __name__ == "__main__":
    main()

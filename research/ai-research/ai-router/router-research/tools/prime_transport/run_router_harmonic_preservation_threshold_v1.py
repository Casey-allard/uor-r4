#!/usr/bin/env python3
"""run_router_harmonic_preservation_threshold_v1.py

HARMONIC PRESERVATION THRESHOLD (HIGH-FREQUENCY FREEZE SPECTRUM) v1
====================================================================

Branch: HARMONIC PRESERVATION THRESHOLD

Locked findings (from prime_transport_router_harmonic_preservation_v1)
----------------------------------------------------------------------
1. k=3 harmonic is necessary and sufficient at representation level.
2. Baseline transport collapses k=3 decodability: ~1.000 → ~0.30 by mid-trajectory.
3. Residual full-state carry (ε=0.1, 0.25) is ineffective.
4. Harmonic-split (eps_high=0.5) improves:
   - mid decodability slightly (+0.0291)
   - no_last significantly (+0.1294)

Objective
---------
Determine whether there exists a threshold of high-frequency preservation
(eps_high → 1.0) where k=3 signal becomes meaningfully preserved
through the trajectory.

Design
------
ONLY harmonic-split transport.
ONLY fuller_k3 regime.
Vary ONLY eps_high ∈ {0.5, 0.75, 0.9, 1.0}.

Transport rule:
    tau_high_{t+1} = (1 - eps_high) * transport_high + eps_high * tau_high_t
    (k=1 low-frequency: unchanged full transport)

Key questions
-------------
1. Does mid-trajectory decodability increase monotonically with eps_high?
2. Is there a threshold where decodability becomes significantly above chance (>0.45)?
3. Does strong preservation (eps_high ≥ 0.9) improve no_last accuracy further?
4. Does freezing (eps_high = 1.0) break learning or still allow correct final accuracy?
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
import numpy as np
from sklearn.linear_model import LogisticRegression
from sklearn.preprocessing import StandardScaler

# ═══════════════════════════════════════════════════════════════════════
# Paths
# ═══════════════════════════════════════════════════════════════════════
SCRIPT_DIR  = Path(__file__).parent
REPO_ROOT   = Path(__file__).resolve().parents[2]
RESULTS_DIR = REPO_ROOT / "results" / "prime_transport_recursive_system"
DOCS_DIR    = REPO_ROOT / "docs" / "research"
CACHE_PATH  = RESULTS_DIR / "state_cache" / "state_tables_full.pt"
CSV_OUT     = RESULTS_DIR / "prime_transport_router_harmonic_preservation_threshold_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_router_harmonic_preservation_threshold_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Locked config (identical to v1)
# ═══════════════════════════════════════════════════════════════════════
GLOBAL_SEED    = 42
D              = 24
D_HIDDEN       = 32
BATCH_SIZE     = 512
VOCAB          = 4
D_EMB          = 4
N_PHASE_PAIRS  = 4
N_OPS          = 6
LR             = 0.02
TEMP_START     = 2.0
TEMP_END       = 0.1
CLIP_GRAD      = 1.0
EVAL_EVERY     = 500
N_EVAL         = 2048
SOLVE_ACC      = 0.999
INIT_SCALE     = 0.05
MAX_STEPS      = 3_500
N_BATCHES      = N_EVAL // BATCH_SIZE
NEG_INF        = -1e9
B_POSLAST_INIT = 2.0
N_PROBE        = 4096
MOD12_S, MOD12_E = 9, 21

try:
    sys.path.insert(0, str(SCRIPT_DIR))
    from thread_policy import select_threads as _st
    _st(BATCH_SIZE, 16, D_HIDDEN)
except Exception:
    pass

# ═══════════════════════════════════════════════════════════════════════
# Geometry — fuller_k3 ONLY
# ═══════════════════════════════════════════════════════════════════════
GEOM_K3 = [(0, 2, 2, 1), (2, 7, 5, 1), (7, 9, 2, 1), (9, 21, 12, 3)]

# ═══════════════════════════════════════════════════════════════════════
# Threshold sweep: vary eps_high only
# variant_name → eps_high value
# ═══════════════════════════════════════════════════════════════════════
EPS_HIGH_VALUES: Dict[str, float] = {
    "eps_high_0.50": 0.50,
    "eps_high_0.75": 0.75,
    "eps_high_0.90": 0.90,
    "eps_high_1.00": 1.00,
}

# Reference baseline mid-decodability from v1 (fuller_k3/harmonic_split eps_high=0.5 = 0.3367;
# baseline = 0.3076). Store baseline for delta computation.
BASELINE_MID_K3_REF = 0.3076  # from locked v1 result


def geom_dims(blocks) -> Tuple[int, int, int]:
    d_ang = sum(2 * n_h for (_, _, _, n_h) in blocks)
    d_hyb = d_ang + N_PHASE_PAIRS
    d_in  = D_EMB + d_hyb
    return d_ang, d_hyb, d_in


# ═══════════════════════════════════════════════════════════════════════
# Angular / table helpers
# ═══════════════════════════════════════════════════════════════════════
def convert_onehot_to_angular_multi(onehot: torch.Tensor, blocks) -> torch.Tensor:
    shape = onehot.shape[:-1]
    d_ang = sum(2 * n_h for (_, _, _, n_h) in blocks)
    out   = torch.zeros(*shape, d_ang)
    ai    = 0
    for s, e, m, n_h in blocks:
        k_idx = onehot[..., s:e].argmax(dim=-1).float()
        for harm in range(1, n_h + 1):
            angle = 2.0 * math.pi * harm * k_idx / float(m)
            out[..., ai]     = torch.cos(angle)
            out[..., ai + 1] = torch.sin(angle)
            ai += 2
    return out


def prepare_tables(TN_oh, tau0_oh, TR, pool_ids, blocks):
    TN_ang   = convert_onehot_to_angular_multi(TN_oh, blocks)
    tau0_ang = convert_onehot_to_angular_multi(tau0_oh, blocks)
    tau0_hyb = torch.cat([tau0_ang, torch.ones(tau0_ang.shape[0], N_PHASE_PAIRS)], dim=1)
    return TN_ang, TR, tau0_hyb, pool_ids


def build_mod12_class_table(tau0_oh: torch.Tensor) -> torch.Tensor:
    return (tau0_oh[:, MOD12_S:MOD12_E].argmax(dim=-1) % VOCAB).long()


# ═══════════════════════════════════════════════════════════════════════
# Transport: harmonic-split only, parameterised by eps_high
# ═══════════════════════════════════════════════════════════════════════
def _apply_harmonic_split(
        base: torch.Tensor,       # (B, d_ang) weighted sum of neighbor reps
        tau_prev: torch.Tensor,   # (B, d_hyb) previous hybrid tau
        blocks,
        eps_high: float,
) -> torch.Tensor:
    """
    Harmonic-split transport:
      k=1 (fundamental): full transport (normalized by fundamental magnitude)
      k>=2 (harmonics):  tau_high_{t+1} = (1-eps_high)*transport_high + eps_high*tau_high_t

    Low-frequency transport is always unchanged.
    eps_high=0.0 → full transport for all harmonics (no freeze)
    eps_high=1.0 → full freeze of k>=2 components
    """
    ang_parts: List[torch.Tensor] = []
    mags:      List[torch.Tensor] = []
    ai = 0
    for _, _, _, n_h in blocks:
        fund = base[:, ai:ai + 2]
        mag  = fund.norm(dim=1, keepdim=True).clamp(1e-8)
        mags.append(mag)
        # k=1: full transport (normalised)
        ang_parts.append(fund / mag)
        # k>=2: blend transported with previous angular pair
        for h_idx in range(1, n_h):
            new_pair  = base[:, ai + h_idx * 2 : ai + h_idx * 2 + 2] / mag
            prev_pair = tau_prev[:, ai + h_idx * 2 : ai + h_idx * 2 + 2]
            blended   = (1.0 - eps_high) * new_pair + eps_high * prev_pair
            ang_parts.append(blended)
        ai += n_h * 2
    return torch.cat(ang_parts + mags, dim=1)


# ═══════════════════════════════════════════════════════════════════════
# Router
# ═══════════════════════════════════════════════════════════════════════
class RouterHarmonicThreshold(nn.Module):
    """
    Delayed-injection router with harmonic-split transport, eps_high sweep.

    Fixed: inject@last, b_pos0=0.0, b_posLast_init=2.0.
    Task:  x0 = mod12_initial_state % 4  (state-specific).
    Only eps_high varies across runs.
    """

    def __init__(self, TN_ang, TR, tau0_hyb, pool_ids,
                 blocks,
                 eps_high: float = 0.5,
                 init_scale: float = INIT_SCALE,
                 seed: int = GLOBAL_SEED):
        super().__init__()
        self.blocks   = blocks
        self.eps_high = eps_high
        d_ang, d_hyb, d_in = geom_dims(blocks)
        self.d_ang = d_ang
        self.d_hyb = d_hyb
        dh  = D_HIDDEN
        dha = max(8, dh // 4)

        self.register_buffer("TN",          TN_ang)
        self.register_buffer("TR",          TR)
        self.register_buffer("tau0_table",  tau0_hyb)
        self.register_buffer("pool_ids",    pool_ids)

        m0 = torch.zeros(1, D); m0[0, 0]     = 1.0
        mL = torch.zeros(1, D); mL[0, D - 1] = 1.0
        self.register_buffer("pos0_mask",    m0)
        self.register_buffer("posLast_mask", mL)

        self.b_pos0    = nn.Parameter(torch.tensor(0.0))
        self.b_posLast = nn.Parameter(torch.tensor(float(B_POSLAST_INIT)))

        gen = torch.Generator().manual_seed(seed)
        def rp(*sh): return nn.Parameter(torch.empty(*sh).normal_(0, init_scale, generator=gen))
        def zp(*sh): return nn.Parameter(torch.zeros(*sh))

        self.W_emb        = rp(VOCAB, D_EMB)
        self.W1           = rp(d_in,  dh);    self.b1     = zp(dh)
        self.W2           = rp(dh,    N_OPS); self.b2     = zp(N_OPS)
        self.W_attn       = rp(dha,   d_hyb); self.b_attn = zp(dha); self.v_attn = rp(dha)
        self.W_pred       = rp(d_hyb, VOCAB); self.b_pred = zp(VOCAB)
        self.W_tok_inject = rp(VOCAB, d_hyb)

    def _inject(self, hybrid: torch.Tensor, x0: torch.Tensor, t: int) -> torch.Tensor:
        if t == D - 1:
            return hybrid + self.W_tok_inject[x0]
        return hybrid

    def _soft_step(self, tau_prev, state_ids, tokens_t, temp):
        tn   = self.TN[state_ids]
        embs = self.W_emb[tokens_t]
        h    = torch.tanh(torch.cat([embs, tau_prev], dim=1) @ self.W1 + self.b1)
        logits = h @ self.W2 + self.b2
        if self.training:
            u = torch.rand_like(logits).clamp_(1e-20, 1.0)
            w = torch.softmax((logits - torch.log(-torch.log(u))) / temp, dim=1)
        else:
            w = torch.softmax(logits / 0.05, dim=1)
        base   = torch.einsum("bi,bij->bj", w, tn)
        hybrid = _apply_harmonic_split(base, tau_prev, self.blocks, self.eps_high)
        new_sids = self.TR[state_ids].gather(
            1, torch.argmax(w, dim=1).unsqueeze(1)).squeeze(1)
        return hybrid, new_sids

    def _eval_transport(self, tau_prev, best_ang) -> torch.Tensor:
        return _apply_harmonic_split(best_ang, tau_prev, self.blocks, self.eps_high)

    def forward(self, state_ids, tokens, x0, temp):
        tau_prev = self.tau0_table[state_ids]
        soft_taus: List[torch.Tensor] = []
        for t in range(D):
            hybrid, state_ids = self._soft_step(tau_prev, state_ids, tokens[:, t], temp)
            tau_prev = self._inject(hybrid, x0, t)
            soft_taus.append(tau_prev)
        st    = torch.stack(soft_taus, dim=1)
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = ((h_a * self.v_attn).sum(dim=-1)
                 + self.pos0_mask * self.b_pos0
                 + self.posLast_mask * self.b_posLast)
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred

    def readout(self, st: torch.Tensor):
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = ((h_a * self.v_attn).sum(dim=-1)
                 + self.pos0_mask * self.b_pos0
                 + self.posLast_mask * self.b_posLast)
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred, alpha

    def readout_masked(self, st: torch.Tensor, mask: torch.Tensor):
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = ((h_a * self.v_attn).sum(dim=-1)
                 + self.pos0_mask * self.b_pos0
                 + self.posLast_mask * self.b_posLast
                 + mask)
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred, alpha


# ═══════════════════════════════════════════════════════════════════════
# Batch / eval helpers
# ═══════════════════════════════════════════════════════════════════════
def make_batch(pool_ids, mod12_classes, rng):
    idx  = torch.randint(pool_ids.shape[0], (BATCH_SIZE,), generator=rng)
    sids = pool_ids[idx]
    x0   = mod12_classes[sids]
    toks = torch.randint(VOCAB, (BATCH_SIZE, D), generator=rng)
    toks[:, 0] = x0
    return sids, toks, x0


def eval_acc(model, pool_ids, mod12_classes) -> Tuple[float, float]:
    """Returns (accuracy, mean_alpha_{D-1})."""
    model.eval()
    rng     = torch.Generator().manual_seed(GLOBAL_SEED + 200)
    correct = 0; aD_sum = 0.0
    B = BATCH_SIZE
    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, mod12_classes, rng)
            tau_prev  = model.tau0_table[sids]
            sids_loop = sids.clone()
            taus: List[torch.Tensor] = []
            for t in range(D):
                tn      = model.TN[sids_loop]
                cur_dir = tau_prev[:, :model.d_ang]
                ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)
                best_op = ang_sim.argmax(dim=1)
                best_ang = tn.gather(
                    1, best_op.view(B, 1, 1).expand(B, 1, model.d_ang)).squeeze(1)
                hybrid   = model._eval_transport(tau_prev, best_ang)
                tau_cur  = model._inject(hybrid, x0, t)
                taus.append(tau_cur)
                tau_prev  = tau_cur
                sids_loop = model.TR[sids_loop].gather(
                    1, best_op.unsqueeze(1)).squeeze(1)
            st = torch.stack(taus, dim=1)
            pred, alpha = model.readout(st)
            correct += (pred.argmax(1) == x0).sum().item()
            aD_sum  += alpha[:, D - 1].mean().item()
    model.train()
    return round(correct / N_EVAL, 4), round(aD_sum / N_BATCHES, 4)


# ═══════════════════════════════════════════════════════════════════════
# Trajectory collection + no_last ablation
# ═══════════════════════════════════════════════════════════════════════
def collect_trajectories(model, pool_ids, mod12_classes):
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 888)
    all_st:   List[torch.Tensor] = []
    all_x0:   List[torch.Tensor] = []
    B = BATCH_SIZE
    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, mod12_classes, rng)
            all_x0.append(x0)
            tau_prev  = model.tau0_table[sids]
            sids_loop = sids.clone()
            taus: List[torch.Tensor] = []
            for t in range(D):
                tn      = model.TN[sids_loop]
                cur_dir = tau_prev[:, :model.d_ang]
                ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)
                best_op = ang_sim.argmax(dim=1)
                best_ang = tn.gather(
                    1, best_op.view(B, 1, 1).expand(B, 1, model.d_ang)).squeeze(1)
                hybrid   = model._eval_transport(tau_prev, best_ang)
                tau_cur  = model._inject(hybrid, x0, t)
                taus.append(tau_cur.clone())
                tau_prev  = tau_cur
                sids_loop = model.TR[sids_loop].gather(
                    1, best_op.unsqueeze(1)).squeeze(1)
            all_st.append(torch.stack(taus, dim=1))
    return all_st, all_x0


def eval_no_last(model, all_st, all_x0) -> float:
    """Accuracy with t=D-1 attention-masked (routing-only; no injection shortcut)."""
    mask    = torch.zeros(1, D); mask[0, D - 1] = NEG_INF
    correct = 0
    with torch.no_grad():
        for st, x0 in zip(all_st, all_x0):
            pred, _ = model.readout_masked(st, mask)
            correct += (pred.argmax(1) == x0).sum().item()
    return round(correct / N_EVAL, 4)


# ═══════════════════════════════════════════════════════════════════════
# Decodability probes
# ═══════════════════════════════════════════════════════════════════════
def linear_probe(X: np.ndarray, y: np.ndarray, label: str) -> float:
    import warnings
    scaler = StandardScaler()
    Xs     = scaler.fit_transform(X)
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        clf = LogisticRegression(max_iter=2000, C=1.0, solver="lbfgs")
        clf.fit(Xs, y)
    acc = float((clf.predict(Xs) == y).mean())
    print(f"      [{label}] decodability={acc:.4f}  (n={X.shape[0]}, d={X.shape[1]})")
    return round(acc, 4)


def run_decodability_probes(model, pool_ids, mod12_classes, label: str) -> Dict[str, float]:
    """Linear decodability of mod-12 class at initial / mid (t=D//2-1) / final."""
    rng  = torch.Generator().manual_seed(GLOBAL_SEED + 777)
    MID  = D // 2 - 1   # t=11 for D=24
    B    = BATCH_SIZE
    n_b  = (N_PROBE + B - 1) // B

    tau0_list:  List[torch.Tensor] = []
    tau_m_list: List[torch.Tensor] = []
    tau_f_list: List[torch.Tensor] = []
    lbl_list:   List[torch.Tensor] = []

    with torch.no_grad():
        for _ in range(n_b):
            sids, _, x0 = make_batch(pool_ids, mod12_classes, rng)
            tau0_list.append(model.tau0_table[sids].cpu())
            lbl_list.append(x0.cpu())
            tau_prev  = model.tau0_table[sids]
            sids_loop = sids.clone()
            for t in range(D):
                tn      = model.TN[sids_loop]
                cur_dir = tau_prev[:, :model.d_ang]
                ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)
                best_op = ang_sim.argmax(dim=1)
                best_ang = tn.gather(
                    1, best_op.view(B, 1, 1).expand(B, 1, model.d_ang)).squeeze(1)
                hybrid  = model._eval_transport(tau_prev, best_ang)
                tau_cur = model._inject(hybrid, x0, t)
                if t == MID:
                    tau_m_list.append(tau_cur.cpu())
                if t == D - 1:
                    tau_f_list.append(tau_cur.cpu())
                tau_prev  = tau_cur
                sids_loop = model.TR[sids_loop].gather(
                    1, best_op.unsqueeze(1)).squeeze(1)

    y        = torch.cat(lbl_list,   dim=0).numpy()
    tau0_arr = torch.cat(tau0_list,  dim=0).numpy()
    tau_m    = torch.cat(tau_m_list, dim=0).numpy()
    tau_f    = torch.cat(tau_f_list, dim=0).numpy()

    return {
        "initial": linear_probe(tau0_arr, y, f"{label}/initial"),
        "mid":     linear_probe(tau_m,    y, f"{label}/mid(t={MID})"),
        "final":   linear_probe(tau_f,    y, f"{label}/final(t={D-1})"),
    }


# ═══════════════════════════════════════════════════════════════════════
# Training
# ═══════════════════════════════════════════════════════════════════════
def train_one(
        variant_name: str,
        TN_ang, TR, tau0_hyb, pool_ids, mod12_classes,
        blocks, eps_high: float,
) -> Dict:
    d_ang, d_hyb, d_in = geom_dims(blocks)
    label = f"{variant_name}/fuller_k3"
    print(f"\n{'=' * 60}")
    print(f"  {label}")
    print(f"  d_tau_ang={d_ang}  d_tau_hyb={d_hyb}  eps_high={eps_high:.2f}")
    print(f"{'=' * 60}")

    model = RouterHarmonicThreshold(
        TN_ang, TR, tau0_hyb, pool_ids, blocks, eps_high=eps_high)
    opt   = torch.optim.SGD(model.parameters(), lr=LR)
    rng_t = torch.Generator().manual_seed(GLOBAL_SEED + 100)
    model.train()

    t0         = time.perf_counter()
    solve_step = None; final_acc = 0.0; alphaD_f = 0.0

    for step in range(1, MAX_STEPS + 1):
        frac = step / max(MAX_STEPS - 1, 1)
        temp = float(TEMP_START * (TEMP_END / TEMP_START) ** frac)
        sids, toks, x0 = make_batch(pool_ids, mod12_classes, rng_t)
        loss = F.cross_entropy(model(sids, toks, x0, temp), x0)
        opt.zero_grad(); loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), CLIP_GRAD)
        opt.step()
        if step % EVAL_EVERY == 0:
            acc, aD = eval_acc(model, pool_ids, mod12_classes)
            if acc >= SOLVE_ACC and solve_step is None:
                solve_step = step
            final_acc = acc; alphaD_f = aD
            print(f"    [{label}] step={step:5d}  acc={acc:.4f}  α_{{D-1}}={aD:.4f}")

    wall = round(time.perf_counter() - t0, 1)
    bL   = round(float(model.b_posLast.item()), 4)
    print(f"  → acc={final_acc:.4f}  solve={solve_step}  wall={wall}s  bL={bL}")
    model.eval()
    return dict(
        model=model, variant=variant_name, regime="fuller_k3",
        blocks=blocks, pool_ids=pool_ids, mod12_classes=mod12_classes,
        eps_high=eps_high,
        final_acc=final_acc, solve_step=solve_step,
        alphaD=alphaD_f, bL=bL, wall=wall,
    )


# ═══════════════════════════════════════════════════════════════════════
# Output writers
# ═══════════════════════════════════════════════════════════════════════
_FIELDNAMES = [
    "variant", "regime", "eps_high", "position",
    "decodability", "final_accuracy", "solve_step",
    "no_last_accuracy", "alpha_last", "runtime_seconds", "note",
]


def write_csv(rows: List[Dict]):
    with open(CSV_OUT, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=_FIELDNAMES)
        w.writeheader()
        for r in rows:
            w.writerow({k: r.get(k, "") for k in _FIELDNAMES})
    print(f"\nCSV written: {CSV_OUT}  ({len(rows)} rows)")


def _decay_at_mid(eps: float) -> str:
    """Expected fraction of initial k=3 signal remaining at t=D//2-1."""
    val = eps ** (D // 2 - 1)
    return f"{val:.4f}"


def write_markdown(results: List[Dict], threshold_verdict: str):
    ts = datetime.datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ")
    L: List[str] = []

    L.append("# Prime Transport Router: Harmonic Preservation Threshold v1\n\n")
    L.append(f"**Generated:** {ts}  \n")
    L.append(f"**Config:** D={D}, D_HIDDEN={D_HIDDEN}, B={BATCH_SIZE}, "
             f"N_eval={N_EVAL}, steps={MAX_STEPS}, LR={LR}, "
             f"b_posLast_init={B_POSLAST_INIT}  \n")
    L.append(f"**Regime:** `fuller_k3` only  \n")
    L.append(f"**Transport:** harmonic-split only — k=1 full transport; k≥2 freeze sweep  \n")
    L.append("**Task:** `target = mod12_initial_state % 4`  (state-specific x0)  \n\n")

    # ── Transport rule ──────────────────────────────────────────────
    L.append("---\n\n## Transport Rule\n\n")
    L.append("```\n")
    L.append("k=1 (low-freq):  tau_k1_{t+1}   = normalize(transport(tau_k1_t))\n")
    L.append("k≥2 (high-freq): tau_high_{t+1} = (1-eps_high)*transport_high + eps_high*tau_high_t\n")
    L.append("```\n\n")
    L.append("| eps_high | description | expected k=3 fraction at t=11 |\n")
    L.append("|----------|-------------|--------------------------------|\n")
    for vname, eps in EPS_HIGH_VALUES.items():
        desc = ("baseline harmonic-split (from v1)"
                if eps == 0.50 else
                "moderate freeze" if eps == 0.75 else
                "strong freeze" if eps == 0.90 else
                "full freeze — k≥2 components static")
        L.append(f"| {eps:.2f} | {desc} | {_decay_at_mid(eps)} |\n")
    L.append("\n")

    # ── Training results ────────────────────────────────────────────
    L.append("---\n\n## Training and Task Results\n\n")
    L.append("| variant | eps_high | accuracy | solve_step | α_{D-1} | no_last | runtime_s |\n")
    L.append("|---------|----------|----------|------------|---------|---------|----------|\n")
    for r in results:
        ss = str(r["solve_step"]) if r["solve_step"] else "—"
        L.append(f"| {r['variant']} | {r['eps_high']:.2f} | {r['final_acc']:.4f} | {ss}"
                 f" | {r['alphaD']:.4f} | {r['no_last']:.4f} | {r['wall']:.1f} |\n")
    L.append("\n")

    # ── Decodability table ──────────────────────────────────────────
    L.append("---\n\n## Decodability Progression (initial / mid / final)\n\n")
    L.append(f"**Locked reference (v1 baseline, fuller_k3):** "
             f"initial=1.0000, mid={BASELINE_MID_K3_REF:.4f} (near chance), final=1.0000  \n\n")
    L.append("| eps_high | initial | mid (t=11) | final (t=23) | Δ_mid_vs_baseline |\n")
    L.append("|----------|---------|------------|-------------|-------------------|\n")
    for r in results:
        p     = r.get("decodability", {})
        mid   = p.get("mid", 0.0)
        delta = round(mid - BASELINE_MID_K3_REF, 4)
        sign  = "+" if delta >= 0 else ""
        L.append(f"| {r['eps_high']:.2f} "
                 f"| {p.get('initial', 0.0):.4f} "
                 f"| {mid:.4f} "
                 f"| {p.get('final', 0.0):.4f} "
                 f"| {sign}{delta:.4f} |\n")
    L.append("\n")

    # ── no_last progression ─────────────────────────────────────────
    L.append("---\n\n## no_last Accuracy vs eps_high\n\n")
    L.append("| eps_high | no_last | final_accuracy | notes |\n")
    L.append("|----------|---------|----------------|-------|\n")
    baseline_nolast_ref = 0.2393  # locked v1 baseline/fuller_k3
    for r in results:
        delta_nl = round(r["no_last"] - baseline_nolast_ref, 4)
        sign_nl  = "+" if delta_nl >= 0 else ""
        frozen   = "full freeze" if r["eps_high"] == 1.0 else ""
        L.append(f"| {r['eps_high']:.2f} | {r['no_last']:.4f} "
                 f"| {r['final_acc']:.4f} "
                 f"| Δ_vs_baseline={sign_nl}{delta_nl:.4f}{' — ' + frozen if frozen else ''} |\n")
    L.append("\n")

    # ── Key questions ───────────────────────────────────────────────
    L.append("---\n\n## Key Questions\n\n")

    mid_vals = [(r["eps_high"], r.get("decodability", {}).get("mid", 0.0)) for r in results]
    mid_vals_sorted = sorted(mid_vals, key=lambda x: x[0])
    mids = [v for _, v in mid_vals_sorted]
    is_monotone = all(mids[i] <= mids[i+1] + 1e-4 for i in range(len(mids)-1))
    max_mid = max(mids) if mids else 0.0
    max_mid_eps = mid_vals_sorted[mids.index(max_mid)][0] if mids else 0.0

    L.append("**Q1: Does mid-trajectory decodability increase monotonically with eps_high?**  \n")
    L.append(f"Mid decodabilities by eps_high: "
             + ", ".join(f"{e:.2f}→{v:.4f}" for e, v in mid_vals_sorted) + ".  \n")
    if is_monotone:
        L.append("YES — decodability increases (weakly) with eps_high, consistent with "
                 "progressive harmonic preservation.  \n\n")
    else:
        L.append("NOT strictly monotone — decodability is non-monotone across eps_high values.  \n\n")

    L.append("**Q2: Is there a threshold where mid decodability is significantly above chance (>0.45)?**  \n")
    above_threshold = [(e, v) for e, v in mid_vals_sorted if v > 0.45]
    if above_threshold:
        L.append(f"YES — threshold crossed at eps_high={above_threshold[0][0]:.2f} "
                 f"(mid decodability={above_threshold[0][1]:.4f} > 0.45).  \n")
        L.append(f"Peak mid decodability: {max_mid:.4f} at eps_high={max_mid_eps:.2f}.  \n\n")
    else:
        L.append(f"NO — maximum mid decodability is {max_mid:.4f} "
                 f"(at eps_high={max_mid_eps:.2f}), which does not exceed 0.45.  \n")
        L.append("No threshold of meaningful signal preservation was found.  \n\n")

    strong_results = [(r["eps_high"], r["no_last"]) for r in results if r["eps_high"] >= 0.9]
    mid_nolast_ref = next((r["no_last"] for r in results if r["eps_high"] == 0.75), None)
    L.append("**Q3: Does strong preservation (eps_high ≥ 0.9) improve no_last accuracy further?**  \n")
    if strong_results:
        strong_str = ", ".join(f"eps_high={e:.2f}→{v:.4f}" for e, v in strong_results)
        ref_str    = f"eps_high=0.75→{mid_nolast_ref:.4f}" if mid_nolast_ref else "N/A"
        best_strong_nl = max(v for _, v in strong_results)
        ref_nl = mid_nolast_ref if mid_nolast_ref else 0.0
        L.append(f"Strong results ({strong_str}) vs moderate reference ({ref_str}).  \n")
        if best_strong_nl > ref_nl + 0.05:
            L.append("YES — strong preservation further improves no_last.  \n\n")
        elif best_strong_nl < ref_nl - 0.05:
            L.append("NO — strong preservation does NOT further improve no_last; "
                     "performance is flat or degraded.  \n\n")
        else:
            L.append("MARGINAL — strong preservation has minimal additional effect on no_last.  \n\n")
    else:
        L.append("N/A  \n\n")

    freeze_result = next((r for r in results if r["eps_high"] == 1.0), None)
    L.append("**Q4: Does freezing (eps_high = 1.0) break learning or still allow correct final accuracy?**  \n")
    if freeze_result:
        facc = freeze_result["final_acc"]
        ss   = freeze_result["solve_step"]
        if facc >= SOLVE_ACC:
            L.append(f"LEARNING IS INTACT — eps_high=1.0 achieves accuracy={facc:.4f} "
                     f"(solve_step={ss}).  \n")
            L.append("Full freeze of k≥2 components does not prevent the network from "
                     "learning via the injection mechanism.  \n\n")
        elif facc >= 0.9:
            L.append(f"LEARNING IS IMPAIRED — eps_high=1.0 achieves partial accuracy={facc:.4f} "
                     f"(solve_step={ss if ss else 'none'}).  \n")
            L.append("Full freeze significantly slows or limits learning.  \n\n")
        else:
            L.append(f"LEARNING IS BROKEN — eps_high=1.0 achieves only accuracy={facc:.4f} "
                     f"(solve_step={ss if ss else 'none'}).  \n")
            L.append("Full freeze of k≥2 prevents effective learning.  \n\n")
    else:
        L.append("N/A  \n\n")

    # ── Threshold verdict ───────────────────────────────────────────
    L.append("---\n\n## Threshold Identification\n\n")
    if max_mid > 0.7:
        L.append("**STRONG threshold detected:** mid-trajectory decodability exceeds 0.70, "
                 "indicating k=3 signal is substantially preserved at high eps_high values.  \n\n")
    elif max_mid > 0.45:
        L.append("**PARTIAL threshold detected:** mid-trajectory decodability exceeds chance "
                 "(>0.45) at high eps_high values, but full preservation (>0.70) is not achieved.  \n\n")
    else:
        L.append("**NO threshold detected:** mid-trajectory decodability remains near chance "
                 "(<0.45) even at eps_high=1.0, indicating the current transport family is "
                 "fundamentally incompatible with harmonic signal preservation.  \n\n")

    L.append("**Summary interpretation:**  \n\n")

    # Whether transport can be made signal-preserving
    if max_mid > 0.45:
        L.append("- Transport CAN be made partially signal-preserving by increasing eps_high.  \n")
    else:
        L.append("- Transport CANNOT be made signal-preserving by eps_high tuning alone.  \n")
        L.append("  The harmonic collapse is not prevented by any tested freeze level.  \n")

    # Whether freezing harms or helps learning
    if freeze_result:
        facc = freeze_result["final_acc"]
        if facc >= SOLVE_ACC:
            L.append("- Full freeze (eps_high=1.0) does NOT harm final-step learning "
                     "(acc=1.0 maintained). The injection mechanism compensates.  \n")
        else:
            L.append(f"- Full freeze (eps_high=1.0) HARMS learning (acc={facc:.4f}).  \n")

    L.append("\n")

    L.append("---\n\n")
    L.append(f"## HARMONIC PRESERVATION THRESHOLD IS: {threshold_verdict}\n\n")

    with open(MD_OUT, "w") as f:
        f.writelines(L)
    print(f"Markdown written: {MD_OUT}")


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main():
    print("=" * 60)
    print("HARMONIC PRESERVATION THRESHOLD — HIGH-FREQUENCY FREEZE SPECTRUM v1")
    print("=" * 60)
    print(f"Transport:  harmonic-split (k=1 full; k>=2 freeze sweep)")
    print(f"Regime:     fuller_k3 only")
    print(f"eps_high:   {list(EPS_HIGH_VALUES.values())}")
    print(f"Task:       x0 = mod12_initial_state % 4  (NOT random)")
    print(f"Fixed:      inject=last  b_posLast_init={B_POSLAST_INIT}  steps={MAX_STEPS}\n")

    print("Loading cache ...")
    t_load = time.perf_counter()
    data   = torch.load(CACHE_PATH, weights_only=False)
    TN_oh  = data["TN_oh"]; tau0_oh = data["tau0_oh"]
    TR     = data["TR"];    pool_ids = data["pool_ids"]
    print(f"  {TN_oh.shape[0]:,} states in {time.perf_counter()-t_load:.3f}s")

    mod12_classes = build_mod12_class_table(tau0_oh)
    TN_ang, TR_t, tau0_hyb, pool_t = prepare_tables(
        TN_oh, tau0_oh, TR, pool_ids, GEOM_K3)

    # ── Train all eps_high values ─────────────────────────────────
    all_results: List[Dict] = []

    for variant_name, eps_high in EPS_HIGH_VALUES.items():
        r = train_one(
            variant_name,
            TN_ang, TR_t, tau0_hyb, pool_t, mod12_classes,
            GEOM_K3, eps_high=eps_high,
        )
        all_results.append(r)

    # ── Post-training measurements ────────────────────────────────
    print(f"\n{'=' * 60}")
    print("POST-TRAINING MEASUREMENTS")
    print("=" * 60)

    for r in all_results:
        label = f"{r['variant']}/fuller_k3"
        model     = r["model"]
        pool_r    = r["pool_ids"]
        mod12_cls = r["mod12_classes"]

        print(f"\n  [{label}] collecting trajectories ...")
        all_st, all_x0 = collect_trajectories(model, pool_r, mod12_cls)
        r["no_last"] = eval_no_last(model, all_st, all_x0)
        print(f"    no_last = {r['no_last']:.4f}")

        print(f"  [{label}] running decodability probes ...")
        r["decodability"] = run_decodability_probes(model, pool_r, mod12_cls, label)

    # ── Summary ──────────────────────────────────────────────────
    print(f"\n{'=' * 60}")
    print("SUMMARY")
    print("=" * 60)
    print(f"  {'variant':<22} {'eps':>5} {'acc':>6} {'solve':>6} "
          f"{'no_last':>8} {'dec_init':>9} {'dec_mid':>8} {'dec_final':>10}")
    for r in all_results:
        p    = r.get("decodability", {})
        ss   = str(r["solve_step"]) if r["solve_step"] else "—"
        print(f"  {r['variant']:<22} {r['eps_high']:>5.2f} {r['final_acc']:>6.4f} {ss:>6} "
              f"{r['no_last']:>8.4f} "
              f"{p.get('initial', 0.0):>9.4f} "
              f"{p.get('mid', 0.0):>8.4f} "
              f"{p.get('final', 0.0):>10.4f}")

    # ── Determine threshold verdict ───────────────────────────────
    max_mid = max(
        r.get("decodability", {}).get("mid", 0.0) for r in all_results)

    if max_mid > 0.7:
        verdict = "STRONG"
    elif max_mid > 0.45:
        verdict = "PARTIAL"
    else:
        verdict = "NONE"

    print(f"\nHARMONIC PRESERVATION THRESHOLD IS: {verdict}")
    print(f"  (max mid decodability = {max_mid:.4f}; "
          f"threshold >0.45=PARTIAL, >0.7=STRONG)")

    # ── Build CSV rows ────────────────────────────────────────────
    csv_rows: List[Dict] = []
    for r in all_results:
        p = r.get("decodability", {})
        for pos_name in ("initial", "mid", "final"):
            csv_rows.append({
                "variant":          r["variant"],
                "regime":           r["regime"],
                "eps_high":         r["eps_high"],
                "position":         pos_name,
                "decodability":     p.get(pos_name, ""),
                "final_accuracy":   r["final_acc"],
                "solve_step":       r["solve_step"] if r["solve_step"] else "",
                "no_last_accuracy": r["no_last"],
                "alpha_last":       r["alphaD"],
                "runtime_seconds":  r["wall"],
                "note": (f"harmonic-split eps_high={r['eps_high']:.2f}; "
                         f"k>=2 freeze; fully frozen" if r["eps_high"] == 1.0
                         else f"harmonic-split eps_high={r['eps_high']:.2f}; "
                              f"k>=2 blended"),
            })

    write_csv(csv_rows)
    write_markdown(all_results, verdict)


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""run_router_r4_retest_under_delayed_trainable_v1.py

FULLER-GEOMETRY FAIR RETEST UNDER TRAINABLE DELAYED REGIME v1

Objective: Determine whether restoring the richer coupled 4-factor
angular+radial geometry matters, now that delayed injection is trainable
via the b_posLast bootstrap mechanism.

LOCKED FINDINGS:
  1. inject@last + b_posLast=2.0 rescues training → acc=1.0000, solve@2500
  2. tau0_direct=0.259 (chance): τ₀ non-sufficient confirmed
  3. no_tau0=1.000: trajectory sufficient without τ₀
  4. last_only=1.000: t=D-1 carries full answer
  5. no_last=0.260: collapse without t=D-1
  6. random_t_gt0=0.274: t>0 taus necessary
  7. The previous R4 restoration failure was cold-start bootstrapping, NOT geometry

FAIR RETEST DESIGN:
  Both regimes use:  inject@last, b_posLast_init=2.0 (b_pos0=0.0), dynamic radial

  Regime A — DELAYED TRAINABLE BASELINE (reduced hybrid):
    PHASE_BLOCKS = [(0,2,2,1),(2,7,5,1),(7,9,2,1),(9,21,12,1)]  # 1 harmonic per block
    D_TAU_ANG = 2+2+2+2 = 8; D_TAU_HYB = 8+4 = 12; D_IN_HYB = 4+12 = 16

  Regime B — RESTORED FULLER GEOMETRY:
    PHASE_BLOCKS = [(0,2,2,1),(2,7,5,2),(7,9,2,1),(9,21,12,3)]  # extra harmonics
    Block 1 (mod=5):  k=1,2 → 4 angular dims  (gcd(2,5)=1: fully separating)
    Block 3 (mod=12): k=1,2,3 → 6 angular dims (captures cyclic substructures)
    D_TAU_ANG = 2+4+2+6 = 14; D_TAU_HYB = 14+4 = 18; D_IN_HYB = 4+18 = 22

  "Fuller geometry" meaning: NOT flat R^4, but richer coupled 4-factor cos/sin
  angular representation that preserves more of the modular arithmetic structure
  already present in the data (mod 5 and mod 12 groups have non-trivial subgroup
  structure that a single harmonic cannot capture).

ABLATIONS (for each regime, on hard-geom trajectories):
  full, tau0_direct, no_tau0, last_only, no_last
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
CSV_OUT     = RESULTS_DIR / "prime_transport_router_r4_retest_under_delayed_trainable_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_router_r4_retest_under_delayed_trainable_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Locked base config (shared by both regimes)
# ═══════════════════════════════════════════════════════════════════════
GLOBAL_SEED   = 42
D             = 24
D_HIDDEN      = 32
BATCH_SIZE    = 512
VOCAB         = 4
D_EMB         = 4
N_PHASE_PAIRS = 4       # always 4 blocks
N_OPS         = 6
LR            = 0.02
TEMP_START    = 2.0
TEMP_END      = 0.1
CLIP_GRAD     = 1.0
EVAL_EVERY    = 500
N_EVAL        = 2048
SOLVE_ACC     = 0.999
INIT_SCALE    = 0.05
MAX_STEPS     = 3_500
N_BATCHES     = N_EVAL // BATCH_SIZE
NEG_INF       = -1e9
B_POSLAST_INIT = 2.0    # confirmed rescuer from bootstrap probe

# ═══════════════════════════════════════════════════════════════════════
# Geometry definitions
# ═══════════════════════════════════════════════════════════════════════
# Each entry: (start_in_onehot, end_in_onehot, modulus, n_harmonics)
GEOM_REDUCED = [(0, 2, 2, 1), (2, 7, 5, 1), (7, 9, 2, 1), (9, 21, 12, 1)]
GEOM_FULLER  = [(0, 2, 2, 1), (2, 7, 5, 2), (7, 9, 2, 1), (9, 21, 12, 3)]

def geom_dims(blocks):
    """Return (d_tau_ang, d_tau_hyb, d_in_hyb)."""
    d_ang = sum(2 * n_h for (_, _, _, n_h) in blocks)
    d_hyb = d_ang + N_PHASE_PAIRS   # +4 radial magnitudes (one per block)
    d_in  = D_EMB + d_hyb
    return d_ang, d_hyb, d_in


try:
    sys.path.insert(0, str(SCRIPT_DIR))
    from thread_policy import select_threads as _st
    _st(BATCH_SIZE, geom_dims(GEOM_REDUCED)[2], D_HIDDEN)
except Exception:
    pass


# ═══════════════════════════════════════════════════════════════════════
# Angular conversion (multi-harmonic)
# ═══════════════════════════════════════════════════════════════════════
def convert_onehot_to_angular_multi(onehot: torch.Tensor, blocks) -> torch.Tensor:
    """
    Convert onehot state representation to multi-harmonic angular.
    blocks: list of (s, e, modulus, n_harmonics)
    """
    shape = onehot.shape[:-1]
    d_ang = sum(2 * n_h for (_, _, _, n_h) in blocks)
    out = torch.zeros(*shape, d_ang)
    ai = 0
    for s, e, m, n_h in blocks:
        k_idx = onehot[..., s:e].argmax(dim=-1).float()  # integer state 0..m-1
        for harm in range(1, n_h + 1):
            angle = 2.0 * math.pi * harm * k_idx / float(m)
            out[..., ai]     = torch.cos(angle)
            out[..., ai + 1] = torch.sin(angle)
            ai += 2
    return out


def prepare_tables_multi(TN_oh, tau0_oh, TR, pool_ids, blocks):
    """Build multi-harmonic hybrid tables for a given geometry."""
    TN_ang   = convert_onehot_to_angular_multi(TN_oh, blocks)   # (N, 6, d_ang)
    tau0_ang = convert_onehot_to_angular_multi(tau0_oh, blocks)  # (N, d_ang)
    # Initial radial magnitudes = 1.0 (one per phase pair)
    tau0_hyb = torch.cat([tau0_ang,
                          torch.ones(tau0_ang.shape[0], N_PHASE_PAIRS)], dim=1)
    return TN_ang, TR, tau0_hyb, pool_ids


# ═══════════════════════════════════════════════════════════════════════
# Router (geometry-agnostic, delayed injection, b_posLast enabled)
# ═══════════════════════════════════════════════════════════════════════
class RouterFairRetest(nn.Module):
    """
    Delayed-injection router with trainable b_posLast.
    Accepts arbitrary geometry dims to support both reduced and fuller geometry.

    Fixed across both regimes:
      inject_position = "last"
      b_pos0_init     = 0.0
      b_posLast_init  = B_POSLAST_INIT (2.0)
      radial          = dynamic
    """
    def __init__(self, TN_ang, TR, tau0_hyb, pool_ids,
                 blocks,
                 init_scale: float = INIT_SCALE,
                 seed: int = GLOBAL_SEED):
        super().__init__()
        self.blocks = blocks
        d_ang, d_hyb, d_in = geom_dims(blocks)
        self.d_ang = d_ang
        self.d_hyb = d_hyb
        dh = D_HIDDEN; dha = max(8, dh // 4)

        self.register_buffer("TN",         TN_ang)
        self.register_buffer("TR",         TR)
        self.register_buffer("tau0_table", tau0_hyb)
        self.register_buffer("pool_ids",   pool_ids)

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
        self.W1           = rp(d_in, dh);      self.b1 = zp(dh)
        self.W2           = rp(dh, N_OPS);     self.b2 = zp(N_OPS)
        self.W_attn       = rp(dha, d_hyb);    self.b_attn = zp(dha); self.v_attn = rp(dha)
        self.W_pred       = rp(d_hyb, VOCAB);  self.b_pred = zp(VOCAB)
        self.W_tok_inject = rp(VOCAB, d_hyb)

    def _inject(self, hybrid: torch.Tensor, x0: torch.Tensor, t: int) -> torch.Tensor:
        if t == D - 1:
            return hybrid + self.W_tok_inject[x0]
        return hybrid

    def _soft_step(self, tau_prev, state_ids, tokens_t, temp):
        """One Gumbel-softmax routing step → (hybrid, new_state_ids)."""
        tn   = self.TN[state_ids]
        embs = self.W_emb[tokens_t]
        h    = torch.tanh(torch.cat([embs, tau_prev], dim=1) @ self.W1 + self.b1)
        logits = h @ self.W2 + self.b2
        if self.training:
            u = torch.rand_like(logits).clamp_(1e-20, 1.0)
            w = torch.softmax((logits - torch.log(-torch.log(u))) / temp, dim=1)
        else:
            w = torch.softmax(logits / 0.05, dim=1)
        B    = state_ids.shape[0]
        base = torch.einsum("bi,bij->bj", w, tn)   # (B, d_ang)
        # Per-block normalization: normalize the fundamental (first 2 dims per block)
        # and compute magnitude from it; higher harmonics ride along unnormalized.
        ang_parts = []
        mags      = []
        ai = 0
        for _, _, _, n_h in self.blocks:
            fund = base[:, ai:ai+2]                          # fundamental
            mag  = fund.norm(dim=1, keepdim=True).clamp(1e-8)
            mags.append(mag)
            # Normalize fundamental; scale higher harmonics by same factor
            for h_idx in range(n_h):
                pair = base[:, ai + h_idx*2 : ai + h_idx*2 + 2]
                ang_parts.append(pair / mag)
            ai += n_h * 2
        dirn   = torch.cat(ang_parts, dim=1)               # (B, d_ang)  normalized
        radial = torch.cat(mags, dim=1)                    # (B, N_PHASE_PAIRS)  one per block
        hybrid = torch.cat([dirn, radial], dim=1)          # (B, d_hyb)
        new_sids = self.TR[state_ids].gather(
            1, torch.argmax(w, dim=1).unsqueeze(1)).squeeze(1)
        return hybrid, new_sids

    def forward(self, state_ids, tokens, x0, temp):
        tau_prev = self.tau0_table[state_ids]
        soft_taus: List[torch.Tensor] = []
        for t in range(D):
            hybrid, state_ids = self._soft_step(
                tau_prev, state_ids, tokens[:, t], temp)
            tau_prev = self._inject(hybrid, x0, t)
            soft_taus.append(tau_prev)
        st    = torch.stack(soft_taus, dim=1)             # (B, D, d_hyb)
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
        pred  = torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred
        return pred, alpha

    def readout_masked(self, st: torch.Tensor, attn_mask: torch.Tensor):
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = ((h_a * self.v_attn).sum(dim=-1)
                 + self.pos0_mask * self.b_pos0
                 + self.posLast_mask * self.b_posLast
                 + attn_mask)
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


def eval_acc(model, pool_ids) -> Tuple[float, float, float]:
    """Returns (accuracy, mean_alpha_0, mean_alpha_{D-1})."""
    model.eval()
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 200)
    correct = 0; a0_sum = 0.0; aD_sum = 0.0
    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, rng)
            logits = model(sids, toks, x0, 0.05)
            correct += (logits.argmax(1) == x0).sum().item()
            # Recompute alpha via hard-geom trajectory for tracking
            tau_prev  = model.tau0_table[sids]
            sids_loop = sids.clone()
            taus = []
            B = BATCH_SIZE
            for t in range(D):
                tn      = model.TN[sids_loop]
                cur_dir = tau_prev[:, :model.d_ang]
                ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)
                best_op = ang_sim.argmax(dim=1)
                best_ang = tn.gather(
                    1, best_op.view(B,1,1).expand(B,1,model.d_ang)
                ).squeeze(1)
                # Per-block normalization + dynamic radial
                hybrid  = _normalize_ang_and_radial(best_ang, model.blocks)
                tau_cur = model._inject(hybrid, x0, t)
                taus.append(tau_cur)
                tau_prev  = tau_cur
                sids_loop = model.TR[sids_loop].gather(
                    1, best_op.unsqueeze(1)).squeeze(1)
            st = torch.stack(taus, dim=1)
            _, alpha = model.readout(st)
            a0_sum += alpha[:, 0].mean().item()
            aD_sum += alpha[:, D-1].mean().item()
    model.train()
    return (round(correct / N_EVAL, 4),
            round(a0_sum / N_BATCHES, 4),
            round(aD_sum / N_BATCHES, 4))


def _normalize_ang_and_radial(ang: torch.Tensor, blocks) -> torch.Tensor:
    """Normalize per-block (using fundamental mag) and compute radial."""
    B = ang.shape[0]
    ang_parts = []
    mags      = []
    ai = 0
    for _, _, _, n_h in blocks:
        fund = ang[:, ai:ai+2]
        mag  = fund.norm(dim=1, keepdim=True).clamp(1e-8)
        mags.append(mag)
        for h_idx in range(n_h):
            pair = ang[:, ai + h_idx*2 : ai + h_idx*2 + 2]
            ang_parts.append(pair / mag)
        ai += n_h * 2
    dirn   = torch.cat(ang_parts, dim=1)
    radial = torch.cat(mags, dim=1)
    return torch.cat([dirn, radial], dim=1)


def collect_hard_geom_trajectories(model, pool_ids) -> Tuple[List, List]:
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 888)
    all_st, all_x0 = [], []
    B = BATCH_SIZE
    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, rng)
            tau_prev  = model.tau0_table[sids]
            sids_loop = sids.clone()
            taus: List[torch.Tensor] = []
            for t in range(D):
                tn      = model.TN[sids_loop]
                cur_dir = tau_prev[:, :model.d_ang]
                ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)
                best_op = ang_sim.argmax(dim=1)
                best_ang = tn.gather(
                    1, best_op.view(B,1,1).expand(B,1,model.d_ang)).squeeze(1)
                hybrid   = _normalize_ang_and_radial(best_ang, model.blocks)
                tau_cur  = model._inject(hybrid, x0, t)
                taus.append(tau_cur.clone())
                tau_prev  = tau_cur
                sids_loop = model.TR[sids_loop].gather(
                    1, best_op.unsqueeze(1)).squeeze(1)
            all_st.append(torch.stack(taus, dim=1))
            all_x0.append(x0)
    return all_st, all_x0


# ═══════════════════════════════════════════════════════════════════════
# Ablations
# ═══════════════════════════════════════════════════════════════════════
ABLATION_DEFS = [
    ("full",       "full trajectory"),
    ("tau0_direct","attention bypassed; pred = τ₀ @ W_pred + b_pred"),
    ("no_tau0",    "position 0 masked"),
    ("last_only",  f"only t={D-1}"),
    ("no_last",    f"t={D-1} masked"),
]


def apply_ablation(model, all_st, all_x0, ablation: str):
    t_start = time.perf_counter()
    correct = 0; a0_sum = 0.0; aD_sum = 0.0
    with torch.no_grad():
        for st, x0 in zip(all_st, all_x0):
            B = st.shape[0]
            if ablation == "full":
                pred, alpha = model.readout(st)
            elif ablation == "tau0_direct":
                pred  = st[:, 0, :] @ model.W_pred + model.b_pred
                alpha = torch.zeros(B, D); alpha[:, 0] = 1.0
            elif ablation == "no_tau0":
                mask = torch.zeros(1, D); mask[0, 0] = NEG_INF
                pred, alpha = model.readout_masked(st, mask)
            elif ablation == "last_only":
                mask = torch.full((1, D), NEG_INF); mask[0, D-1] = 0.0
                pred, alpha = model.readout_masked(st, mask)
            elif ablation == "no_last":
                mask = torch.zeros(1, D); mask[0, D-1] = NEG_INF
                pred, alpha = model.readout_masked(st, mask)
            else:
                raise ValueError(ablation)
            correct += (pred.argmax(1) == x0).sum().item()
            a0_sum  += alpha[:, 0].mean().item()
            aD_sum  += alpha[:, D-1].mean().item()
    wall = round(time.perf_counter() - t_start, 4)
    acc  = round(correct / N_EVAL, 4)
    return acc, round(a0_sum / N_BATCHES, 4), round(aD_sum / N_BATCHES, 4), wall


def _interpret(ablation: str, acc: float, full_acc: float) -> str:
    drop = round(acc - full_acc, 3)
    if ablation == "full":         return "reference"
    if ablation == "tau0_direct":
        if acc >= 0.999:           return "τ₀ encodes answer (shortcut present)"
        if acc < 0.35:             return "τ₀ does NOT encode answer ✓"
        return f"partial: {drop:+.3f}"
    if ablation == "no_tau0":
        if acc >= 0.999:           return "trajectory sufficient without τ₀ ✓"
        if acc < 0.35:             return "τ₀ is critical (collapse)"
        return f"{drop:+.3f}"
    if ablation == "last_only":
        if acc >= 0.999:           return "final position carries full answer ✓"
        if acc > 0.5:              return f"partial signal at t=D-1: {drop:+.3f}"
        return f"{drop:+.3f}"
    if ablation == "no_last":
        if acc < 0.35:             return "t=D-1 critical (collapse without it) ✓"
        if acc >= 0.999:           return "t=D-1 not critical"
        return f"{drop:+.3f}"
    return f"{drop:+.3f}"


# ═══════════════════════════════════════════════════════════════════════
# Training
# ═══════════════════════════════════════════════════════════════════════
def train_regime(blocks, regime_label: str,
                 TN_ang, TR, tau0_hyb, pool_ids):
    d_ang, d_hyb, d_in = geom_dims(blocks)
    print(f"\n{'='*60}")
    print(f"REGIME: {regime_label}")
    print(f"  geometry: d_tau_ang={d_ang}  d_tau_hyb={d_hyb}  d_in={d_in}")
    print(f"  inject=last  b_posLast_init={B_POSLAST_INIT}  b_pos0=0.0  radial=dynamic")
    print(f"{'='*60}")

    model = RouterFairRetest(TN_ang, TR, tau0_hyb, pool_ids, blocks=blocks)
    opt   = torch.optim.SGD(model.parameters(), lr=LR)
    rng_t = torch.Generator().manual_seed(GLOBAL_SEED + 100)
    model.train()

    t0 = time.perf_counter()
    solve_step = None; final_acc = 0.0; alpha0_f = 0.0; alphaD_f = 0.0

    for step in range(1, MAX_STEPS + 1):
        frac = step / max(MAX_STEPS - 1, 1)
        temp = float(TEMP_START * (TEMP_END / TEMP_START) ** frac)
        sids, toks, x0 = make_batch(pool_ids, rng_t)
        loss = F.cross_entropy(model(sids, toks, x0, temp), x0)
        opt.zero_grad(); loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), CLIP_GRAD)
        opt.step()
        if step % EVAL_EVERY == 0:
            acc, a0, aD = eval_acc(model, pool_ids)
            if acc >= SOLVE_ACC and solve_step is None:
                solve_step = step
            final_acc = acc; alpha0_f = a0; alphaD_f = aD
            print(f"    [{regime_label}] step={step:5d}  acc={acc:.4f}"
                  f"  α₀={a0:.4f}  α_{{D-1}}={aD:.4f}")

    wall = round(time.perf_counter() - t0, 1)
    sps  = round(MAX_STEPS / wall, 1)
    bL   = round(float(model.b_posLast.item()), 4)
    print(f"  {regime_label}: acc={final_acc:.4f}  solve={solve_step}"
          f"  sps={sps}  b_posLast={bL}"
          f"  α₀={alpha0_f:.4f}  α_{{D-1}}={alphaD_f:.4f}")
    model.eval()
    return model, final_acc, solve_step, sps, wall, alpha0_f, alphaD_f, bL


# ═══════════════════════════════════════════════════════════════════════
# Output writers
# ═══════════════════════════════════════════════════════════════════════
def write_csv(training_rows: List[Dict], ablation_rows: List[Dict]):
    fieldnames = [
        "regime", "geometry_regime", "ablation",
        "d_tau_ang", "d_tau_hyb",
        "accuracy", "delta_vs_baseline",
        "solve_step", "alpha0", "alphaD",
        "b_posLast_trained", "runtime_seconds", "note"
    ]
    with open(CSV_OUT, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fieldnames)
        w.writeheader()
        for r in training_rows + ablation_rows:
            w.writerow({k: r.get(k, "") for k in fieldnames})
    print(f"\nCSV written: {CSV_OUT}  ({len(training_rows)+len(ablation_rows)} rows)")


def write_markdown(training_rows: List[Dict], ablation_rows: List[Dict],
                   conclusion: str, answers: Dict):
    ts = datetime.datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ")
    L: List[str] = []
    L.append("# Prime Transport Router: Fuller-Geometry Fair Retest Under Trainable Delayed Regime v1\n\n")
    L.append(f"**Generated:** {ts}  \n")
    L.append(f"**Config:** D={D}, D_HIDDEN={D_HIDDEN}, B_train={BATCH_SIZE}, "
             f"N_eval={N_EVAL}, steps={MAX_STEPS}, LR={LR}, "
             f"b_posLast_init={B_POSLAST_INIT}  \n\n")

    L.append("## Definition of Restored Fuller Geometry\n\n")
    L.append("Both regimes use the same training setup (inject@last, b_posLast=2.0, dynamic radial).  \n")
    L.append("Only the angular/radial representation changes.  \n\n")
    L.append("| Component | Reduced Baseline | Restored Fuller |\n")
    L.append("|-----------|------------------|-----------------|\n")
    L.append("| PHASE_BLOCKS | [(0,2,2,1),(2,7,5,1),(7,9,2,1),(9,21,12,1)] | [(0,2,2,1),(2,7,5,**2**),(7,9,2,1),(9,21,12,**3**)] |\n")
    L.append("| Block 0 (mod=2) | 1 harmonic → 2 dims | 1 harmonic → 2 dims |\n")
    L.append("| Block 1 (mod=5) | 1 harmonic → 2 dims | **2 harmonics → 4 dims** (k=1,2; gcd(2,5)=1: fully separating) |\n")
    L.append("| Block 2 (mod=2) | 1 harmonic → 2 dims | 1 harmonic → 2 dims |\n")
    L.append("| Block 3 (mod=12) | 1 harmonic → 2 dims | **3 harmonics → 6 dims** (k=1,2,3: subgroup structure) |\n")
    L.append("| d_tau_ang | **8** | **14** |\n")
    L.append("| d_tau_hyb | **12** | **18** |\n")
    L.append("| d_in_hyb | **16** | **22** |\n")
    L.append("\n**Rationale:** The mod-5 and mod-12 groups have non-trivial subgroup structure "
             "that a single Fourier harmonic cannot capture. Adding k=2 for mod-5 fully distinguishes "
             "all 5 states; k=1,2,3 for mod-12 captures its cyclic substructures (Z₆, Z₄, Z₃, Z₂).  \n\n")

    L.append("---\n\n")
    L.append("## Training Results\n\n")
    L.append("| regime | geometry | d_tau_hyb | accuracy | solve_step | α₀ | α_{D-1} | b_posLast | runtime_s |\n")
    L.append("|--------|----------|-----------|----------|------------|----|---------|-----------|-----------|\n")
    for r in training_rows:
        ss = str(r["solve_step"]) if r["solve_step"] else "—"
        L.append(f"| {r['regime']} | {r['geometry_regime']} | {r['d_tau_hyb']} "
                 f"| {r['accuracy']:.4f} | {ss} "
                 f"| {r['alpha0']:.4f} | {r['alphaD']:.4f} "
                 f"| {r['b_posLast_trained']:.4f} | {r['runtime_seconds']:.1f} |\n")
    L.append("\n")

    L.append("---\n\n")
    L.append("## Ablation Results\n\n")
    L.append("| regime | ablation | accuracy | Δ_vs_full | α₀ | α_{D-1} | interpretation |\n")
    L.append("|--------|----------|----------|-----------|----|---------|----------------|\n")
    for r in ablation_rows:
        delta = round(r["accuracy"] - r["delta_vs_baseline"], 4)
        sign  = "+" if delta >= 0 else ""
        L.append(f"| {r['regime']} | {r['ablation']} | {r['accuracy']:.4f} "
                 f"| {sign}{delta:.4f} "
                 f"| {r['alpha0']:.4f} | {r['alphaD']:.4f} "
                 f"| {r['note']} |\n")
    L.append("\n")

    L.append("---\n\n")
    L.append("## Explicit Answers\n\n")
    for k, v in answers.items():
        L.append(f"**{k}**  \n{v}  \n\n")

    L.append("---\n\n")
    L.append("## Honesty Section\n\n")
    L.append(f"**What is now fairly tested:**  \n{answers.get('honest_tested', '')}  \n\n")
    L.append(f"**What remains unresolved:**  \n{answers.get('honest_unresolved', '')}  \n\n")
    L.append(f"**Whether previous dismissal was premature:**  \n{answers.get('honest_premature', '')}  \n\n")

    L.append("---\n\n")
    L.append(f"## FULLER GEOMETRY UNDER FAIR DELAYED TEST IS: {conclusion}\n\n")

    with open(MD_OUT, "w") as f:
        f.writelines(L)
    print(f"Markdown written: {MD_OUT}")


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main():
    print("=" * 60)
    print("FULLER-GEOMETRY FAIR RETEST UNDER TRAINABLE DELAYED REGIME v1")
    print("=" * 60)
    d_r  = geom_dims(GEOM_REDUCED)
    d_f  = geom_dims(GEOM_FULLER)
    print(f"Baseline (reduced):  d_tau_ang={d_r[0]}  d_tau_hyb={d_r[1]}  d_in={d_r[2]}")
    print(f"Fuller geometry:     d_tau_ang={d_f[0]}  d_tau_hyb={d_f[1]}  d_in={d_f[2]}")
    print(f"Both: inject=last  b_posLast_init={B_POSLAST_INIT}  b_pos0=0.0  steps={MAX_STEPS}")

    print("\nLoading cache ...")
    t0 = time.perf_counter()
    data = torch.load(CACHE_PATH, weights_only=False)
    N_states = data["TN_oh"].shape[0]
    print(f"  {N_states:,} states in {time.perf_counter()-t0:.3f}s")

    # Build tables for both geometries
    TN_oh = data["TN_oh"]; tau0_oh = data["tau0_oh"]
    TR = data["TR"]; pool_ids = data["pool_ids"]

    TN_r, TR_r, tau0_r, pid_r = prepare_tables_multi(TN_oh, tau0_oh, TR, pool_ids, GEOM_REDUCED)
    TN_f, TR_f, tau0_f, pid_f = prepare_tables_multi(TN_oh, tau0_oh, TR, pool_ids, GEOM_FULLER)

    training_rows: List[Dict] = []
    ablation_rows: List[Dict] = []

    # ── Regime A: delayed trainable baseline (reduced geometry) ──────────
    model_r, acc_r, solve_r, sps_r, wall_r, a0_r, aD_r, bL_r = train_regime(
        GEOM_REDUCED, "delayed_baseline", TN_r, TR_r, tau0_r, pid_r)

    training_rows.append(dict(
        regime="delayed_baseline", geometry_regime="reduced",
        ablation="training_summary",
        d_tau_ang=d_r[0], d_tau_hyb=d_r[1],
        accuracy=acc_r, delta_vs_baseline=0.0,
        solve_step=solve_r, alpha0=a0_r, alphaD=aD_r,
        b_posLast_trained=bL_r, runtime_seconds=wall_r,
        note="training result"
    ))

    # ── Regime B: restored fuller geometry ───────────────────────────────
    model_f, acc_f, solve_f, sps_f, wall_f, a0_f, aD_f, bL_f = train_regime(
        GEOM_FULLER, "delayed_fuller", TN_f, TR_f, tau0_f, pid_f)

    training_rows.append(dict(
        regime="delayed_fuller", geometry_regime="fuller",
        ablation="training_summary",
        d_tau_ang=d_f[0], d_tau_hyb=d_f[1],
        accuracy=acc_f, delta_vs_baseline=round(acc_f - acc_r, 4),
        solve_step=solve_f, alpha0=a0_f, alphaD=aD_f,
        b_posLast_trained=bL_f, runtime_seconds=wall_f,
        note="training result"
    ))

    # ── Ablations ─────────────────────────────────────────────────────────
    print(f"\n{'='*60}")
    print("ABLATIONS")
    print("=" * 60)

    for regime_label, model, blocks, TN_ang, TR_t, tau0_hyb, pool_t, acc_train in [
        ("delayed_baseline", model_r, GEOM_REDUCED, TN_r, TR_r, tau0_r, pid_r, acc_r),
        ("delayed_fuller",   model_f, GEOM_FULLER,  TN_f, TR_f, tau0_f, pid_f, acc_f),
    ]:
        geom_label = "reduced" if regime_label == "delayed_baseline" else "fuller"
        d_ang, d_hyb, d_in = geom_dims(blocks)
        print(f"\n  Collecting trajectories for {regime_label} ...")
        all_st, all_x0 = collect_hard_geom_trajectories(model, pool_t)

        full_acc = None
        for abl_name, abl_note in ABLATION_DEFS:
            abl_acc, abl_a0, abl_aD, abl_wall = apply_ablation(
                model, all_st, all_x0, abl_name)
            if full_acc is None:
                full_acc = abl_acc
            delta = round(abl_acc - (full_acc or abl_acc), 4)
            interp = _interpret(abl_name, abl_acc, full_acc or abl_acc)
            sign   = "+" if delta >= 0 else ""
            print(f"    [{abl_name:14s}] acc={abl_acc:.4f}  Δ={sign}{delta:.4f}"
                  f"  α₀={abl_a0:.4f}  α_{{D-1}}={abl_aD:.4f}  — {interp}")
            ablation_rows.append(dict(
                regime=regime_label, geometry_regime=geom_label,
                ablation=abl_name,
                d_tau_ang=d_ang, d_tau_hyb=d_hyb,
                accuracy=abl_acc, delta_vs_baseline=full_acc,
                solve_step="", alpha0=abl_a0, alphaD=abl_aD,
                b_posLast_trained="", runtime_seconds=abl_wall,
                note=interp
            ))

    # ── Summary ───────────────────────────────────────────────────────────
    print(f"\n{'='*60}")
    print("SUMMARY")
    print("=" * 60)
    print(f"  delayed_baseline  acc={acc_r:.4f}  solve={solve_r}  α_{{D-1}}={aD_r:.4f}"
          f"  d_tau_hyb={d_r[1]}")
    print(f"  delayed_fuller    acc={acc_f:.4f}  solve={solve_f}  α_{{D-1}}={aD_f:.4f}"
          f"  d_tau_hyb={d_f[1]}")
    delta = round(acc_f - acc_r, 4)
    print(f"  Δ(fuller - baseline) = {delta:+.4f}")

    # Determine conclusion
    tau0_r_acc = next((r["accuracy"] for r in ablation_rows
                       if r["regime"] == "delayed_baseline" and r["ablation"] == "tau0_direct"), None)
    tau0_f_acc = next((r["accuracy"] for r in ablation_rows
                       if r["regime"] == "delayed_fuller" and r["ablation"] == "tau0_direct"), None)
    notau0_f_acc = next((r["accuracy"] for r in ablation_rows
                         if r["regime"] == "delayed_fuller" and r["ablation"] == "no_tau0"), None)
    nolast_f_acc = next((r["accuracy"] for r in ablation_rows
                         if r["regime"] == "delayed_fuller" and r["ablation"] == "no_last"), None)

    # Is trajectory load-bearing in both?
    notau0_r = next((r["accuracy"] for r in ablation_rows
                     if r["regime"] == "delayed_baseline" and r["ablation"] == "no_tau0"), None)
    nolast_r = next((r["accuracy"] for r in ablation_rows
                     if r["regime"] == "delayed_baseline" and r["ablation"] == "no_last"), None)

    lb_baseline = (notau0_r is not None and notau0_r >= 0.99 and
                   nolast_r is not None and nolast_r < 0.5)
    lb_fuller   = (notau0_f_acc is not None and notau0_f_acc >= 0.99 and
                   nolast_f_acc is not None and nolast_f_acc < 0.5)

    # Determine conclusion
    if acc_f < 0.5 and acc_r >= SOLVE_ACC:
        conclusion = "IRRELEVANT"
        helpful_note = ("The restored fuller geometry failed to train under the same conditions "
                        "that the reduced baseline solves cleanly. Fuller geometry did not help.")
    elif acc_f >= SOLVE_ACC and acc_r >= SOLVE_ACC:
        if solve_f is not None and solve_r is not None and solve_f < solve_r - 500:
            conclusion = "HELPFUL"
            helpful_note = (f"Both regimes solve, but fuller geometry converges faster "
                            f"(solve@{solve_f} vs {solve_r}).")
        elif delta >= 0.01:
            conclusion = "HELPFUL"
            helpful_note = f"Fuller geometry achieves higher accuracy: Δ={delta:+.4f}."
        else:
            conclusion = "IRRELEVANT"
            helpful_note = (f"Both regimes solve with similar performance (Δ={delta:+.4f}). "
                            f"Fuller geometry provides no measurable benefit.")
    elif acc_f >= SOLVE_ACC and acc_r < SOLVE_ACC:
        conclusion = "ESSENTIAL"
        helpful_note = "Only fuller geometry solves; baseline fails."
    elif acc_f >= 0.5 and acc_r >= SOLVE_ACC:
        conclusion = "IRRELEVANT"
        helpful_note = (f"Baseline solves (acc={acc_r:.4f}); fuller geometry only reaches "
                        f"{acc_f:.4f}. Richer representation does not help under this budget.")
    else:
        conclusion = "IRRELEVANT"
        helpful_note = f"Neither regime solves cleanly. Δ={delta:+.4f}."

    answers = {
        "1. Is trajectory load-bearing in delayed_baseline?":
            (f"YES — no_tau0={notau0_r:.4f} (sufficient without τ₀); "
             f"no_last={nolast_r:.4f} (collapse without t=D-1)."
             if lb_baseline else
             f"no_tau0={notau0_r}  no_last={nolast_r}."),
        "2. Is trajectory load-bearing in delayed_fuller?":
            (f"YES — no_tau0={notau0_f_acc:.4f} (sufficient without τ₀); "
             f"no_last={nolast_f_acc:.4f} (collapse without t=D-1)."
             if lb_fuller else
             f"no_tau0={notau0_f_acc}  no_last={nolast_f_acc}."),
        "3. Is fuller geometry beneficial under the fair delayed test?":
            helpful_note,
        "4. Is τ₀ still non-sufficient in restored geometry?":
            (f"YES — tau0_direct={tau0_f_acc:.4f}≈chance." if tau0_f_acc is not None and tau0_f_acc < 0.4
             else f"tau0_direct={tau0_f_acc} — not clearly chance-level."),
        "5. Is previous dismissal of fuller geometry overturned?":
            ("YES — the previous dismissal was premature: the R4 restoration probe failed "
             "due to cold-start bootstrapping, not geometry. This fair retest settles the question."
             if acc_f >= SOLVE_ACC else
             "CONFIRMED — even under fair trainable conditions, fuller geometry does not improve results."),
        "honest_tested":
            "Both regimes use identical inject@last + b_posLast=2.0 setup. "
            "The sole variable is the angular/radial representation (single vs multi-harmonic). "
            "This is the first fair test of richer geometry under trainable delayed injection.",
        "honest_unresolved":
            "Whether even richer geometry (e.g., more harmonics, higher D_HIDDEN) "
            "would change the result at larger D remains untested.",
        "honest_premature":
            ("YES — the previous R4 probe concluded 'INDETERMINATE (training convergence failure)'. "
             "That failure was cold-start bootstrapping, not a property of fuller geometry. "
             "This probe gives the definitive answer."
             if acc_f >= SOLVE_ACC else
             "NO — the previous probe was indeterminate for the right reason (bootstrapping failure), "
             "and this fair retest confirms fuller geometry is not beneficial here."),
    }

    print(f"\nFULLER GEOMETRY UNDER FAIR DELAYED TEST IS: {conclusion}")

    write_csv(training_rows, ablation_rows)
    write_markdown(training_rows, ablation_rows, conclusion, answers)


if __name__ == "__main__":
    main()

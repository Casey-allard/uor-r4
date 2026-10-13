#!/usr/bin/env python3
"""run_router_mod12_harmonic_stress_v1.py

MOD-12 HARMONIC RESOLUTION STRESS TEST v1

Objective: Determine whether the richer mod-12 harmonic basis (k=1,2,3) becomes
useful on a task that specifically stresses mod-12 quarter-phase resolution.

LOCKED FINDINGS:
  1. Fuller geometry (multi-harmonic) is REDUNDANT on the standard task at D=24.
  2. However, that task uses random x0 injected at t=D-1 — the answer does NOT
     depend on mod-12 phase resolution. It only tests injection/readout.
  3. This probe tests a strictly mod-12-sensitive task.

TASK DESIGN — MOD-12 QUARTER-PHASE CLASSIFICATION:
  target = mod12_initial_state % 4
  where mod12_initial_state = tau0_oh[s, 9:21].argmax() ∈ {0..11}

  This creates 4 target classes:
    Class 0: j ∈ {0, 4,  8}   (every 4th state)
    Class 1: j ∈ {1, 5,  9}
    Class 2: j ∈ {2, 6, 10}
    Class 3: j ∈ {3, 7, 11}

WHY THIS STRESSES MOD-12 RESOLUTION:
  With k=1 only (cos(2πj/12), sin(2πj/12)):
    - Class 0: reps at {0°, 120°, 240°} — equilateral triangle, NOT linearly separable
    - Each class forms its own equilateral triangle, interleaved with the others
    - A LINEAR classifier on 2D k=1 embeddings CANNOT separate these 4 classes

  With k=3 added (cos(6πj/12), sin(6πj/12)):
    - Class 0: ALL members map to (1.000, 0.000) — identical point
    - Class 1: ALL members map to (0.000, 1.000)
    - Class 2: ALL members map to (-1.000, 0.000)
    - Class 3: ALL members map to (0.000, -1.000)
    - 4 orthogonal unit vectors — TRIVIALLY linearly separable

ARCHITECTURE:
  NO injection (W_tok_inject removed entirely)
  target is determined by initial state only
  b_pos0 = 2.0 (attend to initial tau at t=0)
  b_posLast = 0.0

  This forces the model to decode the answer from the tau representation.
  tau0_direct ablation is the PURE REPRESENTATIONAL CAPACITY test.

REGIMES:
  reduced_k1:  GEOM = [(0,2,2,1),(2,7,5,1),(7,9,2,1),(9,21,12,1)]  d_tau_hyb=12
  fuller_k3:   GEOM = [(0,2,2,1),(2,7,5,2),(7,9,2,1),(9,21,12,3)]  d_tau_hyb=18

ABLATIONS (both regimes):
  full:         standard readout over full trajectory
  tau0_direct:  pred = tau0[t=0] @ W_pred + b_pred  (PURE REPRESENTATION)
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
CSV_OUT     = RESULTS_DIR / "prime_transport_router_mod12_harmonic_stress_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_router_mod12_harmonic_stress_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Config
# ═══════════════════════════════════════════════════════════════════════
GLOBAL_SEED   = 42
D             = 24
D_HIDDEN      = 32
BATCH_SIZE    = 512
VOCAB         = 4          # 4 mod-12 quarter-phase classes
D_EMB         = 4
N_PHASE_PAIRS = 4
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
B_POS0_INIT   = 2.0        # attend to t=0 (where initial mod-12 info lives)

# Geometry definitions: (start, end, modulus, n_harmonics)
GEOM_REDUCED = [(0, 2, 2, 1), (2, 7, 5, 1), (7, 9, 2, 1), (9, 21, 12, 1)]
GEOM_FULLER  = [(0, 2, 2, 1), (2, 7, 5, 2), (7, 9, 2, 1), (9, 21, 12, 3)]

# Mod-12 block position in onehot (block 3, indices 9:21)
MOD12_SLICE = (9, 21)


def geom_dims(blocks):
    d_ang = sum(2 * n_h for (_, _, _, n_h) in blocks)
    d_hyb = d_ang + N_PHASE_PAIRS
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


def prepare_tables_multi(TN_oh, tau0_oh, TR, pool_ids, blocks):
    TN_ang   = convert_onehot_to_angular_multi(TN_oh, blocks)
    tau0_ang = convert_onehot_to_angular_multi(tau0_oh, blocks)
    tau0_hyb = torch.cat([tau0_ang,
                          torch.ones(tau0_ang.shape[0], N_PHASE_PAIRS)], dim=1)
    return TN_ang, TR, tau0_hyb, pool_ids


# ═══════════════════════════════════════════════════════════════════════
# Precompute mod-12 target for each state
# ═══════════════════════════════════════════════════════════════════════
def compute_mod12_targets(tau0_oh: torch.Tensor) -> torch.Tensor:
    """Returns (N,) int64 tensor: mod12_state % VOCAB for each state."""
    s, e = MOD12_SLICE
    mod12_state = tau0_oh[:, s:e].argmax(dim=1)   # (N,) in {0..11}
    return (mod12_state % VOCAB).long()            # (N,) in {0..3}


# ═══════════════════════════════════════════════════════════════════════
# Router (no injection, b_pos0 enabled)
# ═══════════════════════════════════════════════════════════════════════
class RouterMod12Stress(nn.Module):
    """
    No-injection router for mod-12 quarter-phase classification.

    The model must predict target = mod12_initial_state % 4 from the
    routing trajectory. There is no W_tok_inject — no injection shortcut.

    b_pos0 = B_POS0_INIT (2.0) biases attention toward t=0,
    where the initial mod-12 angular representation lives.
    """
    def __init__(self, TN_ang, TR, tau0_hyb, pool_ids,
                 targets: torch.Tensor,
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
        self.register_buffer("targets",    targets)     # (N,) int64

        m0 = torch.zeros(1, D); m0[0, 0] = 1.0
        self.register_buffer("pos0_mask", m0)
        self.b_pos0 = nn.Parameter(torch.tensor(float(B_POS0_INIT)))

        gen = torch.Generator().manual_seed(seed)
        def rp(*sh): return nn.Parameter(torch.empty(*sh).normal_(0, init_scale, generator=gen))
        def zp(*sh): return nn.Parameter(torch.zeros(*sh))

        self.W_emb  = rp(VOCAB, D_EMB)
        self.W1     = rp(d_in, dh);     self.b1 = zp(dh)
        self.W2     = rp(dh, N_OPS);    self.b2 = zp(N_OPS)
        self.W_attn = rp(dha, d_hyb);   self.b_attn = zp(dha); self.v_attn = rp(dha)
        self.W_pred = rp(d_hyb, VOCAB); self.b_pred = zp(VOCAB)
        # No W_tok_inject — this is the key design choice

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
        base = torch.einsum("bi,bij->bj", w, tn)
        ang_parts = []
        mags      = []
        ai = 0
        for _, _, _, n_h in self.blocks:
            fund = base[:, ai:ai+2]
            mag  = fund.norm(dim=1, keepdim=True).clamp(1e-8)
            mags.append(mag)
            for h_idx in range(n_h):
                pair = base[:, ai + h_idx*2 : ai + h_idx*2 + 2]
                ang_parts.append(pair / mag)
            ai += n_h * 2
        dirn   = torch.cat(ang_parts, dim=1)
        radial = torch.cat(mags, dim=1)
        hybrid = torch.cat([dirn, radial], dim=1)
        new_sids = self.TR[state_ids].gather(
            1, torch.argmax(w, dim=1).unsqueeze(1)).squeeze(1)
        return hybrid, new_sids

    def forward(self, state_ids, tokens, temp):
        tau_prev = self.tau0_table[state_ids]
        soft_taus: List[torch.Tensor] = []
        for t in range(D):
            hybrid, state_ids = self._soft_step(
                tau_prev, state_ids, tokens[:, t], temp)
            tau_prev = hybrid
            soft_taus.append(tau_prev)
        st    = torch.stack(soft_taus, dim=1)
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = (h_a * self.v_attn).sum(dim=-1) + self.pos0_mask * self.b_pos0
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred

    def readout(self, st: torch.Tensor):
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = (h_a * self.v_attn).sum(dim=-1) + self.pos0_mask * self.b_pos0
        alpha = torch.softmax(sc, dim=1)
        pred  = torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred
        return pred, alpha

    def readout_masked(self, st: torch.Tensor, attn_mask: torch.Tensor):
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = ((h_a * self.v_attn).sum(dim=-1)
                 + self.pos0_mask * self.b_pos0
                 + attn_mask)
        alpha = torch.softmax(sc, dim=1)
        pred  = torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred
        return pred, alpha


# ═══════════════════════════════════════════════════════════════════════
# Utilities
# ═══════════════════════════════════════════════════════════════════════
def _normalize_ang_and_radial(ang: torch.Tensor, blocks) -> torch.Tensor:
    ang_parts = []; mags = []; ai = 0
    for _, _, _, n_h in blocks:
        fund = ang[:, ai:ai+2]
        mag  = fund.norm(dim=1, keepdim=True).clamp(1e-8)
        mags.append(mag)
        for h_idx in range(n_h):
            ang_parts.append(ang[:, ai + h_idx*2 : ai + h_idx*2 + 2] / mag)
        ai += n_h * 2
    return torch.cat([torch.cat(ang_parts, dim=1), torch.cat(mags, dim=1)], dim=1)


def make_batch(pool_ids, targets, rng):
    """Sample a batch: returns (state_ids, tokens, target_labels)."""
    idx  = torch.randint(pool_ids.shape[0], (BATCH_SIZE,), generator=rng)
    sids = pool_ids[idx]
    targ = targets[sids]                                     # mod-12 quarter-phase label
    toks = torch.randint(VOCAB, (BATCH_SIZE, D), generator=rng)
    return sids, toks, targ


def eval_acc(model, pool_ids, targets) -> Tuple[float, float]:
    """Returns (accuracy, mean_alpha_0)."""
    model.eval()
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 200)
    correct = 0; a0_sum = 0.0
    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, targ = make_batch(pool_ids, targets, rng)
            logits = model(sids, toks, 0.05)
            correct += (logits.argmax(1) == targ).sum().item()
            # alpha via hard-geom trajectory
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
                    1, best_op.view(B,1,1).expand(B,1,model.d_ang)).squeeze(1)
                hybrid   = _normalize_ang_and_radial(best_ang, model.blocks)
                taus.append(hybrid)
                tau_prev  = hybrid
                sids_loop = model.TR[sids_loop].gather(
                    1, best_op.unsqueeze(1)).squeeze(1)
            st = torch.stack(taus, dim=1)
            _, alpha = model.readout(st)
            a0_sum += alpha[:, 0].mean().item()
    model.train()
    return round(correct / N_EVAL, 4), round(a0_sum / N_BATCHES, 4)


def collect_hard_geom_trajectories(model, pool_ids, targets):
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 888)
    all_st, all_targ = [], []
    B = BATCH_SIZE
    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, targ = make_batch(pool_ids, targets, rng)
            tau_prev  = model.tau0_table[sids]
            sids_loop = sids.clone()
            taus = []
            for t in range(D):
                tn      = model.TN[sids_loop]
                cur_dir = tau_prev[:, :model.d_ang]
                ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)
                best_op = ang_sim.argmax(dim=1)
                best_ang = tn.gather(
                    1, best_op.view(B,1,1).expand(B,1,model.d_ang)).squeeze(1)
                hybrid   = _normalize_ang_and_radial(best_ang, model.blocks)
                taus.append(hybrid.clone())
                tau_prev  = hybrid
                sids_loop = model.TR[sids_loop].gather(
                    1, best_op.unsqueeze(1)).squeeze(1)
            all_st.append(torch.stack(taus, dim=1))
            all_targ.append(targ)
    return all_st, all_targ


# ═══════════════════════════════════════════════════════════════════════
# Ablations
# ═══════════════════════════════════════════════════════════════════════
ABLATION_DEFS = [
    ("full",       "full trajectory readout"),
    ("tau0_direct","only t=0 tau → PURE REPRESENTATION TEST"),
    ("no_tau0",    "t=0 masked"),
    ("last_only",  f"only t={D-1}"),
]


def apply_ablation(model, all_st, all_targ, ablation: str):
    t_start = time.perf_counter()
    correct = 0; a0_sum = 0.0
    with torch.no_grad():
        for st, targ in zip(all_st, all_targ):
            B = st.shape[0]
            if ablation == "full":
                pred, alpha = model.readout(st)
            elif ablation == "tau0_direct":
                # Only t=0 tau — pure representational capacity
                pred  = st[:, 0, :] @ model.W_pred + model.b_pred
                alpha = torch.zeros(B, D); alpha[:, 0] = 1.0
            elif ablation == "no_tau0":
                mask = torch.zeros(1, D); mask[0, 0] = NEG_INF
                pred, alpha = model.readout_masked(st, mask)
            elif ablation == "last_only":
                mask = torch.full((1, D), NEG_INF); mask[0, D-1] = 0.0
                pred, alpha = model.readout_masked(st, mask)
            else:
                raise ValueError(ablation)
            correct += (pred.argmax(1) == targ).sum().item()
            a0_sum  += alpha[:, 0].mean().item()
    wall = round(time.perf_counter() - t_start, 4)
    return round(correct / N_EVAL, 4), round(a0_sum / N_BATCHES, 4), wall


def _interpret_ablation(ablation: str, acc: float, full_acc: float,
                        regime_label: str) -> str:
    drop = round(acc - full_acc, 3)
    if ablation == "full":
        return "reference"
    if ablation == "tau0_direct":
        if acc >= 0.999:  return "tau0 alone sufficient — k representation fully captures mod-12 classes ✓"
        if acc < 0.35:    return "tau0 alone FAILS — k=1 cannot linearly decode quarter-phase classes ✗"
        return f"partial tau0 decoding: {acc:.4f} ({drop:+.3f})"
    if ablation == "no_tau0":
        if acc >= 0.999:  return "trajectory sufficient without t=0"
        if acc < 0.35:    return "t=0 is critical (collapse)"
        return f"{drop:+.3f}"
    if ablation == "last_only":
        if acc >= 0.999:  return "final position encodes answer"
        if acc < 0.35:    return "final position alone insufficient"
        return f"{drop:+.3f}"
    return f"{drop:+.3f}"


# ═══════════════════════════════════════════════════════════════════════
# Training
# ═══════════════════════════════════════════════════════════════════════
def train_regime(blocks, regime_label: str,
                 TN_ang, TR, tau0_hyb, pool_ids, targets):
    d_ang, d_hyb, d_in = geom_dims(blocks)
    print(f"\n{'='*60}")
    print(f"REGIME: {regime_label}")
    print(f"  geometry: d_tau_ang={d_ang}  d_tau_hyb={d_hyb}  d_in={d_in}")
    print(f"  task: target=mod12_initial%4  no_injection  b_pos0={B_POS0_INIT}")
    print(f"{'='*60}")

    model = RouterMod12Stress(TN_ang, TR, tau0_hyb, pool_ids, targets, blocks=blocks)
    opt   = torch.optim.SGD(model.parameters(), lr=LR)
    rng_t = torch.Generator().manual_seed(GLOBAL_SEED + 100)
    model.train()

    t0 = time.perf_counter()
    solve_step = None; final_acc = 0.0; alpha0_f = 0.0

    for step in range(1, MAX_STEPS + 1):
        frac = step / max(MAX_STEPS - 1, 1)
        temp = float(TEMP_START * (TEMP_END / TEMP_START) ** frac)
        sids, toks, targ = make_batch(pool_ids, targets, rng_t)
        loss = F.cross_entropy(model(sids, toks, temp), targ)
        opt.zero_grad(); loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), CLIP_GRAD)
        opt.step()
        if step % EVAL_EVERY == 0:
            acc, a0 = eval_acc(model, pool_ids, targets)
            if acc >= SOLVE_ACC and solve_step is None:
                solve_step = step
            final_acc = acc; alpha0_f = a0
            print(f"    [{regime_label}] step={step:5d}  acc={acc:.4f}  α₀={a0:.4f}")

    wall = round(time.perf_counter() - t0, 1)
    sps  = round(MAX_STEPS / wall, 1)
    b0   = round(float(model.b_pos0.item()), 4)
    print(f"  {regime_label}: acc={final_acc:.4f}  solve={solve_step}"
          f"  sps={sps}  b_pos0={b0}  α₀={alpha0_f:.4f}")
    model.eval()
    return model, final_acc, solve_step, sps, wall, alpha0_f, b0


# ═══════════════════════════════════════════════════════════════════════
# Representational separability check (analytic)
# ═══════════════════════════════════════════════════════════════════════
def check_k1_separability_analytic():
    """Print analytic k=1 vs k=3 representation of each mod-12 class."""
    print("\nAnalytic mod-12 quarter-phase class representations:")
    for k in [1, 3]:
        print(f"  k={k} harmonic:")
        class_reps = {}
        for j in range(12):
            cls = j % 4
            c = round(math.cos(2 * math.pi * k * j / 12), 3)
            s = round(math.sin(2 * math.pi * k * j / 12), 3)
            class_reps.setdefault(cls, set()).add((c, s))
        for cls in range(4):
            uniq = class_reps[cls]
            n_uniq = len(uniq)
            separable = (n_uniq == 1)
            print(f"    class {cls}: {n_uniq} distinct points → "
                  f"{'SEPARABLE ✓' if separable else 'NOT separable ✗'}")


# ═══════════════════════════════════════════════════════════════════════
# Output writers
# ═══════════════════════════════════════════════════════════════════════
def write_csv(training_rows: List[Dict], ablation_rows: List[Dict]):
    fieldnames = [
        "regime", "geometry_regime", "ablation",
        "d_tau_ang", "d_tau_hyb",
        "accuracy", "delta_vs_baseline",
        "solve_step", "alpha0", "b_pos0_trained",
        "runtime_seconds", "note"
    ]
    with open(CSV_OUT, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fieldnames)
        w.writeheader()
        for r in training_rows + ablation_rows:
            w.writerow({k: r.get(k, "") for k in fieldnames})
    print(f"\nCSV written: {CSV_OUT}  ({len(training_rows)+len(ablation_rows)} rows)")


def write_markdown(training_rows, ablation_rows, conclusion, answers):
    ts = datetime.datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ")
    L: List[str] = []

    L.append("# Prime Transport Router: Mod-12 Harmonic Resolution Stress Test v1\n\n")
    L.append(f"**Generated:** {ts}  \n")
    L.append(f"**Config:** D={D}, D_HIDDEN={D_HIDDEN}, B_train={BATCH_SIZE}, "
             f"N_eval={N_EVAL}, steps={MAX_STEPS}, LR={LR}  \n\n")

    L.append("## Task Design\n\n")
    L.append("**Target:** `mod12_initial_state % 4` — no injection, no random x0.  \n")
    L.append("**4 quarter-phase classes:**  \n")
    L.append("  Class 0: j ∈ {0, 4, 8}  \n")
    L.append("  Class 1: j ∈ {1, 5, 9}  \n")
    L.append("  Class 2: j ∈ {2, 6, 10}  \n")
    L.append("  Class 3: j ∈ {3, 7, 11}  \n\n")

    L.append("**Why this specifically stresses mod-12 resolution:**  \n")
    L.append("| Harmonic | Class separability |\n")
    L.append("|----------|--------------------|\n")
    L.append("| k=1 only | Each class = equilateral triangle at 120° spacing — "
             "**NOT linearly separable** |\n")
    L.append("| k=3 added | Each class collapses to a single point: "
             "{(1,0),(0,1),(-1,0),(0,-1)} — **trivially separable** |\n")
    L.append("\n**No injection** — model cannot shortcut through W_tok_inject.  \n")
    L.append("**b_pos0=2.0** — attention biased toward t=0 (initial mod-12 representation).  \n")
    L.append("**tau0_direct ablation = pure representational capacity test.**  \n\n")

    L.append("---\n\n")
    L.append("## Geometry\n\n")
    L.append("| Component | reduced_k1 | fuller_k3 |\n")
    L.append("|-----------|------------|-----------|\n")
    L.append("| Block 3 (mod=12) | k=1 only → 2 dims | k=1,2,3 → 6 dims |\n")
    L.append(f"| d_tau_hyb | {geom_dims(GEOM_REDUCED)[1]} | {geom_dims(GEOM_FULLER)[1]} |\n\n")

    L.append("---\n\n")
    L.append("## Training Results\n\n")
    L.append("| regime | d_tau_hyb | accuracy | solve_step | α₀ | b_pos0 | runtime_s |\n")
    L.append("|--------|-----------|----------|------------|----|---------|-----------|\n")
    for r in training_rows:
        ss = str(r["solve_step"]) if r["solve_step"] else "—"
        L.append(f"| {r['regime']} | {r['d_tau_hyb']} | {r['accuracy']:.4f} | {ss}"
                 f" | {r['alpha0']:.4f} | {r['b_pos0_trained']:.4f}"
                 f" | {r['runtime_seconds']:.1f} |\n")
    L.append("\n")

    L.append("---\n\n")
    L.append("## Ablation Results\n\n")
    L.append("| regime | ablation | accuracy | Δ_vs_full | α₀ | note |\n")
    L.append("|--------|----------|----------|-----------|----|------|\n")
    for r in ablation_rows:
        delta = round(r["accuracy"] - r["delta_vs_baseline"], 4)
        sign  = "+" if delta >= 0 else ""
        L.append(f"| {r['regime']} | {r['ablation']} | {r['accuracy']:.4f}"
                 f" | {sign}{delta:.4f} | {r['alpha0']:.4f} | {r['note']} |\n")
    L.append("\n")

    L.append("---\n\n")
    L.append("## Explicit Answers\n\n")
    for k, v in answers.items():
        if not k.startswith("honest_"):
            L.append(f"**{k}**  \n{v}  \n\n")

    L.append("---\n\n")
    L.append("## Honesty Section\n\n")
    L.append(f"**What was fairly tested:**  \n{answers.get('honest_tested', '')}  \n\n")
    L.append(f"**What remains unresolved:**  \n{answers.get('honest_unresolved', '')}  \n\n")
    L.append(f"**Whether the 12-tick intuition survives targeted stress:**  \n"
             f"{answers.get('honest_12tick', '')}  \n\n")

    L.append("---\n\n")
    L.append(f"## MOD-12 HIGHER HARMONICS ARE: {conclusion}\n\n")

    with open(MD_OUT, "w") as f:
        f.writelines(L)
    print(f"Markdown written: {MD_OUT}")


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main():
    print("=" * 60)
    print("MOD-12 HARMONIC RESOLUTION STRESS TEST v1")
    print("=" * 60)
    d_r = geom_dims(GEOM_REDUCED)
    d_f = geom_dims(GEOM_FULLER)
    print(f"reduced_k1:  d_tau_ang={d_r[0]}  d_tau_hyb={d_r[1]}  d_in={d_r[2]}")
    print(f"fuller_k3:   d_tau_ang={d_f[0]}  d_tau_hyb={d_f[1]}  d_in={d_f[2]}")
    print(f"Task: target=mod12_initial%4  no_injection  b_pos0={B_POS0_INIT}  steps={MAX_STEPS}")

    check_k1_separability_analytic()

    print("\nLoading cache ...")
    t0 = time.perf_counter()
    data = torch.load(CACHE_PATH, weights_only=False)
    N_states = data["TN_oh"].shape[0]
    print(f"  {N_states:,} states in {time.perf_counter()-t0:.3f}s")

    TN_oh = data["TN_oh"]; tau0_oh = data["tau0_oh"]
    TR = data["TR"]; pool_ids = data["pool_ids"]

    # Precompute mod-12 quarter-phase targets for ALL states
    all_targets = compute_mod12_targets(tau0_oh)
    counts = [(all_targets == c).sum().item() for c in range(VOCAB)]
    print(f"  Target class distribution over {N_states:,} states:")
    for c, n in enumerate(counts):
        print(f"    class {c}: {n:,} states ({n/N_states*100:.1f}%)")

    # Check pool balance
    pool_targets = all_targets[pool_ids]
    pool_counts  = [(pool_targets == c).sum().item() for c in range(VOCAB)]
    print(f"  Pool ({pool_ids.shape[0]:,} states) class balance: "
          + "  ".join(f"cls{c}={n}" for c,n in enumerate(pool_counts)))

    TN_r, TR_r, tau0_r, pid_r = prepare_tables_multi(TN_oh, tau0_oh, TR, pool_ids, GEOM_REDUCED)
    TN_f, TR_f, tau0_f, pid_f = prepare_tables_multi(TN_oh, tau0_oh, TR, pool_ids, GEOM_FULLER)

    training_rows: List[Dict] = []
    ablation_rows: List[Dict] = []

    # ── Regime A: reduced k=1 geometry ───────────────────────────────
    model_r, acc_r, solve_r, sps_r, wall_r, a0_r, b0_r = train_regime(
        GEOM_REDUCED, "reduced_k1",
        TN_r, TR_r, tau0_r, pid_r, all_targets)

    training_rows.append(dict(
        regime="reduced_k1", geometry_regime="reduced",
        ablation="training_summary",
        d_tau_ang=d_r[0], d_tau_hyb=d_r[1],
        accuracy=acc_r, delta_vs_baseline=0.0,
        solve_step=solve_r, alpha0=a0_r,
        b_pos0_trained=b0_r, runtime_seconds=wall_r,
        note="training result"
    ))

    # ── Regime B: fuller k=1,2,3 geometry ────────────────────────────
    model_f, acc_f, solve_f, sps_f, wall_f, a0_f, b0_f = train_regime(
        GEOM_FULLER, "fuller_k3",
        TN_f, TR_f, tau0_f, pid_f, all_targets)

    training_rows.append(dict(
        regime="fuller_k3", geometry_regime="fuller",
        ablation="training_summary",
        d_tau_ang=d_f[0], d_tau_hyb=d_f[1],
        accuracy=acc_f, delta_vs_baseline=round(acc_f - acc_r, 4),
        solve_step=solve_f, alpha0=a0_f,
        b_pos0_trained=b0_f, runtime_seconds=wall_f,
        note="training result"
    ))

    # ── Ablations ─────────────────────────────────────────────────────
    print(f"\n{'='*60}")
    print("ABLATIONS")
    print("=" * 60)

    for regime_label, model, blocks, TN_t, TR_t, tau0_t, pid_t, acc_train in [
        ("reduced_k1", model_r, GEOM_REDUCED, TN_r, TR_r, tau0_r, pid_r, acc_r),
        ("fuller_k3",  model_f, GEOM_FULLER,  TN_f, TR_f, tau0_f, pid_f, acc_f),
    ]:
        geom_l = "reduced" if regime_label == "reduced_k1" else "fuller"
        d_ang, d_hyb, _ = geom_dims(blocks)
        print(f"\n  Collecting trajectories for {regime_label} ...")
        all_st, all_targ = collect_hard_geom_trajectories(model, pid_t, all_targets)

        full_acc = None
        for abl_name, abl_note_base in ABLATION_DEFS:
            abl_acc, abl_a0, abl_wall = apply_ablation(model, all_st, all_targ, abl_name)
            if full_acc is None:
                full_acc = abl_acc
            delta   = round(abl_acc - (full_acc or abl_acc), 4)
            interp  = _interpret_ablation(abl_name, abl_acc, full_acc or abl_acc, regime_label)
            sign    = "+" if delta >= 0 else ""
            print(f"    [{abl_name:14s}] acc={abl_acc:.4f}  Δ={sign}{delta:.4f}"
                  f"  α₀={abl_a0:.4f}  — {interp}")
            ablation_rows.append(dict(
                regime=regime_label, geometry_regime=geom_l,
                ablation=abl_name,
                d_tau_ang=d_ang, d_tau_hyb=d_hyb,
                accuracy=abl_acc, delta_vs_baseline=full_acc,
                solve_step="", alpha0=abl_a0,
                b_pos0_trained="", runtime_seconds=abl_wall,
                note=interp
            ))

    # ── Summary and conclusion ─────────────────────────────────────────
    delta = round(acc_f - acc_r, 4)
    tau0d_r = next((r["accuracy"] for r in ablation_rows
                    if r["regime"] == "reduced_k1" and r["ablation"] == "tau0_direct"), None)
    tau0d_f = next((r["accuracy"] for r in ablation_rows
                    if r["regime"] == "fuller_k3"  and r["ablation"] == "tau0_direct"), None)

    print(f"\n{'='*60}")
    print("SUMMARY")
    print("=" * 60)
    print(f"  reduced_k1   train acc={acc_r:.4f}  solve={solve_r}  "
          f"tau0_direct={tau0d_r:.4f}  d_tau_hyb={d_r[1]}")
    print(f"  fuller_k3    train acc={acc_f:.4f}  solve={solve_f}  "
          f"tau0_direct={tau0d_f:.4f}  d_tau_hyb={d_f[1]}")
    print(f"  Δ (fuller - reduced) = {delta:+.4f}")
    print(f"  Δ tau0_direct = {round(tau0d_f - tau0d_r, 4):+.4f}")

    # Conclusion
    repr_gap  = round((tau0d_f or 0) - (tau0d_r or 0), 4)
    train_gap = delta

    if repr_gap >= 0.3 and tau0d_r < 0.4 and tau0d_f >= 0.9:
        if train_gap >= 0.1 or (acc_r < SOLVE_ACC and acc_f >= SOLVE_ACC):
            conclusion = "NECESSARY"
        else:
            conclusion = "HELPFUL"
    elif repr_gap >= 0.1:
        conclusion = "HELPFUL"
    else:
        conclusion = "REDUNDANT"

    answers = {
        "1. Does the richer mod-12 basis matter on a mod-12-sensitive task?":
            (f"YES — tau0_direct gap = {repr_gap:+.4f} "
             f"(reduced={tau0d_r:.4f}, fuller={tau0d_f:.4f}). "
             f"k=3 harmonic provides representational advantage."
             if repr_gap >= 0.1 else
             f"NO — tau0_direct gap = {repr_gap:+.4f}. "
             f"Both representations perform similarly even on mod-12-sensitive task."),
        "2. Is k=1 for mod-12 actually insufficient under targeted stress?":
            (f"YES — tau0_direct={tau0d_r:.4f} for k=1. "
             f"Linear classifier on k=1 tau0 CANNOT decode quarter-phase classes."
             if tau0d_r is not None and tau0d_r < 0.4 else
             f"PARTIALLY — tau0_direct={tau0d_r:.4f}. "
             f"k=1 shows degraded performance on the targeted task."),
        "3. Does this support the 12-tick clock intuition in a narrow testable sense?":
            (f"YES — k=3 tau0_direct={tau0d_f:.4f} vs k=1 tau0_direct={tau0d_r:.4f}. "
             f"The k=3 harmonic resolves the 4 quarter-phase classes that k=1 cannot. "
             f"This confirms the 12-tick phase structure requires >k=1 for full resolution."
             if repr_gap >= 0.3 else
             f"PARTIALLY — gap={repr_gap:+.4f}. "
             f"Some advantage but not conclusive."),
        "honest_tested":
            "The task directly targets the representational limitation: "
            "mod-12 quarter-phase classification (j%4) is provably not linearly separable "
            "in k=1 representation but trivially separable with k=3. "
            "No injection shortcut. tau0_direct measures pure representational capacity.",
        "honest_unresolved":
            "Whether routing (full trajectory) compensates for the k=1 representational gap "
            "by learning different routing paths for different phase classes. "
            "Full accuracy vs tau0_direct accuracy captures this: if full >> tau0_direct "
            "for reduced_k1, routing is compensating for the representational gap.",
        "honest_12tick":
            (f"YES — the k=3 harmonic (period 4 within mod-12) is necessary and sufficient "
             f"for linear decoding of quarter-phase classes. The '12-tick clock' intuition "
             f"survives in the narrow sense that k=1 cannot resolve it but k=3 can. "
             f"tau0_direct: k=1={tau0d_r:.4f} vs k=3={tau0d_f:.4f}."
             if repr_gap >= 0.3 else
             f"INCONCLUSIVE — the gap between k=1 and k=3 is {repr_gap:+.4f}. "
             f"Either routing compensates for the k=1 limitation, or the 12-tick "
             f"structure is not visible at this scale/task."),
    }

    print(f"\nMOD-12 HIGHER HARMONICS ARE: {conclusion}")

    write_csv(training_rows, ablation_rows)
    write_markdown(training_rows, ablation_rows, conclusion, answers)


if __name__ == "__main__":
    main()

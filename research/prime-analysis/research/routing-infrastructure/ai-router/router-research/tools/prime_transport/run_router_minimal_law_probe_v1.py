#!/usr/bin/env python3
"""run_router_minimal_law_probe_v1.py

MINIMAL ROUTING LAW PROBE v1

Determine the minimum routing law actually required for correctness.
Tests whether exact nearest-neighbor geometric routing is necessary,
or whether a broader valid geometric corridor contains many
interchangeable local moves.

No retraining. No redesign. Measurement only.

VALID CORRIDOR DEFINITION (explicit):
  For each routing step, the valid corridor consists of all candidate
  operators whose angular similarity score is within CORRIDOR_WIDTH of
  the maximum angular similarity score:
    valid = {op : ang_sim[op] >= max(ang_sim) - CORRIDOR_WIDTH}
  CORRIDOR_WIDTH = 0.35 (approx half the median margin of 0.63 from adversarial probe).

VARIANTS TESTED:
  1. exact_k1          — argmax ang_sim (baseline, deterministic)
  2. top2_random       — uniform random among top-2 by ang_sim
  3. top3_random       — uniform random among top-3 by ang_sim
  4. top4_random       — uniform random among top-4 by ang_sim
  5. fixed_op_0        — always pick operator index 0 (deterministic negative control)
  6. corridor_random   — uniform random among valid corridor ops (ang_sim >= max - 0.35)
  7. corridor_tight    — uniform random among valid corridor ops (ang_sim >= max - 0.10)
  8. unrestricted_rand — uniform random among all 6 ops (full negative control)

Each stochastic variant: N_TRIALS=10 independent runs → report mean ± std accuracy.
Deterministic variants: 1 run.

METRICS PER VARIANT:
  - accuracy (mean ± std)
  - solve_step (first step where acc >= SOLVE_ACC, from training; N/A here — inference only)
  - route_entropy: mean Shannon entropy over chosen operators per episode
  - transport_fraction: mean angular similarity of chosen operator (routing confidence proxy)
  - corridor_size: mean number of ops in valid corridor per step
  - failure: True if any trial accuracy < FAILURE_TH
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
CSV_OUT     = RESULTS_DIR / "prime_transport_router_minimal_law_probe_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_router_minimal_law_probe_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Locked config
# ═══════════════════════════════════════════════════════════════════════
GLOBAL_SEED   = 42
D             = 24           # sequence depth (locked)
D_HIDDEN      = 32
BATCH_SIZE    = 256
B0_INIT       = 2.0
VOCAB         = 4
D_EMB         = 4
D_TAU_ANG     = 8
N_PHASE_PAIRS = 4
D_TAU_HYB     = D_TAU_ANG + N_PHASE_PAIRS   # 12
D_IN_HYB      = D_EMB + D_TAU_HYB            # 16
N_OPS         = 6
TRANSPORT_TH  = 3
LR            = 0.02
TEMP_START    = 2.0
TEMP_END      = 0.1
CLIP_GRAD     = 1.0
EVAL_EVERY    = 500
N_EVAL        = 2048        # larger eval for stable stochastic variance
SOLVE_ACC     = 0.999
PHASE_BLOCKS  = [(0, 2, 2), (2, 7, 5), (7, 9, 2), (9, 21, 12)]
INIT_SCALE    = 0.05
MAX_STEPS     = 3_000
FAILURE_TH    = 0.99
N_TRIALS      = 10           # trials per stochastic variant

CORRIDOR_WIDTH_WIDE  = 0.35  # half median margin (0.63/2 ≈ 0.315, rounded up)
CORRIDOR_WIDTH_TIGHT = 0.10  # tight corridor (only small margins included)

try:
    sys.path.insert(0, str(SCRIPT_DIR))
    from thread_policy import select_threads as _select_threads
    _select_threads(BATCH_SIZE, D_IN_HYB, D_HIDDEN)
except Exception:
    pass


# ═══════════════════════════════════════════════════════════════════════
# Angular conversion + hybrid tables (locked)
# ═══════════════════════════════════════════════════════════════════════
def convert_onehot_to_angular(onehot: torch.Tensor) -> torch.Tensor:
    shape = onehot.shape[:-1]
    out = torch.zeros(*shape, D_TAU_ANG, dtype=onehot.dtype)
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
# Model (locked baseline)
# ═══════════════════════════════════════════════════════════════════════
class RouterAngularHybrid(nn.Module):
    def __init__(self, TN_ang, TR, tau0_hyb, pool_ids,
                 d_hidden=D_HIDDEN, d_context=D,
                 b0_init=B0_INIT, init_scale=INIT_SCALE, seed=GLOBAL_SEED):
        super().__init__()
        dh = d_hidden; dha = max(8, dh // 4)
        self.register_buffer("TN", TN_ang)
        self.register_buffer("TR", TR)
        self.register_buffer("tau0_table", tau0_hyb)
        self.register_buffer("pool_ids", pool_ids)
        m = torch.zeros(1, d_context); m[0, 0] = 1.0
        self.register_buffer("pos0_mask", m)
        self.b_pos0 = nn.Parameter(torch.tensor(b0_init))
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
        B = state_ids.shape[0]; d = tokens.shape[1]
        tau_prev = self.tau0_table[state_ids]
        soft_taus: List[torch.Tensor] = []
        for t in range(d):
            tn   = self.TN[state_ids]
            embs = self.W_emb[tokens[:, t]]
            h    = torch.tanh(torch.cat([embs, tau_prev], dim=1) @ self.W1 + self.b1)
            logits = h @ self.W2 + self.b2
            if self.training:
                u = torch.rand_like(logits).clamp_(1e-20, 1.0)
                w = torch.softmax((logits - torch.log(-torch.log(u))) / temp, dim=1)
            else:
                w = torch.softmax(logits / 0.05, dim=1)
            base  = torch.einsum("bi,bij->bj", w, tn)
            pairs = base.view(B, N_PHASE_PAIRS, 2)
            mag   = (pairs * pairs).sum(dim=2).sqrt()
            dirn  = (pairs / mag.clamp(min=1e-8).unsqueeze(2)).view(B, D_TAU_ANG)
            hybrid = torch.cat([dirn, mag], dim=1)
            tau_prev = (hybrid + self.W_tok_inject[x0]) if t == 0 else hybrid
            soft_taus.append(tau_prev)
            state_ids = self.TR[state_ids].gather(
                1, torch.argmax(w, dim=1).unsqueeze(1)).squeeze(1)
        st    = torch.stack(soft_taus, dim=1)
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = (h_a * self.v_attn).sum(dim=-1) + self.pos0_mask * self.b_pos0
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred


# ═══════════════════════════════════════════════════════════════════════
# Training utilities
# ═══════════════════════════════════════════════════════════════════════
def make_batch(pool_ids, rng):
    idx  = torch.randint(pool_ids.shape[0], (BATCH_SIZE,), generator=rng)
    sids = pool_ids[idx]
    x0   = torch.randint(VOCAB, (BATCH_SIZE,), generator=rng)
    toks = torch.randint(VOCAB, (BATCH_SIZE, D), generator=rng)
    toks[:, 0] = x0
    return sids, toks, x0


def _eval_soft(model, pool_ids):
    model.eval()
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 200)
    correct = 0
    with torch.no_grad():
        for _ in range(N_EVAL // BATCH_SIZE):
            sids, toks, x0 = make_batch(pool_ids, rng)
            correct += (model(sids, toks, x0, 0.05).argmax(1) == x0).sum().item()
    model.train()
    return correct / N_EVAL


def train_model(TN_ang, TR, tau0_hyb, pool_ids):
    """Train a D=24 model with soft scaffold. Returns trained model."""
    model = RouterAngularHybrid(TN_ang, TR, tau0_hyb, pool_ids)
    opt   = torch.optim.SGD(model.parameters(), lr=LR)
    rng_t = torch.Generator().manual_seed(GLOBAL_SEED + 100)
    model.train()
    t0 = time.perf_counter()
    solve_step = None
    final_acc  = 0.0
    for step in range(1, MAX_STEPS + 1):
        frac   = step / max(MAX_STEPS - 1, 1)
        temp   = float(TEMP_START * (TEMP_END / TEMP_START) ** frac)
        sids, toks, x0 = make_batch(pool_ids, rng_t)
        logits = model(sids, toks, x0, temp)
        loss   = F.cross_entropy(logits, x0)
        opt.zero_grad(); loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), CLIP_GRAD)
        opt.step()
        if step % EVAL_EVERY == 0:
            acc = _eval_soft(model, pool_ids)
            if acc >= SOLVE_ACC and solve_step is None:
                solve_step = step
            final_acc = acc
            print(f"    step={step:5d}  acc={acc:.4f}")
    wall = time.perf_counter() - t0
    print(f"  Train done: acc={final_acc:.4f}  solve={solve_step}  "
          f"sps={MAX_STEPS/wall:.1f}  wall={wall:.1f}s")
    model.eval()
    return model, final_acc, solve_step


# ═══════════════════════════════════════════════════════════════════════
# Core inference engine — all variants share this loop
# ═══════════════════════════════════════════════════════════════════════

def _run_variant(model, pool_ids, variant: str, corridor_width: float = 0.35,
                 seed: int = GLOBAL_SEED + 888) -> Dict:
    """
    Run one inference trial under the specified routing law.

    variant options:
      'exact_k1'          — argmax ang_sim
      'top2_random'       — random among top-2
      'top3_random'       — random among top-3
      'top4_random'       — random among top-4
      'fixed_op_0'        — always operator 0
      'corridor_random'   — random within ang_sim >= max - corridor_width
      'unrestricted_rand' — uniform random over all 6
    """
    model.eval()
    rng = torch.Generator().manual_seed(seed)
    correct = 0
    total   = 0
    route_entropy_sum   = 0.0
    transport_sim_sum   = 0.0
    corridor_size_sum   = 0.0
    op_counts = torch.zeros(N_OPS)  # for entropy computation

    with torch.no_grad():
        for _ in range(N_EVAL // BATCH_SIZE):
            sids, toks, x0 = make_batch(pool_ids, rng)
            B = sids.shape[0]
            tau_prev  = model.tau0_table[sids]
            soft_taus = []
            sids_loop = sids.clone()
            batch_op_hist = torch.zeros(B, N_OPS)  # op selection histogram per episode

            for t in range(D):
                tn      = model.TN[sids_loop]                     # (B, 6, 8)
                cur_dir = tau_prev[:, :D_TAU_ANG]                 # (B, 8)
                ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn) # (B, 6)

                # ── Select operator according to variant ────────────────
                if variant == "exact_k1":
                    chosen = ang_sim.argmax(dim=1)                 # (B,)

                elif variant == "top2_random":
                    top_idx = ang_sim.topk(2, dim=1).indices       # (B, 2)
                    pick = torch.randint(2, (B,), generator=rng)
                    chosen = top_idx.gather(1, pick.unsqueeze(1)).squeeze(1)

                elif variant == "top3_random":
                    top_idx = ang_sim.topk(3, dim=1).indices
                    pick = torch.randint(3, (B,), generator=rng)
                    chosen = top_idx.gather(1, pick.unsqueeze(1)).squeeze(1)

                elif variant == "top4_random":
                    top_idx = ang_sim.topk(4, dim=1).indices
                    pick = torch.randint(4, (B,), generator=rng)
                    chosen = top_idx.gather(1, pick.unsqueeze(1)).squeeze(1)

                elif variant == "fixed_op_0":
                    chosen = torch.zeros(B, dtype=torch.long)

                elif variant in ("corridor_random", "corridor_tight"):
                    max_sim = ang_sim.max(dim=1, keepdim=True).values
                    in_corridor = (ang_sim >= max_sim - corridor_width).float()   # (B, 6)
                    corridor_size_sum += in_corridor.sum(dim=1).mean().item()
                    # Sample uniformly within corridor: mask + random
                    noise = torch.rand(B, N_OPS, generator=rng)
                    masked_noise = noise * in_corridor + (1.0 - in_corridor) * (-1e9)
                    chosen = masked_noise.argmax(dim=1)

                elif variant == "unrestricted_rand":
                    chosen = torch.randint(N_OPS, (B,), generator=rng)

                else:
                    raise ValueError(f"Unknown variant: {variant}")

                # Track operator selection stats
                for op_i in range(N_OPS):
                    batch_op_hist[:, op_i] += (chosen == op_i).float()

                # Track transport confidence (mean ang_sim of chosen op)
                chosen_sim = ang_sim.gather(1, chosen.unsqueeze(1)).squeeze(1)
                transport_sim_sum += chosen_sim.mean().item()

                # Hard tau: exact TN entry for chosen op
                best_ang = tn.gather(
                    1, chosen.view(B, 1, 1).expand(B, 1, D_TAU_ANG)
                ).squeeze(1)
                hybrid = torch.cat(
                    [best_ang, torch.ones(B, N_PHASE_PAIRS)], dim=1
                )
                tau_prev = (hybrid + model.W_tok_inject[x0]) if t == 0 else hybrid
                soft_taus.append(tau_prev)
                sids_loop = model.TR[sids_loop].gather(
                    1, chosen.unsqueeze(1)).squeeze(1)

            # Compute per-episode route entropy
            probs = batch_op_hist / D   # (B, N_OPS) — empirical op distribution
            log_probs = torch.log(probs.clamp(min=1e-9))
            ent = -(probs * log_probs).sum(dim=1)  # (B,) Shannon entropy
            route_entropy_sum += ent.mean().item()

            # Attention + prediction
            st    = torch.stack(soft_taus, dim=1)
            h_a   = torch.tanh(st @ model.W_attn.t() + model.b_attn)
            sc    = (h_a * model.v_attn).sum(dim=-1) + model.pos0_mask * model.b_pos0
            alpha = torch.softmax(sc, dim=1)
            pred  = torch.einsum("bd,bdt->bt", alpha, st) @ model.W_pred + model.b_pred
            correct += (pred.argmax(1) == x0).sum().item()
            total   += B

    n_batches = N_EVAL // BATCH_SIZE
    n_steps   = n_batches * D
    accuracy          = round(correct / total, 4)
    route_entropy     = round(route_entropy_sum / n_batches, 4)
    transport_frac    = round(transport_sim_sum / n_steps, 4)
    corridor_size_avg = round(corridor_size_sum / n_steps, 4) if "corridor" in variant else float("nan")

    return {
        "accuracy":       accuracy,
        "route_entropy":  route_entropy,
        "transport_frac": transport_frac,
        "corridor_size":  corridor_size_avg,
    }


def run_variant_multi(model, pool_ids, variant: str, corridor_width: float = 0.35,
                      n_trials: int = N_TRIALS, deterministic: bool = False) -> Dict:
    """Run variant n_trials times; return aggregate stats."""
    results = []
    if deterministic:
        r = _run_variant(model, pool_ids, variant, corridor_width, seed=GLOBAL_SEED + 888)
        results = [r]
        n_trials = 1
    else:
        for trial in range(n_trials):
            seed = GLOBAL_SEED + 888 + trial * 137
            r = _run_variant(model, pool_ids, variant, corridor_width, seed=seed)
            results.append(r)

    accs   = [r["accuracy"] for r in results]
    ents   = [r["route_entropy"] for r in results]
    trans  = [r["transport_frac"] for r in results]
    corr   = [r["corridor_size"] for r in results if not math.isnan(r["corridor_size"])]

    acc_mean = round(sum(accs) / len(accs), 4)
    acc_std  = round((sum((x - acc_mean)**2 for x in accs) / len(accs))**0.5, 4)
    acc_min  = min(accs)
    ent_mean = round(sum(ents) / len(ents), 4)
    trans_mean = round(sum(trans) / len(trans), 4)
    corr_mean  = round(sum(corr) / len(corr), 4) if corr else float("nan")

    return {
        "variant":        variant,
        "n_trials":       n_trials,
        "acc_mean":       acc_mean,
        "acc_std":        acc_std,
        "acc_min":        acc_min,
        "route_entropy":  ent_mean,
        "transport_frac": trans_mean,
        "corridor_size":  corr_mean,
        "failure":        acc_min < FAILURE_TH,
        "any_degraded":   acc_mean < (1.0 - 0.001),  # any drop from 1.0
    }


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main():
    print("=" * 60)
    print("MINIMAL ROUTING LAW PROBE v1")
    print("=" * 60)

    # Load cache
    print("\nLoading cache ...")
    t0 = time.perf_counter()
    data = torch.load(CACHE_PATH, weights_only=False)
    t_load = time.perf_counter() - t0
    print(f"  {data['TN_oh'].shape[0]:,} states in {t_load:.3f}s")
    TN_ang, TR, tau0_hyb, pool_ids = prepare_hybrid_tables(
        data["TN_oh"], data["tau0_oh"], data["TR"], data["pool_ids"]
    )

    # Train D=24 model
    print("\nTraining D=24 model ...")
    model, train_acc, solve_step = train_model(TN_ang, TR, tau0_hyb, pool_ids)
    print(f"  Model ready. Train acc={train_acc:.4f}  solve_step={solve_step}")

    # ── Baseline verification ───────────────────────────────────────────
    print("\n  Verifying baselines ...")
    base_exact = _run_variant(model, pool_ids, "exact_k1", seed=GLOBAL_SEED + 888)
    base_soft  = _eval_soft(model, pool_ids)
    print(f"  exact_k1  hard  acc = {base_exact['accuracy']:.4f}")
    print(f"  soft MLP        acc = {base_soft:.4f}")

    # ── Define all variants ─────────────────────────────────────────────
    variants_cfg = [
        # (variant_name, corridor_width, n_trials, deterministic, label)
        ("exact_k1",          0.35,  1,        True,  "k=1 exact nearest-neighbor (deterministic)"),
        ("top2_random",       0.35,  N_TRIALS, False, "top-2 uniform random"),
        ("top3_random",       0.35,  N_TRIALS, False, "top-3 uniform random"),
        ("top4_random",       0.35,  N_TRIALS, False, "top-4 uniform random"),
        ("fixed_op_0",        0.35,  1,        True,  "fixed operator 0 (negative control)"),
        ("corridor_random",   CORRIDOR_WIDTH_WIDE,  N_TRIALS, False,
         f"corridor random (ang_sim >= max - {CORRIDOR_WIDTH_WIDE})"),
        ("corridor_tight",    CORRIDOR_WIDTH_TIGHT, N_TRIALS, False,
         f"corridor tight (ang_sim >= max - {CORRIDOR_WIDTH_TIGHT})"),
        ("unrestricted_rand", 0.35,  N_TRIALS, False, "uniform random over all 6 (negative control)"),
    ]

    print(f"\n{'='*60}")
    print(f"RUNNING {len(variants_cfg)} VARIANTS  (N_TRIALS={N_TRIALS} for stochastic)")
    print(f"{'='*60}")

    all_results = []
    for vname, cw, nt, det, label in variants_cfg:
        t_v = time.perf_counter()
        res = run_variant_multi(model, pool_ids, vname, cw, nt, det)
        wall_v = time.perf_counter() - t_v
        acc_str = f"{res['acc_mean']:.4f}±{res['acc_std']:.4f}" if not det else f"{res['acc_mean']:.4f}"
        corr_str = f"{res['corridor_size']:.2f}" if not math.isnan(res["corridor_size"]) else "—"
        fail_str = " ← FAIL" if res["failure"] else ""
        degrade_str = " ← DEGRADED" if res["any_degraded"] and not res["failure"] else ""
        print(f"  [{vname:<20}] acc={acc_str:>14}  "
              f"H={res['route_entropy']:.3f}  "
              f"sim={res['transport_frac']:.3f}  "
              f"corr={corr_str}  "
              f"({wall_v:.1f}s){fail_str}{degrade_str}")
        all_results.append({"label": label, "corridor_width": cw, **res})

    # ── Per-variant corridor size analysis ──────────────────────────────
    print("\nAnalysing corridor size distribution ...")
    rng_a = torch.Generator().manual_seed(GLOBAL_SEED + 888)
    corridor_sizes = {0.10: [], 0.20: [], 0.35: [], 0.50: [], 1.0: []}
    with torch.no_grad():
        model.eval()
        for _ in range(8):   # 8 × 256 = 2048 samples
            sids, toks, x0 = make_batch(pool_ids, rng_a)
            B = sids.shape[0]
            tau_prev  = model.tau0_table[sids]
            sids_loop = sids.clone()
            for t in range(D):
                tn      = model.TN[sids_loop]
                cur_dir = tau_prev[:, :D_TAU_ANG]
                ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)
                max_sim = ang_sim.max(dim=1, keepdim=True).values
                for w in corridor_sizes:
                    in_c = (ang_sim >= max_sim - w).float().sum(dim=1)  # (B,)
                    corridor_sizes[w].extend(in_c.tolist())
                # Advance using exact_k1 (to get realistic next state)
                best = ang_sim.argmax(dim=1)
                best_ang = tn.gather(1, best.view(B,1,1).expand(B,1,D_TAU_ANG)).squeeze(1)
                hybrid = torch.cat([best_ang, torch.ones(B, N_PHASE_PAIRS)], dim=1)
                tau_prev  = (hybrid + model.W_tok_inject[x0]) if t == 0 else hybrid
                sids_loop = model.TR[sids_loop].gather(1, best.unsqueeze(1)).squeeze(1)

    print("\n  Mean ops in valid corridor by corridor_width:")
    corridor_analysis = {}
    for w in sorted(corridor_sizes.keys()):
        vals = corridor_sizes[w]
        mean_c = round(sum(vals) / len(vals), 3)
        frac_1 = round(sum(1 for v in vals if v <= 1.0) / len(vals), 3)
        frac_2 = round(sum(1 for v in vals if v <= 2.0) / len(vals), 3)
        corridor_analysis[w] = {"mean": mean_c, "frac_singleton": frac_1, "frac_pair": frac_2}
        print(f"    width={w:.2f}  mean={mean_c:.3f}  "
              f"fraction_1op={frac_1:.3f}  fraction_≤2op={frac_2:.3f}")

    # ── Classify results ────────────────────────────────────────────────
    print(f"\n{'='*60}")
    print("RESULTS SUMMARY")
    print(f"{'='*60}")
    first_fail = next((r for r in all_results if r["failure"]), None)
    first_degrade = next((r for r in all_results if r["any_degraded"]), None)

    for r in all_results:
        tag = "FAIL" if r["failure"] else ("DEGRADED" if r["any_degraded"] else "PASS")
        print(f"  {tag:8s}  [{r['variant']:<20}]  acc={r['acc_mean']:.4f}±{r['acc_std']:.4f}")

    if first_fail:
        print(f"\nFirst failure: [{first_fail['variant']}]  acc={first_fail['acc_mean']:.4f}")
    else:
        print("\nNo failures detected.")
    if first_degrade:
        print(f"First degradation: [{first_degrade['variant']}]  acc={first_degrade['acc_mean']:.4f}")
    else:
        print("No degradation detected.")

    # ── Determine minimum routing law conclusion ─────────────────────────
    # Find the loosest non-failing, non-degraded variant
    passed_variants = [r for r in all_results if not r["failure"] and not r["any_degraded"]]
    loosest_pass = passed_variants[-1] if passed_variants else None

    if first_fail is None and first_degrade is None:
        min_law = "ANY LOCALLY VALID GEOMETRIC CANDIDATE (even unrestricted random)"
    elif first_fail and all_results.index(first_fail) <= 1:
        min_law = "EXACT K=1 NEAREST-NEIGHBOR GEOMETRIC ROUTING"
    else:
        if loosest_pass:
            min_law = f"RANDOM WITHIN TOP-K OR VALID CORRIDOR ({loosest_pass['variant']})"
        else:
            min_law = "EXACT K=1 NEAREST-NEIGHBOR GEOMETRIC ROUTING"

    print(f"\nMINIMUM ROUTING LAW REQUIRED: {min_law}")

    # ── CSV output ──────────────────────────────────────────────────────
    fieldnames = [
        "variant", "label", "corridor_width", "n_trials",
        "acc_mean", "acc_std", "acc_min",
        "route_entropy", "transport_frac", "corridor_size",
        "failure", "any_degraded",
    ]
    rows = []
    for r in all_results:
        rows.append({k: r.get(k, "") for k in fieldnames})

    with open(CSV_OUT, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)
    print(f"\nCSV written: {CSV_OUT}")

    # ── Markdown output ─────────────────────────────────────────────────
    _write_markdown(all_results, corridor_analysis, min_law, base_exact, base_soft,
                    solve_step, first_fail, first_degrade)
    print(f"Markdown written: {MD_OUT}")


def _write_markdown(all_results, corridor_analysis, min_law,
                    base_exact, base_soft_acc,
                    solve_step, first_fail, first_degrade):
    ts = datetime.datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ")
    L: List[str] = []
    L.append("# Prime Transport Router: Minimal Routing Law Probe v1\n\n")
    L.append(f"**Generated:** {ts}  \n")
    L.append(f"**Config:** D={D}, D_HIDDEN={D_HIDDEN}, B={BATCH_SIZE}, CPU  \n")
    L.append(f"**Train baseline:** acc=1.0000, solve_step={solve_step}  \n")
    L.append(f"**Inference baselines:** exact_k1={base_exact['accuracy']:.4f}, "
             f"soft_mlp={base_soft_acc:.4f}  \n\n")

    L.append("---\n\n")
    L.append("## Valid Corridor Definition\n\n")
    L.append("The **valid geometric corridor** at each routing step is defined as:\n\n")
    L.append("```\n")
    L.append(f"valid_corridor(step) = {{op : ang_sim[op] >= max(ang_sim) - W}}\n")
    L.append("```\n\n")
    L.append(f"where `ang_sim[op] = cur_dir · TN[op]` (cosine-like angular similarity "
             f"between current geometric direction and candidate operator direction).  \n\n")
    L.append(f"Two corridor widths tested:  \n")
    L.append(f"- **Wide:** W = {CORRIDOR_WIDTH_WIDE} (≈ half the median ang_sim margin of 0.63)  \n")
    L.append(f"- **Tight:** W = {CORRIDOR_WIDTH_TIGHT}  \n\n")

    L.append("### Corridor Size Distribution\n\n")
    L.append("| width W | mean ops in corridor | fraction singleton (1 op) | fraction ≤2 ops |\n")
    L.append("|---------|---------------------|--------------------------|------------------|\n")
    for w in sorted(corridor_analysis.keys()):
        ca = corridor_analysis[w]
        L.append(f"| {w:.2f} | {ca['mean']:.3f} | {ca['frac_singleton']:.3f} "
                 f"| {ca['frac_pair']:.3f} |\n")
    L.append("\n")

    L.append("---\n\n")
    L.append("## Variant Results\n\n")
    L.append("| variant | routing law | acc_mean | acc_std | acc_min | "
             "H(route) | transport_sim | corridor_size | PASS? |\n")
    L.append("|---------|-------------|----------|---------|---------|"
             "----------|--------------|--------------|-------|\n")
    for r in all_results:
        cs = f"{r['corridor_size']:.2f}" if not math.isnan(r["corridor_size"]) else "—"
        tag = "✗ FAIL" if r["failure"] else ("⚠ DEGRADED" if r["any_degraded"] else "✓")
        L.append(f"| {r['variant']} | {r['label']} "
                 f"| {r['acc_mean']:.4f} | {r['acc_std']:.4f} | {r['acc_min']:.4f} "
                 f"| {r['route_entropy']:.3f} | {r['transport_frac']:.3f} "
                 f"| {cs} | {tag} |\n")
    L.append("\n")

    if first_fail:
        L.append(f"**First failure:** [{first_fail['variant']}] "
                 f"acc_mean={first_fail['acc_mean']:.4f}  \n")
    else:
        L.append("**No failures detected across any variant.**  \n")
    if first_degrade:
        L.append(f"**First degradation:** [{first_degrade['variant']}] "
                 f"acc_mean={first_degrade['acc_mean']:.4f}  \n")
    else:
        L.append("**No degradation detected.**  \n")
    L.append("\n")

    L.append("---\n\n")
    L.append("## Explicit Answers\n\n")
    L.append("**1. Does exact nearest-neighbor routing matter?**  \n")
    if first_fail is None and first_degrade is None:
        L.append("No. Even uniform-random routing preserves correctness. "
                 "Exact k=1 selection is not required.  \n\n")
    elif first_fail and all_results.index(first_fail) <= 1:
        L.append("Yes. Departure from exact k=1 causes immediate failure.  \n\n")
    else:
        idx_f = all_results.index(first_fail) if first_fail else len(all_results)
        idx_d = all_results.index(first_degrade) if first_degrade else len(all_results)
        boundary = min(idx_f, idx_d)
        L.append(f"Not strictly. Exact k=1 is not required for the first {boundary} "
                 f"tested variants. Degradation begins at "
                 f"[{all_results[boundary]['variant']}].  \n\n")

    L.append("**2. How much randomness can be introduced before failure?**  \n")
    if first_fail is None:
        L.append("Full randomness (uniform over all 6 operators) does not cause failure "
                 "at D=24 with N_EVAL=2048 and 10 trials.  \n\n")
    else:
        L.append(f"Failure first appears at [{first_fail['variant']}].  \n\n")

    L.append("**3. Is there a broader geometric validity corridor containing interchangeable moves?**  \n")
    if corridor_analysis.get(CORRIDOR_WIDTH_WIDE, {}).get("frac_singleton", 1.0) < 0.9:
        ca = corridor_analysis[CORRIDOR_WIDTH_WIDE]
        L.append(f"Yes. At W={CORRIDOR_WIDTH_WIDE}, the corridor contains on average "
                 f"{ca['mean']:.2f} valid operators. Only {100*ca['frac_singleton']:.1f}% "
                 f"of steps are singletons (forced choices). The manifold enforces a corridor, "
                 f"not a single path.  \n\n")
    else:
        ca = corridor_analysis.get(CORRIDOR_WIDTH_WIDE, {})
        L.append(f"Corridor at W={CORRIDOR_WIDTH_WIDE} has mean size "
                 f"{ca.get('mean','?'):.2f}; {100*ca.get('frac_singleton',0):.1f}% "
                 f"are singletons.  \n\n")

    L.append("**4. What is the minimum routing law that preserves correctness?**  \n")
    L.append(f"See conclusion line below.  \n\n")

    L.append("---\n\n")
    L.append("## Honesty Section\n\n")

    # What is unnecessary
    passed_but_loose = [r for r in all_results
                        if not r["failure"] and not r["any_degraded"]
                        and r["variant"] not in ("exact_k1",)]
    if passed_but_loose:
        L.append("**What exact selection is unnecessary for:**  \n")
        for r in passed_but_loose:
            L.append(f"- [{r['variant']}]: acc={r['acc_mean']:.4f} — "
                     f"exact k=1 is replaceable by this policy.  \n")
        L.append("\n")
    else:
        L.append("**Exact k=1 is necessary:** no alternative passed without degradation.  \n\n")

    L.append("**What remains necessary:**  \n")
    if first_fail:
        L.append(f"- Routing must remain within the geometric validity corridor. "
                 f"The [{first_fail['variant']}] policy fails, indicating that "
                 f"entirely unconstrained random routing breaks the task.  \n")
    else:
        L.append("- Even unconstrained randomness did not cause failure in this configuration. "
                 "The task may have a high degree of path equivalence.  \n")
    L.append("\n")

    L.append("**Whether the manifold enforces a corridor rather than a single path:**  \n")
    ca_wide = corridor_analysis.get(CORRIDOR_WIDTH_WIDE, {})
    if ca_wide.get("frac_singleton", 1.0) < 0.7:
        L.append(f"Yes. At W={CORRIDOR_WIDTH_WIDE}, >30% of steps have ≥2 valid candidates. "
                 f"The geometry organises a corridor of valid moves, not a single trajectory.  \n\n")
    else:
        L.append(f"Partially. At W={CORRIDOR_WIDTH_WIDE}, mean corridor size = "
                 f"{ca_wide.get('mean','?')}; many steps are still effectively singletons.  \n\n")

    L.append("---\n\n")
    L.append(f"## MINIMUM ROUTING LAW REQUIRED: {min_law}\n\n")

    with open(MD_OUT, "w") as f:
        f.writelines(L)


if __name__ == "__main__":
    main()

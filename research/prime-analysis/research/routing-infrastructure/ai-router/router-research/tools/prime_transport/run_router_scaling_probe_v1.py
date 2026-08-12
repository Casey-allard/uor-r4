#!/usr/bin/env python3
"""run_router_scaling_probe_v1.py

Forward-scaling measurement sweep.

Objective: Identify the first configuration where the current system
(cache-loaded, hybrid angular+radial, pos0 bias fix) begins to degrade.

This is a measurement sweep only.
- No architecture changes
- No training rule changes
- No BFS, no traversal
- No thread policy changes
- No optimization

Axes:
  D        ∈ {24, 32, 40, 48}      problem depth
  D_HIDDEN ∈ {32, 64, 128}         model capacity
  B        ∈ {256, 512, 1024}      batch size

Metrics recorded per config:
  accuracy, solve_step, steps_per_sec, wall_time_s,
  alpha0, transport_fraction
"""
from __future__ import annotations

import csv
import math
import os
import random as pyrand
import sys
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
SCRIPT_DIR  = Path(__file__).parent
REPO_ROOT   = Path(__file__).resolve().parents[2]
RESULTS_DIR = REPO_ROOT / "results" / "prime_transport_recursive_system"
DOCS_DIR    = REPO_ROOT / "docs" / "research"
CACHE_DIR   = RESULTS_DIR / "state_cache"

CSV_OUT = RESULTS_DIR / "prime_transport_router_scaling_probe_v1.csv"
MD_OUT  = DOCS_DIR    / "prime_transport_router_scaling_probe_v1.md"

RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Locked constants — do not modify
# ═══════════════════════════════════════════════════════════════════════
GLOBAL_SEED   = 42
DEVICE        = "cpu"
VOCAB         = 4
D_EMB         = 4
D_TAU_OH      = 21
D_TAU_ANG     = 8
N_PHASE_PAIRS = 4
D_TAU_HYB     = D_TAU_ANG + N_PHASE_PAIRS   # 12
D_IN_HYB      = D_EMB + D_TAU_HYB            # 16
N_OPS         = 6
PHASE_BLOCKS  = [(0, 2, 2), (2, 7, 5), (7, 9, 2), (9, 21, 12)]
B0_INIT       = 2.0
INIT_SCALE    = 0.05

LR            = 0.02
TEMP_START    = 2.0
TEMP_END      = 0.1
CLIP_GRAD     = 1.0

# Transport operator threshold (ops with index >= 3 are "transport" ops)
TRANSPORT_TH  = 3

# Solve criterion
SOLVE_ACC     = 0.999

# ═══════════════════════════════════════════════════════════════════════
# Scaling axes
# ═══════════════════════════════════════════════════════════════════════
D_VALUES       = [24, 32, 40, 48]
D_HIDDEN_VALUES = [32, 64, 128]
BATCH_VALUES   = [256, 512, 1024]

# Training budget and evaluation schedule
MAX_STEPS  = 2000
EVAL_STEPS = [500, 1000, 1500, 2000]
N_EVAL     = 1024   # eval batch count (in samples)

# ═══════════════════════════════════════════════════════════════════════
# Cache load (no BFS, no traversal)
# ═══════════════════════════════════════════════════════════════════════
def load_cache() -> Dict:
    path = CACHE_DIR / "state_tables_full.pt"
    if not path.exists():
        print(f"ERROR: cache not found at {path}")
        print("  Run run_router_context_compaction_probe_v1.py first.")
        sys.exit(1)
    print(f"Loading cache: {path} ({path.stat().st_size / 1e6:.1f} MB)")
    t0 = time.perf_counter()
    data = torch.load(str(path), weights_only=False)
    elapsed = time.perf_counter() - t0
    print(f"  Loaded: {data['TN_oh'].shape[0]:,} states in {elapsed:.3f}s")
    return data


# ═══════════════════════════════════════════════════════════════════════
# Table conversion (from one-hot cache → hybrid angular format)
# Exact copy from run_router_context_compaction_probe_v1.py — no changes
# ═══════════════════════════════════════════════════════════════════════
def convert_onehot_to_angular(onehot: torch.Tensor) -> torch.Tensor:
    shape = onehot.shape[:-1]
    out = torch.zeros(*shape, D_TAU_ANG, dtype=onehot.dtype)
    ai = 0
    for s, e, m in PHASE_BLOCKS:      # 4 iterations — constant
        k = onehot[..., s:e].argmax(dim=-1).float()
        angle = 2.0 * math.pi * k / float(m)
        out[..., ai]     = torch.cos(angle)
        out[..., ai + 1] = torch.sin(angle)
        ai += 2
    return out


def prepare_hybrid_tables(
    TN_oh: torch.Tensor,
    tau0_oh: torch.Tensor,
    TR: torch.Tensor,
    pool_ids: torch.Tensor,
) -> Tuple[torch.Tensor, torch.Tensor, torch.Tensor, torch.Tensor]:
    TN_ang   = convert_onehot_to_angular(TN_oh)
    tau0_ang = convert_onehot_to_angular(tau0_oh)
    tau0_hyb = torch.cat(
        [tau0_ang, torch.ones(tau0_ang.shape[0], N_PHASE_PAIRS)], dim=1)
    return TN_ang, TR, tau0_hyb, pool_ids


# ═══════════════════════════════════════════════════════════════════════
# Model — exact architecture from run_router_context_compaction_probe_v1.py
# Only d_hidden and d_context are parameterized (for the scaling sweep)
# ═══════════════════════════════════════════════════════════════════════
class RouterAngularHybrid(nn.Module):
    """Hybrid angular+radial model — architecture locked, only dims swept."""

    def __init__(
        self,
        TN_ang: torch.Tensor,
        TR: torch.Tensor,
        tau0_hyb: torch.Tensor,
        pool_ids: torch.Tensor,
        d_hidden: int = 32,
        d_context: int = 24,
        b0_init: float = B0_INIT,
        init_scale: float = INIT_SCALE,
        seed: int = GLOBAL_SEED,
    ):
        super().__init__()
        dh  = d_hidden
        dha = max(8, dh // 4)
        d_tau = D_TAU_HYB   # 12
        d_in  = D_IN_HYB    # 16

        self.register_buffer("TN",          TN_ang)
        self.register_buffer("TR",          TR)
        self.register_buffer("tau0_table",  tau0_hyb)
        self.register_buffer("pool_ids",    pool_ids)
        m = torch.zeros(1, d_context)
        m[0, 0] = 1.0
        self.register_buffer("pos0_mask", m)

        self.b_pos0 = nn.Parameter(torch.tensor(b0_init))

        gen = torch.Generator().manual_seed(seed)

        def rp(*sh: int) -> nn.Parameter:
            return nn.Parameter(
                torch.empty(*sh).normal_(0, init_scale, generator=gen))

        def zp(*sh: int) -> nn.Parameter:
            return nn.Parameter(torch.zeros(*sh))

        self.W_emb        = rp(VOCAB, D_EMB)
        self.W1           = rp(d_in, dh);     self.b1 = zp(dh)
        self.W2           = rp(dh, N_OPS);    self.b2 = zp(N_OPS)
        self.W_attn       = rp(dha, d_tau);   self.b_attn = zp(dha)
        self.v_attn       = rp(dha)
        self.W_pred       = rp(d_tau, VOCAB); self.b_pred = zp(VOCAB)
        self.W_tok_inject = rp(VOCAB, d_tau)

        self._d_context = d_context

    def forward(
        self,
        state_ids: torch.Tensor,
        tokens: torch.Tensor,
        x0: torch.Tensor,
        temp: float,
    ) -> torch.Tensor:
        B     = state_ids.shape[0]
        D_len = tokens.shape[1]
        tau_prev = self.tau0_table[state_ids]     # (B, 12)

        soft_taus: List[torch.Tensor] = []
        for t in range(D_len):
            tn   = self.TN[state_ids]             # (B, 6, 8)
            embs = self.W_emb[tokens[:, t]]       # (B, 4)
            h    = torch.tanh(
                torch.cat([embs, tau_prev], dim=1) @ self.W1 + self.b1)
            logits = h @ self.W2 + self.b2        # (B, 6)

            if self.training:
                u = torch.rand_like(logits).clamp_(1e-20, 1.0)
                w = torch.softmax(
                    (logits - torch.log(-torch.log(u))) / temp, dim=1)
            else:
                w = torch.softmax(logits / 0.05, dim=1)

            base  = torch.einsum("bi,bij->bj", w, tn)    # (B, 8)
            pairs = base.view(B, N_PHASE_PAIRS, 2)
            mag   = (pairs * pairs).sum(dim=2).sqrt()     # (B, 4)
            mag_s = mag.clamp(min=1e-8)
            dirn  = (pairs / mag_s.unsqueeze(2)).view(B, D_TAU_ANG)
            hybrid = torch.cat([dirn, mag], dim=1)        # (B, 12)

            tau_prev = (hybrid + self.W_tok_inject[x0]) if t == 0 else hybrid
            soft_taus.append(tau_prev)

            state_ids = self.TR[state_ids].gather(
                1, torch.argmax(w, dim=1).unsqueeze(1)).squeeze(1)

        st    = torch.stack(soft_taus, dim=1)             # (B, D, 12)
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = (h_a * self.v_attn).sum(dim=-1) + self.pos0_mask * self.b_pos0
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred


# ═══════════════════════════════════════════════════════════════════════
# Batch sampling — exact copy, parameterized
# ═══════════════════════════════════════════════════════════════════════
def make_batch(
    pool_ids: torch.Tensor,
    D: int,
    B: int,
    rng: torch.Generator,
) -> Tuple[torch.Tensor, torch.Tensor, torch.Tensor]:
    idx  = torch.randint(pool_ids.shape[0], (B,), generator=rng)
    sids = pool_ids[idx]
    x0   = torch.randint(VOCAB, (B,), generator=rng)
    toks = torch.randint(VOCAB, (B, D), generator=rng)
    toks[:, 0] = x0
    return sids, toks, x0


# ═══════════════════════════════════════════════════════════════════════
# Evaluation with alpha0 and transport_fraction
# ═══════════════════════════════════════════════════════════════════════
def evaluate(
    model: RouterAngularHybrid,
    pool_ids: torch.Tensor,
    D: int,
    n_eval: int,
) -> Dict:
    """Eval mode pass — records accuracy, alpha0, transport_fraction."""
    model.eval()
    rng  = torch.Generator().manual_seed(GLOBAL_SEED + D + 17)
    B_ev = 256
    n_batches = max(1, n_eval // B_ev)

    correct     = 0
    total       = 0
    alpha_sum   = torch.zeros(D)
    transport_n = 0
    total_steps = 0

    with torch.no_grad():
        for _ in range(n_batches):
            sids, toks, x0 = make_batch(pool_ids, D, B_ev, rng)
            tau_prev  = model.tau0_table[sids]
            cur_sids  = sids.clone()
            soft_taus: List[torch.Tensor] = []

            for t in range(D):
                tn   = model.TN[cur_sids]
                embs = model.W_emb[toks[:, t]]
                h    = torch.tanh(
                    torch.cat([embs, tau_prev], dim=1) @ model.W1 + model.b1)
                logits = h @ model.W2 + model.b2
                w      = torch.softmax(logits / 0.05, dim=1)

                k_hard = w.argmax(dim=1)
                transport_n += int((k_hard >= TRANSPORT_TH).sum().item())
                total_steps += B_ev

                base  = torch.einsum("bi,bij->bj", w, tn)
                pairs = base.view(B_ev, N_PHASE_PAIRS, 2)
                mag   = (pairs * pairs).sum(dim=2).sqrt()
                mag_s = mag.clamp(min=1e-8)
                dirn  = (pairs / mag_s.unsqueeze(2)).view(B_ev, D_TAU_ANG)
                hybrid = torch.cat([dirn, mag], dim=1)

                tau_prev = (hybrid + model.W_tok_inject[x0]) if t == 0 else hybrid
                soft_taus.append(tau_prev)
                cur_sids = model.TR[cur_sids].gather(
                    1, k_hard.unsqueeze(1)).squeeze(1)

            st    = torch.stack(soft_taus, dim=1)
            h_a   = torch.tanh(st @ model.W_attn.t() + model.b_attn)
            sc    = ((h_a * model.v_attn).sum(dim=-1)
                     + model.pos0_mask * model.b_pos0)
            alpha = torch.softmax(sc, dim=1)
            alpha_sum += alpha.sum(dim=0).cpu()

            pred = torch.einsum("bd,bdt->bt", alpha, st) @ model.W_pred + model.b_pred
            correct += int((pred.argmax(1) == x0).sum().item())
            total   += B_ev

    model.train()
    accuracy  = correct / max(total, 1)
    alpha0    = float((alpha_sum[0] / max(total, 1)).item())
    trans_frac = transport_n / max(total_steps, 1)
    return {
        "accuracy":           accuracy,
        "alpha0":             alpha0,
        "transport_fraction": trans_frac,
    }


# ═══════════════════════════════════════════════════════════════════════
# Single-configuration training run
# ═══════════════════════════════════════════════════════════════════════
def run_config(
    TN_ang: torch.Tensor,
    TR: torch.Tensor,
    tau0_hyb: torch.Tensor,
    pool_ids: torch.Tensor,
    D: int,
    d_hidden: int,
    batch_size: int,
) -> Dict:
    """Train one (D, d_hidden, batch_size) configuration and return metrics."""
    model = RouterAngularHybrid(
        TN_ang, TR, tau0_hyb, pool_ids,
        d_hidden=d_hidden, d_context=D,
        seed=GLOBAL_SEED,
    )
    n_params = sum(p.numel() for p in model.parameters())

    opt = torch.optim.SGD(model.parameters(), lr=LR)
    rng = torch.Generator().manual_seed(GLOBAL_SEED + D * 7 + d_hidden * 3)
    model.train()

    solve_step:   Optional[int] = None
    final_acc:    float = 0.0
    final_alpha0: float = 0.0
    final_tfrac:  float = 0.0
    checkpoints:  List[Dict] = []

    eval_set = set(EVAL_STEPS)
    t0 = time.perf_counter()

    for step in range(1, MAX_STEPS + 1):
        frac = step / max(MAX_STEPS - 1, 1)
        temp = float(TEMP_START * (TEMP_END / TEMP_START) ** frac)
        sids, toks, x0 = make_batch(pool_ids, D, batch_size, rng)
        logits = model(sids, toks, x0, temp)
        loss   = F.cross_entropy(logits, x0)
        opt.zero_grad()
        loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), CLIP_GRAD)
        opt.step()

        if step in eval_set:
            met = evaluate(model, pool_ids, D, N_EVAL)
            acc  = met["accuracy"]
            a0   = met["alpha0"]
            tfrac = met["transport_fraction"]
            elapsed = time.perf_counter() - t0
            sps = step / elapsed

            if acc >= SOLVE_ACC and solve_step is None:
                solve_step = step

            checkpoints.append({
                "step":               step,
                "accuracy":           round(acc, 4),
                "alpha0":             round(a0, 4),
                "transport_fraction": round(tfrac, 4),
                "steps_per_sec":      round(sps, 1),
                "elapsed_s":          round(elapsed, 2),
            })

            final_acc    = acc
            final_alpha0 = a0
            final_tfrac  = tfrac

            # Progress line
            solved_tag = " ✓SOLVED" if solve_step == step else ""
            print(
                f"    step={step:>4}  acc={acc:.4f}  "
                f"alpha0={a0:.4f}  tfrac={tfrac:.4f}  "
                f"sps={sps:.1f}{solved_tag}",
                flush=True,
            )

    total_wall = time.perf_counter() - t0
    final_sps  = MAX_STEPS / total_wall

    # Classify outcome
    if solve_step is not None:
        outcome = "SOLVED"
    elif final_acc >= 0.80:
        outcome = "PARTIAL"
    elif final_acc >= 0.40:
        outcome = "DEGRADED"
    else:
        outcome = "FAILED"

    return {
        "D":                   D,
        "d_hidden":            d_hidden,
        "batch_size":          batch_size,
        "n_params":            n_params,
        "solve_step":          solve_step if solve_step else "DNF",
        "final_acc":           round(final_acc, 4),
        "final_alpha0":        round(final_alpha0, 4),
        "final_tfrac":         round(final_tfrac, 4),
        "steps_per_sec":       round(final_sps, 1),
        "total_wall_s":        round(total_wall, 2),
        "outcome":             outcome,
        "checkpoints":         checkpoints,
    }


# ═══════════════════════════════════════════════════════════════════════
# Main sweep
# ═══════════════════════════════════════════════════════════════════════
def main():
    print("=" * 70)
    print("ROUTER SCALING PROBE v1")
    print("=" * 70)
    print(f"D values:        {D_VALUES}")
    print(f"D_HIDDEN values: {D_HIDDEN_VALUES}")
    print(f"Batch values:    {BATCH_VALUES}")
    print(f"Max steps:       {MAX_STEPS}")
    print(f"Eval at steps:   {EVAL_STEPS}")
    total_configs = len(D_VALUES) * len(D_HIDDEN_VALUES) * len(BATCH_VALUES)
    print(f"Total configs:   {total_configs}")
    print()

    # Load cache once — no BFS
    cache = load_cache()
    TN_ang, TR, tau0_hyb, pool_ids = prepare_hybrid_tables(
        cache["TN_oh"], cache["tau0_oh"], cache["TR"], cache["pool_ids"]
    )
    n_states = TN_ang.shape[0]
    print(f"Tables ready: TN={tuple(TN_ang.shape)}, TR={tuple(TR.shape)}")
    print()

    all_results: List[Dict] = []
    config_idx = 0

    for D in D_VALUES:
        for d_hidden in D_HIDDEN_VALUES:
            for batch_size in BATCH_VALUES:
                config_idx += 1
                print(
                    f"[{config_idx}/{total_configs}] "
                    f"D={D}  D_HIDDEN={d_hidden}  B={batch_size}",
                    flush=True,
                )
                result = run_config(
                    TN_ang, TR, tau0_hyb, pool_ids,
                    D=D, d_hidden=d_hidden, batch_size=batch_size,
                )
                all_results.append(result)
                print(
                    f"  → outcome={result['outcome']}  "
                    f"solve_step={result['solve_step']}  "
                    f"final_acc={result['final_acc']}  "
                    f"sps={result['steps_per_sec']}",
                    flush=True,
                )
                print()

    # ──────────────────────────────────────────────────────────────────
    # Write CSV
    # ──────────────────────────────────────────────────────────────────
    fieldnames = [
        "D", "d_hidden", "batch_size", "n_params",
        "solve_step", "final_acc",
        "final_alpha0", "final_tfrac",
        "steps_per_sec", "total_wall_s", "outcome",
        "step_500_acc", "step_1000_acc", "step_1500_acc", "step_2000_acc",
        "step_500_alpha0", "step_1000_alpha0",
        "step_500_tfrac", "step_1000_tfrac",
    ]

    def _ckpt_val(result: Dict, step: int, key: str, default=""):
        for c in result["checkpoints"]:
            if c["step"] == step:
                return c[key]
        return default

    csv_rows = []
    for r in all_results:
        row = {k: r[k] for k in [
            "D", "d_hidden", "batch_size", "n_params",
            "solve_step", "final_acc", "final_alpha0", "final_tfrac",
            "steps_per_sec", "total_wall_s", "outcome",
        ]}
        for step in [500, 1000, 1500, 2000]:
            row[f"step_{step}_acc"]    = _ckpt_val(r, step, "accuracy")
            row[f"step_{step}_alpha0"] = _ckpt_val(r, step, "alpha0")
            row[f"step_{step}_tfrac"]  = _ckpt_val(r, step, "transport_fraction")
        csv_rows.append(row)

    with open(CSV_OUT, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fieldnames)
        w.writeheader()
        w.writerows(csv_rows)
    print(f"CSV written: {CSV_OUT}")

    # ──────────────────────────────────────────────────────────────────
    # Determine scaling limit
    # ──────────────────────────────────────────────────────────────────
    first_degraded = None
    first_failed   = None
    for r in all_results:
        if r["outcome"] in ("DEGRADED", "FAILED", "PARTIAL") and first_degraded is None:
            first_degraded = r
        if r["outcome"] in ("DEGRADED", "FAILED") and first_failed is None:
            first_failed = r

    if first_failed:
        limit_config = first_failed
        limit_tag    = "CONVERGENCE_FAILURE"
    elif first_degraded:
        limit_config = first_degraded
        limit_tag    = "PARTIAL_DEGRADATION"
    else:
        limit_config = all_results[-1]
        limit_tag    = "NO_DEGRADATION_OBSERVED"

    limit_str = (
        f"D={limit_config['D']}  "
        f"D_HIDDEN={limit_config['d_hidden']}  "
        f"B={limit_config['batch_size']}  "
        f"outcome={limit_config['outcome']}"
    )

    print()
    print("=" * 70)
    print("SCALING LIMIT")
    print("=" * 70)
    print(f"  {limit_tag}")
    print(f"  SCALING LIMIT OBSERVED AT: {limit_str}")
    print()

    # ──────────────────────────────────────────────────────────────────
    # Write markdown
    # ──────────────────────────────────────────────────────────────────
    write_markdown(all_results, limit_config, limit_tag, limit_str, n_states)
    print(f"Markdown written: {MD_OUT}")


# ═══════════════════════════════════════════════════════════════════════
# Markdown report
# ═══════════════════════════════════════════════════════════════════════
def write_markdown(
    results: List[Dict],
    limit_config: Dict,
    limit_tag: str,
    limit_str: str,
    n_states: int,
) -> None:
    L: List[str] = []
    A = L.append

    A("# Prime Transport Router — Scaling Probe v1")
    A("")
    A("> **Objective**: Identify the first configuration where the current")
    A("> cache-loaded hybrid angular+radial system degrades.")
    A("> This is a measurement sweep only — no changes to architecture or training.")
    A("")
    A("---")
    A("")

    # Config summary
    A("## Configuration")
    A("")
    A("| Parameter | Values |")
    A("|-----------|--------|")
    A(f"| Problem depth D | {D_VALUES} |")
    A(f"| Model capacity D_HIDDEN | {D_HIDDEN_VALUES} |")
    A(f"| Batch size B | {BATCH_VALUES} |")
    A(f"| Max training steps | {MAX_STEPS} |")
    A(f"| Eval checkpoints | {EVAL_STEPS} |")
    A(f"| Solve criterion (acc ≥) | {SOLVE_ACC} |")
    A(f"| Cache states | {n_states:,} |")
    A(f"| Total configurations | {len(D_VALUES) * len(D_HIDDEN_VALUES) * len(BATCH_VALUES)} |")
    A("")
    A("Locked: LR=0.02, TEMP_START=2.0, TEMP_END=0.1, CLIP_GRAD=1.0, SGD optimizer,")
    A("pos0 bias fix enabled, hybrid angular+radial representation.")
    A("")

    # Full results table
    A("## 1. Full Results Table")
    A("")
    A("| D | D_HIDDEN | B | solve_step | final_acc | alpha0 | transport_frac | sps | wall_s | outcome |")
    A("|---|----------|---|-----------|-----------|--------|----------------|-----|--------|---------|")
    for r in results:
        ss = str(r["solve_step"])
        outcome_icon = {"SOLVED": "✓", "PARTIAL": "~", "DEGRADED": "↓", "FAILED": "✗"}.get(r["outcome"], "?")
        A(f"| {r['D']} | {r['d_hidden']} | {r['batch_size']} "
          f"| {ss} | {r['final_acc']:.4f} | {r['final_alpha0']:.4f} "
          f"| {r['final_tfrac']:.4f} | {r['steps_per_sec']:.1f} "
          f"| {r['total_wall_s']:.1f} | {outcome_icon} {r['outcome']} |")
    A("")

    # Outcome legend
    A("**Outcome codes:** ✓ SOLVED (acc ≥ 0.999)  ~ PARTIAL (acc ≥ 0.80)  "
      "↓ DEGRADED (acc ≥ 0.40)  ✗ FAILED (acc < 0.40)")
    A("")

    # Convergence curves by D
    A("## 2. Convergence Curves (accuracy by checkpoint step)")
    A("")
    for D in D_VALUES:
        d_results = [r for r in results if r["D"] == D]
        if not d_results:
            continue
        A(f"### D = {D}")
        A("")
        header = "| D_HIDDEN | B |"
        for step in EVAL_STEPS:
            header += f" step {step} |"
        A(header)
        sep = "|----------|---|"
        for step in EVAL_STEPS:
            sep += "---------|"
        A(sep)
        for r in d_results:
            row = f"| {r['d_hidden']} | {r['batch_size']} |"
            for step in EVAL_STEPS:
                found = False
                for c in r["checkpoints"]:
                    if c["step"] == step:
                        row += f" {c['accuracy']:.4f} |"
                        found = True
                        break
                if not found:
                    row += " — |"
            A(row)
        A("")

    # Alpha0 and transport_fraction at final step
    A("## 3. Metrics at Final Step")
    A("")
    A("| D | D_HIDDEN | B | final_acc | alpha0 | transport_frac | outcome |")
    A("|---|----------|---|-----------|--------|----------------|---------|")
    for r in results:
        A(f"| {r['D']} | {r['d_hidden']} | {r['batch_size']} "
          f"| {r['final_acc']:.4f} | {r['final_alpha0']:.4f} "
          f"| {r['final_tfrac']:.4f} | {r['outcome']} |")
    A("")

    # Steps per second
    A("## 4. Throughput (steps/sec)")
    A("")
    A("| D | D_HIDDEN | B | sps | wall_s |")
    A("|---|----------|---|-----|--------|")
    for r in results:
        A(f"| {r['D']} | {r['d_hidden']} | {r['batch_size']} "
          f"| {r['steps_per_sec']:.1f} | {r['total_wall_s']:.1f} |")
    A("")

    # First degradation
    A("## 5. Performance Degradation Analysis")
    A("")
    solved   = [r for r in results if r["outcome"] == "SOLVED"]
    partial  = [r for r in results if r["outcome"] == "PARTIAL"]
    degraded = [r for r in results if r["outcome"] == "DEGRADED"]
    failed   = [r for r in results if r["outcome"] == "FAILED"]

    A(f"| Outcome | Count |")
    A(f"|---------|-------|")
    A(f"| SOLVED (acc ≥ 0.999) | {len(solved)} |")
    A(f"| PARTIAL (0.80 ≤ acc < 0.999) | {len(partial)} |")
    A(f"| DEGRADED (0.40 ≤ acc < 0.80) | {len(degraded)} |")
    A(f"| FAILED (acc < 0.40) | {len(failed)} |")
    A("")

    if limit_tag == "NO_DEGRADATION_OBSERVED":
        A("No degradation observed within the tested configuration space.")
        A(f"All {len(results)} configurations solved or partially solved.")
    else:
        A(f"**First non-SOLVED configuration:**")
        A("")
        A(f"```")
        A(f"D        = {limit_config['D']}")
        A(f"D_HIDDEN = {limit_config['d_hidden']}")
        A(f"B        = {limit_config['batch_size']}")
        A(f"outcome  = {limit_config['outcome']}")
        A(f"final_acc = {limit_config['final_acc']:.4f}")
        A(f"solve_step = {limit_config['solve_step']}")
        A(f"alpha0   = {limit_config['final_alpha0']:.4f}")
        A(f"tfrac    = {limit_config['final_tfrac']:.4f}")
        A(f"```")
        A("")

    # Failure mode classification
    A("### Failure Mode Classification")
    A("")
    if first_failed := next((r for r in results if r["outcome"] == "FAILED"), None):
        # Determine which type of failure
        # Check if acc drops after initially rising = instability
        # Check if acc never rises = convergence failure
        # Check if sps collapses = throughput
        ckpts = first_failed["checkpoints"]
        accs = [c["accuracy"] for c in ckpts]
        if accs:
            max_acc = max(accs) if accs else 0
            last_acc = accs[-1] if accs else 0
            if max_acc > 0.5 and last_acc < max_acc * 0.8:
                failure_type = "INSTABILITY (accuracy rose then collapsed)"
            elif max_acc < 0.3:
                failure_type = "CONVERGENCE_FAILURE (accuracy never rose above random)"
            else:
                failure_type = "CONVERGENCE_FAILURE (accuracy stalled below threshold)"
        else:
            failure_type = "UNKNOWN"
        A(f"- **First FAILED config**: D={first_failed['D']}, "
          f"D_HIDDEN={first_failed['d_hidden']}, B={first_failed['batch_size']}")
        A(f"- **Failure type**: {failure_type}")
        A(f"- final_acc={first_failed['final_acc']:.4f}")
    elif partial:
        # Is throughput the issue? Check sps for large configs
        baseline_sps = results[0]["steps_per_sec"] if results else 1.0
        slow_configs = [r for r in results if r["steps_per_sec"] < baseline_sps * 0.1]
        if slow_configs:
            A("- **Failure type**: THROUGHPUT COLLAPSE (sps dropped > 10× from baseline)")
            A(f"- Baseline sps={baseline_sps:.1f}")
            for sc in slow_configs[:3]:
                A(f"  - D={sc['D']}, D_HIDDEN={sc['d_hidden']}, "
                  f"B={sc['batch_size']}: sps={sc['steps_per_sec']:.1f}")
        else:
            A("- **Failure type**: CONVERGENCE_FAILURE (did not reach 0.999)")
    else:
        A("- No clear failure type identified within tested range.")
    A("")

    # Final classification
    A("## 6. Final Classification")
    A("")
    A("```")
    A(f"SCALING LIMIT OBSERVED AT: {limit_str}")
    A("```")
    A("")
    A(f"**Limit type**: {limit_tag}")
    A("")

    # Summary of degradation pattern
    A("### Degradation Pattern Summary")
    A("")
    # Group by D, show whether D_HIDDEN=32 solves at each D
    A("| D | D_HIDDEN=32 | D_HIDDEN=64 | D_HIDDEN=128 |")
    A("|---|------------|------------|-------------|")
    for D in D_VALUES:
        row = f"| {D} |"
        for dh in D_HIDDEN_VALUES:
            # Find any B for this (D, dh)
            matching = [r for r in results if r["D"] == D and r["d_hidden"] == dh]
            if matching:
                outcomes = [r["outcome"] for r in matching]
                accs = [r["final_acc"] for r in matching]
                best_acc = max(accs)
                best_outcome = matching[accs.index(best_acc)]["outcome"]
                row += f" {best_outcome} ({best_acc:.3f}) |"
            else:
                row += " — |"
        A(row)
    A("")

    with open(MD_OUT, "w") as f:
        f.write("\n".join(L) + "\n")


if __name__ == "__main__":
    main()

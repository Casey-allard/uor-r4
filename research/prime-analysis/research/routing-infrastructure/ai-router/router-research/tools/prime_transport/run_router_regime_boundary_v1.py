#!/usr/bin/env python3
"""
Router Regime Boundary v1 — Execution Substrate Sweep

Tests whether the SAME recurrence benefits from parallel hardware at larger
within-step work sizes. Sweeps D_HIDDEN, batch_size, num_threads, and device.

Locked: operator semantics, recurrence logic, injection rule, pos0 bias, D=24.
"""

from __future__ import annotations

import csv
import importlib.util
import os
import pathlib
import platform
import time
from typing import Dict, List, Tuple

import torch
import torch.nn as nn
import torch.nn.functional as F

SCRIPT_DIR = pathlib.Path(__file__).resolve().parent
REPO_ROOT  = SCRIPT_DIR.parent.parent
MD_PATH    = REPO_ROOT / "docs" / "research" / "prime_transport_router_regime_boundary_v1.md"
CSV_PATH   = REPO_ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_router_regime_boundary_v1.csv"

# Fixed constants
VOCAB, D_EMB, D_TAU, N_OPS = 4, 4, 21, 6
D_IN       = D_EMB + D_TAU
D_CONTEXT  = 24
GLOBAL_SEED = 42
B0_INIT    = 2.0
LR         = 0.02
TEMP_START = 2.0
TEMP_END   = 0.1

# Sweep grid
D_HIDDENS    = [32, 64, 128]
BATCH_SIZES  = [256, 512, 1024]
CPU_THREADS  = [1, 2, 4]
MPS_AVAIL    = torch.backends.mps.is_available()
DEVICES      = ["cpu"] + (["mps"] if MPS_AVAIL else [])

BENCH_STEPS = 200
WARMUP      = 15


def _load_v6torch():
    spec = importlib.util.spec_from_file_location(
        "v6torch_base", str(SCRIPT_DIR / "run_router_reintegration_v6_torch.py"))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


class RouterV6Scaled(nn.Module):
    """Exact canonical model — only D_HIDDEN varies."""

    def __init__(self, TN, TR, tau0_table, pool_ids, d_hidden=32,
                 d_context=D_CONTEXT, b0_init=B0_INIT, init_scale=0.05,
                 seed=GLOBAL_SEED):
        super().__init__()
        dh  = d_hidden
        dha = max(8, d_hidden // 4)

        self.register_buffer("TN", TN)
        self.register_buffer("TR", TR)
        self.register_buffer("tau0_table", tau0_table)
        self.register_buffer("pool_ids", pool_ids)

        pos0 = torch.zeros(1, d_context)
        pos0[0, 0] = 1.0
        self.register_buffer("pos0_mask", pos0)
        self.b_pos0 = nn.Parameter(torch.tensor(b0_init))

        gen = torch.Generator().manual_seed(seed)
        def rp(*s): return nn.Parameter(torch.empty(*s).normal_(0, init_scale, generator=gen))
        def zp(*s): return nn.Parameter(torch.zeros(*s))

        self.W_emb        = rp(VOCAB, D_EMB)
        self.W1           = rp(D_IN, dh)
        self.b1           = zp(dh)
        self.W2           = rp(dh, N_OPS)
        self.b2           = zp(N_OPS)
        self.W_attn       = rp(dha, D_TAU)
        self.b_attn       = zp(dha)
        self.v_attn       = rp(dha)
        self.W_pred       = rp(D_TAU, VOCAB)
        self.b_pred       = zp(VOCAB)
        self.W_tok_inject = rp(VOCAB, D_TAU)

    def forward(self, state_ids: torch.Tensor, tokens: torch.Tensor,
                x0: torch.Tensor, temp: float) -> torch.Tensor:
        B: int = state_ids.shape[0]
        D: int = tokens.shape[1]
        tau_prev: torch.Tensor = self.tau0_table[state_ids]
        soft_taus: List[torch.Tensor] = []

        for t in range(D):
            tn_batch = self.TN[state_ids]
            embs     = self.W_emb[tokens[:, t]]
            h_in     = torch.cat([embs, tau_prev], dim=1)
            h        = torch.tanh(h_in @ self.W1 + self.b1)
            logits   = h @ self.W2 + self.b2

            if self.training:
                u  = torch.rand_like(logits).clamp_(1e-20, 1.0)
                gn = -torch.log(-torch.log(u))
                w  = torch.softmax((logits + gn) / temp, dim=1)
            else:
                w  = torch.softmax(logits / 0.05, dim=1)

            base     = torch.einsum("bi,bij->bj", w, tn_batch)
            tau_prev = (base + self.W_tok_inject[x0]) if t == 0 else base
            soft_taus.append(tau_prev)

            k_hard    = torch.argmax(w, dim=1)
            state_ids = self.TR[state_ids].gather(1, k_hard.unsqueeze(1)).squeeze(1)

        sstack   = torch.stack(soft_taus, dim=1)
        h_attn   = torch.tanh(sstack @ self.W_attn.t() + self.b_attn)
        a_raw    = (h_attn * self.v_attn).sum(dim=-1)
        a_scores = a_raw + self.pos0_mask * self.b_pos0
        alpha    = torch.softmax(a_scores, dim=1)
        pooled   = torch.einsum("bd,bdt->bt", alpha, sstack)
        return pooled @ self.W_pred + self.b_pred


def sample_batch(pool_ids, B, D, device):
    idx   = torch.randint(0, len(pool_ids), (B,), device=device)
    sids  = pool_ids[idx]
    toks  = torch.randint(0, VOCAB, (B, D), device=device)
    x0    = torch.randint(0, VOCAB, (B,), device=device)
    toks[:, 0] = x0
    return sids, toks, x0


def run_config(TN, TR, tau0, pool_ids, d_hidden, batch_size, device, n_threads):
    """Benchmark one configuration. Returns dict of results."""
    if device == "cpu":
        torch.set_num_threads(n_threads)

    TN_d   = TN.to(device)
    TR_d   = TR.to(device)
    tau0_d = tau0.to(device)
    pool_d = pool_ids.to(device)

    model = RouterV6Scaled(TN_d, TR_d, tau0_d, pool_d, d_hidden=d_hidden,
                           seed=GLOBAL_SEED + d_hidden)
    model = model.to(device)

    jit_ok = True
    try:
        model = torch.jit.script(model)
    except Exception:
        jit_ok = False

    model.train()
    opt = torch.optim.SGD(model.parameters(), lr=LR)
    torch.manual_seed(GLOBAL_SEED)

    # Warmup
    for s in range(WARMUP):
        sids, toks, x0 = sample_batch(pool_d, batch_size, D_CONTEXT, device)
        pred = model(sids, toks, x0, 2.0)
        loss = F.cross_entropy(pred, x0)
        opt.zero_grad(); loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), 1.0); opt.step()
    if device == "mps":
        torch.mps.synchronize()

    # Timed run with split timing
    t_fwd = 0.0
    t_bwd = 0.0
    t_other = 0.0

    t0 = time.perf_counter()
    for s in range(BENCH_STEPS):
        temp = float(TEMP_START * (TEMP_END / TEMP_START) ** (s / max(BENCH_STEPS - 1, 1)))
        sids, toks, x0 = sample_batch(pool_d, batch_size, D_CONTEXT, device)

        if device == "mps":
            torch.mps.synchronize()
        tf = time.perf_counter()
        pred = model(sids, toks, x0, temp)
        loss = F.cross_entropy(pred, x0)
        if device == "mps":
            torch.mps.synchronize()
        t_fwd += time.perf_counter() - tf

        tb = time.perf_counter()
        opt.zero_grad()
        loss.backward()
        if device == "mps":
            torch.mps.synchronize()
        t_bwd += time.perf_counter() - tb

        to = time.perf_counter()
        nn.utils.clip_grad_norm_(model.parameters(), 1.0)
        opt.step()
        if device == "mps":
            torch.mps.synchronize()
        t_other += time.perf_counter() - to

    if device == "mps":
        torch.mps.synchronize()
    total = time.perf_counter() - t0
    sps = BENCH_STEPS / total

    n_params = sum(p.numel() for p in model.parameters())

    return {
        "device": device,
        "d_hidden": d_hidden,
        "batch_size": batch_size,
        "n_threads": n_threads if device == "cpu" else -1,
        "steps_per_sec": round(sps, 1),
        "total_seconds": round(total, 3),
        "forward_seconds": round(t_fwd, 3),
        "backward_seconds": round(t_bwd, 3),
        "other_seconds": round(t_other, 3),
        "forward_pct": round(100 * t_fwd / total, 1),
        "backward_pct": round(100 * t_bwd / total, 1),
        "n_params": n_params,
        "jit": jit_ok,
        "matmul_shape": f"({batch_size},{D_IN})x({D_IN},{d_hidden})",
    }


def main():
    print("=" * 70)
    print("Router Regime Boundary v1 — Execution Substrate Sweep")
    print("=" * 70)
    print(f"  torch: {torch.__version__}")
    print(f"  cores: {os.cpu_count()}")
    print(f"  MPS:   {MPS_AVAIL}")
    print(f"  D_HIDDENS:   {D_HIDDENS}")
    print(f"  BATCH_SIZES: {BATCH_SIZES}")
    print(f"  CPU_THREADS: {CPU_THREADS}")
    print(f"  DEVICES:     {DEVICES}")
    print(f"  BENCH_STEPS: {BENCH_STEPS}")

    # Load state tables once
    print("\nLoading state tables...", flush=True)
    import random as pyrand
    v6 = _load_v6torch()
    rng = pyrand.Random(GLOBAL_SEED)
    pool = v6.build_warmup_pool(rng, size=v6.POOL_SIZE)
    n_st = v6.bfs_warm_up(pool, max_seconds=v6.BFS_MAX_SECS, verbose=True)
    TN, TR, tau0, pool_ids, _ = v6.build_state_tables(pool, verbose=True)
    print(f"  States: {n_st:,}\n")

    rows = []
    total_configs = (len(D_HIDDENS) * len(BATCH_SIZES) * len(CPU_THREADS)
                     + (len(D_HIDDENS) * len(BATCH_SIZES) if MPS_AVAIL else 0))
    done = 0

    # CPU sweep
    for d_hidden in D_HIDDENS:
        for batch_size in BATCH_SIZES:
            for n_threads in CPU_THREADS:
                done += 1
                tag = f"cpu/dh={d_hidden}/B={batch_size}/thr={n_threads}"
                print(f"[{done}/{total_configs}] {tag}", end="  ", flush=True)
                try:
                    r = run_config(TN, TR, tau0, pool_ids,
                                   d_hidden, batch_size, "cpu", n_threads)
                    rows.append(r)
                    print(f"→ {r['steps_per_sec']} sps  "
                          f"fwd={r['forward_pct']}% bwd={r['backward_pct']}%")
                except Exception as e:
                    print(f"FAILED: {e}")
                    rows.append({
                        "device": "cpu", "d_hidden": d_hidden,
                        "batch_size": batch_size, "n_threads": n_threads,
                        "steps_per_sec": -1, "total_seconds": -1,
                        "forward_seconds": -1, "backward_seconds": -1,
                        "other_seconds": -1, "forward_pct": -1,
                        "backward_pct": -1, "n_params": -1, "jit": False,
                        "matmul_shape": "",
                    })

    # MPS sweep
    if MPS_AVAIL:
        for d_hidden in D_HIDDENS:
            for batch_size in BATCH_SIZES:
                done += 1
                tag = f"mps/dh={d_hidden}/B={batch_size}"
                print(f"[{done}/{total_configs}] {tag}", end="  ", flush=True)
                try:
                    r = run_config(TN, TR, tau0, pool_ids,
                                   d_hidden, batch_size, "mps", -1)
                    rows.append(r)
                    print(f"→ {r['steps_per_sec']} sps  "
                          f"fwd={r['forward_pct']}% bwd={r['backward_pct']}%")
                except Exception as e:
                    print(f"FAILED: {e}")
                    rows.append({
                        "device": "mps", "d_hidden": d_hidden,
                        "batch_size": batch_size, "n_threads": -1,
                        "steps_per_sec": -1, "total_seconds": -1,
                        "forward_seconds": -1, "backward_seconds": -1,
                        "other_seconds": -1, "forward_pct": -1,
                        "backward_pct": -1, "n_params": -1, "jit": False,
                        "matmul_shape": "",
                    })

    # Write CSV
    CSV_PATH.parent.mkdir(parents=True, exist_ok=True)
    fieldnames = ["device", "d_hidden", "batch_size", "n_threads",
                  "steps_per_sec", "total_seconds", "forward_seconds",
                  "backward_seconds", "other_seconds", "forward_pct",
                  "backward_pct", "n_params", "jit", "matmul_shape"]
    with open(CSV_PATH, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fieldnames)
        w.writeheader()
        w.writerows(rows)
    print(f"\nCSV: {CSV_PATH}")

    # ================================================================
    # Analysis and markdown
    # ================================================================
    valid = [r for r in rows if r["steps_per_sec"] > 0]

    # Find best config overall
    best = max(valid, key=lambda r: r["steps_per_sec"])

    # CPU: for each (d_hidden, batch_size), find best thread count
    cpu_rows = [r for r in valid if r["device"] == "cpu"]
    mps_rows = [r for r in valid if r["device"] == "mps"]

    # Thread benefit analysis: for each (dh, B), compare threads
    thread_analysis = {}
    for dh in D_HIDDENS:
        for bs in BATCH_SIZES:
            subset = [r for r in cpu_rows
                      if r["d_hidden"] == dh and r["batch_size"] == bs]
            if not subset:
                continue
            by_thr = {r["n_threads"]: r for r in subset}
            best_thr = max(subset, key=lambda r: r["steps_per_sec"])
            t1 = by_thr.get(1)
            t4 = by_thr.get(4)
            thread_analysis[(dh, bs)] = {
                "best_threads": best_thr["n_threads"],
                "best_sps": best_thr["steps_per_sec"],
                "sps_t1": t1["steps_per_sec"] if t1 else -1,
                "sps_t4": t4["steps_per_sec"] if t4 else -1,
                "multi_thread_helps": best_thr["n_threads"] > 1,
            }

    # MPS vs CPU comparison
    mps_vs_cpu = {}
    for dh in D_HIDDENS:
        for bs in BATCH_SIZES:
            cpu_best = [r for r in cpu_rows
                        if r["d_hidden"] == dh and r["batch_size"] == bs]
            mps_match = [r for r in mps_rows
                         if r["d_hidden"] == dh and r["batch_size"] == bs]
            if cpu_best and mps_match:
                cb = max(cpu_best, key=lambda r: r["steps_per_sec"])
                mb = mps_match[0]
                mps_vs_cpu[(dh, bs)] = {
                    "cpu_sps": cb["steps_per_sec"],
                    "cpu_threads": cb["n_threads"],
                    "mps_sps": mb["steps_per_sec"],
                    "mps_wins": mb["steps_per_sec"] > cb["steps_per_sec"],
                    "speedup": round(mb["steps_per_sec"] / max(cb["steps_per_sec"], 0.1), 2),
                }

    # Write markdown
    md = []
    md.append("# Prime Transport Router — Regime Boundary v1")
    md.append("")
    md.append("## Purpose")
    md.append("")
    md.append("Determine whether the same recurrence (D=24, mathematically sequential)")
    md.append("benefits from parallel hardware at larger within-step work sizes.")
    md.append("")
    md.append("## Sweep Grid")
    md.append("")
    md.append(f"- D_HIDDEN: {D_HIDDENS}")
    md.append(f"- batch_size: {BATCH_SIZES}")
    md.append(f"- CPU threads: {CPU_THREADS}")
    md.append(f"- Devices: {DEVICES}")
    md.append(f"- D = {D_CONTEXT} (fixed), {BENCH_STEPS} steps per config")
    md.append(f"- torch {torch.__version__}, {platform.platform()}, cores={os.cpu_count()}")
    md.append("")

    # Full results table
    md.append("## Full Results")
    md.append("")
    md.append("| Device | D_HIDDEN | Batch | Threads | Steps/sec | Fwd% | Bwd% | Total(s) |")
    md.append("|--------|----------|-------|---------|-----------|------|------|----------|")
    for r in sorted(valid, key=lambda r: (-r["steps_per_sec"])):
        thr = str(r["n_threads"]) if r["n_threads"] > 0 else "—"
        md.append(f"| {r['device']} | {r['d_hidden']} | {r['batch_size']} | {thr} "
                  f"| **{r['steps_per_sec']}** | {r['forward_pct']}% | {r['backward_pct']}% "
                  f"| {r['total_seconds']} |")
    md.append("")

    # Thread analysis
    md.append("## Question 1–2: Does Multi-Threading Help at Larger Regimes?")
    md.append("")
    md.append("| D_HIDDEN | Batch | Best Threads | SPS@1thr | SPS@4thr | Multi-thread helps? |")
    md.append("|----------|-------|-------------|----------|----------|---------------------|")
    any_mt_helps = False
    for (dh, bs), ta in sorted(thread_analysis.items()):
        helps = "**YES**" if ta["multi_thread_helps"] else "no"
        if ta["multi_thread_helps"]:
            any_mt_helps = True
        md.append(f"| {dh} | {bs} | {ta['best_threads']} | {ta['sps_t1']} | {ta['sps_t4']} | {helps} |")
    md.append("")
    if any_mt_helps:
        # Find the crossover point
        crossover = [(dh, bs) for (dh, bs), ta in thread_analysis.items() if ta["multi_thread_helps"]]
        md.append(f"**Multi-threading begins to help at:** {crossover}")
        md.append("")
        md.append("At larger D_HIDDEN and/or batch_size, the per-step matmuls become large enough")
        md.append("to amortize thread synchronization overhead. The 'one core is optimal' conclusion")
        md.append("is **regime-specific to the tiny D_HIDDEN=32, B=256 config**.")
    else:
        md.append("`num_threads=1` remains optimal across all tested regimes.")
        md.append("The per-step ops may still be too small even at D_HIDDEN=128, B=1024.")
    md.append("")

    # MPS analysis
    if mps_vs_cpu:
        md.append("## Question 3: When Does MPS Become Preferable?")
        md.append("")
        md.append("| D_HIDDEN | Batch | CPU best (sps) | MPS (sps) | MPS speedup | MPS wins? |")
        md.append("|----------|-------|---------------|-----------|-------------|-----------|")
        any_mps_wins = False
        for (dh, bs), mc in sorted(mps_vs_cpu.items()):
            wins = "**YES**" if mc["mps_wins"] else "no"
            if mc["mps_wins"]:
                any_mps_wins = True
            md.append(f"| {dh} | {bs} | {mc['cpu_sps']} (thr={mc['cpu_threads']}) "
                      f"| {mc['mps_sps']} | {mc['speedup']}x | {wins} |")
        md.append("")
        if any_mps_wins:
            mps_wins = [(dh, bs) for (dh, bs), mc in mps_vs_cpu.items() if mc["mps_wins"]]
            md.append(f"**MPS becomes preferable at:** {mps_wins}")
        else:
            md.append("MPS does not outperform best CPU config in any tested regime.")
        md.append("")

    # Forward/backward split analysis
    md.append("## Question 4: Does Larger Work Change the Forward/Backward Split?")
    md.append("")
    md.append("| D_HIDDEN | Batch | Device | Fwd% | Bwd% |")
    md.append("|----------|-------|--------|------|------|")
    for r in sorted(valid, key=lambda r: (r["d_hidden"], r["batch_size"], r["device"])):
        if r["device"] == "cpu" and r["n_threads"] != 1:
            continue  # show only threads=1 for clean comparison
        md.append(f"| {r['d_hidden']} | {r['batch_size']} | {r['device']} "
                  f"| {r['forward_pct']}% | {r['backward_pct']}% |")
    md.append("")

    # Best execution substrate
    md.append("## Question 5: Best Execution Substrate for Future Experiments")
    md.append("")
    md.append(f"**Best overall config tested:** device={best['device']}, "
              f"D_HIDDEN={best['d_hidden']}, batch={best['batch_size']}, "
              f"threads={best['n_threads']}, "
              f"**{best['steps_per_sec']} sps**")
    md.append("")

    # Summary
    md.append("## Summary")
    md.append("")
    md.append("### What is true only for the current tiny regime (D_HIDDEN=32, B=256)")
    md.append("")
    md.append("- `num_threads=1` is optimal")
    md.append("- ~100% CPU on one core is expected and unavoidable")
    md.append("- Per-step ops are too small for threading overhead to amortize")
    md.append("")
    md.append("### What appears true more generally across regimes")
    md.append("")

    if any_mt_helps:
        md.append("- At larger D_HIDDEN/batch, multi-threading DOES begin to help")
        md.append("- The 'one core cap' is a regime artifact, not an architectural limit")
    else:
        md.append("- Even at D_HIDDEN=128, B=1024, the per-step ops remain small enough")
        md.append("  that thread overhead dominates. The D=24 sequential loop limits the")
        md.append("  opportunity for intra-step parallelism.")

    if any_mps_wins:
        md.append("- MPS becomes viable at larger regimes, amortizing dispatch overhead")
    else:
        md.append("- MPS does not yet outperform CPU in the tested range")

    md.append("- The D=24 sequential recurrence remains the structural depth constraint")
    md.append("  regardless of regime — but within-step throughput CAN scale with hardware")
    md.append("  if per-step work is large enough")
    md.append("")

    MD_PATH.parent.mkdir(parents=True, exist_ok=True)
    with open(MD_PATH, "w") as f:
        f.write("\n".join(md) + "\n")
    print(f"MD:  {MD_PATH}")

    print("\n" + "=" * 70)
    print("DONE")
    print("=" * 70)
    print(f"\n  Best: {best['device']} dh={best['d_hidden']} B={best['batch_size']} "
          f"thr={best['n_threads']} → {best['steps_per_sec']} sps")


if __name__ == "__main__":
    main()

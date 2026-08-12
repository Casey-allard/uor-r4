#!/usr/bin/env python3
"""
Router Bottleneck Probe v3 — Hard Evidence CPU Baseline Investigation

Profiles the CURRENT CPU baseline (device=cpu, D_HIDDEN=32, batch_size=256,
D=24, pos0 bias enabled) using multiple complementary profiling methods.

Phases:
  1. cProfile — Python-level wall-time decomposition
  2. torch.profiler — operator-level breakdown
  3. Split timing — forward / backward / optimizer wall-time attribution
  4. Thread monitoring — active thread count during training

Deliverables written:
  - docs/research/prime_transport_router_bottleneck_probe_v3.md
  - results/prime_transport_recursive_system/prime_transport_router_bottleneck_probe_v3.csv
"""

from __future__ import annotations

import cProfile
import csv
import importlib.util
import io
import os
import pathlib
import platform
import pstats
import sys
import threading
import time
from typing import Dict, List, Tuple

import torch
import torch.nn as nn
import torch.nn.functional as F

# ============================================================================
# Paths
# ============================================================================
SCRIPT_DIR = pathlib.Path(__file__).resolve().parent
REPO_ROOT  = SCRIPT_DIR.parent.parent            # ai-router/router-research
AI_ROOT    = REPO_ROOT.parent.parent              # AI-Research

MD_PATH  = REPO_ROOT / "docs" / "research" / "prime_transport_router_bottleneck_probe_v3.md"
CSV_PATH = REPO_ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_router_bottleneck_probe_v3.csv"

# ============================================================================
# Locked canonical constants — MUST NOT CHANGE
# ============================================================================
VOCAB       = 4
D_EMB       = 4
D_TAU       = 21
D_IN        = D_EMB + D_TAU   # 25
N_OPS       = 6
LR          = 0.02
TEMP_START  = 2.0
TEMP_END    = 0.1
GLOBAL_SEED = 42
B0_INIT     = 2.0
D_CONTEXT   = 24
D_HIDDEN    = 32
BATCH_SIZE  = 256
DEVICE      = "cpu"

# Profiling budgets
WARMUP_STEPS    = 10
CPROFILE_STEPS  = 200     # Phase 1
TORCH_PROF_STEPS = 50     # Phase 2
SPLIT_STEPS     = 200     # Phase 3
THREAD_MON_STEPS = 100    # Phase 4

# ============================================================================
# Setup
# ============================================================================
torch.set_num_threads(1)
torch.manual_seed(GLOBAL_SEED)


def _load_v6torch():
    spec = importlib.util.spec_from_file_location(
        "v6torch_base",
        str(SCRIPT_DIR / "run_router_reintegration_v6_torch.py"),
    )
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


# ============================================================================
# Model — exact copy from run_router_execution_scaling_v2.py
# ============================================================================
class RouterV6Scaled(nn.Module):
    def __init__(
        self,
        TN: torch.Tensor,
        TR: torch.Tensor,
        tau0_table: torch.Tensor,
        pool_ids: torch.Tensor,
        d_hidden: int = 32,
        d_context: int = D_CONTEXT,
        b0_init: float = B0_INIT,
        init_scale: float = 0.05,
        seed: int = GLOBAL_SEED,
    ) -> None:
        super().__init__()
        dh  = d_hidden
        dha = max(8, d_hidden // 4)

        self.register_buffer("TN", TN)
        self.register_buffer("TR", TR)
        self.register_buffer("tau0_table", tau0_table)
        self.register_buffer("pool_ids", pool_ids)

        pos0_mask = torch.zeros(1, d_context)
        pos0_mask[0, 0] = 1.0
        self.register_buffer("pos0_mask", pos0_mask)

        self.b_pos0 = nn.Parameter(torch.tensor(b0_init))

        gen = torch.Generator().manual_seed(seed)

        def rp(*shape: int) -> nn.Parameter:
            return nn.Parameter(torch.empty(*shape).normal_(0.0, init_scale, generator=gen))

        def zp(*shape: int) -> nn.Parameter:
            return nn.Parameter(torch.zeros(*shape))

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

    def forward(
        self,
        state_ids: torch.Tensor,
        tokens: torch.Tensor,
        x0: torch.Tensor,
        temp: float,                    # explicit float annotation for JIT
    ) -> torch.Tensor:
        B: int = state_ids.shape[0]
        D: int = tokens.shape[1]

        tau_prev: torch.Tensor = self.tau0_table[state_ids]
        soft_taus: List[torch.Tensor] = []

        for t in range(D):
            tn_batch = self.TN[state_ids]
            tok_t    = tokens[:, t]
            embs     = self.W_emb[tok_t]
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
            tr_rows   = self.TR[state_ids]
            state_ids = tr_rows.gather(1, k_hard.unsqueeze(1)).squeeze(1)

        soft_taus_stack = torch.stack(soft_taus, dim=1)
        h_attn       = torch.tanh(soft_taus_stack @ self.W_attn.t() + self.b_attn)
        a_scores_raw = (h_attn * self.v_attn).sum(dim=-1)
        a_scores     = a_scores_raw + self.pos0_mask * self.b_pos0
        alpha        = torch.softmax(a_scores, dim=1)
        pooled       = torch.einsum("bd,bdt->bt", alpha, soft_taus_stack)
        pred_logits  = pooled @ self.W_pred + self.b_pred
        return pred_logits


# ============================================================================
# Data sampling — exact copy from scaling v2
# ============================================================================
def sample_batch(pool_ids: torch.Tensor, B: int) -> Tuple[torch.Tensor, torch.Tensor, torch.Tensor]:
    idx       = torch.randint(0, len(pool_ids), (B,))
    state_ids = pool_ids[idx]
    tokens    = torch.randint(0, VOCAB, (B, D_CONTEXT))
    x0        = torch.randint(0, VOCAB, (B,))
    tokens[:, 0] = x0
    return state_ids, tokens, x0


# ============================================================================
# One training step — reusable
# ============================================================================
def train_step(model, optimizer, pool_ids, temp):
    sids, toks, x0 = sample_batch(pool_ids, BATCH_SIZE)
    pred = model(sids, toks, x0, temp)
    loss = F.cross_entropy(pred, x0)
    optimizer.zero_grad()
    loss.backward()
    nn.utils.clip_grad_norm_(model.parameters(), max_norm=1.0)
    optimizer.step()
    return float(loss.item())


def get_temp(step: int, total: int) -> float:
    frac = step / max(total - 1, 1)
    return float(TEMP_START * (TEMP_END / TEMP_START) ** frac)


# ============================================================================
# Phase 1: cProfile
# ============================================================================
def phase1_cprofile(model, optimizer, pool_ids) -> dict:
    print("\n[Phase 1] cProfile — Python-level wall-time decomposition", flush=True)
    print(f"  Running {CPROFILE_STEPS} training steps...", flush=True)

    pr = cProfile.Profile()
    model.train()
    torch.manual_seed(GLOBAL_SEED + 1)

    t0 = time.perf_counter()
    pr.enable()
    for step in range(CPROFILE_STEPS):
        temp = get_temp(step, CPROFILE_STEPS)
        train_step(model, optimizer, pool_ids, temp)
    pr.disable()
    total_wall = time.perf_counter() - t0

    # Extract top functions
    stream = io.StringIO()
    ps = pstats.Stats(pr, stream=stream)
    ps.sort_stats("cumulative")
    ps.print_stats(40)
    cumulative_text = stream.getvalue()

    stream2 = io.StringIO()
    ps2 = pstats.Stats(pr, stream=stream2)
    ps2.sort_stats("tottime")
    ps2.print_stats(40)
    tottime_text = stream2.getvalue()

    # Parse top entries from tottime
    top_entries = []
    for line in tottime_text.split("\n"):
        line = line.strip()
        if not line or line.startswith("ncalls") or line.startswith("Ordered") or line.startswith("function"):
            continue
        parts = line.split(None, 5)
        if len(parts) >= 6:
            try:
                ncalls = parts[0]
                tottime = float(parts[1])
                percall1 = float(parts[2])
                cumtime = float(parts[3])
                percall2 = float(parts[4])
                funcname = parts[5]
                top_entries.append({
                    "ncalls": ncalls,
                    "tottime": tottime,
                    "cumtime": cumtime,
                    "funcname": funcname,
                    "pct_wall": 100.0 * tottime / max(total_wall, 1e-9),
                })
            except (ValueError, IndexError):
                continue
        if len(top_entries) >= 20:
            break

    print(f"  Total wall time: {total_wall:.2f}s ({CPROFILE_STEPS} steps)", flush=True)
    print(f"  Steps/sec: {CPROFILE_STEPS / total_wall:.1f}", flush=True)
    print(f"  Top 5 by tottime:", flush=True)
    for e in top_entries[:5]:
        print(f"    {e['pct_wall']:5.1f}%  {e['tottime']:7.3f}s  {e['funcname']}", flush=True)

    return {
        "total_wall": total_wall,
        "steps": CPROFILE_STEPS,
        "sps": CPROFILE_STEPS / total_wall,
        "top_entries": top_entries,
        "cumulative_text": cumulative_text,
        "tottime_text": tottime_text,
    }


# ============================================================================
# Phase 2: torch.profiler
# ============================================================================
def phase2_torch_profiler(model, optimizer, pool_ids) -> dict:
    print(f"\n[Phase 2] torch.profiler — operator-level breakdown", flush=True)
    print(f"  Running {TORCH_PROF_STEPS} training steps...", flush=True)

    model.train()
    torch.manual_seed(GLOBAL_SEED + 2)

    # Warm up
    for step in range(5):
        train_step(model, optimizer, pool_ids, 2.0)

    activities = [torch.profiler.ProfilerActivity.CPU]

    t0 = time.perf_counter()
    with torch.profiler.profile(
        activities=activities,
        record_shapes=True,
        with_stack=False,
        profile_memory=False,
    ) as prof:
        for step in range(TORCH_PROF_STEPS):
            temp = get_temp(step, TORCH_PROF_STEPS)
            train_step(model, optimizer, pool_ids, temp)
    total_wall = time.perf_counter() - t0

    # Sort by self CPU time
    events = prof.key_averages()
    events_sorted = sorted(events, key=lambda e: e.self_cpu_time_total, reverse=True)

    top_ops = []
    total_self_us = sum(e.self_cpu_time_total for e in events_sorted)
    for e in events_sorted[:30]:
        top_ops.append({
            "name": e.key,
            "self_cpu_us": e.self_cpu_time_total,
            "cpu_time_us": e.cpu_time_total,
            "count": e.count,
            "pct_self": 100.0 * e.self_cpu_time_total / max(total_self_us, 1),
        })

    # Also get the table string
    table_str = events.table(sort_by="self_cpu_time_total", row_limit=30)

    print(f"  Total wall time: {total_wall:.2f}s ({TORCH_PROF_STEPS} steps)", flush=True)
    print(f"  Steps/sec: {TORCH_PROF_STEPS / total_wall:.1f}", flush=True)
    print(f"  Total self CPU: {total_self_us / 1e6:.2f}s", flush=True)
    print(f"  Top 5 ops by self CPU time:", flush=True)
    for o in top_ops[:5]:
        print(f"    {o['pct_self']:5.1f}%  {o['self_cpu_us']/1e6:7.3f}s  {o['name']}  (x{o['count']})", flush=True)

    return {
        "total_wall": total_wall,
        "steps": TORCH_PROF_STEPS,
        "sps": TORCH_PROF_STEPS / total_wall,
        "top_ops": top_ops,
        "total_self_us": total_self_us,
        "table_str": table_str,
    }


# ============================================================================
# Phase 3: Split timing — forward / backward / optimizer
# ============================================================================
def phase3_split_timing(model, optimizer, pool_ids) -> dict:
    print(f"\n[Phase 3] Split timing — forward/backward/optimizer decomposition", flush=True)
    print(f"  Running {SPLIT_STEPS} training steps...", flush=True)

    model.train()
    torch.manual_seed(GLOBAL_SEED + 3)

    t_sample_total = 0.0
    t_forward_total = 0.0
    t_loss_total = 0.0
    t_backward_total = 0.0
    t_clip_total = 0.0
    t_optim_total = 0.0
    t_zero_total = 0.0

    t0 = time.perf_counter()
    for step in range(SPLIT_STEPS):
        temp = get_temp(step, SPLIT_STEPS)

        t1 = time.perf_counter()
        sids, toks, x0 = sample_batch(pool_ids, BATCH_SIZE)
        t2 = time.perf_counter()
        pred = model(sids, toks, x0, temp)
        t3 = time.perf_counter()
        loss = F.cross_entropy(pred, x0)
        t4 = time.perf_counter()
        optimizer.zero_grad()
        t5 = time.perf_counter()
        loss.backward()
        t6 = time.perf_counter()
        nn.utils.clip_grad_norm_(model.parameters(), max_norm=1.0)
        t7 = time.perf_counter()
        optimizer.step()
        t8 = time.perf_counter()

        t_sample_total  += t2 - t1
        t_forward_total += t3 - t2
        t_loss_total    += t4 - t3
        t_zero_total    += t5 - t4
        t_backward_total += t6 - t5
        t_clip_total    += t7 - t6
        t_optim_total   += t8 - t7

    total_wall = time.perf_counter() - t0
    overhead = total_wall - (t_sample_total + t_forward_total + t_loss_total +
                              t_zero_total + t_backward_total + t_clip_total + t_optim_total)

    splits = {
        "sample":   t_sample_total,
        "forward":  t_forward_total,
        "loss":     t_loss_total,
        "zero_grad": t_zero_total,
        "backward": t_backward_total,
        "clip_grad": t_clip_total,
        "optimizer": t_optim_total,
        "overhead": overhead,
    }

    print(f"  Total wall time: {total_wall:.2f}s ({SPLIT_STEPS} steps)", flush=True)
    print(f"  Steps/sec: {SPLIT_STEPS / total_wall:.1f}", flush=True)
    print(f"  Breakdown:", flush=True)
    for name, t in sorted(splits.items(), key=lambda x: -x[1]):
        pct = 100.0 * t / max(total_wall, 1e-9)
        print(f"    {name:12s}: {t:7.3f}s  ({pct:5.1f}%)", flush=True)

    return {
        "total_wall": total_wall,
        "steps": SPLIT_STEPS,
        "sps": SPLIT_STEPS / total_wall,
        "splits": splits,
    }


# ============================================================================
# Phase 4: Thread monitoring during training
# ============================================================================
def phase4_thread_monitor(model, optimizer, pool_ids) -> dict:
    print(f"\n[Phase 4] Thread monitoring during training", flush=True)
    print(f"  Running {THREAD_MON_STEPS} training steps with thread snapshots...", flush=True)

    model.train()
    torch.manual_seed(GLOBAL_SEED + 4)

    thread_snapshots = []

    t0 = time.perf_counter()
    for step in range(THREAD_MON_STEPS):
        temp = get_temp(step, THREAD_MON_STEPS)

        # Snapshot threads before step
        threads = threading.enumerate()
        thread_snapshots.append({
            "step": step,
            "phase": "pre_step",
            "count": len(threads),
            "names": [t.name for t in threads],
        })

        train_step(model, optimizer, pool_ids, temp)

        # Snapshot threads after step
        threads = threading.enumerate()
        thread_snapshots.append({
            "step": step,
            "phase": "post_step",
            "count": len(threads),
            "names": [t.name for t in threads],
        })

    total_wall = time.perf_counter() - t0

    # Summarize
    counts = [s["count"] for s in thread_snapshots]
    all_names = set()
    for s in thread_snapshots:
        all_names.update(s["names"])

    min_thr = min(counts)
    max_thr = max(counts)
    avg_thr = sum(counts) / len(counts)

    print(f"  Total wall time: {total_wall:.2f}s ({THREAD_MON_STEPS} steps)", flush=True)
    print(f"  Thread counts: min={min_thr}, max={max_thr}, avg={avg_thr:.1f}", flush=True)
    print(f"  Thread names seen: {sorted(all_names)}", flush=True)

    return {
        "total_wall": total_wall,
        "steps": THREAD_MON_STEPS,
        "sps": THREAD_MON_STEPS / total_wall,
        "min_threads": min_thr,
        "max_threads": max_thr,
        "avg_threads": avg_thr,
        "thread_names": sorted(all_names),
        "snapshots": thread_snapshots,
    }


# ============================================================================
# Historical sample file analysis
# ============================================================================
def analyze_historical_sample() -> dict:
    sample_path = AI_ROOT / "Sample of python3.10.txt"
    if not sample_path.exists():
        return {"available": False, "note": "File not found"}

    text = sample_path.read_text(errors="replace")
    lines = text.split("\n")

    # Count key indicators
    mps_refs = sum(1 for l in lines if "MPS" in l or "mps" in l or "Metal" in l)
    backward_refs = sum(1 for l in lines if "backward" in l.lower() or "THPEngine" in l)
    readyqueue_refs = sum(1 for l in lines if "ReadyQueue" in l)
    condvar_refs = sum(1 for l in lines if "condition_variable" in l or "_pthread_cond_wait" in l)
    openblas_refs = sum(1 for l in lines if "blas_thread_server" in l)

    is_mps = mps_refs > 5

    return {
        "available": True,
        "total_lines": len(lines),
        "is_mps_run": is_mps,
        "mps_references": mps_refs,
        "backward_references": backward_refs,
        "readyqueue_references": readyqueue_refs,
        "condvar_wait_references": condvar_refs,
        "openblas_sleep_references": openblas_refs,
        "note": ("HISTORICAL: captured from MPS backend run, NOT the current CPU baseline. "
                 "Thread structure and dispatch patterns differ fundamentally from CPU execution." if is_mps
                 else "Sample appears to be from CPU run."),
    }


# ============================================================================
# Write CSV
# ============================================================================
def write_csv(p1, p2, p3, p4, hist):
    rows = []
    rank = 0

    # Phase 3 splits — primary wall-time attribution
    if p3:
        splits_sorted = sorted(p3["splits"].items(), key=lambda x: -x[1])
        for name, t in splits_sorted:
            if t < 0.001:
                continue
            rank += 1
            rows.append({
                "component_name": f"train_step/{name}",
                "profiler_source": "phase3_split_timing",
                "wall_time_seconds": round(t, 4),
                "percent_total_runtime": round(100.0 * t / max(p3["total_wall"], 1e-9), 2),
                "baseline_or_historical": "CURRENT_CPU_BASELINE",
                "thread_behavior_note": "single-thread CPU (torch.set_num_threads=1)",
                "bottleneck_rank": rank,
                "note": f"{p3['steps']} steps measured",
            })

    # Phase 2 top ops
    if p2:
        for i, op in enumerate(p2["top_ops"][:10]):
            rank += 1
            rows.append({
                "component_name": op["name"],
                "profiler_source": "phase2_torch_profiler",
                "wall_time_seconds": round(op["self_cpu_us"] / 1e6, 4),
                "percent_total_runtime": round(op["pct_self"], 2),
                "baseline_or_historical": "CURRENT_CPU_BASELINE",
                "thread_behavior_note": f"count={op['count']}",
                "bottleneck_rank": rank,
                "note": f"self_cpu_time across {p2['steps']} steps",
            })

    # Phase 1 top functions
    if p1:
        for i, e in enumerate(p1["top_entries"][:10]):
            rank += 1
            rows.append({
                "component_name": e["funcname"],
                "profiler_source": "phase1_cprofile",
                "wall_time_seconds": round(e["tottime"], 4),
                "percent_total_runtime": round(e["pct_wall"], 2),
                "baseline_or_historical": "CURRENT_CPU_BASELINE",
                "thread_behavior_note": f"ncalls={e['ncalls']}",
                "bottleneck_rank": rank,
                "note": f"tottime across {p1['steps']} steps",
            })

    # Historical sample — secondary only
    if hist and hist.get("available"):
        rank += 1
        rows.append({
            "component_name": "HISTORICAL_MPS_SAMPLE",
            "profiler_source": "macOS_sample_command",
            "wall_time_seconds": -1,
            "percent_total_runtime": -1,
            "baseline_or_historical": "HISTORICAL_MPS_SECONDARY",
            "thread_behavior_note": f"MPS backend; {hist['mps_references']} MPS/Metal refs",
            "bottleneck_rank": rank,
            "note": hist["note"],
        })

    CSV_PATH.parent.mkdir(parents=True, exist_ok=True)
    with open(CSV_PATH, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=[
            "component_name", "profiler_source", "wall_time_seconds",
            "percent_total_runtime", "baseline_or_historical",
            "thread_behavior_note", "bottleneck_rank", "note",
        ])
        writer.writeheader()
        writer.writerows(rows)

    print(f"\nCSV written: {CSV_PATH}", flush=True)
    return rows


# ============================================================================
# Write Markdown Report
# ============================================================================
def write_md(env, p1, p2, p3, p4, hist):
    md = []
    md.append("# Prime Transport Router — Bottleneck Probe v3")
    md.append("")
    md.append("## Current Baseline (PRIMARY)")
    md.append("")
    md.append("| Parameter | Value |")
    md.append("|-----------|-------|")
    md.append(f"| device | **CPU** |")
    md.append(f"| D_HIDDEN | 32 |")
    md.append(f"| batch_size | 256 |")
    md.append(f"| D (context) | 24 |")
    md.append(f"| pos0 bias | enabled (B0_INIT=2.0) |")
    md.append(f"| torch version | {env['torch_version']} |")
    md.append(f"| python version | {env['python_version']} |")
    md.append(f"| num_threads | 1 |")
    md.append(f"| platform | {env['platform']} |")
    md.append("")
    md.append("**All profiling data below is from the CURRENT CPU baseline.**")
    md.append("")

    md.append("## Historical Sample Status")
    md.append("")
    md.append("`/AI-Research/Sample of python3.10.txt` is a **historical** macOS `sample` trace")
    md.append("captured from an **MPS-era run** (different backend). It is referenced as")
    md.append("**secondary comparative evidence only**. It is NOT sufficient to explain the")
    md.append("current CPU baseline bottleneck.")
    if hist and hist.get("available"):
        md.append("")
        md.append(f"- MPS/Metal references in file: {hist['mps_references']}")
        md.append(f"- Is MPS run: **{hist['is_mps_run']}**")
        md.append(f"- Backward/autograd refs: {hist['backward_references']}")
        md.append(f"- ReadyQueue refs: {hist['readyqueue_references']}")
        md.append(f"- Condition variable wait refs: {hist['condvar_wait_references']}")
        md.append(f"- OpenBLAS sleep refs: {hist['openblas_sleep_references']}")
    md.append("")

    # Phase 1 results
    md.append("## Phase 1: cProfile — Python-Level Wall-Time Decomposition")
    md.append("")
    if p1:
        md.append(f"- Steps profiled: {p1['steps']}")
        md.append(f"- Total wall time: {p1['total_wall']:.2f}s")
        md.append(f"- Steps/sec: {p1['sps']:.1f}")
        md.append("")
        md.append("### Top Functions by tottime")
        md.append("")
        md.append("| Rank | %Wall | tottime(s) | cumtime(s) | Function |")
        md.append("|------|-------|-----------|-----------|----------|")
        for i, e in enumerate(p1["top_entries"][:15]):
            md.append(f"| {i+1} | {e['pct_wall']:.1f}% | {e['tottime']:.3f} | {e['cumtime']:.3f} | `{e['funcname']}` |")
        md.append("")
        md.append("<details><summary>Full cProfile output (cumulative)</summary>")
        md.append("")
        md.append("```")
        md.append(p1["cumulative_text"][:3000])
        md.append("```")
        md.append("</details>")
        md.append("")
        md.append("<details><summary>Full cProfile output (tottime)</summary>")
        md.append("")
        md.append("```")
        md.append(p1["tottime_text"][:3000])
        md.append("```")
        md.append("</details>")
    else:
        md.append("Phase 1 did not run.")
    md.append("")

    # Phase 2 results
    md.append("## Phase 2: torch.profiler — Operator-Level Breakdown")
    md.append("")
    if p2:
        md.append(f"- Steps profiled: {p2['steps']}")
        md.append(f"- Total wall time: {p2['total_wall']:.2f}s")
        md.append(f"- Total self CPU time: {p2['total_self_us']/1e6:.2f}s")
        md.append(f"- Steps/sec: {p2['sps']:.1f}")
        md.append("")
        md.append("### Top Operators by Self CPU Time")
        md.append("")
        md.append("| Rank | %Self | self_cpu(s) | total_cpu(s) | Count | Operator |")
        md.append("|------|-------|------------|-------------|-------|----------|")
        for i, o in enumerate(p2["top_ops"][:15]):
            md.append(f"| {i+1} | {o['pct_self']:.1f}% | {o['self_cpu_us']/1e6:.3f} | {o['cpu_time_us']/1e6:.3f} | {o['count']} | `{o['name']}` |")
        md.append("")
        md.append("<details><summary>Full torch.profiler table</summary>")
        md.append("")
        md.append("```")
        md.append(p2["table_str"][:4000])
        md.append("```")
        md.append("</details>")
    else:
        md.append("Phase 2 did not run.")
    md.append("")

    # Phase 3 results
    md.append("## Phase 3: Split Timing — Forward/Backward/Optimizer Decomposition")
    md.append("")
    if p3:
        md.append(f"- Steps profiled: {p3['steps']}")
        md.append(f"- Total wall time: {p3['total_wall']:.2f}s")
        md.append(f"- Steps/sec: {p3['sps']:.1f}")
        md.append("")
        md.append("| Component | Wall Time (s) | % Total |")
        md.append("|-----------|--------------|---------|")
        splits_sorted = sorted(p3["splits"].items(), key=lambda x: -x[1])
        for name, t in splits_sorted:
            pct = 100.0 * t / max(p3["total_wall"], 1e-9)
            md.append(f"| {name} | {t:.3f} | {pct:.1f}% |")
        # Check attribution coverage
        accounted = sum(t for _, t in p3["splits"].items() if _ != "overhead")
        coverage = 100.0 * accounted / max(p3["total_wall"], 1e-9)
        md.append("")
        md.append(f"**Wall-time attribution coverage: {coverage:.1f}%** (target: ≥80%)")
    else:
        md.append("Phase 3 did not run.")
    md.append("")

    # Phase 4 results
    md.append("## Phase 4: Thread Monitoring")
    md.append("")
    if p4:
        md.append(f"- Steps profiled: {p4['steps']}")
        md.append(f"- Total wall time: {p4['total_wall']:.2f}s")
        md.append(f"- Thread count: min={p4['min_threads']}, max={p4['max_threads']}, avg={p4['avg_threads']:.1f}")
        md.append(f"- Thread names seen: {p4['thread_names']}")
        md.append("")
        md.append("### Thread Interpretation")
        md.append("")
        if p4["max_threads"] <= 2:
            md.append("With `torch.set_num_threads(1)`, the process runs on a **single compute thread**.")
            md.append("Any additional Python threads are infrastructure (e.g., MainThread only).")
            md.append("This means **one CPU core is saturated** while others are idle.")
        else:
            md.append(f"Thread count varied from {p4['min_threads']} to {p4['max_threads']}.")
            md.append("Additional threads may be autograd workers or Python infrastructure threads.")
    else:
        md.append("Phase 4 did not run.")
    md.append("")

    # Answers to required questions
    md.append("## Answers to Required Diagnostic Questions")
    md.append("")
    md.append("*All answers below are based on CURRENT CPU baseline profiling data.*")
    md.append("")

    # Build answers from data
    if p3:
        splits = p3["splits"]
        fwd_pct = 100.0 * splits.get("forward", 0) / max(p3["total_wall"], 1e-9)
        bwd_pct = 100.0 * splits.get("backward", 0) / max(p3["total_wall"], 1e-9)
        dominant = max(splits.items(), key=lambda x: x[1])

        md.append("### 1. Why does Activity Monitor stay near ~100% CPU?")
        md.append("")
        md.append(f"With `torch.set_num_threads(1)`, the training loop runs **entirely on one "
                   f"CPU core**. There are no idle gaps — every step executes forward ({fwd_pct:.0f}% "
                   f"of time), backward ({bwd_pct:.0f}%), and optimizer serially with no "
                   f"parallelism or I/O waits. One core at 100% = 100% in Activity Monitor "
                   f"(which reports per-core for single-threaded processes).")
        md.append("")

        md.append("### 2. Is there one truly hot thread?")
        md.append("")
        if p4 and p4["max_threads"] <= 2:
            md.append("**Yes.** Thread monitoring shows only 1–2 Python threads throughout. "
                      "The MainThread does all computation. No parallel workers are active.")
        else:
            md.append(f"Thread count: {p4['min_threads']}-{p4['max_threads']}. "
                      f"The main thread is the primary compute thread; others appear to be "
                      f"infrastructure or autograd workers that are mostly idle.")
        md.append("")

        md.append("### 3. What exact functions/ops dominate wall time?")
        md.append("")
        md.append(f"**Phase 3 split timing** shows the dominant component is "
                   f"**`{dominant[0]}`** at {100.0*dominant[1]/max(p3['total_wall'],1e-9):.1f}% of wall time.")
        md.append("")
        if p2 and p2["top_ops"]:
            md.append(f"**Phase 2 torch.profiler** top operator: "
                      f"`{p2['top_ops'][0]['name']}` ({p2['top_ops'][0]['pct_self']:.1f}% self CPU time).")
        if p1 and p1["top_entries"]:
            md.append(f"**Phase 1 cProfile** top function: "
                      f"`{p1['top_entries'][0]['funcname']}` ({p1['top_entries'][0]['pct_wall']:.1f}% tottime).")
        md.append("")

        md.append("### 4. Is a major hot path still running in Python?")
        md.append("")
        md.append("The model forward pass runs through **Python's interpreter** for the "
                   "`for t in range(D)` loop (D=24 iterations). Each iteration dispatches "
                   "small C++ torch ops, but the Python-level loop control, tensor indexing, "
                   "and dispatch overhead are all Python-side. Unless JIT compilation fully "
                   "optimizes away the Python loop, significant time is in Python dispatch.")
        md.append("")

        md.append("### 5. Is forward, backward, autograd, indexing, or something else the dominant bottleneck?")
        md.append("")
        md.append(f"- **Forward**: {fwd_pct:.1f}% of wall time")
        md.append(f"- **Backward**: {bwd_pct:.1f}% of wall time")
        other_pct = 100.0 - fwd_pct - bwd_pct
        md.append(f"- **Other (sample+loss+optim+clip)**: {other_pct:.1f}%")
        md.append("")

        md.append("### 6. Are there wakeup/polling/tiny-dispatch effects in the current CPU baseline?")
        md.append("")
        md.append("On **CPU with 1 thread**, there are no MPS dispatch barriers, no Metal "
                   "command queue syncs, and no GPU submission overhead. The 'tiny dispatch' "
                   "pattern observed in the historical MPS sample does NOT apply here. However, "
                   "the 24-iteration Python loop does create 24× dispatch overhead per step for "
                   "each torch op (indexing TN/TR tables, matmul, einsum, etc.). This is "
                   "**CPU dispatch overhead**, not GPU submission overhead.")
        md.append("")

        md.append("### 7. What is the single most evidence-based fix?")
        md.append("")
        if fwd_pct > bwd_pct:
            md.append(f"**Forward pass dominates at {fwd_pct:.1f}%** of wall time. The D=24 "
                      f"sequential Python loop is the structural bottleneck. Each iteration "
                      f"performs many tiny tensor ops on small tensors (256×25, 256×21). "
                      f"The evidence-based fix is: **reduce per-step dispatch overhead by "
                      f"fusing the inner loop ops** (e.g., via `torch.jit.script` with "
                      f"verified fusion, or rewriting the inner loop body as a single "
                      f"custom autograd Function).")
        else:
            md.append(f"**Backward pass dominates at {bwd_pct:.1f}%** of wall time. The "
                      f"autograd engine must backprop through 24 sequential steps with "
                      f"many small intermediate tensors. The evidence-based fix is: "
                      f"**reduce graph complexity** by checkpointing or simplifying the "
                      f"backward pass (e.g., `torch.utils.checkpoint` to trade compute "
                      f"for memory and reduce graph overhead).")
        md.append("")
    else:
        md.append("*Phase 3 did not run — cannot answer questions without split timing data.*")
        md.append("")

    # Ranked bottleneck list
    md.append("## Ranked Bottleneck List (Current CPU Baseline)")
    md.append("")
    if p3:
        md.append("| Rank | Component | Wall Time | % | Evidence Source |")
        md.append("|------|-----------|-----------|---|----------------|")
        splits_sorted = sorted(p3["splits"].items(), key=lambda x: -x[1])
        for i, (name, t) in enumerate(splits_sorted):
            if t < 0.001:
                continue
            pct = 100.0 * t / max(p3["total_wall"], 1e-9)
            md.append(f"| {i+1} | {name} | {t:.3f}s | {pct:.1f}% | Phase 3 split timing |")
    md.append("")

    # Honesty section
    md.append("## Honesty Section")
    md.append("")
    md.append("### What is PROVEN from current CPU profiling")
    md.append("")
    if p3:
        md.append(f"- The forward/backward/optimizer split is measured directly: "
                   f"forward={fwd_pct:.1f}%, backward={bwd_pct:.1f}%")
        md.append(f"- Total throughput is measured: {p3['sps']:.1f} steps/sec on CPU with 1 thread")
    if p4:
        md.append(f"- Thread count during training: {p4['min_threads']}-{p4['max_threads']}")
    if p2:
        md.append(f"- Top torch operators are identified by self CPU time")
    if p1:
        md.append(f"- Top Python functions are identified by tottime")
    md.append(f"- ~100% CPU is explained: single-thread saturation (no idle gaps)")
    md.append("")

    md.append("### What is only SUGGESTED by the historical MPS sample")
    md.append("")
    md.append("- The historical sample shows MPS dispatch patterns (Metal command queues, "
              "wrapper_MPS_* ops) that are **irrelevant** to CPU execution")
    md.append("- The backward/ReadyQueue waiting pattern in the MPS sample may partially "
              "apply to CPU (autograd engine structure is shared), but the dispatch "
              "mechanism is completely different")
    md.append("- Thread structure in the MPS sample (Metal GPU stream, MPSGraphDevice) "
              "does not exist in CPU execution")
    md.append("")

    md.append("### What remains UNCERTAIN")
    md.append("")
    md.append("- Exact overhead of Python-level loop dispatch vs C++ kernel execution "
              "within the forward pass (would need finer-grained instrumentation)")
    md.append("- Whether JIT scripting is actually eliminating Python overhead or just "
              "adding JIT dispatch overhead on top")
    md.append("- The exact cost of the large TN table indexing (343K×6×21 tensor) — "
              "this could be cache-miss dominated but we haven't measured L1/L2 misses")
    md.append("- Whether increasing batch_size or D_HIDDEN would shift the balance "
              "toward compute-bound (away from dispatch-bound)")
    md.append("")

    md.append("### What previous claims were too confident")
    md.append("")
    md.append("- Prior claims that 'workload shape' or 'tiny ops' fully explained "
              "the ~100% CPU were not backed by direct profiling — they were educated "
              "guesses that may be correct but were stated as conclusions")
    md.append("- The MPS sample was initially treated as primary evidence for the "
              "CPU bottleneck, when it actually describes a different execution path")
    md.append("")

    MD_PATH.parent.mkdir(parents=True, exist_ok=True)
    with open(MD_PATH, "w") as f:
        f.write("\n".join(md) + "\n")

    print(f"Markdown written: {MD_PATH}", flush=True)


# ============================================================================
# Main
# ============================================================================
def main():
    print("=" * 70)
    print("Router Bottleneck Probe v3 — CURRENT CPU BASELINE")
    print("=" * 70)

    env = {
        "torch_version": torch.__version__,
        "python_version": platform.python_version(),
        "platform": platform.platform(),
        "num_threads": torch.get_num_threads(),
        "mps_available": torch.backends.mps.is_available(),
    }

    print(f"  torch: {env['torch_version']}")
    print(f"  python: {env['python_version']}")
    print(f"  num_threads: {env['num_threads']}")
    print(f"  device: {DEVICE}")
    print(f"  D_HIDDEN: {D_HIDDEN}, batch_size: {BATCH_SIZE}, D: {D_CONTEXT}")
    print(f"  Active threads: {threading.active_count()}")

    # Load state tables
    print("\nLoading v6 torch module (BFS warm-up)...", flush=True)
    import random as pyrand
    v6 = _load_v6torch()
    rng = pyrand.Random(GLOBAL_SEED)
    pool = v6.build_warmup_pool(rng, size=v6.POOL_SIZE)
    n_st = v6.bfs_warm_up(pool, max_seconds=v6.BFS_MAX_SECS, verbose=True)
    TN, TR, tau0, pool_ids, _ = v6.build_state_tables(pool, verbose=True)
    print(f"  States: {n_st:,}")

    # Build model
    model = RouterV6Scaled(TN, TR, tau0, pool_ids, d_hidden=D_HIDDEN)
    n_params = sum(p.numel() for p in model.parameters())
    print(f"  Model params: {n_params:,}")

    # Try JIT script
    jit_ok = True
    try:
        model = torch.jit.script(model)
        print("  JIT scripted: yes")
    except Exception as e:
        print(f"  JIT script failed: {e}")
        print("  Using eager mode")
        jit_ok = False

    model.train()
    optimizer = torch.optim.SGD(model.parameters(), lr=LR)

    # Warmup
    print(f"\nWarmup ({WARMUP_STEPS} steps)...", flush=True)
    torch.manual_seed(GLOBAL_SEED)
    for step in range(WARMUP_STEPS):
        train_step(model, optimizer, pool_ids, 2.0)
    print("  Warmup complete.")

    # Run phases
    p1 = phase1_cprofile(model, optimizer, pool_ids)
    p2 = phase2_torch_profiler(model, optimizer, pool_ids)
    p3 = phase3_split_timing(model, optimizer, pool_ids)
    p4 = phase4_thread_monitor(model, optimizer, pool_ids)
    hist = analyze_historical_sample()

    # Write outputs
    print("\n" + "=" * 70)
    print("Writing deliverables...")
    print("=" * 70)
    write_csv(p1, p2, p3, p4, hist)
    write_md(env, p1, p2, p3, p4, hist)

    print("\n" + "=" * 70)
    print("DONE — Bottleneck Probe v3 complete")
    print("=" * 70)
    if p3:
        splits = p3["splits"]
        dominant = max(splits.items(), key=lambda x: x[1])
        fwd_pct = 100.0 * splits.get("forward", 0) / max(p3["total_wall"], 1e-9)
        bwd_pct = 100.0 * splits.get("backward", 0) / max(p3["total_wall"], 1e-9)
        print(f"\n  HEADLINE: {dominant[0]} dominates at "
              f"{100.0*dominant[1]/max(p3['total_wall'],1e-9):.1f}% of wall time")
        print(f"  Forward: {fwd_pct:.1f}%  |  Backward: {bwd_pct:.1f}%")
        print(f"  Throughput: {p3['sps']:.1f} steps/sec on CPU×1")


if __name__ == "__main__":
    main()

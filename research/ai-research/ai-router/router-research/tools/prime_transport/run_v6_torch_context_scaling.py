#!/usr/bin/env python3
"""run_v6_torch_context_scaling.py

Context-length scaling study using the PyTorch v6 execution backend.

Reuses run_router_reintegration_v6_torch.py exactly as-is:
  - same RouterV6 architecture (D_HIDDEN=32, unchanged)
  - same v6 injection semantics (step-0 additive only)
  - same operator geometry (v20–v25)
  - same evaluation protocol

Only the delay set and budget change:
  DELAYS  = [16, 24, 32, 48]
  BUDGET  = 10000

No model changes. No semantic changes.
"""
from __future__ import annotations

import csv
import importlib.util
import random as pyrand
import sys
import time
from pathlib import Path
from typing import Dict, List, Tuple

import torch

# ---------------------------------------------------------------------------
# Import the torch backend module without executing its main()
# ---------------------------------------------------------------------------
_TORCH_SCRIPT = Path(__file__).parent / "run_router_reintegration_v6_torch.py"
_spec = importlib.util.spec_from_file_location("v6torch", _TORCH_SCRIPT)
_v6t  = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_v6t)  # type: ignore[union-attr]

# Pull everything we need into local scope
RouterV6          = _v6t.RouterV6
build_warmup_pool = _v6t.build_warmup_pool
bfs_warm_up       = _v6t.bfs_warm_up
build_state_tables= _v6t.build_state_tables
train_to_budgets  = _v6t.train_to_budgets
evaluate          = _v6t.evaluate

GLOBAL_SEED = _v6t.GLOBAL_SEED
TEMP_END    = _v6t.TEMP_END
POOL_SIZE   = _v6t.POOL_SIZE
BFS_MAX_SECS= _v6t.BFS_MAX_SECS

# ---------------------------------------------------------------------------
# Experiment config (the ONLY things that differ from the base script)
# ---------------------------------------------------------------------------
DELAYS  = [16, 24, 32, 48]
BUDGET  = 10000

RESULTS_DIR = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system"
)
DOCS_DIR = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/docs/research"
)
CSV_OUT = RESULTS_DIR / "prime_transport_router_reintegration_v6_torch_context_scaling.csv"
MD_OUT  = DOCS_DIR    / "prime_transport_router_reintegration_v6_torch_context_scaling.md"


# ---------------------------------------------------------------------------
# Verdict classification
# ---------------------------------------------------------------------------
def _verdict(accuracy: float, chance: float) -> str:
    if accuracy >= 0.95:
        return "solved"
    if accuracy >= chance * 1.5:
        return "learning_not_solved"
    return "near_chance_failed"


# ---------------------------------------------------------------------------
# Run the scaling experiment
# ---------------------------------------------------------------------------
def run_context_scaling(
    TN: torch.Tensor,
    TR: torch.Tensor,
    tau0_table: torch.Tensor,
    pool_ids: torch.Tensor,
) -> Dict[int, dict]:
    results: Dict[int, dict] = {}
    t0_all = time.perf_counter()

    for D in DELAYS:
        chance = 1.0 / D
        print(f"\n=== D={D} (chance={chance:.3f}) ===", flush=True)
        t_d = time.perf_counter()

        model    = RouterV6(TN, TR, tau0_table, pool_ids, seed=GLOBAL_SEED + D)
        scripted = torch.jit.script(model)

        snapshots = train_to_budgets(
            scripted, pool_ids, D,
            budgets=[BUDGET],
            seed=GLOBAL_SEED + D,
        )

        scripted.load_state_dict(snapshots[BUDGET])
        res = evaluate(scripted, pool_ids, D)

        verdict = _verdict(res["accuracy"], chance)

        results[D] = {**res, "verdict": verdict, "chance": chance}

        wall = time.perf_counter() - t_d
        print(
            f"  acc={res['accuracy']:.3f}  vs_chance={res['accuracy']/chance:.2f}x  "
            f"H={res['route_entropy']:.3f}  tr={res['transport_frac']:.3f}  "
            f"alpha0={res['alpha0']:.3f}  [{verdict}]  wall={wall:.1f}s",
            flush=True,
        )

    print(f"\nTotal experiment: {time.perf_counter()-t0_all:.1f}s")
    return results


# ---------------------------------------------------------------------------
# Write CSV
# ---------------------------------------------------------------------------
def write_csv(results: Dict[int, dict]) -> None:
    RESULTS_DIR.mkdir(parents=True, exist_ok=True)
    fieldnames = [
        "delay", "budget", "metric_name", "metric_value",
        "vs_chance", "route_entropy", "transport_usage_fraction",
        "attention_alpha0", "verdict", "note",
    ]
    rows = []
    for D, res in sorted(results.items()):
        chance = res["chance"]
        base = {
            "delay":                    D,
            "budget":                   BUDGET,
            "vs_chance":                round(res["accuracy"] / chance, 4),
            "route_entropy":            round(res["route_entropy"], 4),
            "transport_usage_fraction": round(res["transport_frac"], 4),
            "attention_alpha0":         round(res["alpha0"], 4),
            "verdict":                  res["verdict"],
            "note":                     f"B=256,D_H=32,torch.jit.script,v6_step0_inject",
        }
        rows.append({**base,
                     "metric_name":  "accuracy",
                     "metric_value": round(res["accuracy"], 4)})
        rows.append({**base,
                     "metric_name":  "route_entropy",
                     "metric_value": round(res["route_entropy"], 4)})
        rows.append({**base,
                     "metric_name":  "transport_fraction",
                     "metric_value": round(res["transport_frac"], 4)})
        rows.append({**base,
                     "metric_name":  "attention_alpha0",
                     "metric_value": round(res["alpha0"], 4)})

    with open(CSV_OUT, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fieldnames)
        w.writeheader()
        w.writerows(rows)
    print(f"CSV written: {CSV_OUT}")


# ---------------------------------------------------------------------------
# Write Markdown
# ---------------------------------------------------------------------------
def write_md(results: Dict[int, dict], total_s: float) -> None:
    DOCS_DIR.mkdir(parents=True, exist_ok=True)

    # Find first degradation point
    first_degraded = None
    first_failed   = None
    for D in DELAYS:
        v = results[D]["verdict"]
        if v != "solved" and first_degraded is None:
            first_degraded = D
        if v == "near_chance_failed" and first_failed is None:
            first_failed = D

    lines = [
        "# Prime Transport Router v6 Torch — Context-Length Scaling",
        "",
        "**Status:** Complete",
        "",
        "## Setup",
        "",
        f"- Script: `run_v6_torch_context_scaling.py` (wraps `run_router_reintegration_v6_torch.py`)",
        f"- Delays tested: {DELAYS}",
        f"- Budget: {BUDGET} batches each",
        f"- Model: D_HIDDEN=32, B=256, torch.jit.script backend — unchanged from v6_torch",
        f"- Injection: v6 step-0 additive only — unchanged",
        "",
        "## Results",
        "",
        "| D   | Accuracy | vs Chance | Route H | Transport | alpha0 | Verdict |",
        "|-----|----------|-----------|---------|-----------|--------|---------|",
    ]

    for D in DELAYS:
        res    = results[D]
        chance = res["chance"]
        lines.append(
            f"| {D:>3} "
            f"| {res['accuracy']:.3f}    "
            f"| {res['accuracy']/chance:.2f}×      "
            f"| {res['route_entropy']:.3f}   "
            f"| {res['transport_frac']:.3f}      "
            f"| {res['alpha0']:.3f}  "
            f"| {res['verdict']} |"
        )

    # First degradation
    deg_str = (f"D={first_degraded}" if first_degraded else
               "none — all delays solved")
    fail_str = (f"D={first_failed}" if first_failed else
                "none — no near-chance failure observed")

    lines += [
        "",
        "## Analysis",
        "",
        f"**First degradation:** {deg_str}",
        f"**First near-chance failure:** {fail_str}",
        "",
        "### Attention alpha0",
        "",
    ]

    # alpha0 commentary
    solved_alpha   = {D: results[D]["alpha0"] for D in DELAYS
                      if results[D]["verdict"] == "solved"}
    learning_alpha = {D: results[D]["alpha0"] for D in DELAYS
                      if results[D]["verdict"] == "learning_not_solved"}
    failed_alpha   = {D: results[D]["alpha0"] for D in DELAYS
                      if results[D]["verdict"] == "near_chance_failed"}

    if solved_alpha:
        pairs = ", ".join(f"D={D}: {v:.3f}" for D, v in solved_alpha.items())
        lines.append(f"- Solved delays show concentrated alpha0: {pairs}")
    if learning_alpha:
        pairs = ", ".join(f"D={D}: {v:.3f}" for D, v in learning_alpha.items())
        lines.append(f"- Learning (not solved) delays show partial alpha0 concentration: {pairs}")
    if failed_alpha:
        pairs = ", ".join(f"D={D}: {v:.3f}" for D, v in failed_alpha.items())
        lines.append(f"- Near-chance delays show alpha0 near uniform (1/D): {pairs}")

    lines += [
        "",
        "### Transport usage across context lengths",
        "",
    ]

    for D in DELAYS:
        tr = results[D]["transport_frac"]
        lines.append(f"- D={D}: transport_fraction={tr:.3f}")

    lines += [
        "",
        "### Failure mode diagnosis",
        "",
    ]

    # Diagnosis
    if first_failed is not None:
        res_fail = results[first_failed]
        if res_fail["route_entropy"] > 1.5:
            diag = ("Route entropy remains high at failure point, suggesting the router "
                    "has not collapsed to a structured routing policy. This is consistent "
                    "with a **training budget limit** rather than a representation collapse: "
                    "the model is still exploring rather than converging.")
        else:
            diag = ("Route entropy is low at failure point despite near-chance accuracy. "
                    "This suggests the model has converged to a routing policy that is "
                    "structured but incorrect — consistent with a **representation limit** "
                    "where the current architecture cannot reliably carry the token signal "
                    "across the longer context.")
        lines.append(diag)
    else:
        lines.append(
            "No near-chance failure observed in this delay range. "
            "The current interface remains above chance at all tested delays, "
            "though accuracy degrades with increasing D."
        )

    lines += [
        "",
        "## Honesty",
        "",
    ]

    worked   = [D for D in DELAYS if results[D]["verdict"] == "solved"]
    degraded = [D for D in DELAYS if results[D]["verdict"] == "learning_not_solved"]
    failed   = [D for D in DELAYS if results[D]["verdict"] == "near_chance_failed"]

    worked_s   = ", ".join(f"D={D}" for D in worked)   or "none"
    degraded_s = ", ".join(f"D={D}" for D in degraded) or "none"
    failed_s   = ", ".join(f"D={D}" for D in failed)   or "none"

    lines += [
        f"- **What worked:** {worked_s}",
        f"- **What degraded (learning, not solved):** {degraded_s}",
        f"- **What near-chance failed:** {failed_s}",
        "- **Uncertain:** Whether degraded delays would solve with more budget "
          "or hit a hard representation ceiling.",
        "- No files modified. No operators rebuilt. D_HIDDEN unchanged at 32.",
        "",
        f"Total wall time: {total_s:.1f}s",
        "",
        "## Next Step",
        "",
    ]

    # Recommend exactly one next step
    if first_failed is not None:
        res_fail = results[first_failed]
        if res_fail["route_entropy"] > 1.5:
            # High entropy at failure = budget limited
            rec = (
                f"**Extend training budget for D={first_failed}.** "
                f"Route entropy is still high ({res_fail['route_entropy']:.3f}) at the "
                f"first near-chance failure, indicating the model is still in an "
                f"exploration phase rather than hitting a hard representational wall. "
                f"Run D={first_failed} at 20,000–30,000 batches to determine "
                f"whether it eventually converges."
            )
        else:
            rec = (
                f"**Test compression / memory-layer integration at D={first_failed}.** "
                f"Route entropy is low at the first failure point "
                f"({res_fail['route_entropy']:.3f}), indicating the router has converged "
                f"but cannot carry the token signal. The current tau representation "
                f"(21-dim one-hot) lacks sufficient capacity to retain signal across "
                f"D={first_failed} steps. A memory layer or compressed state embedding "
                f"is the logical next intervention."
            )
    elif first_degraded is not None:
        res_deg = results[first_degraded]
        if res_deg["route_entropy"] > 1.5:
            rec = (
                f"**Extend training budget for D={first_degraded}.** "
                f"First degradation at D={first_degraded} (acc={res_deg['accuracy']:.3f}) "
                f"with high route entropy ({res_deg['route_entropy']:.3f}), "
                f"suggesting it is budget-limited. "
                f"Run at 20,000–30,000 batches before concluding it is a hard ceiling."
            )
        else:
            rec = (
                f"**Test compression / memory-layer integration at D={first_degraded}.** "
                f"First degradation at D={first_degraded} with low entropy, "
                f"indicating a representation limit rather than a training budget issue."
            )
    else:
        rec = (
            "**Extend context-length scaling beyond D=48.** "
            "All tested delays remain above chance. "
            "Test D ∈ {64, 96, 128} to find the actual degradation point."
        )

    lines.append(rec)

    with open(MD_OUT, "w") as f:
        f.write("\n".join(lines) + "\n")
    print(f"MD written: {MD_OUT}")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
def main() -> None:
    t0_all = time.perf_counter()

    print("=== v6 Torch Context-Length Scaling ===")
    print(f"Delays: {DELAYS},  Budget: {BUDGET} batches each")
    print(f"Model: D_HIDDEN={_v6t.D_HIDDEN}, B={_v6t.BATCH_SIZE} (unchanged)\n")

    # 1. Build pool
    py_rng = pyrand.Random(GLOBAL_SEED)
    print("Building warmup pool...", flush=True)
    pool = build_warmup_pool(py_rng, size=POOL_SIZE)

    # 2. BFS warm-up
    print("BFS warm-up...", flush=True)
    bfs_warm_up(pool, max_seconds=BFS_MAX_SECS, verbose=True)

    # 3. Build tensor tables
    print("Building state tensor tables...", flush=True)
    TN, TR, tau0_table, pool_ids, _ = build_state_tables(pool, verbose=True)

    # 4. Verify TorchScript compiles with our setup
    print("\nVerifying torch.jit.script...", end=" ", flush=True)
    _m = RouterV6(TN, TR, tau0_table, pool_ids)
    _s = torch.jit.script(_m)
    del _m, _s
    print("OK")

    # 5. Run experiment
    print("\nRunning context-scaling experiment...", flush=True)
    results = run_context_scaling(TN, TR, tau0_table, pool_ids)

    total_s = time.perf_counter() - t0_all

    # 6. Summary
    print("\n" + "=" * 58)
    print("CONTEXT SCALING SUMMARY")
    print("=" * 58)
    print(f"{'D':>4}  {'acc':>6}  {'vs_chance':>9}  {'entropy':>7}  "
          f"{'transport':>9}  {'alpha0':>6}  verdict")
    print("-" * 58)
    for D in DELAYS:
        r = results[D]
        print(f"  {D:>2}  {r['accuracy']:>6.3f}  "
              f"{r['accuracy']/r['chance']:>9.2f}x  "
              f"{r['route_entropy']:>7.3f}  "
              f"{r['transport_frac']:>9.3f}  "
              f"{r['alpha0']:>6.3f}  {r['verdict']}")
    print(f"\nTotal wall time: {total_s:.1f}s")

    # 7. Write outputs
    write_csv(results)
    write_md(results, total_s)


if __name__ == "__main__":
    main()

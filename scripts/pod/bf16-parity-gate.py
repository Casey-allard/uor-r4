#!/usr/bin/env python3
"""Tabulate the Phase 1 bf16 parity gate (docs/compute/bf16-parity-gate.md).

Reads the gate's sealed report roots under RUNS and prints the frozen decision:
the base NLL table with the seed spread and per-seed differences, the tokens/s
of both arms, and the D19 (`log_recall=off`) session table with its noise.

  scripts/pod/bf16-parity-gate.py RUNS [--json OUT]
"""

import argparse
import json
import statistics
import sys
from pathlib import Path

ARMS = ("f32", "bf16")
SEEDS = (1, 2)
BASE_STEPS = 70609
FT_STEPS = 4000
# Frozen floor on the session criterion (docs/compute/bf16-parity-gate.md §4).
SESSION_FLOOR = 12
NLL_TOLERANCE = 0.01


def load(path: Path):
    try:
        return json.loads(path.read_text())
    except Exception:
        return None


def base_row(runs: Path, arm: str, seed: int):
    root = runs / f"base-{arm}-s{seed}"
    report = load(root / "report.json")
    if report is None:
        return None
    return {
        "nll": report["final"]["nll"],
        "steps": report["completed_steps"],
        "stopped_early": report.get("stopped_early", False),
        "tokens_per_second": report.get("tokens_per_second"),
        "train_seconds": report.get("train_seconds"),
        "precision": report.get("settings", {}).get("precision"),
    }


def session_score(runs: Path, arm: str, seed: int):
    report = load(runs / f"session-off-{arm}-s{seed}" / "report.json")
    if report is None:
        return None
    arm_report = report["arms"]["default"]
    categories = arm_report["by_category"]
    return sum(c["pass"] for c in categories.values()), sum(
        c["of"] for c in categories.values()
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("runs", type=Path)
    parser.add_argument("--json", type=Path)
    args = parser.parse_args()
    runs = args.runs

    lines = []
    out = {"schema": "uor-r4.bf16-parity-gate/1", "runs": str(runs)}
    print(f"# bf16 Phase 1 parity gate — {runs}")

    # --- bases -------------------------------------------------------------
    bases = {(arm, seed): base_row(runs, arm, seed) for arm in ARMS for seed in SEEDS}
    print("\n## Base runs (29M lr 5e-4, 70609 steps, 433.8M tokens)")
    print(f"{'arm':5} {'seed':4} {'steps':>6} {'early':5} {'final dev NLL':>14} {'tokens/s':>10}")
    for arm in ARMS:
        for seed in SEEDS:
            row = bases[(arm, seed)]
            if row is None:
                print(f"{arm:5} {seed:<4} {'-':>6} {'-':5} {'-':>14} {'-':>10}")
                continue
            print(
                f"{arm:5} {seed:<4} {row['steps']:>6} {str(row['stopped_early']):5} "
                f"{row['nll']:>14.8f} {row['tokens_per_second']:>10.0f}"
            )
    out["bases"] = {f"{a}-s{s}": bases[(a, s)] for a in ARMS for s in SEEDS}

    speeds = {a: [bases[(a, s)]["tokens_per_second"] for s in SEEDS if bases[(a, s)]] for a in ARMS}
    if all(speeds[a] for a in ARMS):
        ratio = statistics.median(speeds["bf16"]) / statistics.median(speeds["f32"])
        print(
            f"\ntokens/s median: f32 {statistics.median(speeds['f32']):.0f}, "
            f"bf16 {statistics.median(speeds['bf16']):.0f} -> {ratio:.3f}x"
        )
        out["speedup"] = {
            "f32_median": statistics.median(speeds["f32"]),
            "bf16_median": statistics.median(speeds["bf16"]),
            "ratio": ratio,
        }

    # --- the frozen NLL criterion -----------------------------------------
    decision = {}
    if all(bases.values()) and all(bases[(a, s)]["steps"] == BASE_STEPS for a in ARMS for s in SEEDS):
        spread = abs(bases[("f32", 1)]["nll"] - bases[("f32", 2)]["nll"])
        diffs = {s: abs(bases[("bf16", s)]["nll"] - bases[("f32", s)]["nll"]) for s in SEEDS}
        nll_pass = all(d <= spread and d < NLL_TOLERANCE for d in diffs.values())
        print("\n## Frozen NLL criterion")
        print(f"f32 seed spread {spread:.8f} nats; tolerance 0.01 nats")
        for seed in SEEDS:
            print(
                f"seed {seed}: |bf16 - f32| = {diffs[seed]:.8f} "
                f"({'pass' if diffs[seed] <= spread and diffs[seed] < NLL_TOLERANCE else 'FAIL'})"
            )
        print(f"NLL axis: {'PASS' if nll_pass else 'FAIL'}")
        decision["nll"] = {
            "spread": spread,
            "diffs": diffs,
            "pass": nll_pass,
        }

    # --- sessions ----------------------------------------------------------
    scores = {(arm, seed): session_score(runs, arm, seed) for arm in ARMS for seed in SEEDS}
    print("\n## D19 sessions (log_recall=off) of each fine-tune")
    print(f"{'arm':5} {'seed':4} {'pass':>6} {'of':>6}")
    for arm in ARMS:
        for seed in SEEDS:
            score = scores[(arm, seed)]
            print(f"{arm:5} {seed:<4} " + (f"{score[0]:>6} {score[1]:>6}" if score else f"{'-':>6} {'-':>6}"))
    out["sessions"] = {
        f"{a}-s{s}": ({"pass": scores[(a, s)][0], "of": scores[(a, s)][1]} if scores[(a, s)] else None)
        for a in ARMS
        for s in SEEDS
    }
    if all(scores.values()):
        noise = abs(scores[("f32", 1)][0] - scores[("f32", 2)][0])
        floor = max(noise, SESSION_FLOOR)
        errors = {s: abs(scores[("bf16", s)][0] - scores[("f32", s)][0]) for s in SEEDS}
        session_pass = all(e <= floor for e in errors.values())
        print(f"\nf32 seed noise {noise} turns; frozen floor {SESSION_FLOOR}; allowed {floor}")
        for seed in SEEDS:
            print(
                f"seed {seed}: |bf16 - f32| = {errors[seed]} turns "
                f"({'pass' if errors[seed] <= floor else 'FAIL'})"
            )
        print(f"session axis: {'PASS' if session_pass else 'FAIL'}")
        decision["session"] = {"noise": noise, "floor": floor, "errors": errors, "pass": session_pass}

    if decision:
        adopted = all(part["pass"] for part in decision.values())
        decision["adopt_bf16"] = adopted
        print(f"\n## Decision: {'ADOPT bf16' if adopted else 'DO NOT ADOPT bf16 (yet)'}")
        out["decision"] = decision

    if args.json:
        args.json.write_text(json.dumps(out, indent=2, sort_keys=True) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())

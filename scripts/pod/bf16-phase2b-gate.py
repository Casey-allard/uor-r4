#!/usr/bin/env python3
"""Tabulate the bf16 Phase 2b gate and print the frozen decision.

Implements docs/compute/bf16-phase2b-gate.md sections 3 and 5. It does not define
them: the recipe, the arms and the acceptance rule are frozen before the first
run, and a later change is a new gate with its own record.

  scripts/pod/bf16-phase2b-gate.py RUNS [--json OUT]

  NLL axis   spread = |NLL(current,1) - NLL(current,2)|
             d_s    = |NLL(flash,s) - NLL(current,s)|
             PASS iff, for BOTH seeds, d_s <= spread AND d_s < 0.01 nats
  speed axis the A/B medians; PASS iff flash is >= 5% faster end to end at 29M

Adopt only when both axes pass. A neutral rewrite must not ship (Phase 2a's
record), and a losing NLL costs nothing but nats: a gap is a localiser, not a
verdict on the mechanism (D20 section 4).
"""
from __future__ import annotations

import argparse
import json
import statistics
import sys
from pathlib import Path

ARMS = ("current", "flash")
REFERENCE = "current"
SEEDS = (1, 2)
BASE_STEPS = 70609
NLL_TOLERANCE = 0.01
SPEED_FLOOR = 0.05  # 5% end-to-end at 29M, else the gate fails on speed alone
SPEED_ROUNDS = 3


def load(path: Path):
    try:
        return json.loads(path.read_text())
    except Exception:
        return None


def base_row(runs: Path, arm: str, seed: int):
    report = load(runs / f"base-{arm}-s{seed}" / "report.json")
    if not report:
        return None
    return {
        "nll": report["final"]["nll"],
        "steps": report.get("completed_steps"),
        "stopped_early": report.get("stopped_early", False),
        "tokens_per_second": report.get("tokens_per_second"),
    }


def ab_speeds(runs: Path):
    out = {}
    for arm in ARMS:
        vals = []
        for r in range(1, SPEED_ROUNDS + 1):
            report = load(runs / "ab" / f"{arm}-r{r}" / "report.json")
            if report and report.get("tokens_per_second"):
                vals.append(float(report["tokens_per_second"]))
        out[arm] = vals
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("runs", type=Path)
    ap.add_argument("--json", type=Path)
    args = ap.parse_args()
    runs = args.runs

    bases = {(a, s): base_row(runs, a, s) for a in ARMS for s in SEEDS}
    out = {"runs": str(runs), "base_steps": BASE_STEPS}

    print(f"{'arm':8} {'seed':<5} {'steps':>7} {'stopped':>8} {'final.nll':>15} {'tok/s':>10}")
    for a in ARMS:
        for s in SEEDS:
            row = bases[(a, s)]
            if not row:
                print(f"{a:8} {s:<5} {'MISSING':>7}")
                continue
            print(f"{a:8} {s:<5} {str(row['steps']):>7} {str(row['stopped_early']):>8} "
                  f"{row['nll']:>15.8f} {row['tokens_per_second'] or 0:>10.0f}")
    out["bases"] = {f"{a}-s{s}": bases[(a, s)] for a in ARMS for s in SEEDS}

    complete = all(
        bases[(a, s)] and bases[(a, s)]["steps"] == BASE_STEPS
        and not bases[(a, s)]["stopped_early"]
        for a in ARMS for s in SEEDS
    )

    nll_pass = None
    if complete:
        spread = abs(bases[(REFERENCE, 1)]["nll"] - bases[(REFERENCE, 2)]["nll"])
        diffs = {s: abs(bases[("flash", s)]["nll"] - bases[(REFERENCE, s)]["nll"]) for s in SEEDS}
        nll_pass = all(d <= spread and d < NLL_TOLERANCE for d in diffs.values())
        print()
        print(f"reference seed spread (|NLL(current,1) - NLL(current,2)|): {spread:.8f}")
        print(f"{'seed':<5} {'|flash-current|':>16} {'<= spread':>10} {'< 0.01':>7}")
        for s in SEEDS:
            d = diffs[s]
            print(f"{s:<5} {d:>16.8f} {str(d <= spread):>10} {str(d < NLL_TOLERANCE):>7}")
        print(f"NLL axis: {'PASS' if nll_pass else 'FAIL'}")
        out["nll"] = {"spread": spread, "diffs": {str(s): diffs[s] for s in SEEDS}, "pass": nll_pass}
    else:
        print()
        print(f"NLL axis: NOT RUN — a run counts only at completed_steps == {BASE_STEPS} "
              "with stopped_early false; a capped root is resumed, never used.")
        out["nll"] = None

    speeds = ab_speeds(runs)
    speed_pass = None
    if all(speeds[a] for a in ARMS):
        med = {a: statistics.median(speeds[a]) for a in ARMS}
        ratio = med["flash"] / med["current"]
        speed_pass = ratio >= 1.0 + SPEED_FLOOR
        print()
        for a in ARMS:
            print(f"ab {a:8} rounds={[round(v) for v in speeds[a]]} median={med[a]:.0f} tok/s")
        print(f"speed ratio flash/current: {ratio:.4f}  (floor {1.0 + SPEED_FLOOR:.2f})")
        print(f"speed axis: {'PASS' if speed_pass else 'FAIL'}")
        out["speed"] = {"medians": med, "rounds": speeds, "ratio": ratio, "pass": speed_pass}
    else:
        print()
        print("speed axis: NOT RUN — the A/B stage has not produced all rounds.")
        out["speed"] = None

    if nll_pass is None or speed_pass is None:
        verdict = "INCOMPLETE"
    elif nll_pass and speed_pass:
        verdict = "ADOPT"
    else:
        verdict = "DO NOT ADOPT"
    print()
    print(f"VERDICT: {verdict}")
    out["verdict"] = verdict

    if args.json:
        args.json.write_text(json.dumps(out, indent=2) + "\n")
        print(f"wrote {args.json}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Larger guarded real-signal benchmark on the paired replay slice."""

from __future__ import annotations

import csv
from pathlib import Path

from run_prime_transport_guarded_real_signal_trial import (
    evaluate_angular_guarded,
    evaluate_prime_transport,
    summarize_policy,
)
from real_signal_benchmark_wrapper import AI_RESEARCH_ROOT, ROOT, build_real_signal_traces


OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_larger_real_signal_trial.csv"
REPLAY_PATHS = (
    AI_RESEARCH_ROOT / "MUDBench" / "tmp" / "timing_mode_eval_v1" / "off_baseline.json.replay.json",
    AI_RESEARCH_ROOT / "MUDBench" / "tmp" / "timing_mode_eval_v1" / "equal-cadence_baseline.json.replay.json",
)


def write_rows(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        raise ValueError(f"no rows to write: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def main() -> None:
    traces = build_real_signal_traces(replay_paths=REPLAY_PATHS)
    rows: list[dict[str, object]] = []
    for trace in traces:
        rows.append(evaluate_angular_guarded(trace))
        rows.append(evaluate_prime_transport(trace))

    rows.append(summarize_policy(rows, "angular_hopf_guarded"))
    rows.append(summarize_policy(rows, "base_gap"))
    write_rows(OUT_CSV, rows)
    print(f"prime_transport_larger_real_signal_trial_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()

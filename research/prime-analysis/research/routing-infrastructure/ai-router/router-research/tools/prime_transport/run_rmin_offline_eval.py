#!/usr/bin/env python3
"""Offline evaluation harness for prime-transport routing state candidates."""

from __future__ import annotations

import csv
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
RESULTS_DIR = ROOT / "results" / "prime_transport_recursive_system"

MIN_STATE_PATH = RESULTS_DIR / "visible_threshold_minimal_predictive_state_comparison.csv"
OUT_PATH = RESULTS_DIR / "prime_transport_rmin_offline_eval_summary.csv"


def read_rows(path: Path) -> list[dict[str, str]]:
    with path.open("r", encoding="utf-8", newline="") as handle:
        return list(csv.DictReader(handle))


def write_rows(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        raise ValueError(f"no rows to write: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def unresolved_from_purity(purity: float) -> float:
    return 1.0 - purity


def prototype_reading(state_name: str, label_ratio: float, split_purity: float) -> str:
    if state_name == "R_static":
        return "Exact but too detailed; not a practical compressed router state."
    if state_name == "R_min":
        return "Best first prototype tradeoff: materially predictive and still smaller than spin."
    if state_name == "R_full":
        return "Reference exact predictor; too costly as the first prototype target."
    raise ValueError(f"unknown state: {state_name}")


def recommendation(state_name: str) -> str:
    if state_name == "R_min":
        return "prototype_target"
    if state_name == "R_full":
        return "reference_only"
    return "baseline_only"


def main() -> None:
    source_rows = read_rows(MIN_STATE_PATH)
    source_by_name = {row["state_name"]: row for row in source_rows}

    mapping = {
        "R_static": "chart_exact_address_(b_phi_r)",
        "R_min": "next_return_gap",
        "R_full": "full_spin_predictive_state",
    }

    summary_rows: list[dict[str, object]] = []
    for state_name, source_name in mapping.items():
        row = source_by_name[source_name]
        membership_purity = float(row["mean_membership_purity_fraction"])
        split_purity = float(row["mean_split_partition_purity_fraction"])
        label_ratio = float(row["mean_label_ratio_to_spin"])
        combined_capture = float(row["mean_combined_capture_fraction_of_spin"])

        summary_rows.append(
            {
                "state": state_name,
                "state_definition": row["state_name"],
                "predictive_membership_purity": membership_purity,
                "split_partition_purity": split_purity,
                "label_complexity_ratio_to_spin": label_ratio,
                "combined_capture_fraction_of_spin": combined_capture,
                "promotion_needed_fraction": unresolved_from_purity(split_purity),
                "prototype_reading": prototype_reading(state_name, label_ratio, split_purity),
                "recommendation": recommendation(state_name),
            }
        )

    write_rows(OUT_PATH, summary_rows)
    print(f"summary_csv,{OUT_PATH}")


if __name__ == "__main__":
    main()

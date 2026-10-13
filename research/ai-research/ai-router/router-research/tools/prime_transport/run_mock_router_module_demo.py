#!/usr/bin/env python3
"""Run a tiny offline demo of the mock router module on bounded summary rows."""

from __future__ import annotations

import csv
from pathlib import Path

from mock_router_module import (
    initialize_state,
    promote_state,
    score_or_route,
    should_promote,
    update_state,
)


ROOT = Path(__file__).resolve().parents[2]
SUMMARY_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_rmin_offline_eval_summary.csv"
OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_mock_router_demo.csv"


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


def representative_gap(unresolved_fraction: float) -> int:
    return max(1, int(round(10.0 * float(unresolved_fraction))))


def main() -> None:
    summary_rows = read_rows(SUMMARY_CSV)
    demo_rows: list[dict[str, object]] = []

    for idx, row in enumerate(summary_rows):
        state_name = row["state"]
        unresolved_fraction = float(row["promotion_needed_fraction"])
        base_phase = 7 + idx
        phi = (idx % 11, (idx * 2) % 13)
        fiber_moduli = (11, 13)

        if state_name == "R_static":
            state = initialize_state(mode="R_static", b=base_phase, phi=phi, r=2)
            updated = update_state(state, fiber_moduli=fiber_moduli)
        elif state_name == "R_min":
            state = initialize_state(
                mode="R_min",
                b=base_phase,
                phi=phi,
                r=2,
                next_return_gap=representative_gap(unresolved_fraction),
            )
            updated = update_state(state, fiber_moduli=fiber_moduli)
        else:
            state = initialize_state(mode="R_full", b=base_phase, r=2, spin_H=(1, 0, 1, 1, 0))
            updated = update_state(state, next_bit=1)

        route = score_or_route(updated)
        promote = should_promote(updated, unresolved_fraction=unresolved_fraction)
        promoted_mode = ""
        if promote:
            promoted = promote_state(updated, spin_H=(1, 0, 1, 1, 0))
            promoted_mode = promoted.mode

        demo_rows.append(
            {
                "state": state_name,
                "initial_mode": state.mode,
                "updated_mode": updated.mode,
                "route_key": route["route_key"],
                "score_hint": route["score_hint"],
                "unresolved_fraction": unresolved_fraction,
                "should_promote": int(promote),
                "promoted_mode": promoted_mode,
            }
        )

    write_rows(OUT_CSV, demo_rows)
    print(f"demo_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()

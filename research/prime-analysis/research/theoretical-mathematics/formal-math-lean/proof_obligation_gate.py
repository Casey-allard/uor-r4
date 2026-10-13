#!/usr/bin/env python3
"""Strict O1-O5 obligation gate and ledger updater."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List


ALLOWED = {
    "O1": "A1 residual bound analytic uplift",
    "O2": "A2 infinite-tail truncation proof",
    "O3": "A3 bridge growth/offdiag analytic closure",
    "O4": "A4 base-uniform asymptotic constants",
    "O5": "RH-equivalent endpoint implication",
}


def now_utc() -> str:
    return datetime.now(timezone.utc).isoformat()


def parse_csv(raw: str) -> List[str]:
    return [x.strip() for x in raw.split(",") if x.strip()]


def init_ledger() -> Dict[str, Any]:
    return {
        "schema_version": "1.0",
        "created_utc": now_utc(),
        "updated_utc": now_utc(),
        "rules": {
            "fixed_obligations": list(ALLOWED.keys()),
            "no_new_obligation_categories": True,
            "each_step_must_target_exactly_one_obligation": True,
            "each_step_must_state_removed_assumption": True,
            "each_step_must_attach_evidence": True,
        },
        "obligations": {
            k: {
                "title": v,
                "state": "open",
                "reduction_count": 0,
                "events": [],
            }
            for k, v in ALLOWED.items()
        },
        "history": [],
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="Update strict O1-O5 proof obligation ledger.")
    ap.add_argument("--ledger", default="research/output/proof_obligation_ledger.json")
    ap.add_argument("--obligation", required=True, choices=sorted(ALLOWED.keys()))
    ap.add_argument("--summary", required=True, help="One-line description of the step.")
    ap.add_argument(
        "--removed-assumption",
        required=True,
        help="Specific assumption reduced or eliminated in this step.",
    )
    ap.add_argument(
        "--remaining-work",
        required=True,
        help="Concrete remaining work for the same obligation.",
    )
    ap.add_argument(
        "--evidence",
        required=True,
        help="Comma-separated artifact paths proving the step.",
    )
    ap.add_argument("--close", action="store_true", help="Mark obligation closed.")
    ap.add_argument("--output-md", default="")
    args = ap.parse_args()

    ledger_path = Path(args.ledger)
    if ledger_path.exists():
        ledger = json.loads(ledger_path.read_text(encoding="utf-8"))
    else:
        ledger = init_ledger()

    # Hard guard: no obligation categories outside O1-O5.
    found = set(ledger.get("obligations", {}).keys())
    if found - set(ALLOWED.keys()):
        raise RuntimeError("ledger contains non-canonical obligations; refusing update")

    ob = args.obligation
    ev = parse_csv(args.evidence)
    if not ev:
        raise ValueError("at least one evidence artifact is required")

    event = {
        "timestamp_utc": now_utc(),
        "obligation": ob,
        "summary": args.summary,
        "removed_assumption": args.removed_assumption,
        "remaining_work": args.remaining_work,
        "evidence": ev,
        "closed_in_step": bool(args.close),
    }

    ob_row = ledger["obligations"][ob]
    ob_row["events"].append(event)
    ob_row["reduction_count"] = int(ob_row.get("reduction_count", 0)) + 1
    if args.close:
        ob_row["state"] = "closed"
    else:
        ob_row["state"] = "open"

    ledger["history"].append(event)
    ledger["updated_utc"] = now_utc()

    ledger_path.parent.mkdir(parents=True, exist_ok=True)
    ledger_path.write_text(json.dumps(ledger, indent=2) + "\n", encoding="utf-8")

    md_path = Path(args.output_md) if args.output_md else ledger_path.with_suffix(".md")
    lines = [
        "# Proof Obligation Ledger",
        "",
        f"- Updated (UTC): `{ledger['updated_utc']}`",
        "",
        "## Obligation States",
        "",
        "| id | state | reductions |",
        "|---|---|---:|",
    ]
    for oid in sorted(ALLOWED.keys()):
        row = ledger["obligations"][oid]
        lines.append(f"| {oid} | {row['state']} | {row['reduction_count']} |")

    lines += [
        "",
        "## Latest Event",
        "",
        f"- obligation: `{event['obligation']}`",
        f"- summary: {event['summary']}",
        f"- removed assumption: {event['removed_assumption']}",
        f"- remaining work: {event['remaining_work']}",
        "- evidence:",
    ]
    for p in event["evidence"]:
        lines.append(f"  - `{p}`")
    lines.append("")
    md_path.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"ledger_json": str(ledger_path), "ledger_md": str(md_path)}, indent=2))


if __name__ == "__main__":
    main()

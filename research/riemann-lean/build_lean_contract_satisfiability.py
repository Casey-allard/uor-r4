#!/usr/bin/env python3
"""Validate Lean contract bridge values against expected lock rules."""

from __future__ import annotations

import argparse
import json
import math
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Tuple


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def _index_bindings(contract: Dict[str, Any]) -> Dict[str, Dict[str, Any]]:
    out: Dict[str, Dict[str, Any]] = {}
    for row in contract.get("bindings", []):
        out[row["field"]] = row
    return out


def _close(a: float, b: float, tol: float = 1e-12) -> bool:
    return math.isclose(a, b, rel_tol=0.0, abs_tol=tol)


def _check(name: str, passed: bool, actual: Any, expected: Any, note: str) -> Dict[str, Any]:
    return {
        "check_id": name,
        "pass": bool(passed),
        "actual": actual,
        "expected": expected,
        "note": note,
    }


def _run_checks(bridge: Dict[str, Any]) -> Tuple[List[Dict[str, Any]], Dict[str, Any]]:
    contracts = bridge["contracts"]
    l1 = _index_bindings(contracts["L1ArtifactContract"])
    l2 = _index_bindings(contracts["L2ArtifactContract"])
    o2 = _index_bindings(contracts["O2Constants"])
    o3 = _index_bindings(contracts["O3Constants"])

    checks: List[Dict[str, Any]] = []

    # O2 exact locks.
    checks.append(_check("o2_nbound_c1_lock", _close(float(o2["nbound_c1"]["value"]), 0.1038), o2["nbound_c1"]["value"], 0.1038, "HSW2021 lock"))
    checks.append(_check("o2_nbound_c2_lock", _close(float(o2["nbound_c2"]["value"]), 0.2573), o2["nbound_c2"]["value"], 0.2573, "HSW2021 lock"))
    checks.append(_check("o2_nbound_c3_lock", _close(float(o2["nbound_c3"]["value"]), 9.3675), o2["nbound_c3"]["value"], 9.3675, "HSW2021 lock"))
    checks.append(_check("o2_nbound_h_lock", _close(float(o2["nbound_h"]["value"]), 1.0), o2["nbound_h"]["value"], 1.0, "HSW2021 lock"))
    checks.append(
        _check(
            "o2_citation_url_lock",
            str(o2["citation_url"]["value"]) == "https://arxiv.org/abs/2107.06506",
            o2["citation_url"]["value"],
            "https://arxiv.org/abs/2107.06506",
            "Primary citation URL lock",
        )
    )

    # O3 exact locks.
    checks.append(_check("o3_A_offabs_lock", _close(float(o3["A_offabs"]["value"]), 0.0), o3["A_offabs"]["value"], 0.0, "Direct O3 lock"))
    checks.append(_check("o3_C_offabs_lock", _close(float(o3["C_offabs"]["value"]), 0.03292827711413939), o3["C_offabs"]["value"], 0.03292827711413939, "Direct O3 lock"))
    checks.append(_check("o3_k_abs_lock", _close(float(o3["k_abs"]["value"]), 0.005725212627704354), o3["k_abs"]["value"], 0.005725212627704354, "Direct O3 lock"))
    checks.append(_check("o3_A_diag_lock", _close(float(o3["A_diag"]["value"]), 0.0), o3["A_diag"]["value"], 0.0, "Direct O3 lock"))
    checks.append(_check("o3_C_diag_lock", _close(float(o3["C_diag"]["value"]), 1.0), o3["C_diag"]["value"], 1.0, "Direct O3 lock"))
    checks.append(_check("o3_A_E2_lock", _close(float(o3["A_E2"]["value"]), 0.0), o3["A_E2"]["value"], 0.0, "Direct O3 lock"))
    checks.append(_check("o3_C_E2_lock", _close(float(o3["C_E2"]["value"]), 1.1195893906678458), o3["C_E2"]["value"], 1.1195893906678458, "Direct O3 lock"))

    # L1/L2 contract sanity.
    c0 = float(l1["Ctr_components.C0_ref_O4"]["value"])
    babs = float(l1["Ctr_components.b_ref_abs"]["value"])
    ctr = float(l1["Ctr"]["value"])
    checks.append(_check("l1_ctr_formula", _close(ctr, c0 + babs), ctr, c0 + babs, "Ctr := C0_ref_O4 + |b_ref|"))
    checks.append(_check("l2_ch_nonnegative", float(l2["CH"]["value"]) >= 0.0, l2["CH"]["value"], ">= 0", "CH nonnegative"))
    checks.append(_check("l2_A_H_endpoint_class", _close(float(l2["A_H"]["value"]), 2.0), l2["A_H"]["value"], 2.0, "Endpoint class exponent target"))

    failed = [c for c in checks if not c["pass"]]
    summary = {
        "total_checks": len(checks),
        "passed_checks": len(checks) - len(failed),
        "failed_checks": len(failed),
        "status": "pass" if not failed else "fail",
        "failed_check_ids": [f["check_id"] for f in failed],
    }
    return checks, summary


def main() -> None:
    ap = argparse.ArgumentParser(description="Validate Lean contract bridge satisfiability")
    ap.add_argument("--bridge-json", default="research/output/lean_contract_bridge_2026-02-17.json")
    ap.add_argument("--output-json", default="research/output/lean_contract_satisfiability_2026-02-17.json")
    ap.add_argument("--output-md", default="research/output/lean_contract_satisfiability_2026-02-17.md")
    args = ap.parse_args()

    bridge = _read_json(args.bridge_json)
    checks, summary = _run_checks(bridge)

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "bridge_json": args.bridge_json,
        "summary": summary,
        "checks": checks,
    }

    out_json = Path(args.output_json)
    out_json.parent.mkdir(parents=True, exist_ok=True)
    out_json.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    lines = [
        "# Lean Contract Satisfiability",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        f"- Bridge: `{args.bridge_json}`",
        f"- Status: `{summary['status']}`",
        f"- Passed: `{summary['passed_checks']}/{summary['total_checks']}`",
        "",
        "| check_id | pass | actual | expected | note |",
        "|---|---|---:|---:|---|",
    ]
    for c in checks:
        lines.append(f"| {c['check_id']} | {c['pass']} | {c['actual']} | {c['expected']} | {c['note']} |")
    lines.append("")
    Path(args.output_md).write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out_json), "md": args.output_md, "status": summary["status"]}, indent=2))


if __name__ == "__main__":
    main()

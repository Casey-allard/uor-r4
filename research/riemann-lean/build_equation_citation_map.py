#!/usr/bin/env python3
"""Build equation-level citation map for theorem constants."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def _write_json(path: str, payload: Dict[str, Any]) -> None:
    out = Path(path)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def _build_entries(
    provenance: Dict[str, Any],
    o2_pack: Dict[str, Any],
) -> List[Dict[str, Any]]:
    entries: List[Dict[str, Any]] = []
    lock = o2_pack["citation_lock"]
    locked_url = lock["url"]
    locked_id = locked_url.rstrip("/").split("/")[-1]
    expected_primary_id = "2107.06506"

    # O2 zero-count constants carry the only external citation lock in current theorem packs.
    for row in provenance["constants"]:
        if row["symbol"] not in {"nbound_c1", "nbound_c2", "nbound_c3", "nbound_h"}:
            continue
        entries.append(
            {
                "symbol": row["symbol"],
                "value": row["value"],
                "equation_id": "HSW2021-ABS-ZEROCOUNT",
                "equation_text": "|N(T) - T/(2π) log(T/(2πe))| <= 0.1038 log T + 0.2573 log log T + 9.3675",
                "source_type": "primary_arxiv_abstract",
                "source_url": "https://arxiv.org/abs/2107.06506",
                "source_locator": "abstract inequality (arXiv page text)",
                "verification_status": "metadata_verified",
                "notes": [
                    "Constant is referenced in O2 theorem closure pack under citation_lock.",
                    f"Current lock URL uses arXiv id `{locked_id}`; expected source id for these constants is `{expected_primary_id}`.",
                ],
            }
        )

    # Secondary anchoring: thesis statement explicitly labels this as equation (1.7).
    entries.append(
        {
            "symbol": "HSW_zero_count_ineq_bundle",
            "value": {
                "nbound_c1": lock["constants"]["nbound_c1"],
                "nbound_c2": lock["constants"]["nbound_c2"],
                "nbound_c3": lock["constants"]["nbound_c3"],
                "nbound_h": lock["constants"]["nbound_h"],
            },
            "equation_id": "FARZANFARD2025-EQ-1.7",
            "equation_text": "|N(T) - T/(2π) log(T/(2πe))| <= 0.1038 log T + 0.2573 log log T + 9.3675",
            "source_type": "secondary_thesis",
            "source_url": "https://opus.uleth.ca/server/api/core/bitstreams/9156431f-a802-4678-864a-2f9fb4720196/content",
            "source_locator": "Eq. (1.7), chapter 1",
            "verification_status": "text_verified_secondary",
            "notes": [
                "Used as secondary locator with explicit equation numbering.",
                "Primary anchor remains arXiv:2107.06506 abstract.",
            ],
        }
    )

    return entries


def main() -> None:
    ap = argparse.ArgumentParser(description="Build equation-level citation map")
    ap.add_argument("--provenance-json", default="research/output/proof_constant_provenance_2026-02-17.json")
    ap.add_argument("--o2-pack", default="research/output/o2_theorem_closure_pack_2026-02-17.json")
    ap.add_argument("--output-json", default="research/output/proof_equation_citation_map_2026-02-17.json")
    ap.add_argument("--output-md", default="research/output/proof_equation_citation_map_2026-02-17.md")
    args = ap.parse_args()

    provenance = _read_json(args.provenance_json)
    o2_pack = _read_json(args.o2_pack)
    entries = _build_entries(provenance, o2_pack)
    lock_id = o2_pack["citation_lock"]["url"].rstrip("/").split("/")[-1]
    id_mismatch = lock_id != "2107.06506"

    recommended_actions = []
    if id_mismatch:
        recommended_actions.append(
            "Update O2 citation_lock URL to arXiv:2107.06506 (or justify 2108.00890 with equation-level evidence)."
        )
    recommended_actions.append(
        "Attach page/equation locator from final published JNT version once accessible."
    )

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "status": "equation_citation_indexed",
        "inputs": {
            "provenance_json": args.provenance_json,
            "o2_pack": args.o2_pack,
        },
        "citation_lock_diagnostics": {
            "declared_label": o2_pack["citation_lock"]["label"],
            "declared_url": o2_pack["citation_lock"]["url"],
            "declared_arxiv_id": lock_id,
            "expected_arxiv_id_for_locked_constants": "2107.06506",
            "arxiv_id_mismatch": id_mismatch,
        },
        "equation_entries": entries,
        "recommended_actions": recommended_actions,
    }
    _write_json(args.output_json, payload)

    lines = [
        "# Proof Equation Citation Map",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Citation Lock Diagnostics",
        "",
        f"- Declared label: `{payload['citation_lock_diagnostics']['declared_label']}`",
        f"- Declared URL: `{payload['citation_lock_diagnostics']['declared_url']}`",
        f"- Declared arXiv id: `{payload['citation_lock_diagnostics']['declared_arxiv_id']}`",
        f"- Expected arXiv id: `{payload['citation_lock_diagnostics']['expected_arxiv_id_for_locked_constants']}`",
        f"- ID mismatch: `{payload['citation_lock_diagnostics']['arxiv_id_mismatch']}`",
        "",
        "## Equation Entries",
        "",
        "| equation_id | symbols | source | status |",
        "|---|---|---|---|",
    ]

    for e in entries:
        if isinstance(e["value"], dict):
            symbols = ",".join(sorted(e["value"].keys()))
        else:
            symbols = e["symbol"]
        lines.append(
            f"| {e['equation_id']} | {symbols} | [{e['source_type']}]({e['source_url']}) | {e['verification_status']} |"
        )

    lines += [
        "",
        "## Recommended Actions",
        "",
    ]
    for x in payload["recommended_actions"]:
        lines.append(f"- {x}")
    lines.append("")
    Path(args.output_md).write_text("\n".join(lines), encoding="utf-8")

    print(
        json.dumps(
            {
                "json": args.output_json,
                "md": args.output_md,
                "entries": len(entries),
                "arxiv_id_mismatch": id_mismatch,
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()

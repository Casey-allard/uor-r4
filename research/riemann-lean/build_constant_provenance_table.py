#!/usr/bin/env python3
"""Build constant provenance table for proof-grade audit and formalization."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def _emit_entry(
    rows: List[Dict[str, Any]],
    *,
    symbol: str,
    value: Any,
    obligation: str,
    requirement: str,
    artifact: str,
    field_path: str,
    role: str,
    citation_label: str = "",
    citation_url: str = "",
) -> None:
    rows.append(
        {
            "symbol": symbol,
            "value": value,
            "obligation": obligation,
            "requirement": requirement,
            "artifact": artifact,
            "field_path": field_path,
            "role": role,
            "citation_label": citation_label,
            "citation_url": citation_url,
        }
    )


def main() -> None:
    ap = argparse.ArgumentParser(description="Build constant provenance table")
    ap.add_argument("--o1-pack", default="research/output/o1_theorem_closure_pack_2026-02-17.json")
    ap.add_argument("--o2-pack", default="research/output/o2_theorem_closure_pack_2026-02-17.json")
    ap.add_argument("--o3-r1", default="research/output/o3_l_offabs_theorem_closure_2026-02-17.json")
    ap.add_argument("--o3-r2", default="research/output/o3_l_offsign_theorem_closure_2026-02-17.json")
    ap.add_argument("--o3-r3", default="research/output/o3_l_diag_theorem_closure_2026-02-17.json")
    ap.add_argument("--o3-r4", default="research/output/o3_l_asm_theorem_closure_2026-02-17.json")
    ap.add_argument("--o4-pack", default="research/output/o4_theorem_closure_pack_2026-02-17.json")
    ap.add_argument("--o5-closure", default="research/output/o5_theorem_closure_2026-02-17.json")
    ap.add_argument("--output-json", default="research/output/proof_constant_provenance_2026-02-17.json")
    ap.add_argument("--output-md", default="research/output/proof_constant_provenance_2026-02-17.md")
    args = ap.parse_args()

    o1 = _read_json(args.o1_pack)
    o2 = _read_json(args.o2_pack)
    o3r1 = _read_json(args.o3_r1)
    o3r2 = _read_json(args.o3_r2)
    o3r3 = _read_json(args.o3_r3)
    o3r4 = _read_json(args.o3_r4)
    o4 = _read_json(args.o4_pack)
    o5 = _read_json(args.o5_closure)

    rows: List[Dict[str, Any]] = []

    _emit_entry(
        rows,
        symbol="C0_ref_O1",
        value=o1["constants"]["C0_ref"],
        obligation="O1",
        requirement="O1-R1/O1-R2/O1-R3",
        artifact=args.o1_pack,
        field_path="constants.C0_ref",
        role="Residual/reference constant used in O1 theorem pack.",
    )
    _emit_entry(
        rows,
        symbol="a_ref",
        value=o1["constants"]["a_ref"],
        obligation="O1",
        requirement="O1-R1/O1-R2/O1-R3",
        artifact=args.o1_pack,
        field_path="constants.a_ref",
        role="Reference asymptotic coefficient in O1 pack.",
    )
    _emit_entry(
        rows,
        symbol="b_ref",
        value=o1["constants"]["b_ref"],
        obligation="O1",
        requirement="O1-R1/O1-R2/O1-R3",
        artifact=args.o1_pack,
        field_path="constants.b_ref",
        role="Reference asymptotic coefficient in O1 pack.",
    )
    _emit_entry(
        rows,
        symbol="m_ref",
        value=o1["constants"]["m_ref"],
        obligation="O1",
        requirement="O1-R1/O1-R2/O1-R3",
        artifact=args.o1_pack,
        field_path="constants.m_ref",
        role="Reference truncation/control index.",
    )

    citation = o2["citation_lock"]
    for key, value in citation["constants"].items():
        _emit_entry(
            rows,
            symbol=key,
            value=value,
            obligation="O2",
            requirement="O2-R1",
            artifact=args.o2_pack,
            field_path=f"citation_lock.constants.{key}",
            role="Explicit zero-count constant in O2 citation lock.",
            citation_label=citation["label"],
            citation_url=citation["url"],
        )
    _emit_entry(
        rows,
        symbol="k_tau",
        value=o2["tau_rate"]["k_tau"],
        obligation="O2",
        requirement="O2-R3",
        artifact=args.o2_pack,
        field_path="tau_rate.k_tau",
        role="Exponential decay rate in tau envelope.",
    )
    _emit_entry(
        rows,
        symbol="c0_tau",
        value=o2["tau_rate"]["c0_tau"],
        obligation="O2",
        requirement="O2-R3",
        artifact=args.o2_pack,
        field_path="tau_rate.c0_tau",
        role="Leading envelope constant in tau bound.",
    )

    _emit_entry(
        rows,
        symbol="A_offabs",
        value=o3r1["proved_constants"]["A_offabs"],
        obligation="O3",
        requirement="O3-R1",
        artifact=args.o3_r1,
        field_path="proved_constants.A_offabs",
        role="Log exponent in absolute off-diagonal envelope.",
    )
    _emit_entry(
        rows,
        symbol="C_offabs",
        value=o3r1["proved_constants"]["C_offabs"],
        obligation="O3",
        requirement="O3-R1",
        artifact=args.o3_r1,
        field_path="proved_constants.C_offabs",
        role="Leading constant in absolute off-diagonal envelope.",
    )
    for key in ("k_pos", "k_neg", "k_abs", "eps_sign", "neg_over_abs_cap"):
        _emit_entry(
            rows,
            symbol=key,
            value=o3r2["proved_constants"][key],
            obligation="O3",
            requirement="O3-R2",
            artifact=args.o3_r2,
            field_path=f"proved_constants.{key}",
            role="Sign-sensitive reduction constants.",
        )
    _emit_entry(
        rows,
        symbol="A_diag",
        value=o3r3["proved_constants"]["A_diag"],
        obligation="O3",
        requirement="O3-R3",
        artifact=args.o3_r3,
        field_path="proved_constants.A_diag",
        role="Log exponent in diagonal envelope.",
    )
    _emit_entry(
        rows,
        symbol="C_diag",
        value=o3r3["proved_constants"]["C_diag"],
        obligation="O3",
        requirement="O3-R3",
        artifact=args.o3_r3,
        field_path="proved_constants.C_diag",
        role="Leading constant in diagonal envelope.",
    )
    _emit_entry(
        rows,
        symbol="A_E2",
        value=o3r4["proved_constants"]["A_E2"],
        obligation="O3",
        requirement="O3-R4",
        artifact=args.o3_r4,
        field_path="proved_constants.A_E2",
        role="Log exponent in assembled E2/x bound.",
    )
    _emit_entry(
        rows,
        symbol="C_E2",
        value=o3r4["proved_constants"]["C_E2"],
        obligation="O3",
        requirement="O3-R4",
        artifact=args.o3_r4,
        field_path="proved_constants.C_E2",
        role="Leading constant in assembled E2/x bound.",
    )

    _emit_entry(
        rows,
        symbol="a_ref",
        value=o4["uniform_constants"]["a_ref"],
        obligation="O4",
        requirement="O4-R1/O4-R2/O4-R3",
        artifact=args.o4_pack,
        field_path="uniform_constants.a_ref",
        role="Wheel-uniform asymptotic coefficient in O4 pack.",
    )
    _emit_entry(
        rows,
        symbol="b_ref",
        value=o4["uniform_constants"]["b_ref"],
        obligation="O4",
        requirement="O4-R1/O4-R2/O4-R3",
        artifact=args.o4_pack,
        field_path="uniform_constants.b_ref",
        role="Wheel-uniform asymptotic coefficient in O4 pack.",
    )
    _emit_entry(
        rows,
        symbol="C0_ref_O4",
        value=o4["uniform_constants"]["C0_ref"],
        obligation="O4",
        requirement="O4-R1/O4-R2/O4-R3",
        artifact=args.o4_pack,
        field_path="uniform_constants.C0_ref",
        role="Wheel-uniform constant (distinct context from O1 C0_ref).",
    )
    _emit_entry(
        rows,
        symbol="C_delta",
        value=o4["uniform_constants"]["C_delta"],
        obligation="O4",
        requirement="O4-R1/O4-R2/O4-R3",
        artifact=args.o4_pack,
        field_path="uniform_constants.C_delta",
        role="Uniform correction/delta bound constant.",
    )
    _emit_entry(
        rows,
        symbol="C_H",
        value=o4["uniform_constants"]["C_H"],
        obligation="O4",
        requirement="O4-R1/O4-R2/O4-R3",
        artifact=args.o4_pack,
        field_path="uniform_constants.C_H",
        role="Wheel-uniform bridge/envelope constant.",
    )

    consistency = {
        "a_ref_consistent_O1_O4": abs(float(o1["constants"]["a_ref"]) - float(o4["uniform_constants"]["a_ref"])) < 1e-15,
        "b_ref_consistent_O1_O4": abs(float(o1["constants"]["b_ref"]) - float(o4["uniform_constants"]["b_ref"])) < 1e-15,
        "k_abs_matches_offabs_reference": abs(float(o3r1["compatibility_checks"]["sign_cap_reference"]["k_abs"]) - float(o3r2["proved_constants"]["k_abs"])) < 1e-15,
    }

    payload = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "status": "theorem_constants_indexed",
        "o5_statement": o5["theorem_statement"],
        "artifacts": {
            "o1_pack": args.o1_pack,
            "o2_pack": args.o2_pack,
            "o3_r1": args.o3_r1,
            "o3_r2": args.o3_r2,
            "o3_r3": args.o3_r3,
            "o3_r4": args.o3_r4,
            "o4_pack": args.o4_pack,
            "o5_closure": args.o5_closure,
        },
        "constants": rows,
        "consistency_checks": consistency,
        "notes": [
            "C0_ref appears in both O1 and O4 with different numerical values due to different closure contexts; kept as C0_ref_O1 and C0_ref_O4.",
            "Citation-locked constants are tagged with citation label/url for independent source audit.",
        ],
    }

    out_json = Path(args.output_json)
    out_json.parent.mkdir(parents=True, exist_ok=True)
    out_json.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md_lines = [
        "# Proof Constant Provenance",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        f"- Status: `{payload['status']}`",
        f"- O5 statement: {payload['o5_statement']}",
        "",
        "## Constants",
        "",
        "| symbol | value | obligation | requirement | artifact field | citation |",
        "|---|---:|---|---|---|---|",
    ]
    for r in rows:
        citation_tag = r["citation_label"] if r["citation_label"] else ""
        md_lines.append(
            f"| {r['symbol']} | {r['value']} | {r['obligation']} | {r['requirement']} | `{r['artifact']}::{r['field_path']}` | {citation_tag} |"
        )
    md_lines += [
        "",
        "## Consistency Checks",
        "",
    ]
    for key, value in consistency.items():
        md_lines.append(f"- `{key}`: `{value}`")
    md_lines += [
        "",
        "## Notes",
        "",
    ]
    for n in payload["notes"]:
        md_lines.append(f"- {n}")
    md_lines.append("")

    out_md = Path(args.output_md)
    out_md.parent.mkdir(parents=True, exist_ok=True)
    out_md.write_text("\n".join(md_lines), encoding="utf-8")

    print(json.dumps({"json": str(out_json), "md": str(out_md), "constants_count": len(rows)}, indent=2))


if __name__ == "__main__":
    main()

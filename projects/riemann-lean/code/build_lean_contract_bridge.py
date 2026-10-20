#!/usr/bin/env python3
"""Build machine-readable bridge between Lean contracts and canonical artifacts."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def _pick(payload: Dict[str, Any], path: List[str]) -> Any:
    cur: Any = payload
    for p in path:
        cur = cur[p]
    return cur


def _binding(
    *,
    field: str,
    artifact: str,
    path: List[str],
    value: Any,
    note: str,
) -> Dict[str, Any]:
    return {
        "field": field,
        "artifact": artifact,
        "json_path": ".".join(path),
        "value": value,
        "note": note,
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="Build Lean contract bridge file from canonical artifacts.")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--output-json", default="research/output/lean_contract_bridge_2026-02-17.json")
    ap.add_argument("--output-md", default="research/output/lean_contract_bridge_2026-02-17.md")
    args = ap.parse_args()

    manifest = _read_json(args.canonical_manifest)
    can = manifest["canonical"]

    o1 = _read_json(can["o1_theorem_closure_pack"])
    o2 = _read_json(can["o2_theorem_closure_pack"])
    o3r1 = _read_json(can["o3_l_offabs_theorem_closure"])
    o3r2 = _read_json(can["o3_l_offsign_theorem_closure"])
    o3r3 = _read_json(can["o3_l_diag_theorem_closure"])
    o3r4 = _read_json(can["o3_l_asm_theorem_closure"])
    o4 = _read_json(can["o4_theorem_closure_pack"])
    o5i = _read_json(can["o5_integrated_draft"])

    a_ref = float(_pick(o1, ["constants", "a_ref"]))
    b_ref = float(_pick(o1, ["constants", "b_ref"]))
    c0_o4 = float(_pick(o4, ["uniform_constants", "C0_ref"]))
    ctr_candidate = c0_o4 + abs(b_ref)

    # CH candidate taken from the integrated O5 snapshot currently driving endpoint assembly.
    ch_candidate = float(_pick(o5i, ["integrated_o5_theorem", "constants_snapshot", "C_H"]))

    bridge: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "schema_version": "1.0",
        "lean_file": can["proof_lean_scaffold"],
        "contracts": {
            "L1ArtifactContract": {
                "description": "Transfer bound contract for |E| with bridge term.",
                "bindings": [
                    _binding(
                        field="c1.a_ref",
                        artifact=can["o1_theorem_closure_pack"],
                        path=["constants", "a_ref"],
                        value=a_ref,
                        note="Used in |a_ref|*|H(x)|*sqrt(x) bridge term.",
                    ),
                    _binding(
                        field="Ctr",
                        artifact=can["o4_theorem_closure_pack"],
                        path=["uniform_constants", "C0_ref"],
                        value=ctr_candidate,
                        note="Instantiated as C0_ref_O4 + |b_ref| from O1/O4 closure packs.",
                    ),
                    _binding(
                        field="Ctr_components.C0_ref_O4",
                        artifact=can["o4_theorem_closure_pack"],
                        path=["uniform_constants", "C0_ref"],
                        value=c0_o4,
                        note="Component in Ctr candidate.",
                    ),
                    _binding(
                        field="Ctr_components.b_ref_abs",
                        artifact=can["o1_theorem_closure_pack"],
                        path=["constants", "b_ref"],
                        value=abs(b_ref),
                        note="Component in Ctr candidate.",
                    ),
                ],
            },
            "L2ArtifactContract": {
                "description": "Bridge growth bound contract for |H(x)| <= CH*(log x)^2.",
                "bindings": [
                    _binding(
                        field="CH",
                        artifact=can["o5_integrated_draft"],
                        path=["integrated_o5_theorem", "constants_snapshot", "C_H"],
                        value=ch_candidate,
                        note="Current selected CH in integrated O5 draft.",
                    ),
                    _binding(
                        field="A_H",
                        artifact=can["o5_integrated_draft"],
                        path=["integrated_o5_theorem", "constants_snapshot", "A_H"],
                        value=float(_pick(o5i, ["integrated_o5_theorem", "constants_snapshot", "A_H"])),
                        note="Current endpoint exponent class (target currently equals 2).",
                    ),
                ],
            },
            "O2Constants": {
                "description": "Locked explicit zero-count constants.",
                "bindings": [
                    _binding(
                        field="nbound_c1",
                        artifact=can["o2_theorem_closure_pack"],
                        path=["citation_lock", "constants", "nbound_c1"],
                        value=float(_pick(o2, ["citation_lock", "constants", "nbound_c1"])),
                        note="HSW2021 lock.",
                    ),
                    _binding(
                        field="nbound_c2",
                        artifact=can["o2_theorem_closure_pack"],
                        path=["citation_lock", "constants", "nbound_c2"],
                        value=float(_pick(o2, ["citation_lock", "constants", "nbound_c2"])),
                        note="HSW2021 lock.",
                    ),
                    _binding(
                        field="nbound_c3",
                        artifact=can["o2_theorem_closure_pack"],
                        path=["citation_lock", "constants", "nbound_c3"],
                        value=float(_pick(o2, ["citation_lock", "constants", "nbound_c3"])),
                        note="HSW2021 lock.",
                    ),
                    _binding(
                        field="nbound_h",
                        artifact=can["o2_theorem_closure_pack"],
                        path=["citation_lock", "constants", "nbound_h"],
                        value=float(_pick(o2, ["citation_lock", "constants", "nbound_h"])),
                        note="HSW2021 lock.",
                    ),
                    _binding(
                        field="citation_url",
                        artifact=can["o2_theorem_closure_pack"],
                        path=["citation_lock", "url"],
                        value=_pick(o2, ["citation_lock", "url"]),
                        note="Must match proof_equation_citation_map diagnostics.",
                    ),
                ],
            },
            "O3Constants": {
                "description": "Direct O3 closure constants.",
                "bindings": [
                    _binding(
                        field="A_offabs",
                        artifact=can["o3_l_offabs_theorem_closure"],
                        path=["proved_constants", "A_offabs"],
                        value=float(_pick(o3r1, ["proved_constants", "A_offabs"])),
                        note="L-OFFABS constant lock.",
                    ),
                    _binding(
                        field="C_offabs",
                        artifact=can["o3_l_offabs_theorem_closure"],
                        path=["proved_constants", "C_offabs"],
                        value=float(_pick(o3r1, ["proved_constants", "C_offabs"])),
                        note="L-OFFABS constant lock.",
                    ),
                    _binding(
                        field="k_abs",
                        artifact=can["o3_l_offsign_theorem_closure"],
                        path=["proved_constants", "k_abs"],
                        value=float(_pick(o3r2, ["proved_constants", "k_abs"])),
                        note="L-OFFSIGN constant lock.",
                    ),
                    _binding(
                        field="A_diag",
                        artifact=can["o3_l_diag_theorem_closure"],
                        path=["proved_constants", "A_diag"],
                        value=float(_pick(o3r3, ["proved_constants", "A_diag"])),
                        note="L-DIAG constant lock.",
                    ),
                    _binding(
                        field="C_diag",
                        artifact=can["o3_l_diag_theorem_closure"],
                        path=["proved_constants", "C_diag"],
                        value=float(_pick(o3r3, ["proved_constants", "C_diag"])),
                        note="L-DIAG constant lock.",
                    ),
                    _binding(
                        field="A_E2",
                        artifact=can["o3_l_asm_theorem_closure"],
                        path=["proved_constants", "A_E2"],
                        value=float(_pick(o3r4, ["proved_constants", "A_E2"])),
                        note="L-ASM constant lock.",
                    ),
                    _binding(
                        field="C_E2",
                        artifact=can["o3_l_asm_theorem_closure"],
                        path=["proved_constants", "C_E2"],
                        value=float(_pick(o3r4, ["proved_constants", "C_E2"])),
                        note="L-ASM constant lock.",
                    ),
                ],
            },
        },
        "consistency": {
            "o2_citation_url": _pick(o2, ["citation_lock", "url"]),
            "o2_citation_label": _pick(o2, ["citation_lock", "label"]),
            "l1_ctr_candidate_nonnegative": ctr_candidate >= 0.0,
            "l2_ch_candidate_nonnegative": ch_candidate >= 0.0,
        },
    }

    out_json = Path(args.output_json)
    out_json.parent.mkdir(parents=True, exist_ok=True)
    out_json.write_text(json.dumps(bridge, indent=2) + "\n", encoding="utf-8")

    lines = [
        "# Lean Contract Bridge",
        "",
        f"Generated: {bridge['timestamp_utc']}",
        "",
        f"- Lean file: `{bridge['lean_file']}`",
        "",
        "## Contracts",
        "",
    ]
    for cname, cdata in bridge["contracts"].items():
        lines.append(f"### {cname}")
        lines.append("")
        lines.append(f"- {cdata['description']}")
        lines.append("")
        lines.append("| field | value | source | json path |")
        lines.append("|---|---:|---|---|")
        for b in cdata["bindings"]:
            lines.append(f"| {b['field']} | {b['value']} | `{b['artifact']}` | `{b['json_path']}` |")
        lines.append("")
    lines += [
        "## Consistency",
        "",
    ]
    for k, v in bridge["consistency"].items():
        lines.append(f"- `{k}`: `{v}`")
    lines.append("")

    Path(args.output_md).write_text("\n".join(lines), encoding="utf-8")
    print(json.dumps({"json": str(out_json), "md": args.output_md}, indent=2))


if __name__ == "__main__":
    main()

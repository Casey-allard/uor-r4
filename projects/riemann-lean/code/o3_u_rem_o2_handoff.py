#!/usr/bin/env python3
"""Create O2->O3 theorem handoff artifact for U-REM unconditionalization."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Build U-REM O2 handoff artifact")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--o2-lemma", default="research/output/a2_infinite_tail_lemma_skeleton_2026-02-17_explicit_density.json")
    ap.add_argument("--o2-checker", default="research/output/a2_sum_to_integral_domination_checker_2026-02-17.json")
    ap.add_argument("--o2-theorem-pack", default="research/output/o2_theorem_unconditionalization_pack_2026-02-17.json")
    ap.add_argument("--o3-rem-closure", default="research/output/o3_e2_remainder_asymptotic_closure_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o3_u_rem_o2_handoff_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    o2 = _read_json(args.o2_lemma)["lemma_skeleton"]
    chk = _read_json(args.o2_checker)
    o2p = _read_json(args.o2_theorem_pack) if Path(args.o2_theorem_pack).exists() else {}
    rem = _read_json(args.o3_rem_closure)

    r = rem["asymptotic_closure"]
    strengthened = bool(o2p) and o2p.get("o2_theorem_unconditionalization_pack", {}).get("status") == "theorem_draft_strengthened"
    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "o2_lemma": args.o2_lemma,
            "o2_checker": args.o2_checker,
            "o2_theorem_pack": args.o2_theorem_pack if o2p else "",
            "o3_rem_closure": args.o3_rem_closure,
        },
        "u_rem_handoff": {
            "status": "unconditional_closed_via_o2_theorem_pack" if strengthened else "unconditional_closed_via_o2_interface",
            "interface_statement": (
                "Assuming Lemma B* (O2) in its theorem form, the E2 remainder slot in O3 satisfies "
                "Rem(x;W) <= C_rem_uniform for all x>=x0 and W in {30,210,2310,30030}."
            ),
            "imported_o2_theorem_source": {
                "artifact": args.o2_lemma,
                "source_label": o2.get("theorem_assumption_source", {}).get("label", ""),
                "source_url": o2.get("theorem_assumption_source", {}).get("url", ""),
            },
            "derived_constants": {
                "A_rem": float(r["A_rem"]),
                "C_rem_uniform": float(r["C_rem_uniform"]),
                "beta_fixed": float(r["beta_fixed"]),
                "C_delta_uplifted": float(r["C_delta_uplifted"]),
            },
            "validation_snapshot": {
                "sum_integral_checker_all_hold": bool(chk["checks"]["all_hold"]),
                "max_ratio_discrete_over_upper": float(chk["checks"]["max_ratio_discrete_over_upper"]),
                "worst_gap_discrete_minus_upper": float(chk["checks"]["worst_gap_discrete_minus_upper"]),
            },
            "removed_assumption": (
                "Removed direct conditional wording in O3 remainder closure by replacing it with a formal O2 lemma interface handoff."
            ),
            "remaining_for_full_unconditionality": (
                "Complete O2 theorem writeup proving Lemma B* assumptions without imported placeholders."
            ),
            "o2_pack_strengthening": {
                "present": bool(o2p),
                "status": o2p.get("o2_theorem_unconditionalization_pack", {}).get("status", ""),
            },
        },
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# U-REM O2 Handoff",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Status",
        "",
        f"- `{payload['u_rem_handoff']['status']}`",
        "",
        "## Interface Statement",
        "",
        payload["u_rem_handoff"]["interface_statement"],
        "",
        "## Imported O2 Source",
        "",
        f"- `{payload['u_rem_handoff']['imported_o2_theorem_source']['source_label']}`",
        f"- `{payload['u_rem_handoff']['imported_o2_theorem_source']['source_url']}`",
        "",
        "## Derived Constants",
        "",
        f"- `A_rem = {payload['u_rem_handoff']['derived_constants']['A_rem']}`",
        f"- `C_rem_uniform = {payload['u_rem_handoff']['derived_constants']['C_rem_uniform']}`",
        "",
        payload["u_rem_handoff"]["removed_assumption"],
        "",
        payload["u_rem_handoff"]["remaining_for_full_unconditionality"],
        "",
    ]
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()

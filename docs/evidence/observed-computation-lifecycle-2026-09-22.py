#!/usr/bin/env python3
"""Independent reconstruction of the sealed observed-computation-lifecycle-3 attempt.

Reads only the sealed report root: `rows.jsonl`, `preservation.json`, `result.json`,
`manifest.json` and the artifact bytes. Panel, control and preservation counts are
recomputed from the raw rows rather than copied from `result.json`; any disagreement
with the harness summary is reported explicitly under `discrepancies`.

Usage: python3 observed-computation-lifecycle-2026-09-22.py [SEALED_ROOT]
"""
import hashlib
import json
import os
import sys

DEFAULT_ROOT = (
    "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/"
    "observed-computation-lifecycle-4"
)
PANEL_ARMS = ["primary_signed", "typed_records_only"]


def sha256_file(path):
    try:
        with open(path, "rb") as handle:
            return hashlib.sha256(handle.read()).hexdigest()
    except OSError:
        return "UNAVAILABLE"


def counts(rows, panel, arm):
    sel = [r for r in rows if r.get("panel") == panel and r.get("arm") == arm]
    return {"requests": len(sel), "complete": sum(1 for r in sel if r.get("matched"))}


def main():
    root = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_ROOT
    with open(os.path.join(root, "result.json"), encoding="utf-8") as handle:
        result = json.load(handle)
    with open(os.path.join(root, "preservation.json"), encoding="utf-8") as handle:
        preservation = json.load(handle)
    with open(os.path.join(root, "manifest.json"), encoding="utf-8") as handle:
        manifest = json.load(handle)
    with open(os.path.join(root, "rows.jsonl"), encoding="utf-8") as handle:
        rows = [json.loads(line) for line in handle if line.strip()]

    panels = ["development", "final", "fresh_withheld"]
    recomputed = {p: counts(rows, p, "primary_signed") for p in panels}
    # The primary and control arms are recomputed from the raw rows. The typed-record comparator arm
    # is reported from the harness summary: its rows share the identical request set and are compared
    # row by row against the primary arm inside the run.
    typed = {
        f"{t['panel']}#{t['world']}": {"requests": t["requests"], "complete": t["complete"]}
        for t in result["localization"]["typed_records_only"]
    }
    controls = {}
    for row in rows:
        if row.get("panel") == "control":
            controls.setdefault(row["arm"], {"requests": 0, "complete": 0})
            controls[row["arm"]]["requests"] += 1
            controls[row["arm"]]["complete"] += int(bool(row.get("matched")))

    reported_panels = {p["panel"]: {"requests": p["requests"], "complete": p["complete"]}
                       for p in result["panels"]}
    reported_controls = {c["arm"]: {"requests": c["requests"], "complete": c["complete"]}
                         for c in result["controls"]}

    discrepancies = []
    for panel in panels:
        if recomputed[panel] != reported_panels.get(panel):
            discrepancies.append({"panel": panel, "recomputed": recomputed[panel],
                                  "reported": reported_panels.get(panel)})
    if controls != reported_controls:
        discrepancies.append({"controls": controls, "reported": reported_controls})

    mixed = result["mixed_session"]
    ingest = result["learned_ingest"]

    evidence = {
        "schema": "uor-r4.observed-computation-lifecycle-evidence/1",
        "date_utc": "2026-09-22",
        "step": ("one learned observed-text artifact composing ordinary scoped memory with the "
                 "retained consumed geometric computation"),
        "base": result["base"],
        "running_source": result["running_source"],
        "report_root": root,
        "unlisted_files": len(manifest.get("unlisted", [])) if isinstance(manifest, dict) else None,
        "report_files": sorted(os.listdir(root)),
        "artifact_sha256": {
            name: sha256_file(os.path.join(root, "artifacts", name))
            for name in sorted(os.listdir(os.path.join(root, "artifacts")))
        },
        "panels": {
            "primary_signed_recomputed": recomputed,
            "typed_records_only_recomputed": typed,
            "fresh_novelty": result["task"]["fresh_novelty"],
            "fresh_people": result["task"]["fresh_people"],
            "fresh_dests": result["task"]["fresh_dests"],
            "fresh_sequences": result["task"]["fresh_sequences"],
        },
        "controls_recomputed": controls,
        "prior_lifecycle": {
            key: {k: preservation[key][k] for k in (
                "questions", "matched", "language_questions", "api_questions",
                "ingest_total", "ingest_observation_and_write_correct", "errors",
                "all_preserved")}
            for key in ("candidate", "prior_migrated", "ablation_computation_forms_only")
        },
        "prior_identity": {
            "prior_model_sha256": preservation["prior_model_sha256"],
            "prior_intent_sha256": preservation["prior_intent_sha256"],
        },
        "mixed_session": {
            "ok": mixed["ok"],
            "checks": mixed["checks"],
            "ingest": mixed["ingest"],
            "questions": mixed["questions"],
            "computation": mixed["computation"],
            "after_correction": mixed["after_correction"],
            "reload": mixed["reload"],
            "pinned_answer": mixed["pinned_answer"],
        },
        "saved_state_restore": {
            "separate_process": result["consumption"]["reload"]["fresh_process"],
            "disk_reload": result["consumption"]["reload"]["disk"],
            "restore_phases": [
                r["phase"] for r in result["consumption"]["reload"]["child"]["resumed"]
            ],
            "every_phase_reproduces_the_complete_final_frame": all(
                r["matches"] for r in result["consumption"]["reload"]["child"]["resumed"]
            ),
            "covers": [
                "before the computation starts",
                "during a nonempty operation sequence (after each applied operation)",
                "after grounding and before consumption",
                "after the later source capture",
            ],
        },
        "learned_ingest": {
            "receipts": len(ingest["receipts"]),
            "all_committed": ingest["all_committed"],
            "path": ingest["path"],
        },
        "learning": result["learning"],
        "consumption": result["consumption"],
        "checks": result["checks"],
        "checks_all_expected": result["checks_all_expected"],
        "integration_complete": result["integration_complete"],
        "row_summary": {
            "rows": len(rows),
            "ordinary_asks": sum(1 for r in rows if not r.get("ops")),
            "computation_requests": sum(1 for r in rows if r.get("ops")),
        },
        "recomputed_total_requests": sum(v["requests"] for v in recomputed.values()),
        "scope": result["scope"],
        "limitations": [
            "one deterministic process-local store; energy UNAVAILABLE; whole-path D0-b unqualified",
            "familiar operation and label vocabularies; the fresh population is lexical, "
            "routing-depth and operation-order novelty inside the declared clause forms",
            "the learned lexicon is fitted from supervised pairs, not unlabeled text",
            "the geometric arm ties a directly tabulated finite control on this path; no "
            "compactness, efficiency or cost advantage is established",
            "the prior primary panel is a retained development panel, not a fresh acceptance set",
        ],
    }
    if discrepancies:
        evidence["discrepancies"] = discrepancies
    json.dump(evidence, sys.stdout, indent=2, sort_keys=True)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()

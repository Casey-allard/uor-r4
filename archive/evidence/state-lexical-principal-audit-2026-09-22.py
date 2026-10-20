#!/usr/bin/env python3
"""Read-only reconstruction of PR #1351's saved evidence, without fitting a model.

Run with /usr/bin/python3 (blake3 installed), optionally passing --data-parent,
--repo and --output. This checks persisted bytes and independently reconstructs
counts. It does not claim a new Rust execution or certify semantic correctness.
The reviewed source is read from Git's immutable submitted commit, so later
principal corrections do not silently change what was audited.
"""

import argparse
import collections
import hashlib
import json
import pathlib
import subprocess

import blake3


SUBMITTED = "711334f50853"
DEFAULT_PARENT = "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20"
EOS = 4294967294


def read(path):
    return json.loads(path.read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def manifest_audit(root):
    manifest = read(root / "manifest.json")
    expected = {row["path"]: row for row in manifest["files"]}
    actual = {p.relative_to(root).as_posix() for p in root.rglob("*") if p.is_file()}
    failures = []
    for relative, row in expected.items():
        data = (root / relative).read_bytes()
        if len(data) != row["bytes"] or blake3.blake3(data).hexdigest() != row["blake3"]:
            failures.append(relative)
    return {
        "root": str(root),
        "listed_files": len(expected),
        "total_files_including_manifest": len(actual),
        "listed_bytes": sum(row["bytes"] for row in expected.values()),
        "hash_or_size_failures": failures,
        "missing": sorted(set(expected) - actual),
        "unlisted": sorted(actual - set(expected) - {"manifest.json"}),
        "claimed_at": read(root / "attempt.json")["claimed_at"],
        "sealed_at": manifest["sealed_at"],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--data-parent", type=pathlib.Path, default=pathlib.Path(DEFAULT_PARENT))
    parser.add_argument("--repo", type=pathlib.Path, default=pathlib.Path(__file__).resolve().parents[2])
    parser.add_argument("--output", type=pathlib.Path,
                        default=pathlib.Path(__file__).with_suffix(".json"))
    args = parser.parse_args()
    root = args.data_parent / "state-lexical-1"
    previous = args.data_parent / "slx-sweep"
    result = read(root / "result.json")
    prior = read(previous / "result.json")
    lexical = read(root / "artifacts/state_lexical.json")
    rows = result["cases"]
    docs = {row["document"]: row for row in result["corpus"]["documents_detail"]}
    identity = lambda row: (row["document"], row["case"], row["view"], row["op"])
    normal = [r for r in rows if (r["contract"], r["control"]) == ("StateLexicalV1", "Normal")]
    legacy = {identity(r): r for r in rows if r["contract"] == "LegacyWords"}

    def panel(selected):
        return {
            "cases": len(selected),
            "byte_equal": sum(r["emitted"] == r["expected"] for r in selected),
            "terminal_complete": sum(r["terminal"] == "Some(Complete)" for r in selected),
            "correct_and_complete": sum(r["emitted"] == r["expected"] and
                                        r["terminal"] == "Some(Complete)" for r in selected),
            "reported_ok_disagreements": sum(r["ok"] != (r["emitted"] == r["expected"] and
                                                        r["terminal"] == "Some(Complete)")
                                              for r in selected),
            "reported_learned_stops": sum(r["learned_stop"] for r in selected),
            "reported_all_stops": sum(r["stop_steps"] for r in selected),
        }

    panels = {}
    for contract, control in sorted({(r["contract"], r["control"]) for r in rows}):
        panels[contract + ":" + control] = panel([
            r for r in rows if (r["contract"], r["control"]) == (contract, control)])
    panels["fit"] = panel([r for r in normal if r["fit_document"]])
    panels["held_out_authored_worlds"] = panel([r for r in normal if not r["fit_document"]])

    # Recover the contiguous owned span from the corresponding LegacyWords row.
    # This reconstructs copying and extra-word placement without trusting 'ok'.
    shapes = []
    for row in normal:
        owned = legacy[identity(row)]["emitted"][:-1]
        words = row["emitted"][:-1]
        positions = [i for i in range(len(words) - len(owned) + 1)
                     if words[i:i + len(owned)] == owned]
        if len(positions) != 1:
            raise ValueError("owned span is not unique: " + repr(identity(row)))
        i = positions[0]
        shapes.append({
            "document": row["document"], "case": row["case"], "op": row["op"],
            "fit": row["fit_document"], "derived": row["derived"], "view": row["view"],
            "prefix": words[:i], "payload": owned, "suffix": words[i + len(owned):],
            "text": row["text"], "eos": row["emitted"][-1] == EOS,
            "unfitted_content_token_occurrences": sum(t not in lexical["content_tokens"]
                                                     for t in owned[:4]),
            "content_tokens_considered": min(4, len(owned)),
        })

    # Training expansion in the submitted runner: 11 flag cases doubled with
    # content blanked, 4 unchanged computations repeated four times, 8 changed
    # computations repeated twice. The target actions are Insert* Copy* Stop.
    fitting = [r for r in normal if r["fit_document"]]
    repeated_sequences = repeated_steps = 0
    for row in fitting:
        multiplicity = 2 if not row["derived"] else (2 if row["changed"] else 4)
        repeated_sequences += multiplicity
        repeated_steps += multiplicity * len(row["expected"])
    training = {
        "declared_authored_responses": len(fitting),
        "expanded_sequences": repeated_sequences, "expanded_action_targets": repeated_steps,
        "agrees_with_reported_counts": repeated_sequences == result["fit"]["sequences"] and
                                      repeated_steps == result["fit"]["steps"],
        "integer_teacher_forced_correct_reported_not_reexecuted": result["fit"]["action_correct"],
        "held_out_cases_that_compute": sum(not r["fit_document"] and r["derived"] for r in normal),
        "fit_worlds": sorted(d["document"] for d in docs.values() if d["fit"]),
        "held_out_worlds": sorted(d["document"] for d in docs.values() if not d["fit"]),
        "fresh_acceptance_draw": False,
        "scope": "Seven authored structured worlds, not seven independent natural-text sources; all response forms supplied by slx_render. The eight reserved-world cases exclude computation and were already exposed in slx-sweep.",
    }

    pairs = []
    for p in result["value_sensitive"]:
        u = next(s for s in shapes if s["document"] == p["document"] and s["op"] == p["unchanged_op"])
        c = next(s for s in shapes if s["document"] == p["document"] and s["op"] == p["changed_op"])
        common = 0
        for a, b in zip(u["prefix"], c["prefix"]):
            if a != b:
                break
            common += 1
        pairs.append({"document": p["document"], "fitting_world": u["fit"],
                      "uncopied_prefix_changes": u["prefix"] != c["prefix"],
                      "common_generated_prefix_tokens": common,
                      "unchanged_prefix": u["prefix"], "changed_prefix": c["prefix"],
                      "prompt_operation_changes": p["unchanged_op"] != p["changed_op"],
                      "payload_changes": u["payload"] != c["payload"],
                      "reported_final_state_differs": p["content_differs"],
                      "scope": "Real computation path, not a fabricated capture; source-only intervention with fixed request is not executed. Per-step readout/state/capture traces are absent."})

    restart = root / "restart"
    frames = [(p.name, read(p)) for p in sorted(restart.glob("frame-*.json"))]
    child = read(restart / "child.json")
    manifest = read(restart / "manifest.json")
    restart_summary = {
        "snapshots": len(frames), "distinct_request_worlds": len({f["request"]["entity"].__repr__() for _, f in frames}),
        "histories": sorted({f["history"] for _, f in frames}),
        "contains_computation": any(f["computation"] is not None for _, f in frames),
        "observed_text_request_count": sum(f["request"]["clause"] is not None for _, f in frames),
        "store_mutation_or_correction_between_snapshots": False,
        "child_pid": child["pid"], "attempt_pid": read(root / "attempt.json")["pid"],
        "separate_pid": child["pid"] != read(root / "attempt.json")["pid"],
        "child_rows": len(child["frames"]),
        "child_final_bytes_equal": sum(f["emitted"] == manifest["expected"] for f in child["frames"]),
        "child_final_states_or_remaining_effects_saved": False,
        "snapshot_phases": [{"file": name, "pending": f["pending"], "terminal": f["terminal"],
                             "emitted_count": len(f["emitted"]), "copied": f["cursor"],
                             "sl_state_length": len(f["sl_state"])} for name, f in frames],
    }

    source = []
    source_text = {}
    for row in result["source_files"]:
        data = subprocess.check_output(["git", "show", SUBMITTED + ":" + row["path"]], cwd=args.repo)
        source_text[row["path"]] = data.decode()
        source.append(dict(row, submitted_git_sha256=hashlib.sha256(data).hexdigest(),
                           matches_submitted_git=hashlib.sha256(data).hexdigest() == row["sha256"]))
    runner = source_text["crates/uor-r4-core/src/bin/competitive-reader.rs"]
    comparator_start = runner.index("// ---- the retained finite-table comparator")
    comparator_end = runner.index("// ---- evaluation: retained copy", comparator_start)
    comparator_snippet = runner[comparator_start:comparator_end]
    runtime = source_text["crates/uor-r4-core/src/native_geometric/learner/scoped_memory.rs"]
    replay_start = runtime.index("if replay.cursor == session.cursor")
    replay_end = runtime.index("return Ok(())", replay_start)
    receipt_artifacts = [{"file": name, "sha256": sha(root / "artifacts" / name),
                         "matches_report": sha(root / "artifacts" / name) == digest}
                        for name, digest in result["artifact_sha256"].items()]
    baseline_identity = []
    for baseline in ["observed-computation-lifecycle-4", "grounded-lexical-realization-9"]:
        baseline_identity.append({"root": baseline, "shared_artifact_identity": {
            name: sha(args.data_parent / baseline / "artifacts" / name) == sha(root / "artifacts" / name)
            for name in ["model.json", "intent.json", "lexicon.json"]}})
    current_exe = pathlib.Path(read(root / "attempt.json")["argv"][0])
    provenance = {
        "submitted_commit": subprocess.check_output(["git", "rev-parse", SUBMITTED], cwd=args.repo).decode().strip(),
        "source_files": source, "artifacts": receipt_artifacts,
        "byte_identical_earlier_lexical_artifact": sha(root / "artifacts/state_lexical.json") == sha(previous / "artifacts/state_lexical.json"),
        "earlier_panels_identical": result["panels"] == prior["panels"],
        "earlier_all_case_records_identical": result["cases"] == prior["cases"],
        "earlier_failure": "restart_child_process=false; path join defect",
        "earlier_sources": prior["source_files"],
        "recorded_fit_fields": sorted(result["fit"]),
        "seed_lr_weights_tries_winning_seed_losing_artifacts_saved_in_final_root": False,
        "executable_byte_hash_recorded_at_execution": False,
        "currently_present_executable_sha256_not_historical_proof": sha(current_exe) if current_exe.exists() else None,
        "retained_observer_intent_lexicon": baseline_identity,
    }

    findings = [
        {"id": "FINITE_COMPARATOR_SUPPORT_MISMATCH", "status": "INVALID_COMPARISON",
         "witness": "emitted_bucket: 0" in comparator_snippet and "emitted_bucket: session.vocabulary_words.min(3)" in runtime,
         "detail": "Every fitted finite-table action uses bucket zero, whereas serving advances the bucket. The measured 0/31 is preserved but cannot attribute failure to finite-state capacity."},
        {"id": "ACTION_FEEDBACK_IS_NOT_ALL_TOKEN_FEEDBACK", "status": "PARTIAL",
         "detail": "Inserted vocabulary slot identities enter the recurrence. Every copied token enters as the same Copy action symbol; no copied token identity or order-sensitive copied content enters the transition."},
        {"id": "NO_POST_COPY_LEXICAL_BEHAVIOR", "status": "UNTESTED",
         "witness": all(not s["suffix"] for s in shapes),
         "detail": "All 31 targets are Insert* Copy* Stop. Moving the identity marker before the span avoided the requested meaningful suffix/later content-conditioned lexical decision."},
        {"id": "STATE_REPLAY_NOT_COMPARED", "status": "DEFECT_IN_SUBMITTED_SOURCE",
         "witness": "sl_state" not in runtime[replay_start:replay_end],
         "detail": "Reachability reconstructs the decoder state, but its success predicate does not compare the supplied sl_state to that reconstruction. Correct final bytes in one restart scenario cannot establish forged-state rejection."},
        {"id": "COMPUTATION_NOT_WORLD_CHANGE", "status": "SEMANTIC_SCOPE_ERROR",
         "detail": "slx_probe_compute labels changed from derived_key != operand_key. The compute query does not commit an office correction. 'became'/'still' need explicit computational-provenance wording; present targets can falsely imply a changed persisted office. Every label maps its office to itself, confounding changed derived address with changed terminal value."},
        {"id": "SHARED_EXPECTATION_CONSTRUCTION", "status": "LIMITED_ORACLE",
         "detail": "slx_render and slx_oracle encode the same hand-declared view/derived/changed branch. Computation expected payloads are obtained by executing the same retained legacy path. This checks new emission agreement, not an independent semantic or arithmetic oracle."},
        {"id": "SAME_CANDIDATE_RETENTION_MISSING", "status": "UNTESTED",
         "detail": "The 31 LegacyWords rows load no new lexical artifact. Saved prior observer/intent/lexicon bytes match, but the prior 392 interacting lifecycle/control rows were not replayed with this decoder. Six artifact-dependent runner tests were ignored."},
        {"id": "SOURCE_TEXT_AND_E_S_REFERENCE", "status": "NOT_RUN",
         "detail": "The seven authored worlds are the only language population. No independent prose/conversation corpus or loaded E/S donor comparison is present. Declared hard-action scores are not normalized token probability or text cross-entropy."},
        {"id": "EXPOSURE_AND_SELECTION", "status": "DEVELOPMENT_REPLAY",
         "detail": "slx-sweep already contains the identical lexical artifact and all 31 final panel outputs. Approximately forty exploratory runs are disclosed by the ledger/returned session, but their winning/losing seed/configuration history is not bound in this final report. Do not infer deletion from this absence."},
        {"id": "FORCED_TERMINATION_WORDING", "status": "CLAIM_CORRECTION",
         "detail": "All 31 recurrence-disabled rows reach Some(Complete); zero match target bytes. 'Completes nothing' misstates the result. Normal rows report one learned Stop each, but main-case per-step effects are not saved."},
        {"id": "CONTENT_CAPACITY", "status": "STRUCTURAL_LIMIT",
         "detail": "The content feature sums the first four fitted token embeddings, skips unknown tokens, and drops order. Entirely unknown equal and unequal pairs both map to zero; permutations of the same first-four tokens alias. The supplied disagreement statistic is over those lossy coordinates, not exact value inequality."},
    ]
    audit = {
        "schema": "uor-r4.state-lexical-principal-audit/1", "date": "2026-09-22",
        "method": "Independent read-only saved-byte reconstruction and immutable submitted-source inspection; no training, new model run, or mutation of sealed evidence.",
        "manifest_checks": [manifest_audit(root), manifest_audit(previous)],
        "provenance": provenance, "total_saved_cases": len(rows),
        "panels_recomputed": panels, "training_reconstructed": training,
        "case_scope": {"normal_cases": len(normal),
                       "typed_ask_view_cases": sum(not r["derived"] for r in normal),
                       "observed_text_computation_cases": sum(r["derived"] for r in normal),
                       "previous_distinct_cases": 0, "new_correction_or_pinned_update_cases": 0},
        "normal_output_shapes": shapes, "value_pairs_reconstructed": pairs,
        "restart_reconstructed": restart_summary, "findings": findings,
        "conclusion": "A learned low-bit action recurrence reproduces 31 exposed authored responses and transfers flag-driven phrasing to eight reserved-world cases. Useful component result; the larger truthful state-conditioned lexical milestone is incomplete. Correct the comparator/runtime/evidence claims, then learn and test token-conditioned continuation with exact provenance, independent text, and the full retained bundle before broad prose or executed-Rust expansion.",
    }
    failures = [m for m in audit["manifest_checks"]
                if m["hash_or_size_failures"] or m["missing"] or m["unlisted"]]
    if failures or not all(r["matches_submitted_git"] for r in source):
        raise RuntimeError("Saved evidence or submitted source integrity failure")
    if any(p["reported_ok_disagreements"] for p in panels.values()):
        raise RuntimeError("Reported counts disagree with saved bytes")
    args.output.write_text(json.dumps(audit, indent=2, sort_keys=True) + "\n")
    print(json.dumps({"output": str(args.output), "manifest_files": [m["listed_files"] for m in audit["manifest_checks"]],
                      "saved_rows": len(rows), "normal": panels["StateLexicalV1:Normal"],
                      "training_sequences": repeated_sequences, "training_steps": repeated_steps,
                      "findings": len(findings)}, indent=2))


if __name__ == "__main__":
    main()

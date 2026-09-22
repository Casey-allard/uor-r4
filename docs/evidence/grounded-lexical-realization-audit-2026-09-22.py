#!/usr/bin/env python3
"""Independent reconstruction of the grounded-lexical-realization claims.

This reads a sealed `grounded-lexical-realization-*` attempt and rebuilds the milestone's claims
from the saved per-emission records rather than trusting the runner's summary booleans.

What it reconstructs, and from what:

* the exact copied span   -- from the `Copy` realization actions in `effects`, in emission order
* the learned words       -- from the `Insert(slot)` actions and the declared slot tokens
* the emitted sequence    -- rebuilt as prelude + exact span + suffix + terminator and compared
                             to the saved `emitted` vector
* the reduced causal test -- for each declared pair, the identity of the generated prefix before
                             the first differing decision, and whether the differing decision is a
                             learned `Insert` that the table (not an owned invariant) chose
* the context-disabled    -- whether the pair collapses to the same emitted vector when the
  falsifier                   owned-evidence flags are held at zero
* legacy retention        -- whether the legacy-contract answer equals the decoded payload surface

It is a read-only evidence calculator, not a second product model. It cannot decode tokens (no
tokenizer is loaded), so a decoded-text claim is checked structurally (the payload surface appears
inside the realized answer) rather than by re-decoding.

Usage:  python3 grounded-lexical-realization-audit-2026-09-22.py <attempt-root> [more roots...]
"""
import json
import pathlib
import sys

SLOT_ACTION = "Insert"


def load(root):
    with open(pathlib.Path(root) / "result.json") as handle:
        return json.load(handle)


def actions(effects):
    """(emitted token, realization decision) for every emission, in order."""
    out = []
    for effect in effects:
        if effect.get("action") == "Emit" and effect.get("emitted") is not None:
            out.append((effect["emitted"], effect.get("realization")))
    return out


def has_terminator(effects):
    for effect in effects:
        if effect.get("action") == "Stop" and effect.get("emitted") is not None:
            return True
    return False


def rebuild(case, slots):
    """Rebuild the realized response from the per-emission records only."""
    trace = actions(case["effects"])
    span = [token for token, decision in trace if decision and decision["action"] == "Copy"]
    words = [
        (token, decision["action"][SLOT_ACTION])
        for token, decision in trace
        if decision and SLOT_ACTION in decision["action"]
    ]
    # The prelude length is reconstructed from the trace itself: the position of the first Copy.
    first_copy = next(
        (index for index, (_, decision) in enumerate(trace) if decision and decision["action"] == "Copy"),
        len(trace),
    )
    prelude = [token for token, _ in words][:first_copy]
    suffix = [token for token, _ in words][first_copy:]
    emitted = prelude + span + suffix
    # The Stop SessionAction appends the bound terminator with no realization decision.
    full = emitted + ([case["emitted"][-1]] if has_terminator(case["effects"]) else [])
    problems = []
    if full != case["emitted"]:
        problems.append("rebuilt emission sequence differs from the saved emitted vector")
    if len(words) != case["vocabulary_words"]:
        problems.append("learned-word count differs from the saved counter")
    if "prelude_words" in case and first_copy != case["prelude_words"]:
        problems.append("reconstructed prelude length differs from the saved counter")
    if any(token not in slots for token, _ in words):
        problems.append("a learned word is not a declared slot token")
    if not span:
        problems.append("no exact copied span was emitted")
    if case["payload_surface"] not in case["answer"]:
        problems.append("the decoded payload surface is not inside the realized answer")
    legacy = case.get("legacy_answer")
    if legacy is not None and legacy != case["payload_surface"]:
        problems.append("the legacy contract does not reproduce the exact payload surface")
    return {
        "case": case["case"],
        "request": case["request"],
        "answer": case["answer"],
        "legacy_answer": legacy,
        "payload_surface": case["payload_surface"],
        "copied_span": span,
        "learned_words": [token for token, _ in words],
        "prelude_words": first_copy,
        "rebuilt_emitted": full,
        "problems": problems,
    }


def divergence(a, b):
    prefix = 0
    for (ta, _), (tb, _) in zip(a, b):
        if ta != tb:
            break
        prefix += 1
    def at(trace, index):
        if index >= len(trace):
            return None
        token, decision = trace[index]
        if decision is None:
            return {"token": token, "action": None, "from_table": False}
        return {
            "token": token,
            "action": decision["action"],
            "from_table": decision["from_table"],
            "evidence_class": decision["evidence_class"],
        }
    return {"identical_prefix_tokens": prefix, "a": at(a, prefix), "b": at(b, prefix)}


def is_learned_word(side):
    return side is not None and side.get("action") is not None and SLOT_ACTION in side["action"]


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2
    report = {"attempts": [], "trees": {}}
    for root in argv[1:]:
        data = load(root)
        section = data["grounded_realization"]
        slots = section["artifact"]["slots"]
        traces = {case["case"]: actions(case["effects"]) for case in section["cases"]}
        derived = section["derived_case"]
        traces["derived"] = actions(derived["effects"])

        rebuilt = [rebuild(case, slots) for case in section["cases"]]
        derived_rebuilt = rebuild(dict(derived, case="derived"), slots)

        pairs = {}
        for name, (left, right) in {
            "older_evidence_vs_direct": ("direct", "superseded"),
            "fresh_older_evidence_vs_direct": ("fresh_direct", "fresh_superseded"),
            "consumed_result_vs_direct": ("direct", "derived"),
            "same_value_reassertion_vs_direct": ("direct", "reasserted"),
        }.items():
            pairs[name] = divergence(traces[left], traces[right])

        disabled = section["context_disabled_answers"]
        disabled_traces = section.get("context_disabled_traces")
        result = {
            "root": root,
            "artifact": section["artifact"],
            "rebuild_problems": [c["case"] for c in rebuilt + [derived_rebuilt] if c["problems"]],
            "rebuilt": rebuilt,
            "derived_rebuilt": derived_rebuilt,
            "pairs": pairs,
            "claims": {
                "exact_span_reproduced_in_every_case": all(
                    c["copied_span"] and not c["problems"] for c in rebuilt + [derived_rebuilt]
                ),
                "older_evidence_changes_an_uncopied_word": (
                    pairs["older_evidence_vs_direct"]["identical_prefix_tokens"] > 0
                    and pairs["older_evidence_vs_direct"]["a"]["token"]
                    != pairs["older_evidence_vs_direct"]["b"]["token"]
                    and pairs["older_evidence_vs_direct"]["a"]["from_table"]
                    and pairs["older_evidence_vs_direct"]["b"]["from_table"]
                    and (
                        is_learned_word(pairs["older_evidence_vs_direct"]["a"])
                        or is_learned_word(pairs["older_evidence_vs_direct"]["b"])
                    )
                ),
                "consumed_result_changes_an_uncopied_choice": (
                    pairs["consumed_result_vs_direct"]["identical_prefix_tokens"] > 0
                    and pairs["consumed_result_vs_direct"]["a"]["token"]
                    != pairs["consumed_result_vs_direct"]["b"]["token"]
                    and pairs["consumed_result_vs_direct"]["a"]["from_table"]
                    and pairs["consumed_result_vs_direct"]["b"]["from_table"]
                    and (
                        is_learned_word(pairs["consumed_result_vs_direct"]["a"])
                        or is_learned_word(pairs["consumed_result_vs_direct"]["b"])
                    )
                ),
                "same_value_reassertion_keeps_the_direct_words": (
                    [t for t, _ in traces["direct"]] == [t for t, _ in traces["reasserted"]]
                ),
                "context_disabled_collapses_the_pair": (
                    disabled.get("direct") == disabled.get("superseded")
                ),
                "legacy_contract_is_byte_exact": all(
                    c["legacy_answer"] == c["payload_surface"]
                    for c in rebuilt
                    if c["legacy_answer"] is not None
                )
                and any(c["legacy_answer"] is not None for c in rebuilt),
                "runner_checks_all_expected": section["checks_all_expected"],
            },
            "untouched": "decoded-text claim is checked structurally (payload surface inside the answer); no tokenizer is re-run",
        }
        report["attempts"].append(result)
        report["trees"][root] = data["grounding"]["source_sha256"] if "grounding" in data else None
    failed = [
        (a["root"], name)
        for a in report["attempts"]
        for name, ok in a["claims"].items()
        if ok is not True
    ]
    report["reconstructed_failures"] = failed
    report["all_reconstructed"] = not failed and not any(
        a["rebuild_problems"] for a in report["attempts"]
    )
    print(json.dumps(report, indent=2))
    return 0 if report["all_reconstructed"] else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))

#!/usr/bin/env python3
"""Classify the open reply panel's failures from a sealed chat-grade report.

Phase 1 of the open-reply-panel diagnostic (#2029). No model runs: every
number comes from the report's own grader verdict fields plus deterministic
text rules stated in RULES below.

Usage:
  open_reply_panel_classify.py REPORT.json [--panel A.json,B.json] \\
      [--out classification.json] [--tsv verdicts.tsv]
  open_reply_panel_classify.py --selftest

  --panel   verify the report against the panel bytes it claims to have graded
            (id, category and user turns for every row); a mismatch exits 1.
  --out     the full classification, including each failure's reply text.
  --tsv     one row per failure: id, category, tier, verdict class, flags.

The open reply panel is `everyday-32.json` + `heldout-200-a.json` +
`heldout-200-b.json` (232 rows), which are not tracked in this repository; see
docs/labs/open-reply-panel-2026-10-09/README.md for their sha256 and where the
sealed report and panel bytes are stored.
"""
import collections
import json
import re
import sys
from pathlib import Path

# ---------------------------------------------------------------- verdicts
# chat-grade's own rule: acceptable = fluent AND relevant (no row checks on
# this panel). The verdict fields are r["grades"]["fluent"|"relevant"], each
# true/false/None (None = the judge's answer did not start with yes or no).
VERDICT_CLASSES = {
    (True, True): "acceptable",
    (True, False): "fluent_only",
    (False, True): "relevant_only",
    (False, False): "neither",
}

# ---------------------------------------------------------------- mechanisms
RULES = """Deterministic text rules over the failing row's final reply; each is a
necessary-condition marker, not a full diagnosis:
  repeat5       some 5-word sequence of whitespace tokens occurs twice or more
                (verbatim degenerate repetition)
  no_terminal   the last non-space character is not . ! ? " ' ) ] } or a
                backtick: the reply was cut mid-clause (a 64-token cap or a
                cycle stop, not a finished sentence)
  echo_user     the final reply contains the whole of the row's last user turn
                verbatim (punctuation-normalised), i.e. it restates the request
  stub          the reply is two words or fewer
  code_fence    an odd number of ``` fences: a code block was opened and not
                closed
  question      the reply contains a question mark
  role_leak     the reply contains a literal "User:" or "Assistant:" label
  restate_high  at least 70% of the reply's distinct content words appear in
                the last user turn (a tighter restatement measure than echo_user)
  refusal       the reply contains one of chat-grade's own ABSTAIN_PHRASES
                (chat-grade.rs, the shared `abstain` list), matched on word
                boundaries with any run of spaces between the words
"""

TERMINAL = set('.!?"\')]}*`:')

# chat-grade.rs `ABSTAIN_PHRASES`, verbatim: the list the frozen instrument
# itself uses for an abstention. A refusal here is "the reply declines or
# disclaims rather than answers", on the instrument's own definition.
ABSTAIN_PHRASES = [
    "don't know",
    "dont know",
    "do not know",
    "didn't tell",
    "did not tell",
    "haven't told",
    "have not told",
    "not sure",
    "no way to know",
    "no way of knowing",
    "can't know",
    "cannot know",
    "can't",
    "cant",
    "cannot",
    "can not",
    "not able",
    "unable",
    "no idea",
    "i wish i could",
    "impossible",
]
_ABSTAIN_RES = [
    re.compile(r"\b" + re.escape(p).replace(r"\ ", r"\s+") + r"\b") for p in ABSTAIN_PHRASES
]


def tokens(text):
    return re.findall(r"[A-Za-z0-9']+", text.lower())


def normalise(text):
    return " ".join(tokens(text))


def repeated_ngram(toks, n=5):
    """True when some n-gram of tokens occurs at least twice."""
    if len(toks) < n * 2:
        return False
    seen = set()
    for i in range(len(toks) - n + 1):
        gram = tuple(toks[i : i + n])
        if gram in seen:
            return True
        seen.add(gram)
    return False


def mechanism_flags(reply, user_turns):
    stripped = reply.rstrip()
    last_user = user_turns[-1]
    toks = tokens(reply)
    user_toks = tokens(last_user)
    return {
        "repeat5": repeated_ngram(toks, 5),
        "no_terminal": bool(stripped) and stripped[-1] not in TERMINAL,
        "echo_user": bool(normalise(last_user)) and normalise(last_user) in normalise(reply),
        "stub": len(toks) <= 2,
        "code_fence": stripped.count("```") % 2 == 1,
        "question": "?" in reply,
        "role_leak": bool(re.search(r"\b(User|Assistant):", reply)),
        "restate_high": bool(user_toks)
        and len(set(toks) & set(user_toks)) / max(1, len(set(toks))) >= 0.7,
        "refusal": any(r.search(reply.lower()) for r in _ABSTAIN_RES),
    }


def verdict_class(row):
    return VERDICT_CLASSES[(bool(row["grades"]["fluent"]), bool(row["grades"]["relevant"]))]


def tier_of(row, ill_posed):
    if row["category"] == "heldout_first_turn":
        return "K-ill-posed" if row["id"] in ill_posed else "K-clean"
    return "everyday"


def classify(report, ill_posed):
    rows = report["rows"]
    crosstab = {
        "by_category": collections.defaultdict(collections.Counter),
        "by_tier": collections.defaultdict(collections.Counter),
        "overall": collections.Counter(),
    }
    failures = []
    for row in rows:
        cls = verdict_class(row)
        tier = tier_of(row, ill_posed)
        crosstab["overall"][cls] += 1
        crosstab["by_category"][row["category"]][cls] += 1
        crosstab["by_tier"][tier][cls] += 1
        if cls == "acceptable":
            continue
        turns = row["conversation"]
        reply = turns[-1]["assistant"]
        flags = mechanism_flags(reply, [t["user"] for t in turns])
        failures.append(
            {
                "id": row["id"],
                "category": row["category"],
                "tier": tier,
                "verdict_class": cls,
                "reply_tokens": len(tokens(reply)),
                "flags": flags,
                "reply": reply,
            }
        )
    counts = collections.Counter()
    for f in failures:
        for flag, on in f["flags"].items():
            if on:
                counts[flag] += 1
    # Cover partition: the first rule that fires, in a fixed precedence.
    precedence = [
        "stub",
        "role_leak",
        "code_fence",
        "echo_user",
        "repeat5",
        "no_terminal",
        "refusal",
        "restate_high",
        "question",
    ]
    cover = collections.Counter()
    for f in failures:
        label = "other"
        for flag in precedence:
            if f["flags"][flag]:
                label = flag
                break
        cover[label] += 1
    return {
        "rows": len(rows),
        "acceptable": crosstab["overall"]["acceptable"],
        "failing": len(failures),
        "crosstab": {
            "overall": dict(crosstab["overall"]),
            "by_category": {k: dict(v) for k, v in crosstab["by_category"].items()},
            "by_tier": {k: dict(v) for k, v in crosstab["by_tier"].items()},
        },
        "mechanism_counts": dict(counts),
        "mechanism_cover_precedence": precedence,
        "mechanism_cover": dict(cover),
        "rules": RULES,
        "failures": failures,
    }


def bind_panel(report, panel_paths):
    """Verify a sealed report against the panel bytes it claims to have graded.

    Every report row must match a panel request on id, category and user turns.
    Returns (checked, mismatches). A mismatch is a hard error: the report is not
    bound to that panel.
    """
    panel = []
    for path in panel_paths:
        data = json.load(open(path))
        panel.extend(data["requests"] if isinstance(data, dict) and "requests" in data else data)
    by_id = {row["id"]: row for row in panel}
    mismatches = []
    for row in report["rows"]:
        request = by_id.get(row["id"])
        if request is None:
            mismatches.append(f"{row['id']}: not in panel")
            continue
        if request["category"] != row["category"]:
            mismatches.append(f"{row['id']}: category {request['category']} != {row['category']}")
        if list(request["user_turns"]) != [t["user"] for t in row["conversation"]]:
            mismatches.append(f"{row['id']}: user turns differ")
    return len(report["rows"]), mismatches


def selftest():
    assert verdict_class({"grades": {"fluent": True, "relevant": False}}) == "fluent_only"
    assert repeated_ngram(tokens("a b c d e a b c d e"), 5)
    assert not repeated_ngram(tokens("a b c d e f g h i j"), 5)
    assert mechanism_flags("I do not know", ["x"])["no_terminal"]
    assert not mechanism_flags("I do not know.", ["x"])["no_terminal"]
    assert mechanism_flags("hello there friend", ["hello there friend"])["echo_user"]
    assert mechanism_flags("```", ["x"])["code_fence"]
    assert mechanism_flags("ok", ["x"])["stub"]
    assert mechanism_flags("I cannot help with that.", ["x"])["refusal"]
    assert not mechanism_flags("I can do that.", ["x"])["refusal"]
    print("selftest ok")


def tsv_rows(result):
    flags = [
        "no_terminal",
        "repeat5",
        "restate_high",
        "code_fence",
        "question",
        "echo_user",
        "stub",
        "role_leak",
        "refusal",
    ]
    lines = ["\t".join(["id", "category", "tier", "verdict_class", "reply_words", *flags])]
    for failure in result["failures"]:
        lines.append(
            "\t".join(
                [
                    failure["id"],
                    failure["category"],
                    failure["tier"],
                    failure["verdict_class"],
                    str(failure["reply_tokens"]),
                    *["1" if failure["flags"][f] else "0" for f in flags],
                ]
            )
        )
    return "\n".join(lines) + "\n"


def main() -> int:
    if "--selftest" in sys.argv:
        selftest()
        return 0
    argv = [a for a in sys.argv[1:] if not a.startswith("--")]
    option = lambda name: (  # noqa: E731
        sys.argv[sys.argv.index(name) + 1] if name in sys.argv else None
    )
    if not argv:
        print(__doc__)
        return 2
    report_path = argv[0]
    report = json.load(open(report_path))
    root = Path(__file__).resolve().parents[1]

    panel = option("--panel")
    if panel:
        checked, mismatches = bind_panel(report, panel.split(","))
        print(f"panel binding: {checked} rows checked, {len(mismatches)} mismatches")
        for line in mismatches[:20]:
            print("  ", line)
        if mismatches:
            return 1

    ill_posed = set((root / "data/panels/heldout-ill-posed-v3-ids.txt").read_text().split())
    result = classify(report, ill_posed)

    out_path = option("--out")
    if out_path:
        json.dump(result, open(out_path, "w"), indent=1)
    tsv_path = option("--tsv")
    if tsv_path:
        open(tsv_path, "w").write(tsv_rows(result))

    print(f"rows={result['rows']} acceptable={result['acceptable']} failing={result['failing']}")
    print("overall:", result["crosstab"]["overall"])
    print("by_tier:", json.dumps(result["crosstab"]["by_tier"], indent=1))
    print("by_category:")
    for k, v in sorted(result["crosstab"]["by_category"].items()):
        print(f"  {k:24s} {v}")
    print("mechanism counts over failures:", json.dumps(result["mechanism_counts"], indent=1))
    print("cover (first rule by precedence):", json.dumps(result["mechanism_cover"], indent=1))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

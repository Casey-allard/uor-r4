#!/usr/bin/env python3
"""Compare two sealed `chat-grade grade` reports of the same panel at different token caps.

For #2029. CPU only: reads two sealed report JSON files, generates nothing and grades
nothing. The primary use is the cap-96 measurement: `--before` is the 64-token run and
`--after` the 96-token run, both on the same machine and build, so the cap is the only
delta. It also serves the cross-platform anchor check (the sealed macOS 64-token report
against the Linux 64-token control).

Reports, for the two runs and for every panel category and tier:
  * `acceptable` (the criterion's field: fluent and relevant), and the four verdict classes
    the previous round used (`acceptable`, `neither`, `fluent_only`, `relevant_only`);
  * the movement of the FAILING SET, which is the interesting number: how many rows that
    failed before are acceptable after, and how many that were acceptable before fail
    after (the discordant pairs), with the two-sided exact McNemar p;
  * the `no_terminal` mid-clause marker among failures, before and after;
  * with `--tokens-after`, the token-count distribution of the after run's replies, so a
    reader can see whether the new cap is simply the next wall.

Usage:
    python3 scripts/open_reply_panel_cap_compare.py --before A.json --after B.json \
        [--tokens-after COUNTS.json] [--out summary.json] [--tsv rows.tsv]
"""
import argparse
import collections
import json
import math
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

import open_reply_panel_classify as classifier  # noqa: E402

OBSERVED_CAP = 64  # the earlier round's instrument; --cap overrides


def rows_by_id(report):
    return {row["id"]: row for row in report["rows"]}


def reply_of(row):
    return row["conversation"][-1]["assistant"]


def exact_mcnemar(b, c):
    """Two-sided exact McNemar p for b and c discordant pairs."""
    n = b + c
    if n == 0:
        return 1.0
    k = min(b, c)
    tail = sum(math.comb(n, i) for i in range(k + 1)) / (2 ** n)
    return min(1.0, 2 * tail)


def table(pairs, key):
    """Counts per verdict class for a grouping key."""
    out = collections.defaultdict(collections.Counter)
    for row, verdict in pairs:
        out[key(row)][verdict] += 1
    return {k: dict(v) for k, v in out.items()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--before", required=True)
    parser.add_argument("--after", required=True)
    parser.add_argument("--tokens-after")
    parser.add_argument("--cap", type=int, default=OBSERVED_CAP)
    parser.add_argument("--out")
    parser.add_argument("--tsv")
    args = parser.parse_args()

    before = json.load(open(args.before))
    after = json.load(open(args.after))
    ill_posed = set((ROOT / "data/panels/heldout-ill-posed-v3-ids.txt").read_text().split())
    b_rows, a_rows = rows_by_id(before), rows_by_id(after)
    if set(b_rows) != set(a_rows):
        raise SystemExit("the two reports do not carry the same row ids")

    counts = json.load(open(args.tokens_after)) if args.tokens_after else None
    after_order = [row["id"] for row in after["rows"]]
    tokens = dict(zip(after_order, counts)) if counts else {}

    per_row = []
    for row_id in [row["id"] for row in before["rows"]]:
        b_row, a_row = b_rows[row_id], a_rows[row_id]
        b_verdict, a_verdict = classifier.verdict_class(b_row), classifier.verdict_class(a_row)
        b_reply, a_reply = reply_of(b_row), reply_of(a_row)
        b_flags = classifier.mechanism_flags(b_reply, [t["user"] for t in b_row["conversation"]])
        a_flags = classifier.mechanism_flags(a_reply, [t["user"] for t in a_row["conversation"]])
        entry = {
            "id": row_id,
            "category": b_row["category"],
            "tier": classifier.tier_of(b_row, ill_posed),
            "before": b_verdict,
            "after": a_verdict,
            "before_ok": b_verdict == "acceptable",
            "after_ok": a_verdict == "acceptable",
            "before_mid_clause": b_flags["no_terminal"],
            "after_mid_clause": a_flags["no_terminal"],
            "same_reply": b_reply == a_reply,
            "after_tokens": tokens.get(row_id, {}).get("tokens"),
            "after_ids": tokens.get(row_id, {}).get("ids"),
        }
        per_row.append(entry)

    before_ok = sum(1 for r in per_row if r["before_ok"])
    after_ok = sum(1 for r in per_row if r["after_ok"])
    gained = [r["id"] for r in per_row if not r["before_ok"] and r["after_ok"]]
    lost = [r["id"] for r in per_row if r["before_ok"] and not r["after_ok"]]
    failures_before = [r for r in per_row if not r["before_ok"]]
    changed_failures = [r for r in failures_before if r["after_ok"]]

    summary = {
        "before": {"path": args.before, "acceptable": before_ok, "rows": len(per_row),
                   "model_sha256": before.get("model_sha256"),
                   "executable_sha256": before.get("executable_sha256"),
                   "tokenizer_sha256": before.get("tokenizer_sha256"),
                   "max_new_tokens": before.get("max_new_tokens"),
                   "grader_digest": (before.get("grader") or {}).get("digest")},
        "after": {"path": args.after, "acceptable": after_ok, "rows": len(per_row),
                  "model_sha256": after.get("model_sha256"),
                  "executable_sha256": after.get("executable_sha256"),
                  "tokenizer_sha256": after.get("tokenizer_sha256"),
                  "max_new_tokens": after.get("max_new_tokens"),
                  "grader_digest": (after.get("grader") or {}).get("digest")},
        "delta_acceptable": after_ok - before_ok,
        "failing_before": len(failures_before),
        "failures_changed_to_acceptable": len(changed_failures),
        "failures_changed_ids": [r["id"] for r in changed_failures],
        "failures_unchanged": len(failures_before) - len(changed_failures),
        "gained": len(gained),
        "lost": len(lost),
        "gained_ids": gained,
        "lost_ids": lost,
        "exact_mcnemar_p": exact_mcnemar(len(gained), len(lost)),
        "same_reply_rows": sum(1 for r in per_row if r["same_reply"]),
        "by_category": {
            "before": table([(r, r["before"]) for r in per_row], lambda r: r["category"]),
            "after": table([(r, r["after"]) for r in per_row], lambda r: r["category"]),
        },
        "by_tier": {
            "before": table([(r, r["before"]) for r in per_row], lambda r: r["tier"]),
            "after": table([(r, r["after"]) for r in per_row], lambda r: r["tier"]),
        },
        "mid_clause_failures": {
            "before": sum(1 for r in per_row if r["before_mid_clause"] and not r["before_ok"]),
            "after": sum(1 for r in per_row if r["after_mid_clause"] and not r["after_ok"]),
        },
        "mid_clause_replies": {
            "before": sum(1 for r in per_row if r["before_mid_clause"]),
            "after": sum(1 for r in per_row if r["after_mid_clause"]),
        },
        "after_token_distribution": None,
    }
    if tokens:
        hist = collections.Counter(t["tokens"] for t in tokens.values())
        cap = after.get("max_new_tokens", args.cap)
        failures_token_hist = collections.Counter(
            r["after_tokens"] for r in per_row if not r["after_ok"] and r["after_tokens"]
        )
        summary["after_token_distribution"] = {
            "cap": cap,
            "total_tokens": sum(hist.values()),
            "replies_at_or_above_cap": sum(n for k, n in hist.items() if k >= cap),
            "failure_histogram": dict(sorted(failures_token_hist.items())),
            "histogram": dict(sorted(hist.items())),
            "failures_at_or_above_cap": sum(
                n for k, n in failures_token_hist.items() if k >= cap),
        }
    if args.out:
        json.dump(summary, open(args.out, "w"), indent=1)
    if args.tsv:
        columns = ["id", "category", "tier", "before", "after", "before_ok", "after_ok",
                   "before_mid_clause", "after_mid_clause", "same_reply", "after_tokens"]
        with open(args.tsv, "w") as handle:
            handle.write("\t".join(columns) + "\n")
            for row in per_row:
                handle.write("\t".join(
                    str(int(row[c]) if isinstance(row[c], bool) else ("" if row[c] is None else row[c]))
                    for c in columns) + "\n")
    print(json.dumps({k: v for k, v in summary.items()
                      if k not in ("after_token_distribution",)}, indent=1))
    if summary["after_token_distribution"]:
        dist = summary["after_token_distribution"]
        print("after cap:", dist["cap"],
              "| replies at or above cap:", dist["replies_at_or_above_cap"], "of",
              len(per_row), "| failures at or above cap:", dist["failures_at_or_above_cap"])
        print("after failure token histogram:", json.dumps(dist["failure_histogram"]))
    return 0


if __name__ == "__main__":
    sys.exit(main())

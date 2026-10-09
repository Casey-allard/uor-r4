#!/usr/bin/env python3
"""Cap-truncation or voluntary stop: token counts for the open reply panel (#2029).

CPU only. No model, no generation, no grading. This re-reads the sealed
`chat-grade` reports and counts the tokens of every stored reply with the exact
tokenizer the report names, using `crates/uor-r4-tokenizer`'s own engine.

WHY THE COUNT DECIDES IT. `chat-grade`'s generation loop
(`stack_dialogue::greedy_reply_with`, `Reply::stop_with`) runs at most
`max_new_tokens` steps and ends early only on EOS or on a short repeated cycle
(`cycle_repeats=3`). `reply_panel` stores `decode(ids without EOS)`. Rank-ordered
BPE is the minimal segmentation of the decoded bytes, so for
`t = |encode(stored_text)|`:

    t <= len(ids) <= max_new_tokens

`t == max_new_tokens` therefore proves the loop exhausted its budget: the model
never emitted EOS and never entered a terminal cycle, and the reply is cut where
the budget ran out. `t < max_new_tokens` is reported as a voluntary stop, with the
caveat that a cap run whose ids were split non-canonically would also land below
the cap; the distribution below says how much room that caveat has.

USAGE

    python3 scripts/open_reply_panel_token_counts.py extract --report R.json --out reply-texts.json
    cargo run --release -p uor-r4-tokenizer --example token-counts -- \
        TOKENIZER.json reply-texts.json token-counts.json
    python3 scripts/open_reply_panel_token_counts.py summarise \
        --report R.json --tokens token-counts.json --out summary.json --tsv rows.tsv
"""
import argparse
import collections
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

import open_reply_panel_classify as classifier  # noqa: E402


def rows_of(report):
    return report["rows"]


def reply_of(row):
    """The model's final reply for a row: the last assistant entry of the graded
    conversation, which `reply_panel` filled with `decode(ids without EOS)`."""
    return row["conversation"][-1]["assistant"]


def extract(args):
    report = json.load(open(args.report))
    texts = [reply_of(row) for row in rows_of(report)]
    json.dump(texts, open(args.out, "w"))
    ids = [row["id"] for row in rows_of(report)]
    json.dump(ids, open(str(args.out) + ".ids", "w"), indent=1)
    print(f"{len(texts)} replies written to {args.out} (ids beside it)")
    return 0


def summarise(args):
    report = json.load(open(args.report))
    counts = json.load(open(args.tokens))
    rows = rows_of(report)
    if len(counts) != len(rows):
        raise SystemExit(f"{len(counts)} token counts for {len(rows)} rows")
    cap = args.cap
    ill_posed = set(
        (ROOT / "data/panels/heldout-ill-posed-v3-ids.txt").read_text().split()
    )

    per_row = []
    for row, count in zip(rows, counts):
        reply = reply_of(row)
        tokens = count["tokens"]
        flags = classifier.mechanism_flags(
            reply, [turn["user"] for turn in row["conversation"]]
        )
        verdict = classifier.verdict_class(row)
        per_row.append(
            {
                "id": row["id"],
                "category": row["category"],
                "tier": classifier.tier_of(row, ill_posed),
                "verdict_class": verdict,
                "acceptable": verdict == "acceptable",
                "tokens": tokens,
                "at_cap": tokens >= cap,
                "mid_clause": flags["no_terminal"],
                "repeat5": flags["repeat5"],
                "decode_roundtrip": count.get("decode_roundtrip", True),
                "flags": flags,
                "reply": reply,
            }
        )

    def tab(predicate, label):
        """at-cap split over the population `predicate` selects."""
        rows_ = [r for r in per_row if predicate(r)]
        at = sum(1 for r in rows_ if r["at_cap"])
        return {
            "population": label,
            "rows": len(rows_),
            "at_cap": at,
            "below_cap": len(rows_) - at,
            "at_cap_share": round(at / len(rows_), 4) if rows_ else None,
        }

    failures = per_row and [r for r in per_row if not r["acceptable"]] or []
    mid_clause = [r for r in per_row if r["mid_clause"]]
    tabs = [
        tab(lambda r: True, "panel"),
        tab(lambda r: r["acceptable"], "accepted"),
        tab(lambda r: not r["acceptable"], "rejected (all failures)"),
        tab(lambda r: r["mid_clause"], "mid-clause replies"),
        tab(lambda r: r["mid_clause"] and not r["acceptable"], "mid-clause failures"),
        tab(lambda r: not r["mid_clause"] and not r["acceptable"],
            "failures with terminal punctuation"),
        tab(lambda r: r["repeat5"] and not r["acceptable"], "failures with repeat5"),
        tab(lambda r: not any(r["flags"][f] for f in ("repeat5", "no_terminal"))
            and not r["acceptable"], "failures with neither marker"),
        tab(lambda r: r["tier"] == "K-clean", "tier K-clean"),
        tab(lambda r: r["tier"] == "K-ill-posed", "tier K-ill-posed"),
        tab(lambda r: r["tier"] == "everyday", "tier everyday"),
        tab(lambda r: r["tier"] == "K-clean" and not r["acceptable"],
            "K-clean failures"),
        tab(lambda r: r["tier"] == "K-ill-posed" and not r["acceptable"],
            "K-ill-posed failures"),
    ]

    histogram = collections.Counter(r["tokens"] for r in per_row)
    failures_histogram = collections.Counter(
        r["tokens"] for r in per_row if not r["acceptable"]
    )
    mid_histogram = collections.Counter(r["tokens"] for r in mid_clause)
    # Population split on the two markers and the cap.
    quad = collections.Counter(
        (r["at_cap"], r["mid_clause"]) for r in per_row if not r["acceptable"]
    )
    tier_rejected = collections.Counter(
        (r["tier"], r["at_cap"]) for r in per_row if not r["acceptable"]
    )
    summary = {
        "report": args.report,
        "rows": len(per_row),
        "acceptable": sum(1 for r in per_row if r["acceptable"]),
        "failing": len(per_row) - sum(1 for r in per_row if r["acceptable"]),
        "cap": cap,
        "tokenizer": report.get("tokenizer_sha256"),
        "executable_sha256": report.get("executable_sha256"),
        "max_new_tokens_in_report": report.get("max_new_tokens"),
        "models": {"model": report.get("model"), "model_sha256": report.get("model_sha256")},
        "lossy_decodes": [r["id"] for r in per_row if not r["decode_roundtrip"]],
        "crosstab": tabs,
        "failure_quadrant_at_cap_mid_clause": {
            f"at_cap={a},mid_clause={m}": n for (a, m), n in sorted(quad.items())
        },
        "rejected_by_tier_and_cap": {
            f"{tier},at_cap={a}": n for (tier, a), n in sorted(tier_rejected.items())
        },
        "token_histogram": dict(sorted(histogram.items())),
        "failure_token_histogram": dict(sorted(failures_histogram.items())),
        "mid_clause_token_histogram": dict(sorted(mid_histogram.items())),
        "total_tokens": sum(r["tokens"] for r in per_row),
        "rows_per_cap": {
            "at_cap": sum(1 for r in per_row if r["at_cap"]),
            "below_cap": sum(1 for r in per_row if not r["at_cap"]),
        },
    }
    if args.out:
        json.dump(summary, open(args.out, "w"), indent=1)
    if args.tsv:
        columns = ["id", "category", "tier", "verdict_class", "acceptable", "tokens",
                   "at_cap", "mid_clause", "repeat5", "decode_roundtrip"]
        with open(args.tsv, "w") as handle:
            handle.write("\t".join(columns) + "\n")
            for row in per_row:
                handle.write("\t".join(str(int(row[c]) if isinstance(row[c], bool) else row[c])
                                       for c in columns) + "\n")
    print(json.dumps({k: v for k, v in summary.items()
                      if k not in ("token_histogram", "failure_token_histogram",
                                   "mid_clause_token_histogram")}, indent=1))
    print("failure token histogram:", json.dumps(summary["failure_token_histogram"]))
    print("mid-clause token histogram:", json.dumps(summary["mid_clause_token_histogram"]))
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    p_extract = sub.add_parser("extract")
    p_extract.add_argument("--report", required=True)
    p_extract.add_argument("--out", required=True)
    p_extract.set_defaults(func=extract)
    p_sum = sub.add_parser("summarise")
    p_sum.add_argument("--report", required=True)
    p_sum.add_argument("--tokens", required=True)
    p_sum.add_argument("--cap", type=int, default=64)
    p_sum.add_argument("--out")
    p_sum.add_argument("--tsv")
    p_sum.set_defaults(func=summarise)
    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())

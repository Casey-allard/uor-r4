#!/usr/bin/env python3
"""Classify the open reply panel's completed-but-rejected failures, and compare them with
the accepted set. For #2029. CPU only: reads one sealed report and a token-count file,
generates nothing and grades nothing.

POPULATIONS (232 sealed rows, `chat-grade grade`, 29M chat fine-tune, cap 64):

  accepted                  verdict_class == acceptable
  completed-but-rejected    a FAILURE whose reply ends in terminal punctuation and whose
                            canonical token count is below the 64-token cap, i.e. it is not
                            the budget (`no_terminal` is chat-grade's own rule: the last
                            non-space character is not . ! ? " ' ) ] } * ` :)
  mid-clause failures       the rest of the failures - the budget-cut ones of Result 4
  failures                  everything not accepted

PRIMARY CLASSIFICATION is the grader's own two verdict fields, because they are the only
mechanism labels the instrument itself carries. SECONDARY classifications are deterministic
text rules; each is a marker, not a diagnosis, and each is named as this script's rule. Any
property that does not separate the sets is reported as not separating.

No category is invented: where the data cannot distinguish two possibilities the script says
so rather than splitting them.

Usage:
    python3 scripts/open_reply_panel_completed_classify.py --report R.json \
        --tokens TOKENS.json [--out summary.json] [--tsv rows.tsv]
"""
import argparse
import collections
import json
import math
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

import open_reply_panel_classify as classifier  # noqa: E402

# This script's own text rules. Each is a necessary-condition marker, not a diagnosis.
# `dangling_tail` is the one that matters for this piece: a reply that ends with terminal
# punctuation but whose final word is a function word has the punctuation without the
# clause ("... and." / "... of." / "... the.").
DANGLING_TAIL = {
    "and", "or", "but", "so", "because", "if", "when", "while", "as", "than", "that",
    "the", "a", "an", "of", "to", "in", "on", "at", "for", "with", "from", "by", "into",
    "about", "over", "under", "between", "is", "are", "was", "were", "be", "been", "being",
    "will", "would", "can", "could", "should", "may", "might", "must", "do", "does", "did",
    "have", "has", "had", "my", "your", "his", "her", "its", "our", "their", "this", "these",
    "those", "there", "here", "not", "no", "yes", "very", "more", "most", "such", "each",
    "every", "some", "any", "all", "both", "either", "neither", "also", "then", "too",
}
CONTENT_STOP = DANGLING_TAIL | {
    "i", "you", "he", "she", "it", "we", "they", "me", "him", "us", "them", "what", "which",
    "who", "whom", "whose", "where", "why", "how", "one", "two", "three", "get", "got",
    "make", "made", "go", "going", "like", "just", "really", "well", "much", "many", "lot",
}


def token_words(text):
    return classifier.tokens(text)


def content_words(text):
    return [w for w in token_words(text) if w not in CONTENT_STOP and len(w) > 2]


def extra_flags(reply, user_turns):
    words = token_words(reply)
    last = user_turns[-1] if user_turns else ""
    last_content = set(content_words(last))
    reply_content = set(content_words(reply))
    earlier = " ".join(user_turns[:-1])
    return {
        "dangling_tail": bool(words) and words[-1] in DANGLING_TAIL,
        "ends_ellipsis_or_colon": reply.rstrip().endswith(("...", ":", "\u2026")),
        "has_digit": any(c.isdigit() for c in reply),
        "no_content_overlap_with_last_turn": bool(last_content) and not (reply_content & last_content),
        "high_content_overlap_with_last_turn": bool(reply_content)
        and len(reply_content & last_content) / len(reply_content) >= 0.5,
        "shares_with_earlier_turn_only": bool(user_turns[1:])
        and bool(reply_content & set(content_words(earlier)))
        and not (reply_content & last_content),
    }


def fisher_exact(a, b, c, d):
    """Two-sided Fisher exact p for [[a, b], [c, d]]."""
    n = a + b + c + d
    if n == 0:
        return 1.0

    def hyper(x):
        return (math.comb(a + b, x) * math.comb(c + d, a + c - x)) / math.comb(n, a + c)

    observed = hyper(a)
    total = 0.0
    for x in range(0, min(a + b, a + c) + 1):
        p = hyper(x)
        if p <= observed + 1e-12:
            total += p
    return min(1.0, total)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", required=True)
    parser.add_argument("--tokens", required=True)
    parser.add_argument("--cap", type=int, default=64)
    parser.add_argument("--out")
    parser.add_argument("--tsv")
    args = parser.parse_args()

    report = json.load(open(args.report))
    counts = dict(zip([r["id"] for r in report["rows"]], json.load(open(args.tokens))))

    rows = []
    for row in report["rows"]:
        reply = row["conversation"][-1]["assistant"]
        users = [turn["user"] for turn in row["conversation"]]
        flags = classifier.mechanism_flags(reply, users)
        flags.update(extra_flags(reply, users))
        verdict = classifier.verdict_class(row)
        tokens = counts[row["id"]]["tokens"]
        accepted = verdict == "acceptable"
        mid_clause = flags["no_terminal"]
        at_cap = tokens >= args.cap
        if accepted:
            population = "accepted"
        elif mid_clause:
            population = "mid_clause_failure"
        elif at_cap:
            population = "cap_terminated_failure"
        else:
            population = "completed_but_rejected"
        rows.append({
            "id": row["id"],
            "category": row["category"],
            "verdict_class": verdict,
            "tokens": tokens,
            "words": len(token_words(reply)),
            "population": population,
            "flags": flags,
            "reply": reply,
        })

    pops = collections.Counter(r["population"] for r in rows)
    groups = {
        "accepted": [r for r in rows if r["population"] == "accepted"],
        "completed_but_rejected": [r for r in rows if r["population"] == "completed_but_rejected"],
        "mid_clause_failure": [r for r in rows if r["population"] == "mid_clause_failure"],
        "cap_terminated_failure": [r for r in rows if r["population"] == "cap_terminated_failure"],
    }

    # Primary: the grader's own verdict fields.
    primary = {name: dict(collections.Counter(r["verdict_class"] for r in group))
               for name, group in groups.items()}

    # Secondary: text rules, with a separation test between accepted and completed-rejected.
    rules = sorted(set().union(*[set(r["flags"]) for r in rows]))
    comparison = []
    accepted, completed = groups["accepted"], groups["completed_but_rejected"]
    for rule in rules:
        a = sum(1 for r in accepted if r["flags"][rule])
        b = len(accepted) - a
        c = sum(1 for r in completed if r["flags"][rule])
        d = len(completed) - c
        comparison.append({
            "rule": rule,
            "accepted_true": a, "accepted_of": len(accepted),
            "completed_true": c, "completed_of": len(completed),
            "accepted_share": round(a / len(accepted), 4) if accepted else None,
            "completed_share": round(c / len(completed), 4) if completed else None,
            "share_gap": round((a / len(accepted) if accepted else 0)
                               - (c / len(completed) if completed else 0), 4),
            "fisher_exact_p": round(fisher_exact(a, b, c, d), 4),
        })
    comparison.sort(key=lambda row: -abs(row["share_gap"]))

    # Length, the one continuous property.
    def stats(group):
        if not group:
            return None
        vals = sorted(r["tokens"] for r in group)
        words = sorted(r["words"] for r in group)
        return {
            "rows": len(group),
            "tokens_median": vals[len(vals) // 2],
            "tokens_mean": round(sum(vals) / len(vals), 1),
            "tokens_min": vals[0], "tokens_max": vals[-1],
            "words_median": words[len(words) // 2],
            "words_mean": round(sum(words) / len(words), 1),
        }

    summary = {
        "report": args.report,
        "rows": len(rows),
        "cap": args.cap,
        "populations": dict(pops),
        "primary_verdict_classes": primary,
        "length": {name: stats(group) for name, group in groups.items()},
        "text_rule_comparison": comparison,
        "completed_but_rejected_ids": [r["id"] for r in completed],
        "notes": {
            "panel_has_no_frozen_checks": (
                "The open panel's ids are talk-*/do-*/ask-*/follow-*/heldout-* and chat-grade's "
                "embedded checks files carry only conv-* ids, so checked_rows is 0 and no row has "
                "an expected value. 'Does the reply contain a value from the panel's own checks' "
                "is therefore NOT MEASURABLE on this panel - there are no check values to contain."
            ),
        },
    }
    if args.out:
        json.dump(summary, open(args.out, "w"), indent=1)
    if args.tsv:
        columns = ["id", "category", "verdict_class", "population", "tokens", "words"] + rules
        with open(args.tsv, "w") as handle:
            handle.write("\t".join(columns) + "\n")
            for row in rows:
                handle.write("\t".join(
                    [row["id"], row["category"], row["verdict_class"], row["population"],
                     str(row["tokens"]), str(row["words"])]
                    + [str(int(row["flags"][rule])) for rule in rules]) + "\n")

    print(json.dumps({k: v for k, v in summary.items() if k != "text_rule_comparison"}, indent=1))
    print("\ntext rules, accepted vs completed-but-rejected, by |share gap|:")
    print(f"{'rule':40s} {'accepted':>10s} {'completed':>10s} {'gap':>7s} {'p':>7s}")
    for row in comparison:
        print(f"{row['rule']:40s} {row['accepted_true']:4d}/{row['accepted_of']:<5d} "
              f"{row['completed_true']:4d}/{row['completed_of']:<5d} "
              f"{row['share_gap']:+7.3f} {row['fisher_exact_p']:7.4f}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

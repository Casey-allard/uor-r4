#!/usr/bin/env python3
"""Author the deterministic row checks for the open reply panel (#2029). CPU only.

WHY THIS EXISTS. The open reply panel's reading is a judge's: it has no frozen row checks, its
failures are diffuse across two classifications (Results 6 and 7), its cap is not the constraint
(Result 5), and Result 7 measured that the accepted set is not separable from the
completed-but-rejected set by any judge-free property once one canned greeting is set aside —
that greeting being 18.6 % of all accepted rows at 29M and 23.9 % at 100M. This file builds the
deterministic component: row checks in the shape the memory panels use, so at least one part of
the reading cannot move between identical runs, and a canned string cannot carry it.

TWO COMPONENTS, because the panel has two kinds of row.

  * CONTENT rows — requests whose correct answer must contain a specific piece of content
    (a fact, an API name, a number). Check kind `any`: the reply must contain at least one of
    the row's anchors. The anchors are the row's own required content, so they differ per row
    and none of them occurs in any canned reply. THAT is what makes a single canned string
    unable to pass across rows: it would have to contain every row's content word at once.
  * ILL-POSED rows — the requests chat-grade's own `missing_material` rule already selects
    (the recorded id list `data/panels/heldout-ill-posed-v3-ids.txt`): the text refers to
    material it does not contain, so the correct behaviour is to ask for it or decline, and the
    defect is inventing content. Check kind `abstain_exact`, which is exactly that rule
    (no fabricated specific, no agreement, no forbidden answer class). Read against the best
    constant, as v3 does for its clarify and unknowable categories, because a constant
    abstention passes it by design — that is reported, not hidden.

AUTHORING DISCIPLINE. Anchors were written from the REQUEST and from what the answer must
contain, never from what the model replied; the observed replies were used only to build the
canned-reply control list, which is what a wrong answer looks like. Every anchor is dropped if
it occurs in the row's own last user turn, because `chat-grade validate_checks` refuses a check
term that is in the last turn (a row answerable by echoing the request is not a check). No
anchor is loosened to make a canned string pass or fail: the control arm reports what it finds.

    python3 scripts/reply_panel_checks_build.py --panel-dir DIR --out-checks F.tsv \
        --out-provenance F.tsv --out-canned F.txt
"""
import argparse
import json
import os
import re
import sys

# REQUIRED CONTENT per row: content the answer must contain, written from the request.
# Kept deliberately small and row-specific. An anchor that is not in the reply means the reply
# did not contain the thing the request asked for.
CONTENT_ANCHORS = {
    "ask-01": ["chlorophyll", "pigment", "sunlight", "carotenoid"],
    "ask-02": ["honey"],
    "ask-03": ["hydrat", "water helps", "fluid", "energy", "temperature"],
    "ask-05": ["caffeine", "relax", "breathing", "routine", "schedule", "screen"],
    "ask-06": ["cow", "chicken", "pig", "horse", "sheep", "goat", "duck", "animal"],
    "ask-07": ["fusion", "nuclear", "hydrogen", "energy", "pressure", "gravity"],
    "ask-08": ["breathe", "breathing", "relax", "exercise", "talk", "slow"],
    "heldout-022": ["heapq", "heapify", "heappush", "heappop", "push"],
    "heldout-043": ["96"],
    "heldout-064": ["itself", "calls itself", "function calls", "repeat"],
    "heldout-081": ["antibod", "immune", "nutrient", "first milk", "colostrum is"],
    "heldout-103": ["28"],
    "heldout-126": ["printf", "stdio", "include", "main("],
    "heldout-132": ["curdl", "strain", "acid", "lemon", "separate"],
    "heldout-145": ["extract", "website", "html", "requests", "beautifulsoup", "automat"],
    "heldout-153": ["smell", "touch", "grip", "dust", "water", "trumpet"],
    "heldout-155": ["reverse", "reversed", "string", "def "],
    "heldout-172": ["bitwise", "log2", "binary", "divide", "while"],
    "heldout-186": ["listdir", "glob", "os.walk", "import os", "scandir"],
    "heldout-188": ["water", "spine", "stem", "root", "store", "stomata"],
    "heldout-194": ["seen", "dictionary", "enumerate", "visited", "duplicate is"],
}

# CANNED strings: the memorised replies that carry acceptance without engaging with the request.
# Measured from the sealed reports (Result 7): these are the observed high-frequency replies,
# most of them rejected everywhere but the leading one accepted 8 of 9 times at 29M and 11 of 19
# at 100M. This list is the control arm's input; it must pass ZERO checks.
CANNED = [
    "Hello! How can I help you today?",
    "I'm good, thanks for asking.",
    "I'm a helpful assistant that runs on your computer.",
    "Sure, I can help with that. Here is the information you need:",
    "Hi Dr. Thompson, I'll remember that.",
    "I understand your request and will provide a response that meets the specified constraints.",
    "Okay, I'll remember that.",
    "Congratulations on your upcoming day.",
    "I saw a hard day at the park.",
    "The value of the value is 10.",
    "The sun is hot.",
    "A bee says buzz.",
    "The fall is change color in the fall.",
]


def words(text):
    """chat-grade's `words()`."""
    text = text.replace("\u2019", "'").lower()
    out = []
    for piece in re.split(r"[^\w']|_", text):
        piece = piece.strip("'")
        if piece:
            out.append(piece)
    return out


def contains_phrase(text, phrase):
    return bool(phrase) and any(text[i:i + len(phrase)] == phrase
                                for i in range(len(text) - len(phrase) + 1))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--panel-dir", required=True)
    parser.add_argument("--out-checks", required=True)
    parser.add_argument("--out-provenance", required=True)
    parser.add_argument("--out-canned", required=True)
    parser.add_argument("--ill-posed", default="data/panels/heldout-ill-posed-v3-ids.txt")
    args = parser.parse_args()

    rows = []
    for name in ("everyday-32.json", "heldout-200-a.json", "heldout-200-b.json"):
        rows.extend(json.load(open(os.path.join(args.panel_dir, name))))
    by_id = {row["id"]: row for row in rows}
    ill_posed = set(open(args.ill_posed).read().split())
    canned_words = [words(text) for text in CANNED]

    checks, provenance, problems = [], [], []
    for row_id in sorted(by_id):
        row = by_id[row_id]
        last = words(row["user_turns"][-1])
        if row_id in ill_posed:
            checks.append((row_id, "abstain_exact", "none", "-", "-", "-"))
            provenance.append({
                "id": row_id, "kind": "abstain_exact", "anchor": "-",
                "source": "data/panels/heldout-ill-posed-v3-ids.txt",
                "why": "the request's text does not contain the material it refers to "
                       "(chat-grade's missing_material rule); the check requires an abstention "
                       "and rejects a fabricated specific",
            })
            continue
        anchors = CONTENT_ANCHORS.get(row_id)
        if not anchors:
            continue
        kept = []
        for anchor in anchors:
            phrase = words(anchor)
            if not phrase:
                problems.append(f"{row_id}: anchor {anchor!r} is empty under words()")
                continue
            if contains_phrase(last, phrase):
                problems.append(f"{row_id}: anchor {anchor!r} occurs in the last user turn; dropped")
                continue
            hit = [text for text, text_words in zip(CANNED, canned_words)
                   if contains_phrase(text_words, phrase)]
            if hit:
                problems.append(f"{row_id}: anchor {anchor!r} also occurs in canned reply "
                                f"{hit[0]!r}; dropped")
                continue
            kept.append(anchor)
        if not kept:
            problems.append(f"{row_id}: no anchor survived; row left unchecked")
            continue
        checks.append((row_id, "any", "none", "|".join(kept), "-", "-"))
        provenance.append({
            "id": row_id, "kind": "any", "anchor": "|".join(kept),
            "source": "authored from the request in scripts/reply_panel_checks_build.py "
                      "CONTENT_ANCHORS",
            "why": "content the answer must contain; not present in the request and not present "
                   "in any canned reply, so no single canned string can pass this row",
        })

    with open(args.out_checks, "w") as handle:
        handle.write("# id\tkind\thistory\tterms\tforbid\tkeys -- deterministic row checks for "
                     "the open reply panel (#2029). Checked rows are the ones where a "
                     "judge-free criterion exists: content rows (`any`, required content) and "
                     "the recorded ill-posed rows (`abstain_exact`, which rejects a fabricated "
                     "specific). See docs/labs/reply-panel-deterministic-checks-2026-10-09/.\n")
        for check in checks:
            handle.write("\t".join(check) + "\n")
    with open(args.out_provenance, "w") as handle:
        handle.write("# id\tkind\tanchor\tsource\twhy\n")
        for item in provenance:
            handle.write("\t".join(item[key] for key in ("id", "kind", "anchor", "source", "why"))
                         + "\n")
    with open(args.out_canned, "w") as handle:
        handle.write("\n".join(CANNED) + "\n")

    counts = {}
    for _, kind, *_ in checks:
        counts[kind] = counts.get(kind, 0) + 1
    print(json.dumps({
        "panel_rows": len(rows),
        "checked_rows": len(checks),
        "checked_share": round(len(checks) / len(rows), 4),
        "by_kind": counts,
        "unchecked_rows": len(rows) - len(checks),
        "canned_strings": len(CANNED),
        "authoring_problems": problems,
    }, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main())

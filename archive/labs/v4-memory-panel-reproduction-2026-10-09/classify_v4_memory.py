#!/usr/bin/env python3
"""Classify the v4 memory-panel failures of one chat-grade reply record.

Reimplements chat-grade's frozen `exact` check (crates/uor-r4-training/src/bin/
chat-grade.rs, `words` + `contains_phrase` + `RowCheck::passes`) and names, for
every failing row, which of the check's own failure modes fired:

  expected_present   the reply names the expected value  -> only a key/forbid
                     hit can still fail it (a form/prefix or hedge failure)
  wrong_value        the reply names a forbidden distractor value
  key_confusion      the reply names a word of the distractor's key
  no_value           none of the above and no expected value: the value is
                     simply not in the reply

Usage: classify_v4_memory.py CHECKS.tsv REPLIES.json [--ids-only]
"""
import json
import sys

ABSTAIN_PHRASES = [
    "don't know", "dont know", "do not know", "didn't tell", "did not tell",
    "haven't told", "have not told", "not sure", "no way to know",
    "no way of knowing", "can't know", "cannot know", "can't", "cant",
    "cannot", "can not", "not able", "unable", "no idea", "i wish i could",
    "impossible",
]


def words(text):
    """chat-grade's word rule: lowercase, split off non-alphanumeric except
    apostrophes, trim apostrophes at each end, drop empties."""
    text = text.replace("\u2019", "'").lower()
    out, current = [], ""
    for c in text:
        if c.isalnum() or c == "'":
            current += c
        else:
            if current:
                out.append(current.strip("'"))
            current = ""
    if current:
        out.append(current.strip("'"))
    return [w for w in out if w]


def contains_phrase(text, phrase):
    if not phrase:
        return False
    return any(text[i:i + len(phrase)] == phrase
               for i in range(len(text) - len(phrase) + 1))


def term_list(field):
    return [words(p) for p in field.split("|") if p and p != "-"]


def load_checks(path):
    rows = {}
    with open(path) as handle:
        for line in handle:
            line = line.rstrip("\n")
            if not line.strip() or line.startswith("#"):
                continue
            f = line.split("\t")
            rows[f[0]] = {
                "kind": f[1],
                "terms": term_list(f[3]),
                "forbid": term_list(f[4]) if len(f) > 4 else [],
                "keys": term_list(f[5]) if len(f) > 5 else [],
            }
    return rows


def hits(reply_words, phrases):
    return [p for p in phrases if contains_phrase(reply_words, p)]


def passes(check, reply_words):
    if check["kind"] != "exact":
        return None
    if hits(reply_words, check["forbid"]):
        return False
    if hits(reply_words, check["keys"]):
        return False
    return bool(hits(reply_words, check["terms"]))


def main():
    checks = load_checks(sys.argv[1])
    replies = json.load(open(sys.argv[2]))
    rows = replies["panel"]["rows"]
    tally = {}
    for row in rows:
        if row["category"] != "multi_turn_memory":
            continue
        check = checks[row["id"]]
        reply = row["turns"][-1]["reply"]
        reply_words = words(reply)
        ok = passes(check, reply_words)
        if ok is None:
            continue
        expected = hits(reply_words, check["terms"])
        forbid = hits(reply_words, check["forbid"])
        keys = hits(reply_words, check["keys"])
        abstains = [p for p in ABSTAIN_PHRASES if p in reply.lower()]
        if ok:
            tally["pass"] = tally.get("pass", 0) + 1
            continue
        if expected:
            label = "form_failure_expected_present"
        elif forbid:
            label = "wrong_value"
        elif keys:
            label = "key_confusion"
        else:
            label = "no_value"
        tally[label] = tally.get(label, 0) + 1
        if "--ids-only" not in sys.argv:
            print(f"{row['id']}\t{label}\texpected={check['terms']} "
                  f"forbid={check['forbid']} keys={check['keys']}\n"
                  f"\treply={reply!r}\n"
                  f"\thits expected={expected} forbid={forbid} keys={keys} "
                  f"abstain_phrases={abstains}")
    print()
    for key in sorted(tally):
        print(f"{key}: {tally[key]}")


if __name__ == "__main__":
    main()

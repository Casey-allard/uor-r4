#!/usr/bin/env python3
"""Second-level analysis of a copyfid score summary.

    copyfid-analyse.py --panel PANEL_DIR --score SUMMARY_JSON

The summary already gives per-condition rates. This adds the two things the
verdict needs:

  * which held-out *template* the failures sit in. The panel's held-out form
    index is `TRAIN_FORMS + (relation_index + slot) % HELD_OUT_FORMS`, so the
    template behind every `unseen*` row is recoverable from the panel manifest's
    slot order (which is the relation table order).
  * whether a competing statement makes the arm answer with the distractor
    instead of the asked value, and how that trades against strict hits.
"""

from __future__ import annotations

import argparse
import json
import os
import sys

# Must stay in step with the generator's ASK_TEMPLATES/STATE_TEMPLATES tails.
HELD_OUT_ASKS = [
    "What was that {t} I told you about?",
    "Have you kept a note of my {t}?",
    "My {t} has slipped my mind. What is it?",
    "Did I ever mention my {t} to you?",
    "Tell me again about my {t}.",
]
HELD_OUT_STATES = [
    "For the record, my {t} is {v}.",
    "As I recall, my {t} is {v}.",
    "My {t}, I am quite sure, is {v}.",
    "If I remember correctly, my {t} is {v}.",
    "I am fairly sure my {t} is {v}.",
]
TRAIN_FORMS = 8
HELD_OUT_FORMS = 5


def relation_index(panel_manifest: dict) -> dict:
    """relation id -> index in the relation table (the slot list is that order)."""
    return {slot["id"]: position for position, slot in enumerate(panel_manifest["slots"])}


def held(index: int, slot: int) -> int:
    return TRAIN_FORMS + (index + slot) % HELD_OUT_FORMS


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--panel", required=True)
    parser.add_argument("--score", required=True)
    args = parser.parse_args()

    with open(os.path.join(args.panel, "manifest.json")) as handle:
        manifest = json.load(handle)
    with open(args.score) as handle:
        score = json.load(handle)
    index_of = relation_index(manifest)

    for label, arm in score["arms"].items():
        print(f"== {label}")
        templates = {"unseen-ask": {}, "unseen-state": {}}
        for row_id, row in arm["rows"].items():
            condition = row.get("condition")
            if condition not in templates:
                continue
            relation = row_id.rsplit("-", 1)[-1]
            if relation not in index_of:
                continue
            position = index_of[relation]
            # unseen-ask: canonical statement, ask = held(1)
            # unseen-state: held-out statement held(0), canonical ask
            form = held(position, 1) if condition == "unseen-ask" else held(position, 0)
            key = form - TRAIN_FORMS
            bucket = templates[condition].setdefault(key, {"n": 0, "strict": 0})
            bucket["n"] += 1
            bucket["strict"] += int(bool(row["strict"]))
        for condition, buckets in templates.items():
            table = HELD_OUT_ASKS if condition == "unseen-ask" else HELD_OUT_STATES
            print(f"  {condition} by held-out template:")
            for key in sorted(buckets):
                bucket = buckets[key]
                print(
                    "    form {form} n={n:2d} strict={s:2d} ({rate:.0%})  {template}".format(
                        form=TRAIN_FORMS + key,
                        n=bucket["n"],
                        s=bucket["strict"],
                        rate=bucket["strict"] / bucket["n"],
                        template=table[key],
                    )
                )
        # competition trade-off on the rows that state a distractor
        comp = [r for r in arm["rows"].values() if r.get("condition") == "unseen-comp"]
        if comp:
            both = sum(
                1
                for r in comp
                if r["strict"] and r.get("distractor")
            )
            print(
                "  unseen-comp: strict={s}/{n}, distractor={d}/{n}, both={b}".format(
                    s=sum(int(bool(r["strict"])) for r in comp),
                    d=sum(int(bool(r.get("distractor"))) for r in comp),
                    b=both,
                    n=len(comp),
                )
            )
    return 0


if __name__ == "__main__":
    sys.exit(main())

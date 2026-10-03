#!/usr/bin/env python3
"""Score copy/binding arms on a synthetic-memory panel.

Usage:
    copyfid-score.py --panel PANEL_DIR --arm LABEL=CHAT_JSON --arm ... \
        [--values VALUES_JSON] --out SUMMARY_JSON

`--panel` is a directory holding `requests.json` and `expected.json` in the
format `lut-chat` reads. `--arm` names a `chat.json` produced by `lut-chat
requests=...`. Every arm is scored on every row of the same panel; the score is
the presence of the expected value in the *last* turn's reply.

Two rates are reported for every cell:

  lenient   the expected value appears as a case-insensitive substring.
  strict    the expected value appears as a whole-word phrase -- the value's
            words in order with no word character touching either end, so
            "greenish" does not count for "green" and "nine" does not count
            inside "nineteen".

Degeneracy is reported separately so padding or repetition cannot be read as
copying: `empty`, `truncated` (the sampler hit max_new_tokens), `no_eos`,
`repeat` (a word 4-gram occurs three or more times) and `echo` (five or more
consecutive words shared with the last user turn).

Exit status is 0 even when a row failed to produce a reply: the rates are the
result, and a missing reply is counted as a miss and reported under `missing`.
"""

from __future__ import annotations

import argparse
import json
import math
import os
import re
import sys
from collections import defaultdict

CONDITIONS = [
    "seen",
    "seen-comp",
    "unseen-state",
    "unseen-ask",
    "unseen",
    "unseen-comp",
    "perturb-ask",
]


def parse_condition(row_id: str) -> str:
    """Panel ids are cf-form-<condition>-<relation>."""
    body = row_id[len("cf-form-") :] if row_id.startswith("cf-form-") else row_id
    for condition in ["seen-comp", "unseen-comp", "perturb-ask", "unseen-state", "unseen-ask", "seen", "unseen"]:
        if body.startswith(condition + "-"):
            return condition
    return "panel"


def relation_of(row_id: str) -> str:
    body = row_id[len("cf-form-") :] if row_id.startswith("cf-form-") else row_id
    for condition in ["seen-comp", "unseen-comp", "perturb-ask", "unseen-state", "unseen-ask", "seen", "unseen"]:
        if body.startswith(condition + "-"):
            return body[len(condition) + 1 :]
    return body


def phrase_pattern(value: str) -> re.Pattern:
    words = [re.escape(word) for word in value.split()]
    return re.compile(r"(?<!\w)" + r"\s+".join(words) + r"(?!\w)", re.IGNORECASE)


def wilson(hits: int, n: int, z: float = 1.96) -> tuple[float, float]:
    if n == 0:
        return (0.0, 0.0)
    p = hits / n
    denom = 1.0 + z * z / n
    centre = (p + z * z / (2 * n)) / denom
    half = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / denom
    return (max(0.0, centre - half), min(1.0, centre + half))


def load_replies(chat_path: str) -> dict:
    with open(chat_path) as handle:
        chat = json.load(handle)
    rows = chat.get("record", {}).get("rows", [])
    out = {}
    for row in rows:
        turns = row.get("turns") or []
        if not turns:
            out[row.get("id")] = None
            continue
        last = turns[-1]
        out[row.get("id")] = {
            "reply": last.get("reply") or "",
            "stop": last.get("stop"),
            "model_eos": last.get("model_eos"),
            "user": last.get("user") or "",
            "turns": len(turns),
        }
    return out


def words_of(text: str) -> list[str]:
    return re.findall(r"[A-Za-z0-9']+", text.lower())


def is_degenerate(reply: str, user: str) -> dict:
    words = words_of(reply)
    user_words = words_of(user)
    empty = not reply.strip()
    repeat = False
    if len(words) >= 12:
        counts = defaultdict(int)
        for index in range(len(words) - 3):
            counts[tuple(words[index : index + 4])] += 1
        repeat = any(count >= 3 for count in counts.values())
    if not repeat and len(words) >= 4:
        for index in range(len(words) - 3):
            if len(set(words[index : index + 4])) == 1:
                repeat = True
                break
    echo = False
    if user_words and words:
        run = 0
        best = 0
        for start in range(len(words)):
            for offset in range(len(user_words)):
                length = 0
                while (
                    start + length < len(words)
                    and offset + length < len(user_words)
                    and words[start + length] == user_words[offset + length]
                ):
                    length += 1
                best = max(best, length)
        echo = best >= 5
        run = best
    return {
        "empty": empty,
        "repeat": repeat,
        "echo": echo,
        "echo_run": run if user_words else 0,
        "words": len(words),
    }


def stated_competing(requests: list, values: dict | None) -> dict:
    """row id -> the value stated by the competing (middle) user turn.

    The panel's competing rows put the distractor in the second user turn; the
    value it states is recovered by matching the relation value table against
    that turn, so "did the arm answer with the distractor?" needs no extra
    panel metadata.
    """
    if not values:
        return {}
    known = [value for value_list in values.values() for value in value_list]
    stated = {}
    for row in requests:
        turns = row.get("user_turns") or []
        if len(turns) < 3:
            continue
        middle = turns[1]
        found = [v for v in known if phrase_pattern(v).search(middle)]
        # Exactly one value must be present, or the recovery is ambiguous.
        if len(found) == 1:
            stated[row.get("id")] = found[0]
    return stated


def score_arm(
    label: str, chat_path: str, expected: dict, values: dict | None, stated: dict | None = None
) -> dict:
    # `+`-separated paths merge into one arm: `lut-chat` caps a panel at 128
    # requests, so a larger panel is run as chunks and scored as a union.
    replies: dict = {}
    paths = chat_path.split("+")
    for path in paths:
        replies.update(load_replies(path))
    chat_path = chat_path
    cells: dict[str, dict] = defaultdict(
        lambda: {
            "n": 0,
            "strict": 0,
            "lenient": 0,
            "missing": 0,
            "empty": 0,
            "truncated": 0,
            "no_eos": 0,
            "repeat": 0,
            "echo": 0,
            "words": 0,
            "other_value": 0,
            "distractor": 0,
        }
    )
    per_row = {}
    for row_id, want in expected.items():
        condition = parse_condition(row_id)
        cell = cells[condition]
        cell["n"] += 1
        record = replies.get(row_id)
        if record is None:
            cell["missing"] += 1
            per_row[row_id] = {"strict": False, "lenient": False, "missing": True}
            continue
        reply = record["reply"]
        pattern = phrase_pattern(want)
        lenient = want.lower() in reply.lower()
        strict = bool(pattern.search(reply))
        cell["lenient"] += int(lenient)
        cell["strict"] += int(strict)
        cell["words"] += len(words_of(reply))
        if record["stop"] == "max_new_tokens":
            cell["truncated"] += 1
        if record["model_eos"] is False:
            cell["no_eos"] += 1
        degenerate = is_degenerate(reply, record["user"])
        for key in ("empty", "repeat", "echo"):
            cell[key] += int(degenerate[key])
        asked = relation_of(row_id)
        if values and asked in values:
            others = [v for v in values[asked] if v.lower() != want.lower()]
            if any(v.lower() in reply.lower() for v in others):
                cell["other_value"] += 1
        # The distractor the arm was shown, when the row states one: a reply
        # that contains it copied the wrong fact rather than failing to copy.
        distractor = (stated or {}).get(row_id)
        if distractor and phrase_pattern(distractor).search(reply):
            cell["distractor"] += 1
        per_row[row_id] = {
            "strict": strict,
            "lenient": lenient,
            "reply": reply,
            "condition": condition,
            "wanted": want,
            "distractor": bool(distractor and phrase_pattern(distractor).search(reply)),
        }
    summary = {"label": label, "chat": chat_path, "cells": {}, "rows": per_row}
    keys = [c for c in CONDITIONS if c in cells]
    keys += sorted(c for c in cells if c not in CONDITIONS)
    for condition in keys:
        cell = cells.get(condition)
        if not cell or cell["n"] == 0:
            continue
        row = dict(cell)
        row["strict_rate"] = cell["strict"] / cell["n"]
        row["lenient_rate"] = cell["lenient"] / cell["n"]
        row["strict_ci95"] = list(wilson(cell["strict"], cell["n"]))
        row["lenient_ci95"] = list(wilson(cell["lenient"], cell["n"]))
        row["mean_words"] = cell["words"] / cell["n"]
        for key in (
            "empty",
            "truncated",
            "no_eos",
            "repeat",
            "echo",
            "missing",
            "other_value",
            "distractor",
        ):
            row[key + "_rate"] = cell[key] / cell["n"]
        summary["cells"][condition] = row
    return summary


def mcnemar(a: dict, b: dict) -> dict:
    """Exact McNemar on strict hits over the shared rows of two arms."""
    rows_a = a["rows"]
    rows_b = b["rows"]
    shared = [key for key in rows_a if key in rows_b]
    only_a = sum(1 for key in shared if rows_a[key]["strict"] and not rows_b[key]["strict"])
    only_b = sum(1 for key in shared if rows_b[key]["strict"] and not rows_a[key]["strict"])
    n = only_a + only_b
    if n == 0:
        p = 1.0
    else:
        # two-sided exact binomial with p=0.5
        k = min(only_a, only_b)
        p = sum(math.comb(n, i) for i in range(0, k + 1)) / (2**n) * 2
        p = min(1.0, p)
    return {"shared_rows": len(shared), "only_a": only_a, "only_b": only_b, "p_exact": p}


def markdown(summaries: list[dict], pair: dict | None, values_known: bool) -> str:
    lines = []
    lines.append("| arm | condition | n | strict | strict 95% CI | lenient | other | distractor | empty | trunc | no_eos | repeat | echo |")
    lines.append("|---|---|---|---|---|---|---|---|---|---|---|---|---|")
    for summary in summaries:
        keys = [c for c in CONDITIONS if c in summary["cells"]]
        keys += sorted(k for k in summary["cells"] if k not in CONDITIONS)
        for condition in keys:
            cell = summary["cells"].get(condition)
            if not cell:
                continue
            ci = cell["strict_ci95"]
            lines.append(
                "| {label} | {cond} | {n} | {s}/{n} = {sr:.1%} | [{lo:.1%}, {hi:.1%}] | {l}/{n} = {lr:.1%} | {o:.1%} | {d:.1%} | {e:.1%} | {t:.1%} | {ne:.1%} | {r:.1%} | {ec:.1%} |".format(
                    label=summary["label"],
                    cond=condition,
                    n=cell["n"],
                    s=cell["strict"],
                    sr=cell["strict_rate"],
                    lo=ci[0],
                    hi=ci[1],
                    l=cell["lenient"],
                    lr=cell["lenient_rate"],
                    o=cell["other_value_rate"],
                    d=cell["distractor_rate"],
                    e=cell["empty_rate"],
                    t=cell["truncated_rate"],
                    ne=cell["no_eos_rate"],
                    r=cell["repeat_rate"],
                    ec=cell["echo_rate"],
                )
            )
    if pair:
        lines.append("")
        lines.append(
            "McNemar exact (strict, shared rows): only {a} = {only_a}, only {b} = {only_b}, p = {p:.4g} (n = {n})".format(
                a=pair["a"],
                b=pair["b"],
                only_a=pair["mcnemar"]["only_a"],
                only_b=pair["mcnemar"]["only_b"],
                p=pair["mcnemar"]["p_exact"],
                n=pair["mcnemar"]["shared_rows"],
            )
        )
        lines.append("")
        lines.append("| condition | only {a} | only {b} | p (exact) |".format(a=pair["a"], b=pair["b"]))
        lines.append("|---|---|---|---|")
        for condition, cell in pair["by_condition"].items():
            lines.append(
                "| {cond} | {only_a} | {only_b} | {p:.4g} |".format(
                    cond=condition, only_a=cell["only_a"], only_b=cell["only_b"], p=cell["p_exact"]
                )
            )
    if not values_known:
        lines.append("")
        lines.append("(no `values.json` supplied: `other_value` not measured)")
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--panel", required=True, help="directory with requests.json/expected.json")
    parser.add_argument("--arm", action="append", default=[], help="LABEL=path/to/chat.json")
    parser.add_argument("--values", help="JSON object relation -> [values]")
    parser.add_argument("--out", required=True, help="summary JSON path")
    parser.add_argument("--pair", help="A,B labels for an exact McNemar test")
    args = parser.parse_args()

    expected_path = os.path.join(args.panel, "expected.json")
    with open(expected_path) as handle:
        expected = json.load(handle)
    values = None
    if args.values:
        with open(args.values) as handle:
            loaded = json.load(handle)
        if isinstance(loaded, dict) and "values" in loaded and "schema" in loaded:
            # the panel's own manifest.json: {"values": [{"relation":..,"values":[..]}]}
            values = {entry["relation"]: entry["values"] for entry in loaded["values"]}
        else:
            values = loaded

    summaries = []
    for spec in args.arm:
        label, _, path = spec.partition("=")
        if not path:
            print(f"bad --arm {spec!r}; want LABEL=path", file=sys.stderr)
            return 2
        with open(os.path.join(args.panel, "requests.json")) as handle:
            requests = json.load(handle)
        summaries.append(score_arm(label, path, expected, values, stated_competing(requests, values)))

    pair = None
    if args.pair:
        first, _, second = args.pair.partition(",")
        by_label = {summary["label"]: summary for summary in summaries}
        if first in by_label and second in by_label:
            # Per condition, not just pooled: the headline is the held-out
            # conditions, and a pooled test would be dominated by the trained
            # conditions where both arms are at ceiling.
            conditions = []
            for summary in (by_label[first], by_label[second]):
                for condition in summary["cells"]:
                    if condition not in conditions:
                        conditions.append(condition)
            pair = {
                "a": first,
                "b": second,
                "mcnemar": mcnemar(by_label[first], by_label[second]),
                "by_condition": {
                    condition: mcnemar(
                        {
                            "rows": {
                                key: value
                                for key, value in by_label[first]["rows"].items()
                                if value.get("condition") == condition
                            }
                        },
                        {
                            "rows": {
                                key: value
                                for key, value in by_label[second]["rows"].items()
                                if value.get("condition") == condition
                            }
                        },
                    )
                    for condition in conditions
                },
            }

    table = markdown(summaries, pair, values is not None)
    print(table)
    with open(args.out, "w") as handle:
        json.dump(
            {"panel": args.panel, "expected_rows": len(expected), "arms": {s["label"]: s for s in summaries}, "pair": pair},
            handle,
            indent=1,
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Tabulate Step 0c rehearsal-probe reports (rehearsal_probe.json) as Markdown.

Usage: step0c_tabulate.py LABEL=ROOT [LABEL=ROOT ...]
Reads only the JSON the Rust probe wrote; computes nothing new beyond ratios.
"""
import json
import sys


def cell(tally, key):
    entry = tally.get(key)
    if not entry or not entry["of"]:
        return "-"
    return f"{entry['pass']}/{entry['of']} ({entry['rate']:.2f})"


def main():
    runs = []
    args = sys.argv[1:]
    global full_tables
    full_tables = "--full" in args
    for arg in [a for a in args if a != "--full"]:
        label, root = arg.split("=", 1)
        runs.append((label, json.load(open(f"{root}/rehearsal_probe.json"))))
    for label, report in runs:
        t = report["tally"]
        print(f"\n### {label}: {report['mqar_rows']} MQAR rows, "
              f"wall {report['wall_seconds']:.0f} s\n")
        print("Free run (i)\n")
        print("| scope | judge pass | value appears | rehearse | bare | other |")
        print("|---|---|---|---|---|---|")
        scopes = [("all", "free/all")]
        scopes += [(f"D{d}", f"free/distance/{d}") for d in (16, 64, 200)]
        scopes += [(f"key pieces {k}", f"free/key_pieces/{k}") for k in ("1", "2", "3", "4+")]
        for name, p in scopes:
            if f"{p}/judge_pass" not in t:
                continue
            print(f"| {name} | {cell(t, p + '/judge_pass')} | {cell(t, p + '/value_appears')} | "
                  f"{cell(t, p + '/form_rehearse')} | {cell(t, p + '/form_bare')} | "
                  f"{cell(t, p + '/form_other')} |")
        print("\nFree-run pass by reply form\n")
        print("| form | judge pass | value appears |")
        print("|---|---|---|")
        for form in ("rehearse", "bare", "other"):
            p = f"free/by_form/{form}"
            if f"{p}/judge_pass" in t:
                print(f"| {form} | {cell(t, p + '/judge_pass')} | {cell(t, p + '/value_appears')} |")
        print("\nForced prefixes (ii: `is`; iii: `eq`, `colon`; extra: `is_lc`, `bare` = \"It's\")\n")
        print("| arm | scope | first content piece | content in top-5 | raw first piece | full value | value text |")
        print("|---|---|---|---|---|---|---|")
        for arm in ("is", "is_lc", "eq", "colon", "bare"):
            arm_scopes = [("all", f"{arm}/all")]
            arm_scopes += [(f"D{d}", f"{arm}/distance/{d}") for d in (16, 64, 200)]
            arm_scopes += [(f"key pieces {k}", f"{arm}/key_pieces/{k}") for k in ("1", "2", "3", "4+")]
            arm_scopes += [(f"value pieces {k}", f"{arm}/value_pieces/{k}") for k in ("1", "2", "3+")]
            arm_scopes += [(f"free form {f}", f"{arm}/free_form/{f}") for f in ("rehearse", "bare", "other")]
            if not full_tables:
                arm_scopes = arm_scopes[:4]
            for name, p in arm_scopes:
                if f"{p}/first" not in t:
                    continue
                print(f"| {arm} | {name} | {cell(t, p + '/first_content')} | {cell(t, p + '/content_top5')} | "
                      f"{cell(t, p + '/first')} | {cell(t, p + '/full')} | {cell(t, p + '/text_value')} |")
            unclean = t.get(f"{arm}/boundary_unclean")
            if unclean and unclean["pass"]:
                print(f"| {arm} | boundary unclean | {unclean['pass']}/{unclean['of']} | | | | |")
        # Per exact key piece count (assertion form, " key"), from the rows.
        buckets = {}
        for row in report["rows"]:
            k = row["key_pieces_assertion_form"]
            label_k = str(k) if k < 7 else "7+"
            buckets.setdefault(label_k, []).append(row)
        arms = ("is", "is_lc", "eq", "colon", "bare")
        print("\nFirst content piece by key piece count (assertion form)\n")
        print("| key pieces | rows | " + " | ".join(arms) + " | free judge pass |")
        print("|---|---|" + "---|" * (len(arms) + 1))
        for label_k in sorted(buckets):
            rows = buckets[label_k]
            cells = []
            for arm in arms:
                hits = sum(1 for r in rows if r["forced"].get(arm, {}).get("first_content"))
                cells.append(f"{hits}/{len(rows)}")
            free = sum(1 for r in rows if r["free"]["judge_pass"])
            print(f"| {label_k} | {len(rows)} | " + " | ".join(cells) + f" | {free}/{len(rows)} |")
        # Per value type: a numeral's first content piece is one digit
        # (about 1 in 10 by chance); a word's is a longer piece.
        print("\nFirst content piece and full value by value type\n")
        print("| value type | rows | " + " | ".join(f"{a} first / full" for a in arms) + " | free judge pass |")
        print("|---|---|" + "---|" * (len(arms) + 1))
        for kind in ("numeral", "word"):
            rows = [r for r in report["rows"] if r["value"].isdigit() == (kind == "numeral")]
            cells = []
            for arm in arms:
                first = sum(1 for r in rows if r["forced"].get(arm, {}).get("first_content"))
                full = sum(1 for r in rows if r["forced"].get(arm, {}).get("full"))
                cells.append(f"{first} / {full}")
            free = sum(1 for r in rows if r["free"]["judge_pass"])
            print(f"| {kind} | {len(rows)} | " + " | ".join(cells) + f" | {free}/{len(rows)} |")


if __name__ == "__main__":
    main()

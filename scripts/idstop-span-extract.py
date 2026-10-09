#!/usr/bin/env python3
"""idstop-span-extract: score the serving-time SPAN-EXTRACTION read-out on the
pointer panel and replay it offline over the recorded traces. Companion to
docs/evidence/span-extraction-preregistration-2026-10-09.md, to
docs/evidence/span-extraction-2026-10-09.txt (the result) and to
crates/uor-r4-training/examples/bind-probe-exact.rs (the driver).

The rule (`pointer_span_extract` in the driver, `SpanExtract` in the library)
replaces the reply by the span the pointer itself locked onto, dropping the
prefix the model emitted before it locked. This script is scoring and audit
only: it never decides anything the binary did not already record.

    idstop-span-extract.py [PATH OPTIONS] MODE TAG...

    MODE
      score TAG...     the 2x2, exact/Wilson, the gained and lost cells against
                       the rule-off baseline, exact McNemar and Newcombe 95 %,
                       every changed cell by name, and the template-firing
                       count (a firing whose span is disjoint from the row's
                       recorded value span reproduced the template).
      replay TAG...    the four (mode x anchor) combinations replayed offline
                       from each record's own reply_trace, and checked against
                       the span_extract and reply_ids the binary recorded.
      strict S=N ...   both-ways field comparison: a field only the new record
                       has counts as a difference too.

    TAG is "<name>-f<floor>", name in off|mode1|mode2|mode3, resolving to
    RUNS/<name>/<name>-arm-ptr-f<floor>-<cond>.json for the three conditions.
    mode1 = mid-reply runs, anchor kept (HEADLINE); mode2 = mid-reply runs,
    anchor dropped (FITTED VARIANT); mode3 = any run, anchor kept (broad
    control); off = the rule off at the 72/84 configuration.

    PATH OPTIONS (defaults are the span-extract round's volume layout)
      --work DIR   /workspace/uor-r4/deepseek/term-weight
      --rate DIR   WORK/bindprobe/rate      rows_rate_<cond>.json (the panel)
      --runs DIR   WORK/bindprobe/span-extract  off/, mode1/, mode2/, mode3/
      --grid DIR   WORK/bindprobe/idstop/grid   the sealed floor 0.75/1.0
                                                identity-stop baselines
      --baseline T the rule-off tag every comparison is against (off-f0.5)
"""
import argparse
import importlib.util
import json
import os
import sys

CONDS = ["natural", "sure", "noted"]
MODES = ["off", "mode1", "mode2", "mode3"]
# (mode, anchor) for the offline replay. mode3 is the broad control; the
# fourth combination (any run, anchor dropped) is replayed too, because the
# traces give it for free.
REPLAYS = [("mid", "kept"), ("mid", "dropped"), ("any", "kept"), ("any", "dropped")]


def load_score_module():
    """The sealed scorer's statistics, so this script's numbers use exactly the
    formulas the idstop round used (`wilson`, `newcombe`, `binom_two_sided`)."""
    path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "idstop-score.py")
    spec = importlib.util.spec_from_file_location("idstop_score", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


SCORE = load_score_module()
norm, wilson = SCORE.norm, SCORE.wilson
newcombe, binom_two_sided = SCORE.newcombe, SCORE.binom_two_sided


def configure(args):
    global WORK, RATE, RUNS, GRID, BASELINE
    WORK = args.work
    RATE = args.rate or os.path.join(WORK, "bindprobe", "rate")
    RUNS = args.runs or os.path.join(WORK, "bindprobe", "span-extract")
    GRID = args.grid or os.path.join(WORK, "bindprobe", "idstop", "grid")
    BASELINE = args.baseline
    rows = {}
    for cond in CONDS:
        path = os.path.join(RATE, "rows_rate_%s.json" % cond)
        with open(path) as handle:
            for row in json.load(handle):
                if row.get("kind") == "novel":
                    rows[(cond, row["id"])] = row
    if len(rows) != 84:
        raise SystemExit("the panel is %d novel cells, expected 84" % len(rows))
    return rows


def split_tag(tag):
    name, _, floor = tag.rpartition("-f")
    if name not in MODES or not floor:
        raise SystemExit("tag must be <off|mode1|mode2|mode3>-f<floor>, got %s" % tag)
    return name, floor


def triple(tag, rows):
    """Load a tag's three condition files as {cell: record}."""
    name, floor = split_tag(tag)
    out = {}
    for cond in CONDS:
        stem = os.path.join(RUNS, name, "%s-arm-ptr-f%s" % (name, floor))
        path = next((candidate for candidate in
                     (stem + "-%s.json" % cond, stem + "-ident-%s.json" % cond)
                     if os.path.exists(candidate)), None)
        if path is None:
            raise SystemExit("no file for %s %s under %s" % (tag, cond, os.path.dirname(stem)))
        with open(path) as handle:
            blob = json.load(handle)
        records = {record["id"]: record for record in blob["records"]}
        for (c, rid) in rows:
            if c == cond:
                out[(c, rid)] = records[rid]
    return out


def baseline_triple(rows):
    """The rule-off run the gains and losses are measured against."""
    if BASELINE.startswith("off-"):
        return triple(BASELINE, rows)
    return grid_triple(BASELINE.split("-f")[1], rows)


def grid_triple(floor, rows):
    out = {}
    for cond in CONDS:
        path = os.path.join(GRID, "rate-arm-ptr-f%s-ident-%s.json" % (floor, cond))
        with open(path) as handle:
            blob = json.load(handle)
        records = {record["id"]: record for record in blob["records"]}
        for (c, rid) in rows:
            if c == cond:
                out[(c, rid)] = records[rid]
    return out


def exact(record, row):
    return int(norm(record["reply"]) == row["expected"][0])


def fired(record):
    return record.get("span_extract") is not None


def runs_of(trace, eos):
    """Maximal pointer-locked runs: matches_source and the source advancing by
    one each step. A step with no selection, a step whose emitted id is not the
    window id at its source, and an EOS step all break a run."""
    runs, current = [], None
    for index, step in enumerate(trace):
        source = step.get("source")
        if source is None or not step.get("matches_source") or step["id"] == eos:
            # A non-locked step ENDS the open run; push it before clearing.
            if current is not None:
                runs.append(current)
            current = None
            continue
        if current is not None and source == current["last"] + 1:
            current["last"] = source
        else:
            if current is not None:
                runs.append(current)
            current = {"start_step": index, "first": source, "last": source}
    if current is not None:
        runs.append(current)
    return runs


def replay(record, mode, anchor, min_span=2, eos=1):
    """The rule's own decision, replayed from a record's reply_trace. Mirrors
    `SpanExtract::find` in crates/uor-r4-training/src/geometric_stack.rs."""
    trace = record.get("reply_trace")
    if not trace:
        return None
    best = None
    for run in runs_of(trace, eos):
        if mode == "mid" and run["start_step"] == 0:
            continue
        emitted = run["last"] - run["first"] + (1 if anchor == "kept" else 0)
        if emitted < min_span:
            continue
        if best is None or emitted > best[0]:
            best = (emitted, run)
    if best is None:
        return None
    _, run = best
    steps = trace[run["start_step"]:run["start_step"] + (run["last"] - run["first"] + 1)]
    ids = [step["id"] for step in steps]
    if anchor == "dropped":
        ids = ids[1:]
    return {
        "window_index": run["first"] + (0 if anchor == "kept" else 1),
        "anchor": run["first"],
        "extracted": len(ids),
        "run_start_step": run["start_step"],
        "run_steps": run["last"] - run["first"] + 1,
        "ids": ids,
        "run_end_step": run["start_step"] + run["last"] - run["first"],
    }


def recorded_span(record):
    span = record.get("span_extract")
    if span is None:
        return None
    return {key: span[key] for key in
            ("window_index", "anchor", "extracted", "run_start_step", "run_steps")}


def payload(record):
    return {key: record["span_extract"][key] for key in
            ("window_index", "anchor", "extracted", "run_start_step", "run_steps")}


def cmd_score(args, tags):
    rows = configure(args)
    for tag in tags:
        cells = triple(tag, rows)
        name, floor = split_tag(tag)
        # The comparison is against the same floor's identity-stop run: the
        # rule-off run of this round at 0.5, the sealed grid run above it.
        if floor == "0.5":
            base, base_label = baseline_triple(rows), BASELINE
        else:
            base, base_label = grid_triple(floor, rows), "sealed-grid-f%s-ident" % floor
        base_exact = sum(exact(base[cell], rows[cell]) for cell in rows)
        k = sum(exact(cells[cell], rows[cell]) for cell in rows)
        gained = [cell for cell in rows
                  if exact(cells[cell], rows[cell]) and not exact(base[cell], rows[cell])]
        lost = [cell for cell in rows
                if not exact(cells[cell], rows[cell]) and exact(base[cell], rows[cell])]
        fired_cells = [cell for cell in rows if fired(cells[cell])]
        lo, hi = wilson(k, 84)
        p = binom_two_sided(len(gained), len(lost))
        ci = newcombe(base_exact, 84, k, 84)
        print()
        print("## %s  (baseline %s exact %d/84)" % (tag, base_label, base_exact))
        print("   head: %s" % json.dumps({key: json.load(open(
            os.path.join(RUNS, name, "%s-arm-ptr-f%s-%s.json" % (name, floor, CONDS[0])))
            ).get(key) for key in ("pointer_gate_floor", "pointer_copy_stop",
                                   "pointer_copy_stop_identity", "copy_trace",
                                   "span_extract", "span_extract_min")}))
        print("exact %d/84 = %.4f  Wilson [%.4f, %.4f]  vs %s %d/84  delta %+d"
              % (k, k / 84, lo, hi, base_label, base_exact, k - base_exact))
        print("gained %d: %s" % (len(gained), ", ".join("%s/%s" % c for c in gained) or "none"))
        print("lost   %d: %s" % (len(lost), ", ".join("%s/%s" % c for c in lost) or "none"))
        print("exact McNemar (two-sided, %d vs %d) p=%.4f   Newcombe 95%% [%+.4f, %+.4f]%s"
              % (len(gained), len(lost), p, ci[0], ci[1],
                 "  (contains 0: a better point estimate, NOT an improvement)" if ci[0] <= 0 <= ci[1] else ""))
        print("rule fired on %d/84" % len(fired_cells))
        # Where each firing's emitted span landed, against the row's own
        # recorded value span (scoring only; the rule never sees it).
        inside = outside = partial = starts_in = covers = 0
        for cell in fired_cells:
            record = cells[cell]
            span = set(range(record["span_extract"]["window_index"],
                             record["span_extract"]["window_index"]
                             + record["span_extract"]["extracted"]))
            value = set(record["span_positions"]["expected"])
            if span <= value:
                inside += 1
            elif not (span & value):
                outside += 1
            else:
                partial += 1
            if record["span_extract"]["window_index"] in value:
                starts_in += 1
            if value <= span:
                covers += 1
        print("firing anatomy: emitted span fully inside the value span %d, "
              "starts in the value span %d, covers the value span %d, "
              "partially overlapping %d, DISJOINT (reproduced the template) %d"
              % (inside, starts_in, covers, partial, outside))
        for cell in sorted(fired_cells, key=lambda c: (CONDS.index(c[0]), c[1])):
            record = cells[cell]
            span = set(range(record["span_extract"]["window_index"],
                             record["span_extract"]["window_index"]
                             + record["span_extract"]["extracted"]))
            value = set(record["span_positions"]["expected"])
            where = "inside" if span <= value else ("DISJOINT" if not (span & value) else "partial")
            print("   changed %-22s %-24s -> %-24s expected %-10s span win[%d..%d] %s%s"
                  % ("%s/%s" % cell, repr(base[cell]["reply"]), repr(record["reply"]),
                     rows[cell]["expected"][0], record["span_extract"]["window_index"],
                     record["span_extract"]["window_index"] + record["span_extract"]["extracted"] - 1,
                     where, "  EXACT" if exact(record, rows[cell]) else ""))


def cmd_replay(args, tags):
    rows = configure(args)
    for tag in tags:
        cells = triple(tag, rows)
        name, floor = split_tag(tag)
        # The same floor's identity-stop run, exactly as in `score`: the reply
        # ids of a cell where nothing fired must be that run's ids.
        if floor == "0.5":
            base, base_label = baseline_triple(rows), BASELINE
        else:
            base, base_label = grid_triple(floor, rows), "sealed-grid-f%s-ident" % floor
        print("## %s  (baseline %s)" % (tag, base_label))
        for (mode, anchor) in REPLAYS:
            span_agree = span_differ = id_agree = id_differ = 0
            mismatch = []
            for cell in rows:
                record = cells[cell]
                predicted = replay(record, mode, anchor)
                recorded = recorded_span(record)
                expected_payload = None if predicted is None else {
                    key: predicted[key] for key in
                    ("window_index", "anchor", "extracted", "run_start_step", "run_steps")}
                if expected_payload == recorded:
                    span_agree += 1
                else:
                    span_differ += 1
                    mismatch.append((cell, "span", recorded, expected_payload))
                # The reply ids themselves, and (when nothing fired) the
                # untouched reply of the rule-off run.
                want_ids = base[cell]["reply_ids"] if predicted is None else predicted["ids"]
                if record["reply_ids"] == want_ids:
                    id_agree += 1
                else:
                    id_differ += 1
                    mismatch.append((cell, "reply_ids", record["reply_ids"], want_ids))
            note = ""
            if (mode, anchor) == ("mid", "kept"):
                note = "  <- mode1 (HEADLINE)"
            elif (mode, anchor) == ("mid", "dropped"):
                note = "  <- mode2 (FITTED VARIANT)"
            elif (mode, anchor) == ("any", "kept"):
                note = "  <- mode3 (broad control)"
            print("  replay %-4s %-8s fired %2d/84  span decisions agree %d/84, "
                  "reply ids agree %d/84%s"
                  % (mode, anchor, sum(1 for cell in rows if replay(cells[cell], mode, anchor)),
                     span_agree, id_agree, note))
            for item in mismatch[:5]:
                print("      MISMATCH %s" % (item,))
        # The rule's own reply identity: every fired span must be the window
        # slice it claims, id for id.
        bad = []
        for cell in rows:
            record = cells[cell]
            if not fired(record):
                continue
            span = record["span_extract"]
            window = record["window_ids"]
            start = span["window_index"]
            ids = window[start:start + span["extracted"]]
            if record["reply_ids"] != ids + ([1] if record["reply_eos"] else []):
                bad.append((cell, "reply != window slice"))
        print("  reply == the window slice the span names: %s"
              % ("OK" if not bad else "FAILED %s" % bad[:5]))


def flatten(prefix, value, out):
    if isinstance(value, dict):
        for key, item in value.items():
            flatten(prefix + "." + key, item, out)
    elif isinstance(value, list):
        out[prefix] = json.dumps(value, sort_keys=True)
    else:
        out[prefix] = value


def cmd_strict(args, pairs):
    bad = 0
    for pair in pairs:
        sealed, _, new = pair.partition("=")
        with open(sealed) as handle:
            a = json.load(handle)
        with open(new) as handle:
            b = json.load(handle)
        ar = {record["id"]: record for record in a["records"]}
        br = {record["id"]: record for record in b["records"]}
        differences, added = [], []
        for rid in sorted(set(ar) | set(br)):
            fa, fb = {}, {}
            flatten("", ar.get(rid, {}), fa)
            flatten("", br.get(rid, {}), fb)
            for key in sorted(set(fa) | set(fb)):
                if key not in fb:
                    differences.append((rid, key, "missing in new"))
                elif key not in fa:
                    added.append((rid, key, fb[key]))
                elif fa[key] != fb[key]:
                    differences.append((rid, key, fa[key], fb[key]))
        heads = [key for key in set(a) | set(b) if key != "records" and a.get(key) != b.get(key)]
        print("%s vs %s: %d differing fields, %d fields only the new file has, heads differ in %s"
              % (os.path.basename(sealed), os.path.basename(new), len(differences), len(added),
                 heads or "nothing"))
        for item in (differences + added)[:5]:
            print("   %s" % (item,))
        bad += 1 if differences or added or heads else 0
    print("STRICT-COMPARE %s" % ("PASS" if not bad else "FAIL (%d files)" % bad))
    return 1 if bad else 0


def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--work", default=os.environ.get(
        "IDSTOP_WORK", "/workspace/uor-r4/deepseek/term-weight"))
    parser.add_argument("--rate")
    parser.add_argument("--runs")
    parser.add_argument("--grid")
    parser.add_argument("--baseline", default="off-f0.5")
    parser.add_argument("mode", choices=["score", "replay", "strict"])
    parser.add_argument("tags", nargs="+")
    args = parser.parse_args()
    if args.mode == "score":
        cmd_score(args, args.tags)
        return 0
    if args.mode == "replay":
        cmd_replay(args, args.tags)
        return 0
    return cmd_strict(args, args.tags)


if __name__ == "__main__":
    sys.exit(main())

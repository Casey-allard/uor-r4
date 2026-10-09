#!/usr/bin/env python3
"""idstop: score the token-identity copy stop's 84-cell panel and classify the
cells where it does not fire. Companion to
docs/evidence/idstop_nonfiring_classification_2026-10-09.txt (the idstop3 round:
arm-ptr, gate floor 0.5, identity stop min_span=2) and to
crates/uor-r4-training/examples/bind-probe-exact.rs, the driver that produced
the runs.

    idstop-score.py [PATH OPTIONS] MODE [TAG...]

    MODE
      check              parser self-check against the sealed references
      classify TAG...    the 2x2 (fires x exact) per tag
      simcheck TAG...    replay CopyStopRun over each record's trace and compare
                         the fire decision and the matched span with the record
      report   TAG...    2x2 + where the value's first token landed + the
                         truncation upper bound for any stop rule
      reasons  TAG...    the reason breakdown for the non-firing, non-exact
                         (d) cells, with present-rates

    TAG is either a bare name, resolved against --round/trace, --round/default,
    --grid, --rate, --ptrident (first complete triple wins), or DIR::TAG for an
    explicit directory. A tag names a triple of files
    <DIR>/rate-<tag>-<cond>.json or <DIR>/<tag>-<cond>.json, one per condition
    (natural, sure, noted), 28 novel rows each. The record's `id` is the row key
    and the expected surface form is the row's own `expected[0]` ("vantel"),
    never the record id ("n_vantel"): comparing the reply against " Vantel."
    silently scores every cell zero.

    PATH OPTIONS (defaults are the pod network volume layout of the idstop3
    round; --work defaults to $IDSTOP_WORK)
      --work DIR      /workspace/uor-r4/deepseek/term-weight
      --rate DIR      WORK/bindprobe/rate  rows_rate_<cond>.json and the sealed
                                           arm-ptr runs (35/84)
      --ptrident DIR  WORK/ptrident        the sealed identity-arm runs
                                           (arm-a-w05 10/84, arm-c-ctrl 24/84)
      --grid DIR      WORK/bindprobe/idstop/grid  the previous round's grid
      --round DIR     WORK/bindprobe/idstop3      this round's trace/ and
                                           default/ runs

    Every input lives on the lab's EU-RO-1 network volume rfsx702p68
    (uor-shared-EU-RO-1); none of it is in git. Pass --work (or the individual
    directories) to score a mirror elsewhere. Each input file records its own
    floor, stop, identity switch and copy_trace flag, and `classify` prints
    them.
"""
import argparse
import json
import math
import os
import re
import sys

Z = 1.959963984540054
CONDS = ["natural", "sure", "noted"]
WORK = None
RATE = None
PTRIDENT = None
GRID = None
ROUND = None
PANEL = {}
NOVEL_IDS = []
EXPECTED = {}
CELLS = []
ROWS = {}


def norm(s):
    return re.sub(r"^[^A-Za-z0-9]+|[^A-Za-z0-9]+$", "", s.strip()).lower()


def wilson(k, n, z=Z):
    if n == 0:
        return (0.0, 1.0)
    p = k / n
    d = 1 + z * z / n
    c = p + z * z / (2 * n)
    r = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n))
    return ((c - r) / d, (c + r) / d)


def newcombe(k1, n1, k2, n2):
    l1, u1 = wilson(k1, n1)
    l2, u2 = wilson(k2, n2)
    a = k2 / n2 - k1 / n1
    lo = a - math.sqrt((k2 / n2 - l2) ** 2 + (u1 - k1 / n1) ** 2)
    hi = a + math.sqrt((u2 - k2 / n2) ** 2 + (k1 / n1 - l1) ** 2)
    return (lo, hi)


def binom_two_sided(b, c):
    n = b + c
    if n == 0:
        return 1.0
    k = min(b, c)
    return min(1.0, 2 * sum(math.comb(n, i) for i in range(0, k + 1)) / 2 ** n)


def mean(xs):
    xs = [x for x in xs if x is not None]
    return sum(xs) / len(xs) if xs else float("nan")


def configure(args):
    """Resolve the path options and load the 84-cell panel."""
    global WORK, RATE, PTRIDENT, GRID, ROUND, PANEL, NOVEL_IDS, EXPECTED, CELLS, ROWS
    WORK = args.work
    RATE = args.rate or os.path.join(WORK, "bindprobe", "rate")
    PTRIDENT = args.ptrident or os.path.join(WORK, "ptrident")
    GRID = args.grid or os.path.join(WORK, "bindprobe", "idstop", "grid")
    ROUND = args.round or os.path.join(WORK, "bindprobe", "idstop3")
    PANEL = {}
    for cond in CONDS:
        path = os.path.join(RATE, "rows_rate_%s.json" % cond)
        try:
            rows = json.load(open(path))
        except FileNotFoundError:
            raise SystemExit(
                "no rows at %s: pass --work (or --rate) for the volume that holds "
                "the idstop panel, e.g. --work /workspace/uor-r4/deepseek/term-weight "
                "on the pod, or a mirror of it" % path)
        PANEL[cond] = [r for r in rows if r.get("kind") == "novel"]
    NOVEL_IDS = sorted(r["id"] for r in PANEL[CONDS[0]])
    for cond in CONDS:
        if sorted(r["id"] for r in PANEL[cond]) != NOVEL_IDS:
            raise SystemExit("condition %s has a different novel panel" % cond)
    EXPECTED = {r["id"]: r["expected"][0] for r in PANEL[CONDS[0]]}
    ROWS = {r["id"]: r for r in PANEL[CONDS[0]]}
    CELLS = [(c, i) for c in CONDS for i in NOVEL_IDS]
    if len(CELLS) != 84:
        raise SystemExit("the panel is %d cells, expected 84" % len(CELLS))


def resolve(tag):
    """A tag's directory: explicit DIR::TAG, else the first default directory
    that holds the whole triple."""
    if "::" in tag:
        where, name = tag.split("::", 1)
        return where, name
    candidates = [os.path.join(ROUND, "trace"), os.path.join(ROUND, "default"),
                  GRID, RATE, PTRIDENT]
    for where in candidates:
        if all(find(where, tag, cond) for cond in CONDS):
            return where, tag
    raise SystemExit("no complete triple for tag %s in any of %s" % (tag, candidates))


def find(directory, tag, cond):
    for pattern in ("%s/rate-%s-%s.json", "%s/%s-%s.json"):
        path = pattern % (directory, tag, cond)
        if os.path.exists(path):
            return path
    return None


def score(directory, tag):
    out = {"exact": {}, "present": {}, "recs": {}, "head": None}
    for cond in CONDS:
        path = find(directory, tag, cond)
        if path is None:
            raise SystemExit("no file for tag %s cond %s in %s" % (tag, cond, directory))
        blob = json.load(open(path))
        out["head"] = out["head"] or {k: blob.get(k) for k in
                                      ("schema", "model", "protocol", "max_new_tokens",
                                       "pointer_gate_floor", "pointer_copy_stop",
                                       "pointer_copy_stop_identity", "copy_trace")}
        records = {r["id"]: r for r in blob["records"]}
        missing = [i for i in NOVEL_IDS if i not in records]
        if missing:
            raise SystemExit("%s misses %s" % (path, missing))
        for i in NOVEL_IDS:
            r = records[i]
            exp = EXPECTED[i]
            reply = r["reply"]
            out["exact"][(cond, i)] = int(norm(reply) == exp)
            out["present"][(cond, i)] = int(exp in reply.strip().lower())
            out["recs"][(cond, i)] = r
    return out


def pooled(d):
    return sum(d["exact"].values())


def check():
    """The parser must reproduce the known sealed numbers before anything new
    is believed."""
    known = [
        (RATE, "arm-ptr", 35, (0.5149, 0.8351, 0.5610, 0.4901)),
        (PTRIDENT, "rate-arm-a-w05", 10, None),
        (PTRIDENT, "rate-arm-c-ctrl", 24, None),
    ]
    bad = 0
    for directory, tag, expected_exact, channels in known:
        d = score(directory, tag)
        k = pooled(d)
        got = (
            mean([d["recs"][c]["pointer"]["gate"] if d["recs"][c]["pointer"] else None for c in CELLS]),
            mean([d["recs"][c]["pointer"]["expected"] if d["recs"][c]["pointer"] else None for c in CELLS]),
            mean([d["recs"][c]["evidence"]["read_expected_max"] for c in CELLS]),
            mean([d["recs"][c]["evidence"]["out_expected"] for c in CELLS]),
        )
        ok = k == expected_exact
        if channels:
            ok = ok and all(abs(a - b) < 5e-5 for a, b in zip(got, channels))
        bad += 0 if ok else 1
        print("%s %-14s exact %2d/84 (expected %2d)  channels %.4f/%.4f/%.4f/%.4f%s"
              % ("OK  " if ok else "WRONG", tag, k, expected_exact, got[0], got[1], got[2], got[3],
                 "" if not channels else "  (expected %.4f/%.4f/%.4f/%.4f)" % channels))
    print("PARSER-CHECK %s" % ("PASS" if bad == 0 else "FAIL"))
    return 1 if bad else 0


# ---------------------------------------------------------------- the 2x2
def fired(r):
    return r.get("copy_stop") is not None


def classify(directory, tag):
    d = score(directory, tag)
    cells = {"a": [], "b": [], "c": [], "d": []}
    for c, i in CELLS:
        r = d["recs"][(c, i)]
        ex = d["exact"][(c, i)]
        f = fired(r)
        cells[("a" if f else "c") if ex else ("b" if f else "d")].append((c, i))
    k = pooled(d)
    lo, hi = wilson(k, 84)
    print("## %s  (head: %s)" % (tag, json.dumps(d["head"])))
    print("exact %d/84 = %.4f  Wilson [%.4f, %.4f]" % (k, k / 84, lo, hi))
    for key, name in (("a", "fires + exact"), ("b", "fires + NOT exact"),
                      ("c", "no fire + exact"), ("d", "no fire + NOT exact")):
        n = len(cells[key])
        print("  %s %s: %d/84 = %.4f   %s" % (key, name, n, n / 84,
              ", ".join("%s/%s" % cell for cell in cells[key])))
    fired_n = len(cells["a"]) + len(cells["b"])
    print("  fired %d/84, exact among fired %d, present %d/84, mean gate %.4f, mean ptr %.4f, mean pmix %.4f"
          % (fired_n, len(cells["a"]), sum(d["present"].values()),
             mean([d["recs"][c]["pointer"]["gate"] if d["recs"][c]["pointer"] else None for c in CELLS]),
             mean([d["recs"][c]["pointer"]["expected"] if d["recs"][c]["pointer"] else None for c in CELLS]),
             mean([d["recs"][c]["evidence"]["out_expected"] for c in CELLS])))
    return d, cells


# ------------------------------------------------- reason breakdown for (d)
def value_ids(r):
    """The value's own token ids in the window, in position order."""
    span = sorted(r["span_positions"]["expected"])
    return [r["window_ids"][p] for p in span], span


def short_cycle(tokens, repeats=3):
    """reference_eval::short_cycle_period_with, verbatim."""
    if repeats < 2:
        return None
    for period in range(1, 5):
        span = period * repeats
        if len(tokens) < span:
            continue
        tail = tokens[len(tokens) - span:]
        if all(tail[i:i + period] == tail[:period]
               for i in range(period, span, period)):
            return period
    return None


def emitted_sequence(r):
    """Every id the decoder emitted: the reply's own ids plus the id the
    identity rule dropped, which the trace does not keep a step for."""
    ids = list(r["reply_ids"])
    cs = r.get("copy_stop")
    if cs and cs.get("dropped") is not None:
        ids = ids + [cs["dropped"]]
    return ids


def simulate(r, min_span=2, eos=None):
    """Re-run the identity rule over the record's emitted ids, exactly as
    `greedy_reply_with_copy_stop` does: the EOS and short-cycle stops are
    checked BEFORE the copy stop at every step, and a step the copy stop ends
    on is dropped from the reply (so it has no trace step).

    Returns (fire, runs, cur): whether the rule fired, every run it opened
    (start, len, break step, whether the span lies inside the value's own
    positions) and the run still open at the end.
    """
    window = list(r["window_ids"])
    trace = r.get("reply_trace") or []
    emitted = emitted_sequence(r)
    span = set(r["span_positions"]["expected"])
    # The reply's own last id is the EOS id exactly when it ended on EOS.
    eos = r["reply_ids"][-1] if r.get("reply_eos") else None
    runs = []
    cur = None
    fire = False
    reply_ids = []
    end = "cap"
    for k, eid in enumerate(emitted):
        sel = trace[k] if k < len(trace) else None
        window.append(eid)
        reply_ids.append(eid)
        if sel is not None:
            # Reply::stop_with runs first: EOS, then a short terminal cycle.
            if eid == eos:
                end = "eos"
                break
            if short_cycle(reply_ids, 3) is not None:
                end = "cycle"
                break
        if cur is None:
            if sel is not None and sel["source_id"] == eid:
                cur = {"start": sel["source"], "len": 1, "break_step": None}
        else:
            nxt = cur["start"] + cur["len"]
            if nxt < len(window) and window[nxt] == eid:
                cur["len"] += 1
            else:
                cur["break_step"] = k
                runs.append(cur)
                if cur["len"] >= min_span:
                    fire = True
                    end = "fire"
                    cur = None
                    break
                cur = None
                if sel is not None and sel["source_id"] == eid:
                    cur = {"start": sel["source"], "len": 1, "break_step": None}
    if cur is not None:
        runs.append(cur)
    for run in runs:
        run["in_value"] = all(p in span
                              for p in range(run["start"], run["start"] + run["len"]))
        run["overlaps_value"] = any(p in span
                                    for p in range(run["start"], run["start"] + run["len"]))
        run["start_in_value"] = run["start"] in span
    return fire, runs, cur, end


def reason_of(r, d, cell):
    """One reason code per non-firing, non-exact cell, plus its evidence. The
    code is v0_class's: where the value's first token landed and what the rule's
    state was at that step."""
    vids, span = value_ids(r)
    v0 = vids[0] if vids else None
    trace = r.get("reply_trace") or []
    emit_v0 = [s for s in trace if s["id"] == v0]
    v0_open = [s for s in emit_v0 if s["source_id"] == s["id"] and s["source"] in span]
    states, run_at_end, end = trace_states(r)
    why = v0_class(states, span)
    fire, runs, cur, sim_end = simulate(r)
    in_value = [x for x in runs if x["start_in_value"]]
    best = max([x["len"] for x in in_value], default=0)
    if end == "eos" and run_at_end is not None:
        why += "; EOS ended the reply while a run was alive (length %d)" % run_at_end["len"]
    return {
        "cell": cell, "why": why, "present": d["present"][cell],
        "v0": v0, "v0_emits": len(emit_v0), "v0_opens": len(v0_open),
        "runs": runs, "in_value": in_value, "best_in_value": best,
        "open_any": len(runs), "end": end,
        "src_ids_at_v0": [s["source_id"] for s in emit_v0],
        "src_at_v0": [s["source"] for s in emit_v0],
        "span": span, "reply": r["reply"].replace("\n", "\\n"),
        "trace": [(s["step"], s["id"], s["source"], s["source_id"], s["matches_source"])
                  for s in trace],
    }


def reasons(directory, tag):
    d = score(directory, tag)
    print("## reason breakdown for the (d) cells: %s" % tag)
    print("")
    tally = {}
    details = []
    present_d = 0
    for c, i in CELLS:
        r = d["recs"][(c, i)]
        if fired(r) or d["exact"][(c, i)]:
            continue
        x = reason_of(r, d, (c, i))
        tally[x["why"].split(";")[0]] = tally.get(x["why"].split(";")[0], 0) + 1
        present_d += x["present"]
        details.append(x)
    nd = len(details)
    print("(d) cells: %d; value text PRESENT in the reply: %d/%d = %.4f"
          % (nd, present_d, nd, present_d / nd if nd else float("nan")))
    for why, n in sorted(tally.items(), key=lambda kv: -kv[1]):
        print("  %-46s %d" % (why, n))
    print("")
    print("| cell | present | reason | v0 emits | v0 opens in value | runs opened | best in-value run | end | src_id at v0 emits | reply |")
    print("|---|---|---|---|---|---|---|---|---|---|")
    for x in details:
        print("| %(cell)s | %(present)d | %(why)s | %(v0_emits)d | %(v0_opens)d | %(open_any)d | %(best_in_value)d | %(end)s | %(src_ids_at_v0)s | `%(reply)s` |" % x)
    print("")
    for x in details:
        print("### %s  present=%d  span=%s" % (x["cell"], x["present"], x["span"]))
        print("runs: %s" % [(r["start"], r["len"], r["start_in_value"], r["break_step"]) for r in x["runs"]])
        print("trace (step, id, source, source_id, matches_source): %s" % (x["trace"],))
        print("reply: `%s`" % x["reply"])
    return d, details


def fire_check(directory, tag):
    """Validate the offline simulator against the recorded copy_stop: the fire
    decision on every cell and, where it fired, the span it reports."""
    d = score(directory, tag)
    ok = bad = span_ok = span_bad = 0
    for c, i in CELLS:
        r = d["recs"][(c, i)]
        sim, runs, cur, end = simulate(r)
        rec = fired(r)
        if sim != rec:
            bad += 1
            print("  MISMATCH %s/%s sim=%s recorded=%s" % (c, i, sim, rec))
            continue
        ok += 1
        if rec:
            reported = r["copy_stop"]
            last = runs[-1] if runs else None
            if last and last["start"] == reported["window_index"] and last["len"] == reported["copied"]:
                span_ok += 1
            else:
                span_bad += 1
                print("  SPAN-MISMATCH %s/%s sim=%s recorded=(%s,%d)"
                      % (c, i, last, reported["window_index"], reported["copied"]))
    print("SIM-CHECK %s: fire %d agree / %d differ; fired-span %d agree / %d differ"
          % ("PASS" if bad == 0 and span_bad == 0 else "FAIL", ok, bad, span_ok, span_bad))
    return bad == 0 and span_bad == 0


def truncatable(r, expected):
    """Upper bound for ANY rule that ends the reply at some step: the reply is
    a concatenation of id bytes, so ending the reply at a step yields a text
    prefix of it (EOS excluded, as the probe's own decode does). True when some
    prefix normalises to exactly the expected value."""
    reply = r["reply"]
    return any(norm(reply[:k]) == expected for k in range(len(reply) + 1))


def trace_states(r, min_span=2):
    """The rule's state at every emitted id: whether a run was already open
    when the id arrived, what the pointer's source held, and how the step
    ended. Same loop as `simulate`, instrumented."""
    window = list(r["window_ids"])
    trace = r.get("reply_trace") or []
    emitted = emitted_sequence(r)
    span = set(r["span_positions"]["expected"])
    vids, _ = value_ids(r)
    v0 = vids[0] if vids else None
    eos = r["reply_ids"][-1] if r.get("reply_eos") else None
    states = []
    cur = None
    reply_ids = []
    end = "cap"
    for k, eid in enumerate(emitted):
        sel = trace[k] if k < len(trace) else None
        state = {
            "step": k, "id": eid, "is_v0": eid == v0, "in_value": eid in vids,
            "source": sel["source"] if sel else None,
            "source_id": sel["source_id"] if sel else None,
            "match": bool(sel and sel["source_id"] == eid),
            "source_in_span": bool(sel and sel["source"] in span),
            "run_before": (cur["start"], cur["len"]) if cur else None,
            "run_after": None, "opened": None, "extended": False,
            "broke": None, "fired": False, "end": None,
        }
        window.append(eid)
        reply_ids.append(eid)
        if sel is not None:
            if eid == eos:
                end = "eos"
                state["end"] = "eos"
                states.append(state)
                break
            if short_cycle(reply_ids, 3) is not None:
                end = "cycle"
                state["end"] = "cycle"
                states.append(state)
                break
        if cur is None:
            if sel is not None and sel["source_id"] == eid:
                cur = {"start": sel["source"], "len": 1, "break_step": None}
                state["opened"] = cur["start"]
        else:
            nxt = cur["start"] + cur["len"]
            if nxt < len(window) and window[nxt] == eid:
                cur["len"] += 1
                state["extended"] = True
            else:
                state["broke"] = (cur["start"], cur["len"])
                if cur["len"] >= min_span:
                    cur = None
                    end = "fire"
                    state["fired"] = True
                    state["end"] = "fire"
                    states.append(state)
                    break
                cur = None
                if sel is not None and sel["source_id"] == eid:
                    cur = {"start": sel["source"], "len": 1, "break_step": None}
                    state["opened"] = cur["start"]
        state["run_after"] = (cur["start"], cur["len"]) if cur else None
        states.append(state)
    return states, cur, end


def v0_class(states, span):
    """Where the value's first emitted token landed, in four codes."""
    first_v0 = next((s for s in states if s["is_v0"]), None)
    if first_v0 is None:
        return "A v0 never emitted"
    if first_v0["match"] and first_v0["source_in_span"] and first_v0["run_before"] is None:
        return "B v0 emitted, pointer on the value, no run open (open available)"
    if first_v0["match"] and first_v0["source_in_span"]:
        return "C v0 emitted, pointer on the value, a run already open"
    if first_v0["match"]:
        return "D v0 emitted, source id matches but outside the value span"
    if first_v0["run_before"] is not None:
        return "E v0 emitted, run open, pointer elsewhere"
    return "F v0 emitted, no run open, pointer elsewhere (pointer wrong)"


def report(directory, tag, min_span=2):
    d = score(directory, tag)
    cells = {"a": [], "b": [], "c": [], "d": []}
    for c, i in CELLS:
        r = d["recs"][(c, i)]
        ex = d["exact"][(c, i)]
        f = fired(r)
        cells[("a" if f else "c") if ex else ("b" if f else "d")].append((c, i))
    print("## %s" % tag)
    print("exact %d/84; 2x2: fires+exact %d, fires+NOT %d, nofire+exact %d, nofire+NOT %d"
          % (pooled(d), len(cells["a"]), len(cells["b"]), len(cells["c"]), len(cells["d"])))
    print("present %d/84" % sum(d["present"].values()))
    print("")
    fired_cells = cells["a"] + cells["b"]
    inside = starts_in = outside = 0
    for c, i in fired_cells:
        r = d["recs"][(c, i)]
        rep = r["copy_stop"]
        span = set(r["span_positions"]["expected"])
        pos = list(range(rep["window_index"], rep["window_index"] + rep["copied"]))
        if pos and all(p in span for p in pos):
            inside += 1
        if rep["window_index"] in span:
            starts_in += 1
        if not any(p in span for p in pos):
            outside += 1
    print("FIRINGS %d/84: span fully inside the value span %d, starts in the value span %d,"
          " fully outside %d" % (len(fired_cells), inside, starts_in, outside))
    print("")
    tally = {}
    anchored_cells = []
    trunc_recoverable = []
    d_trunc = 0
    for c, i in CELLS:
        r = d["recs"][(c, i)]
        states, cur, end = trace_states(r, min_span)
        vids, span = value_ids(r)
        code = v0_class(states, span)
        tally[code] = tally.get(code, 0) + 1
        if any(s["run_before"] and s["is_v0"] and s["run_before"][0] < span[0] for s in states):
            anchored_cells.append((c, i))
        if not d["exact"][(c, i)]:
            if truncatable(r, EXPECTED[i]):
                trunc_recoverable.append((c, i))
                if (c, i) in cells["d"]:
                    d_trunc += 1
    for code, n in sorted(tally.items()):
        print("  %-64s %d/84" % (code, n))
    print("")
    print("a run anchored BEFORE the value span reached the value's first token: %d/84" % len(anchored_cells))
    print("not exact but exact after SOME prefix truncation (upper bound for any stop rule): %d/84 %s"
          % (len(trunc_recoverable), trunc_recoverable))
    print("")
    dd = cells["d"]
    print("(d) cells: %d; present %d/%d; exact after some prefix truncation %d/%d"
          % (len(dd), sum(d["present"][x] for x in dd), len(dd), d_trunc, len(dd)))
    print("")
    print("| cell | present | class | v0 step | v0 source | v0 source_id | v0 in span | run open before | run at end | end | reply |")
    print("|---|---|---|---|---|---|---|---|---|---|---|")
    for c, i in dd:
        r = d["recs"][(c, i)]
        states, cur, end = trace_states(r, min_span)
        vids, span = value_ids(r)
        code = v0_class(states, span)[0]
        f = next((s for s in states if s["is_v0"]), None)
        print("| %s/%s | %d | %s | %s | %s | %s | %s | %s | %s | %s | `%s` |"
              % (c, i, d["present"][(c, i)], code,
                 f["step"] if f else "-", f["source"] if f else "-",
                 f["source_id"] if f else "-", f["source_in_span"] if f else "-",
                 f["run_before"] if f else "-", cur, end,
                 r["reply"].replace("\n", "\\n")))
    return d, cells


def main():
    parser = argparse.ArgumentParser(
        description="idstop: score and classify the token-identity copy stop's 84-cell panel",
        formatter_class=argparse.RawDescriptionHelpFormatter, epilog=__doc__)
    parser.add_argument("--work", default=os.environ.get(
        "IDSTOP_WORK", "/workspace/uor-r4/deepseek/term-weight"),
        help="root of the term-weight artifacts (default $IDSTOP_WORK or the pod volume path)")
    parser.add_argument("--rate", help="default WORK/bindprobe/rate")
    parser.add_argument("--ptrident", help="default WORK/ptrident")
    parser.add_argument("--grid", help="default WORK/bindprobe/idstop/grid")
    parser.add_argument("--round", help="default WORK/bindprobe/idstop3")
    parser.add_argument("mode", choices=["check", "classify", "simcheck", "report", "reasons"])
    parser.add_argument("tags", nargs="*", help="TAG or DIR::TAG")
    args = parser.parse_args()
    configure(args)
    if args.mode == "check":
        sys.exit(check())
    if not args.tags:
        parser.error("%s needs at least one tag" % args.mode)
    for argument in args.tags:
        where, tag = resolve(argument)
        if args.mode == "classify":
            classify(where, tag)
        elif args.mode == "simcheck":
            fire_check(where, tag)
        elif args.mode == "report":
            report(where, tag)
        else:
            reasons(where, tag)
    sys.exit(0)


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""idstop: the cross-checks around the 2x2 classification of the token-identity
copy stop at arm-ptr / gate floor 0.5 -- the class x 2x2 crosstab, whether the
non-firing cells keep the same-floor reply without the stop, the anatomy of the
23 firings against the value's own span, and the (d) cells in three floors of
context. Companion to docs/evidence/idstop_nonfiring_classification_2026-10-09.txt.

    idstop-crosschecks.py [PATH OPTIONS] [TAG OPTIONS]

    PATH OPTIONS are idstop-score.py's: --work, --rate, --ptrident, --grid,
    --round (defaults to the pod volume layout, --work default $IDSTOP_WORK).
    TAG OPTIONS
      --trace-tag TAG   the run WITH the stop and copy_trace=1
                        (default trace-arm-ptr-f0.5-ident)
      --grid-tag TAG    the same floor with the stop OFF
                        (default rate-arm-ptr-f0.5-none)
      --floor0-tag TAG  the unfloored sealed reference (default arm-ptr)

    Inputs live on the lab's EU-RO-1 network volume rfsx702p68
    (uor-shared-EU-RO-1), under bindprobe/idstop3/trace/ (this round),
    bindprobe/idstop/grid/ (the previous round) and bindprobe/rate/ (sealed).
"""
import argparse
import importlib.util
import os
import sys
from collections import Counter
from pathlib import Path

_SPEC = importlib.util.spec_from_file_location(
    "idstop_score", Path(__file__).with_name("idstop-score.py"))
sc = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(sc)


def main():
    parser = argparse.ArgumentParser(
        description="idstop: cross-checks around the 2x2 classification",
        formatter_class=argparse.RawDescriptionHelpFormatter, epilog=__doc__)
    parser.add_argument("--work", default=os.environ.get(
        "IDSTOP_WORK", "/workspace/uor-r4/deepseek/term-weight"))
    parser.add_argument("--rate")
    parser.add_argument("--ptrident")
    parser.add_argument("--grid")
    parser.add_argument("--round")
    parser.add_argument("--trace-tag", default="trace-arm-ptr-f0.5-ident")
    parser.add_argument("--grid-tag", default="rate-arm-ptr-f0.5-none")
    parser.add_argument("--floor0-tag", default="arm-ptr")
    args = parser.parse_args()
    sc.configure(args)
    trace = sc.score(*sc.resolve(args.trace_tag))
    nostop = sc.score(*sc.resolve(args.grid_tag))
    floor0 = sc.score(*sc.resolve(args.floor0_tag))

    cross = {}
    for c, i in sc.CELLS:
        r = trace["recs"][(c, i)]
        states, cur, end = sc.trace_states(r)
        vids, span = sc.value_ids(r)
        code = sc.v0_class(states, span)[0]
        cls = (("a" if sc.fired(r) else "c") if trace["exact"][(c, i)]
               else ("b" if sc.fired(r) else "d"))
        cross.setdefault(code, {}).setdefault(cls, []).append("%s/%s" % (c, i))
    print("CLASS x 2x2 (%s):" % args.trace_tag)
    for code in sorted(cross):
        print("  %s: %s" % (code, {k: len(v) for k, v in sorted(cross[code].items())}))
        for k in sorted(cross[code]):
            print("      %s: %s" % (k, ", ".join(cross[code][k])))

    same = diff = 0
    diffs = []
    for c, i in sc.CELLS:
        if sc.fired(trace["recs"][(c, i)]):
            continue
        if trace["recs"][(c, i)]["reply"] == nostop["recs"][(c, i)]["reply"]:
            same += 1
        else:
            diff += 1
            diffs.append("%s/%s" % (c, i))
    print("\nNON-FIRING cells with the same reply as the same floor without the stop:"
          " %d, differing: %d %s" % (same, diff, diffs))

    print("\nFIRINGS (%s):" % args.trace_tag)
    for c, i in sc.CELLS:
        r = trace["recs"][(c, i)]
        if not sc.fired(r):
            continue
        rep = r["copy_stop"]
        span = set(r["span_positions"]["expected"])
        pos = list(range(rep["window_index"], rep["window_index"] + rep["copied"]))
        tag = ("INSIDE" if all(p in span for p in pos) else
               "OUTSIDE" if not any(p in span for p in pos) else "PARTIAL")
        print("  %-10s %-16s span=%-14s value=%-24s copied=%d emitted=%d drop=%-5s"
              " exact=%d reply=%r"
              % (c, i, tag, pos, rep["copied"], rep["emitted"], rep["dropped"],
                 trace["exact"][(c, i)], r["reply"]))

    print("\n(d) cells in context (stop | no stop at the same floor | floor 0):")
    for c, i in sc.CELLS:
        r = trace["recs"][(c, i)]
        if sc.fired(r) or trace["exact"][(c, i)]:
            continue
        print("  %-10s %-16s stop=%r | nostop=%r | f0=%r"
              % (c, i, r["reply"], nostop["recs"][(c, i)]["reply"],
                 floor0["recs"][(c, i)]["reply"]))

    ended = Counter()
    for c, i in sc.CELLS:
        states, cur, end = sc.trace_states(trace["recs"][(c, i)])
        ended[(end, "run_alive" if cur else "no_run")] += 1
    print("\nHOW THE REPLY ENDED: %s" % dict(ended))
    return 0


if __name__ == "__main__":
    sys.exit(main())

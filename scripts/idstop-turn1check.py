#!/usr/bin/env python3
"""idstop: how often the identity copy stop acts on TURN 1's reply, which
changes turn 2's window for the cells where the stop does not fire at turn 2.
The probe generates turn 1 greedily when the panel row pins none, so the rule
runs there too; this script counts the cells whose turn-1 ids differ with the
rule on, and prints the stop / no-stop / floor-0 replies of those cells.
Companion to docs/evidence/idstop_nonfiring_classification_2026-10-09.txt.

    idstop-turn1check.py [PATH OPTIONS] [TAG OPTIONS]

    PATH OPTIONS are idstop-score.py's: --work, --rate, --ptrident, --grid,
    --round (defaults to the pod volume layout, --work default $IDSTOP_WORK).
    TAG OPTIONS
      --trace-tag TAG   the run WITH the stop and copy_trace=1
                        (default trace-arm-ptr-f0.5-ident)
      --grid-tag TAG    the same floor with the stop OFF
                        (default rate-arm-ptr-f0.5-none)

    Inputs live on the lab's EU-RO-1 network volume rfsx702p68
    (uor-shared-EU-RO-1), under bindprobe/idstop3/trace/ (this round) and
    bindprobe/idstop/grid/ (the previous round).
"""
import argparse
import importlib.util
import os
import sys
from pathlib import Path

_SPEC = importlib.util.spec_from_file_location(
    "idstop_score", Path(__file__).with_name("idstop-score.py"))
sc = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(sc)


def main():
    parser = argparse.ArgumentParser(
        description="idstop: turn-1 window changes caused by the copy stop",
        formatter_class=argparse.RawDescriptionHelpFormatter, epilog=__doc__)
    parser.add_argument("--work", default=os.environ.get(
        "IDSTOP_WORK", "/workspace/uor-r4/deepseek/term-weight"))
    parser.add_argument("--rate")
    parser.add_argument("--ptrident")
    parser.add_argument("--grid")
    parser.add_argument("--round")
    parser.add_argument("--trace-tag", default="trace-arm-ptr-f0.5-ident")
    parser.add_argument("--grid-tag", default="rate-arm-ptr-f0.5-none")
    args = parser.parse_args()
    sc.configure(args)
    trace = sc.score(*sc.resolve(args.trace_tag))
    nostop = sc.score(*sc.resolve(args.grid_tag))

    non_firing = [(c, i) for c, i in sc.CELLS if not sc.fired(trace["recs"][(c, i)])]
    changed = [(c, i) for c, i in non_firing
               if trace["recs"][(c, i)]["turn1_ids"] != nostop["recs"][(c, i)]["turn1_ids"]]
    print("non-firing cells: %d; of them with a different turn-1 reply"
          " (turn 2's window changed by the rule): %d" % (len(non_firing), len(changed)))
    for c, i in changed:
        a, b = trace["recs"][(c, i)], nostop["recs"][(c, i)]
        print("  %-10s %-16s stop=%-30r nostop=%-30r turn1 ids %d vs %d exact=%d/%d"
              % (c, i, a["reply"], b["reply"], len(a["turn1_ids"]), len(b["turn1_ids"]),
                 trace["exact"][(c, i)], nostop["exact"][(c, i)]))

    firing_changed = [(c, i) for c, i in sc.CELLS
                      if sc.fired(trace["recs"][(c, i)])
                      and trace["recs"][(c, i)]["turn1_ids"] != nostop["recs"][(c, i)]["turn1_ids"]]
    print("firing cells with a different turn-1 reply: %d %s"
          % (len(firing_changed), ["%s/%s" % x for x in firing_changed]))
    same = sum(1 for c, i in non_firing
               if trace["recs"][(c, i)]["reply"] == nostop["recs"][(c, i)]["reply"])
    print("non-firing cells whose turn-2 reply is byte-identical to the no-stop run: %d/%d"
          % (same, len(non_firing)))
    return 0


if __name__ == "__main__":
    sys.exit(main())

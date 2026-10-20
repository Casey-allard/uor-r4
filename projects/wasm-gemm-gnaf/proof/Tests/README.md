# Tests/

SPEC §18 requires executable tests over malformed binaries, every enabled
instruction rule, zero/maximal dimensions, overflow modes, floating special
classes, every status result, compiler round-trips, attention false negatives,
dependency invalidation, partition gaps, and artifact mutation.

The falsification half that does not depend on unbuilt layers is implemented and
passing in `Tools/mutation.py` (7 planted faults, all rejected) and registered in
`model/falsifiers.json`.

Tests whose subject does not yet exist are **not** stubbed here. A suite that
passes because its target is absent is worse than no suite (SPEC §18).

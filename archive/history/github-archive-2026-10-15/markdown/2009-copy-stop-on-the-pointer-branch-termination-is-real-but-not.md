# #2009 — Copy-stop on the pointer branch: termination is real but not merely termination (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T13:01:19Z
- **Updated:** 2026-10-09T13:20:15Z
- **Closed:** 2026-10-09T13:02:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2009
- **Merged:** 2026-10-09T13:02:57Z
- **Merge commit:** 9eb767a589e9af3ca36bfa10548edb1fa1db9a75
- **Base / Head:** main / UOR-Foundation:deepseek/gate-floor-impl
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Copy-stop on the pointer branch: termination is real but it is not merely termination

Follow-up to the gate-floor breakthrough, whose own curve exposed the next constraint: exact copies
peak at F=0.5 and collapse after it (arm-ptr 35 -> 40 -> 49 -> 16 -> 6), and at F=1.0 the value is
present in 79/84 replies with p_mixture 0.8700 yet exact copies fall to 6/84 because the replies stop
terminating ("Fennor.\n\nAss name is Fennor. Fennor.\n\nAss name is Fennor. Fennor.\n\nA").

THE CHANGE. An optional serving-time rule pointer_copy_stop=MIN_SPAN (default 0 = today's behaviour)
that ends generation when an emitted id breaks a run of at least MIN_SPAN ids reproduced from the
window the POINTER ITSELF selected - its argmax source at each step and the consecutive positions
after it. The rule never sees the expected value. A companion pointer_copy_trace=1 records the
pointer's own selection at every step, which is what made the mechanism measurable. Read-out only:
no training, no parameters, nothing serialized.

THE RESULT (84 cells per arm; the F=0 reference reproduces the sealed counts exactly).

    arm         F=0    F=0.5       F=0.75       F=1.0
    arm-ptr      35    49 -> 50    16 -> 47     6 -> 46
    arm-a-w05    10    59 -> 59    42 -> 65    26 -> 64
    arm-c-ctrl   24    52 -> 52    11 -> 54     7 -> 52

  TERMINATION IS REAL AND LARGE: at F=1.0 the rule recovers +40 cells (p=2.0e-11, 41 gained / 1
  lost), and at F=0.75 it recovers +31 on arm-ptr with 31 gained / 0 lost.
  BUT IT IS NOT MERELY TERMINATION: 46/84 is still short of the F=0.5 plateau's 49/84, and the rule
  adds EXACTLY ZERO at F=0.5, where the replies already terminate.
  AND THE CAUSE IS MEASURED: per-step traces show the matched run lies inside the value's own span on
  only 20 of 79 cells at F=1.0 (2 of 50 on arm-a-w05, 18 of 82 on the control) - the rest copy the
  PROMPT TEMPLATE, "User: My nam...". At F=0.5 the rule fires on 6 cells and NONE of those 6 is exact.
  So above ~0.5 the pointer keeps re-reading the template it is prompted with even while the mixture
  probability of the value's first token stays at 0.8700. Values tokenize to roughly 5-7 ids, so a
  run-length rule lands in the wrong place by coincidence.

  A NEW PANEL BEST, WITH ITS CAVEAT STATED: arm-a-w05 at F=0.75 with the stop scores 65/84 = 0.7738
  [0.6735, 0.8501]. Its gain over F=0.75 without the rule is +23 cells with 23 gained / 0 lost
  (p=2.4e-07); its gain over the 59/84 F=0.5 plateau is +6 with p=1, i.e. a TIE by the interval - so
  the new best is real against the unstopped floor and not distinguishable from the plateau.

  DEFAULT PROVEN UNCHANGED, NOT ASSERTED: F=0 runs were compared record by record against the sealed
  files - 288 records, ZERO differing fields in every pre-existing field across 9 of 9 arm-condition
  files - and the exact counts reproduce the sealed 35/84, 10/84 and 24/84 with the final binary.
  Three focused tests pass (the rule fires only on a copied contiguous span including the
  repeated-window and min-span guards; the API contract; the default decoder is bit-identical) and
  rustfmt is clean on every touched file.

NEXT INSTRUMENT, NAMED: a stop tied to the VALUE'S OWN TOKENIZATION rather than a run length - a
token-identity stop - since the failure is that the pointer follows the template while the value's
token probability stays high; and min_span=3 is already implemented and not yet run.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


# #2007 — Run card: a serving-side gate floor tests the gate hypothesis with zero training (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T11:04:13Z
- **Updated:** 2026-10-09T13:20:13Z
- **Closed:** 2026-10-09T11:05:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2007
- **Merged:** 2026-10-09T11:05:53Z
- **Merge commit:** b06d0f00855eb6622e9396d0e6558de1a5bdb290
- **Base / Head:** main / UOR-Foundation:deepseek/gate-floor-card
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Run card: a SERVING-SIDE gate floor tests the gate hypothesis with zero training

Three measurements from tonight converge.

1. The best binding state is the ORIGINAL arm-ptr checkpoint, and further fine-tuning COSTS recall:
   an identical fine-tune with no other change scored 0.2857 against the baseline 0.4167. So the next
   experiment should not train anything.
2. The identity term failure is a GATE COLLAPSE WITH SPAN MASS INTACT: pointer mass on the value span
   0.8399 against the matched control 0.7415, while the copy gate falls 0.2439 to 0.1163 and the
   emitted probability of the value falls 0.2292 to 0.1300. The model still FINDS the value and stops
   USING it.
3. Gate supervision during training raises the gate (0.2644 to 0.2959 at weight 0.1, 0.3559 at 0.3,
   0.4164 at 1.0) but degrades the panel monotonically, so the gate cannot be bought with a loss term.

Together: the gate is the bottleneck, training cannot reach it, and training is now known to be
harmful on this axis - so the remaining test belongs at SERVING TIME.

THE EXPERIMENT: an optional serving-time floor on the gate, pointer_gate_floor=F (default 0 =
unchanged) computing gate' = max(gate, F), scored on the EXISTING arm-ptr checkpoint at F = 0, 0.25,
0.5, 0.75, 1.0, with the identity arm and the matched control as references. No training, no new
parameters, bit-for-bit unchanged at the default.

WHY IT IS RIGHT: it tests the mechanism rather than a hypothesis about it. The claim is that the value
is available and under-used; raising the gate at inference increases how much of the output comes from
pointer attention that is already landing on the span. If a floor recovers copies, the learned
attention already contains the answer and only its MIXING WEIGHT was wrong - a strikingly simple fix
for a problem that has resisted eight routes. If it does not, the span mass, though high, does not
carry the value into the output, and the next target is the READ rather than the gate.

PREDICTIONS FIXED BEFORE THE RUN: if the gate is the constraint, exact copy rises monotonically with F
and more on the identity arm whose gate collapsed, since it has the most headroom; if the span mass is
uninformative, there is no monotone rise and F=1 should be WORSE than the untrained mix, because pure
attention discards the generator that supplies fluency. Score with the probe built from current main,
since a probe predating the identity field scores identity models WRONGLY - 8 of 32 rows differed on
arm A while the dot-only arm was bit-identical.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


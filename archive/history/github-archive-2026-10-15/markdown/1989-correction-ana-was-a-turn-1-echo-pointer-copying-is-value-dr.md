# #1989 — Correction: Ana. was a turn-1 echo; pointer copying is value-driven, the gate is the bottleneck (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T02:43:51Z
- **Updated:** 2026-10-09T13:21:36Z
- **Closed:** 2026-10-09T02:45:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1989
- **Merged:** 2026-10-09T02:45:30Z
- **Merge commit:** 9231d34e62cc4192600304ca1a42fac52be9aa46
- **Base / Head:** main / UOR-Foundation:deepseek/pointer-verify
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

CORRECTION: round 171 recorded that the pointer arm retention turn returned exactly Ana. That is wrong - the panel item is the assistant turn-1 reply, the document decodes as User: My name is Ana. / Assistant: Ana. / User: What is my name? / Assistant: Ag., and the re-run is bit-identical on 14 of 14 rows, so it is my reading error. CONFIRMATION: the copying is nevertheless VALUE-DRIVEN - rows differing only in the value emit Ben., Zorba., Quillon., Anvil., each its own value, and two of those values exist in no store family or chat prior where earlier arms produced Zn./Zinc./Anany./Ane. Pointer mass on the value span is 0.90-1.00 in 9 of 21 cells, read plus pointer mass at or above TAU in 14 of 21, value-paired DELTA +3.85 to +10.39 nats against 1/7 argmax and 0/63 value-containing replies for the commitment arms. WHY ana LOSES: the copy head sits at 0.99 on the span while the learned gate is only 0.07-0.11, so the generator outvotes the copy at the second token; the rows that copy whole spans have gates 0.60-0.99. The bottleneck is the GATE, and pointer gate supervision exists and was not used - the direct next intervention. References #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


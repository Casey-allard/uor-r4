# #2008 — Gate floor at 0.5 recovers recall on every arm with zero losses (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T11:31:40Z
- **Updated:** 2026-10-09T13:20:17Z
- **Closed:** 2026-10-09T11:33:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2008
- **Merged:** 2026-10-09T11:33:20Z
- **Merge commit:** b7541b9f91bd37069ed4994088ae338a50314a24
- **Base / Head:** main / UOR-Foundation:deepseek/gate-floor-result
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

A serving-time gate floor at 0.5 recovers recall on every arm with ZERO losses

The run card's experiment: gate' = max(gate, F) at the two serving mixture sites only, default 0 =
bit-for-bit unchanged, no training, no new parameters, no serialization or export change.

  arm            F=0              F=0.5             delta        significance
  arm-ptr        35/84 = 0.4167   49/84 = 0.5833    +14/84       p=1.22e-4   14 gained / 0 lost
  arm-a-w05      10/84 = 0.1190   59/84 = 0.7024    +49/84       p=3.55e-15  49 gained / 0 lost
  arm-c-ctrl     24/84 = 0.2857   52/84 = 0.6190    +28/84       p=7.45e-9   28 gained / 0 lost

NOT ONE CELL IS LOST ON ANY ARM, and the arm whose identity term had destroyed its recall - written
off two rounds ago - becomes THE BEST ARM MEASURED ON THIS PANEL at 0.7024, above arm-ptr at any
floor. It repairs 22 of 28 values and loses none.

THE MECHANISM IS PURELY A MIXING WEIGHT. Span mass and read mass are FLAT across F, bit-identical
wherever the window is unchanged (arm-ptr 0.8351 -> 0.8321; a-w05 0.8399 -> 0.8392), while the gate
rises and exact copies rise with it; on the 79 of 84 unchanged-window cells the gate equals
max(gate_F0, F) with zero mismatches. The learned attention ALREADY CONTAINED THE ANSWER and only its
mixing weight was wrong - the identity term's damage was a serving-time miscalibration, not lost
information.

THE CURVE PEAKS AT 0.5 AND THE CARD'S DICHOTOMY WAS INCOMPLETE. Exact copies are not monotone
(35 -> 40 -> 49 -> 16 -> 6) and F=1.0 is far worse than F=0, as predicted. Measured cause: at F=1.0
the gate is exactly 1.0, p_mixture is 0.8700 and the value is PRESENT in 79/84 replies, yet exact
copies are 6/84 because the replies degenerate ("Fennor.\n\nAss name is Fennor. Fennor. ..."). ABOVE
~0.5 THE CONSTRAINT IS THE GENERATOR'S CONTRIBUTION TO THE REPLY'S SHAPE AND TERMINATION, not the
value's identity. The next target is therefore the read-out's termination - a conditional or decaying
floor schedule, or an explicit stop rule on the pointer branch.

THE DEFAULT IS PROVABLY UNCHANGED: the F=0 records are identical in all three conditions to the
previously recorded arm-ptr files across every field, with the added floor header the only difference
(canonical-JSON sha256 f380b871 / 5a2368de / 36007e4d on both sides).

THE IDENTITY TERM SHOULD NOT BE WRITTEN OFF: the same checkpoint goes from 0.1190 to 0.7024 by
flooring the gate. The two earlier negatives remain correct AS MEASURED - the term degrades an
UNFLOORED read-out - and are superseded in practice by this result.

F=0.5 was the card's PRE-REGISTERED MIDPOINT and is the argmax on all three arms independently, so it
was not chosen post hoc. Not measured: any floor schedule, any other panel, and any integer/D11 port -
export still refuses a pointer with an identity term, so this is an offline-capability result so far.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


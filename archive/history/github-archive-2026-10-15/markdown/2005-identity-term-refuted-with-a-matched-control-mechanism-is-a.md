# #2005 — Identity term refuted with a matched control; mechanism is a gate collapse (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T10:51:53Z
- **Updated:** 2026-10-09T13:20:36Z
- **Closed:** 2026-10-09T10:53:31Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2005
- **Merged:** 2026-10-09T10:53:31Z
- **Merge commit:** be38426c91be211b9a90228fbb425ffe2c2ca462
- **Base / Head:** main / UOR-Foundation:deepseek/identity-control
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

#2004 compared the identity arms against the ORIGINAL arm-ptr checkpoint, mixing the identity term with the extra fine-tuning both identity arms received. A third arm supplies the control: same fine-tune, same recipe, same 12,207 steps, identity omitted. arm-ptr 35/84 = 0.4167 [0.3171,0.5235]; A (0.5) 10/84 = 0.1190 [0.0660,0.2054]; B (1.0) 0/84 = 0.0000; CONTROL 24/84 = 0.2857 [0.2002,0.3900]. THE CONTROL SPLITS THE EFFECT: extra fine-tuning alone costs about 13 points; against the matched control the identity term costs a further -0.1667 at w=0.5 (p=0.0161) and -0.2857 at w=1 (p=1.19e-07). So the conclusion survives but the attribution was too generous - about 13 of the 30 points are fine-tuning, the term own contribution about 17 at half weight and 29 at full. THE MECHANISM IS MEASURED: at w=0.5 the pointer still attends the value span as much as the baseline (0.8399 against the control 0.7415) while the copy GATE collapses (0.1163 against 0.2439), so the mixture stops emitting the value (p_mixture 0.1300 against 0.2292) - a gate/readout collapse with unchanged span mass. ARM B reproduces the frozen-curve signature, its hit rate identical to all printed digits at every one of thirteen evals while reachability stays 0.4729. A TOOLING FINDING: the existing probe predated the identity field and scores identity models wrongly - 8 of 32 rows differ on arm A while the dot-only arm is bit-identical - so the old binary was preserved and a new one built; using yesterday probe would have produced numbers that looked like an identity effect and were partly a stale-binary artefact. It also establishes that additional fine-tuning on this mixture costs recall by itself. References #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


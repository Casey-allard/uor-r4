# #2006 — Retraction: extending this run costs recall; best state is the original checkpoint (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T10:55:41Z
- **Updated:** 2026-10-09T13:19:59Z
- **Closed:** 2026-10-09T10:57:19Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2006
- **Merged:** 2026-10-09T10:57:19Z
- **Merge commit:** fd300d7c883b5c642b3397b6472723d5e8e57425
- **Base / Head:** main / UOR-Foundation:deepseek/extend-retraction
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

After the pointer arm first emitted a recalled value, the expert review recommended extending that run rather than redesigning it, and I repeated that recommendation twice. THE MATCHED CONTROL REFUTES IT FOR THIS AXIS: a control arm ran the identical fine-tune as the identity arms - same checkpoint, mixture, recipe and 12,207 steps - with only the identity term omitted, and scored 24/84 = 0.2857 against the un-fine-tuned baseline 35/84 = 0.4167. So additional fine-tuning ALONE costs about 13 points of recall, and the identity arms' larger losses are that cost plus the term's own. WHY IT IS A RETRACTION: the advice assumed the selection curve tracked what matters, but the session had already shown three times that dev_pointer_hit_rate does not - it sits at the baseline's level for an arm whose exact copies fell to 10/84, and is frozen to all printed digits for an arm that cannot copy at all. WHAT CHANGES: the best measured binding state is the ORIGINAL arm-ptr checkpoint, not a longer run; future arms must use a MATCHED fine-tune control, not just the starting checkpoint; the failure is localised to the GATE (span mass 0.8399 vs the control's 0.7415, gate 0.1163 vs 0.2439, emitted probability 0.1300 vs 0.2292); so the remaining lever is a change that keeps the gate high while preserving span attention - neither more training nor more identity. References #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


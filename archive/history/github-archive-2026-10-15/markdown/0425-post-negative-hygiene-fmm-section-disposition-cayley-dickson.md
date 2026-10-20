# #425 — Post-negative hygiene: FMM section disposition, cayley_dickson module, r4-attention deferral record

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-05T01:04:49Z
- **Updated:** 2026-08-05T03:49:13Z
- **Closed:** 2026-08-05T03:49:13Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/425
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Three leftovers with no disposition record (completeness review 2026-08-05): (1) the FMM section is still compiled into every scored artifact + packed-kernel path exists, merged hours BEFORE the family was measured dead (#290 RESULT-52); docs/fmm_290_novel_context_protocol.md still says the decision 'remains open' — do a #410-style measured-dead removal or a dated deferral record. (2) graph-runtime pub mod cayley_dickson: zero callers since #410 — delete (history preserves text, #376 precedent) or record. (3) r4-attention Spin(4) engine: shipped, selectable, never measured, absent from the #249 deferral record — add to a dated deferral record or run one teacher-parity BDD with the flag on. Also: paste the issue-side closure verdicts for #374/#394/#395 + the E8 design law into docs (#393 doc is the natural home).

## Comments (1)

### comment 5187310297 — 2026-08-05T03:49:12Z — Casey-allard

Scope delivered (PR #426, merged): FMM removed measured-dead-style with parser retained, cayley_dickson deleted, dated deferral records for r4-attention and bott_fock (→#424), E8 design law + closure verdicts pasted into docs.


---
_Comment/review ids in this file: 5187310297_


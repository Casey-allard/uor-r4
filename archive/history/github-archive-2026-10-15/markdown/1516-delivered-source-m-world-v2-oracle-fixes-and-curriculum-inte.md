# #1516 — [Delivered source] M-world v2 oracle fixes and curriculum interfaces

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-29T20:09:28Z
- **Updated:** 2026-10-01T06:27:27Z
- **Closed:** 2026-10-01T06:27:27Z
- **Labels:** enhancement, roadmap:historical
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1516
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

<!-- active-roadmap-2026-10-01 -->
**October 1 adopted roadmap — HISTORICAL.** PR #1564 / D19; adoption task #1563.

The four named oracle regressions, open-value/MQAR/copy curriculum and truncated-prefix source were delivered through #1548/#1556 and related source merges. Preserve v1/R1 scoring identities. A large truncated-prefix adoption comparison was NOT_RUN and is transferred to #1552 only if a future causal work card requires it; it is not marked passed. Closure is source/instrument scope, not language qualification.

[Canonical plan](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/project-track.md) · [Mechanism admissibility](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/mechanism-admissibility-2026-10.md). Historical gates/results and full capability acceptance remain unchanged. Live comments own newer work; this header does not reset cumulative resources or transfer another lab's live files/jobs.
<!-- /active-roadmap-2026-10-01 -->


<details>
<summary>Preserved pre-adoption issue body and original acceptance</summary>

Found by the non-author re-review of #1503 (`d44d68fb`). They are fixed in M-world v2 (A1, #1508), not in v1, so R1's sealed scoring stays reproducible.

1. **`Check::Larger` accepts an echo of the development prompt:** "Out of 3 and 7, which is greater?" is accepted for 7.
2. **`Check::Larger`'s negation window is too narrow:** it fails "The bigger number is 7, the smaller is 3."
3. **`Check::LastLetter` is case-exact,** so a correct lowercase letter fails.
4. **`Check::Numbers` counts the pronoun "one" as 1.**

**Also for v2:**
- open value pools (5k+ names plus random-syllable names per episode), which caused R1's memorization;
- MQAR and copy curricula;
- truncated-prefix episodes.

</details>


## Comments (1)

### comment 5925979881 — 2026-10-01T06:27:27Z — Casey-allard

D19 roadmap reconciliation: The four named oracle regressions, open-value/MQAR/copy curriculum and truncated-prefix source were delivered through #1548/#1556 and related source merges. Preserve v1/R1 scoring identities. A large truncated-prefix adoption comparison was NOT_RUN and is transferred to #1552 only if a future causal work card requires it; it is not marked passed. Closure is source/instrument scope, not language qualification.

Disposition only at this stated scope; see #1563 and protected PR #1564.


---
_Comment/review ids in this file: 5925979881_


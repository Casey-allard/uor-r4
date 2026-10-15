# #2226 — Mechanism brief: the invariant-addressed read (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T01:43:31Z
- **Updated:** 2026-10-12T01:53:02Z
- **Closed:** 2026-10-12T01:49:29Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2226
- **Merged:** 2026-10-12T01:49:29Z
- **Merge commit:** b5535f959b82014fafb328876ab2395db22f374a
- **Base / Head:** main / UOR-Foundation:deepseek/addressed-read-brief
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**The direction you pointed at — Spiralcore's addressing/route layers, the murmurations result, Claude's attention work — stated as a mechanism brief in the form `docs/mechanisms/` requires**, so it can be pre-registered, fitness-checked and tested rather than discussed.

**The mechanism:** the read is the answer path (facts removed → 0/40; supervised read silenced → 0/40 with zero wrong values) and the **only quadratic component** in the stack, and the geometry sits at **0.05 %** of the arithmetic because the deployed read is the flat control. So: give every source position an **invariant address** (prime/UOR identity, icosian phase, murmuration-style family key), **route** the query to a small candidate set by address, and read only those — `O(n)`/`O(log n)` with the geometry doing the work instead of parameterising it.

**The brief contains, per the project's rules:** the first-principles idea; **what has been tried, honestly** — including this week's geometric-read result (ahead on every seed, below its own pre-registered margin), the identity pointer keyed on the wrong six tokens, phase/lineage measured as *scorers* and never as *addresses*, and the addressed memory replacing an **MLP** rather than the read; what success looks like (panel wrong-value primary, **served tokens/s beside it**); what failure looks like; the fair test (**trained in from step 0, ≥3 seeds or ≥200 fresh rows, the frozen panel as acceptance**); and the open design questions (which invariant, how to route, what granularity, exact vs learned).

**Two controls are required, and the second is the point.** (1) The full softmax read at the same budget. (2) A **null address** — the same routing machinery with the address **permuted** — so a routing win cannot be credited to geometry without evidence. That trap is exactly where this line has fallen before, and the null control is what makes the next result attributable.

Line: invariant-addressed read · count 0/3 · headline **unchanged** (brief only; the pre-registration and its `TEST FITNESS` review go on [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029) before any compute)

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


# #1863 — Emit the read's selection: it is independent of the question and the answer (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T00:23:12Z
- **Updated:** 2026-10-08T00:24:53Z
- **Closed:** 2026-10-08T00:24:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1863
- **Merged:** 2026-10-08T00:24:53Z
- **Merge commit:** c2a41a3047af22c77164886ab5b0d0a89c7b0c3a
- **Base / Head:** main / UOR-Foundation:deepseek/entry-scorer
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this adds

The entry ceiling now records `source_provenance` beside the boundary state — including the read-state bridge trace with `query_state_codes`, `selected_source_state_codes`, `selected_candidate` and `selected_ordinal`. Diagnostic output only; no decision path reads it.

## Why, and what it found

The state probe showed the boundary state is metric-free: same-question and different-question rows are equidistant (7.931 vs 7.931 over 130,816 retained pairs), so no readout on it can transfer. This change made the read's own internals measurable offline, without a model, and they say why.

**The state has no factorised channels.** The transfer panel names its construction in the case id, so rows can differ in exactly one factor — role assignment (content) or question (form). Per code position over 16 pairs of each kind: `codes_invariant_to_assignment` is **empty**, `codes_that_separate_query_perfectly` is **empty**, and every code changes with the question in 100 % of pairs and with the content in 94–100 %. All 8 codes are hash components.

**The channels the read already computes fail the same test** (within vs across, same-question rows should be closer):

| channel | within | across |
|---|---:|---:|
| aggregated retained state | 7.938 | 7.941 |
| `query_state_codes` | 7.908 | 7.918 |
| `selected_source_state_codes` | 7.954 | 7.613 |

All 32 query-state vectors are distinct — the query channel carries the entity, so it is row-specific rather than form-specific.

**Root cause: the selection ignores the question and the answer.** `selected_ordinal` takes only the values {0, 5, 6}, and its distribution is the same for both forms and both answers:

| | ordinal 0 | ordinal 5 | ordinal 6 |
|---|---:|---:|---:|
| asked form 0 / form 1 | 9 / 8 | 4 / 3 | 3 / 5 |
| answer 617 / answer 2997 | 8 / 9 | 3 / 4 | 5 / 3 |

The read does not use what was asked to choose which occurrence to read. That explains the hash state, the 53 % 1-NN transfer, the flat emission, and entry 0 on both panels, without a further hypothesis.

## Gates this hands the next change

1. **Selection gate** — CPU-only, offline from this provenance plus the panel's segment relations: the selected occurrence must become question-determined, where today the distribution is identical across forms.
2. **State gate** — one short read: the boundary state must then show same-question clustering (within strictly below across).
3. **Then** the emission and the entry decision, targeting entry > 0 on the transfer panel.

## Verification

`cargo check -p uor-r4-training --example geometric-frozen-map-fit` clean, `cargo test -p uor-r4-training --example geometric-frozen-map-fit` **13 passed / 0 failed**, touched-file `rustfmt --check` clean.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


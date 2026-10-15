# #1859 — Read a declared panel: entry is 0/32 on the prospectively fixed transfer panel (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T21:20:50Z
- **Updated:** 2026-10-07T21:22:38Z
- **Closed:** 2026-10-07T21:22:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1859
- **Merged:** 2026-10-07T21:22:38Z
- **Merge commit:** 3f6adaf37d18a20efec3cf6c8c515193d5d609f0
- **Base / Head:** main / UOR-Foundation:deepseek/entry-scorer
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this adds

Two things needed to read a panel other than the pinned one.

**1. A declared panel for the zero-update entry read.** The fitter asserted four panel file hashes and required 512 paired rows, so nothing else could be measured. A declared panel (`panel_inputs` + `panel_labels`) replaces the pinned one **for `entry_ceiling` only**: hashes and row count are recorded in the report rather than asserted, the swap-pair audit and the retained-panel baseline evaluation are skipped, and nothing fits on it. Any other mode with a declared panel is refused.

**2. The transfer panel's `pair_id` label field**, declared explicitly so `deny_unknown_fields` stays in force for every other field rather than being dropped.

## Why, and what it measured

The adversarial review of the entry work required a held-out measurement the retained panel cannot make: its 512 rows are all exposed training rows with twelve query signatures and two answer forms, so any "held-out" number there is memorisation. The repository's own held-out instrument is `native-bank-transfer-panel` (prospectively fixed complete-value recombination, role/order/query transfer within two supported prose forms). It could not be read before this change.

`deepseek/entry-transfer/transfer-entry-attempt1/`, 21.8 s, `declared_panel: true`, `panel_rows: 32`:

| | retained panel (512, all exposed) | transfer panel (32, new values/roles/order) |
|---|---:|---:|
| entry correct, mixed pool | 0 / 512 | **0 / 32** |
| entry correct, Generate-only | 2 / 512 | **0 / 32** |
| gold rank median (of 4,096) | 1,145 | **1,136** |
| gold mass tied with | ~247 tokens | 256 tokens |
| top-1 mass share | 0.066 % | **0.070 %** (uniform 0.024 %) |
| float vs integer rank | identical, 512/512 top-1 | **identical, 32/32 top-1** |
| winners Copy-dominated | 512 / 512 | 32 / 32 |

The failure is not a property of the retained constructions, not memorisation of those rows, and not the export: on new values, roles and order the artifact behaves identically, emits neither template (a constant predictor scores 16/32), and its emission stays flat at rank ~1,136.

## Instrument gap this exposed

The transfer panel varies values, roles, order and query position **within two prose forms** and explicitly claims no unseen-word generalisation, so the review's target — ≥8 answer forms and ≥50 question signatures on held-out wordings — does not exist in the repository. The generator that emits such text (`heldout-memory-panel`) writes a different schema (user-turn requests), which the native fitter cannot consume. Recorded here so the next mechanism step budgets for that panel or accepts measuring transfer within two forms.

## Verification

`cargo check -p uor-r4-training --example geometric-frozen-map-fit --example entry-scorer-feasibility` clean, `cargo test -p uor-r4-training --example geometric-frozen-map-fit` **13 passed / 0 failed**, touched-file `rustfmt --check` clean. Defaults unchanged: both new fields default to unset, so every prior mode behaves exactly as before.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


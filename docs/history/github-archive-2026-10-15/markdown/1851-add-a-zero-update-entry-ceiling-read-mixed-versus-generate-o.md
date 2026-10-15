# #1851 — Add a zero-update entry-ceiling read: mixed versus Generate-only, with the gold rank in the emission (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T18:24:17Z
- **Updated:** 2026-10-09T14:18:19Z
- **Closed:** 2026-10-07T18:26:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1851
- **Merged:** 2026-10-07T18:26:06Z
- **Merge commit:** 571f5f9e4b87a7f951c222d592ba43b062d3870f
- **Base / Head:** main / UOR-Foundation:deepseek/entry-ceiling
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this adds

A zero-update diagnostic mode, `mode = "entry_ceiling"`, plus a panel read inside the prediction-control path. Nothing about fitting, serving or authentication changes: the ceiling uses the existing target-free native forward, reads labels only after it, and the prediction-control artifact is still recomposed from the pinned lineage with no override.

It reports, per row: the entry decision with the physical Copy channel present and with it removed, whether the gold was ever a Copy candidate, the winner's Copy share, and the gold's **rank and mass fraction in the fully ordered Generate-only distribution** — at the entry position and at the position after it, where the model is given the correct first token as its prefix.

## Why it was needed

Three claims were live on #820: the entry is out-weighted, the entry is out-competed by Copy, and component calibration on the eight control cases generalises. Each is cheap to test at population scale and none had been measured on the 512-row panel rather than on eight selected rows.

## What it found (sealed roots on the canonical volume)

| artifact | entry mixed / Generate-only | entry gold rank median (of 4,096) | next-token gold rank median | gold in entry top-10 |
|---|---:|---:|---:|---:|
| frozen parent `checkpoint-0128` | 0 / 512 · 2 / 512 | 1,145 | 1,189 | 9 / 512 |
| Codex recomposition @ `503d64b39` | 8 / 512 · 9 / 512 | 929 | 878 | 22 / 512 |

Roots `deepseek/entry-ceiling/{ceiling-attempt3,recomposed-panel-attempt3}/`. The parent run reproduces the sealed admission baseline exactly (entry 0/512, Copy-covered 351/4,128, first-divergent 40/1,024), and the control win reproduces in full (`entry_teacher_correct` 8, `entry_own_correct` 8, `first_divergent_own_prefix_reached` 8, `paired_own_position_both_correct` 4), with its correct panel indices being exactly `[0,1,4,5,8,9,12,13]`.

The emission is at ~28th percentile and giving it the correct prefix does not sharpen it. Correctness tracks copyability, which is why entry — the one position with an empty answer prefix — is 0/512. Full discussion and the three retired hypotheses are on [#820](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6044163802).

## Verification

- `cargo check -p uor-r4-training --example geometric-frozen-map-fit` — clean.
- `cargo test -p uor-r4-training --example geometric-frozen-map-fit` — **12 passed, 0 failed**, including the new `entry_ceiling_mode_is_a_distinct_zero_update_configuration`, the retained-campaign default test and the parity test.
- Touched-file `rustfmt --check` clean.
- Defaults are unchanged: `mode = "fit"`, `loss_scope = "all"`, `updates = 128`. The new mode is opt-in and zero-update by construction.

Scope: train and development are the identical 512 exposed construction rows; this is a capability bound on two artifacts and not held-out transfer, chat or reasoning evidence.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


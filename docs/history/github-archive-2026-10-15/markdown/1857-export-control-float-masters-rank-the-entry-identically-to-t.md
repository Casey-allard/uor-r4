# #1857 — Export control: float masters rank the entry identically to the exported integers (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T20:58:26Z
- **Updated:** 2026-10-07T21:00:01Z
- **Closed:** 2026-10-07T21:00:01Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1857
- **Merged:** 2026-10-07T21:00:01Z
- **Merge commit:** fe928f73f55ae7769bb3cfc901b5fbf5eba0b009
- **Base / Head:** main / UOR-Foundation:deepseek/entry-scorer
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this adds

`mode = "entry_ceiling"` with `ceiling_float = true` scores the **same read state** twice over all 512 entry rows: once with the exported integer table (`score_into`) and once with the float masters (`forward_prepared_coefficients_only`). Plus the baselines and tie counts a rank needs to be readable at all.

This answers the one alternative the adversarial review of the entry work found unexcluded: every previous measurement read the integer, post-export path, so "the learned emission is uninformative" was never separated from "the export destroyed what the float masters carried".

## Result

Sealed root `deepseek/entry-ceiling/export-control-attempt1/`, 223.0 s:

| | integer (exported) | float (masters) |
|---|---:|---:|
| gold rank median | 1,145 | 1,145 |
| gold rank p90 | 2,971 | 2,971 |
| gold rank max | 4,083 | 4,083 |
| median rank delta | — | **0** |
| top-1 token agreement | — | **512 / 512** |

Coefficient quantization, the magnitude-2 clamp and the Q24/Q31 conversion change nothing about the entry decision in any row. The export is not the constraint.

Baselines now reported alongside: **majority 0.5** (a constant `ĠYour` predictor scores 256/512; the artifact scores 0/512), **uniform share 0.024 %** versus a measured top-1 share of 0.066 %, and **tie group at the gold's mass: median 247 tokens, max 371** — the pool quantizes at 256 steps/octave, so a rank of 1,145 is "inside a ~250-token tie band" and the flatness, not the rank, is the load-bearing evidence.

## Corrections carried in this branch's record

- Withdrawn: the claim that a bounded coefficient range caps a token's mass share at ~0.7 % and explains the failure. A bounded span bounds the max/min mass *ratio*, not the max share; the floor is uniformity (0.024 %) and the measurement sits just above it. The ±1.6–1.7 nat figure is the realized artifact bound, not an architectural law.
- Stated rather than implied: the pair tables' 0.25–0.336 against a 0.5 two-class majority.

## Verification

`cargo check -p uor-r4-training --example geometric-frozen-map-fit` clean, `cargo test -p uor-r4-training --example geometric-frozen-map-fit` **13 passed / 0 failed**, touched-file `rustfmt --check` clean. The flag defaults to false, so every prior mode is unchanged.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


# #506 — docs(#320): SmolLM2-360M P2 baseline — teacher floor 3.79 bits

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-08T19:24:43Z
- **Updated:** 2026-10-09T13:08:50Z
- **Closed:** 2026-08-08T19:45:14Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/506
- **Merged:** 2026-08-08T19:45:14Z
- **Merge commit:** 2127d2d1b90792818712a42ceca9e0bf9e7a0404
- **Base / Head:** main / UOR-Foundation:docs/320-smollm2-360m-p2
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Records the **P2 SmolLM2-360M baseline** in `docs/smollm2_teacher_baseline_320.md` (doc-only). Full pipeline: observe → 20k-record corpus → cover → score (`relative_tla`) → teacher-floor report. Full result table is in the [#320 P2 comment](https://github.com/UOR-Foundation/uor-r4/issues/320).

## Headline

**Teacher floor 3.7908 bits/token** — lower than the 135M cloud floor (4.20) and the original 135M doc floor (4.95). A measurably more competent teacher with a higher ceiling. The artifact sits **+17.39 bits over the floor** (compiler-bound at 20k rehearsal): TLA top-1 5.54% / teacher-argmax 8.61% / 21.18 WB bits; graph Rule 1+2 5.0% / 14.78; #399 A-mode fwd-anchor 5.7%.

The ceiling lift is real but only cashable at broad-corpus scale, where the substrate can actually reach a lower floor — which is the case for the broad-corpus program (the agreed next big run).

## Provenance note

The descriptor's pinned revision `2366112…` now **404s on HF** (upstream rewrote it). Used current `main` (`a10cc151…`); its `model.safetensors` is **byte-identical (723,674,912 bytes)** to the July-pinned copy, so the weights are the same release — a metadata-commit rev bump. κ recorded in the doc.

## Not in scope

**P3 (the baseline re-pin) stays a maintainer decision** per AGENTS.md — this PR records the P2 measurement only. **#320 stays open** for that decision.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


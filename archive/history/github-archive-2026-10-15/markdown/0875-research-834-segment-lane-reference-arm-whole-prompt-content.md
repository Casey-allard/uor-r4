# #875 — research(#834): Ψ segment-lane reference arm — whole-prompt content is predictive (positive)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-20T19:40:44Z
- **Updated:** 2026-10-09T13:08:55Z
- **Closed:** 2026-08-20T19:57:22Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/875
- **Merged:** 2026-08-20T19:57:22Z
- **Merge commit:** 37bc2d0ca079822b9a84f7a553baeea06fe742be
- **Base / Head:** main / UOR-Foundation:issue-834-psi-arm
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

Follow-up to the #834 run (PR #874), which found the **deployed** model suffix-local (`NO PROMPT-CONDITIONING ARM ESTABLISHED` for the fittable window arms). Its three #835 Ψ-family arms were UNAVAILABLE because they aren't lowered. Per the maintainer decision (respecting the run contract — a deployed lowering under #836 needs a positive verdict first), this PR builds the #835 **segment lane** as an *offline reference arm* and re-tests it teacher-grounded, to answer the decisive question: **does whole-prompt content carry predictive signal beyond the suffix?**

## Method (reference-only, from the corpus; no engine, no live teacher)

From **288,794 TRAIN positions** (document-disjoint by story via `split_positions`), two co-occurrence tables predicting the recorded teacher argmax: a 2-token **suffix** baseline and a whole-prompt **content** (segment-lane) table, each bounded to top-64 per key (#835 bounded lane). Scored on **all 72,130 held-out positions**; λ fixed at 1.0 before evaluation.

## Result

- suffix baseline top-1 vs teacher **246.6‰**; Ψ (suffix + whole-prompt content) **264.1‰** → **Ψ-delta +17.5‰ (paired 95% CI [+15.9, +19.0])**, interval excludes zero.
- λ-sweep (exploratory, not the verdict): 0.5→264, 1→264, 2→261, 4→248, 8→223 ‰ — not an over-weighting artifact.
- **minimal pairs** (same 2-token suffix, different story, different teacher answer): Ψ **follows 10/4,722** (95% lower bound 0.8‰) where the suffix baseline — a pure function of the suffix key — follows **0/4,722** by construction.

## Verdict: `SELECT` (positive signal)

Whole-prompt content is predictive of the teacher's answer beyond the 2-token suffix. Combined with #874 §6.1: **the deployed artifact is suffix-local and discards this real signal, which a bounded segment-lane mechanism recovers.** This is the run contract's positive that legitimately warrants building #835/#836.

**Effect size — do not over-read:** the gain is **modest** (+1.75pp overall; only 10 of 4,722 hardest same-suffix pairs resolved). The segment lane helps on average (candidate-support widening + scoring) but rarely resolves genuine same-suffix context dependence. #836 should be pursued with calibrated expectations; a deployed lowering must re-clear the causal gate on the packed path (#836's own acceptance).

## Tests and verification

- `cargo test -p uor-r4-api --release --test psi_arm_run_834 -- --ignored` — the run (ignored; skips if the bundle is absent). Deterministic (stable result_cid `blake3:50c77978…`).
- Full api suite green (two ignored harnesses now); `clippy --all-targets -D warnings`, `fmt --check`, `check_claim_wording.py`, `xtask validate` (every gate passed) — all clean.

## Evidence / conformance / compatibility

`docs/psi_arm_834_result.json` (CID-bound); `docs/prompt_arms_bakeoff_834.md` §6.2. Reference-only / off-serving; no `model/ids.toml` row, no `CONFORMANCE.md` regen; RF-01/21/22/27/28. No serving/format/ABI change.

## Closure

**References #834.** This is the positive that legitimately triggers **#836** (lower + certify the #835 persistent-state mechanism, then re-run this harness against the packed path). #834 stays open per the maintainer decision. Untracked `obs-text/`, `tf1-pkg/` untouched.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


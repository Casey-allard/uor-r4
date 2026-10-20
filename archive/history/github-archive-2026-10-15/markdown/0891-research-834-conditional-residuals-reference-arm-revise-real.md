# #891 — research(#834): conditional-residuals reference arm — REVISE (real, unconditionally sub-floor)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-21T05:39:50Z
- **Updated:** 2026-10-09T13:08:55Z
- **Closed:** 2026-08-21T06:00:44Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/891
- **Merged:** 2026-08-21T06:00:44Z
- **Merge commit:** 77d09023d191672a2268e1b1c70d0e3b75d22b46
- **Base / Head:** main / UOR-Foundation:issue-834-conditional-residuals
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

#834's five-arm scope had one arm never built or measured: `conditional-residuals` (persistent state minus the corpus marginal). Per the 2026-08-21 maintainer decision recorded on the issue, this PR builds it as an offline reference arm (the #875 precedent) and records the teacher-grounded verdict: **REVISE — real but unconditionally sub-floor** (+16.2‰ vs the suffix baseline, paired 95% CI [+14.6, +17.9]; the interval's UPPER bound sits below the frozen 20‰ `CAUSAL_FLOOR_PERMILLE`). No lowering track opens (#887: the bar stands). This closes the #834 arm space: every arm of the five-arm scope now carries an evidence-backed disposition, and none cleared the floor.

The arm's distinctive mechanism is falsified two ways: CR-vs-Ψ on identical candidates and tables is −1.3‰ [−2.4, −0.1] (the marginal subtraction slightly hurts), and the residual-shuffle null (+16.0‰ [14.4, 17.6]) lands at the real arm while changing 14,478 predictions — the specific marginal alignment contributes nothing; what carries the arm is §6.2's content evidence.

## Scope

- `crates/uor-r4-api/tests/conditional_residuals_run_834.rs` — new ignored reference harness (research-leaf; deterministic, offline, 0 teacher evaluations; reproduces §6.2 to the digit as its correctness gate; 2,000-position in-harness double-run determinism check; prompt-swap / residual-shuffle / trivial-prior / suffix-only controls with non-degeneracy asserts; `cr-narrow` attribution arm).
- `docs/conditional_residuals_834_result.json` — CID-bound run record (corpus_meta_cid `blake3:aa9d1767…`, result_cid `blake3:feece1b3…`).
- `docs/prompt_arms_bakeoff_834.md` — §6.3 appended (run, controls, verdict, arm-space disposition). Record history unchanged.
- `docs/RESEARCH.md` — S1 summary reconciled with the new measurement (paragraph appended after the §6.2 summary).

## Non-goals

No `model/ids.toml` row, no `CONFORMANCE.md` regeneration (research-leaf per the #835/#832 precedent), no serving-path or runtime changes, no reopening of the retired segment-lane track (#836/#886/#887), no λ tuning (λ = 1.0 fixed before evaluation; the sweep is exploratory and flat).

## Acceptance-criteria status (issue #834)

- All arms and controls on identical identities and evaluation slices — DONE (one harness, one pass, fixed argmax decoder, #833 bundle, full 72,130 held-out).
- Candidate-support vs score-conditioning separately identified — DONE (`cr-narrow` +3.1‰ [2.3, 4.0] vs widened +16.2‰ — most movement is candidate availability).
- No control passes through zero variance or identical outputs — DONE (swap changed 27,758 predictions, rshuf 14,478; asserted non-degenerate).
- Decision is SELECT / REVISE / NO-ARM under the posted threshold — DONE: **REVISE** under the pre-registered rule posted to #834 before the run (SELECT required paired lower bound ≥ 20‰; #887-calibrated, stricter than §6.2's rule).
- Raw per-pair attribution and aggregate uncertainty replayable from CID-bound records — DONE (record carries the raw integer counts; result_cid binds them to the corpus CID).

## Tests and verification

- Run contract posted to #834 BEFORE the run; outcome posted against it (issue comments of 2026-08-21).
- Binding cheap instrument (merged #873 `prompt_arms_bakeoff_834`) re-run at 7bb8b4ec: **PASS 12/12**.
- Harness-correctness gate: §6.2 reproduced to the digit (suffix 246.6‰; Ψ 264.1‰ / +17.5‰ [15.9, 19.0]; 4,722 pairs; Ψ-follow 10; baseline-follow 0) — hard asserts, run aborts on drift.
- Local gates all green before push: `cargo fmt --check`; `python3 scripts/check_claim_wording.py`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; no_std ladder (`uor-r4-graph-format` no-default-features / +alloc); wasm lib build; `xtask validate` (R1/R4/R5/gnaf); `repo-conformance` (R2/R3); `cargo deny check bans` (R6); `cargo test --workspace --no-fail-fast`.

## Evidence artifacts

`docs/conditional_residuals_834_result.json` (CID-bound), `docs/prompt_arms_bakeoff_834.md` §6.3, run log figures mirrored in the #834 outcome comment.

## Compatibility and migration

None. Research artifacts are non-serving; no format, runtime, or artifact changes. The harness is `#[ignore]`d (needs the local #833 bundle) and skips cleanly when the bundle is absent — CI runs it as compile-only.

## Conformance effects

None. RF-01 / RF-21 / RF-22 / RF-27 / RF-28 evidence extended by records only; no new built capability; `CONFORMANCE.md` untouched.

## Claim-boundary changes

`docs/RESEARCH.md` and record §6.3 now assert: the conditional-residuals mechanism's increment over raw content evidence is non-positive at this scale; the whole-prompt-content arm class is real but sub-floor in every measured variant; the five-arm space is dispositioned with no arm clearing the floor; S1 meets the #822 kill/redesign criterion. The S1 stage verdict against the full child set remains the maintainer's call on #822.

## Negative or unavailable results

The REVISE verdict IS the required scientific output per the issue's claim status ("a negative result is the required scientific output, not an implementation backlog"). Nothing is UNAVAILABLE: every contracted input was present (bundle, splits, recorded teacher argmax, frozen floor).

## Intentionally excluded

Pre-existing untracked user dirs (`obs-text/`, `tf1-pkg/`) are not part of this change and are not staged.

Closes #834.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01J2R2qs9kfcYpsjUua1SYmi


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


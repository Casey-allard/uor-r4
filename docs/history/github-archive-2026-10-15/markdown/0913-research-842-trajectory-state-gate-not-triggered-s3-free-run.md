# #913 — research(#842): trajectory-state gate — NOT TRIGGERED; S3 free-running generation not established

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-22T02:14:04Z
- **Updated:** 2026-08-22T02:38:54Z
- **Closed:** 2026-08-22T02:36:23Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/913
- **Merged:** 2026-08-22T02:36:22Z
- **Merge commit:** abcad4e5d084289f053907284185f3c5908e9237
- **Base / Head:** main / UOR-Foundation:issue-842-trajectory-state-gate
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

S3 item C (#842) is the trajectory-state gate: decide, from the frozen sibling-A (#841) and sibling-B (#840) evidence, whether a bounded trajectory-state mechanism (topic / entity / anti-cycle memory) is warranted, then issue the authoritative S3 free-running generation verdict. The guarded risk: adding such state without a binding diagnostic could raise surface diversity while leaving semantic coherence unchanged and needlessly grow the trusted runtime.

**Outcome:** `NOT TRIGGERED` — no bounded trajectory-state mechanism is warranted; and, for the stage, **`GENERATION-NOT-ESTABLISHED`** — coherent free-running generation is not established at this representation scope.

## What it does

A predeclared, non-vacuous diagnostic (`crates/uor-r4-api/tests/state_trajectory_gate_842.rs`) classifies every free-running failure of the frozen prompt-family v1 (n=100, greedy, deployed `R4Engine`, teacher-free) by pairing the **student-prefix** rollout against the **teacher-prefix** rollout and probing the deployed candidate list at the divergence step:

- `SingleStepAt0` — diverged at step 0 (no prior trajectory; state-irrelevant)
- `CandidateGap` — recorded token not among the deployed candidates (re-ranking cannot reach it)
- `RankLimit` — token was a candidate but the perfect context fails at the same-or-earlier step (context-independent)
- `StateStarvation` — token was a candidate AND the perfect context survives longer (the ONLY state-addressable cause)

**Result (`result_cid blake3:86376394…`; corpus `blake3:aa9d1767…`; graph `blake3:bc2366f1…`):** the instrument reproduces #841 exactly (304‰ TF, 99/100 suffix-local, student median 0 / diverged-at-0 590‰ / paths 0/3197/3), and finds **0/100 state-starvation** cases — survived 0 · single-step-at-0 59 · candidate-gap 16 · rank-limit 25. Two representation-independent bounds cap any state mechanism against the frozen §6 bar: **step-0 invariance** (reachable at0-drop 0‰ vs the 100‰ bar; the 590‰ > 500‰ step-0 fraction pins the median at 0) and the **teacher-prefix upper bound** (the drift-free reference is itself median 0 / at0 590‰ — clears neither prong).

## Scope and non-goals

- **Scope:** the cheap diagnostic + confusion fixture, the reachability arithmetic, the binary TRIGGERED/NOT-TRIGGERED decision, and the S3 generation verdict. No runtime state is specified or lowered (NOT TRIGGERED → the #842 non-goal "no runtime code lands when NOT TRIGGERED" is honored).
- **Non-goals (honored):** no repetition penalty called semantic state; no state added when the diagnostic points to candidate/evidence/decoding; no correction/state iteration beyond budget.

## Acceptance-criteria status

- Diagnostic predeclared, non-vacuous, independently reproducible — **met** (pure classifier + confusion fixture; the ceiling test reports TRIGGERED on a planted clearing shape; CID-bound result reproduces #841).
- No runtime code lands when NOT TRIGGERED — **met** (test + docs only).
- Production horizon ladder clears the frozen sequence threshold OR the negative is published — **negative published** (GENERATION-NOT-ESTABLISHED).
- README/ROADMAP/RESEARCH and conformance scope match the verdict — **met** (RESEARCH.md S3 item C added; ROADMAP/README carry no S3 generation claim; conformance unchanged).

## Tests and verification

- New harness: fixture teeth (non-ignored, CI-run) — `classifier_distinguishes_every_cause`, `reachability_bar_can_report_both_verdicts` (proves the instrument can report TRIGGERED — it can fail), `at0_reachable_drop_is_zero_under_step0_invariance`, plus metrics/median teeth. Heavy run `state_trajectory_gate_run_842` is `#[ignore]`/bundle-gated and reproduces #841 + asserts the NOT-TRIGGERED gate.
- Full local ladder green: `cargo fmt --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` (0 warnings); `cargo test --workspace --no-fail-fast --offline` (177 suites, 0 failures); `python3 scripts/check_claim_wording.py`; register conformance R1 `check-model` (31 ids, unchanged), R2/R3 `repo-conformance`, R4 `audit-deferral`, R5 `audit-limits`, R6 `cargo deny check bans` (ok); gnaf firewall/scan.

## Evidence artifacts

- `docs/state_trajectory_gate_842.md` — the frozen diagnostic + the S3 verdict record.
- `docs/state_trajectory_gate_842_result.json` — CID-bound result (`result_cid blake3:86376394…`).
- `crates/uor-r4-api/tests/state_trajectory_gate_842.rs` — the harness.

## Compatibility and migration

None. No serving, runtime, format, or compiler behavior changes; no artifact/section is added; old artifacts are unaffected (absent-section identity preserved by construction — nothing is added).

## Conformance effects

RF-14 / RF-21 / RF-22 / RF-29 evidence language (extends existing capabilities). **No new built capability, no `model/ids.toml` row, no `CONFORMANCE.md` regeneration** — evaluation/instrument infrastructure, identical treatment to #841 and #840. P-4, allocation-free steady state, and determinism are preserved (no runtime touched).

## Claim-boundary changes

The S3 free-running generation claim is recorded as **not established** (`GENERATION-NOT-ESTABLISHED`) at this representation scope, and the bounded trajectory-state mechanism is **`NOT TRIGGERED`**. This meets the programme global falsifier (no bounded correction or state reduces the frozen free-running gap → the deployed artifact is a teacher-forced continuation/retrieval system, not a generative engine at this scope) and is consistent with #840.

## Negative / unavailable results

This PR's headline result is an evidence-backed negative (NOT TRIGGERED / GENERATION-NOT-ESTABLISHED). Sampled-mode and judge-bearing families remain `UNAVAILABLE` (no decoder/seed/judge identity pinned). Re-entry requires a representation with cross-step free-running memory under the frozen #841 §6 bar.

## Intentionally excluded

No runtime/format/compiler/model change (NOT TRIGGERED). The S3 tracker #824 stage verdict (PROMOTE/REVISE/LIMIT/RETIRE) over its three completed children is a separate tracker action, not part of this issue.

Closes #842


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


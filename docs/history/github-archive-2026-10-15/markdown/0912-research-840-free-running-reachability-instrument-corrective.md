# #912 — research(#840): free-running reachability instrument — corrective observation not launched (GENERATION-NOT-ESTABLISHED)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-22T00:58:18Z
- **Updated:** 2026-08-22T01:25:51Z
- **Closed:** 2026-08-22T01:19:51Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/912
- **Merged:** 2026-08-22T01:19:51Z
- **Merge commit:** 35f8cc876fde07e1c1fdb193d37959fb7e9e08a6
- **Base / Head:** main / UOR-Foundation:issue-840-student-prefix-corrections
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

S3 item B (#840) begins against the promoted skip-mix representation (RF-31, activated in #910; the maintainer HOLD was lifted by the S1 PROMOTE). Its scope is a bounded multi-round corrective-observation experiment governed by the repository long-run discipline, which requires a binding cheap instrument and reachability arithmetic **before** any run measured in hours.

This PR delivers that instrument and its executed result. **Outcome: the corrective run is NOT launched** — the reachable ceiling is ~100× below the frozen §6 bar and the activated lane regresses free-running — so the disposition is **`GENERATION-NOT-ESTABLISHED`** for corrective student-prefix observation against this representation. Evidence-backed negative result; no serving behavior changes.

## What the instrument measures (teacher-free, deployed R4Engine)

Re-measures the frozen #841 prompt-family v1 free-running gap on TWO engines built from the same recompiled sections (the #908 machinery): `base` (empty lane sections — reproduces #841 exactly, and its base graph CID is byte-identical to #908's `base_artifact_cid`) and `skip` (real fitted SKMX/PSIB — the deployed RF-31 lane, reached via `predict_decision`).

| metric (H=32, greedy) | base | skip-mix RF-31 |
|---|---|---|
| median first-divergence | 0 | 0 |
| diverged-at-step-0 | 590‰ | 620‰ |
| ≤4-period cycle collapse | 710‰ | 1000‰ |
| matched teacher-forced agreement | 304‰ | 348‰ |
| suffix-locality (FR ≡ suffix-only, /100) | 99 | 19 |
| free-running Graph-path footprint | ~1‰ | ~1‰ |
| rollouts changed by the lane | — | 100/100 |

The lane helps *teacher-forced* prediction (+44‰, consistent with #908's +28.45‰ deployed causal) but moves *free-running* away from coherence (exposure-bias amplification). The correctable (graph/skip-mix) footprint is ~1‰ — ~100× below the §6 bar's required 100‰ diverged-at-0 drop.

## Scope

- Reachability instrument `crates/uor-r4-api/tests/free_running_reachability_840.rs` (fixture teeth non-ignored; `free_running_reachability_run_840` ignored/bundle-gated).
- Evidence record `docs/free_running_reachability_840.md` + CID-bound result `docs/free_running_reachability_840_result.json` (`result_cid blake3:e9f48e20…`).
- Reconcile `docs/RESEARCH.md` with the S3 item B negative result.

## Non-goals (intentionally excluded)

- No corrective-observation subsystem, no corrective rounds, no corrective compile — the run contract gates the launch out (reachable ceiling below the target effect).
- No serving/runtime/format/compiler code changes; served output is byte-identical.
- No `model/ids.toml` row, no new capability, no `CONFORMANCE.md` regeneration (evaluation/instrument infra extending RF-14/21/22/29, identical treatment to #841).
- The formal S3 generation-claim decision (promote/revise/limit/retire) belongs to #842, not this PR.

## Acceptance-criteria status (#840)

#840's acceptance criteria concern a corrective arm actually launched. Per its run contract's do-not-launch clause ("Do not launch the run if the reachable ceiling is below the target effect"), the corrective arm is not built; the reachability instrument + arithmetic is the completion evidence, recorded truthfully as `GENERATION-NOT-ESTABLISHED` / representation-limiting. The frozen #841 §6 bar and the #838 selective-prediction gates are unchanged (the #887 discipline).

## Verification

- `cargo fmt --check` ✓
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` ✓
- `python3 scripts/check_claim_wording.py` ✓
- `cargo run -p xtask -- validate` (R1 check-model + R4 audit-deferral + R5 audit-limits + gnaf) ✓ — `CONFORMANCE.md` current, no regeneration
- `cargo test -p uor-r4-api` ✓ — the 3 new fixture teeth pass; the heavy instrument is bundle-gated/ignored in CI
- Instrument run (teacher-free) reproduces #841's base numbers exactly and #908's base graph CID; result deterministic (identical `result_cid` across runs)
- No dependency/Cargo.lock change → R6 (`cargo deny bans`), `cargo audit`, and the wasm build are unaffected. The full workspace ladder re-runs in the merge queue (CI split; the queue is the binding verdict).

## Evidence artifacts

`docs/free_running_reachability_840.md` (record) + `docs/free_running_reachability_840_result.json` (`result_cid blake3:e9f48e20…`; corpus.meta `blake3:aa9d1767…`, the attested #833 broad-clean bundle).

## Compatibility / migration

None. Test + docs only; no format, protocol, or serving change.

## Conformance effects

None to `CONFORMANCE.md`. Evaluation/instrument infrastructure extending RF-14/21/22/29 evidence language.

## Claim-boundary changes

Records that free-running generation is **not established** under the promoted skip-mix representation — meeting the programme global falsifier (bounded student-prefix correction does not reduce the frozen free-running gap → the deployed artifact is a teacher-forced continuation/retrieval system, not a generative engine at this scope). No positive claim is added.

Closes #840.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_018jYv6QJiKAerYh9yJWUkgd


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


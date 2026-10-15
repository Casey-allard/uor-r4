# #862 — architecture: designate one normative R4G1 scorer (#831)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-20T02:28:26Z
- **Updated:** 2026-10-09T13:08:54Z
- **Closed:** 2026-08-20T02:50:15Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/862
- **Merged:** 2026-08-20T02:50:15Z
- **Merge commit:** 1c377df7fcc3fffca30f32db5aeec4df32d89131
- **Base / Head:** main / UOR-Foundation:issue-831-normative-scorer
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

The repo carried several scoring surfaces whose production-vs-reference roles were not
expressed as one normative production semantics (the #831 problem). On the deployed path the
served token is chosen by `R4G1Runtime` while the serve/abstain decision is chosen by
`R4Engine`/`GraphScorer` on the *same* position — two implementations that are **not** exactly
equivalent (the Cayley–Dickson term runs only in the `R4G1Runtime` generation slice; see
`crates/uor-r4-graph-certify/tests/r4g1_cd_ab.rs`). A certifier (Gate C) measurement was
therefore discussed as though it applied to what the deployed path serves — silent drift that
would invalidate downstream S1–S7 gates.

**Outcome:** one normative scorer is designated and documented; the reference/certifier
scorers are given distinct identities and claim boundaries; divergence is made fail-closed;
and a differential with a planted negative proves the boundary has teeth. Closes #831.

## Scope and non-goals

**In scope:** an ADR designating the normative scorer; a reachability inventory; explicit
scoping of the reference/certifier/legacy/router surfaces; a differential + planted-negative +
reachability + fail-closed test; conformance-register and doc reconciliation.

**Non-goals (per the issue):** improving model quality or changing the decode policy; removing
reference implementations where scoping suffices; calling bit-different scorers equivalent.
No format/ABI bytes change (convergence is a non-goal; distinct-identity scoping is used).

## Decision (ADR-0001)

The one normative scorer for deployed inference is the **deployed R4G1 runtime scoring path**
(`uor-r4-graph-runtime` `R4G1Runtime` + `scoring`), specified by
`uor-r4-graph-format::scoring_semantics` v1.0.0 and constrained by `docs/inference_contract.md`.
Tie-break (ScoreQ desc, id asc), saturating accumulation, no-double-counting, the single
`ScoreStatus` scorer-status space, patch owner (`try_push_patch`, fail-closed), and witness
owner (replay mismatch fail-closed) are named. `GraphScorer` (reference/certifier + D4 policy
resolver), the Gate C harness, the retired `score_candidates_legacy`, and the `uor-r4-router`
are explicitly scoped out of the served-selection role.

## Acceptance-criteria status

- [x] Exactly one implementation/specification named normative for deployed inference — ADR §Decision.1.
- [x] All production entry points and certificates demonstrate reachability — ADR §Reachability + test `reference_and_normative_reachable_from_same_artifact`.
- [x] Reference/certifier-only implementations explicitly scoped and differentially tested — ADR §Decision.2 + `spec_and_runtime_accumulators_agree*` / `spec_and_runtime_selectors_agree`.
- [x] Patch and witness semantics have one normative owner and incompatible artifacts fail closed — ADR §Decision.1 + `incompatible_artifacts_fail_closed`.
- [x] A planted semantic divergence fails the relevant test — `planted_divergence_is_detected`.

## Tests and verification

New: `crates/uor-r4-api/tests/normative_scorer_831.rs` (7 tests, default suite) — spec-vs-runtime
accumulator/selector differential across three crates, planted negatives (higher-id-wins
selector + wrapping accumulator) detected, no-double-counting per implementation, single-source
scorer status space, reachability of both scorers from one artifact, and the incompatible-artifact
fail-closed boundary.

Gates run locally (real Mac, pinned toolchain), all green:
`cargo fmt --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
`cargo check -p uor-r4-graph-format --no-default-features (+ --features alloc)`;
`python3 scripts/check_claim_wording.py`; register conformance R1 (`check-model`), R4
(`audit-deferral`), R5 (`audit-limits`), R2/R3 (`repo-conformance`), R6 (`cargo deny check bans`);
`cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib`; and the full
`cargo test --workspace --no-fail-fast` (every suite green; the live-teacher parity BDD is the
documented slow mode and its graph scenarios skip on the pre-existing quality gate, unrelated
to this change).

## Evidence artifacts

`docs/adr/0001-normative-r4g1-scorer.md` (decision record) and `docs/normative_scorer_831.md`
(per-issue evidence record + acceptance map).

## Compatibility and migration

No format/ABI/bytes change. `scoring_semantics` v1.0.0 unchanged; the ADR names it as the
single spec owner and records that any future change to tie-break/saturation/accumulation
versions the spec and supersedes the ADR.

## Conformance effects

`model/ids.toml` RF-23 (`r4g1_runtime`, `normative-runtime`/`deployed-serving`) evidence
extended to cite the ADR; `CONFORMANCE.md` regenerated via `xtask check-model --write` (generated
file, not hand-edited). No new RF id (reuse/extend, per the issue). ADR maps RF-09/10/22/15/21
to their scopes.

## Claim-boundary changes

Establishes semantic ownership + reachability; makes reference↔normative divergence fail closed
rather than silent. Does **not** claim the two scorers are exactly equivalent and does **not**
establish any generation-quality result.

## Negative / unavailable results

None asserted as PASS. The reference scorer's EXCT path correctly fails closed without the
pinned teacher container (used to construct the reachability fixture).

## Intentionally excluded

Per-token **CID-bound attribution suites** are the deliverable of sibling item #832 (the ADR
names the attribution owner; the suites are out of scope here). No runtime convergence/refactor
(non-goal). `obs-text/` and `tf1-pkg/` untracked user artifacts were deliberately not staged.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


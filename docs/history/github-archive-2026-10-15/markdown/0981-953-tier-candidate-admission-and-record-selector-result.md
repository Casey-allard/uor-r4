# #981 — #953: tier candidate admission and record selector result

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-28T01:15:36Z
- **Updated:** 2026-08-28T01:18:47Z
- **Closed:** 2026-08-28T01:16:51Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/981
- **Merged:** 2026-08-28T01:16:51Z
- **Merge commit:** 3520ae33529b2300ce923a6476a74fa0648fa816
- **Base / Head:** main / UOR-Foundation:issue-953-tiered-admission-repair
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- add versioned `PrimaryThenAdjacentSpinFallbackV1` admission: I1/I2/ordered-sentence/divisor primary; adjacent-spin always consulted and truthfully traced, but examined/admitted only on empty-primary fallback
- propagate policy identity plus physical availability/examined/admitted accounting through schema-2 generation reports without changing H4 cost or selection
- preserve the frozen natural-agreement fixture identities and historical support record; bind repaired support as `blake3:aab38fc513521cdd495bad74cc4a87754ec43ecdef5cb6e098b101412d3d7fe9`
- preserve the #969 empty-primary fallback and its record

## Frozen outcome

The support preflight now matches exactly under equal left/right work:

- step 1: 7 rows, 8 available, 3 examined/admitted, `{still}`, 5 keys, 5 comparisons
- step 2: 7 rows, 11 available, 6 examined/admitted, `{run,runs}`, 6 keys, 12 comparisons
- the physical adjacent row exposes 5 available entries at both steps, with fallback false and 0 examined/admitted

The one permitted four-arm run plus internal replay produced:

- left/full: `still run`
- right/full: `still run`
- left/state-disabled: `still runs`
- right/state-disabled: `still runs`

Record: `blake3:dfe03d4c56f7e5e9cf48d524f2f0b10482c4b3b85fae152dd29c64543caa0b79`.

The terminal remains `REVISE_I1_GENERATOR_IN_PLACE`: admission is repaired, but the identity-derived candidate representation/scorer does not distinguish the required same-object, order-sensitive choice. #953 stays open; #973 and #954 stay blocked. The next bounded in-place hypothesis is construction-only candidate-context placement under same-frame, disabled, placement-permuted, order-shuffled, and held-out anti-recall controls.

## Focused verification

- natural agreement freeze/historical support/repaired preflight: 3 passed, explicit four-arm evidence ignored in routine replay
- explicit natural four-arm evidence: passed once with complete byte-identical replay
- existing #953 relabel regression: passed
- #969 adjacent-only fallback and causal-path regression: passed
- #958 policy/trace/bounds regressions: 8 passed
- `cargo check -p uor-r4-core --all-targets --offline`
- `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib`
- `cargo fmt --all -- --check`
- `python3 scripts/check_claim_wording.py`
- `git diff --check`

Workspace-wide tests/clippy, no-std ladders, corpus/model/teacher work, BDD, Gate C, kappa reproduction, audit, fuzz, formal proof, conformance, product QA, performance, and release qualification are `NOT_RUN`.

References #953

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


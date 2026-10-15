# #872 — spec(#835): define persistent prompt-conditioned predictive state (S1 item A)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-20T16:35:54Z
- **Updated:** 2026-10-09T14:17:18Z
- **Closed:** 2026-08-20T16:56:44Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/872
- **Merged:** 2026-08-20T16:56:44Z
- **Merge commit:** 8785aedd717e8746f501bb265799efc886b61446
- **Base / Head:** main / UOR-Foundation:issue-835-prompt-state-spec
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

S1 (#822) needs a frozen, experimentable contract for persistent prompt-conditioned state
before any arm can be fit (#834) or lowered (#836). #784 showed distinct prompts converge on a
shared continuation and the deployed NGRAM/EXCT path is suffix-local; "bidirectional anchors"
was not yet a causal, bounded, packable semantics. This PR delivers item A of #822: a
specification that freezes the state semantics, the compiler–runtime boundary, the witness
schema, the budgets, and the causal benchmark with its falsifiers — plus an executable
reference model that proves the contract is deterministic, bounded, and benchmarkable.

## Scope and non-goals

**In scope.** A reference-only / off-serving-path specification (`docs/prompt_state_spec_835.md`)
and its executable reference model + evidence (`crates/uor-r4-api/tests/prompt_state_spec_835.rs`).

**Non-goals (honored).** No access to future completion tokens at inference; sampling /
repetition penalties / prompt-template injection are not treated as persistent state; no
commitment to W(3,3) or any geometry before the #834 bake-off. This PR does **not** implement the
deployed capability (that is #836), and does **not** establish that persistent state improves
prompt causality (that is measured in #834).

## Acceptance-criteria status

- [x] Reference semantics determine the same state and score contributions from the same prompt,
  artifact, and prior state — spec §6; test `determinism`.
- [x] Every lane has a causal benchmark intervention and a bounded deployed representation —
  spec §2 lane table; §11.
- [x] Absent sections preserve historical behavior; invalid capacities/indices fail with typed
  errors — spec §8 (compatibility obligation carried to #836) and §6; tests
  `invalid_capacity_and_index_are_typed_errors`.
- [x] The specification includes byte/operation/state budgets and explicit witness replay
  semantics — spec §10 and §9; test `witness_replay`.
- [x] No deployed transition uses multiply, divide, floating point, heap allocation, clock,
  network, or RNG — spec §3/§5 allowed-op enumeration; the deployed-path proof (allocation
  census / P-4 scan) is discharged by #836.

## Tests and verification

`crates/uor-r4-api/tests/prompt_state_spec_835.rs` (11 tests, default `cargo test`): determinism,
`state_is_fixed_capacity`, `transition_is_causal`, `contributions_independent_of_decode`,
`capacity_edges_and_eviction`, `tie_break_evicts_higher_key`, `saturation_no_overflow`,
`reset_and_continuation`, `witness_replay`, `invalid_capacity_and_index_are_typed_errors`, and
`planted_negatives_have_teeth`.

Local gates run green: `cargo fmt --check`; `cargo clippy -p uor-r4-api --all-targets
--all-features -D warnings`; `cargo test -p uor-r4-api --test prompt_state_spec_835`;
`python3 scripts/check_claim_wording.py`; `cargo run -p xtask -- validate` (R1 check-model 29 ids,
R4 audit-deferral, R5 audit-limits, gnaf firewall/root/scan/conformance). The merge queue
re-verifies the full ladder.

## Evidence artifacts

- `docs/prompt_state_spec_835.md` — the frozen contract (claim-labeled per
  `docs/formal_vocabulary.md`).
- `crates/uor-r4-api/tests/prompt_state_spec_835.rs` — the executable reference model, binding the
  frozen #832 control vocabulary (`ControlKind`, `MetricStatus`, `is_degenerate_control`) so the
  planted negatives speak the same language as the committed `s1-causal-prompt-pairs` suite.

## Anti-vacuity / planted negatives

`planted_negatives_have_teeth` asserts the causal arm is non-degenerate (1000‰) and separates
from the prompt-swap and suffix-only nulls, and that a constant (non-causal) arm is flagged
degenerate against the null — so a zero reading means "no effect", not "broken harness". The
control arm is asserted non-degenerate before any comparison.

## Compatibility and migration

The proposed optional R4G1 `PSTATE` section lives in the optional-kind space with a
`feature_bits_required` gate; an artifact without the section, or a client without the feature
bit, behaves identically to the current baseline. Any request-state ABI change needs versioned
negotiation and a migration/recompile note. This PR changes no artifact bytes and no runtime
behavior; it is specification + reference-model evidence only.

## Conformance effects

Maps to and extends the evidence of existing capabilities RF-01, RF-21, RF-22, RF-27, RF-28. It
introduces **no** new built RF capability, adds **no** `model/ids.toml` row, and regenerates
**no** `CONFORMANCE.md` (the #832 precedent for a spec/infrastructure leaf). The built-capability
order (`ids.toml` → Gherkin → failing marker test → implementation → regenerated `CONFORMANCE.md`)
applies to the deployed lowering in #836.

## Claim-boundary changes

The reference model is reference-only / off-serving-path and is not credited as deployed-serving
evidence. The deployed-representation guarantees (allocation-freedom, multiplication-free hot
path) are stated as **Definitions/Assumptions** the #836 lowering must discharge, labeled
Unproven here — not asserted proven by this specification.

## Negative / unavailable results

None. This PR freezes a contract; the S1 causal result (positive, negative, or not-triggered) is
the outcome of #834 against these controls.

## Intentionally excluded

The untracked user directories `obs-text/` and `tf1-pkg/` are not staged. No `model/ids.toml`,
`CONFORMANCE.md`, `model/ledger.toml`, or Gherkin changes (out of scope for a spec leaf).

Closes #835.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


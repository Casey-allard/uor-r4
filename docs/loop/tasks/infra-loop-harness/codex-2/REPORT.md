# REPORT: infra-loop-harness, codex-2

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- The codex-1 fresh-context review rejected the control because matching vocabulary was enough:
  “D85 is rejected; the float master remains the default” could pass a rule that only called
  `contains`.
- The first decision fixture similarly proved that a phrase was absent, not that the decision
  cell contradicted the policy. The old check also scanned the whole row, so a correct sentence
  in the rationale or rejected-alternatives column could satisfy it.
- The first model pages described the native path as implemented while the roadmap and REPORT
  said its state schema, operator registry, replay and trainer did not exist.
- The first report used a whole-repository formatter as task evidence; that formatter proposed
  thousands of unrelated historical changes. Only package-scoped Rust formatting is used here.
- The prebuilt `cargo xtask` alias was absent; the equivalent repository checks were invoked with
  `cargo run --locked -p xtask -- ...`.
- The full harness check still reports 311 existing warnings, including stale claims, presence
  and prep records. It has zero errors; those unrelated warnings are not hidden as passes.
- The recovered final repair did not compile: a filtered iterator called `and_then` instead of
  `filter_map`.
- Its fixture still represented the pre-repair contract, so the first rerun could not prove the
  new protocol-marker or model-status requirements. The fixture now declares the exact five model
  surfaces, their pending/historical status line and the canonical protocol pointer.
- The first full-package rerun after the second review repair reported 663 passed, 20 failed and
  1 ignored. The test temp directories were under `local/codex/tmp`, inside the shared checkout,
  so the harness's R10 write guard correctly refused the synthetic ledger, receipt, ratchet and
  multi-lab writes. This is an instrument-location failure, not a passing run or a product result.
- The initial formatting check found one rustfmt layout mismatch in the strengthened path
  assertion. The package was formatted; the rerun remains pending.

## What changed

`native-space.json` now stores one canonical assertion. The checker requires its exact marker in
the charter, plan, roadmap item, protocol, change log and every model surface; requires the AGENTS
bullet and milestone rule to equal the assertion; and requires the D85 decision cell to equal the
canonical assertion plus its fixed historical-boundary suffix. Matching text cannot hide in the
rationale or rejected alternatives. The checker also proves every declared parity field and
forbidden default-path phrase is part of the canonical assertion.

The final repair stores the protocol marker and pending/historical model-status line in
`native-space.json`, requires both values literally, checks all five model surfaces in fixed
order, and requires the harness skill pointer to carry exactly one canonical protocol marker.
The isolated fixtures now exercise the same complete shape as the live repository.

The test suite now covers a complete fixture, every missing source, a malformed or contradictory
charter block, a decision corrected only in its rationale, wrong skill name and pointer, missing
plan/roadmap markers, a contradictory AGENTS sentence, a reworded milestone rule, missing model
markers and weakened canonical/parity/forbidden fields.

The second review repair adds the required owner-approval section to the active `codex-2` ORDER,
attributes each native-space finding to the source that actually failed instead of always naming
the contract, accepts the roadmap's valid `ready` and `done` lifecycle states, and adds
completed-item and unknown-status fixtures. It also adds the existing `uor-r4-discrete` CUDA/CPU
path and the integer `discrete_d11.rs` serving path to the native-state item's scope, and aligns
the charter, skill and change-log float exception with D85's exact reference-or-diagnostic rule.

Model documentation now distinguishes the D85 contract from the implementation that is still
pending. `docs/harness/claims.json` is identified as generated claim state. The rule registry's
stale 53/13 prose is corrected to 54/14.

Size: the original repair was about 730 added/deleted lines across 21 files. The recovered final
marker/status repair adds about 383 changed lines across nine files. Both exceed their ORDER
estimates because they replace presence checks with exact cross-record contracts and isolated
contradiction fixtures. Size is reported, not capped (D48); no unrelated source was changed.

## Measurements

- Native-space focused tests: 5 passed.
- Native-space focused tests after the second review repair: 7 passed.
- Full harness package: 682 library tests passed, 1 ignored; the binary, every integration target
  and the doc test passed.
- Full harness package after the second review repair: 663 passed, 20 failed, 1 ignored; the
  failures are the shared-checkout R10 refusal caused by placing test temp directories under
  `local/codex/tmp`. This run is NOT a pass and must be rerun with `TMPDIR` outside the checkout.
- Final clean full harness package rerun with `TMPDIR=/var/tmp/uor-harness.KQCj53`: 684 passed,
  0 failed, 1 ignored; the binary, every integration target and the doc test passed.
- Native-space rule on the real repository: 1 selected, 0 failed.
- Full harness inventory: 346 rules, 0 errors, 311 existing warnings, exit 0.
- Harness tests: 682 library tests passed, 1 ignored; every integration and doc test passed.
- Roadmap registry: 145 items valid, 26 ready, 11 held, 1 decision and 107 done.
- Stack contracts: 5 valid; generated views current.

No model, training, speed, capability or export result was measured.

## Checks run at the head

Run at the final task head unless explicitly identified as an earlier superseded run:

- `cargo fmt --package uor-r4-harness -- --check` - passed.
- `cargo fmt --package xtask -- --check` - passed.
- `git diff --check` - passed.
- `CARGO_TARGET_DIR=local/harness-target cargo test --locked -p uor-r4-harness` - all targets
  passed.
- Second-repair focused run:
  `CARGO_TARGET_DIR=local/harness-target cargo test --locked -p uor-r4-harness native_space_contract`
  - 7 passed.
- Second-repair full run with in-checkout `TMPDIR` - 663 passed, 20 failed, 1 ignored; not a pass.
- Final clean full run with `TMPDIR=/var/tmp/uor-harness.KQCj53` - 684 passed, 0 failed,
  1 ignored; all targets passed.
- `CARGO_TARGET_DIR=local/harness-target cargo clippy --locked -p uor-r4-harness --all-targets --
  -D warnings` - passed.
- `CARGO_TARGET_DIR=local/harness-target cargo build --locked --release -p uor-r4-harness` -
  passed.
- `UOR_LAB=codex local/harness-target/release/uor-harness check --only
  harness.native_space_contract` - passed.
- `UOR_LAB=codex local/harness-target/release/uor-harness check` - 346 rules, 0 errors, 311
  existing warnings.
- `CARGO_TARGET_DIR=local/harness-target cargo run --locked -p xtask -- stack-contract` - passed.
- `CARGO_TARGET_DIR=local/harness-target cargo run --locked -p xtask -- roadmap check` - passed.
- `UOR_LAB=codex local/harness-target/release/uor-harness loop report --check --order
  docs/loop/tasks/infra-loop-harness/codex-2/ORDER.md` - passed after this REPORT was added.

## Skills used

- `uor-research-architect`: kept the repair at the contract boundary and made no model claim.
- `working-discipline`: re-read the reviewed failure, used focused probes and reported actual
  commands and limitations.
- `scope-guard`: separated the active codex-2 repair from historical codex-1 and checked the
  generated claim record explicitly.
- `solution-integrity`: replaced token presence with a structured affirmation and planted
  contradiction cases.
- `native-space-cycle`: supplied the state, operator, transition, optimizer and serialization
  terms encoded in the canonical assertion.

## Still open

- The new repair head needs its own fresh-context exact-head review.
- Clippy, full harness check, report check and PR check still need a clean rerun at the committed
  second-repair head.
- The first native implementation remains `native-space-state-contract`; no model run is open.
- Existing float/export measurements remain historical and do not qualify D85.
- PR delivery, queue and post-merge verification remain after review.

## Plain summary

The first version checked that important words appeared somewhere in each file. That was too weak:
it could accept a sentence that rejected the policy. The repair defines one exact statement of the
policy and requires that statement, not a paraphrase, in each authoritative record. It also reads
only the D85 decision itself rather than letting matching words from the reasons or rejected
alternatives satisfy the check. Model pages now say clearly that the native path is the required
destination, while its state schema, shared operators and bit-identical replay are still the next
bounded implementation task.

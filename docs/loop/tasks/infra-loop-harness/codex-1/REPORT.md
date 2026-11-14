# REPORT: infra-loop-harness, codex-1

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- The first focused native-space test run failed because its harness-pointer fixture did not name
  the authoritative protocol. The fixture was corrected; the checker deliberately requires that
  link.
- The first full harness run then failed on two real-record drifts: D85 said "same typed operator
  definitions" instead of the contract's "same typed operators", and the roadmap item required
  raw-word parity without the literal "bit-identical". Both records were aligned to the invariant.
- The milestone contradiction test initially proved only that D85 was absent, not that one parity
  field was missing. The diagnostic now names the missing field, and the test plants that exact
  fault.
- `cargo fmt --all -- --check` is unusable as a task signal in this checkout: the installed
  formatter proposes thousands of changes in untouched historical files. Package-scoped
  formatting passed. No unrelated file was reformatted.
- `cargo xtask ...` is unavailable because the local alias is not installed. The same repository
  checks were run explicitly with `cargo run --locked -p xtask -- ...`.
- The full harness check reports 311 remaining warnings, including stale claims, presence and prep
  records. It reports zero errors. This unit removed its own unclaimed-branch warning by claiming
  the branch; the remaining warnings predate it and are not hidden as passes.

## What changed

D85 now makes one persistent native geometric state the default model path. The owner decision
is carried consistently through the charter, agent onboarding, plan, decision log, milestone
owner rules, model documentation, change log and a mandatory `native-space-cycle` skill. The new
skill makes a state/operator inventory and bit-identical transition fixture prerequisites for
model work; it forbids an F32 master plus export-gap conversion as the default path.

`harness.native_space_contract` is registered, dispatched and covered by unit tests. It verifies
the owner decision's substance, the charter marker block, both skills, the harness pointer, the
plan and roadmap item, AGENTS, milestone parity fields, the change log, owner-path registration
and every required model surface. Missing or contradictory records fail with an error.

A new ready roadmap item, `native-space-state-contract`, defines the next implementation unit:
the versioned state schema, operator registry, finite replay fixture and its planted fault. This
unit does not implement or run a model.

Size: 1,275 added/deleted lines across 32 files, about 1.59 times the ORDER's 800-line upper
estimate. By class this is about 665 code/registry lines, 603 documentation/record lines and
7 generated lines. Tests are embedded in the touched harness module.

## Measurements

- The native-space rule on the real repository: 1 rule selected, 0 failed.
- The full harness inventory: 346 rules, 0 errors, 311 warnings, exit 0.
- Harness tests: 679 library tests passed, 1 ignored; the binary, every integration target and
  the doc test passed.
- The roadmap registry: 145 items valid, 26 ready, 11 held, 1 decision and 107 done.
- Stack contracts: 5 contracts valid; both generated views current.

No model, training, speed, capability or export result was measured. This is a control-plane
unit.

## Checks run at the head

Run at the final task head unless a check is explicitly identified as an earlier superseded run:

- `cargo fmt --package uor-r4-harness -- --check` - passed.
- `git diff --check` - passed.
- `CARGO_TARGET_DIR=local/harness-target cargo test --locked -p uor-r4-harness` - all targets
  passed.
- `CARGO_TARGET_DIR=local/harness-target cargo clippy --locked -p uor-r4-harness --all-targets --
  -D warnings` - passed.
- `CARGO_TARGET_DIR=local/harness-target cargo build --locked --release -p uor-r4-harness` -
  passed.
- `UOR_LAB=codex local/harness-target/release/uor-harness check --only
  harness.native_space_contract` - passed.
- `UOR_LAB=codex local/harness-target/release/uor-harness check` - 346 rules, 0 errors, 311
  existing warnings.
- `CARGO_TARGET_DIR=local/harness-target cargo run --locked -p xtask -- stack-contract` - passed,
  generated files current.
- `CARGO_TARGET_DIR=local/harness-target cargo run --locked -p xtask -- roadmap check` - passed,
  145 items.
- `UOR_LAB=codex local/harness-target/release/uor-harness loop report --check --order
  docs/loop/tasks/infra-loop-harness/codex-1/ORDER.md` - passed after this REPORT was added.

## Skills used

- `uor-research-architect`: reconciled the change with the charter, current source and artifact
  boundaries.
- `working-discipline`: read the live state and plan first, checked the repository against the
  notes, used focused checks and reported negatives first.
- `scope-guard`: audited the inherited dirty worktree and aligned the ORDER with the owner-approved
  model and harness surfaces.
- `solution-integrity`: identified the unchecked parity fields and turned them into planted-fault
  tests instead of documentation-only assertions.
- `native-space-cycle`: supplied the state, operator, transition, optimizer and serialization
  checklist now enforced by the harness.

## Still open

- `native-space-state-contract` is ready but not implemented. It must inventory the current
  training and serving call chains before any default-path training run.
- D85 changes the default architecture boundary. Existing float/export measurements remain
  historical evidence and are not native-space qualification.
- This branch still needs its exact-head fresh-context review, PR delivery and post-merge
  verification under CHARTER §5.5.

## Plain summary

The owner clarified that the project should not train one model in floating point and then convert
it into a different model for serving. Training and serving must carry the same integer or
fixed-point geometric state, use the same typed operations, and make identical route, memory and
token decisions. This change writes that rule into the project’s governing records and gives the
harness a red check that notices when those records drift apart. It also creates the next bounded
task: define every field and shared operation in that state, then prove with a planted-fault test
that the training and serving paths produce identical raw integer outputs before anyone starts a
model run.

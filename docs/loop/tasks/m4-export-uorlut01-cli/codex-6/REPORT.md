# REPORT: m4-export-uorlut01-cli, codex-6

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim m4-export-uorlut01-cli --lab-topic q31-token-objective` was refused with exit 2
  because PR #3983 claims overlapping `crates/uor-r4-integer/src/discrete_d11.rs` scope. This unit
  adds one test and records, and did not touch that file.
- The first target mapping used `q12 << 19`, which overflows the Q31 domain when a writer value is
  above 1.0 (the writer's i16 Q12 range is ±8). The update refused it; the target now uses
  `q12 << 16`, the writer's 1/8 range mapped into Q31's ±1.
- **The objective is a negative on this configuration.** The independent alignment dot fell from
  `2,476,676,547,100,606,464` to `2,142,903,011,938,598,912` over the 112 records, and the served
  token ids on the eight-token sequence were unchanged (`[125, 236, 10, 186, 8, 161, 92, 205]`
  before and after). The writer-block target under the current integer update does not improve the
  measured alignment. This closes the objective/update pairing, not the mechanism (CHARTER §4.1).

## What changed

- A new test, `crates/uor-r4-integer/tests/q31_token_objective.rs`, defines the first token
  objective:
  - tokens map to the synthetic trunk's writer rows (Q12, scaled into Q31);
  - at each transition the traced recurrence lane carries are the values and the next token's
    matching 4-word block is the target;
  - the shared oracle `Q31HamiltonModel::step_reference` trains 64 steps on 112 records; and
  - the test reports the independent integer dot before and after, the served token ids, and
    requires the artifact to carry exactly the trained 48 bytes.
- The ORDER's done-when is corrected to report the served tokens rather than require a change.
- One M4 notebook row records the negative.
- No production, format, CLI or existing-test change.

Size: one new test plus the loop and notebook records.

## Measurements

- Records: `112` (7 transitions × 2 recurrence layers × 8 lanes); sequence
  `[3, 5, 8, 13, 21, 34, 55, 89]`; training `64` steps; target scale `q12 << 16`.
- Alignment dot: `2,476,676,547,100,606,464` before → `2,142,903,011,938,598,912` after (fell).
- Served token ids: `[125, 236, 10, 186, 8, 161, 92, 205]` before and after (unchanged).
- The trained state changed and the `/12` artifact carries exactly those 48 bytes.
- No language, capability, speed, memory, M4-bar or milestone result is measured.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --test
  q31_token_objective` — `1` passed (the negative is measured, not asserted positive).
- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --lib q31_serving`
  — `8` passed.
- `cargo clippy --locked --no-deps -p uor-r4-integer --features serving-wasm,fbs-synthetic --test
  q31_token_objective` — clean.
- `git diff --check` — passed.
- `uor-harness check --only harness.native_space_contract` — 1 rule, 0 failed.
- `uor-harness loop report --check`, `loop pr-check --local` and the exact-head review — at the head.

## Skills used

- `native-space-cycle`: tokens, carries, update and artifact stay in one integer space; the target
  is a raw Q31 block, and no float is used in the serving path.
- `trace-back`: the alignment fell rather than rose; the first suspect (the initial `<< 19`
  overflow) was fixed, and the remaining fall is recorded as the configuration's result, not read
  as noise.
- `scope-guard`: avoided the file claimed by PR #3983 and every out-of-scope path.
- `test-freeze`: added a new test; changed or deleted none.
- `working-discipline`: recorded the negative as a mechanism-class measurement of one
  configuration, with the exact numbers.

## Still open

- The writer-block target with the current update does not improve alignment. The next
  configuration to test is a **normalized target** (unit-scale per block) and a loss/update that
  is checked against the same independent dot, or a different record construction (e.g., the
  post-drive `after` carry instead of `before`). A positive one is required before a GPU
  pre-registration can claim a mechanism.
- A real language-model training run and the M4 bar remain open.
- The org runner-capacity bottleneck remains an owner/PM-level issue.

## Plain summary

For the first time, the training target came from tokens: a token's row in the model's writer
table gave the direction the recurrence carry should point. The gate moved, but the measured
alignment got worse on the training sequence, and the served tokens did not change. That is a
real negative for this exact recipe, recorded with its numbers; it says the recipe needs a
normalized target or a different update, not that the one-space mechanism is closed.

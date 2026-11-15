# REPORT: m4-export-uorlut01-cli, codex-7

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim m4-export-uorlut01-cli --lab-topic q31-token-normalized` was refused with exit 2
  because PR #3983 claims overlapping `crates/uor-r4-integer/src/discrete_d11.rs` scope. The unit
  extends one test and the records; it did not touch that file.
- **The normalized target is also a negative.** The independent alignment dot fell from
  `18,514,432,582,351,337,035` to `17,116,574,668,483,326,999` over the same 112 records, and the
  served token ids were unchanged. Together with codex-6's raw target
  (`2,476,676,547,100,606,464 → 2,142,903,011,938,598,912`), two target scales both lower the dot.
- **Structural finding:** the state has one shared 4-word gate, while the records ask it to align
  112 lane carries toward 112 different next-token directions. A single rotation cannot fit them;
  the update moves to a compromise that lowers the aggregate dot. The next configuration needs
  per-lane (or per-lane-group) gates, which is a state and serving-contract change, or a different
  objective/update.
- The test asserts only the mechanics; the sign of the delta is the result, not an expectation.

## What changed

- `crates/uor-r4-integer/tests/q31_token_objective.rs` gains `normalized_target` (per-block unit
  Q31 scale, largest coordinate ±2^31) and `normalized_records`, plus
  `normalized_writer_targets_train_the_one_q31_gate`. The existing raw-target test is unchanged.
- One M4 notebook row records the second negative and the structural finding.
- No production, format, CLI or serving change.

Size: about 90 lines of test plus the loop and notebook records.

## Measurements

- Records: `112` (7 transitions × 2 layers × 8 lanes); sequence
  `[3, 5, 8, 13, 21, 34, 55, 89]`; training `64` steps.
- Raw target (codex-6): dot `2,476,676,547,100,606,464 → 2,142,903,011,938,598,912` (fell).
- Normalized target: dot `18,514,432,582,351,337,035 → 17,116,574,668,483,326,999` (fell).
- Served tokens: `[125, 236, 10, 186, 8, 161, 92, 205]` before and after in both cases.
- The trained state changed and the `/12` artifact carries exactly it.
- No language, capability, speed, memory, M4-bar or milestone result is measured.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --test
  q31_token_objective` — `2` passed (raw and normalized; both negatives measured).
- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --lib q31_serving`
  — `8` passed.
- `cargo clippy --locked --no-deps -p uor-r4-integer --features serving-wasm,fbs-synthetic --test
  q31_token_objective` — clean in the touched file.
- `git diff --check` — passed.
- `uor-harness check --only harness.native_space_contract` — 1 rule, 0 failed.
- `test_freeze` — GREEN with the `TEST-CHANGE:` line on the commit that extends the test.
- `uor-harness loop report --check`, `loop pr-check --local` and the exact-head review — at the head.

## Skills used

- `trace-back`: the same failure appeared at two target scales, so the suspect moved from the
  scale to the structure (one gate against many directions).
- `native-space-cycle`: the comparison stays in raw integer words and names the state/operator
  change the next configuration needs.
- `scope-guard`: avoided the file claimed by PR #3983 and every out-of-scope path.
- `test-freeze`: extended the test; the commit names the change and the existing raw case is
  unchanged.

## Still open

- **Per-lane gates (or per-lane-group gates):** this changes the state from one 48-byte gate to
  several and changes the `/12` section and the serving transition. It needs a decision card
  before implementation.
- **A different objective/update:** e.g., a served-token classification objective, or a loss whose
  independent measure is checked against it; also a decision-level choice.
- A real language-model training run and the M4 bar remain open; a GPU pre-registration needs a
  positive mechanism first.
- The org runner-capacity bottleneck remains an owner/PM issue.

## Plain summary

Normalizing the token target did not help: the alignment got worse again, and the served tokens
stayed the same. Two recipes failing the same way points at the state shape rather than the target:
one shared quaternion gate is being asked to point many different lanes in many different
directions at once. The next step is either more than one gate or a different objective, and that
is a design decision before more code.

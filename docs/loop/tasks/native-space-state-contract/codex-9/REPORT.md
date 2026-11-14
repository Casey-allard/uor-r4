# REPORT: native-space-state-contract, codex-9

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- The exact-head review found that `codex-8/REPORT.md` called two serving test functions six
  tests. The focused result itself already said `2 passed`.
- The ORDER's Clippy requirement was ambiguous because the report recorded only a
  package-wide `-D warnings` run that is red on unrelated existing diagnostics.

## What changed

The codex-8 REPORT now states the two actual serving test functions. It also records a normal,
package-scoped Clippy run that includes `uor-r4-integer` and `q31_serving.rs`, exits zero with
pre-existing warnings, and reports zero diagnostics whose path contains `q31_serving`.

The strict `-D warnings` baseline failure remains recorded. No lint was lowered, no warning was
suppressed, and no unrelated Rust file changed.

Size: two record corrections, about 10 changed lines plus this report.

## Measurements

- `cargo clippy --locked --no-deps -p uor-r4-integer --features serving-wasm --lib` exited 0.
- Filtered diagnostics naming `q31_serving.rs`: 0.
- The package-wide `-D warnings` run still exits 101 on existing unrelated diagnostics.
- No model measurement changed in this record-only repair.

## Checks run at the head

- Package-scoped normal Clippy with `--no-deps` - passed, with `q31_serving_diagnostics=0`.
- Package-wide integer Clippy with `-D warnings` - remains red on the existing baseline.
- `loop report --check` - passed.
- `git diff --check` - passed.
- Exact-head review records are posted on PR #3977; this report does not substitute for them.
- Queue recovery created duplicate canceled/success check runs at one SHA. After all results were
  green, GitHub still rejected direct enqueue with “5 of 5 required status checks are expected”;
  `codex-10` records the fresh-head delivery repair.

## Skills used

- `working-discipline`: kept the repair to the evidence that the reviewer identified.
- `solution-integrity`: separated a passing command with known warnings from a strict live because
  the altered lint posture would be a weaker check.
- `native-space-cycle`: confirmed this repair changes no state, operator or serialization path.

## Still open

- `IntegerStackSession` still needs to consume the loaded Q31 state in its actual token recurrence.
- PR CI, queue, merge and fresh-main verification remain.

## Plain summary

The serving boundary code did not need a correctness repair. The evidence record had the wrong test
count and did not distinguish existing package warnings from the new file's Clippy result. This
repair corrects those statements without suppressing lints or changing model behavior.

# REPORT: infra-loop-harness, deepseek-48 (prep.rs bugs #3041 and #3042)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item: `infra-loop-harness` (#2904). Base: `15d5728c7`. `References #3041, #3042`.

## What did not work, first

- **#3041 was live on the exact flow we had just used.** `test_fitness` returned the first anchored verdict (`return verdict` inside the loop), so a sheet that appends a later FIT under an older NOT_FIT still read NOT_FIT - the NOT_FIT→FIT re-review path #3052 exercised. Fixed: the loop records the latest verdict and returns it after scanning.
- **#3042 made the rendered block unusable in a sheet.** The clearance paragraph ended `(D22). ` with a trailing space and its continuation leaked nine spaces before `Every mechanism...`, so `git diff --check` rejected a pasted block. Fixed: moved to `FIT_CLEARANCE_NOTE` with clean text, and a test now calls `render_fit` itself and holds the emitted block to that standard (restoring the old inline literal fails the test). The `render_fit` doc comment that the first edit had attached to the const is restored.
- **Both fixes are additive and local.** No other function changes; no sheet is re-rendered (existing sheets keep their old text until next touched).

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-harness/src/prep.rs` | `test_fitness` returns the last anchored verdict; new `FIT_CLEARANCE_NOTE` const (clean text) used by `render_fit`; two inline regression tests |
| this folder | ORDER and this REPORT |

## Measurements

- `cargo test -p uor-r4-harness --lib prep`: **32 passed** (includes the two new tests). Full suite/clippy/fmt results recorded after they run.
- These are finite host tests; no GPU, no model compute.

## Checks run at the head

`cargo test -p uor-r4-harness`, clippy `-D warnings`, fmt, `loop report --check`, `loop pr-check --local`, `loop gates --full`; a non-author record is requested at the current head (code PR).

## Skills used

- `solution-integrity`: fixed the two carded behaviours at their source with regression tests instead of patching callers; scoped the fix to the shared module's existing tests (inline, since the umbrella scope does not list a prep test file).
- `scope-guard`: only `prep.rs` + task folder; no sheet or bar change.

## Still open

- PM closes #3041 and #3042 after merge.

## Plain summary

- A later TEST FITNESS verdict now wins, and the rendered `prep fit` block is clean text; both with regression tests. This fixes the two verified prep.rs bugs without touching any sheet.
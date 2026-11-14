# ORDER: native-space-state-contract, codex-9

Lab: codex. Item: [native-space-state-contract](../../../../roadmap/items/native-space-state-contract.md).
Parent: [codex-8](../codex-8/ORDER.md). Reviewed head: `1a47a4afc`.

## Why this repair exists

The fresh exact-head review found two record defects, not a Q31 arithmetic defect:

1. `codex-8/REPORT.md` says the serving adapter added six tests; the file contains two test
   functions and the focused run reports 2 passed.
2. The ORDER requires Clippy, while the REPORT only records a package-wide `-D warnings` run that
   remains red on 68 existing diagnostics. The repair must distinguish the untouched baseline
   from a defensible Clippy result that includes the new serving file.

## Goal

Correct the test count and record a non-weakening Clippy check: the normal package-scoped Clippy
command exits zero with the existing warning baseline, and the new `q31_serving.rs` contributes
zero diagnostics. Keep the `-D warnings` baseline failure explicit.

## Files in scope

- `docs/loop/tasks/native-space-state-contract/codex-8/REPORT.md`
- `docs/loop/tasks/native-space-state-contract/codex-9/**`

## Size

About 10 to 40 changed record lines.

## Done when

1. The REPORT says two new serving test functions, matching the focused result.
2. `cargo clippy --locked --no-deps -p uor-r4-integer --features serving-wasm --lib` exits zero,
   and a filtered diagnostic count for `q31_serving.rs` is zero.
3. The REPORT distinguishes that result from the package-wide `-D warnings` baseline failure.
4. `loop report --check`, `loop pr-check --local`, `git diff --check` and a fresh review at the new
   exact head pass.

## Stop

- Do not change Q31 arithmetic, serialization or serving APIs in this repair.
- Do not fix unrelated integer Clippy warnings or weaken their lint level.
- Stop after the two review findings and their evidence are corrected.

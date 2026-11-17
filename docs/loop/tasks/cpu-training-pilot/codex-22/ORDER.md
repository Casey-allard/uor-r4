# ORDER — retain carry-hold test receipts without replacement

Lab: codex. Item: [CPU training pilot](../../../../roadmap/items/cpu-training-pilot.md), #3506. Maintainer repair for the exact-head review of [PR #3889](https://github.com/UOR-Foundation/uor-r4/pull/3889) at `d824de345`.

## Scope

The test module of `crates/uor-r4-cpu-train/src/bin/cpu-carry-hold-audit.rs` and this ORDER/REPORT.

## Cancel probe

Batch reads already require Linux descriptor containment. The success test currently runs on every target. Both optional evidence hooks call a writer that truncates existing files.

## Done when

Test the successful batch on Linux and the explicit refusal with no output on other targets. Verify that both retained-report paths use create-new semantics and a pre-existing receipt keeps its exact bytes. Record executed checks and their platform.

## Stop list

Leave production checkpoint parsing, checkpoint data, the historical experiment, its evidence hashes and its resource ledger untouched.

## Follow-up to review at `0f4c31185`

The second review found target-specific Linux open flags encoded as numeric literals. Extend this repair to the reader’s `custom_flags` argument, the CPU crate’s Linux-only dependency declaration and the workspace lockfile. Use the locked `libc` constants, leave descriptor containment and regular-file checks in place, and check the host and Linux targets again.

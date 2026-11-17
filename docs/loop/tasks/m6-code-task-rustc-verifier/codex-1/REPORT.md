# REPORT for the M6 verifier temporary files

Lab: codex. Item: [m6-code-task-rustc-verifier](../../../../roadmap/items/m6-code-task-rustc-verifier.md). [ORDER](ORDER.md). Fixes #3876.

## What did not work

Two canonical test compilations previously could write task.rs in the same directory. The contribution resolved that collision. Reviewing the subprocess caller found a second clock-only name for stdout and stderr: two simultaneous calls could still return an I/O error at file creation.

The first adoption head lost support for a configured temporary root that had yet to be created. Fresh review identified the change before delivery.

## What changed

Preserved @Rajaykumar12's original commit from [#4038](https://github.com/UOR-Foundation/uor-r4/pull/4038). A process-wide sequence and exclusive directory creation give each verifier call its own directory. The compiler and test executable write their captures beside their source and binary, under distinct compile and run basenames. The directory guard cleans them up on either success or failure.

The verifier creates any missing ancestors of the configured temporary root, then exclusively creates its own child. Dropping the child preserves the root.

## Measurements

Four temporary-directory tests passed, including the 16-thread same-label test and the nested missing-root test. Seven integration tests passed, including the expected compile error, assertion failure, timeout, refusals for missing test results or zero tests, and all 80 frozen canonical CodeTasks programs. A separate run of the passing-source integration test inherited a missing TMPDIR and passed with the real compiler. Its new root was empty after verification.

## Checks run at the head

Rust 1.97.1 on macOS arm64. Ran the harness library filter `m6_rustc_verifier`, integration target `panel_m6_rustc_verifier` and strict all-target harness Clippy at repaired source `b2762a1d7`. Harness formatting and the whitespace check passed. Claim wording passed on the initial adoption.

## Skills used

Source inspection and host validation under the ORDER.

## Still open

The unit covers trusted canonical verifier inputs and concurrent host test isolation. M6 model evaluation is outside this repair.

## Plain summary

Parallel verifier calls have separate sources, binaries and captured output. All 80 frozen canonical CodeTasks programs passed on the maintainer host.

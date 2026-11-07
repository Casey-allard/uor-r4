# REPORT: m6-code-task-rustc-verifier, deepseek-1

Lab: deepseek. Item:
[m6-code-task-rustc-verifier](../../../../roadmap/items/m6-code-task-rustc-verifier.md)
(#3432). Order: [ORDER.md](./ORDER.md). Base: `596bf7453`.

## What did not work, first

- Outside contributor `sharmis-tp`'s PR #3754 had the useful implementation,
  but it did not merge as delivered. Required CI failed on two strict-Clippy
  `needless_return` findings, and the policy gate rejected the missing
  maintainer ORDER and REPORT.
- The first implementation placed the runner in a new top-level module outside
  the roadmap item's declared scope rather than beside the M6 panel.
- A successful test process with no parseable libtest result was reported as
  `Pass { passed: 1 }`. A source containing no tests could also return
  `Pass { passed: 0 }`. Both are fail-open for an instrument whose only job is
  to prove that tests ran and passed.
- The first tests lived in the implementation module and the notebook change was
  only a link bullet, not the required dated result row with rustc, host and
  verdict.

## What changed

- Adopted both contributor commits with `sharmis-tp` preserved as the git
  author.
- Moved the implementation to
  `crates/uor-r4-harness/src/panel_m6_rustc_verifier.rs` and registered it in
  `panels.rs` beside `m6_reasoning`.
- Added an explicit trusted-source warning: the runner is not a sandbox and
  must never receive model-emitted or untrusted Rust.
- Made result handling fail closed. A pass now requires a parsed libtest result
  with at least one passing test and zero failures. Missing output is
  `UnexpectedOutput`; zero tests and non-zero failures are `TestFailure`.
- Moved finite source controls to
  `crates/uor-r4-harness/tests/panel_m6_rustc_verifier.rs`.
- Added controls for passing source, compile error, assertion failure, run
  timeout, successful-process-without-result, zero tests and all 80 frozen
  canonical programs.
- Added the frozen-panel pointer, the dated M6 notebook row and front-page
  contributor credit.

The frozen M6 panel rows, oracle, bars, noise and seal are unchanged.

## Measurements

Host:

- `rustc 1.97.1 (8bab26f4f 2026-07-14)`;
- AMD Ryzen 9 9950X, 16 cores / 32 threads;
- Linux `x86_64`.

The canonical-program run compiled and executed every one of the 80 frozen
CodeTasks rows:

- **80 / 80 rows passed**;
- **320 / 320 held-out vector assertions passed**;
- each row's generated source reported exactly one passing libtest function;
- the seven verifier integration tests completed in **10.06 s**.

This verifies the panel's canonical/reference programs and `to_rust_source`.
It does not verify model-emitted code, establish model coding capability or
replace any M6 qualification run.

## Checks run at the head

- `cargo test --locked -p uor-r4-harness`: passed. The library ran
  **677 passed, 1 ignored, 0 failed**; every binary, integration and doc-test
  target passed as well.
- code was incorporated from
  `fe78500ee73541aac753e60c8097f10bd5682e30` and
  `948361fa7710d8247f888ca9a763d56f34237b71`.
- focused verifier integration tests: 7 passed.
- `cargo clippy --locked -p uor-r4-harness --all-targets -- -D warnings`:
  passed.
- `cargo fmt -p uor-r4-harness -- --check`: passed.
- `uor-harness check`: passed; only repository-wide advisory warnings unrelated
  to this change remained.
- D23, D25 and D32 ratchets: passed.
- claim-wording gate: passed.

## Skills used

No system-provided Codex skill applied. The repository's loop, harness evidence
and delivery rules were used directly.

## Still open

- This item is instrument QA only. M6 remains behind M4 and is not a current
  model-quality result.
- The verifier is trusted-source only; a future model-facing executable-code
  path still needs a real sandbox and its own registered evaluation design.

## Plain summary

The outside contribution identified the right missing check and implemented a
bounded compiler/test runner. The adopted version moves it into the M6 panel
scope, refuses ambiguous test output, runs all 80 canonical programs and all
320 vector assertions, records the result, and credits the contributor.

References #3432, #2034.

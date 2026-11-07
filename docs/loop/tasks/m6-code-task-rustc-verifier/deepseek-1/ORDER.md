# ORDER: m6-code-task-rustc-verifier, deepseek-1

Lab: deepseek. Item: [m6-code-task-rustc-verifier](../../../../roadmap/items/m6-code-task-rustc-verifier.md) (#3432).
Base: `596bf74532beefc8d307936e8bbb35af03a7cb94`.

## Goal

Adopt outside contributor `sharmis-tp`'s PR #3754 as a maintainer-owned
change. Finish the bounded `rustc --test` verifier for the 80 frozen M6
CodeTasks canonical programs, make it fail closed, place it in the item's
declared scope, add the required records and CI wiring, credit the contributor,
and merge it. Preserve the contributor's commit authorship.

The verifier covers the project's canonical/reference programs only. It is
instrument QA for the transpiler and sandbox oracle, not model-written code and
not a capability result.

## Claim check

- The item is ready, host-only, open to anyone, and names this verifier.
- No open task currently owns the item; the contributor's PR is the source.
- M6 is behind M4, so this remains off the current primary model path.
- The branch `deepseek/m6-rustc-verifier` is based on current merged main.

## Cancel probe

Class: instrument.

1. PR #3754 is merged-ready only after the maintainer adoption, ORDER/REPORT,
   fail-closed parser and CI fixes below.
2. The exact contributor head passes its six focused verifier tests locally.
3. The frozen panel is unchanged and `rustc 1.97.1` is available on the host.

Result: GO.

## Done when

1. The std-only runner is registered under `src/panels.rs` beside the M6 panel
   and remains inside the roadmap item's declared scope.
2. A successful child process reports pass only when a parsed libtest result has
   `passed > 0` and `failed == 0`; missing or malformed output fails closed.
3. Integration tests cover pass, compile error, assertion failure, timeout,
   malformed/no test result, and all 80 frozen canonical programs.
4. `cargo test -p uor-r4-harness`, strict package Clippy, fmt, harness checks,
   ratchets and claim wording pass.
5. The frozen-panel README points to the verifier and test without changing the
   frozen rows, oracle, bar, noise or seal.
6. The M6 notebook has one dated result row with 80/80, rustc version, host and
   the instrument-only verdict.
7. The front-page README credits `sharmis-tp` after adoption.
8. A maintainer REPORT is delivered, the adoption PR is merged, and the original
   contributor PR is closed with a thank-you linking the merged commit.

## Stop and ask if

- the verifier requires changing a frozen panel row, oracle, bar, noise or seal;
- the harness gate requires a scope/bar change rather than a path correction;
- the contributor's authorship cannot be preserved.

## Files in scope

- `crates/uor-r4-harness/src/panel*.rs`
- `crates/uor-r4-harness/tests/panel*.rs`
- `docs/labs/m6-reasoning-panel-freeze/**`
- `docs/harness/panels/m6-reasoning.json`
- `docs/research/notebook/m6-reasoning-coding.md`
- `docs/loop/tasks/m6-code-task-rustc-verifier/**`
- `docs/roadmap/items/m6-code-task-rustc-verifier.md`
- `README.md`
- `docs/harness/claims.json`

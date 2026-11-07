---
id: m6-code-task-rustc-verifier
title: Add the rustc --test verifier the frozen M6 code-task bar names
milestone: M6
area: evaluation
status: ready
open_to: anyone
needs_gpu: false
depends_on: [m6-reasoning-panel-freeze]
notebook: docs/research/notebook/m6-reasoning-coding.md
scope: [crates/uor-r4-harness/src/panel*.rs, crates/uor-r4-harness/tests/panel*.rs, docs/labs/m6-reasoning-panel-freeze/**, docs/harness/panels/m6-reasoning.json, docs/research/notebook/m6-reasoning-coding.md, docs/loop/tasks/m6-code-task-rustc-verifier/**, docs/roadmap/items/m6-code-task-rustc-verifier.md, docs/harness/claims.json, README.md]
---

## Why it matters

The M6 code-task bar already promises a second, independent check. The registered bar's fourth item is "`code-tasks-80`: `>= 56 / 80` (`70.0%` passing all 4 held-out test vectors in the 256-step sandbox and `rustc --test` verifier)" ([frozen bar item 4](../../labs/m6-reasoning-panel-freeze/README.md)), and the frozen panel registry names the same verifier in its grader text and code-task note ([m6-reasoning.json](../../harness/panels/m6-reasoning.json)). The panel was frozen on 24 October 2026 with a non-author `TEST FITNESS: FIT` before any run it scores. This item builds the instrument cross-check that the frozen bar's literal `rustc --test verifier` clause names; #2865's pre-registration is where that model-facing clause must be bound or dropped (D68).

Nothing in the repository performs it. `SandboxProgram::to_rust_source` emits a standalone module with `#[test] fn verify_vectors()` (`crates/uor-r4-harness/src/panel_m6_reasoning.rs:497-577`), but its only call site, `crates/uor-r4-harness/tests/panel_m6_reasoning.rs:92-101`, asserts two substrings; the source is never compiled or run. The gap is already on the record: the 24 October FIT addendum called a compile/run guard a nice-to-have — its reviewer manually compiled and ran all 80 — and deliberately left the guard as a follow-up ([#2034 comment 6384785241](https://github.com/UOR-Foundation/uor-r4/issues/2034#issuecomment-6384785241), condition 4). It was named publicly by Yassine Amraoui (@wwwportal) on 30 October ([#2034 comment 6516951994](https://github.com/UOR-Foundation/uor-r4/issues/2034#issuecomment-6516951994)) and answered the same day ([#2034 comment 6517502248](https://github.com/UOR-Foundation/uor-r4/issues/2034#issuecomment-6517502248)); this item is the roadmap record that answer lacked.

Keep the claim modest. The verifier compiles and runs each frozen CodeTasks row's **canonical/reference program** — the Rust transpiler and the sandbox oracle — not code a model writes. The graded CodeTasks answer today is four integers parsed and compared by `grade_code_task_row` (`crates/uor-r4-harness/src/panel_m6_reasoning.rs:1958-2014`), and the frozen panel has no channel for a candidate's code. So this is instrument QA for the bar and its transpiler, not a capability result and not evidence that the model can write executable code.

The milestone dependency that D77 asks for is #2865's registered bar: `m6-qualify-reasoning-coding` (`docs/roadmap/items/m6-qualify-reasoning-coding.md`) is the item that will score this frozen panel, and its frozen code-task bar contains the literal clause quoted above; this item supplies the instrument cross-check over the panel's canonical programs, and #2865's pre-registration must decide whether that clause binds model-facing grading or is dropped or reworded. M6 sits behind M4, which is not done, so this unit is not on the current lab critical path; it is ready and open to an outside contributor now.

## What is missing

- **A bounded `rustc --test` runner.** `crates/uor-r4-core/src/native_geometric/workspace_coding.rs` has a bounded host-compiler pattern — `compile_single` (`:438`), `execute_binary` (`:546`), `iterative_repair` (`:568`), a 5 s process deadline (`:16`) and a pid-only kill (`:56-85`) — but no `--test`, no test-count parsing, no `cargo` and no offline handling (grep finds no `--test`, `test result`, `passed` or `offline` in that file). It belongs to a PARKED mechanism's crate and must not be extended for this ([mechanism board](../../mechanisms/README.md); [native-geometric reopen conditions](../../mechanisms/native-geometric.md)).
- **One entry point in `crates/uor-r4-harness`, beside the panel,** std only and additive: it takes the emitted source the caller already holds and returns pass/fail counts, or a typed error (compile error, failed assertion, timeout). It runs `rustc --edition 2021 --test <file> -o <bin>` in a fresh temporary directory (no Cargo project and no network, so `--offline` is moot), with a wall-clock deadline on both compile and run, killing only the spawned child's own pid (never a name or pattern, [L-005](../../harness/mistakes/L-005.md)), with `GIT_*` removed from the child environment and no extra environment given. It then runs the binary and parses its `test result: … N passed; M failed` line, and removes the temporary directory afterwards.
- **Finite CPU tests** in `crates/uor-r4-harness/tests/: a passing case, a compile error, a failing assertion and a timeout kill; plus the frozen 80 CodeTasks rows' canonical programs returning 80/80, so the verifier agrees with the sandbox oracle and the transpiler. Prefer the normal test job over `#[ignore]`; measure its cost and record it in the PR. If it is `#[ignore]`d, say why and give the exact run command.
- **A record.** `docs/research/notebook/m6-reasoning-coding.md` has no verifier row, and nothing ties the bar's literal verifier clause to a command.

## Done when

1. A std-only, host-only `rustc --test` runner lives in `crates/uor-r4-harness` with the bounds above, registered in `crates/uor-r4-harness/src/panels.rs` beside the panel modules. No new dependencies, no model path, no GPU, no serving change.
2. Finite CPU tests pass: a passing case, a compile error, a failing assertion, a timeout kill, and the frozen 80 CodeTasks rows' canonical programs at 80/80. `cargo test -p uor-r4-harness` is green and `uor-harness check` is clean.
3. The 80-row test runs in the normal harness test job, and its cost is recorded; if it is `#[ignore]`d, the reason and run command are in the test's doc comment.
4. One row in `docs/research/notebook/m6-reasoning-coding.md`: the verifier, 80/80, the rustc version and the host, and the scope stated plainly — instrument QA on the panel's canonical programs, not model-emitted code, no capability result.
5. The frozen panel record is unchanged in rows, oracles, bars, noise and seal. A one-line pointer to the verifier and its test is added to the lab README (or the panel JSON grader note says where the verifier lives). No fresh FIT is needed because no row or oracle changes; any later row or oracle change still needs one (the 2026-10-24 FIT addendum's condition (d)).
6. The PR says `Closes #<this issue>`. An outside contributor's PR is adopted by a maintainer lab that adds the ORDER and REPORT and runs the gates (CHARTER §5.5); a lab PR carries its own.

## Cancel if

The M6 qualification pre-registration (#2865) decides, as a bar decision card under D68, to drop or reword the `rustc --test` verifier clause; or the frozen `m6-reasoning` panel or its code-task oracle is superseded, re-frozen for a different code-task design, or re-sealed. Then this item is re-scoped or closed with the decision recorded, not built. A bounded `rustc --test` runner with the 80/80 frozen-panel test already on `main` cancels only the code — the record and CI wiring remain.

## How to start

1. Read the bar and registry, the panel module and its substring test: [frozen bar](../../labs/m6-reasoning-panel-freeze/README.md), [m6-reasoning.json](../../harness/panels/m6-reasoning.json), `crates/uor-r4-harness/src/panel_m6_reasoning.rs` (`to_rust_source`, `grade_code_task_row`), `crates/uor-r4-harness/tests/panel_m6_reasoning.rs:92-101`.
2. Read the pattern to copy and the bounds to keep: `crates/uor-r4-core/src/native_geometric/workspace_coding.rs` (`:16`, `:56-85`, `:438-467`, `:546-561`), the warning that it is not a sandbox for untrusted programs (`:3-4`), [L-005](../../harness/mistakes/L-005.md), and `.github/workflows/harness.yml`.
3. Post a one-line intent on the issue and claim it. This needs no GPU, no pre-registration sheet and no model: builds, finite host tests and host checks need none (CHARTER §4.5). An outside contributor may take it now; a lab picks it up only when M6's grading path is assembled, because it advances none of the open owner direction's five steps.
4. Implement and test it. Run `~/.cargo/bin/cargo test -p uor-r4-harness` — the laptop's PATH `cargo` can resolve to a non-Rust tool, which `onboard.rs` already guards — and `uor-harness check`.
5. Get the fresh-context adversarial review before the PR goes ready, merge through the queue, and verify on fresh `main` (CHARTER §5.5, D66).
6. Record the notebook row and the one-line pointer in the same PR, then close the issue.

## Rules that apply

- **Rust first** (CHARTER §3.11): Rust only, std only, no new Python and no new dependencies.
- **Host work:** builds, finite host tests and host checks need no pre-registration sheet (§4.5). This engages no GPU research compute (§3.5), no model path and no serving change.
- **Instrument QA only** (§4.1–§4.2, D31): the verifier confirms the bar's literal verifier clause and the transpiler/oracle agreement; the frozen oracle still decides the pass count, and this is not a capability result.
- **Trusted input only:** the source is the project's own `to_rust_source` output for the frozen reference programs. A compiled test binary runs as a plain process; a timeout is not a sandbox. This item must not run model-emitted or untrusted source (`workspace_coding.rs:3-4` says the same).
- **The frozen panel stays frozen:** no row, oracle, bar, noise or seal change; no re-freeze and no fresh FIT for this unit; a later row or oracle change still needs one (the 2026-10-24 FIT addendum's condition (d)).
- **Records** (§4.1, §5.6): one notebook row per result; negatives first in the REPORT.

## Sources

- [Yassine's comment](https://github.com/UOR-Foundation/uor-r4/issues/2034#issuecomment-6516951994) and [the reply](https://github.com/UOR-Foundation/uor-r4/issues/2034#issuecomment-6517502248), 30 October 2026.
- [deepseek's FIT addendum](https://github.com/UOR-Foundation/uor-r4/issues/2034#issuecomment-6384785241), condition 4 (the manual 80/80 compile).
- [Frozen M6 code-task bar item 4](../../labs/m6-reasoning-panel-freeze/README.md) and [the frozen panel registry](../../harness/panels/m6-reasoning.json).
- `crates/uor-r4-harness/src/panel_m6_reasoning.rs` (`to_rust_source` `:497-577`, `grade_code_task_row` `:1958-2014`) and `crates/uor-r4-harness/tests/panel_m6_reasoning.rs:92-101`.
- `crates/uor-r4-core/src/native_geometric/workspace_coding.rs` and [the mechanism board](../../mechanisms/README.md).
- [The M6 qualification item](m6-qualify-reasoning-coding.md), [the panel-freeze item](m6-reasoning-panel-freeze.md) and [the M6 notebook](../../research/notebook/m6-reasoning-coding.md).
- [CHARTER](../../../CHARTER.md) §3.11, §4.1–§4.2, §4.5, §5.5, §5.6; [L-005](../../harness/mistakes/L-005.md).

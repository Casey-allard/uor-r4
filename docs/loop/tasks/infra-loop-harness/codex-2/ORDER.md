# ORDER: infra-loop-harness, codex-2

Lab: codex. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Setup row: 2.
Parent: [codex-1](../codex-1/ORDER.md). Reviewed head: `076ede2c1`.

## Owner approval

The owner approved the native train/serve pivot and the control-plane update:

> "yes please update your corvuswing loopkit and harness too so that you can continue down the correct path"

Recorded at `2026-11-14T02:57:18Z`. This repair records the same approval for changes to
`CHARTER.md`, `AGENTS.md`, `docs/rules/changes.md`, `docs/integration/DECISIONS.md`,
`docs/harness/native-space.json` and its registration in `docs/harness/owner-paths.json`.

## Why this repair exists

The fresh-context review of codex-1 returned CHANGES:

- the native-space rule checked for required words, not for an affirming statement;
- `forbidden_default_path` was not enforced as a prohibition;
- the milestone rule did not require the parity relationship itself;
- planted faults did not cover the pointer, exact plan/roadmap markers or a contradiction
  that retained the old vocabulary;
- model pages described the native path as implemented before its contract exists;
- the branch claim and rule-index counts had drifted.

## Goal

Make `harness.native_space_contract` require one exact canonical assertion, parse the D85
decision cell instead of accepting matching text from its rationale or rejected alternatives,
and prove that missing, wrong and contradictory sources each fail for the named reason.

## Files in scope

- `.agents/skills/native-space-cycle/SKILL.md`
- `AGENTS.md`
- `CHARTER.md`
- `PLAN.md`
- `crates/uor-r4-harness/rules/registered-rules.json`
- `crates/uor-r4-harness/src/harnessrules.rs`
- `docs/harness/README.md`
- `docs/harness/milestones.json`
- `docs/harness/native-space.json`
- `docs/harness/skills/native-space-cycle/SKILL.md`
- `docs/integration/DECISIONS.md`
- `docs/model/**`
- `docs/roadmap/items/native-space-state-contract.md`
- `docs/rules/changes.md`
- `docs/loop/tasks/infra-loop-harness/codex-2/**`
- `xtask/src/stack_contract.rs`

## Size

About 100 to 350 changed lines for the exact markers, repair tests, record alignment and
generated model view.

## Done when

1. The exact canonical assertion is stored once in `native-space.json` and required in the
   owner decision cell, charter marker block, AGENTS bullet, milestone rule, plan, roadmap item,
   protocol, change log, skill pointer and every model surface.
2. The decision check parses the D85 decision cell and cannot pass from rationale or rejected
   alternatives.
3. Isolated tests prove missing sources, missing markers, wrong names and pointers, decision-cell
   contradictions, unsupported parity/forbidden fields and a contradictory model statement fail.
4. Model pages describe D85 as the required default and the native trainer as pending, not as an
   implemented capability.
5. The generated stack contract and roadmap views are current, and the stale rule counts are fixed.
6. `cargo test --locked -p uor-r4-harness`, Clippy, the focused native-space check, the full
   harness check, `git diff --check` and `loop report --check` pass at the repair head.

## Stop

- Do not implement or run a model.
- Do not delete historical float/export evidence.
- Do not weaken a test to obtain green.
- STOP and report when done. Do not start the next task.

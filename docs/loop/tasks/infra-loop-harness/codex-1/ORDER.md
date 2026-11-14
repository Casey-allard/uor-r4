# ORDER: infra-loop-harness, codex-1

Lab: codex. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Setup row: 2.
Base: `88b7bd5c`.

## Owner approval

The owner approved the architecture pivot and the control-plane update:

> "yes please update your corvuswing loopkit and harness too so that you can continue down the correct path"

Recorded at `2026-11-14T02:57:18Z`. This approves changes to `CHARTER.md`, `AGENTS.md`,
`docs/rules/changes.md`, and `docs/integration/DECISIONS.md` for one native train/serve
representation and matching harness controls. It also approves
`docs/harness/native-space.json` and its registration in `docs/harness/owner-paths.json`.

## Goal

Make the project's live rules, Codex skills, and `uor-harness` checks enforce the owner's
native-space invariant:

- training and serving carry one persistent integer/geometric state;
- weights, activations, carry state, geometric routes, memory keys and reads use the same
  typed native operators in training and serving;
- an artifact is a representation-preserving serialization, not a second model produced by
  an export-gap conversion;
- a prompt and state replayed through the training path and the served integer path must
  produce the same route choices, carry state, read choices and emitted token ids bit for bit;
- floating point remains available only outside this native path as reference or diagnostic
  code, never as a master model on the default path.

This ORDER updates the contract and its controls. It does not implement the native trainer.
A later roadmap item takes that model implementation.

## Cancel probe

Cancel this control-plane unit if:

1. `uor-harness check --only harness.native_space_contract` does not fail when the project
   skill, owner decision, charter marker, milestone policy, or typed native path file is
   missing or contradicts the contract;
2. the new Codex skill does not name the training/serving parity gate, the persistent state
   boundary, and the no-float-master rule;
3. the patch claims a model, speed, capability, or export result that was not run.

## Files in scope

- `.agents/skills/**`
- `AGENTS.md`
- `CHARTER.md`
- `PLAN.md`
- `docs/harness/README.md`
- `docs/harness/milestones.json`
- `docs/harness/native-space.json`
- `docs/harness/skills/**`
- `docs/harness/owner-paths.json`
- `docs/integration/DECISIONS.md`
- `docs/model/**`
- `docs/roadmap/items/infra-loop-harness.md`
- `docs/roadmap/items/native-space-state-contract.md`
- `docs/roadmap/README.md`
- `docs/rules/changes.md`
- `crates/uor-r4-harness/rules/registered-rules.json`
- `crates/uor-r4-harness/src/checkdispatch.rs`
- `crates/uor-r4-harness/src/harnessrules.rs`
- `crates/uor-r4-harness/src/ruleinventory.rs`
- `crates/uor-r4-harness/src/mistakes.rs`
- `crates/uor-r4-harness/src/prep.rs`
- `crates/uor-r4-harness/tests/**`
- `xtask/src/stack_contract.rs`
- `docs/loop/tasks/infra-loop-harness/codex-1/**`

If another file is needed, stop and report it before changing it.

## Size

About 400 to 800 changed lines across the owner record, skills, harness rule, tests and ORDER.

## Done when

1. `D85` records the owner decision and `CHARTER.md` carries one owner rule for the native
   shared-space cycle.
2. `.agents/skills/native-space-cycle/SKILL.md` and the harness skill pointer make agents
   begin implementation with the state/operator parity contract, not a float trainer.
3. `docs/harness/native-space.json` names the owner decision, source skill, required project
   records and forbidden default path.
4. `harness.native_space_contract` is registered, dispatched and checked by `uor-harness check`.
5. Unit tests prove a missing decision, skill, charter marker, milestones policy or typed
   path condition turns the check red; the real repository passes.
6. `AGENTS.md`, `PLAN.md`, `docs/harness/milestones.json` and the project skill all state the
   same invariant and prohibit a default float-master/export-gap path.
7. `cargo test --locked -p uor-r4-harness`, `cargo clippy --locked -p uor-r4-harness`,
   `uor-harness check --only harness.native_space_contract`, `uor-harness check`,
   `git diff --check`, and `uor-harness loop report --check` pass.

## Stop

- Do not implement, launch or claim a model run in this unit.
- Do not delete the historical float-training code, records or evidence.
- Do not weaken a test to make the new contract pass.
- Do not silently change the D11 integer serving ban or any GPU cap.
- STOP and report when done. Do not start the next task.

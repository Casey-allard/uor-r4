# ORDER: infra-loop-harness, deepseek-116 (stack contract v2 and runtime audit)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Base: `ce3c3e57b`.
Setup row: 2 (record maintenance and model-contract tooling).

## Goal

Turn the merged stack-contract proof of concept into the rebuildable identity
of the current FBS model path. The contract must describe the exact training
and serving domains, checkpoint sidecars, artifact schema/sections, mechanism
flags, evidence, load order and executable probes for arms r, rt and rk. A
cross-representation audit must compare the contract with the run config,
checkpoint, artifact header, tokenizer, integer session and real product chat
path, and it must never turn missing evidence into a pass.

This unit is support tooling. It does not train, change a model result, launch a
pod, touch Claude's paused branch, or claim that any arm is a capable chatbot.

## Cancel probe

At the base, schema `/1` has no exact artifact schema, structured evidence,
training versus serving context, mechanism flags, sidecars, load/cleanup order
or executable fixture reference. `uor-chat` has no scripted multi-turn product
path and no local command audits one composition across representations. The
unit stands.

## Files in scope

- `conformance/stack/**`
- `conformance/generated/**`
- `crates/repo-model/Cargo.toml`
- `crates/repo-model/src/stack.rs`
- `crates/uor-r4-integer/src/bin/uor-chat.rs`
- `crates/uor-r4-integer/src/bin/fbs-export.rs`
- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/stack/mod.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-harness/src/main.rs`
- `crates/uor-r4-harness/src/prep.rs`
- `docs/harness/gates.json`
- `docs/loop/templates/REPORT.md`
- `docs/model/STACK-CONTRACT.md`
- `docs/roadmap/items/infra-loop-harness.md`
- `xtask/Cargo.toml`
- `xtask/src/main.rs`
- `xtask/src/stack_contract.rs`
- `xtask/src/stack_audit.rs`

## Done when

1. Schema `/2` is readable by the compiler, legacy `/1` remains readable, and
   v2 silently missing any exact identity fails validation.
2. Arm r is the single current FBS contract; rt is a candidate; rk is a
   measured-negative historical contract. Their reported schema, hashes,
   metrics and absent rk artifact are cited rather than inferred.
3. Every v2 probe has a hashed `uor-r4.stack-probe/1` fixture with declared
   execution layers and structural expectations.
4. `uor-chat --script` exercises the same product reply path over deterministic
   multi-turn scripts, including per-turn explicit/automatic memory controls.
5. `cargo run -p xtask -- stack-audit` compares contract, run config,
   checkpoint sidecars, artifact header, tokenizer, integer chat and fixtures.
   Its outcomes distinguish `MATCH`, `MISMATCH`, `UNAVAILABLE` and `NOT_RUN`.
6. Optional `fbs-export --stack-contract` provenance changes only the
   diagnostic header source object; omitting it is byte-for-byte unchanged.
7. Generated Markdown and JSON include evidence status and remain deterministic.
8. A named `stack-contract` loop gate has a planted dependency-cycle fault that
   turns it red.
9. Focused tests, contract generation, harness tests, loop report/pr checks,
   exact-head review and fresh-main verification pass.

## Stop and report

Stop if the change needs a GPU run, a model result, an artifact-format change
that existing readers refuse, edits to Claude's branch/worktree, or a claim that
the current arm is useful chat. A missing local store artifact is recorded as
`UNAVAILABLE`; it is not filled with a guessed hash.

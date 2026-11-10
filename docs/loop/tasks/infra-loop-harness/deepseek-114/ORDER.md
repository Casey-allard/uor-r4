# ORDER: infra-loop-harness, deepseek-114 (stack contract compiler)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Base: `8a5bd5278`.
Setup row: 2 (record maintenance and model-contract tooling).

## Goal

Add a machine-readable stack contract and deterministic generated views so an
agent can reconstruct a model composition from source without relying on a chat
summary. The first contract records the historical FBS b64L T0 baseline; the
tooling is general and does not change model execution.

## Cancel probe (run first)

At the base, `conformance/stack/` and `docs/model/STACK-CONTRACT.md` are absent,
and `cargo xtask` has no `stack-contract` command. No existing repository model
or harness contract describes model shape, mechanisms, artifact sections,
commands, readings and boundary probes together. The unit stands.

## Files in scope

- `conformance/stack/**`
- `conformance/generated/**`
- `crates/repo-model/src/lib.rs`
- `crates/repo-model/src/stack.rs`
- `docs/model/STACK-CONTRACT.md`
- `docs/roadmap/items/infra-loop-harness.md`
- `docs/start/FILEMAP.md`
- `docs/loop/tasks/infra-loop-harness/deepseek-114/**`
- `xtask/src/main.rs`
- `xtask/src/stack_contract.rs`

## Done when

1. A contract names its lane, source pin, tokenizer and input hashes, model
   shape, artifact sections, mechanisms, commands, readings, probes and
   explicit non-claims.
2. Validation rejects duplicate IDs, malformed hashes, unresolved source
   evidence, unknown artifact sections, and readings or probes that name no
   mechanism.
3. `cargo xtask stack-contract --write` deterministically generates a Markdown
   view and canonical JSON; `--check` rejects stale or hand-edited output.
4. Package tests, formatting, Clippy and existing `check-model` pass.
5. No model runtime, GPU pod, Claude branch or model result changes.

## Stop and report

Stop if the schema requires runtime model changes, a GPU run, an artifact
format change, or edits outside this unit. Stop rather than inventing values
for the current Claude memory-line configuration; that contract update follows
the live branch after it lands.

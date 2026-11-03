# ORDER: one-model-end-to-end, deepseek-8

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `5e50f3e193746996c6157aa7b82cb6b6bc27584f`.

## Goal

Provide a lawfully compiled replacement for the oracle corpus Claude's
ordered-key round needs. The historical `examples/m-world.rs` is retained source
evidence and has no Cargo target after D29, so its documented
`corpus recall=oracle` command cannot be built on current `main`.

The replacement must produce matched M-world v2 stores with `recall=off` and
`recall=oracle`, prove that the oracle store contains real memory turns rather
than only `Memory: none.`, and stay composable with the existing
`mix-chat-corpus` path.

This unit supplies corpus evidence and regression cover. It does not implement
or change the copy key, model training, serving, or Claude's claude-260
research decision.

## Scope

- `crates/uor-r4-training/Cargo.toml`
- `crates/uor-r4-training/src/bin/m-world-recall-corpus.rs`
- `crates/uor-r4-training/src/bin/mix-chat-corpus.rs`
- `docs/loop/tasks/one-model-end-to-end/deepseek-8/**`
- `docs/roadmap/items/one-model-end-to-end.md`

## Required design

- Generate M-world v2 training stores without importing any historical module
  or banned operation.
- Count oracle recall lines that carry a value separately from lines that say
  `Memory: none.`, and record both in the manifest.
- Keep `recall=off` as a matched control and emit no memory turns there.
- Support the existing M-world mix, open values, v2/v2c variants, protocol 1/2,
  a fresh output root, exclusion panels and SHA-256 file identities.
- Add a finite CPU test over the pinned byte-BPE tokenizer and M-world v2 that:
  - generates the same nonempty oracle corpus twice and compares the token
    store, response mask and manifest byte for byte;
  - decodes the token store and requires real `Memory:` content;
  - generates a default-off control and requires no recall block or memory
    turns.

## Cancel probe

Treat a failure to reproduce the same token bytes on the second run as an
instrument/corpus defect and stop before claiming deterministic training data.

## Done when

- The focused binary test passes.
- `cargo fmt --check`, the relevant crate tests and `git diff --check` pass.
- A small bounded corpus is generated locally and its decoded memory census is
  recorded in the REPORT.
- A fresh exact-head review approves the PR.

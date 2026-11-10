# REPORT: infra-loop-harness, deepseek-118

Lab: deepseek. Order: [ORDER.md](ORDER.md). Base: `73f707d29`.

## What did not work, first

- The first provisional `/10` arm-r export was made before the contract
  revisions were final. Its semantic binding was stale, so it was not used as
  evidence. The artifact below was regenerated from final contract hash
  `299a9a2f3297eede06376dab952f0330352531ec20bb56c5cc393f3368dbcf6f`.
- Strict Clippy with `-D warnings` is not clean at this head. It stops on 61
  warnings in untouched integer-library files, including `codec.rs`,
  `discrete_d11.rs`, and pre-existing lines in `stack/format.rs`. The focused
  compile and tests are clean; this unit did not mix an unrelated warning
  cleanup into the artifact-admission change.
- Whole-workspace `cargo fmt --check` also reports pre-existing drift outside
  this diff. `git diff --check` passes, and the changed Rust hunks were kept in
  rustfmt style.
- The first implementation inferred binding from metadata presence. That would
  make an older `/6`–`/9` artifact carrying optional pre-policy provenance
  unreadable. The review changed enforcement to the `/10` schema itself and
  added a regression test proving legacy provenance remains informational.

## What changed

New product exports now default to schema `uor-r4.lut-stack/10`. The header
binds the contract source path, source SHA-256, parsed-contract semantic
BLAKE3 and tokenizer SHA-256. Parsing rejects a `/10` artifact with any missing
or malformed binding. `uor-chat` hashes the supplied tokenizer and refuses a
mismatch before generation or server startup. Historical schemas retain their
feature-derived parsing and treat old optional provenance as informational.

Historical `/6` through `/9` artifacts remain readable and byte-reproducible.
The canonical exporter requires the explicit `--legacy-artifact` switch for
that path and rejects mixing it with binding flags. `/10` point-key insertion
preserves the binding and schema. The stack audit now treats contract and
tokenizer identity as required for binding-required contracts and `NOT_RUN`
only for explicitly grandfathered legacy contracts.

All four existing contracts are explicitly marked `legacy-grandfathered` under
this task's recorded migration decision; their revisions and generated views
are updated. The change adds 658 lines and removes 131 before this report. The
ORDER did not state a line estimate.

## Measurements

Final arm-r contract semantic BLAKE3:
`299a9a2f3297eede06376dab952f0330352531ec20bb56c5cc393f3368dbcf6f`.

The final arm-r `/10` artifact has SHA-256
`a1d705586b35d7da282913b1e21b242a74c0661b32d73e2ec89f56310a8140e8`.
Its product-path one-token run completed at 112.87 tokens/s. The same artifact
with the GPT-2 tokenizer was refused before generation:

```text
bound artifact names tokenizer d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89,
supplied tokenizer is bce45d88d6dd39215c429f165f23efed712b89d909c0aa31a0372580ad2a4cbd
```

Legacy reproduction remained exact:

- arm-r `/7`: `707dfdc3b564df95497d57303decccebe42ac382045d7a0aae461137fec18ad0`
- arm-rt `/9`: `39672ffaad41051b87b81dec2e1f7c859cf8230939628056828486f19fd86d01`
- tokenizer: `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`

The arm-r legacy audit is `MATCH`: 37 checks match and the two intentionally
unbound legacy checks are `NOT_RUN`. The arm-rt audit has 34 matches, the same
two `NOT_RUN` checks, and only its two known copied-value probe negatives
(`It's 40.` instead of `47`).

## Checks run at the head

- `cargo check --offline -p repo-model -p xtask -p uor-r4-integer --features uor-r4-integer/serving-wasm`: pass.
- `cargo test --offline -p repo-model -p xtask`: 22 + 228 passed.
- `cargo test --offline -p uor-r4-integer --features serving-wasm --lib`: 518 passed, 1 ignored.
- `cargo test --offline -p uor-r4-integer --features serving-wasm --bin fbs-export --bin uor-chat`: 4 + 9 passed.
- `cargo run --offline -q -p xtask -- stack-contract --check`: four contracts current.
- `UOR_LAB=deepseek uor-harness loop gates --quick`: all four gates green.
- `UOR_LAB=deepseek uor-harness loop gates --break stack-contract`: `CAUGHT`, red with `dependency cycle`.
- Real `/10` chat admission, mismatch refusal, `/7` and `/9` reproduction, and both real artifact audits: results above.

## Skills used

None.

## Still open

- No GPU, pod, training run or serving-arithmetic change was made or needed.
- Direct in-process CPU/test exporters outside the canonical `fbs-export` path
  still construct historical schemas; extending mandatory binding to those
  helper paths should be a separately scoped migration so active training
  recipes are not broken without coordinated command changes.

## Plain summary

The old model-file formats were easy to mix with the wrong recipe or tokenizer
because those identities were optional notes. The new format makes them part of
the file identity. A newly exported model says exactly which contract recipe
and tokenizer made it, the reader checks those fields, and chat refuses a
tokenizer that does not match. Old model files still work and can still be
rebuilt exactly, but only when the caller explicitly says it is reproducing a
legacy artifact. No model weights or serving mathematics changed.

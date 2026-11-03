# REPORT: copy-code key guard, deepseek-4

Lab: deepseek. Item:
[m4-export-uorlut01-cli](../../../../roadmap/items/m4-export-uorlut01-cli.md)
(#2859). Order: [ORDER.md](ORDER.md). Base: `b6ebf7002`.

## What did not work

- **`--copy-codes` does not yet audit copy key 2.** Its control and trace comparison reconstruct a query from one token, while key 2 uses ordered pairs. The new unit refuses that artifact rather than producing a false comparison.
- **This does not fix or validate Claude's #3579 model.** It only prevents the excluded checker from misreporting a key-2 artifact after that branch lands.
- **No model or throughput result is claimed.** The change reads one header and never executes a model.

## What changed

`fbs-toks --copy-codes` now reads the `UORLUT01` header before parsing the artifact. A missing `hamming.copy_key` means the one-token default 1. Any explicit non-one key is refused with a message naming the key and stating that this checker supports one-token codes only.

The header-only guard can protect a copy-key-2 artifact before main's typed snapshot parser knows the new optional field. Malformed magic, header lengths, Hamming records and key types fail closed.

## Measurements

- `fbs-toks` binary suite: **7 passed, 0 failed**.
- Integer library suite: **443 passed, 0 failed, 1 ignored**.
- The new control covers absent key = 1, explicit keys 0/2/3 refused, a non-container, and a non-integer key.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm --bin fbs-toks`: 7 passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm --lib`: 443 passed, 1 ignored.
- `git diff --check`: passed.

## Skills used

No available Codex skill applied. The copy-code checker, copy-key semantics and container header were read directly.

## Still open

- After #3579 merges, either retain this fail-closed guard or replace it with a fully pair-aware copy-code check.
- The ordinary Hamming trace comparison already compares recorded paired codes, so it does not need this guard.

## Plain summary

The legacy copy-code checker now refuses artifacts whose copy heads use anything except the one-token key. This prevents a future key-2 artifact from producing misleading code-consistency counts. Host tests pass; no model was run.

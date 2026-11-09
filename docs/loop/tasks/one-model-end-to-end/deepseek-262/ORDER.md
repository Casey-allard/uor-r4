# ORDER: one-model-end-to-end, deepseek-262 (exact carry ablation)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `2c5d2a407`.

## Goal

Produce a validated `−C` artifact for an asking model: remove the readout
question carry while preserving the ask tables, all other sections and the
artifact schema.

## Cancel probe

- No open PR changes `crates/uor-r4-integer/src/stack/format.rs`,
  `crates/uor-r4-integer/src/stack/tests.rs` or
  `crates/uor-r4-integer/src/bin/fbs-toks.rs`.
- The existing artifact adder can create ask heads and a readout carry, and the
  integer session can serve both with or without a carry.
- No artifact transformer removes a carry while retaining the ask heads.

**Result (at claim):** pass. This is a missing exact control for Claude-339's
language-carry run and for future ask-head evaluations.

## Design

- Add `drop_hamming_carry_uorlut01` to the integer stack format API.
- Refuse an artifact with no Hamming read or no readout carry.
- Rebuild the data block without `hamming.d{depth}.carry`, retaining every
  matrix, every table and every `hamming.d{depth}.ask` table byte for byte.
- Clear only the record's `carry_rows`; keep its `asks`, depths, copy keys,
  margins, rotations, weights and source provenance.
- Reparse the output and validate the Hamming image before returning it.
- Add `fbs-toks ARTIFACT --drop-hamming-carry --out FILE`. The output is written
  only to a new path and reports both artifact hashes and the removed gains.
- Test carried ask artifacts, no-carry refusal, unchanged ask tables, changed
  later-row queries and unchanged schema.

## Files in scope

- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-integer/src/stack/mod.rs`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `docs/loop/tasks/one-model-end-to-end/deepseek-262/**`
- `docs/research/notebook/float-training.md`

## Done when

1. A carried ask artifact becomes a valid ask-only artifact with the carry
   section and `carry_rows` absent.
2. Every ask table and all non-carry data survive the transformation.
3. The ablated artifact's later-row query differs from the carried artifact and
   follows the no-carry semantics.
4. Missing-carry, missing-read and output-exists cases fail closed.
5. Focused and full integer tests, formatting, policy gates and full CI pass.
6. REPORT and notebook row are committed; a fresh exact-head review approves;
   the PR merges and is verified on fresh `main`.

## Stop and ask

Do not change the artifact schema, serving bars, model weights, training
objective or any Claude branch. Do not write into an existing output path.

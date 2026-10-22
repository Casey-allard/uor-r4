# ORDER: number-aware-tokenizer, antigravity-14

Lab: antigravity. Item: [number-aware-tokenizer](../../../../roadmap/items/number-aware-tokenizer.md). Base: `55311a46b`.

## Goal

The item's done-when reads:

> A number-aware vocabulary is trained with the Rust BPE trainer, pinned by identity hash, and included in the tokenizer comparison. The bar for what counts as "number-aware" is not yet registered: the dossier names multi-digit pieces, a numeric slot, or reclaiming the three idle superscript-digit slots. Registering the definition is the first step below.

Estimated diff size: ~320 lines across `crates/uor-r4-tokenizer/**`, `docs/mechanisms/tokenizer.md`, `docs/harness/prep/number-aware-tokenizer-DRAFT-2026-10-22.md`, `docs/labs/number-aware-tokenizer/README.md`, and `docs/roadmap/items/number-aware-tokenizer.md`.

## Cancel probe (run first)

Ran `rg -n "digits_individual|split_digits" crates/uor-r4-tokenizer/src/` and `ls docs/harness/prep/number-aware-tokenizer* docs/labs/number-aware-tokenizer*`:
- `crates/uor-r4-tokenizer/src/lib.rs:559-645` (`for_each_pre_token` and `split_digits`) already supports `digits_individual: Some(false)` (`{"type":"Sequence","pretokenizers":[{"type":"Digits","individual_digits":false},{"type":"ByteLevel","add_prefix_space":false}]}`), which splits at numeric/non-numeric transitions before `pre_tokenize` so contiguous digit runs stay intact without attaching a leading `' '` (`"Ġ"`), whereas `digits_individual: Some(true)` (used by `d36d3e87`) splits every single digit and `digits_individual: None` (plain `ByteLevel`) attaches a leading space to space-preceded digit runs.
- Neither a digit-piece census helper / deterministic `V=4096` and `V=16384` three-arm comparison in `crates/uor-r4-tokenizer`, nor `docs/harness/prep/number-aware-tokenizer-DRAFT-2026-10-22.md`, nor `docs/labs/number-aware-tokenizer/README.md` exists on `origin/main`.
- Result: work is not already done on `main`; continue.

## Files in scope

- `crates/uor-r4-tokenizer/**`
- `models/**`
- `docs/mechanisms/tokenizer.md`
- `docs/harness/prep/number-aware-tokenizer*`
- `docs/labs/number-aware-tokenizer/**`
- `docs/roadmap/items/number-aware-tokenizer.md`
- `docs/loop/tasks/number-aware-tokenizer/antigravity-14/**`

## Done when

- Design note and decision card for the number-aware definition and comparison bar posted on `#2872` and `#2028`.
- `crates/uor-r4-tokenizer` provides `DigitPieceCensus` (`ByteBpeTokenizer::digit_piece_census`) and a deterministic corpus builder + three-arm `BpeTrainer` / `train-bpe` comparison (`Digits(individual_digits=true)`, plain `ByteLevel`, and number-aware `Digits(individual_digits=false)`) at matched vocabulary size (`V = 4096` and `V = 16384`), verifying byte-identical repeated runs, pinned `blake3` and `sha256` hashes, pure digit piece counts by length, and distinct multi-digit encodings for `"84"`, `"48"`, and longer numbers.
- Pre-registration draft `docs/harness/prep/number-aware-tokenizer-DRAFT-2026-10-22.md` (conforming to `prep.sheet_shape`), lab record `docs/labs/number-aware-tokenizer/README.md`, and `docs/mechanisms/tokenizer.md` are updated and verified with `uor-harness check` and `uor-harness loop gates --full`.

## Stop and ask if

- Any file outside the scope list above would need to be modified.
- Any GPU training or evaluation run would be triggered (model training/scoring on a new tokenizer requires a non-author FIT review and a typed D40 owner release).

---
id: number-aware-tokenizer
title: Train a number-aware tokenizer and pin its identity
milestone: M1
area: tooling
status: ready
open_to: lab
needs_gpu: false
depends_on: []
scope: [crates/uor-r4-tokenizer/**, models/**, docs/mechanisms/tokenizer.md, docs/harness/prep/number-aware-tokenizer*, docs/labs/number-aware-tokenizer/**]
---

## Why it matters

The tokenizer fixes the vocabulary size and id space that the prime-zeta codebook and the address cells are sized by, and its hash pins every corpus and artifact. The dossier lists a number-aware tokenizer as never tested, so the planned comparison has only two arms (4,096 and 16,384 pieces). A third trained variant closes that gap and tells us whether digit-split numbers remain a barrier. See [the tokenizer page](../../mechanisms/tokenizer.md).

## What is missing

The tokenizer dossier lists a number-aware tokenizer as never tested, and none was ever trained. Only `d36d3e87` (4,096 pieces) and a 16,384-piece trainer exist. In the pinned vocabulary there are 10 pure single-digit pieces and 0 pure multi-digit pieces. A 9 October untrained probe showed that a two-digit value keeps its order as two pieces (84 and 48 stay distinct), so the need for this tokenizer was not shown. The matched 4,096 versus 16,384 comparison sheet is also not yet registered, so there is nothing yet to include the new vocabulary in.

## Done when

A number-aware vocabulary is trained with the Rust BPE trainer, pinned by identity hash, and included in the tokenizer comparison. The bar for what counts as "number-aware" is not yet registered: the dossier names multi-digit pieces, a numeric slot, or reclaiming the three idle superscript-digit slots. Registering the definition is the first step below.

## How to start

1. Read [`docs/mechanisms/tokenizer.md`](../../mechanisms/tokenizer.md) (especially "Numbers" and "What has never been tested"), [`docs/labs/rust-bpe-trainer-2026-10-19/README.md`](../../labs/rust-bpe-trainer-2026-10-19/README.md) and [`docs/harness/prep/m1-distill-d1-donor-corpus-DRAFT-2026-10-19.md`](../../harness/prep/m1-distill-d1-donor-corpus-DRAFT-2026-10-19.md). The pinned `tokenizer.json` itself is not committed. Its full sha256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` is checked in `crates/uor-r4-discrete/src/controller.rs:192` and written in `docs/labs/rung0-pilot-release-2026-10-14/sheet.md:90`, so ask on the issue for the file if you need to compare against it.
2. Post a design note on the issue before training. It must define "number-aware" (which of multi-digit pieces, numeric slot or reclaimed slots, and the vocabulary size), state the training text and its hash, and list the controls. Controls are: the 4,096 pinned vocabulary, a plain ByteLevel vocabulary at the same size, and a Digits-then-ByteLevel vocabulary at the same size. Check in `crates/uor-r4-tokenizer/src/training.rs` (the `digits_individual` setting, lines 64-66) whether the Digits step splits every digit or keeps runs together, because that decides which arm is number-aware. The design note must also say this is host-only data preparation with no softmax, no matmul and no float, and that it produces no model result.
3. Train on the laptop with the shipped command, no pod: `~/.cargo/bin/cargo run --locked --release -p uor-r4-tokenizer --bin train-bpe -- --input TRAIN_TEXT.txt --output NEW_TOKENIZER.json --vocab-size N` (add `--pre-tokenizer FILE.json` for a Digits arm). The output path must be fresh. Run it twice and confirm the two files are byte-identical.
4. Pin the identity. The trainer prints `blake3:<hex>` of the raw file bytes on stderr; that is the runtime tokenizer identity. The existing pin `d36d3e87` and the discrete driver config are the sha256 of the file bytes, so also record `shasum -a 256 NEW_TOKENIZER.json` and name both hashes in the issue. Check the attained size, because an exhausted corpus gives fewer pieces than requested. Report digit statistics against `d36d3e87`: the count of pure digit pieces by length, and how `84`, `48` and a few longer numbers encode. Run `~/.cargo/bin/cargo test -p uor-r4-tokenizer` and add a round-trip test for numbers.
5. Draft the comparison sheet as a pre-registration. It must name the vocabularies by identity hash, the panel and its hash, the metric, and the stop rules. Training or scoring a model on a new tokenizer is GPU work that needs a non-author FIT review and an owner release (CHARTER 4.5, 5.1), so do not run it from this issue.
6. Open a PR that says `References #<this issue>` until the vocabulary is pinned and listed in the registered comparison, then `Closes`. Record the result as one row in [`docs/research/notebook/data-tokenizer.md`](../../research/notebook/data-tokenizer.md).

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:13`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:95`](../../../docs/mechanisms/README.md)
- [`docs/mechanisms/tokenizer.md`](../../mechanisms/tokenizer.md)
- [`docs/labs/rust-bpe-trainer-2026-10-19/README.md`](../../labs/rust-bpe-trainer-2026-10-19/README.md)

# REPORT: number-aware-tokenizer, antigravity-16

Lab: antigravity. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Omission of `docs/roadmap/README.md` re-render on status flip in `antigravity-14`:** Flipping [`docs/roadmap/items/number-aware-tokenizer.md`](../../../../roadmap/items/number-aware-tokenizer.md) to `status: done` without running `cargo run --locked -p xtask -- roadmap render` left `docs/roadmap/README.md` stale (`41 ready / 17 done` instead of `40 ready / 18 done`), causing `docs archive and links` (`cargo xtask roadmap check`) to fail in CI (`https://github.com/UOR-Foundation/uor-r4/pull/2966#issuecomment-6341920536`). Merging `origin/main` and running `cargo run --locked -p xtask -- roadmap render` resolves the stale roadmap index cleanly.
- **Plain `ByteLevel(add_prefix_space=false)` (`digits_individual = None`, `train-bpe` default policy) fails to form pure 2-digit pieces on space-preceded numbers:** Because `pre_tokenize` attaches a leading space (`' '` $\mapsto$ `'Ġ'`) to any space-preceded digit run (`" d1d2"` $\mapsto$ `"Ġd1d2"`), the adjacent pair `("Ġ", d1)` sums frequencies across all trailing digits $d_2 \in \{0, \dots, 9\}$, satisfying $\operatorname{count}(\text{``Ġ''}, d_1) = \sum_{d_2} \operatorname{count}(\text{``Ġ''}, d_1, d_2) \ge \operatorname{count}(d_1, d_2)$. Greedy BPE merges `("Ġ", d1)` into `"Ġd1"` *before* $(d_1, d_2)$ can merge, consuming $d_1$ so that only the 5 two-digit suffixes of 3+-digit literals ever form pure multi-digit pieces (`pure_multi_digit = 5`, `space_prefixed_digit >= 109`), and unprefixed `"48"` or `"(48)"` stays split into 2 single-digit tokens (`["4", "8"]`).
- **Pre-tokenization with `Digits(individual_digits=false)` + `ByteLevel(add_prefix_space=false)` (`digits_individual = Some(false)`) resolves the boundary absorption cleanly:** `split_digits(segment, false)` splits at `numeric != in_digits` transitions before `ByteLevel` runs (`" 84"` $\mapsto$ `[" ", "84"]`), eliminating `"Ġd1"` pairs completely (`space_prefixed_digit = 0`, `mixed_digit_other = 0`) and producing `pure_multi_digit = 109` (`pure_two_digit = 100`, `pure_three_digit = 6`, `pure_four_plus_digit = 3`).
- **`docs/research/notebook/data-tokenizer.md` is outside the item's `scope`:** [`docs/roadmap/items/number-aware-tokenizer.md`](../../../../roadmap/items/number-aware-tokenizer.md) defines `scope: [crates/uor-r4-tokenizer/**, models/**, docs/mechanisms/tokenizer.md, docs/harness/prep/number-aware-tokenizer*, docs/labs/number-aware-tokenizer/**]`, so the 22 October row is recorded in [`docs/mechanisms/tokenizer.md`](../../../../mechanisms/tokenizer.md) and [`docs/labs/number-aware-tokenizer/README.md`](../../../../labs/number-aware-tokenizer/README.md) within `scope`.

## What changed

This task closes #2872 by adding a deterministic digit-piece census, canonical pre-tokenizer policy constructors, the 3-arm number-aware BPE comparison at $V = 4{,}096$ and $V = 16{,}384$, pinned `blake3` and `sha256` digests, the draft comparison sheet, and the re-rendered roadmap index (`40 ready / 18 done`):

- **Digit-piece census ([`crates/uor-r4-tokenizer/src/lib.rs`](../../../../../crates/uor-r4-tokenizer/src/lib.rs)):** Added `DigitPieceCensus` (`vocab_size`, `pure_single_digit`, `pure_multi_digit`, `pure_two_digit`, `pure_three_digit`, `pure_four_plus_digit`, `space_prefixed_digit`, `mixed_digit_other`) and `ByteBpeTokenizer::digit_piece_census()`.
- **Pre-tokenizer policies and comparison corpus ([`crates/uor-r4-tokenizer/src/training.rs`](../../../../../crates/uor-r4-tokenizer/src/training.rs)):** Added `individual_digits_policy`, `plain_byte_level_policy`, `number_aware_digits_policy`, and `number_aware_comparison_corpus` (`170,176` bytes, SHA-256 `05f21705704413d77e95cba3eb457682b96be12af460e28d33e54705605c91bd`).
- **Integration test ([`crates/uor-r4-tokenizer/tests/training.rs`](../../../../../crates/uor-r4-tokenizer/tests/training.rs)):** Added `number_aware_three_arm_comparison_and_digit_piece_census` verifying all 3 arms (`ctrl1`, `ctrl2`, `trt`) at both $V = 4{,}096$ and $V = 16{,}384$, byte-identical repeated runs, pinned `blake3` content addresses (`address()`), single-token encoding of `"84"` and `"48"` with distinct IDs, and lossless round-trip decoding.
- **Documentation, lab record, pre-registration sheet, and roadmap render ([`docs/labs/number-aware-tokenizer/README.md`](../../../../labs/number-aware-tokenizer/README.md), [`docs/harness/prep/number-aware-tokenizer-DRAFT-2026-10-22.md`](../../../../harness/prep/number-aware-tokenizer-DRAFT-2026-10-22.md), [`docs/mechanisms/tokenizer.md`](../../../../mechanisms/tokenizer.md), [`docs/roadmap/items/number-aware-tokenizer.md`](../../../../roadmap/items/number-aware-tokenizer.md), [`docs/roadmap/README.md`](../../../../roadmap/README.md)):** Documented the 3-arm results, `blake3` and `sha256` digests, and draft pre-registration sheet (`TEST FITNESS: PENDING`), marked `number-aware-tokenizer` `status: done`, and re-rendered `docs/roadmap/README.md` (`40 ready / 18 done`).
- **Size by class:** ~145 lines of library/trainer code, ~108 lines of integration tests, and ~265 lines of documentation, task records, and pre-registration records across 10 files.

## Measurements

- **Three-arm `train-bpe` comparison on `number_aware_comparison_corpus()` (`170,176` bytes, SHA-256 `05f21705704413d77e95cba3eb457682b96be12af460e28d33e54705605c91bd`):**
  - Command: `~/.cargo/bin/cargo run --locked --release -p uor-r4-tokenizer --bin train-bpe -- --input <corpus.txt> --output <out.json> --vocab-size <4096|16384> --min-frequency 1 --pre-tokenizer <policy.json>` (run twice per arm; verified byte-identical via `cmp -s`)
  - Result:
    - **Control 1 (`ctrl1`, `Digits(individual_digits=true)` + `ByteLevel`):**
      - $V = 4{,}096$: `pure_single_digit = 10`, `pure_multi_digit = 0`, `space_prefixed_digit = 0`, `encode("84").len() = 2`, `encode("48").len() = 2`, `blake3:ed3f625d4fbbfbc212ff6ba511943e6130539ce8df533a355648f028b01b548a`, `sha256:9fc81d936a39b5a9babf565afba6199408cea3d4b7a018f38b10ebc181fcf842`.
      - $V = 16{,}384$: `pure_single_digit = 10`, `pure_multi_digit = 0`, `space_prefixed_digit = 0`, `blake3:311a1e7c582be16e9620bcc82d66a80fb70d8c160201b02f5f64e5274ff248fa`, `sha256:c6b1fef8767e5a7954d6117fb7daf4a90604dee2c3992ff9b8eaa577b232eeeb`.
    - **Control 2 (`ctrl2`, plain `ByteLevel(add_prefix_space=false)`):**
      - $V = 4{,}096$: `pure_single_digit = 10`, `pure_multi_digit = 5`, `space_prefixed_digit = 110`, `encode(" 48").len() = 1`, `encode("48").len() = 2`, `blake3:81522e79624f40f83a85df319ef26a12f9a5d313930c9e6873763e91ec70bb5f`, `sha256:a922cfe42c47092c49e2761e0c43416fb7741865b0f229ad52d1959de8e98ebd`.
      - $V = 16{,}384$: `pure_single_digit = 10`, `pure_multi_digit = 5`, `space_prefixed_digit = 109`, `blake3:ec4c2c66f6bd2a588324bd61ef73fd0be4a59cf1b60c5751bb4f597a297d8a0a`, `sha256:e704f2dcbb14e49984251ed000f59e58cb1fd3974b2df75b37c3e334a97e7d34`.
    - **Treatment (`trt`, `Digits(individual_digits=false)` + `ByteLevel(add_prefix_space=false)`):**
      - $V = 4{,}096$: `pure_single_digit = 10`, `pure_multi_digit = 109` (`100` 2-digit, `6` 3-digit, `3` 4+-digit), `space_prefixed_digit = 0`, `mixed_digit_other = 0`, `encode("84").len() = 1`, `encode("48").len() = 1` (`encode("84") != encode("48")`), `encode("2026").len() = 1`, `encode("16384").len() = 1`, `blake3:c38dabf516cc142e0c1974d35d807b0040b7c079be32f95c0cdc3fa452d2fc36`, `sha256:84dee8b03e5951e9878fafb2eccc3e5e31a75b806de32f71d6665356a9bd92b5`.
      - $V = 16{,}384$: `pure_single_digit = 10`, `pure_multi_digit = 109` (`100` 2-digit, `6` 3-digit, `3` 4+-digit), `space_prefixed_digit = 0`, `mixed_digit_other = 0`, `blake3:411c366d7e307a87cfe605e0a8784b7abe184bbb53ea06c455fc2b40f1641ce2`, `sha256:46dbe12715660f1b3e4b4b80c33b7111f20dc9f5d290e440a3cc8fd31bf2163f`.

## Checks run at the head

- `~/.cargo/bin/cargo test --locked -p uor-r4-tokenizer`: `ok. 23 passed; 0 failed` (`13` unit + `2` CLI + `8` integration tests).
- `~/.cargo/bin/cargo run --locked -p xtask -- roadmap check`: `roadmap check: 88 items OK (by status: ready 40, held 26, decision 4, done 18)`.
- `~/.cargo/bin/cargo run --locked -p xtask -- links --baseline /private/tmp/base-links.json --renames-from "$(git merge-base origin/main HEAD)"`: `new broken: 0`.
- `uor-harness loop gates --full`: `all 17 gate(s) green`.
- `uor-harness loop report --check`: `REPORT ok`.
- `uor-harness loop pr-check --local`: `pr-check: clean`.

## Skills used

- `uor-research-architect`: followed for deterministic integer BPE vocabulary auditing, zero-softmax / zero-matmul compliance, and pre-registered three-arm controls.

## Still open

- Executing the downstream discrete model comparison across `ctrl1`, `ctrl2`, and `trt` requires a non-author `TEST FITNESS: FIT` review on [`docs/harness/prep/number-aware-tokenizer-DRAFT-2026-10-22.md`](../../../../harness/prep/number-aware-tokenizer-DRAFT-2026-10-22.md), owner confirmation of the proposed bar on `#2028` (`https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6341103106`), PM dispatch, and Casey's typed D40 release.

## Plain summary

The pinned 4,096-piece tokenizer (`d36d3e87`) splits every digit into a separate piece (`Digits(individual_digits=true)`), so multi-digit numbers like `"84"` and `"48"` are always encoded as two single-digit tokens (`["8", "4"]` and `["4", "8"]`) that share the same unordered set of pieces inside a context window. Simply removing the `Digits` splitter (`ByteLevel(add_prefix_space=false)`) does not fix this, because spaces before numbers attach to the first digit (`" 84"` becomes `"Ġ84"`), and the space-plus-first-digit pair `"Ġ8"` occurs across all ten second digits (`0`–`9`), so BPE merges `"Ġ8"` first and leaves unprefixed two-digit numbers like `"48"` split into two tokens (`pure_multi_digit = 5`, `space_prefixed_digit >= 109`). By setting `Digits(individual_digits=false)` before `ByteLevel(add_prefix_space=false)` (`number_aware_digits_policy(false)`), the pre-tokenizer splits at boundaries between digits and non-digits (`" 84"` becomes `[" ", "84"]`) while keeping contiguous digits together. On the deterministic 170,176-byte benchmark corpus at both 4,096 and 16,384 pieces, this eliminates all space-prefixed digit pieces (`space_prefixed_digit = 0`) and forms 109 pure multi-digit pieces (all 100 two-digit numbers `"00"`–`"99"` plus frequent 3-digit and 4+-digit numbers), giving `"84"` and `"48"` distinct single-token IDs with pinned `blake3` and `sha256` content hashes.

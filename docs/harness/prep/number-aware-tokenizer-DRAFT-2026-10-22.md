# Number-aware byte-level BPE pre-tokenization (`Digits(individual_digits=false)` + `ByteLevel`)

- **Status:** draft pre-registration, `TEST FITNESS: PENDING` (see the end of the sheet).
  - Corpus re-tokenization and M1/M4 training remain HELD. A run needs a non-author `TEST FITNESS: FIT` review (CHARTER §4.5), the owner's bar confirmation on `#2028` (`https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6341103106`), PM dispatch, and Casey's typed D40 release.
- **Line:** Data and tokenizer (`docs/mechanisms/tokenizer.md`, item `number-aware-tokenizer` `#2872`). **Lab:** antigravity.
- **Parent:** [`m1-attention-a8-address-cells-learning-2026-10-19.md`](m1-attention-a8-address-cells-learning-2026-10-19.md) (the A8 address-structured cells sheet).
  - **What changes from the parent:** replaces single-digit splitting (`Digits(individual_digits=true)`) and space-attached plain `ByteLevel` pre-tokenization with `Digits(individual_digits=false)` + `ByteLevel(add_prefix_space=false)` (`uor_r4_tokenizer::training::number_aware_digits_policy(false)`), keeping contiguous digit runs together while splitting at digit/non-digit boundaries before BPE counting.
- **Applicable owner decisions:** D11, D21, D22, D23, D24, D25, D28, D29, D31, D32, D37, D40.
  - Pure integer BPE frequency counting and deterministic code-point tie-breaking (`crates/uor-r4-tokenizer/src/training.rs`).
  - Zero normalized exponentials or squashing functions (CHARTER §3.2, D23).
  - Zero dense matrix multiplications or GEMM (CHARTER §3.3, D25).
  - Active `models/tokenizer.json` ($V = 4{,}096$, `blake3:1f5e713f3be23f9268b8617a9f3e211c8a316719a120a2aa5c6fd326c8c38315`) is unchanged until a corpus rebuild is cleared.

## 1. Why: single-digit splitting aliases reversed multi-digit numbers and plain `ByteLevel` merges leading spaces before digit pairs

1. **Commutative window collision under `Digits(individual_digits=true)` (`ctrl1`):** Splitting `"84"` into `["8", "4"]` and `"48"` into `["4", "8"]` gives both multi-digit numbers the identical multiset of single-digit token IDs inside a commutative context window unless position order separates them, and consumes $k$ positions for every $k$-digit integer (`pure_multi_digit = 0`).
2. **Leading-space absorption under plain `ByteLevel(add_prefix_space=false)` (`ctrl2`):** When digit runs are preceded by a space (`" d1d2"` $\mapsto$ `"Ġd1d2"`), the pair `("Ġ", d_1)` sums frequencies across all trailing digits $d_2 \in \{0, \dots, 9\}$, so $\operatorname{count}(\text{``Ġ''}, d_1) = \sum_{d_2} \operatorname{count}(\text{``Ġ''}, d_1, d_2) \ge \operatorname{count}(d_1, d_2)$. Greedy BPE merges `("Ġ", d_1)` into `"Ġd_1"` *before* $(d_1, d_2)$ can merge, consuming $d_1$ so that only 5 two-digit suffixes of 3+-digit literals ever form pure multi-digit tokens (`pure_multi_digit = 5`, `space_prefixed_digit >= 109`), and unprefixed `"48"` remains split into 2 tokens (`["4", "8"]`).
3. **Boundary-clean multi-digit merges under `Digits(individual_digits=false)` + `ByteLevel` (`trt`):** `split_digits(segment, false)` splits at `numeric != in_digits` transitions before `ByteLevel` runs (`" 84"` $\mapsto$ `[" ", "84"]`), eliminating `"Ġd_1"` pairs (`space_prefixed_digit = 0`, `mixed_digit_other = 0`) and forming all 100 two-digit pieces `"00"`..`"99"` plus frequent 3-digit and 4+-digit pieces (`pure_multi_digit = 109`).

## 2. Construction and matched controls

All three arms use `BpeTrainer` (`crates/uor-r4-tokenizer/src/training.rs`, `min_frequency = 1`) on the identical corpus at matched vocabulary sizes ($V = 4{,}096$ and $V = 16{,}384$):

1. **Treatment (`trt` — `number_aware_digits_policy(false)`):** `Sequence([Digits(individual_digits=false), ByteLevel(add_prefix_space=false)])`.
2. **Control 1 (`ctrl1` — `individual_digits_policy(false)`):** `Sequence([Digits(individual_digits=true), ByteLevel(add_prefix_space=false)])` (current `models/tokenizer.json` policy).
3. **Control 2 (`ctrl2` — `plain_byte_level_policy(false)`):** `ByteLevel(add_prefix_space=false)` (`train-bpe` default policy without `Digits` boundary splitting).

## 3. Bar

- **Pre-run finite controls (CPU, landed in `crates/uor-r4-tokenizer/tests/training.rs`):**
  1. `trt` achieves `pure_single_digit == 10`, `pure_multi_digit >= 109` (`pure_two_digit >= 100`, `pure_three_digit >= 5`, `pure_four_plus_digit >= 3`), `space_prefixed_digit == 0`, and `mixed_digit_other == 0` at both $V = 4{,}096$ and $V = 16{,}384$.
  2. `ctrl1` achieves `pure_multi_digit == 0`; `ctrl2` achieves `pure_multi_digit == 5` and `space_prefixed_digit >= 109`.
  3. Under `trt`, `"84"` and `"48"` each encode to 1 token with distinct IDs (`encode("84") != encode("48")`), and `" 84"` encodes to `[id(" "), id("84")]`.
  4. Repeated `train-bpe` runs produce byte-identical outputs (`blake3:c38dabf516cc142e0c1974d35d807b0040b7c079be32f95c0cdc3fa452d2fc36` at $V = 4{,}096$ and `blake3:411c366d7e307a87cfe605e0a8784b7abe184bbb53ea06c455fc2b40f1641ce2` at $V = 16{,}384$).
- **Pre-registered corpus and model bar (proposed via decision card on `#2028`, CHARTER §5.2):**
  1. **Vocabulary & boundary invariants on full training corpus:** `pure_single_digit == 10`, `pure_two_digit == 100`, `space_prefixed_digit == 0`, `mixed_digit_other == 0`, and exact round-trip `decode(encode(x)) == x` across all held-out evaluation splits.
  2. **Downstream numeric discrimination & M4 BPB contrast:** On a paired M1/M4 trunk run at matched $V$, `trt` improves multi-digit copy/successor accuracy on numeric spans over `ctrl1` (`individual_digits=true`) without degrading overall bits-per-byte ($\text{BPB}_{\text{trt}} \le \text{BPB}_{\text{ctrl1}}$ on the held-out evaluation stream, where BPB normalizes by UTF-8 byte length rather than token count).

## 4. Base artifact

- **Tokenizer trainer & runtime:** `crates/uor-r4-tokenizer` (`BpeTrainer`, `ByteBpeTokenizer`, `train-bpe`).
- **Active baseline tokenizer (unchanged by this preparation):** `models/tokenizer.json` ($V = 4{,}096$, `blake3:1f5e713f3be23f9268b8617a9f3e211c8a316719a120a2aa5c6fd326c8c38315`).

## 5. Data split

- **Host finite-control benchmark:** `number_aware_comparison_corpus()` (`170,176` bytes, SHA-256 `05f21705704413d77e95cba3eb457682b96be12af460e28d33e54705605c91bd`).
- **Prospective corpus rebuild (`V = 16,384`):** Runs strictly on the training split when a $V = 16{,}384$ corpus build is cleared; held-out DEV and fresh evaluation splits are never passed to `train-bpe` (CHARTER §3.10, D22 §4).

## 6. Steps

1. Land `DigitPieceCensus`, `ByteBpeTokenizer::digit_piece_census`, and the policy helpers (`individual_digits_policy`, `plain_byte_level_policy`, `number_aware_digits_policy`, `number_aware_comparison_corpus`) in `crates/uor-r4-tokenizer`.
2. Verify all 8 integration tests in `crates/uor-r4-tokenizer/tests/training.rs` and record the exact `blake3` and `sha256` digests in [`docs/labs/number-aware-tokenizer/README.md`](../../labs/number-aware-tokenizer/README.md).
3. Obtain a non-author `TEST FITNESS: FIT` review on this sheet and owner confirmation of the §3 numeric bar on `#2028`.
4. When a $V = 16{,}384$ corpus re-tokenization is cleared, run `train-bpe --pre-tokenizer` with `number_aware_digits_policy(false)` on the training split and verify `digit_piece_census()` before sealing the new tokenizer artifact.

## 7. Stop conditions

- Any non-deterministic output across repeated `train-bpe` runs, any `space_prefixed_digit > 0` or `mixed_digit_other > 0` under `number_aware_digits_policy`, or any round-trip `decode(encode(x)) != x` mismatch is an immediate stop for diagnosis (D31).
- Active `models/tokenizer.json` and pinned M1 token splits (`3dee858f…` / `0919e90f…`) must not be overwritten while M1 comparisons depend on the $V = 4{,}096$ baseline.

## 8. Cost and minimum test power

- **Cost:**
  - `preparation`: $\$0.00$ (host-only deterministic BPE training and unit/integration tests on laptop).
  - `run`: $\$0.00$ for host `train-bpe` corpus build; any subsequent GPU trunk comparison will be budgeted on its own M1/M4 trunk sheet via `scripts/pod/uor-pod`.
  - `controls`: included in the 3-arm host `train-bpe` comparison (`ctrl1`, `ctrl2`, `trt`).
  - `evaluation`: $\$0.00$ for host `digit_piece_census` and round-trip verification.
  - `retries`: $\$0.00$.
  - `storage`: $< 2\text{ MiB}$ per emitted `tokenizer.json`.
- **Minimum test power (D22 §3):**
  - Host deterministic verification: $2$ repeated runs per arm across $3$ arms at $2$ vocabulary scales ($V = 4{,}096$ and $V = 16{,}384$, $12$ runs total) on `170,176` bytes covering all $100$ two-digit pairs `"00"`..`"99"`.
  - Downstream model contrast (when scheduled): $\ge 2$ paired seeds and $\ge 200$ held-out numeric rows.

TEST FITNESS: PENDING

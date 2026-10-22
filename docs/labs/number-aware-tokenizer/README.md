# Number-Aware Byte-Level BPE Pre-Tokenization (`number-aware-tokenizer`, `#2872`)

**Date:** 2026-10-22
**Lab:** Antigravity (`antigravity-14`)
**Item:** [`number-aware-tokenizer`](../../roadmap/items/number-aware-tokenizer.md) (`#2872`)
**Design Note:** [`#2872` comment 6341102629](https://github.com/UOR-Foundation/uor-r4/issues/2872#issuecomment-6341102629)
**Pre-Registration Draft:** [`docs/harness/prep/number-aware-tokenizer-DRAFT-2026-10-22.md`](../../harness/prep/number-aware-tokenizer-DRAFT-2026-10-22.md) (`TEST FITNESS: PENDING`)

## 1. Why Plain `ByteLevel` Fails to Form Pure 2-Digit Pieces on Space-Preceded Numbers

Under plain `ByteLevel(add_prefix_space=false)` (`digits_individual = None`), `pre_tokenize` attaches a leading space (`' '` $\mapsto$ `'Ġ'`) to any space-preceded digit run (`" d1d2"` $\mapsto$ `"Ġd1d2"`). Because every two-digit pair $(d_1, d_2)$ in `"Ġd1d2"` is preceded by `'Ġ'`, the adjacent pair `("Ġ", d_1)` accumulates the sum of frequencies across all trailing digits $d_2 \in \{0, \dots, 9\}$:
$$\operatorname{count}(\text{``Ġ''}, d_1) = \sum_{d_2} \operatorname{count}(\text{``Ġ''}, d_1, d_2) \ge \operatorname{count}(d_1, d_2).$$
Consequently, greedy BPE merges `("Ġ", d_1)` into `"Ġd_1"` *before* $(d_1, d_2)$ can merge. Once `"Ġd_1"` is formed, $d_1$ is consumed and $(d_1, d_2)$ is destroyed; the subsequent merge forms `"Ġd_1d_2"` (a space-prefixed digit token) while unprefixed `"48"` or `"(48)"` remains split into two single-digit tokens `["4", "8"]`. Only the 5 two-digit pairs that also appear as suffixes of 3+-digit literals ever form pure multi-digit pieces (`pure_multi_digit = 5`, `space_prefixed_digit >= 109`).

Under `Digits(individual_digits=true)` + `ByteLevel(add_prefix_space=false)` (`digits_individual = Some(true)`), `split_digits(segment, true)` isolates every Unicode numeric character into its own 1-character pre-token before BPE counting, so zero multi-digit merges can ever occur (`pure_multi_digit = 0`).

Under the number-aware policy `Digits(individual_digits=false)` + `ByteLevel(add_prefix_space=false)` (`digits_individual = Some(false)`, [`number_aware_digits_policy(false)`](../../../crates/uor-r4-tokenizer/src/training.rs)), `split_digits(segment, false)` splits at `numeric != in_digits` transitions before `ByteLevel` runs (`" 84"` $\mapsto$ `[" ", "84"]`, `"(84)"` $\mapsto$ `["(", "84", ")"]`). That separates leading whitespace and punctuation from digit runs (`space_prefixed_digit = 0`, `mixed_digit_other = 0`) while keeping contiguous digits together so BPE merges frequent multi-digit runs into pure digit tokens (`pure_multi_digit = 109`, `pure_two_digit = 100`, `pure_three_digit = 6`, `pure_four_plus_digit = 3`).

## 2. Three-Arm Comparison & Deterministic Content Addresses (`train-bpe`)

All three arms were trained twice on the deterministic 170,176-byte benchmark corpus [`number_aware_comparison_corpus()`](../../../crates/uor-r4-tokenizer/src/training.rs) (SHA-256 `05f21705704413d77e95cba3eb457682b96be12af460e28d33e54705605c91bd`) at both $V = 4{,}096$ and $V = 16{,}384$, producing byte-identical outputs across repeated runs (`cmp -s` exit 0):

| Arm | Pre-Tokenizer Policy | $V$ | `pure_single_digit` | `pure_multi_digit` (`2d` / `3d` / `4+d`) | `space_prefixed_digit` | `encode("84")` / `encode("48")` | `blake3` Content Address (`address()`) | File SHA-256 |
|---|---|---|---|---|---|---|---|---|
| Control 1 (`ctrl1`) | `Digits(individual_digits=true)` + `ByteLevel` | $4{,}096$ | $10$ | $0$ ($0 / 0 / 0$) | $0$ | $2$ / $2$ tokens | `blake3:ed3f625d4fbbfbc212ff6ba511943e6130539ce8df533a355648f028b01b548a` | `9fc81d936a39b5a9babf565afba6199408cea3d4b7a018f38b10ebc181fcf842` |
| Control 1 (`ctrl1`) | `Digits(individual_digits=true)` + `ByteLevel` | $16{,}384$ | $10$ | $0$ ($0 / 0 / 0$) | $0$ | $2$ / $2$ tokens | `blake3:311a1e7c582be16e9620bcc82d66a80fb70d8c160201b02f5f64e5274ff248fa` | `c6b1fef8767e5a7954d6117fb7daf4a90604dee2c3992ff9b8eaa577b232eeeb` |
| Control 2 (`ctrl2`) | Plain `ByteLevel(add_prefix_space=false)` | $4{,}096$ | $10$ | $5$ ($5 / 0 / 0$) | $110$ | $1$ / $2$ tokens | `blake3:81522e79624f40f83a85df319ef26a12f9a5d313930c9e6873763e91ec70bb5f` | `a922cfe42c47092c49e2761e0c43416fb7741865b0f229ad52d1959de8e98ebd` |
| Control 2 (`ctrl2`) | Plain `ByteLevel(add_prefix_space=false)` | $16{,}384$ | $10$ | $5$ ($5 / 0 / 0$) | $109$ | $1$ / $2$ tokens | `blake3:ec4c2c66f6bd2a588324bd61ef73fd0be4a59cf1b60c5751bb4f597a297d8a0a` | `e704f2dcbb14e49984251ed000f59e58cb1fd3974b2df75b37c3e334a97e7d34` |
| Treatment (`trt`) | `Digits(individual_digits=false)` + `ByteLevel` | $4{,}096$ | $10$ | $109$ ($100 / 6 / 3$) | $0$ | $1$ / $1$ token (distinct IDs) | `blake3:c38dabf516cc142e0c1974d35d807b0040b7c079be32f95c0cdc3fa452d2fc36` | `84dee8b03e5951e9878fafb2eccc3e5e31a75b806de32f71d6665356a9bd92b5` |
| Treatment (`trt`) | `Digits(individual_digits=false)` + `ByteLevel` | $16{,}384$ | $10$ | $109$ ($100 / 6 / 3$) | $0$ | $1$ / $1$ token (distinct IDs) | `blake3:411c366d7e307a87cfe605e0a8784b7abe184bbb53ea06c455fc2b40f1641ce2` | `46dbe12715660f1b3e4b4b80c33b7111f20dc9f5d290e440a3cc8fd31bf2163f` |

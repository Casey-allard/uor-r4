# Tokenizer

**Status: ACTIVE.** Every current corpus, driver, test panel and trained artifact is bound to one pinned tokenizer with 4,096 pieces (identity `d36d3e87`). A Rust trainer for a 16,384-piece tokenizer landed on 19 October as tooling only, and a three-arm number-aware pre-tokenization comparison (`Digits(individual_digits=false)` + `ByteLevel`) landed on 22 October as host tooling and a draft comparison sheet. No model has been trained or scored on any other tokenizer ([trainer record](../labs/rust-bpe-trainer-2026-10-19/README.md), [donor-corpus sheet](../harness/prep/m1-distill-d1-donor-corpus-DRAFT-2026-10-19.md), [number-aware record](../labs/number-aware-tokenizer/README.md), [number-aware sheet](../harness/prep/number-aware-tokenizer-DRAFT-2026-10-22.md)).

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run, **hypothesis** means an untested idea (see [the charter](../../CHARTER.md), section 4.6). Unfamiliar project terms (K16, A8, admission, NOT_RUN) are in the [glossary](../start/glossary.md).

## In plain words

The model never sees letters. It sees numbered pieces of text, and this component cuts text into those pieces and turns pieces back into text. It starts from raw bytes, so any text can be encoded. Then it learns which neighbouring pairs of pieces occur most often and glues them into bigger pieces, so common words become one piece and rare words stay in several. The project's pinned tokenizer has 4,096 pieces. This is a standard technique, not a novel one. What is distinctive here is that the tokenizer's fingerprint is recorded in every file that depends on it.

## What it replaces in a transformer

It fills the same slot as a transformer's tokenizer; it replaces no transformer part with a different mechanism. A transformer has a tokenizer too, usually byte-level BPE. This page is on the board because the tokenizer fixes the vocabulary size and id space that the [prime-zeta codebook](./prime-zeta-codebook.md) and the [address-structured cells](./address-cells.md) are sized by. It also pins the identity that makes results comparable.

![A sentence at the bottom as bytes, merge arcs lifting pairs into larger pieces, and the final piece ids on top, with the number 84 shown as the two pieces 8 and 4. Beside it, a ladder of 3,837 merges with a vocabulary slider, and a strip from raw text through the tokenizer to a prepared corpus file with check marks for file hash, token count and tokenizer identity.](../figures/mechanisms/tokenizer.svg)

*Bytes become pieces by repeated merging, and the fingerprint of the merge table travels with every prepared file. A two-digit number is still two pieces.*

## How it works

**Vocabulary layout (Definition).** Ids 0, 1 and 2 are start, end and unknown. Ids 3 to 258 are the 256 single bytes. The rest are learned merges. For the pinned tokenizer that is $3+256+3837=4096$.

**Training (Definition).** Count adjacent pairs over the *unique* pieces of text in the corpus (pre-tokens), each weighted by how often it occurs. Then repeat: merge the pair with the highest weighted count into a new piece, and recount only the words that contain it. Ties go to the pair with the smaller left, then right, token string, so the result is deterministic. Special tokens are atomic. No merge crosses a special token, a pre-token boundary or an input file.

**Encoding (Definition).** Apply the merges to a text's bytes in rank order. This is a table lookup with no learned arithmetic.

**Identity (Definition).** The tokenizer is stored as a `tokenizer.json`. Its raw-byte BLAKE3 hash is the runtime content address returned by [`ByteBpeTokenizer::address`](../../crates/uor-r4-tokenizer/src/lib.rs). The [discrete controller](../../crates/uor-r4-discrete/src/controller.rs) separately uses a SHA-256 file pin (`config.tokenizer.sha256`, verified by [`FilePin::read`](../../crates/uor-r4-discrete/src/corpus.rs)). Prepared corpora and artifacts bind the tokenizer using their declared hash domain. A different tokenizer is therefore a different corpus.

**Prepared corpora.** The `UORT` v1 format is a 64-byte header followed by 16-bit token ids, with start markers and an optional binary response mask. Before reading a target, the discrete driver verifies the file hash, the token count and the tokenizer identity.

**One dialogue encoding.** A `DialogueProtocol` object fixes how roles and turns are written out, so training, test panels and chat use identical token segments. The need is real. Two branches once used incompatible serialisations, and even identical visible text can encode differently, because the merge rules may join a marker's trailing space to the first content token.

**Numbers.** In the pinned vocabulary (`Digits(individual_digits=true)` + `ByteLevel`) no piece made only of digits has more than one digit: it has 10 pure single-digit pieces and 0 pure multi-digit pieces (measurement, below; audited via [`ByteBpeTokenizer::digit_piece_census`](../../crates/uor-r4-tokenizer/src/lib.rs)). Under plain `ByteLevel(add_prefix_space=false)` (`digits_individual = None`), leading spaces attach to digit runs (`" d1d2"` $\mapsto$ `"Ġd1d2"`), so `("Ġ", d1)` sums across all trailing digits $d_2$ and merges before $(d_1, d_2)$, forming $\ge 109$ space-prefixed digit pieces while leaving unprefixed `"48"` split into 2 tokens (`pure_multi_digit = 5`). Under `Digits(individual_digits=false)` + `ByteLevel(add_prefix_space=false)` ([`number_aware_digits_policy(false)`](../../crates/uor-r4-tokenizer/src/training.rs)), `split_digits(segment, false)` splits at digit/non-digit boundaries while keeping contiguous digits together, yielding `pure_single_digit = 10`, `pure_multi_digit = 109` (`100` two-digit, `6` three-digit, `3` four-plus-digit), `space_prefixed_digit = 0`, and distinct single-token IDs for `"84"` and `"48"` at both $V = 4{,}096$ (`blake3:c38dabf5…`) and $V = 16{,}384$ (`blake3:411c366d…`). A separate numeral codec reads consecutive digit pieces left to right into one ordered value, so pieces 8, 4 give 84 and pieces 4, 8 give 48.

**Rule check.** Everything here is integer bookkeeping on the host and is not model compute. It involves no softmax or squashing (D23), no matrix multiplication (D30) and no float. The encode path was not itself put through the D11 instruction audit. It sits outside the served numerical kernels, and the integer runtime takes the tokenizer as an optional dependency for text sessions.

## What has been tried

No experiment has varied the tokenizer. Every row below is side evidence about data preparation, indexing or diagnosis. The 16,384-piece row is tooling validation, not a model result.

| Date | What was run | Result | Verdict | Record |
|---|---|---|---|---|
| 26 Jul | TinyStories test of the first compiled pipeline against a teacher (#201) | Bad scores traced to token indexing in the observation extraction, not to the zero-multiply runtime. The runtime reproduced the reference once indexing was correct (kernel top-1 28.9%) | positive | [issue archive](../../archive/history/github-archive-2026-10-15/markdown/0201-epic-token-index-mismatch-resolution-diagnostic-pivot.md) |
| 3 Sep | Fixed text-to-clause segmentation adapter, controlled grammar, no fitting (#1094) | 1,600/1,600 valid rows exact; 22,400/22,400 roles consumed; limited to the controlled grammar | positive | [comparison](../../archive/r4_retained_comparison_1094.md) |
| 26 Sep | Review of two dialogue serialisations across branches | Incompatible; one `DialogueProtocol` contract written. No model run | inconclusive | [contract](../../archive/integration/dialogue-protocol-contract-2026-09-26.md) |
| 8 Oct | 7.16M-parameter stack on chat data only against the sealed mixed-corpus model, one seed, 512 windows (#1894) | In-domain data was worth −0.1592 bits per byte (1.1199 to 0.9607). Going from 7M to 20M parameters was worth −0.0837 | positive | [issue archive](../../archive/history/github-archive-2026-10-15/markdown/1894-data-vs-architecture-settled-in-domain-data-is-worth-0-159-b.md) |
| 9 Oct | 2,860 documents pairing a demand with exactly the requested short answer, mixed into the chat corpus (#1958) | Short-answer form compliance appeared once demands were paired with matching short answers. A data-pairing effect, not a tokenizer effect | positive | [issue archive](../../archive/history/github-archive-2026-10-15/markdown/1958-breakthrough-demands-paired-with-matching-short-answers-prod.md) |
| 9 Oct | Recount of sealed numeric-memory replies against the frozen vocabulary | Numeric rows failed 12/13 (92.3%) against word rows 18/27 (66.7%). 10 single-digit pieces, 0 multi-digit. Token count and digit confusion refuted as causes | positive | [diagnosis](../../archive/labs/v5-numeric-diagnosis-2026-10-09/README.md) |
| 9 Oct | Untrained probe: does a two-digit value keep its order as two pieces? | Yes. 5/5 numeral tests pass; 84 and 48 stay distinct. The failure is in the learned read and emit path, so a number-aware tokenizer was not shown to be needed | negative, for that hypothesis | [probe](../../archive/labs/numeric-addressability-probe-2026-10-09/README.md), [plan](../../archive/labs/numeric-addressability-plan-2026-10-09/README.md) |
| 19 Oct | Deterministic Rust trainer for 16,384 pieces, checked against an independent exhaustive recount | An 8,192-document synthetic fixture attains exactly 16,384 pieces and round-trips unseen text; 40 deterministic corpora recount-match; two regressions found and fixed | positive, tooling only | [trainer](../labs/rust-bpe-trainer-2026-10-19/README.md) |
| 22 Oct | Three-arm number-aware BPE pre-tokenization comparison (`ctrl1` `individual_digits=true`, `ctrl2` plain `ByteLevel`, `trt` `individual_digits=false`) at $V=4{,}096$ and $V=16{,}384$ (#2872) | Plain `ByteLevel` merges `("Ġ", d1)` before `(d1, d2)` (`pure_multi_digit = 5`, `space_prefixed_digit >= 109`); `Digits(individual_digits=false)` + `ByteLevel` yields `pure_multi_digit = 109` (`100` 2-digit, `6` 3-digit, `3` 4+-digit), `space_prefixed_digit = 0`, and byte-identical `train-bpe` outputs (`blake3:c38dabf5…` at $V=4{,}096$, `blake3:411c366d…` at $V=16{,}384$) | positive, tooling only | [record](../labs/number-aware-tokenizer/README.md), [sheet](../harness/prep/number-aware-tokenizer-DRAFT-2026-10-22.md) |

**Reading these honestly.** The numeric misses were measured on a pointer-copy float stack with no memory operator, so they speak about that learned read and the 4,096 vocabulary, not about byte-level BPE as a mechanism. The dialogue clash was a coordination defect, fixed by a shared protocol, not a tokenizer failure. The data-versus-size row is one seed and says little about vocabulary size.

## What has never been tested

- Whether 16,384 pieces help or hurt the discrete model. No model has used any tokenizer other than the pinned one.
- Whether a number-aware tokenizer (`Digits(individual_digits=false)` + `ByteLevel`, built and pinned at $V=4{,}096$ and $V=16{,}384$ in `#2872`) improves downstream discrete model training or numeric copy/successor accuracy.
- Whether the tokenizer interacts with prime-zeta placement or cell coverage. The prime-zeta qualification used $V=4096$ only.
- Fairness across scripts and languages. The training corpora are English chat and code.
- Reclaiming the three superscript-digit slots (idle vocabulary) for multi-digit pieces.

## What comes next

The mechanism is ACTIVE, so this is a plan, not a reopening. The next step is executing the matched 4,096-against-16,384 and number-aware comparison ([`number-aware-tokenizer-DRAFT-2026-10-22.md`](../harness/prep/number-aware-tokenizer-DRAFT-2026-10-22.md)) once a corpus rebuild and GPU model comparison are cleared. Open questions it should answer: does 16,384 improve the prime-zeta and address-cell numbers or just spread cells thinner (the planned shape uses 512 cells instead of 256), do multi-digit pieces (`Digits(individual_digits=false)`) outperform single-digit pieces on numeric spans at matched BPB, and would training the tokenizer on the donor corpus make it a different corpus identity.

## Also known as

BPE; byte-pair encoding; `uor-r4-tokenizer`; `hf_bpe` (its old name inside the core crate); the pinned tokenizer `d36d3e87`; `train-bpe`; `DigitPieceCensus`; `number_aware_digits_policy`; the `UORT` prepared-corpus format; `DialogueProtocol`; data preparation.

## Code

- [`uor-r4-tokenizer/src/lib.rs`](../../crates/uor-r4-tokenizer/src/lib.rs): runtime byte-level BPE, `tokenizer.json` parser, raw-byte identity, and [`DigitPieceCensus`](../../crates/uor-r4-tokenizer/src/lib.rs) (`ByteBpeTokenizer::digit_piece_census`).
- [`training.rs`](../../crates/uor-r4-tokenizer/src/training.rs): the deterministic merge trainer (weighted pair counts, pair-to-word index, lazy heap) and pre-tokenizer policy constructors (`number_aware_digits_policy`, `individual_digits_policy`, `plain_byte_level_policy`, `number_aware_comparison_corpus`). [`bin/train-bpe.rs`](../../crates/uor-r4-tokenizer/src/bin/train-bpe.rs): the host-only command, for vocabularies from 259 to 65,536.
- [`dialogue.rs`](../../crates/uor-r4-tokenizer/src/dialogue.rs): the shared dialogue encoding. [`tests/training.rs`](../../crates/uor-r4-tokenizer/tests/training.rs): checks against an independent recount.
- [`uor-r4-discrete/src/corpus.rs`](../../crates/uor-r4-discrete/src/corpus.rs): the prepared-corpus loader that verifies hash, count and identity.
- [`native_geometric/numeral.rs`](../../crates/uor-r4-core/src/native_geometric/numeral.rs): the numeral codec (older engine, outside the selected build).
- Corpus preparation: `prepare-knowledge-corpus`, `prepare-chat-parquet` and `mix-chat-corpus` in `crates/uor-r4-training/src/bin/`.

## Related mechanisms

[prime-zeta-codebook](./prime-zeta-codebook.md): maps each id to a prime and phase position; the tokenizer fixes $V$. [address-cells](./address-cells.md): the cell placement is sized by the vocabulary. [discrete-learning](./discrete-learning.md): the driver verifies tokenizer identity on every corpus. [native-geometric](./native-geometric.md): the numeral codec and an older lexical codec sit on token ids. [lut-serving](./lut-serving.md): the served engine carries the tokenizer for text sessions.

# ORDER: real-text-corpus, claude-292

Lab: claude. Item: [real-text-corpus](../../../../roadmap/items/real-text-corpus.md). Base: `5bd24915e`.

## Goal

The item's done-when reads:

> 1. **The builders,** in `real_text.rs` and one bin, `prepare-real-text`, with tests and negative gates:
>    - OASST2's tree to conversations, with each filter pinned by a test;
>    - Wikipedia's parquet to BOS-opened documents;
>    - the split by source id;
>    - the tokenizer pin refused on a wrong SHA;
>    - the exclusion rule, with tests that pin an 8-word hit, a 4-to-7-word turn matched whole, and a shorter turn counted as uncovered.
>
>    The training crate's tests pass.
> 2. **The corpus and its panel,** built on the laptop (data preparation, not training; see Rules that apply).
>    - **Each manifest records** the source, revision, licence and attribution; the counts of trees, threads, articles and tokens; the rows dropped, and why; and bytes per token.
> 3. **The census against the frozen test panels** reads no shared prompt, with every excluded document counted and every uncovered short turn counted.
> 4. **Storage:** in the private store under `results/<lab>/real-text-corpus-<date>/`, with a `SHA256SUMS`, read back. No corpus is committed.
> 5. **One notebook row in `data-tokenizer.md`, and a REPORT with the negatives first.** P5's sheet unit then moves P5's Data line to real text, through a decision card.

**This unit is done-when 1, the builders.** Done-when 2 to 5 are unit 2: the full corpus, its panel, the census and storage.

- **Claim check, (a) the step:** the decided order (#2028 6634218223) puts a human-written real-text corpus, 2 CPU units, before P5 on real text. This is its unit 1.
- **Claim check, (b) the model:** nothing that trains or serves changes yet. This unit builds the data path P5 trains and scores on: real human conversations and articles, instead of the templated M-world, whose replies are 75–78% verbatim in TRAIN (claude-202).

## Cancel probe (run first)

- **Does a builder already exist?** Search `crates/` on the base for an OASST2 tree reader, and for a Wikipedia builder that writes BOS-opened documents and splits by id.
- **Does an open order hold any file in the scope?** Check every open PR's file list.
- **Is the training crate green at the base?** Run its tests with `--features parquet-input --no-fail-fast`, so that a later red is this unit's.
- **Classed:** instrument. The unit is cancelled only if a builder that meets done-when 1 already exists.

## Files in scope

- `crates/uor-r4-training/Cargo.toml`
- `crates/uor-r4-training/src/lib.rs`
- `crates/uor-r4-training/src/real_text.rs`
- `crates/uor-r4-training/src/bin/prepare-real-text.rs`
- `crates/uor-r4-training/tests/real_text*.rs`
- `docs/harness/prep/real-text-corpus*`
- `docs/roadmap/items/real-text-corpus.md`
- `docs/loop/tasks/real-text-corpus/**`
- `docs/research/notebook/data-tokenizer.md`

## Done when

1. **`real_text.rs` is compiled in** (`lib.rs`, not behind `cfg(any())`). It holds:
   - **OASST2:** one tree's JSON to its thread. At each node, drop the children that are deleted, not passed by review (`review_result` not `true`), flagged `synthetic`, or not `lang == "en"`. Take the best remaining child: lowest `rank`, a null rank after every ranked one, a tie broken by `message_id`. Map `prompter` to `user`, and keep a thread only if it holds an assistant reply. Every drop is counted by reason.
   - **Wikipedia:** an article to a document: `<|bos|>`, the cleaned title and body (the rule of `knowledge_corpus::clean_article`, `knowledge_corpus.rs:236-262`, ported), `<|eos|>`. The text after BOS follows the convention `DialogueEncoder` uses for content after a special token, recorded in the manifest.
   - **The split:** held-out when the first 8 bytes of SHA-256 of the source id, read as a `u64`, mod 1,000 fall below a set count. So a document's side depends only on its id.
   - **The tokenizer pin:** a wrong SHA-256 of `tokenizer.json` against `d36d3e87…` (`hamilton_sheet.rs:53-55`, copied as a constant naming its source) is refused before any output is written.
   - **The exclusion rule,** ported from `knowledge_corpus.rs` (`:4-18`, `:34`, `PanelExclusion` at `:141-234`) with both parts and the uncovered count, and every hash hit confirmed against the stored words.
   - **The decode round trip:** a document whose tokens do not decode back to its exact text is dropped and counted, as `prepare-knowledge-corpus.rs:302` does.
2. **`prepare-real-text`** (feature `parquet-input`), with modes `oasst` and `wiki`, writes `train/` and `heldout/`, each with `tokens.u16`, `response_mask.u8` and `manifest.json`, through `CorpusWriter`. The manifest records:
   - the source, revision, licence and attribution, each a required argument;
   - the tokenizer's SHA-256 and the tokens' and mask's SHA-256;
   - the counts: trees and threads, or articles; tokens; every drop by reason; bytes per token;
   - the split rule and its count.
3. **Tests,** each with a negative gate where it pins a rule. Set the rule aside, see the test fail, then restore:
   - each OASST2 filter, and the rank order with a null rank and a tie;
   - the opening BOS and closing EOS of a Wikipedia document, read from a parquet fixture written by the test;
   - the split's stability under a reordered input;
   - the refused wrong SHA;
   - the exclusion rule's three cases: an 8-word hit, a 4-to-7-word turn matched whole, a shorter turn counted as uncovered.
4. **The training crate's tests pass** with `--features parquet-input --no-fail-fast`, compared with the base count. The float crate is untouched.
5. **A smoke run:** `prepare-real-text oasst` over the real OASST2 file with the pinned tokenizer, into `local/claude/`, with its counts in the REPORT. Nothing from it is stored; unit 2 builds and stores the corpus.
6. **A fresh exact-head review passes,** and `loop report --check`, `loop pr-check --local`, `uor-harness check` and `loop gates --full` pass.

## Stop and decide (never wait) if

- the parquet build cannot read the codec of a test fixture or a real shard. Then record which codec, and decide between another reader path and leaving Wikipedia to unit 2;
- a rule of the item cannot be met as written. For example, OASST2 replies may carry no rank anywhere. Then fix the item's text in this PR, and say so in the REPORT;
- the smoke run's counts look wrong, for example almost every tree dropped by one filter. Then classify instrument or data before going on.

## Cancel probe result (5 Nov, about 15:00Z)

- **No builder meets done-when 1.**
  - Nothing in `crates/` reads OASST2 trees: no file names `message_tree_id` or OASST.
  - For Wikipedia, the historical `prepare-knowledge-corpus.rs` (an unregistered bin, using the compiled-out `knowledge_corpus`) reads `wikimedia/wikipedia` parquet rows (`article_from_parquet`, `:184-213`). It cleans articles, applies the exclusion rule, checks the decode round trip (`:302`) and lists drops in `excluded.jsonl`. But it splits by position (`k % heldout_every`, `:28-30`), and its doc names no opening BOS.
  - So this unit ports its pieces into `real_text.rs` and the new bin, and turns neither historical module back on.
- **No open order holds a file in the scope.** The only open PR is this unit's draft, #3733.
- **The training crate is green at the base** (`5bd24915e`), with `--features parquet-input --no-fail-fast`: 123 passed, 0 failed, 1 ignored, across 7 targets. The targets are lib 87, `chat-grade` 26, `m-world-recall-corpus` 8 (1 ignored), `prepare-chat-parquet` 1, `prepare-text-parquet` 1, and `mix-chat-corpus` and the doc tests 0 each.
- **Classed:** instrument. Nothing cancels the unit.

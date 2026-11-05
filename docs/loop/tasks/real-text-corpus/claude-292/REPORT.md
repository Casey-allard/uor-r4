# REPORT: real-text-corpus, claude-292 (the builders)

Lab: claude. Order: [ORDER.md](ORDER.md). Item: [real-text-corpus](../../../../roadmap/items/real-text-corpus.md), done-when 1. Base: `5bd24915e`. PR #3733.

**Result: the builders exist, and their rules are tested.**
- `real_text.rs` does four things:
  - turns an OASST2 tree into its best thread;
  - turns a Wikipedia row into a document that opens with BOS;
  - splits by source id;
  - refuses a wrong tokenizer, and screens every document against the panels' user turns.
- `prepare-real-text` writes `train/` and `heldout/`, which the CPU trainer's manifest check reads.
- On the real OASST2 file it reads 13,854 trees and keeps 5,391 threads (5,110 train, 281 held out).
- On one real Wikipedia shard, it reads and builds 2,028 articles.
- Both runs take seconds on the laptop: 2.7 s and 1.7 s by `time` at `8d10c7469`. That is a single measurement each, not recorded in a file.

**Claim check.** This is unit 1 of the real-text corpus, which the decided order (#2028 6634218223) puts before P5 on real text. Nothing that trains or serves changes yet: this unit builds the data path P5 trains and scores on.

## What did not work, first

- **The exact-head review asked for changes; they are made, each with a negative gate.**
  - Four fresh reviewers read the head `8d10c7469` in slices: the bin's arguments and writers, its run path, its tests, and the library's fix delta.
  - **Medium, found by all three bin slices: `vocab_size=` was never checked against the tokenizer.**
    - With the pinned tokenizer (4,096 ids) and `vocab_size=8192`, the run succeeded. It wrote 8,192 into both UORT headers and both manifests, and the CPU trainer would have accepted that.
    - Now `vocab_size` defaults to the tokenizer's id count, and a different value is refused before the output root is claimed.
  - **Medium: the codec refusal was not pinned end to end.**
    - Every fixture was uncompressed, so deleting the check in `read_wiki` still passed every test.
    - This build cannot write GZIP or LZ4: parquet is built with snap and zstd only, and no binary fixture may be committed. So the new test writes an uncompressed shard, then rewrites its footer so every column chunk names GZIP.
    - The check now runs before the claim, so a refused shard leaves nothing behind. Without it, the reader fails later with "Disabled feature at compile time: flate2", after the root is already claimed.
  - **Low, fixed:**
    - `protocol=` must be 1 or 2.
    - A named column (`id`, `title`, `text`) holding a type other than a string, an integer or null is an error. A title stored as plain bytes used to be dropped silently.
    - The tokenizer's sha is taken from the bytes the pin was checked on, not from a second read.
    - Two panels with the same file name are refused, since a hit names its panel by file name.
    - Both manifests are written to `.partial` files before either is renamed into place.
    - `totals.stopped_at_max_docs` says when `max_docs` was reached and reading stopped there. The source counts then describe the input read up to that point. If the cap falls exactly on a batch's end, that can still be the whole input (from the second review, documented, not tested).
    - The argument test asserts each refusal's text. The end-to-end test checks `bytes_per_token` against its formula.
  - **Low, recorded, not fixed:**
    - A failing test leaves its temp folder behind.
    - The wiki test builds its expected tokens with the same `clean_article` the bin calls. The library's tests pin the cleaning rule on its own.
    - The accounting check (kept + excluded + dropped = considered) cannot fail as the code stands; it stays as a guard.
    - If the second manifest's rename fails, the first manifest is already in place. Both renames are within one output root, so this is unlikely.
  - **The library's fix delta:** all nine findings from the first review are fixed. One new low item was fixed too: `walk_tree` kept a thread that opens on an assistant turn, and the builder then dropped it. The walk now applies the builder's rule. No OASST2 tree has an assistant root, so the smoke counts did not move.
- **Two builder subagents ran out of context** at about 214k and 200k tokens before handing back. So I wrote the bin myself and gave the library's first review fixes to a smaller agent. Nothing they committed was lost; the cost was time.
- **28 of 2,028 Wikipedia articles (1.4%) are dropped as too long** at `max_tokens` 8,192. They are the longest articles, so their share of the tokens is larger. The item does not say whether a long article is dropped, cut or split, so unit 2 decides before it builds the corpus. The count is in each manifest.
- **The smoke screened against the chat panels, not the frozen test panels.**
  - It used the repo's `data/panels/conversational-v2..v5.json`: 448 requests and 714 user turns.
  - DEV, Fresh V2, P-dev and the long panels were not found in this unit as lists of user turns. Unit 2 writes each one's user turns in the `{id, user_turns}` form `parse_panel` reads, then runs the census against them.
- **The short-request rule excludes common phrases.**
  - Six of the nine OASST2 exclusions are 4- and 5-word panel turns found whole inside a thread: "i have a problem" (2), "can you make it longer" (2), "what is her name" and "how old am i".
  - That is the rule ported from `knowledge_corpus.rs` working as written, and it errs toward excluding. It costs 6 of 5,400 threads.
- **Workspace clippy with `-D warnings` was red before this unit,** in `uor-r4-integer` and at `chat-grade.rs:4893`. Neither is this unit's. The two new files lint clean in both the normal and the test profile.
- **The EOS count is checked only in total.**
  - BOS equals the document count in both splits.
  - The kept documents hold 9,979 EOS.
  - Over all 5,400 threads the expected count is 9,996. That figure is derived from the oracle's thread-length table: one EOS per assistant turn, plus one at the end of a thread that ends on a user turn.
  - So the 9 excluded threads should hold the other 17. That was not checked thread by thread.

## What changed

In `crates/uor-r4-training`, the item's scope:
- **`src/real_text.rs`** (new; `pub mod real_text;` in `lib.rs`):
  - **OASST2:**
    - `walk_tree` drops a child that is deleted, not passed by review, synthetic, or not English, and counts each drop by reason.
    - It takes the best remaining child: the lowest rank, a null rank last, ties by `message_id`.
    - It maps prompter to user, and keeps the thread only if its first assistant turn has a user turn before it.
  - **Wikipedia:** `clean_article`, ported from `knowledge_corpus.rs:236-262`. A document is `<|bos|>`, then ` {title}\n\n{text}` with protocol 2's leading space (`SCHEMA_V2`), then `<|eos|>`.
  - **The split:** `heldout(id, k)` holds when the first 8 bytes of SHA-256 of the id, read as a `u64`, mod 1,000 fall below `k`.
  - **The tokenizer pin:** `PINNED_TOKENIZER_SHA256`. A test reads `hamilton_sheet.rs` and fails if the two differ.
  - **The exclusion rule:** `PanelExclusion` covers 8-word windows, and 4-to-7-word turns matched whole. Shorter turns are listed as uncovered, and every hash hit is confirmed against the stored words.
  - **`DocumentBuilder`:** builds threads and articles, and drops a document on its special text, a failed decode round trip, or its length, counting each.
- **`src/bin/prepare-real-text.rs`** (new; `Cargo.toml` registers it behind `parquet-input`):
  - **Modes and arguments:** `oasst` and `wiki`. `source`, `revision`, `licence`, `attribution` and `heldout_per_mille` are required.
  - **Refused before the claim:** a wrong pin, a wrong `vocab_size`, and an unsupported codec.
  - **Output:** `train/` and `heldout/` through `CorpusWriter`, plus `excluded.jsonl`, which lists each excluded document with the panel request it hit.
  - **The manifest** carries every field the CPU trainer checks (`corpus.rs:304-392`). It also records the source, the drop and exclusion counts by reason, bytes per token, the split rule and the uncovered short turns.
- `docs/loop/tasks/real-text-corpus/claude-292/`: the ORDER and this REPORT.

## Measurements

The smoke runs used the pinned tokenizer, `heldout_per_mille` 50, and panels `conversational-v2..v5`. Their outputs are in `local/claude/` and are not stored.

| | OASST2 (`trees.jsonl`, rev `179dd21f`) | Wikipedia (`20231101.en` shard 00020, rev `b04c8d1c`, `max_docs` 2,000) |
|---|---|---|
| Read | 13,854 trees | 4,096 rows (one batch: the bin's `BATCH` is 4,096) |
| Before documents | root dropped: not English 8,451, deleted 2; no assistant reply 1 | — |
| Child nodes dropped | deleted 953, review not true 616, not English 4, synthetic 0 | — |
| Documents considered | 5,400, equal to the oracle's count | 2,028 (stopped at `max_docs`) |
| Kept: train / held out | 5,110 / 281 | 1,898 / 102 |
| Excluded | 9 (3 by 8-word window, 6 by short request) | 0 |
| Dropped | 0 | 28, all too long |
| Tokens: train / held out | 3,798,405 / 217,043 | 1,546,929 / 80,493 |
| Bytes per token | 2.612 | 2.333 |
| Uncovered short panel turns | 1 ("They were chocolate.") | 1 |

- **The held-out share** is 281 / 5,391 = 5.2% and 102 / 2,000 = 5.1%, against a target of 5.0%.
- **The trainer's fields check out:**
  - `tokens_bytes` = 64 + 2 × tokens;
  - `response_tokens` = tokens − documents for Wikipedia, since only BOS has mask 0;
  - `vocab_size` = 4,096.

## Checks run at the head

- **The training crate's suite at `c810bdd2b`** (`--features parquet-input --no-fail-fast`): 163 passed, 0 failed, 1 ignored, over 8 targets.
  - The targets: lib 118, `chat-grade` 26, `m-world-recall-corpus` 8 (1 ignored), `prepare-real-text` 9, `prepare-chat-parquet` 1, `prepare-text-parquet` 1, and `mix-chat-corpus` and the doc tests 0 each.
  - At the base it was 123 passed, 0 failed, 1 ignored over 7 targets. The difference is the 31 library tests and the 9 bin tests.
- **Negative gates.** Each rule was set aside, its test seen to fail, and the file restored from git:
  - **The library** (by the builders at `b5037320e` and `d9fe4c56d`, reported): each OASST2 filter; the rank order with a null rank and a tie; the split; the pin; the exclusion's three cases; the decode round trip.
  - **The bin, by me:**
    - the pin moved after the claim;
    - every document sent to train;
    - protocol 1 by default;
    - the vocabulary check;
    - the pre-claim codec check;
    - the `stopped_at_max_docs` flag;
    - the column-type error;
    - the panel-name check.
  - **The library, by me:** the walk's admission rule.
- **Style:** `rustfmt` on both new files; no line over 100 characters; no banned word. Clippy is clean on the bin in the normal and test profiles.
- **Review, in three parts:**
  - The fresh exact-head review of `8d10c7469`, in four slices, asked for changes.
  - The review of the fix commit `c810bdd2b` approved with two low items, recorded above.
  - A separate fact check of this REPORT against the manifests, token files, oracle and logs found three wording fixes, now made.

## What it unlocks

Unit 2 builds the corpus with this bin:
- OASST2 in full, and three Wikipedia shards;
- the census against the frozen test panels;
- storage under `results/claude/real-text-corpus-<date>/` with a `SHA256SUMS`.

Then P5 can train and score on human-written text instead of the templated M-world, whose replies are 75–78% verbatim in TRAIN (claude-202).

## Skills used

No skill applied. The loop's ORDER and REPORT procedure was used.

## Still open

Unit 2 (done-when 2–5):
- write the frozen panels' user turns as `{id, user_turns}`;
- decide how to handle Wikipedia articles over 8,192 tokens;
- build and census the full corpus;
- store it with a `SHA256SUMS`, and add one row to `data-tokenizer.md`.

## Plain summary

The project can now turn real human conversations (OASST2) and Wikipedia articles into training files in the format our trainer reads, with a fixed held-out split and the test prompts screened out. The review caught one real bug, a vocabulary size that was never checked against the tokenizer, and it is fixed. On the real data, 5,391 conversations and 2,000 articles came through in seconds. Next comes building the full corpus, screening it against the frozen test panels and storing it.

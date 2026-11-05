# ORDER: real-text-corpus, claude-293

Lab: claude. Item: [real-text-corpus](../../../../roadmap/items/real-text-corpus.md). Base: `43e2d3206`.

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

**This unit is done-when 2 to 5.** Done-when 1, the builders, merged in claude-292 (#3733, `43e2d3206`).

- **Claim check, (a) the step:** the decided order (#2028 6634218223) puts a human-written real-text corpus, 2 CPU units, before P5 on real text. This is its unit 2, the last.
- **Claim check, (b) the model:** nothing that trains or serves changes. This unit makes the corpus and held-out panel that P5 trains and scores on: real conversations and articles, instead of the templated M-world, whose replies are 75–78% verbatim in TRAIN (claude-202).

**Facts this ORDER rests on** (read at `43e2d3206`):
- **The five frozen panels are UORT splits, not turn lists.** None has a `{id, user_turns}` file the bin's `panel=` reads:
  - DEV is the store's `w2-inputs/2026-10-25/driver-data/ft/dev`;
  - Fresh V2 is in `inputs/fresh-v2-sealed/2026-10-26/` (a preservation tar);
  - P-dev is `inputs/claude/heldout-panels-2026-10-29/p-dev`;
  - p-long is `results/claude/one-model-end-to-end-long-context-2026-11-04/inputs/p-long`;
  - p-long4k is `results/claude/integer-long-context-2026-11-05/inputs/p-long4k`.
  - p-long and p-long4k also carry a `user_turns.json`, written by `heldout-panel` as `{conversation, user_turns}`.
- **The layout** (`dialogue.rs:325-372`): a document opens at `<|bos|>`. Every prompt token, the separating `\n` and the role markers have mask 0; a reply's content and its `<|eos|>` have mask 1. A mask-0 run decodes to `User: <content>` segments under both protocols (v1's marker carries the space, v2's content does), ending at `\nAssistant:`.
- **The trainer windows a long document;** it does not refuse one. `build_document_windows` (`hamilton_sheet.rs:1866-1927`) cuts each document into chunks of `context` targets and never crosses a `<|bos|>`.
- **The census** (`train-in-space --census 1`, `in_space_long.rs:4919`) runs on the laptop without a GPU (claude-202: 4.2 s, 1.1 GB for the 7.2M-token TRAIN). It takes `--train-split DIR` with both SHA-256s, `--fresh-v2 DIR` and `--panel NAME=DIR`. It reads DEV from `--data-root`. It holds hash sets over all of TRAIN at 8 and 13 tokens, about 150 bytes per TRAIN token. The laptop has 16 GB.

**Decisions** (mine, recorded here and in the REPORT; each is easy to reverse, since the raw sources stay in the store and the bin is deterministic):
1. **The frozen panels' user turns are read back from their splits** by a new mode, `prepare-real-text panel-turns`, through `real_text::split_user_turns`.
   - It is checked on real data: the turns it reads from p-long and p-long4k must equal their stored `user_turns.json` exactly.
2. **The exclusion screens against nine panels:** the five frozen panels, and the four chat panels the smoke used (`data/panels/conversational-v2..v5.json`, which `chat-grade` scores).
3. **Long documents are kept whole:** `max_tokens=1048576` (2^20). The trainer windows them. claude-292's 8,192 dropped the longest 1.4% of articles, a larger share of the tokens, and nothing in the trainer needs that cap.
4. **The Wikipedia part is 15,000 kept articles from each of the three stored shards** (`max_docs=15000`, one output root per shard).
   - **The size:** about 15M tokens each, so about 45M with OASST2's 4M. That is about 7 times today's 7.5M-token corpus, room for P5's data axis at several fractions.
   - **The census** fits the laptop at about 2.5 GB a split. The three full shards (about 600M tokens) would not fit.
   - More can be built later from the same shards with the same bin.
5. **OASST2 in full,** from the stored `trees.jsonl.gz` (`gunzip -k`).
6. **`heldout_per_mille=50`** for every source, as in the smoke.
7. **No mix is chosen here.** Each source's `train/` and `heldout/` is stored on its own. P5's sheet unit picks the mix and the mask, through its decision card.

## Cancel probe (run first)

- **Does a converter exist?** Search `crates/` at the base for code that reads user turns back from a UORT split. The unit is cancelled only if one exists and gives `{id, user_turns}`.
- **Does an open order hold a file in the scope?** Check every open PR's file list.
- **Is the training crate green at the base?** Run its tests with `--features parquet-input --no-fail-fast`, so that a later red is this unit's.
- **Classed:** instrument.

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

1. **`real_text::split_user_turns`** returns each document's user turns from a split's tokens and mask. It skips system turns, and skips `<|bos|>` and a terminal `<|eos|>`. It refuses, as a defect, a run that opens on no role marker, an `<|unk|>`, or a token and mask of unequal length. Tests:
   - turns encoded by `DialogueEncoder` under protocols 1 and 2 come back exactly, with a system turn and a document that ends on a user turn;
   - each refusal has its own test;
   - a negative gate: set aside the marker strip, and the round-trip test fails.
2. **`prepare-real-text panel-turns split=DIR tokenizer=FILE name=NAME out=NEW_FILE`** checks the tokenizer pin first, and writes `[{id: "NAME:<doc index>", user_turns}]`, skipping and counting a document with no user turn. It refuses an existing `out`. A test covers the bin. On real data, its turns for p-long and p-long4k equal their stored files.
3. **The corpus, built on the laptop** with the pinned tokenizer and the nine panels (decisions 2–6):
   - `oasst2/`;
   - `wiki-00000/`, `wiki-00020/` and `wiki-00040/`;
   - each with `train/`, `heldout/`, `excluded.jsonl` and manifests holding every count done-when 2 names.
4. **The census** runs once for each of the four TRAIN splits, against DEV, Fresh V2, P-dev, p-long and p-long4k, and against that source's own `heldout/`.
   - The exact prompt share is 0.0 for every frozen panel, or each nonzero share is explained by the documents behind it.
   - The REPORT lists every excluded document and every uncovered short turn.
5. **Stored** under `results/claude/real-text-corpus-2026-11-05/` beside `raw/`: the four corpora, the panels' user turns, the census JSON and a `SHA256SUMS`, read back.
   - Each Wikipedia shard is fetched one at a time and deleted after its build.
6. **One notebook row** in `data-tokenizer.md`, and the item set to `status: done`. Its render is owed if main's roadmap check is still red.
7. **Checks:**
   - the training crate's tests pass, compared with the base count;
   - a fresh exact-head review passes;
   - `loop report --check`, `loop pr-check --local`, `uor-harness check` and `loop gates --full` pass.

## Stop and decide (never wait) if

- the turns read from p-long or p-long4k differ from their stored files. Then the converter or the layout is wrong: classify, fix, and build nothing until they match;
- the census needs more memory than the laptop has. Then shrink `max_docs`, and record it;
- a frozen panel's exact prompt share is not 0.0. Then find the documents behind it, and decide between excluding them and recording why they are not shared prompts.

## Cancel probe result (5 Nov, before 16:00Z)

- **No converter exists.** Five files in `crates/*/src` name both `user_turns` and `response_mask`:
  - `stack_dialogue.rs`, `synthetic-memory-corpus.rs` and `m-world-recall-corpus.rs` write turns while they generate, or read them from JSON;
  - `real_text.rs` and `prepare-real-text.rs` are claude-292's.
  - None reads turns back from a split.
- **No overlap:** of the open PRs, only this unit's draft (#3738) touches a file in the scope.
- **The base is green:** at `43e2d3206`, `cargo test -p uor-r4-training --features parquet-input --no-fail-fast` gives 163 passed, 0 failed, 1 ignored over 8 targets.
- **The inputs match their pins:** DEV's tokens `0919e90f…` and mask `a838bb1a…` (`hamilton_sheet.rs:48-52`); Fresh V2's `0b7e754c…` and `a1cda529…` (`in_space_long.rs:4604-4609`), extracted from its preservation tar, whose `SHA256SUMS` passes; P-dev's tokens `790b7e12…`; p-long's and p-long4k's `SHA256SUMS` pass; the tokenizer `d36d3e87…`.
- **Classed:** instrument. The unit goes ahead.

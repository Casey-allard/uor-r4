---
id: real-text-corpus
title: A human-written real-text corpus and held-out panel, for P5 on real text
milestone: research
area: training
status: ready
open_to: lab
needs_gpu: false
depends_on: []
notebook: docs/research/notebook/data-tokenizer.md
scope: [crates/uor-r4-training/Cargo.toml, crates/uor-r4-training/src/lib.rs, crates/uor-r4-training/src/real_text.rs, crates/uor-r4-training/src/bin/prepare-real-text.rs, crates/uor-r4-training/tests/real_text*.rs, docs/harness/prep/real-text-corpus*, docs/roadmap/items/real-text-corpus.md, docs/loop/tasks/real-text-corpus/**, docs/research/notebook/data-tokenizer.md]
---

## Why it matters

Every corpus and panel so far is the templated M-world. claude-202's census found 75–78% of DEV's and Fresh V2's replies verbatim in TRAIN, so a scale probe on it measures how fast a model memorizes templates, not how it scales on language. The decided order (#2028 6634218223) puts a real-text corpus before P5, so that P5's slope is read on real text, with the templated data kept as one contrast cell.

## What is missing

- **What exists already:**
  - **The raw sources are in the private store** (Casey, 5 Nov, in claude's chat: "Yes, download both"): `results/claude/real-text-corpus-2026-11-05/raw/`, with a `SHA256SUMS` and a `SOURCES.md` (revisions, licences, attribution), read back on 5 Nov.
    - **OASST2:** `2023-11-05_oasst2_ready.trees.jsonl.gz`, Apache-2.0, at revision `179dd21fc55192153d94adb0e0ce8f69e222bf75`.
    - **English Wikipedia:** three of `wikimedia/wikipedia` `20231101.en`'s 41 shards (00000, 00020 and 00040), CC BY-SA 3.0 and GFDL, at revision `b04c8d1ceb2f5cd4588862100d08de323dccfbaa`.
  - **`prepare-chat-parquet`** (feature `parquet-input`) already writes the trainer's format: `tokens.u16` and `response_mask.u8` through `CorpusWriter`, with a `uor-r4-chat-corpus/v1` manifest. It encodes with `DialogueEncoder` (`crates/uor-r4-tokenizer/src/dialogue.rs:226`).
  - **`parquet` 60.0.0** is in `Cargo.lock`, behind the training crate's `parquet-input` feature (`crates/uor-r4-training/Cargo.toml:15-16`).
  - **`train-in-space --census 1`** counts the exact turns and token n-grams a TRAIN split shares with DEV, Fresh V2 and each named panel (`in_space_long.rs:4919`, `census`). It counts only; it filters nothing.
  - **`knowledge_corpus::PanelExclusion`** (`crates/uor-r4-training/src/knowledge_corpus.rs:141`; the rule is in the module doc, `:4-18`) has two parts, after `milestone_world::words` splitting (lowercased, punctuation removed):
    - a document is dropped whole when any of its 8-word windows equals an 8-word window of a panel user turn;
    - a panel turn of 4 to 7 words (`MIN_SHORT_REQUEST_WORDS = 4`, `:34`) is matched as a whole word sequence instead, and a turn of fewer words is counted as uncovered, not matched.
    - Every hashed hit is confirmed against the stored words, so a collision drops nothing.
    - The module is compiled out (`lib.rs:162-163`), and it imports `EXCLUSION_NGRAM` from `milestone_world_v2_probe`, which is compiled out too (`lib.rs:171-172`). `prepare-knowledge-corpus.rs` is not a registered bin.
    - So the rule, both parts and the uncovered count, is carried into the new `real_text.rs` with its own constants, rather than turning two historical modules back on. `milestone_world` is compiled in (`lib.rs:169`), so its `words` can be reused.
- **OASST2's trees as conversations:** nothing reads the trees.
  - Read them from JSONL, after `gunzip -k` on the laptop, so no gzip crate enters `Cargo.lock`.
  - Keep English only, and drop deleted messages, messages that failed review, and messages flagged `synthetic`.
  - Follow the best-ranked reply at each turn: the lowest `rank` first, a reply with no rank after every ranked one, and a tie broken by `message_id`. A test pins this rule.
  - Map `prompter` to `user`.
- **Wikipedia as pretraining text:** each article as `<|bos|>`, ` {title}\n\n{text}`, `<|eos|>`.
  - The masking choice belongs to the sheet unit: every target, or the trainer's assistant-only mask.
  - `prepare-text-parquet` writes no opening BOS, which the trainer requires.
  - The shards' parquet codec is unread; check it before relying on the build's readers.
- **The tokenizer pin:** the prepare bins record the tokenizer's SHA-256, but do not check it against `d36d3e87` (`crates/uor-r4-float-train/src/hamilton_sheet.rs:55`).
- **A held-out real-text panel,** split by tree id and by article id before tokenizing. P5 scores it, as the decided order says, by integer top-1 against unigram and T2 count tables, with bytes per token under the pinned tokenizer.
- **The census against the frozen test panels:** DEV, Fresh V2, P-dev, p-long and p-long4k. Run the token census, plus the exclusion rule over their user turns, and drop and count every document it excludes.
  - Wikipedia's documents have no turns, so for them only the census's token n-gram counts apply, besides the exclusion rule.

## Done when

1. **The builders,** in `real_text.rs` and one bin, `prepare-real-text`, with tests and negative gates:
   - OASST2's tree to conversations, with each filter pinned by a test;
   - Wikipedia's parquet to BOS-opened documents;
   - the split by source id;
   - the tokenizer pin refused on a wrong SHA;
   - the exclusion rule, with tests that pin an 8-word hit, a 4-to-7-word turn matched whole, and a shorter turn counted as uncovered.

   The training crate's tests pass.
2. **The corpus and its panel,** built on the laptop (data preparation, not training; see Rules that apply).
   - **Each manifest records** the source, revision, licence and attribution; the counts of trees, threads, articles and tokens; the rows dropped, and why; and bytes per token.
3. **The census against the frozen test panels** reads no shared prompt, with every excluded document counted and every uncovered short turn counted.
4. **Storage:** in the private store under `results/<lab>/real-text-corpus-<date>/`, with a `SHA256SUMS`, read back. No corpus is committed.
5. **One notebook row in `data-tokenizer.md`, and a REPORT with the negatives first.** P5's sheet unit then moves P5's Data line to real text, through a decision card.

## How to start

Read:
- `prepare-chat-parquet.rs`, for the writer, the manifest and the encoder;
- `knowledge_corpus.rs`, lines 141 onward, for the exclusion rule and its tests;
- the store's `SOURCES.md`, with `scripts/pod/uor-store ls results/claude/real-text-corpus-2026-11-05/raw/`.

Unpack OASST2 with `gunzip -k` into the unit's `local/<lab>/`. Fetch one Wikipedia shard at a time with `uor-store get`, and delete it once its documents are written; the laptop's disk floor is 30 GiB.

## Rules that apply

- **Rust only** (CHARTER §3.11): the builders are Rust in the training crate; no ad-hoc scripts.
- **No corpus, binary or secret is committed** (§3.13): corpora live in the private store, and the repository records their hashes.
- **Data preparation runs on the laptop.** §3.5's GPU-first rule covers training, qualification and the model and parity executions; building a corpus is none of these, and AGENTS.md's Compute rule runs CPU-only builds and checks on the laptop. Training on the corpus belongs to P5's GPU units.
- **Kept out until Casey's two §3.10 readings** (the decided order): model-written sets (UltraChat, SmolTalk), and FineWeb-Edu's classifier-filtered text.

## Sources

- Casey's download approval, 5 Nov, in claude's chat (quoted above), logged on #2028 (6650617655).
- The decided order, #2028 6634218223 (the real-text corpus before P5, human-written only).
- claude-202's census (`one-model-end-to-end`, #3407) for the verbatim share of the templated panels.

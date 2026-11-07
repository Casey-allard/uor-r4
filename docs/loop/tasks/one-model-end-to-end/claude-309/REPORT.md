# REPORT: one-model-end-to-end, claude-309

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](./ORDER.md). Laptop only; no GPU, no spend.

## What did not work, first

- **(b64L) answers none of `m2-fresh256`: 0 of 256 rows complete, 0 of 128 pairs, on both seeds.** The bar is 104 rows and 52 pairs. Every reply ran to the 64-token cap: 0 EOS stops, 0 BOS stops, 0 context stops. Every stratum is 0 (length2, length4, length8, reassert, update).
- **The miss is the model's, not the scorer's or the serving path's.** Three controls:
  - **Loose match:** I dropped the EOS condition and looked for each row's accepted answer (lower-cased, final period dropped) anywhere in the reply text. 0 of 256 on each seed: no reply contains a full accepted answer. Some replies copy part of a stated fact (33 rows on seed 1 and 29 on seed 2 hold a 4+-letter word from the answer), but they give the same reply for the job question and the home question. That is copying, not recall.
  - **The float model does not stop where its references stop:** the trained float model's 32-token greedy replies on DEV are in the seal (`samples-memory.json`). All 16 references end within 25 tokens (EOS at offsets 6 to 25). The float model emits EOS id 1 on 0 of 16 prompts, on both seeds. On oasst2 (`samples-oasst2-greedy-memory.json`, seed 1), 2 of the 16 references end within 32 tokens, and the float model stops on neither. The served replies on the same prompts (claude-308's seed-1 `i3/`) never emit EOS either. Not ending a reply is the trained model's behaviour, not a product of export or serving.
  - **The scorer can pass a row:** the known-answer test scores an accepted answer followed by EOS as complete. The same reply without EOS scores incomplete. With the EOS condition set aside, that test fails, and it passes again once the condition is restored.
- **The replies are loops, not answers.** On seed 1, 76 of the 256 replies are the same text: " My curopeans (1999…9". Seed 2's most common reply is " My currently live in the 1999…9" (36 rows). The served model copies words from the prompt, then repeats a digit, a letter or a prompt phrase (seed 1's second most common reply, 53 rows, is " Indians (dddd…").
- **This closes the configuration, not the mechanism (§4.1).** It covers (b64L) at 16,384 steps on D3, under the `one-user-turn/v1` rendering. The other rendering named in the ORDER, with facts in separate turns, is unread. The loose-match control shows recall is absent under this rendering, so a different layout alone is not expected to change the reading.

## What changed

- **New bin `served-reply-panel`** in the item's instrument crate (`docs/labs/one-model-end-to-end/instrument/src/bin/served-reply-panel.rs`, feature `cpu-parity`). It works in five steps:
  1. **Inputs:** it checks the panel's `inputs`, `labels` and `plan` and the tokenizer against the manifest's shas, and refuses an artifact that serves the writer head.
  2. **Rendering:** it decodes each packet's `Context` and query ids and skips `Source`. It joins them into one user message and encodes it with `DialogueEncoder::encode_assistant_prefix` (protocol 2), so the prompt ends on the open `Assistant:` marker.
  3. **Generation:** it drives `CpuFbsForward` greedily with the nearest head. The stop rule runs in this order: EOS, then BOS, then the cap, then the context limit. The library's `generate_reply` stops on id 0 (the served BOS), not on EOS id 1, so the bin uses its own rule.
  4. **Scoring:** it drops the reply's leading protocol-2 space and applies `FrozenAnswers::accepts`. A row is complete only when it stopped on EOS, decodes as UTF-8, and is accepted. `pairs_both_complete_distinct` compares reply texts within each `pair_id`. Strata come from `plan.json` and are checked against the manifest's `strata_rows`.
  5. **Output:** a JSON file with totals, per-stratum counts and every row.
- **`Cargo.toml`:** the bin, and `dep:uor-r4-core` added to `cpu-parity`. `Cargo.lock` gains 4 dependency lines under `uor-r4-cpu-train` (it was behind that crate's own dependencies). No new crates.
- **Departures from the ORDER's pinned text, both decided before any reading.**
  - The ORDER names `encode_document`. The bin uses the same encoder's `encode_assistant_prefix`, which shares its encoding code and ends on the open `Assistant:` marker, where `encode_document` would close the turn.
  - The stop rule adds a BOS stop, as the served path has, ahead of the cap and the context limit.
  - Neither changed a reading: 0 BOS stops on both seeds.
- **Review round 1 fixes (code):**
  - The context stop now equals the served `reply_stop` (`point_key.rs:305`) for every prompt length from 1 to 192 and every kept count from 0 to 63. A test pins this; with the old, one-token-early rule, the test fails.
  - A prompt exactly the context's length is no longer refused.
  - Distinctness compares stripped reply text.
  - The tests read the tokenizer from `UOR_PANEL_TOKENIZER` and pass with a note when it is unset, instead of panicking on a laptop path.
  - Re-scored with the fixed bin, both seeds' rows are identical to the first run's.
- **Queue fix (D32, no float in an instrument's compiled code):** the queue's banned-presence check counted 12 float sites in the bin. One was the `f64` tokens-per-second; the other 11 were tuple indexes after `]` or `)` (`arms[0].1`), which its lexer reads as float literals. Timing is now integer milliseconds, and the tuples are destructured. A third run's rows are identical to the second's on both seeds.
- **The harness exception, as recorded in the ORDER:** `loop claim` refused on deepseek's `hamilton_sheet.rs` overlap, so the ORDER was written by hand. The builder subagent then got a `loop suborder` scoped to the instrument crate.

## Measurements

Panel `m2-fresh256` (inputs `1de2fc28…`, labels `3b5c5477…`, plan `0ed1ab98…`, tokenizer `d36d3e87…`). Artifacts as in claude-308. The table is the third run, with the float-free bin `be3d4cb0…` (see What changed). The first run (bin `424bd917…`, outputs `0d6dec9a…`, `066dc827…`) and the second (bin `38d53fb6…`, outputs `a6adc15b…`, `f0088f6c…`, 912.9 and 914.8 tok/s) gave identical rows; the output shas differ only by timing.

| Seed | Rows complete | Pairs both complete, distinct | EOS / BOS / cap / context stops | Full answer anywhere in reply | Tokens/s | Output sha256 |
|---|---|---|---|---|---|---|
| 1 | 0 of 256 | 0 of 128 | 0 / 0 / 256 / 0 | 0 | 779 | `d02ec432c52dfbe5c858b12e03e7f047da639c8f0fbc8a184dee3c5901bfac25` |
| 2 | 0 of 256 | 0 of 128 | 0 / 0 / 256 / 0 | 0 | 892 | `8eef4db1fb33fdf21087d14de8a9ef7b8266729a8bc2a4be8c0ef2ab8f92497b` |

Each seed generated 16,384 reply tokens in 18 to 21 s (seed 1 ran while another build shared the laptop). Tokens per second is now an integer: tokens × 1000 / milliseconds. The timer starts after the inputs are checked, and it includes each row's artifact parse and prompt steps.

## Checks run at the head

- `cargo build --release --offline --locked --features cpu-parity --bin served-reply-panel`: exit 0.
- `UOR_PANEL_TOKENIZER=<pinned tokenizer.json> cargo test --offline --locked --features cpu-parity --bin served-reply-panel --no-fail-fast`: 5 passed, 0 failed. The tests cover:
  - the rendering of a two-context packet with a `Source` between them;
  - the order of the stop rules;
  - cap and context parity with the served `reply_stop`;
  - acceptance, including the leading-space strip and a non-UTF-8 reply;
  - the pair metric.
- **Negative gates:**
  - With the EOS condition taken out of `complete`, `known_answer_needs_eos_and_strips_leading_space` failed (exit 101; the builder's run).
  - With the context stop put back one token early, `cap_and_context_stops_match_the_served_rule` failed (my run).
  - Both pass after the restore and a rebuild.
- **Banned literals:** no hit in `git diff f5adca314 -- docs/labs`.
- `loop report --check` on this REPORT.

## Skills used

None beyond the loop's own steps.

## Still open

- **The model does not end a reply or recall a stated fact.** Both are training problems, not pipeline problems. The pipeline is now measured end to end: train, export, serve, then score whole replies on a held-out M2 panel, laptop-only, about 20 s per seed. The next training decision can be judged by this score. A run is worth its spend only if it moves this score off 0, or at least gets the model emitting EOS on its own corpus.
- **`m2-reply512` is not scored.** Its `inputs.json` and `labels.json` are not located in the store. Only their hashes are recorded (`b9661606…`, `84991e0…`).
- **The per-turn rendering is unread.**
- **claude-308's one-scorer parity** (served against trained on the same scorer) is still open.

## Plain summary

There is now a tool that asks the question that matters: given a short conversation that states some facts, does the served model answer a question about them correctly, and stop? It runs the shipped integer model on the laptop at roughly 800 to 900 tokens per second and scores all 256 held-out questions in about 20 seconds.

The current best model gets 0 of 256 on both seeds. No reply contains the right answer, though some echo bits of the stated facts whatever the question. And it never stops talking: it runs into a string of 9s or a repeated letter or phrase. The trained model, before export, also fails to stop where its reference replies stop, so this is a training problem, not a serving problem. With the path from training to serving to scoring now working end to end, the next training run can be judged by whether it moves this number.

# REPORT: one-model-end-to-end, claude-310

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](./ORDER.md). Laptop only; no model, no GPU, no spend.

## What did not work, first

- **EOS was a rare training target for (b64L): 756 per million of its scored targets.** D3 scores 27,416,554 targets, and 20,729 of them are EOS, about one every 1,320. DEV, the conversations its replies are judged on, has 85,812 per million, about one every 12 scored reply tokens, about 113 times more often. This is consistent with claude-309's reading (the served model stopped on 0 of 256 panel replies, and the float model emitted EOS on 0 of 16 DEV prompts). No run here varied the EOS share, so it does not test the cause.
- **Most of D3's scored targets, and most of its EOS, are Wikipedia.** The wiki roots are scored in full (every non-BOS token) and hold 24,181,749 of the 27,416,554 scored targets (88%). Each wiki article ends in one scored EOS: 12,945 of the 20,729. Between consecutive scored EOS, the median run is 510 scored targets over all of D3. In DEV it is 11 (max 50).
- **In oasst2 the mask is a reply mask, but the replies are long.** It leaves 533,543 non-BOS tokens unscored: the non-reply tokens (user and system turns, role markers, separators). This matches the encoder, which gives mask 1 only to assistant content and its EOS (`crates/uor-r4-tokenizer/src/dialogue.rs:11-14`). The 1,599 unscored EOS fit the encoder's unmasked terminal EOS for documents that end on a non-assistant turn; this unit did not count them per document. oasst2 has 7,784 scored EOS in 3,234,805 scored targets, 2,406 per million: the median reply runs 349 scored targets and the mean about 416. DEV scores 33% of its positions, and its replies are short.

## What changed

- **New bin `corpus-eos-exposure`** in the item's instrument crate (`docs/labs/one-model-end-to-end/instrument/src/bin/corpus-eos-exposure.rs`, no feature needed). It reads a `tokens.u16` (bare, or behind the 64-byte `UORT` header when the header's count equals the mask's length) and its `response_mask.u8`. It refuses a mask byte other than 0 or 1, or a length mismatch. With a manifest, it splits the file into the manifest's roots in order. It checks that their `tokens_taken` sum to the file and that each root's scored count equals its `scored_taken`.
- **The scoring rule is the trainer's**, read at the pin `ff5156bcc` (`crates/uor-r4-float-train/src/hamilton_sheet.rs`, `build_document_windows`):
  - :1868 describes "scoring each eligible target position (`p > doc_start && tokens[p] != bos_id && mask[p] == 1`)".
  - :1903–1905 pushes `tokens[p]` as the target, and :1902 reads its input at `p - 1`. Context never crosses a BOS (:1894).
  - The sliding variant (:1967, :2020) and `in_space_long.rs` (:845, :2682) use the same rule.
- **Review round 1 (code):** each root's `scored_taken` must now equal the trainer-scored count; the looser match on raw mask ones is gone. Each root must also start at a BOS, so no document spans two roots. A test pins each check; with the BOS check disabled, two tests fail. The D3 output is byte-identical after the change (`e6534537…`).
- **Per root and in total, it reports:** positions, scored targets, EOS, scored EOS, BOS, and the runs of scored targets that end at each scored EOS (count, median, max). Runs reset at each BOS.

## Measurements

D3 (`tokens.u16` `ed8f2537…`, `response_mask.u8` `ec6e520c…`, its manifest) and DEV (`ft/dev`, tokens `0919e90f…`, mask `a838bb1a…`).

| root | positions | scored | EOS | EOS scored | scored EOS per million | median run |
|---|---|---|---|---|---|---|
| oasst2 | 3,773,419 | 3,234,805 | 9,383 | 7,784 | 2,406 | 349 |
| wiki-00000 | 8,011,538 | 8,010,674 | 864 | 864 | 107 | 6,546 |
| wiki-00020 | 8,255,877 | 8,247,907 | 7,970 | 7,970 | 966 | 466 |
| wiki-00040 | 7,927,279 | 7,923,168 | 4,111 | 4,111 | 518 | 1,223 |
| **D3** | 27,968,113 | 27,416,554 | 22,328 | 20,729 | 756 | 510 (max 96,128) |
| **DEV** | 42,856 | 14,287 | 1,226 | 1,226 | 85,812 | 11 (max 50) |

The rates are parts per million, truncated to an integer, as the bin records them per root and in total. The cancel probe passed: no open PR or claim counted D3's EOS targets, and D3's two arrays hashed to the pinned `ed8f2537…` and `ec6e520c…`. Outputs: `out/d3.json` `e6534537f2c2baf285d474da7de47aaa580e0fdea7f77d8abb1508cc25d06ede`, `out/dev.json` `19225cd3fbcad73111d600e2ef0252d3721c5bed9a299fa0ab32179140d1c18a`. Every root's scored count matched its manifest `scored_taken`, and 0 scored targets were left without a closing EOS.

## Checks run at the head

- `cargo build --release --offline --locked --bin corpus-eos-exposure`: exit 0.
- `cargo test --offline --locked --bin corpus-eos-exposure`: 6 passed. The tests cover the scoring rule (where the target at p and at p+1 give different EOS counts), the per-root split and its sum check, the mask and length refusals, and the run statistics.
- **Negative gate:** with the scoring rule swapped for the next-token rule (`mask[p - 1]`), the rule's test and two others failed (3 of 6). They pass again after the restore and a rebuild (the builder's run; I re-ran the 6 tests at the head).
- **No float sites:** none of the float type tokens or float-literal patterns in the bin.

## Skills used

None beyond the loop's own steps.

## Still open

- **The lever this points to:** more dialogue with short replies in TRAIN, so that EOS is nearer DEV's rate, or weight the EOS target. Each would be judged by claude-309's `served-reply-panel`: whether replies stop, then whether they are right. That run needs its own prep sheet and pre-run review.
- **Not measured:** the trained model's EOS rank at the end of a reference reply. That would show whether EOS is nearly chosen or nowhere close.

## Plain summary

The model we serve never stops talking. This unit counted one possible reason. Among the tokens the model was trained to predict, an "end of reply" mark comes about once every 1,320, mostly at the end of Wikipedia articles: 88% of what it learned to predict was Wikipedia. In the conversations it is tested on, one comes about every 12 reply tokens. That is consistent with a model that keeps writing rather than stops, but nothing here tests it. The next training change to try is more short-reply dialogue, or extra weight on the end mark, judged by whether the served replies stop and are right.

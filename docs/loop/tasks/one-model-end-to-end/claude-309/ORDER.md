# ORDER: one-model-end-to-end, claude-309

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `f5adca314`.

## Goal

**A scorer for a served model's whole replies on M2's held-out panel, and its first reading on (b64L).** claude-308 (#3822) showed claude-301's (b64L) exports and serves end to end. No tool yet asks whether a served `UORLUT01` artifact's reply is a correct answer:
- The panel grader (`native-bank-generalization.rs:829-953`) loads only native-geometric field checkpoints.
- `m2-evidence` and `reply_eval` score token corpora, not the panel's packets, and count any EOS stop as complete.

This unit adds a CPU bin, `served-reply-panel`, to the item's instrument crate. It renders each `m2-fresh256` packet as a dialogue, generates a reply with the served artifact, and scores it by the panel's own acceptance rule. It then records (b64L)'s reading.

**Claim check.**
- **(a) The step:** Casey's "Yes, pipeline first" (#2028, 6709947526). The path from training to serving ends in a score of whether replies are right. That score is what every later training run is judged by.
- **(b) What it changes in the model:** nothing in the weights. It is the measuring instrument for the served model that M2 is decided on. This unit unblocks the next training decision: a run is worth its spend only if this score can move.

## Harness exception (recorded)

`uor-harness loop claim one-model-end-to-end --lab-topic served-reply-score` refused: the item's scope names `crates/uor-r4-float-train/src/hamilton_sheet.rs`, which deepseek's open `float-kernel-fusion` order (PR #3827) also names.
- This unit touches no file in that order and no `crates/` file. Its files are listed below.
- Under Casey's standing approval ("i approve you to make harness exceptions as needed to do what I ask"), this ORDER is written by hand, with the narrower scope below.

## Cancel probe (run first)

Every check passes, or the unit is cancelled:
1. No open PR or claim, by any lab, scores `UORLUT01` replies on `m2-fresh256` or `m2-reply512`.
2. `data/m2-fresh256/{inputs,labels,plan}.json` in the store hash to the panel manifest's shas (`1de2fc28…`, `3b5c5477…`, `0ed1ab98…`). The manifest's `tokenizer_sha256` equals the tokenizer in claude-308's grades (`d36d3e87…`).
3. claude-308's artifacts are present at their recorded shas (`9a370566…`, `5039cade…`).

## The rendering, pinned before any reading

The packets carry token ids. The scorer decodes each `Context` segment's `token_ids` and the `query_ids` to text with the pinned tokenizer. It then encodes one dialogue with `DialogueEncoder::encode_document` (protocol `uor-r4.literal-role-dialogue/2`), the encoder (b64L)'s oasst2 corpus was built with:
- **One user turn:** the `Context` texts in event order, then the question, joined by single spaces.
- **Then the assistant marker:** generation starts after it.

`Source` segments are skipped. Their `original_source_ids` repeat a span of the `Context` text, which a language model already reads.

Why one turn: the training corpus alternates user and assistant turns. Splitting the facts across several user turns would need either back-to-back user turns, which the corpus never has, or invented assistant replies. A per-turn rendering is a later configuration. A miss under this rendering closes this configuration only, never the mechanism (§4.1).

## Scoring, the panel's own rule

- **Decoding:** greedy, with the artifact's nearest head, as `fbs-toks` serves it.
- **Stop:** at the tokenizer's `<|eos|>` (id 1), the 192-token context, or 64 reply tokens.
- **Complete:** a row is complete when it stopped on EOS, its reply decodes as UTF-8, and `uor_r4_core::answer_oracle::FrozenAnswers::accepts` accepts the text (`answers.accepted`).
- **The bar's metric** is `pairs_both_complete_distinct` over `pair_id`, as the manifest names it: both rows of a pair are complete and their outputs differ. The manifest's bar is 104 of 256 rows and 52 of 128 pairs.
- **Reported:** rows complete, pairs, and stops by kind. Per stratum: length2, length4, length8, reassert and update. Also tokens per second.

This is a CPU development reading of the served product path, not an M2 qualification: the qualifying reading stays on the GPU (§3.5, D84).

## Files in scope

- `docs/labs/one-model-end-to-end/instrument/**` (its `Cargo.toml`, `Cargo.lock`, and the new `src/bin/served-reply-panel.rs` and its tests)
- `docs/loop/tasks/one-model-end-to-end/claude-309/**`
- `docs/research/notebook/float-training.md`

## Done when

1. `served-reply-panel` builds `--offline --locked`. Its unit tests pin each of the following on hand-made packets: the rendering, the stop rule, acceptance and the pair metric.
2. A known-answer test: a reply that is a panel answer scores complete, and the same reply without its EOS does not.
3. (b64L) seeds 1 and 2 scored on all 256 rows, with the JSON outputs and their shas recorded.
4. The REPORT, negatives first, with tokens per second, and one notebook row.

## Stop and ask if

Nothing waits on Casey (§5.1). Laptop only; no GPU, no spend. Bound: two units. Past the bound, record it in PLACE.md and go on.

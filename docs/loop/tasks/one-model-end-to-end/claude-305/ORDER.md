# ORDER: one-model-end-to-end, claude-305

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `5388788e4` (`origin/main`, 7 Nov 2026; claude-304 merged).

## Goal

This serves Casey's one track (#2028 6675163438, "Yes, one track"). claude-301's (b64L) met the bar at width 256 on TRAIN D3 (27,968,113 tokens), "met; unseen ahead". claude-304's decision review stopped width 1,024 and named the outcome map's default as the next step: more human text at width 256.

claude-301's tokens lever was `ahead`, but those were more positions over the same text. (b64L)'s 201.3M positions are about 7.2 passes over D3. More distinct text has not been tested.

This unit builds **TRAIN D4**, about 2–3× D3, and the tables the bar needs, on the laptop, at no cost. A later unit trains (b64L) on it under its own sheet. Splitting the work keeps the GPU spend in a unit whose data is already known to be sound.

How D4 is built (scout, from claude-292/293's REPORTs and the scale-probe sheet):
- `prepare-real-text` again, on the same three wiki shards already in the store: `wikimedia/wikipedia` 20231101.en, shards 00000, 00020 and 00040.
- The same tokenizer, protocol, `heldout_per_mille=50`, `max_tokens` and panels as claude-293. Only `max_docs` grows, about 3× each.
- OASST2 in full, unchanged.
- The split hashes each document's id. A larger `max_docs` therefore keeps every old document on its old side, and the held-out panel stays claude-293's four `heldout/` folders (1,460,697 rows).
- T2, the unigram and the seen set depend on TRAIN, so they are rebuilt on D4.

## Claim check

- **(a) The step:** Casey's one track, the outcome map's default, "more human text at width 256" (claude-302's sheet, and claude-304's decision review).
- **(b) What it changes in the model:** the training data. This unit makes the TRAIN the next (b64L) run trains on, and the baselines it is scored against.

## Cancel probe (run first)

1. The kill switch is off.
2. No pod of mine is running.
3. The three raw shards and OASST2 are in the store under `results/claude/real-text-corpus-2026-11-05/raw/`, with `SHA256SUMS`.
4. The laptop has at least 25 GiB free.
5. **Determinism:** rebuild `wiki-00020` at claude-293's own settings. It must reproduce the stored `train/` and `heldout/` files byte for byte. If it does not, the old panel cannot be reused, and this unit stops as an instrument finding.

**Result, 7 Nov, before any D4 work. Class: instrument. GO.**
1. The kill switch is off; `loop next` gives no hold.
2. 0 pods of mine are running; claude-304's pod is down.
3. The raw shards are in the store, and `SHA256SUMS` checks OK for train-00020.
4. The laptop has 39 GiB free.
5. The rebuild of `wiki-00020` (max_docs 8400, tokenizer `d36d3e87…`, `prepare-real-text` at `5388788e4`) is byte-identical on train and held-out `tokens.u16`, `response_mask.u8` and `excluded.jsonl`. Both manifests differ only in their `path` strings, the input and panel paths on this laptop, with 0 other differing lines. It ran in 3.1 s at a peak of 97 MB.

## Done when

1. Every D4 root is built, and each old `heldout/` folder is byte-identical to claude-293's. Each new root's documents form a superset of the old root's, on the same sides.
2. D4's mix, T2, unigram and seen set are built with the scale-probe instrument at `e052abf41`, and their counts on the panel are recorded. The census is run per root if the laptop's 16 GB requires it.
3. D4 and its tables are sealed in the store with a `SHA256SUMS` read back.
4. A REPORT with the negatives first, one row in `docs/research/notebook/float-training.md`, and `References #3407`. The REPORT names the next unit's TRAIN by sha.

## Stop and ask if

"Ask" means decide, record it here and in the REPORT, and go on; nothing waits.
- The determinism check fails. That is an instrument finding: record it, and do not build D4 on a different panel.
- A build or the census exceeds the laptop's memory. Split it per root, or smaller. Moving any of it to a pod needs its own cost line.
- The laptop's free disk falls under 10 GiB.
- Casey says "hold".

## Files in scope

- `docs/labs/one-model-end-to-end/**`
- `docs/harness/prep/one-model-end-to-end*`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`
- `crates/uor-r4-float-train/src/hamilton_sheet.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/flock_read_memory.rs`
- `crates/uor-r4-float-train/src/flock_read_hamming.rs`
- `crates/uor-r4-training/examples/m-world.rs`
- `crates/uor-r4-float-train/src/fbs_export.rs`
- `crates/uor-r4-integer/src/fbs_snapshot.rs`
- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/stack/hamming.rs`
- `crates/uor-r4-integer/src/stack/synthetic.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-training/src/bin/m-world-recall-corpus.rs`
- `crates/uor-r4-training/src/bin/mix-chat-corpus.rs`
- `crates/uor-r4-training/Cargo.toml`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `crates/uor-r4-training/src/milestone_world_v2.rs`

Run, not edited: `crates/uor-r4-training/src/bin/prepare-real-text.rs` and `docs/labs/scale-probe/instrument` (at `e052abf41`). Outputs go to `local/claude/claude-305/` and the store.

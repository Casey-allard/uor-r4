# ORDER — written-state-count-model (claude-411)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Unit: claude-411. Task: `written-state-count-model`, the held-out split of the D87 donor
corpus before any held-out BPB is scored on it.

## Goal

A Rust instrument that splits the D87 donor corpus (D87, `docs/integration/DECISIONS.md`;
records as in claude-403's ORDER) into TRAIN, DEV-tune and DEV-test **by prompt**, removes
exact and near duplicates, writes per-split manifests, and audits the split with an exact
13-gram TRAIN/DEV overlap census on the native 4,096-id token stream. The red team found that
DEV leaks: alternatives of one prompt (up to four under D86) can sit on both sides, and long
n-gram counts would memorize DEV. A held-out number means nothing until this census passes.

## Scope

- The standalone package `docs/labs/d87-split-audit/instrument/` (its own `[workspace]`), with
  two binaries, `d87-split` and `d87-audit`, and fixture tests on small synthetic shards.
  Tokenization reuses claude-403's `student-ingest` library (path dependency), so the audited
  stream is the trainer's stream.
- A census run on wick's container `uor-r4-claude` (CPU only, 12 CPUs, 16 GB): the shards of
  `api-a` and `part-0` are copied from the host's
  `/mnt/plexpool/uor-r4-corpus/m2-donor-corpus-2026-11-16/` into `/workspace/in/d87/`, each
  copy's sha256 checked against its original. The originals are never moved, renamed,
  rewritten or recompressed, and the copies are read-only inputs.
- This is a data-preparation census, not GPU training or evaluation: it trains nothing, scores
  no model and uses no GPU, so it needs no pre-registration sheet (§4.5).

## Rules the instrument follows

1. **Split key.** D87 records carry no separate prompt-id field. The bank (claude-400,
   `docs/labs/m2-donor-corpus/instrument/src/bank.rs`) names each record
   `<source>-<sha256(normalize(first user text))[..16]>`, plus `-t2` for a two-turn record, so
   the id is itself derived from the first user message. The split key is therefore a keyed
   SHA-256 of the bank's `normalize` of the **first user message**; this keeps together every
   alternative of one prompt, the same prompt under two sources or two banks, and a prompt
   generated twice (pod 3 and the API). The bucket is the key's first 8 bytes modulo the
   ratios' common denominator. Ratios are integer parameters; default DEV-tune 1/200 and
   DEV-test 1/100 of prompts.
2. **Duplicates**, judged in a fixed order (shard paths sorted, then line order), first kept:
   (a) exact: SHA-256 of the record's whole line text; (b) repeated id; (c) near: the same
   normalized answer text (the bank's `normalize` of every turn's answer, joined) under a
   different id, unless it is shorter than a floor (default 32 normalized bytes), so that
   short common answers ("06", "yes") are not counted as duplicates. Each rule is exact and
   deterministic.
3. **Manifests**: per split, the sorted record ids, and counts of records, prompts and dropped
   duplicates per rule; `split-summary.json` holds integer ratios only. No floating point in
   any count.
4. **Audit**: each DEV split's 13-grams are packed exactly (13 ids × 12 bits = 156 bits in
   three u64 words), so a match is an exact 13-gram compare, with no hash collision. Grams do
   not cross record boundaries. TRAIN's grams are streamed against the sorted DEV set; each DEV
   position counts once, however often TRAIN repeats its gram. Gate (the track's decision
   P0(d)): overlapped DEV-test positions ≤ 1% of DEV-test positions, or the split is redone.
5. **Target count**: per split, tokens, response-mask tokens and text bytes, with bytes per
   target as integer pairs, to reconcile the claimed 1.87e9 targets with the 2.1e9 figure.

## Gates

- G1: `cargo test --manifest-path docs/labs/d87-split-audit/instrument/Cargo.toml` passes all
  fixture tests (prompt alternatives stay together; same output hash twice; a planted duplicate
  is removed; a planted TRAIN/DEV 13-gram is counted exactly once; a DEV with no overlap
  reports 0).
- G2: the same build with `--locked` from a clean target directory compiles.
- G3: the wick census completes with a `split-summary.json` with a nonzero record count, and
  the overlap fraction is reported.
- G4: the overlap fraction is checked against 1%: PASS, or FAIL with the redo documented.
- G5: `git log --format=%s origin/main..HEAD` shows this ORDER first and REPORT last.

No GPU; no pod; no corpus, manifest over 1 MB or binary is committed.

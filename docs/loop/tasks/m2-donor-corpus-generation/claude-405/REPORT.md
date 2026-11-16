# REPORT: m2-donor-corpus-generation, claude-405 (the D87 corpus recorded as a protected asset)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **There is no permanent snapshot yet** [claude]. Wick's dataset `plexpool/uor-r4-corpus` has only rolling hourly `zfs-auto-snap` snapshots, and these expire. A named snapshot needs `sudo`, so it is left to Casey as a recommendation.
- **The 770× figure is a projection** [claude]. It uses claude-403's 1.85× token ratio, which was measured on pod 3's records only. Nobody has tokenized `api-a` with our tokenizer yet.
- **The first draft overstated three things** [claude]. The fresh review caught them:
  - it said `part-0` could be regenerated;
  - it called this the first large training corpus;
  - it said each folder carries a storage manifest.

  All three are corrected at this head.

## What changed

`docs/model/data-and-tokenizer.md` gets a new section, **The D87 donor corpus: a protected asset**. It covers:
- what the corpus is: two sealed parts with their Merkle roots;
- what it cost: about $854 per claude-400, with the two unused fan-out pods named separately;
- why it matters: projected scored targets, no bit-replay of either part, independence from the tokenizer, and the provenance kept with it;
- where both copies live;
- the rules: never delete, move or rewrite either copy; derived streams are caches; runnable check steps; the recommended snapshot.

The page's tables also change:
- the corpora table and the status table each gain the corpus's row;
- the `distill-d1` row is marked superseded, and so is its section, which now names the claude-390 pilot.

## Measurements

None. This is a record. Wick's file counts (`part-0` 81 / 158 MB, `api-a` 457 / 1,012 MB, `inputs` 13 / 205 MB) are from a listing on 16 Nov [claude].

## Checks run at the head

- Fresh-context review of `0e588de96` [claude]: CHANGES, with six findings and one gap, all taken at `98d6e333b`. It confirmed:
  - records, tokens and roots match claude-400;
  - the cost parts sum to $854;
  - the source counts sum to 457,601;
  - claude-403's ratios match;
  - the TRAIN pins match;
  - the anchors resolve;
  - the wick counts match.
- `git diff --check` is clean.
- `loop pr-check --local`: run before queueing.

## Skills used

- `scope-guard`: one model page and this task folder. No store, file or snapshot was touched.

## Still open

1. Casey's ZFS snapshot (`sudo zfs snapshot plexpool/uor-r4-corpus@d87-sealed-2026-11-16`).
2. A third copy before any change to either store.
3. Ingesting `api-a` with `student-ingest`, which turns the 770× projection into a count.

## Plain summary

The donor corpus cost about $854 to make and cannot be made again bit for bit. The model docs now say what it is, why it is worth keeping, where both copies are, and how to check them. They also say it must never be deleted, moved or rewritten.

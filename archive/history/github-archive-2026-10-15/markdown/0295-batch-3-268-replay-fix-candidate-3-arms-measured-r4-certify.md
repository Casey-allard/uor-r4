# #295 — batch-3: #268 replay fix + candidate-3 arms (measured), R4_CERTIFY_ROWS_ONLY fast mode

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-30T04:01:14Z
- **Updated:** 2026-10-09T13:08:41Z
- **Closed:** 2026-07-30T04:12:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/295
- **Merged:** 2026-07-30T04:12:53Z
- **Merge commit:** 4cc195d7b2eedf836da9a92adbd5ca10f74df249
- **Base / Head:** main / UOR-Foundation:batch-3
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Three commits, all evidence posted on #268:

1. **#268 story-contiguous replay fix** — the 13.4-bit "teacher floor" was measuring stream interleaving (~4.6-token mean effective context), not the teacher. Stable-sort by story, reset only at article boundaries. Measured: floor 13.446 → 11.651; band counts match article structure; context gradient clean and monotone.
2. **R4_CERTIFY_ROWS_ONLY fast mode** — promised on the batch-2 record: rows-only certify with recorded skips for the P5 compression witnesses, long-context certification, and the C serving row. A rows-only log is not a certificate and says so.
3. **#268 candidate-3 arms** — `evaluate-report --bos` (BOS at every held-out story boundary; oracle cache sized seq_len+1 in BOS mode — the smoke gate caught the pos-128 panic on exactly-sized att/KV buffers) and `--max-held-out-stories N` smoke cap (reports stamped SMOKE, not quotable). Report schema 2→3 (`source.bos_prefix`). **Measured (4 arms, one run, posted on #268): BOS retired (+0.04 bits both checkpoints); base vs instruct −0.48 bits concentrated in bands 0-15; floor quotable as 11.17 (base) / full-context band ~10.8, checkpoint- and framing-robust.** Store metrics bit-identical across arms — correct invariant.

#268 stays open: the external-referee experiment (same text, same checkpoint, outside the pipeline) is proposed on the issue to decide whether the remaining ~10.8-bit full-context floor is ours or the model's.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


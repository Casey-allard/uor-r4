# #449 — test(compiler): #448 — cover induction determinism regression test (nondeterminism claim REFUTED)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-06T04:12:43Z
- **Updated:** 2026-10-09T13:09:10Z
- **Closed:** 2026-08-06T05:01:47Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/449
- **Merged:** 2026-08-06T05:01:47Z
- **Merge commit:** 4957bbd2a71d0391ad5c558fff2097a46104e44e
- **Base / Head:** main / UOR-Foundation:i448-determinism
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #448 as not-a-bug, with a regression test so the question stays settled.

**Controlled result:** same binary (sha256 `3c11f535…`), same corpus, frozen artifacts passed explicitly, `env -i` scrubbed, two back-to-back score runs → `diff -r` **empty**. Identical region count, identical Gate C table (Rule 1+2 26.4% / 8.2674), identical `score.r4g1` κ `blake3:ec9391ce…` and sha256. The pipeline is deterministic.

**Actual cause of the 10-vs-8 discrepancy:** `--artifacts` defaults to `compiler::ART_PATH` = the shared mutable `/tmp/tless_artifacts.bin`. A concurrent script staged the 10k container into that path and restored it afterwards, so an overlapping score run silently read a different teacher container. Isolated exactly: with the 500k container → 8 regions / Rule 1+2 26.4%; with the 10k container → 10 regions / **29.1% / 7.9892**, reproducing the other reported row to the digit. Split-criterion and capacity env vars were each tested and are NOT the cause (capacity is neutral here since n_train ≈ n_ref).

**Regression test (non-ignored):** `cover_induction_is_deterministic_across_repeats_and_worker_counts` — asserts `build_observations_with_threads` is field-identical to the serial lane at T ∈ {1,2,3,5,8,64} (even, ragged and over-subscribed splits) and that induction gives identical region count and cover κ on repeat and at every worker count. 0.03s.

Prime suspects reviewed and cleared: the `rayon::current_num_threads()` chunking merges integer BTreeMap counts (order-independent); k-means uses `par_iter` only for distance work with the f64 reduction kept in ascending observation order.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


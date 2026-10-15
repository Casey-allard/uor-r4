# #2214 — Archive the unverified fused-memory kernel as a patch and an INDEX row (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T21:47:15Z
- **Updated:** 2026-10-11T21:56:30Z
- **Closed:** 2026-10-11T21:53:02Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2214
- **Merged:** 2026-10-11T21:53:02Z
- **Merge commit:** eaf92bbe1efd81f5d50afe301883069095382323
- **Base / Head:** main / UOR-Foundation:deepseek/kernel-archive
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**The kernel compiles after the two type-shape fixes, and it is not correct.** Measured, not assumed: a pod bootstrapped **at the branch head** fails the stack's own CUDA parity suite with **36 of 37 tests failing**, while a pod at **plain `main`** passes **37 of 37** on the same day, same bootstrap. So the divergence is the kernel's, and the branch is not mergeable.

**What is verified in this patch:** it compiles under `--features cuda` (the `E0308` on the launcher's return shape and the `E0603` on its visibility are fixed). **What is not:** its arithmetic. It is preserved as a **starting point, not a result** — a patch plus an INDEX row in `docs/history/branch-archive/`, with the revival recipe (bootstrap with `--keep-on-failure`, run the parity suite, fix against it, and only then measure).

**Two platform facts this episode established, both worth having on the record:**

1. **The pod bootstrap's parity test is the gate that catches CUDA-only correctness regressions** — it caught this one *after* the compile errors were fixed.
2. **CI does not compile `--features cuda`**, so it passed on both the broken and the fixed states. For every lab touching CUDA code, the bootstrap is currently the only check, and `--keep-on-failure` is what makes it usable for iteration.

Line: fused address kernel · count 0/3 · headline **unchanged** — the retracted claim stays withdrawn, and this round adds a *verified* negative (it compiles, it diverges) rather than a number

References #2029
PR; sleep 240; gh pr checks 2>&1 | grep -v skipping | head -2; gh pr merge --squash --delete-branch=false 2>&1 | tail -1


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


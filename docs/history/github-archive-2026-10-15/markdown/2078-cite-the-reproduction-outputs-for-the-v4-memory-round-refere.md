# #2078 — Cite the reproduction outputs for the v4 memory round (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T19:35:06Z
- **Updated:** 2026-10-09T19:38:23Z
- **Closed:** 2026-10-09T19:36:07Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2078
- **Merged:** 2026-10-09T19:36:07Z
- **Merge commit:** 86134d98035b8dd5abd3c3e2739b3f87e3c3cbb2
- **Base / Head:** main / UOR-Foundation:deepseek/v4-memory-outputs-citation-20261009
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #2029. Docs-only follow-up to #2075/#2076, requested by the lead: the round's strongest claim —
device-independence — was not checkable from `main`, because the record cited the sealed *inputs* by sha256
but nothing this round produced.

Adds to `docs/labs/v4-memory-panel-reproduction-2026-10-09/README.md` a short "where the outputs of this
round are" subsection, and one clause to the current-state entry:

* the object: `icloud:UOR-R4/results/memory-v4/runs.tar`, **329,728 bytes**, md5
  **`b7870d3f0c1c9d55c263c3f8bdfe13cc`**, index row `icloud:UOR-R4/index/runs.tsv` (lab `memory-v4`,
  2026-10-09T19:33:18Z);
* what is inside: `runs/replies-v4-ftq-cpu/` (the CPU-generated `replies.json`, 64 rows, `device=cpu`,
  `model_sha256 9c0d9019…`, with its sealed root files) and `runs/grade-v4-repro1/` (the re-grade report,
  `multi_turn_memory` `check_pass` 31 / `acceptable` 27, including the four rows whose grader verdicts
  differ from the pod's at the same digest);
* why cited rather than committed: it is run output, not source — the storage rule keeps bulk in iCloud
  with `main` holding the index and the identities;
* the fact it supports: all 64 rows byte-identical to the sealed CUDA replies in both reply strings and
  generated ids, so greedy decoding on this artifact is device-independent and the panel reproduces with
  no pod.

Verified before landing, not assumed: `cloud-store fetch runs <dest>` was run and the round trip returns
md5 `b7870d3f0c1c9d55c263c3f8bdfe13cc` with the numbers intact (`check_pass` 31 / `acceptable` 27; 64 rows,
`device=cpu`, `model_sha256 9c0d9019…`).

No code, no numbers changed, nothing in STATUS/ROADMAP/#2028. `python3 scripts/check_claim_wording.py` →
gate passed.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


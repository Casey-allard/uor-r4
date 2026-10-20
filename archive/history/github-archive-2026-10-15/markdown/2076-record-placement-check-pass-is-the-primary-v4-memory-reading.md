# #2076 — Record placement: check_pass is the primary v4 memory reading because the LLM judge is not stable (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T19:21:35Z
- **Updated:** 2026-10-09T19:28:22Z
- **Closed:** 2026-10-09T19:26:16Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2076
- **Merged:** 2026-10-09T19:26:16Z
- **Merge commit:** eff6516d40b66e4fa73f0cc25a552bae16fd492a
- **Base / Head:** main / UOR-Foundation:deepseek/v4-memory-number-stability-20261009
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #2029. Doc-only follow-up to #2075 (`f7ef028b0`), requested by the lead before close.

The merged record already carried the grader-instability caveat, but in the "Reproduced" section and in
"Limitations" rather than where the `31/40` vs `27/40` ambiguity is discussed. This moves it next to that
number so a reader cannot miss why one reading is primary:

* `docs/labs/v4-memory-panel-reproduction-2026-10-09/README.md`, "The name of the number": `check_pass` is
  the primary reading and `acceptable` the secondary **because `check_pass` is deterministic and the LLM
  judge is not** — the re-grade reproduces both exactly, yet 4 of 64 rows carry a different grader verdict
  at the same `qwen2.5:7b` digest; a candidate that moves `acceptable` without moving `check_pass` has to
  show which rows the judge changed its mind on.
* `docs/integration/current-state.md`, the same point folded into the ambiguity sentence itself.

Docs only; no code, no panel file and no measurement changed. No new number is claimed.

Checks: `python3 scripts/check_claim_wording.py` → gate passed.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


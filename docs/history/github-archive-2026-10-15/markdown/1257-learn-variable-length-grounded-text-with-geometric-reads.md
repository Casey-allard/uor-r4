# #1257 — Learn variable-length grounded text with geometric reads

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-13T20:52:14Z
- **Updated:** 2026-10-09T13:17:32Z
- **Closed:** 2026-09-13T20:53:28Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1257
- **Merged:** 2026-09-13T20:53:28Z
- **Merge commit:** 1c511da421f9895a33600346d99dd0248c3666e2
- **Base / Head:** main / UOR-Foundation:codex/geometric-text-stream
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Result

The previous typed attention loop emitted one byte and EOS. This adds a bounded text path that repeatedly selects a supplied span, learns byte/EOS output and applies a learned cursor-advance decision after an actual emitted symbol. A single frozen 256-epoch fit passes `PASS_GROUNDED_VARIABLE_TEXT`: 64/64 development responses including EOS, 32/32 changed-source pairs, and 48/48 answers longer than any training answer. All five disabled/reversed controls produce 0/64 exact responses.

The warm geometric reader is trainable but its exported tables stayed unchanged. Eight writer tables and one advance entry changed. Query encoding, span bindings and boundaries remain supplied; the cursor policy uses presence/non-EOS flags. Alternating teacher-position output objectives are not end-to-end recurrent gradients. The historical adaptive mode remains separate from this text loop. Grounded copying is established at the declared scope; novel prose, combined multi-hop text and final qualification remain unfinished. Retain `15baec48`, with no model promotion.

## Validation and preservation

- Release `cargo test -p uor-r4-core --lib --offline --release text_attention -- --nocapture`: 4 focused tests passed; the explicit fit/report test ran separately and passed.
- Frozen executable: 9 relational, 5 dependent and 3 adaptive focused tests passed. All 576 new Full trajectories reproduce after artifact reload.
- New text loop preserves 8,256 prior one-read outputs; separate historical adaptive mode preserves 2,240 mixed-depth examples.
- `cargo fmt --all --check`, `python3 scripts/check_claim_wording.py`, and `git diff --check` passed. CI compatibility acknowledgements are not counted as executed tests.
- All 36 prior sealed roots/1,070 files, bound prior source/executable, original dirty checkouts and retained normal model verify unchanged. New report exclusively claimed, sealed and verified.

Model/build/check charge: 130,613/1,800,000 ms; shared ledger 124,987,758/132,950,000 ms. No allowance extension, paid compute, cleanup or V3–V7 replay. Complete resource and delivery receipts stay in the established local handoff. Result, exact evidence, current state and navigation are updated; the next responsibility is a single recurrent path coupling dependent source selection to multi-span text output.

References #973. References #964.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


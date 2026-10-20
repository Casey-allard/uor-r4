# #2184 — Arm S stopped by owner direction: record, A1536 control, SoftSort CUDA speed fix archived (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T00:28:48Z
- **Updated:** 2026-10-11T00:30:59Z
- **Closed:** 2026-10-11T00:29:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2184
- **Merged:** 2026-10-11T00:29:53Z
- **Merge commit:** 7bd2a21ad2596fa142cc5f71127cfc291bc1f162
- **Base / Head:** main / UOR-Foundation:claude/softsort-s-record
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Line "wide learned flock read" · paused by owner stop (count unchanged) · headline before → after: softmax at runtime yes, served 0.886838 BPB → unchanged.**

Closes the arm S cycle under the delivery cadence, after the owner's direction of 11 October recorded on #2032. A SoftSort/rank substitute for softmax that still scores by L2 distance is not a geometric read, and runs must answer stated questions. **No KEEP or REJECT:** this is an owner stop of the configuration, not a measured negative (D22 §1).

## Contents
- **`docs/labs/softsort-s-2026-10-11/README.md`:**
  - what ran: the A1536 softmax control finished at float NLL 1.4916751 (ctx 1536); the retune reached step 1,000 before the stop;
  - three tooling findings: the serial host selection fixed in #2182, candle's slow CUDA `index_add` backward, and candle's broken `ScatterAdd` backward;
  - cost, about $8.50;
  - the lesson that shapes the next design.
- **`docs/history/branch-archive/claude_softsort-gather-20261011.patch` + INDEX row:** the ×3.3 CUDA fix for the SoftSort read, measured at 5,315 → 17,415 tok/s. It is paused, not negative, and `git apply --check` passes on current `main`.
- **`docs/integration/current-state.md`:** a newest-first entry with Next.
- **`STATUS.md` / `ROADMAP.md`:** the arm S state, plus a new ring-product read row (design, owner review).

Would the next step differ if this came out the other way? It records an owner stop, not an experiment. Without it, `main` would still say arm S runs next (STATUS, ROADMAP, current-state) and the speed-fix source would exist only on a branch. The preservation and cadence rules require both.

References #2032

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6103761934 — 2026-10-11T00:29:00Z — Casey-allard

**Exact-head checks at `ab0a7c6e0039187faf113fbf1814cd60d02b3d4c`** (docs and patch only; no Rust changed):
- `git apply --check docs/history/branch-archive/claude_softsort-gather-20261011.patch` on this tree: exit 0. The archived patch applies.
- `git diff --check origin/main..HEAD`: exit 0.
- `python3 scripts/check_claim_wording.py`: exit 0, gate passed.
- The relative links in the new record resolve.

**Exact-head self-review (Claude lab):**
- Every number in the record matches this session's logs: A1536 final 1.4916751 over 786,432 targets; the retune dev NLL at step 1,000 of 2.726 and 2.956; 5,315 and 17,415 tok/s; pod times 21:48Z–00:10Z at $3.57/h.
- The patch is exactly `git diff d97fe3989 1b1d11f50 -- crates`.
- STATUS and ROADMAP change only the M4 read rows.
- The outcome is labelled an owner stop, not a negative.
- APPROVE.


---
_Comment/review ids in this file: 6103761934_


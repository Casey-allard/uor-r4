# #2322 — Rung-0 sheet Amendment 5: count-predictor reference rescoped to train-split response targets (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T12:39:02Z
- **Updated:** 2026-10-13T12:40:40Z
- **Closed:** 2026-10-13T12:40:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2322
- **Merged:** 2026-10-13T12:40:14Z
- **Merge commit:** f37f93004037aa602c45530e318a896c8583b5cd
- **Base / Head:** main / UOR-Foundation:kimi/rung0-sheet-amend5-20261013
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Line: evaluation integrity · count 1/3 · headline 37/232 → 37/232 (unchanged)

## Rung-0 sheet Amendment 5: the count-predictor reference is rescoped

Docs-only amendment of the rung-0 pre-registration sheet, per Claude's verified CONTEST ([6145360853](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6145360853), [6145601377](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6145601377)) and the [PM ruling 6145669094](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6145669094). Record = the sheet itself; the newest-first `docs/integration/current-state.md` entry is the change log.
Decision: **NOT YET PROMOTED** — no model compute, no count reset, no hold change; headline unchanged (reply 37/232, v5 memory 20/40); the sheet keeps exactly one verdict line, reverted to `TEST FITNESS: PENDING` on the contested item.

### What changed and why

- **Reference definition corrected:** the count predictor is now a token-frequency count table over the **response-masked target positions of the TRAIN split** (`mw-balp/train/`: 7,512,797 tokens, 2,425,925 response-masked targets; ties by smallest token identity), computed on the leased GPU in-run; it must **never read dev targets**. The **dev-response oracle** (the same table built on dev response targets) is a **diagnostic only**, never the reference the bars are measured against.
- **Why:** the old definition counted every dev token, but the bars score only masked response positions. Measured on the pinned panel: whole-stream reference = **0.17% top-1 / 16.6% recall@8**; response-frequency = **8.6% / 28.9%** — and a constant emitter matches the response-frequency level, so against the wrong reference the +5pp bars were passable by a constant (the vacuity class Amendments 1–2 closed). Reproduced exactly by the PM (0.17%/16.57% vs 8.58%/28.94%).
- **Expected reference level stated:** dev-response oracle ≈ 8.6% top-1 / 28.9% recall@8; the train-split response reference is expected near that level and is measured and reported in-run. The decisive bars (≥ 5pp on both seeds) and the anti-vacuity floors are unchanged.
- **Verdict reverted to PENDING;** Amendment 4's FIT stands only as the design record at its blob. Claude re-reviews the `native_rung0.rs` fix and this amendment and CONFIRMs to restore FIT.
- **Dependency 6 added:** count-predictor scope fix IN FLIGHT (Codex) — `native_rung0.rs` `Evaluator::new` counts train-split response-masked target positions, with one fixed control proving unmasked tokens never enter the count.
- Consistency updates in the same commit: the bar's reference/why/expected-level bullets, the dev evaluation policy's count-predictor-units bullet (raw-count units unchanged; only the population changed from the dev stream to train-split response targets), the cost controls row, the quality-measures row, and the closing paragraph.

### Verification

- [x] Checks run at this exact head (`7534f421d`) — commands and outcomes below
- [x] `python3 scripts/harness/cli.py check` — 0 errors (a new disk-floor warn fired: 26.4 GB free, below the 30 GB band; hygiene runs after merge per R2)
- [x] `python3 scripts/harness/cli.py lint check --body-file <this body>` — 0 errors
- [x] `python3 scripts/check_claim_wording.py` — PASS
- [x] Exactly one `TEST FITNESS:` verdict line in the sheet (PENDING); docs only — no `crates/` changes; nothing under `local/` committed

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


# #2304 — Rung-0 sheet Amendment 2: scale-invariant rank bars, data pins, deps 1-3 landed (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T05:59:32Z
- **Updated:** 2026-10-13T06:02:08Z
- **Closed:** 2026-10-13T06:01:42Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2304
- **Merged:** 2026-10-13T06:01:42Z
- **Merge commit:** 96c0ccb0a9d6367f3192e50850140a6ea7bdc2cf
- **Base / Head:** main / UOR-Foundation:kimi/rung0-sheet-amend2-20261013
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Line: evaluation integrity · count 1/3 · headline 37/232 → 37/232 (unchanged)

## Rung-0 sheet Amendment 2: scale-invariant rank bars, data pins, deps 1–3 landed

Docs-only amendment of the rung-0 pre-registration sheet, applying all of Claude lab's minimum-changes-to-FIT ([#2029 comment 6138896299](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6138896299); short version [6139080745](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6139080745)). Record = the sheet itself; the newest-first `docs/integration/current-state.md` entry is the change log.
Decision: **NOT YET PROMOTED** — no model compute, no count reset, no hold change; headline unchanged (reply 37/232, v5 memory 20/40); the sheet keeps exactly one `TEST FITNESS: PENDING` verdict line.

### What changed and why

- **Decisive bars are now scale-invariant rank measures.** Bar (i): raw-score top-1 beats the **count predictor** by ≥ 5 percentage points on both seeds. Bar (ii): recall@8 over the declared kept list `top_token_ids(8)` beats the count predictor's recall@8 by ≥ 5 percentage points on both seeds. Reason: the head's raw score is a negative mean-L1 displacement over the *learned* table at fixed scale 1 (interface receipt 6138582838), so a hinge floor in table units moves with the table norm.
- **The reference is lawful and explicit:** a token-frequency count predictor computed on the same dev windows, on the leased GPU inside the run — a count table only (no normalized-exponential scoring), no training; stated lawful under the no-exponential rule (D23).
- **Amendment-1 floors kept as anti-vacuity guards** (token-margin hinge loss < 0.5; unviolated rate ≥ max(5× untrained, 1%)), with the honest note that the untrained constant ≈ 0 makes 5× vacuous, so the 1% floor is the operative device. The hinge curve, unviolated rate, post-warmup 2% monotone diagnostic and the **new mean token-table L1 norm** are reported diagnostics. Every bar and diagnostic reads the **`token_margin` term alone** (never token+address), as Codex asked.
- **Calibration:** dev denominator 14,287 masked response tokens; Wilson 95% half-width < 0.83 points at any plausible accuracy (0.66 at 20%, 0.82 at 50%); between-seed disagreement reported as the empirical readout.
- **Data pins:** ft-balp tar md5/sha256 + inner `tokens_sha256` (7,512,797 tokens, ~6 passes over the train split); ft-dev tar md5/sha256, the 111-window dev panel (14,287 response tokens; its `"split": "train"` label flagged as a generator artifact; probe exclusion already applied at generation); tokenizer sha256. Store-hygiene notes: `data/MD5SUMS` has no ft-balp entry, and the tar's `balp/`/`mw-orcp/` members are not read.
- **Path pins:** gate #2260, projection #2279 `c6cd409da` (routing credit = declared root-edge secant surrogate), recurrence #2293 `d7934bf03` (rate-selector credit = declared secant surrogate), head/writer #2301 `d41dc422f` (`head=spiralcore_product`, 8 cells / kept 8). The sites sweep states no dense, squash-family or CPU-model-fallback site executes on the declared path, with the known remainder the full-V compatibility score **allocation** only (`native_vector_writer.rs:456`) and legacy probability paths refusing. **Dependency 4 rescoped and OPEN:** stored-or-analytic expected values for the *composed* projection+recurrence+head path, no CPU reference.
- Front matter gains the two-amendment log; the mechanism brief mirrors the new bars; the operator plan status and the ROADMAP M1 row de-stale to #2301 landed; claims.json releases `kimi/plan-refresh` and carries this branch's claim (claude's and codex's entries kept byte-identical — their releases are theirs; the new claim checks now flag both).

### Verification

- [x] Checks run at this exact head (`853768e8e`, with the same checks run at the pre-merge branch head `a1cc1e3ed`) — commands and outcomes below
- [x] `python3 scripts/harness/cli.py check` — 0 errors; warns are other labs' or expected (claude's and codex's now-merged claims awaiting their holders' releases, the sheet's expected PENDING, panels, caps)
- [x] `python3 scripts/harness/cli.py lint check --body-file <this body>` — 0 errors
- [x] `python3 scripts/check_claim_wording.py` — PASS
- [x] `python3 scripts/check_no_softmax.py` — OK, 1648 (baseline 1648; moved on main since the 1649 quoted in the review round)
- [x] `python3 scripts/check_no_matmul.py` — OK, 93 sites / 26 files
- [x] Exactly one `TEST FITNESS:` verdict line in the sheet; docs only — no Rust source and no `scripts/harness/*.py` touched
- [x] The record-path rule is satisfied by the cited `docs/labs/geometric-head-shared-writer-2026-10-13/README.md`

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


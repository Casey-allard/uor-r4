# #2306 — Rung-0 sheet Amendment 3: the dev evaluation policy is pinned (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T06:14:54Z
- **Updated:** 2026-10-13T06:17:15Z
- **Closed:** 2026-10-13T06:16:54Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2306
- **Merged:** 2026-10-13T06:16:54Z
- **Merge commit:** ffe9c94030e120ee326fa6f76788d85e6d10b958
- **Base / Head:** main / UOR-Foundation:kimi/rung0-sheet-amend3-20261013
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Line: evaluation integrity · count 1/3 · headline 37/232 → 37/232 (unchanged)

## Rung-0 sheet Amendment 3: the dev evaluation policy is pinned

Docs-only amendment of the rung-0 pre-registration sheet per the [PM ratification](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6139449267), pinning the exact dev policy Codex's CLI/metrics agent implements against. Record = the sheet itself; the newest-first `docs/integration/current-state.md` entry is the change log.
Decision: **NOT YET PROMOTED** — no model compute, no count reset, no hold change; headline unchanged (reply 37/232, v5 memory 20/40); the sheet keeps exactly one `TEST FITNESS: PENDING` verdict line.

### What changed and why

- **Authoritative dev source:** the dev token **file** is authoritative — 42,888 u16 in `data/ft-dev.tar` (md5 `0a1d0a05020a5d80b5d008fd5b731134`, sha256 `45e93e32918f8ad5c977ad3a98926b561173049aacd8bfce2b6e2e49a0390b5c`). The manifest's `tokens: 42,856` differs by 32; the run **reports both numbers and flags the mismatch** — never silently picks one.
- **Scored set:** every `response_mask[p]=1` position with an in-document predecessor; context **resets at each `<|bos|>`** boundary; no scored position reads across a boundary.
- **Coverage:** all 42,888 positions — **111 non-overlapping 384-token windows plus one tail window** (264 positions; its context may overlap the previous window but it scores only its tail). Windows are a **batching device, not an exclusion rule** (this supersedes Amendment 2's "exactly 111 windows" phrasing).
- **Denominator:** the run **computes and reports** the scored count; the pre-registered expectation is the mask sum over scored positions, cited at n ≈ 14.3k (manifest `response_tokens` 14,287; Wilson 95% half-width < 0.83 points); any mismatch with 14,287 is reported, not absorbed.
- **Count-predictor units:** ordering-only for the decisive bars (`score(t)` = occurrence count, ties by smallest identity; top-1 = argmax; recall@8 = membership in the 8 highest-count identities); its hinge/unviolated diagnostics use raw counts and are **never numerically compared** with the model's mean-L1-units hinge.
- **Config pin completed:** `head=spiralcore_product projection=spiralcore_fbs rec_gates=geometric gate=hamilton`; F32; CUDA-only; no sampling.
- **Dependency 5 registered (pre-execution):** Codex's complete-path repair (claim `codex/rung0-complete-path-20261013`; ordinary-recurrence geometric dispatch, CLI `rec_gates`/native-eval wiring, head row-cap 4096→6144, `RateSelect` backward secant sign, interior rate-bin init with `Historical` untouched) **must merge before execution**; its commits will be cited in the FIT re-request, and the PR must record the exact interior bin scores for the sheet's initialization pin.
- Front-matter amendment log gains Amendment 3; claims.json releases `kimi/rung0-sheet-amend2` and carries this branch's claim (claude's and codex's entries kept byte-identical).

### Verification

- [x] Checks run at this exact head (`df70cce19`, with the same checks run at the pre-merge branch head `1e1abaa0c`) — commands and outcomes below
- [x] `python3 scripts/harness/cli.py check` — 0 errors; warns are peers' claims awaiting their holders' releases (claude, codex) plus the standing notices
- [x] `python3 scripts/harness/cli.py lint check --body-file <this body>` — 0 errors
- [x] `python3 scripts/check_claim_wording.py` — PASS
- [x] Exactly one `TEST FITNESS:` verdict line in the sheet; docs only — no Rust source and no `scripts/harness/*.py` touched
- [x] The record-path rule is satisfied by the cited `docs/labs/geometric-head-shared-writer-2026-10-13/README.md`

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


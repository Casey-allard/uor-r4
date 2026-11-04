# REPORT: record the conversation-first Studio publish

Lab: kimi. Item: [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md) (#3440). Order: [ORDER.md](ORDER.md). Base: `a877ef9162d86908873c5edca0f1d0711bb6321a`. This is a docs-only live-record PR, not a new implementation or milestone claim.

## What did not work

- The first record draft said “hosted desktop/mobile generation” in four places although only desktop generation was measured; all now distinguish hosted desktop generation from mobile layout inspection.
- `docs/integration/current-state.md` initially lacked the publish entry; it now begins with the dated 4 November record and exact identifiers.
- The local PR checker requires a new ORDER/REPORT pair for this record unit; this folder supplies them rather than bypassing the gate.
- No useful conversation, visual understanding, M7/M8 completion, production readiness or Pages-hosted API is claimed.

## What changed

Updated `current-state.md`, the product page, serving notebook, roadmap item, Studio README and kimi-5 REPORT with source merge `a877ef916`, reviewed tree `3c4c6470`, `gh-pages` `eb832431a` / tree `2d5fe037`, the Pages built status, hosted WASM `df368909…`, unchanged `models.json` `96e296b7…12554a`, development-file 404s, native prerelease and claim limits.

## Measurements

- Source PR #3651 merged with the independently approved tree; fresh detached `main` passed Studio, Node, runner and worker checks.
- `gh-pages` advanced by fast-forward `233b97fb9 → eb832431a0db59661c701d5b37168e9cfa3d3ae4`; the Pages API reports **built** with `updated_at` `2026-11-04T07:26:52Z` and no build ID is invented.
- Hosted desktop generation completed 32 tokens. Mobile layout at 390×852, Native setup and all five cards were inspected. Hosted index matches the candidate; development files return 404.
- Public runner checksums and an empty-cache `/healthz` launch passed. No token appears in logs.

## Checks run at the head

Report checks, the local ORDER/REPORT/PR checks and `git diff --check` pass. Independent exact-head record review returned CHANGES for the mobile wording and missing current-state entry, then APPROVE after both were corrected at `82dcafefe`. This record PR's protected merge and fresh-main verification remain pending at this final record head.

## Skills used

Live GitHub/Pages identity verification, hosted desktop/mobile browser inspection, exact claim wording, current-state reconciliation and independent exact-head review. No GPU, paid compute, new dependency or code/model change was used.

## Still open

Delete merged branches/worktrees and temporary build outputs while preserving evidence and at least 30 GiB free. The independent source review's non-blocking UI residuals remain outside this record PR.

## Plain summary

The live records now match the actual conversation-first Studio publish and its evidence, without mobile-generation, useful-chat, visual-understanding or milestone overclaims.

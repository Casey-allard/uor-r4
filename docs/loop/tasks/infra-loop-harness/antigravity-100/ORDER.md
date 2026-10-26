# ORDER: infra-loop-harness, antigravity-100

Lab: antigravity. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file maintenance, as in claude-90 and antigravity-98). Base: `87024eca8`.

## Goal

Unblock Step 8 of `PLAN.md` (`m1-close-saved-model`, `#2837`) and prevent `uor-harness loop next` from re-dispatching Step 7 (`m1-fresh-panel-transfer`, `#2841`) before a saved discrete model passes M1 bars (a)–(d) on `ft/dev` across two seeds:

1. **`docs/roadmap/items/m1-fresh-panel-transfer.md` (`#2841`, Step 7 of `PLAN.md`):**
   - Set `status: held` under a fresh-context D68 decision review (`DECISION.md` and `#2841` owner log) and update `## What is missing` to record that Steps 1–3 and 5 (`d11-scorer fresh-score` gate, `refuse_sealed_fresh_source`, design note `#issuecomment-6423580146`, stream seal verification of `0b7e754c…` / `a1cda529…` / `df5d074a…` / `c2d4c7e5…` and `4bd108ad…` / `82ead8db…` / `8287d20d…`, pre-registration sheet `docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md` with non-author `TEST FITNESS: FIT` `#issuecomment-6423732020`, lab note `docs/labs/m1-fresh-panel-transfer/README.md`, and notebook row) landed in `#3292` (`antigravity-99`), and that Step 4 is held (`UNAVAILABLE_NOT_RUN`) until `m1-close-saved-model` (`#2837`) produces a saved discrete model that passes M1 bars (a)–(d) on `ft/dev` across two seeds.
2. **`docs/roadmap/items/m1-close-saved-model.md` (`#2837`, Step 8 of `PLAN.md`):**
   - Update `depends_on: [m1-a8-full-dose-rerun, m1-bvg-k-stage-t-training, m1-export-gap-and-toks]` (all 3 `status: done` on `origin/main`), breaking the circular dependency between Step 8 (`#2837`) and Step 7 (`#2841`).
   - Expand `scope:` to `[crates/uor-r4-discrete/**, docs/labs/m1-d11-artifact-scorer/instrument/**, docs/harness/prep/m1-close-saved-model*, docs/labs/m1-close-saved-model/**, docs/loop/tasks/m1-close-saved-model/**, docs/research/notebook/m1-discrete-attention.md, docs/roadmap/items/m1-close-saved-model.md, docs/roadmap/items/m1-fresh-panel-transfer.md]` so `m1-close-saved-model` can score through `d11-scorer`, write its notebook row in `docs/research/notebook/m1-discrete-attention.md`, and update its roadmap status.

## Cancel probe (run first)

1. Verify why `uor-harness loop claim infra-loop-harness` refuses and check file overlap with open PRs (`#3289`, `#3169`), and verify why `uor-harness loop next` currently offers `m1-fresh-panel-transfer` instead of `m1-close-saved-model`.
2. **Cancel probe result (executed 2026-10-26, classed `instrument`):**
   - `uor-harness loop claim infra-loop-harness` refuses because idle draft PR `#3169` (`codex-1`) touches `infra-loop-harness`; per the umbrella-item rule (`run-antigravity.txt` Part 3.2), we verified zero file overlap with `#3169` and `#3289`.
   - `uor-harness loop next` currently offers `GO item m1-fresh-panel-transfer` because `#3292` left `m1-fresh-panel-transfer.md` at `status: ready` while `m1-close-saved-model.md` lists `m1-fresh-panel-transfer` in `depends_on` and omits `docs/research/notebook/m1-discrete-attention.md` from `scope:`.

## Files in scope

- `docs/roadmap/items/m1-fresh-panel-transfer.md`
- `docs/roadmap/items/m1-close-saved-model.md`
- `docs/loop/tasks/infra-loop-harness/antigravity-100/**`

## Done when

1. `docs/roadmap/items/m1-fresh-panel-transfer.md` is set to `status: held` with its `#3292` preparation recorded, and `docs/roadmap/items/m1-close-saved-model.md` has `depends_on: [m1-a8-full-dose-rerun, m1-bvg-k-stage-t-training, m1-export-gap-and-toks]` and `docs/research/notebook/m1-discrete-attention.md` + `docs/labs/m1-d11-artifact-scorer/instrument/**` in `scope:`.
2. Fresh-context D68 decision review (`DECISION.md`) passes and is logged as `[antigravity] owner log:` on `#2904`, `#2841`, and `#2837`.
3. `xtask roadmap check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` pass.

## Stop and ask if

- Never stop to wait on Casey; log the D68 decision review on `#2904`, `#2841`, and `#2837`.

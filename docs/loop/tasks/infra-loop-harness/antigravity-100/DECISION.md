# D68 Decision Card: Hold `m1-fresh-panel-transfer` Step 4 on Two-Seed DEV Pass and Unblock `m1-close-saved-model` (`antigravity-100`)

- **Date:** 2026-10-26
- **Lab:** `antigravity` (`antigravity-100`, PR `#3293`)
- **Items:** [`m1-fresh-panel-transfer`](../../../../roadmap/items/m1-fresh-panel-transfer.md) (`#2841`, Step 7 of `PLAN.md`) and [`m1-close-saved-model`](../../../../roadmap/items/m1-close-saved-model.md) (`#2837`, Step 8 of `PLAN.md`)
- **Authority:** `CHARTER.md` §5.1, §5.2, D68 (status/dependency/scope update approved by the lab's fresh-context decision review and logged for the owner; zero changes to §3 or GPU caps)

## 1. Whole-Project Context (`PLAN.md` Steps 4–8 -> Milestone M1 `#2029`)

1. In `#3292` (`antigravity-99`), Steps 1–3 and 5 of `m1-fresh-panel-transfer` (`#2841`) landed on `main`:
   - `inst::fresh_transfer` (`refuse_sealed_fresh_source`, `validate_fresh_transfer_gate`, `fresh_rows`, `fresh_rows_from_corpus`, `build_train_count_tables`, `score_fresh_against_count_tables`, `evaluate_fresh_transfer_two_seeds`, and `d11-scorer fresh-score`, `18 / 18` tests),
   - the Step 2 design note on `#2841` (`#issuecomment-6423580146`),
   - stream SHA-256 verification of the sealed archives (`0b7e754c…` / `a1cda529…` / `df5d074a…` / `c2d4c7e5…` and `4bd108ad…` / `82ead8db…` / `8287d20d…`),
   - the pre-registration sheet `docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md` with non-author `TEST FITNESS: FIT` and D68 review (`#issuecomment-6423732020`),
   - `docs/labs/m1-fresh-panel-transfer/README.md` and the notebook row in `docs/research/notebook/m1-discrete-attention.md`.
2. Step 4 of `m1-fresh-panel-transfer.md` (`:33`) states: *"Do not run it until `m1-export-gap-and-toks` has produced a saved model that passes the earlier bars on two seeds, and a fresh-context pre-run review has passed after its FIT (CHARTER §5.1, D67, D77). Score once."*
3. Because `#3292` left `m1-fresh-panel-transfer.md` at `status: ready` while `m1-close-saved-model.md` (`#2837`, Step 8 of `PLAN.md`) listed `m1-fresh-panel-transfer` in `depends_on`, `uor-harness loop next` repeatedly offered `GO item m1-fresh-panel-transfer` even though Step 4 cannot open the sealed fresh panel until `m1-close-saved-model` produces a saved discrete model that passes M1 bars (a)–(d) on `ft/dev` across two seeds.
4. In addition, `m1-close-saved-model.md` declared `notebook: docs/research/notebook/m1-discrete-attention.md` but omitted `docs/research/notebook/m1-discrete-attention.md` and `docs/labs/m1-d11-artifact-scorer/instrument/**` from `scope:`.

## 2. Decision

1. Set `docs/roadmap/items/m1-fresh-panel-transfer.md` (`#2841`) to `status: held` and record in `## What is missing` that Steps 1–3 and 5 are complete (`#3292`) and Step 4 (`d11-scorer fresh-score` on `fresh_v2`) is held (`UNAVAILABLE_NOT_RUN`) until `m1-close-saved-model` (`#2837`) produces a saved discrete model that passes M1 bars (a)–(d) on `ft/dev` across two seeds (matching `claude-114`'s precedent on `#2839`).
2. Update `docs/roadmap/items/m1-close-saved-model.md` (`#2837`, Step 8 of `PLAN.md`):
   - `depends_on: [m1-a8-full-dose-rerun, m1-bvg-k-stage-t-training, m1-export-gap-and-toks]` (all 3 `status: done` on `origin/main`), breaking the circular dependency between Step 8 (`#2837`) and Step 7 (`#2841`).
   - `scope: [crates/uor-r4-discrete/**, docs/labs/m1-d11-artifact-scorer/instrument/**, docs/harness/prep/m1-close-saved-model*, docs/labs/m1-close-saved-model/**, docs/loop/tasks/m1-close-saved-model/**, docs/research/notebook/m1-discrete-attention.md, docs/roadmap/items/m1-close-saved-model.md, docs/roadmap/items/m1-fresh-panel-transfer.md]`.
   - Record the current `main` baseline in `## What is missing`.

## 3. Reversal

Revert PR `#3293` or restore `status: ready` on `m1-fresh-panel-transfer.md` once a two-seed DEV bar-(a)–(d)-passing model is saved and exported to `UORLUT01`.

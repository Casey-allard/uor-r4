# ORDER: infra-loop-harness, antigravity-109

Lab: antigravity. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file maintenance, as in antigravity-100 and antigravity-107). Base: `883efd902`.

## Goal

Unblock `uor-harness loop next` and `uor-harness loop claim` on Step 7 (`m1-fresh-panel-transfer`, `#2841`) and Step 8 (`m1-close-saved-model`, `#2837`) of `PLAN.md` now that `antigravity-106` (`#3304`), `antigravity-107` (`#3308`), and `antigravity-108` (`#3310`) have landed the 19-table `uor-r4.lut-discrete-trunk/1` (`trunk-joint-pointer-v1-d11`) clean-room `IntegerDiscreteModel` reader/evaluator and `d11_score_rows` / `DeviceScorer` kernel on `origin/main`:

1. **`docs/roadmap/items/m1-close-saved-model.md` (`#2837`, Step 8 of `PLAN.md`):**
   - Drop the temporary one-unit `crates/uor-r4-integer/**` path from `scope:` (added in `antigravity-107` `#3308` and now complete on `origin/main`) so `m1-close-saved-model` no longer overlaps `d11-exporter-float-trunk` (`PR #3306`).
2. **`docs/roadmap/items/m1-fresh-panel-transfer.md` (`#2841`, Step 7 of `PLAN.md`):**
   - Under a fresh-context D68 decision review (`DECISION.md` and owner log on `#2904`, `#2841`, and `#2837`), release `status: held` -> `status: ready` now that its explicit hold condition (`m1-close-saved-model` producing a saved discrete model that passes Bars (a)–(d) on `ft/dev` across two seeds, with 19-table `uor-r4-integer` and `d11-scorer` kernel support merged on `main`) is satisfied.

## Cancel probe (run first)

1. `uor-harness loop claim infra-loop-harness --lab-topic unhold-m1-fresh` refused on idle draft `PR #3309` (`claude-138`). Checked `gh pr view 3309 --json files` and `gh pr view 3306 --json files`: `0` overlapping files (`#3309` touches `labs.rs`, `loopguard.rs`, `onboard.rs`, `receipt.rs`, `infra-loop-harness.md`). Hand-written `ORDER.md` per PART 3 Step 2 umbrella exception.
2. Confirmed (`instrument` classification on `883efd902`): `uor-harness loop next` outputs `WAIT` because `m1-close-saved-model.md` still lists `crates/uor-r4-integer/**` (overlapping `PR #3306`) and `m1-fresh-panel-transfer.md` is still `status: held`.

## Files in scope

- `docs/roadmap/items/m1-close-saved-model.md`
- `docs/roadmap/items/m1-fresh-panel-transfer.md`
- `docs/loop/tasks/infra-loop-harness/antigravity-109/**`

## Done when

1. `docs/roadmap/items/m1-close-saved-model.md` drops `crates/uor-r4-integer/**` from `scope:`, and `docs/roadmap/items/m1-fresh-panel-transfer.md` is updated to `status: ready`.
2. Fresh-context D68 decision review (`DECISION.md`) passes and is logged as `[antigravity] owner log:` on `#2904`, `#2841`, and `#2837`.
3. `xtask roadmap check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` pass.

## Stop and ask if

- Never stop to wait on Casey; log the D68 decision review on `#2904`, `#2841`, and `#2837`.

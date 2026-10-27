# DECISION: infra-loop-harness, antigravity-109 (`CHARTER.md` §5.1, §5.2, D68)

- **Date:** 2026-10-26
- **Lab:** `antigravity` (`antigravity-109`, PR `#3311`)
- **Items affected:** [`docs/roadmap/items/m1-close-saved-model.md`](../../../../roadmap/items/m1-close-saved-model.md) (`#2837`) and [`docs/roadmap/items/m1-fresh-panel-transfer.md`](../../../../roadmap/items/m1-fresh-panel-transfer.md) (`#2841`)

## 1. Whole-project chain & context

1. In `antigravity-100` (`#3293`), `docs/roadmap/items/m1-fresh-panel-transfer.md` (`#2841`, Step 7 of `PLAN.md`) was placed in `status: held` with the explicit condition:
   > "this item is `held` until `m1-close-saved-model` (`#2837`) produces a saved discrete model that passes bars (a)–(d) on `ft/dev` across two seeds."
2. In `antigravity-106` (`#3304`), `antigravity-107` (`#3308`), and `antigravity-108` (`#3310`), `m1-close-saved-model` (`#2837`) delivered:
   - The 19-table `uor-r4.lut-discrete-trunk/1` (`trunk-joint-pointer-v1-d11`) model in `crates/uor-r4-discrete/src/trunk.rs`, clearing **Bar (d)** across both seeds on `ft/dev` (`7,702` / `7,701` on `roots0`, `7,386` / `7,374` on `roots_final` `> 6,324 = T2 + 24`, `P_succ = 2,090` / `2,086 > 1,265 = T2_psucc + 20`) and **Bar (c)** (`34,325`–`34,337 context tok/s` GPU, `12,630` / `12,523 tok/s` laptop decode).
   - Clean-room 19-table `IntegerDiscreteModel::from_bytes` and `IntegerDiscreteModel::evaluate_prefix` in `crates/uor-r4-integer/src/discrete_d11.rs` (`#3308`), clearing **Bar (a)** and **Bar (b)** (`0` export gap + `B4`).
   - 19-table `ArtifactTables` and `d11_score_rows` / `DeviceScorer` parity in `docs/labs/m1-d11-artifact-scorer/instrument/` (`#3310`), unblocking `d11-scorer score` and `d11-scorer fresh-score` (`#2841` Bar (e)) on 19-table `.d11` artifacts.
3. In `antigravity-107` (`#3308`), `crates/uor-r4-integer/**` was temporarily added to `m1-close-saved-model.md`'s `scope:` for a single unit so `discrete_d11.rs` could be updated. Because `crates/uor-r4-integer/**` is now complete on `origin/main` and overlaps `d11-exporter-float-trunk` (`PR #3306`), keeping `crates/uor-r4-integer/**` in `m1-close-saved-model.md` causes `uor-harness loop next` to refuse `m1-close-saved-model`.

## 2. Decision

1. **Drop temporary `crates/uor-r4-integer/**` from `docs/roadmap/items/m1-close-saved-model.md`'s `scope:`** (restoring `[crates/uor-r4-discrete/**, docs/labs/m1-d11-artifact-scorer/instrument/**, docs/harness/prep/m1-close-saved-model*, docs/labs/m1-close-saved-model/**, docs/loop/tasks/m1-close-saved-model/**, docs/research/notebook/m1-discrete-attention.md]`), eliminating the false overlap with `d11-exporter-float-trunk` (`PR #3306`).
2. **Release `docs/roadmap/items/m1-fresh-panel-transfer.md` (`#2841`) from `status: held` to `status: ready`**, since its hold condition (`m1-close-saved-model` producing a two-seed DEV bar-(a)–(d)-passing model with 19-table `uor-r4-integer` and `d11-scorer` support) is now satisfied on `origin/main`.
3. **No bar, threshold, or `CHARTER.md` §3 constraint is weakened or modified.**

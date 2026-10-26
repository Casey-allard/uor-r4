# D68 Decision Card: Narrow `m2-pin-train-mask-sheet` (`#2851`) Scope to Docs Paths (`antigravity-102`, #3297)

- **Lab / Session:** `antigravity` (`antigravity-102`, PR `#3297`)
- **Umbrella item:** [`docs/roadmap/items/infra-loop-harness.md`](../../../../roadmap/items/infra-loop-harness.md) (`#2904`, Setup row 2)
- **Target item:** [`docs/roadmap/items/m2-pin-train-mask-sheet.md`](../../../../roadmap/items/m2-pin-train-mask-sheet.md) (`#2851`, M2)
- **Authority:** [`CHARTER.md §5.1`](../../../../../CHARTER.md), D68 (`docs/integration/DECISIONS.md`)

## 1. Whole-project context (`CHARTER.md §5.1`)

1. **`m2-pin-train-mask-sheet` (`#2851`) is a docs-only item (`needs_gpu: false`):**
   - `m2-pin-train-mask-sheet.md:54` (`## Done when`) specifies:
     > "A recorded answer from the PM or owner on whether the Amendment 6 sheet is live under D37 or superseded, and a one-page sheet that is ready for an owner hold-release decision: either the pins re-verified against current main and restated in a refreshed sheet, or the pins carried into the new discrete-law sheet, with a non-author FIT. The sheet itself runs nothing: no training or optimizer update is authorised by it."
2. **Why `crates/uor-r4-training/**` and `data/**` in `scope:` caused a false lock (`task-9980`):**
   - After `#3296` (`antigravity-101`) released `m2-pin-train-mask-sheet` to `status: ready`, `uor-harness loop next` (`task-9980`) refused `#2851` because `crates/uor-r4-training/**` in `m2-pin-train-mask-sheet.md:11` overlapped `crates/uor-r4-training/**` of `d11-exporter-float-trunk` (`#2820`, `claude-134` PR `#3289`).
   - As `claude-90` established for `m1-bvg-k-block-apply-c2` (`#2835`), an item that only writes a pre-registration sheet, lab record, and notebook row should not lock an entire crate (`crates/uor-r4-training/**`) or `data/**`.

## 2. Exact changes approved under D68

1. **`docs/roadmap/items/m2-pin-train-mask-sheet.md` (`#2851`):**
   - Narrow `scope:` from `[crates/uor-r4-training/**, data/**, docs/harness/prep/m2-pin-train-mask-sheet*, docs/harness/milestones.json, docs/labs/m2-pin-train-mask-sheet/**, docs/research/notebook/m2-grounded-reply.md, docs/roadmap/items/m2-pin-train-mask-sheet.md]` to `[docs/harness/prep/m2-pin-train-mask-sheet*, docs/harness/milestones.json, docs/labs/m2-pin-train-mask-sheet/**, docs/research/notebook/m2-grounded-reply.md, docs/roadmap/items/m2-pin-train-mask-sheet.md]`.
2. **No bar, status, or technical constraint changed:**
   - All M2 bars, TRAIN mask pins (`mw-balp/train/response_mask.u8`, `b5e7352d…`, `2,425,925`), `## Done when`, and `CHARTER.md §3` constraints remain untouched.

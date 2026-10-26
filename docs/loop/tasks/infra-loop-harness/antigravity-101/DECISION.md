# D68 Decision Card: Release `m2-pin-train-mask-sheet` (`#2851`) to `status: ready` on the Discrete Branch (`antigravity-101`, #3296)

- **Lab / Session:** `antigravity` (`antigravity-101`, PR `#3296`)
- **Umbrella item:** [`docs/roadmap/items/infra-loop-harness.md`](../../../../roadmap/items/infra-loop-harness.md) (`#2904`, Setup row 2)
- **Target item:** [`docs/roadmap/items/m2-pin-train-mask-sheet.md`](../../../../roadmap/items/m2-pin-train-mask-sheet.md) (`#2851`, M2)
- **Authority:** [`CHARTER.md §5.1`](../../../../../CHARTER.md), D68 (`docs/integration/DECISIONS.md`)

## 1. Whole-project context (`CHARTER.md §5.1`)

1. **`m2-pin-train-mask-sheet` (`#2851`) release condition (`m2-pin-train-mask-sheet.md:43-45`):**
   - On 25 Oct 2026 (`claude-101`), `m2-pin-train-mask-sheet` was set to `status: held` with an explicit release rule:
     > "A lab may release it when either of these holds:
     > - **The D37 branch:** `float-training-module` (#2823) and `d11-exporter-float-trunk` (#2820) are done. The D37 sheet then needs its sealed fresh slice and a non-author FIT.
     > - **The discrete branch:** `m1-trunk-assembly-and-training` (#2844) is `done`, or a merged discrete trainer in `crates/uor-r4-discrete` is named by commit. Either gives a one-page M2 discrete-law sheet a current trainer to pin and a current head to FIT at."
2. **All discrete-branch prerequisites are now `status: done` on `origin/main`:**
   - [`m1-trunk-assembly-and-training.md`](../../../../roadmap/items/m1-trunk-assembly-and-training.md) (`#2844`) is `status: done` (merged in `#3258` as `d1ef51fb2cf895fa27908f90f45269923d466727`).
   - [`m1-bvg-k-stage-t-training.md`](../../../../roadmap/items/m1-bvg-k-stage-t-training.md) (`#2836`) is `status: done` (merged in `#3239` as `1528426b4a269ef8e2386c718193a84534452be4`).
   - [`m2-fresh-heldout-panel.md`](../../../../roadmap/items/m2-fresh-heldout-panel.md) (`#2849`) is `status: done` (merged in `#3134` as `b9ca0e093be0cccf9e65a20f0b4d5a87a98d6822`, sealing the 256-row `m2-fresh256` panel and creating `docs/research/notebook/m2-grounded-reply.md`).
3. **Why releasing `m2-pin-train-mask-sheet` (`#2851`) now keeps the fleet moving without overlap:**
   - `m1-close-saved-model` (`#2837`) is currently blocked on `crates/uor-r4-discrete/**` by `kimi-43` (`PR #3295` on `r-ranking-with-admission-intact`, `#2884`), and `d11-exporter-float-trunk` (`#2820`) is claimed by `claude-134` (`PR #3289`).
   - `m2-pin-train-mask-sheet` (`#2851`, `needs_gpu: false`) has zero file overlap with `#3295`, `#3289`, or `#3169`, and completing its discrete-law one-page M2 pre-registration sheet + non-author FIT satisfies the `m2-pin-train-mask-sheet` dependency of `m2-train-grounded-replies` (`#2853`).

## 2. Exact changes approved under D68

1. **`docs/roadmap/items/m2-pin-train-mask-sheet.md` (`#2851`):**
   - `status: held` -> `status: ready`
   - `depends_on: []` -> `depends_on: [m1-trunk-assembly-and-training, m2-fresh-heldout-panel]` (both `status: done` on `origin/main`)
   - Add `notebook: docs/research/notebook/m2-grounded-reply.md`
   - `scope:` -> `[crates/uor-r4-training/**, data/**, docs/harness/prep/m2-pin-train-mask-sheet*, docs/harness/milestones.json, docs/labs/m2-pin-train-mask-sheet/**, docs/research/notebook/m2-grounded-reply.md, docs/roadmap/items/m2-pin-train-mask-sheet.md]`
   - Record the discrete-branch release note in `## What is missing`.
2. **No bar or technical constraint changed:**
   - The M2 bars (`>= 256 / 512` saved questions and `>= 40%` / `>= 52 / 128` pairs on `m2-fresh256`), the TRAIN mask pins (`mw-balp/train/response_mask.u8`, `7,512,797` B, sha256 `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`, `expected_train_targets=2425925`), and `CHARTER.md §3` are untouched.

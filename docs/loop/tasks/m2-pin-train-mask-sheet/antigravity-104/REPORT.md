# REPORT: m2-pin-train-mask-sheet, antigravity-104

Lab: antigravity. Order: [ORDER.md](ORDER.md).

## What did not work, first

- No mechanism negative and no model compute ran in this unit (`needs_gpu: false`; host-only pre-registration sheet, pin verification, and non-author `TEST FITNESS` review).
- During the cancel probe, an initial short-SHA expansion (`1528426b...`, `d1ef51fb...`, `b9ca0e09...`) failed `git merge-base --is-ancestor` because those short SHAs were branch head SHAs before squash-merge; we replaced them with the verified squash-merge commits on `origin/main` (`c695ad21a72c5b91def22458eebbe796e07d5392` for `#3239`, `6f38dd393865a0b4320a2e28bee3be675130dc16` for `#3258`, and `f1f55c0c1095d9b4da8457f655ac72d42d9f0180` / `0201e026b6358db4951c5a972b278e1538b198b4` for `#3147`/`#3148`).
- The companion D37 offline-float pre-registration sheet ([`docs/harness/prep/m2-pin-train-mask-sheet-2026-10-23.md`](../../../harness/prep/m2-pin-train-mask-sheet-2026-10-23.md)) remains at `TEST FITNESS: NOT_FIT` until the separate D29-scanned float module (`float-training-module`, `#2823`) and D11 float-trunk exporter (`d11-exporter-float-trunk`, `#2820`) land and a float fresh-transfer slice is sealed.
- M2 training and rental holds (`training_hold: true`, `rental_hold: true` in `docs/harness/milestones.json`) remain `HELD`; the sheet runs nothing and a `TEST FITNESS: FIT` verdict releases no hold (`CHARTER.md` §4.5, §5.1).

## What changed

- **Size by class:** `0` test lines, `0` code lines, `+221 / -13` docs/generated lines across `7` files (within the ORDER's docs-only scope; ratio `1.0x`).
1. **Wrote the one-page discrete-law M2 pre-registration sheet ([`docs/harness/prep/m2-pin-train-mask-sheet-discrete-2026-10-26.md`](../../../harness/prep/m2-pin-train-mask-sheet-discrete-2026-10-26.md)):**
   - Completes the discrete-law route of `m2-pin-train-mask-sheet` (`#2851`) in accordance with the owner's scope ruling (`2026-10-24T03:42Z`, [`#2028` comment `6379430093`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6379430093): *"I want both because we do not know which we will ultimately need"*), now that `m1-bvg-k-stage-t-training` (`#2836`, merged in `#3239` at `c695ad21a72c5b91def22458eebbe796e07d5392`), `m1-trunk-assembly-and-training` (`#2844`, merged in `#3258` at `6f38dd393865a0b4320a2e28bee3be675130dc16`), and `m2-fresh-heldout-panel` (`#2849`, merged in `#3147`/`#3148` at `f1f55c0c1095d9b4da8457f655ac72d42d9f0180` / `0201e026b6358db4951c5a972b278e1538b198b4`) are `done` on `main`.
   - Carries all four Amendment 6 mechanical count-reference pins into the current discrete law:
     - `mw-balp/train/tokens.u16` (`7,512,797` `u16` payload tokens, sha256 `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`) and `mw-balp/train/response_mask.u8` (`7,512,797` bytes, sha256 `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`);
     - `expected_train_targets=2425925` (`expected_eligible_targets = 2425925`), enforced fail-closed by `crates/uor-r4-discrete/src/controller.rs:184-191` and `docs/labs/m1-bvg-k-stage-t-training/configs/t16-{frozen-zero,relation}-seed{1,2}.json`;
     - frozen count-reference source commits `2fd6358bbc52d2a202a872ad0954ebb12894606f` (`11` finite controls PASS) and PR `#2328` merge `9e904a83448c451f738d5c1d45224dfdf73eb84e` (ancestor of `main`);
     - the current discrete trainer and Stage 3 trunk CLI (`stage-t-run` and `trunk-run` in `crates/uor-r4-discrete`, with deterministic compiled digests `driver_source_sha256 = 04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d`, `geometry_sha256 = 46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327`, `kernel_sha256 = 0f9ccfa7b4748292f83a40227e30ea7b69c893a1649fa8afa3f7d547802429d6`, and zero-gap `D11` `uor-r4.lut-discrete/1` integer artifact export).
   - Pins both the rung-0 discrete rank-margin bars (`top-1` and `recall@8` >= count predictor + `5 pp` across `Seed 1` & `Seed 2` on `14,287` `ft/dev` rows and `14,287` `fresh_v2` rows) and the M2 grounded-reply milestone bars (`complete >= 256 / 512` on `m2-reply512` and `pairs_both_complete_distinct >= 52 / 128` with single-row `complete >= 104 / 256` on the sealed 256-row `m2-fresh256` panel at HF commit `fb8c5e4fcd3568cc59be8be3073151295b98069c`, graded by the existing deterministic complete-reply evaluator unchanged).
   - Audited by an independent fresh-context non-author reviewer (`d929c424-0c35-4319-9260-6c9e41913c06`), recording `TEST FITNESS: FIT - M2 #2030 discrete-geometric-learning` and posting the FIT review on `#2851` (`#issuecomment-6425469030`) and `#2030` (`#issuecomment-6425469648`).
2. **Updated [`docs/labs/m2-pin-train-mask-sheet/README.md`](../../../labs/m2-pin-train-mask-sheet/README.md), [`docs/harness/milestones.json`](../../../harness/milestones.json), [`docs/research/notebook/m2-grounded-reply.md`](../../../research/notebook/m2-grounded-reply.md), [`docs/roadmap/items/m2-pin-train-mask-sheet.md`](../../../../roadmap/items/m2-pin-train-mask-sheet.md), and [`docs/roadmap/README.md`](../../../roadmap/README.md):**
   - Recorded the discrete-law sheet and its `TEST FITNESS: FIT` alongside the retained D37 sheet in `docs/labs/m2-pin-train-mask-sheet/README.md` and `docs/harness/milestones.json` (preserving `training_hold: true` and `rental_hold: true`).
   - Appended the M2 notebook rows in `docs/research/notebook/m2-grounded-reply.md`.
   - Set `docs/roadmap/items/m2-pin-train-mask-sheet.md` to `status: done` and re-rendered `docs/roadmap/README.md`.

## Measurements

- Nothing was measured on a GPU (`needs_gpu: false`). Host-only verification confirmed all four Amendment 6 mechanical count-reference pins (`mw-balp/train/response_mask.u8` `7,512,797` B sha256 `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`, `expected_train_targets = 2,425,925`, `ft/dev` `14,287` targets, and `m2-fresh256` `256` rows / `128` pairs) against `crates/uor-r4-discrete/src/controller.rs:184-199`, `docs/labs/m1-bvg-k-stage-t-training/configs/`, and `docs/harness/panels/m2-fresh256.json`.

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --no-fail-fast -p uor-r4-harness --lib preprules::tests` -> `17 passed; 0 failed` (including `the_real_sheets_pass_the_shape_rule` and `the_real_sheets_are_processed`)
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --no-fail-fast -p uor-r4-harness --lib prep::tests` -> `13 passed; 0 failed`
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run -p xtask -- roadmap check` -> `roadmap check: 94 items OK`
- `uor-harness loop report --check --order docs/loop/tasks/m2-pin-train-mask-sheet/antigravity-104/ORDER.md` -> `PASS`
- `uor-harness loop pr-check --local` -> `verdict: PASS`
- `uor-harness loop gates --full` -> `11 / 11` `GREEN`

## Skills used

- `uor-research-architect` (`.agents/skills/uor-research-architect/SKILL.md`): verified all four Amendment 6 mechanical count-reference pins, discrete trainer and trunk commits, and sealed `m2-fresh256` digests against `origin/main`.
- `scope-guard` (`/Users/casey.allard/.gemini/config/skills/scope-guard/SKILL.md`): kept changes strictly inside `m2-pin-train-mask-sheet`'s scope (`docs/harness/prep/m2-pin-train-mask-sheet*`, `docs/harness/milestones.json`, `docs/labs/m2-pin-train-mask-sheet/**`, `docs/research/notebook/m2-grounded-reply.md`, `docs/roadmap/README.md`, and the `status: done` transition in `docs/roadmap/items/m2-pin-train-mask-sheet.md`).

## Still open

- **Companion D37 offline-float sheet (`docs/harness/prep/m2-pin-train-mask-sheet-2026-10-23.md`):** Remains at `TEST FITNESS: NOT_FIT` until `float-training-module` (`#2823`) and `d11-exporter-float-trunk` (`#2820`) land and a float fresh-transfer slice is sealed.
- **Downstream discrete M2 training (`m2-train-grounded-replies`, `#2854`):** Remains gated on `m2-donor-corpus-generation` (`#2850`), `m1-close-saved-model` (`#2837`), and `m1-d11-exporter-discrete` (`#2821`), with M2 `training_hold: true` and `rental_hold: true` preserved.

## Plain summary

Before running any GPU training or fresh-transfer evaluation for Milestone 2 (M2, grounded conversation where the model answers questions from exact stored memory), the project charter requires a one-page pre-registration sheet that locks in the exact training data checksums, target counts, command-line steps, pass/fail bars, and stop rules, and passes a review by an independent fresh-context reviewer (`TEST FITNESS: FIT`). Earlier, the project owner asked to keep both an offline floating-point pre-registration sheet and a pure-integer discrete-law pre-registration sheet. In this unit, we wrote the one-page discrete-law M2 pre-registration sheet (`docs/harness/prep/m2-pin-train-mask-sheet-discrete-2026-10-26.md`) pinning the `2,425,925`-target response-masked training set (`mw-balp/train/response_mask.u8`), the merged discrete coordinate-block trainer (`stage-t-run`) and single-vector geometric trunk (`trunk-run`), the M2 development bar (`>= 256 / 512` complete correct replies on `m2-reply512`), and the sealed 256-row fresh held-out panel (`>= 52 / 128` opposite-role history pairs on `m2-fresh256`), verified that an independent fresh-context review marked it `TEST FITNESS: FIT`, recorded the result in the M2 research notebook, and closed `#2851`.

Closes #2851
References #2030

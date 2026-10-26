# Notebook: grounded reply (M2)

This notebook records results about the native learner completing correct replies grounded in exact memory. It has no result rows yet. Panel construction and seals are instrument evidence, not learned-language results.

## Results

Sources: [`archive/catalog.jsonl`](../../../archive/catalog.jsonl). Verdict words are defined on the [format page](./README.md#the-five-verdict-words).

| Date | What ran | Number | Bar | Verdict | Artifact |
|---|---|---|---|---|---|
| | *No result rows yet.* | | | | |
| 2026-10-23 | Host-only `saved437` dev512 replay spread audit and construction/overlap/control/seal verification of the 256-row replacement fresh held-out panel `m2-fresh256` (`#2849`, `$0`). | `saved437` dev512 replay mismatches `0 / 512` (`7,908` steps), half-panel split `219 / 256` vs `218 / 256`, 16-block cluster SE `6.24` rows (`8.64` rows / `4.32` pairs at `40%` bar); `m2-fresh256` has `0` exact/word/2-gram overlaps against all `74` historical literals and `0 / 128` pairs on all 4 question-blind and role-swap controls (`NOT_RUN` for candidate scoring). | `pairs_both_complete_distinct >= 52 / 128` (`complete >= 104 / 256`) on `m2-fresh256` and `complete >= 256 / 512` on `m2-reply512`. | **positive** (panel construction & seal `TEST FITNESS: FIT`, candidate scoring `NOT_RUN`) | [`docs/labs/m2-fresh-heldout-panel/README.md`](../../labs/m2-fresh-heldout-panel/README.md), [`m2-fresh256.json`](../../harness/panels/m2-fresh256.json) |
| 2026-10-26 | Host-only verification of the four Amendment 6 count-reference pins against `origin/main` and non-author `TEST FITNESS` review of the discrete-law M2 pre-registration sheet `m2-pin-train-mask-sheet-discrete-2026-10-26.md` (`#2851`, `$0`). | `mw-balp/train/response_mask.u8` `7,512,797` B (`b5e7352d…`), `expected_train_targets = 2,425,925`, `ft/dev` `14,287` targets (`a838bb1a…`), `fresh_v2` `14,287` targets (`fe873639…`), and `m2-fresh256` `256` rows (`1de2fc28…`) verified across `stage-t-run` (`c695ad21…`) and `trunk-run` (`6f38dd39…`); `17 / 17` `preprules::tests` pass (`NOT_RUN` for M2 training, holds unchanged). | Rung-0 `top-1` & `recall@8` >= count predictor + `5 pp` on `14,287` DEV & `14,287` `fresh_v2` rows across 2 seeds; M2 `complete >= 256 / 512` on `m2-reply512` and `pairs_both_complete_distinct >= 52 / 128` on `m2-fresh256`. | **positive** (discrete-law sheet `TEST FITNESS: FIT`; D37 companion sheet retained at `NOT_FIT` pending `#2823`/`#2820`; M2 training `HELD`) | [`docs/labs/m2-pin-train-mask-sheet/README.md`](../../labs/m2-pin-train-mask-sheet/README.md), [`m2-pin-train-mask-sheet-discrete-2026-10-26.md`](../../harness/prep/m2-pin-train-mask-sheet-discrete-2026-10-26.md) |
| 2026-10-26 | Post-merge executable-chain audit of the discrete M2 sheet (`m2-pin-train-mask-sheet-cli-repair`, `#3303`, `kimi-45`): source parser inspection and byte comparison only; no model execution or pod (`$0`). | `TEST FITNESS: NOT_FIT`: Stage T/trunk required arguments and binary staging absent, trunk Step 0 precedes its checkpoint, anchor build/Stage T grade absent, no M2 student, no candidate-specific GPU reply evaluator or D11/native-bank bridge. Bar 1 is an M1 composite, not a faithful M2/Amendment bar; best antecedent top-8 `4,674 < 4,775`. M2 student cost/power `UNDEFINED`. Superseded recipe matches the #3300 base-blob sections with zero diff. | A FIT must describe an executable end-to-end configuration; no compute authorized and no mechanism/bar/hold/source change. | **negative** (`INSTRUMENT_NOT_FIT`; #2851 remains a pins-delivery close only; #2853 must provide student/evaluator/bridge under a new ORDER; not a model or mechanism negative) | [`docs/labs/m2-pin-train-mask-sheet/README.md`](../../labs/m2-pin-train-mask-sheet/README.md), [`superseded recipe`](../../labs/m2-pin-train-mask-sheet/superseded-recipe-2026-10-26.md) |

## What a row here will need

A row needs a fixed bar, the exact saved model and panel identities, the number and denominator, and the grader's deterministic result on the sealed fresh panel. Development and fresh transfer results stay distinct. A panel seal, fixture pass or metadata change is not a capability result.

## Go deeper

- [Format and verdict words](./README.md).
- [`CHARTER.md`](../../../CHARTER.md), sections 3.10, 4.3 and 5.6.

# ORDER: spiralcore-fbs-offline-float-sheet, antigravity-81

Lab: antigravity. Item: [spiralcore-fbs-offline-float-sheet](../../../../roadmap/items/spiralcore-fbs-offline-float-sheet.md) (#2891). Base: `ca168bbae`.

## Goal

Author the one-page pre-registration sheet `docs/harness/prep/spiralcore-fbs-offline-float-sheet-2026-10-25.md` for `spiralcore-fbs-offline-float-sheet` (#2891) with all 8 required blocks (`bar`, `base`, `split`, `steps`, `stop`, `cost`, `power`, `fit`), obtain its fresh-context non-author `TEST FITNESS: FIT` review (`CHARTER.md` §4.5) and D68 decision review on the pre-registered bars (`CHARTER.md` §5.1), and update `docs/mechanisms/spiralcore-fbs.md` to link the FIT-cleared sheet so the GPU runner + execution unit can proceed immediately once `crates/uor-r4-float-train` is free.

## Cancel probe (run first)

At `ca168bbae4f587e9fb99a9c7b79b4530cdc3f908`:
- Inspected `docs/roadmap/items/spiralcore-fbs-offline-float-sheet.md` (#2891), `docs/mechanisms/spiralcore-fbs.md`, `archive/labs/spiralcore-fbs-projection-2026-10-12/README.md`, `docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/spiralcore-fbs.json`, and `docs/labs/r-secant-surrogate-descent/README.md`.
- Both `depends_on` items (`float-training-module` #2823 and `r-secant-surrogate-descent` #2886, `G-Fbs-Descent = PASS`) are `done` on `main`.
- Checked `docs/harness/prep/spiralcore-fbs-offline-float-sheet*`: no pre-registration sheet exists yet, and `CHARTER.md` §4.5 requires a one-page sheet with a non-author `TEST FITNESS: FIT` review before any GPU training or evaluation run.
- **Cancel probe result (`instrument`):** Missing pre-registration sheet and `TEST FITNESS: FIT` review confirmed; proceed to author the sheet, run fresh-context `TEST FITNESS` and D68 reviews, and update `docs/mechanisms/spiralcore-fbs.md`.

## Files in scope

- `docs/mechanisms/spiralcore-fbs.md`
- `docs/harness/prep/spiralcore-fbs-offline-float-sheet*`
- `docs/labs/spiralcore-fbs-offline-float-sheet/**`
- `docs/loop/tasks/spiralcore-fbs-offline-float-sheet/**`

## Done when

- `docs/harness/prep/spiralcore-fbs-offline-float-sheet-2026-10-25.md` is committed with all 8 `REQUIRED_BLOCKS` (`uor-harness check` `prep.sheet_shape` passes with 0 errors), pinning the question, 4 matched geometric arms, bars/decision table, instrument checks, `mw-balp/train` + `ft/dev` + fresh-V2 SHA-256 digests, D11 export-gap protocol, tokens/s measurement protocol, cost, and stop rules.
- Fresh-context non-author `TEST FITNESS: FIT` review (`CHARTER.md` §4.5), D68 decision review (`CHARTER.md` §5.1), and D66 exact-head review (`CHARTER.md` §5.5) pass and are posted on `#2891` and `#3233`.
- `docs/mechanisms/spiralcore-fbs.md` links the FIT-cleared sheet, and `uor-harness loop report --check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` pass (`References #2891`).

## Stop and ask if

- Never stop and wait. If the fresh-context `TEST FITNESS` reviewer finds any ambiguity in the arms, bars, or pins, fix the sheet and re-review in the same unit.

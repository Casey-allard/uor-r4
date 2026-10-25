# ORDER: m1-normalization-mechanism, antigravity-84

Lab: antigravity. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md) (#2842). Base: `dcc4fe443`.

## Goal

Complete the pre-registration sheet `docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md` for `OrbitRootSnapNormV1` (`#2842`), obtain a fresh-context non-author `TEST FITNESS: FIT` review (`CHARTER.md` §4.5, §5.1, D68), record the notebook row in `docs/research/notebook/m1-discrete-attention.md`, and close `#2842` (`status: done`):

1. Resolve all four blocking prerequisites and three non-blocking items from `claude-57`'s `2026-10-23` `NOT_FIT` review in `docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md`:
   - **Prerequisite 1 (runnable GPU entry & commands):** Cite the merged discrete normalization entry points and scorers from `claude-66` (`#3116`), `claude-67` (`#3119`), `claude-68` (`#3121`), and `claude-72` (`#3131`) (`discrete-stack normalization-build-identity`, `normalization-check-plan`, `normalization-owner-release`, `normalization`, `normalization-resume`, `normalization-dev-b3`, `normalization-dev-paired`, and `normalization-dev-structure`) across `ScaleMatchedSnapV1`, `UnitSnapControlV1`, and `UnnormalizedControlV1` and seeds `1` and `2`, running `N_snap` under `m1-normalization-n_snap-injective-6600bdb1306d-relation-k8-seed{1,2}-batches0..16404`.
   - **Prerequisite 2 (single configuration pinned):** Pin this sheet to the discrete A8 proxy trunk (`ReadSpec::Relation { kept: 8, sweeps: 1 }`, `address_structured_v1` codebook `6600bdb1306d…`, driver source pin `cb686691d136…`); the assembled Stage 3 trunk (`#2844`) registers under its own sheet.
   - **Prerequisite 3 (mechanically scoreable bar terms):** Quote Casey's `2026-10-24T03:41:41Z` ruling on `#2028` (`6379430093`) in §3: strict non-inferiority margin against `C_unscaled` is $b - c > -F_I$ where $F_I = \max(24, \lceil 2\sqrt{b + c}\rceil)$, and fresh-V2 replication threshold is $b - c \ge \lceil 2\sqrt{b + c}\rceil$ (no floor).
   - **Prerequisite 4 & items 5–7:** Add the 3-branch outcome decision table (`PASS_PROMOTE`, `FAIL_DEV_CLOSE_CONFIG`, `FAIL_TRANSFER_CLOSE_CONFIG`), expected effect size vs noise, full `fresh_v2` digests (`0b7e754c…`, `a1cda529…`, `df5d074a…`), and updated §6 steps.
2. Append the fresh-context `TEST FITNESS: FIT` record at the end of `## Verdict` so `prep::test_fitness` parses `FIT`.
3. Record the notebook row in `docs/research/notebook/m1-discrete-attention.md`, set `m1-normalization-mechanism.md` to `status: done`, and re-render `docs/roadmap/README.md`.

## Cancel probe (run first)

At `dcc4fe4435e9b2f9cea0e130954e685757fedefa`:
- `uor-harness prep list | grep m1-normalization-mechanism` reports `NOT_FIT`.
- `cargo test --locked -p uor-r4-discrete --lib normalization` (`20/20`) and `--lib dev_paired` (`3/3`) pass.
- **Cancel probe result (`instrument`):** All code prerequisites and owner rulings (`6356034555`, `6379430093`) are landed on `main`; proceed with the sheet update and `TEST FITNESS: FIT` review.

## Files in scope

- `docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md`
- `docs/labs/m1-normalization-mechanism/**`
- `docs/roadmap/items/m1-normalization-mechanism.md`
- `docs/roadmap/README.md`
- `docs/research/notebook/m1-discrete-attention.md`
- `docs/loop/tasks/m1-normalization-mechanism/antigravity-84/**`

## Done when

- `docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md` resolves all `claude-57` prerequisites, passes a fresh-context non-author `TEST FITNESS: FIT` and D68 decision review, and `uor-harness prep list` reports `FIT`.
- `m1-normalization-mechanism.md` is `status: done`, `docs/roadmap/README.md` is re-rendered, a row is recorded in `docs/research/notebook/m1-discrete-attention.md`, and `uor-harness loop gates --full` is green (`Closes #2842`).

## Stop and ask if

- Never stop and wait. Decide any sheet or bar clarification via fresh-context D68 decision review and log it for Casey.

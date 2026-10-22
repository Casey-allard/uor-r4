# ORDER: m1-address-as-read, antigravity-17 (follow-up: prep.sheet_shape blocks)

Lab: antigravity. Item: [m1-address-as-read](../../../../roadmap/items/m1-address-as-read.md) (#2833). Base: `ce7782a74`.

## Goal

Complete [`docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md`](../../../../harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md) (landed in `#2950`) to the full 8-block `prep.sheet_shape` specification by adding the four missing headings (`Base artifact`, `Data split`, `Steps`, `Minimum test power`) flagged on `#2028` so that `uor-harness check` passes with `0 errors` in the `harness rules and tests` CI job.

Estimated diff size: ~55 lines across `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md` and `docs/loop/tasks/m1-address-as-read/antigravity-17/**`.

## Cancel probe (run first)

Ran `uor-harness check` on `ce7782a74`:
- Reports `error prep.sheet_shape [docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md]: missing block(s): Base artifact, Data split, Steps, Minimum test power`.
- Result: repair is needed; continue.

## Files in scope

- `docs/harness/prep/m1-address-as-read*`
- `docs/loop/tasks/m1-address-as-read/antigravity-17/**`

## Done when

- `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md` carries all 8 required blocks (`Bar`, `Base artifact`, `Data split`, `Steps`, `Stop`, `Cost`, `Minimum test power`, `TEST FITNESS: PENDING`).
- `uor-harness check` reports `0 errors` (`prep.sheet_shape` passes on every sheet in `docs/harness/prep/`).
- `uor-harness loop gates --full`, `uor-harness loop report --check`, and `uor-harness loop pr-check --local` pass.

## Stop and ask if

- Any file outside the scope list above would need to change.

# ORDER: verify the merged Studio lifecycle migration, kimi-73

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `bfa5de6ce73dbd459f1d2dca1e4e677ccc2bae54`, tree `b5dea93d0dde75f3134cc8576f279d57f08232df`.

## Finding and cancel probe

The requested mounted browser-state migration is already present through merged PR #3859, including the `kimi-67` implementation and `kimi-68`/`kimi-69` repairs. The cancel probe is its existing early writer, verified marker, routed persistence, mount refusal, and real-browser success/failure verifier. Exact-head checks pass, so no second migrator or product repair is justified.

The first complete acceptance attempt passed all product/browser/legacy checks but `loop pr-check --local` rejected the empty branch because it added no ORDER/REPORT. This unit records and delivers only the requested verification plan and its evidence; it does not manufacture a product diff to satisfy the harness.

## Files in scope

- `docs/superpowers/plans/2026-11-08-studio-lifecycle-migration.md`.
- `docs/loop/tasks/studio-typed-shell-upgrade/kimi-73/`.
- Generated evidence under `local/kimi/studio-lifecycle-migration/` (not committed).

No product, verifier, model/engine/worker/tokenizer/`studio_api.js`, dependency, native/package, production cutover/publication, Phase B, real browser profile, credential, or external-endpoint change.

## Done when

- The existing migration passes focused and complete typed/plain-JS checks, typecheck/build, the typed built-browser matrix, all four legacy checks, harness report/local-PR/full-gate checks, and fresh exact-head review.
- The report distinguishes the initial generated-artifact prerequisite and empty-diff PR-check failure from migration behavior, and records every NOT_RUN honestly.
- Only this unit's documentation is delivered through the protected PR path; actual merge and fresh-main identity are verified.

## Stop and report

Stop before a product edit without a newly reproduced migration defect and separately reviewed ORDER, any real-data/credential/external-endpoint use, verifier weakening, destructive cleanup, or production publication. A failed protected check is not routed around.

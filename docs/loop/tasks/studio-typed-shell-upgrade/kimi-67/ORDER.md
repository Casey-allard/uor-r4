# ORDER: non-destructive mounted browser-state migration, kimi-67

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `7fea6c761417c0e8b589cb85ed1ea11eb1216067`, tree `ef59d3d3fbd473593c612291e690a723f4325b23`.

## Findings to repair

- `migration.ts` only reads fixtures and creates plans. No writer, verified marker, rollback, destination conflict handling, or mounted migration exists.
- Typed Studio still constructs an empty lifecycle store and reuses legacy writers. A copy-only unit would let later typed edits rewrite the rollback keys.
- No real-browser check proves migrated session/workspace/settings restoration, post-migration writes, or reopened-state persistence.

## Selected approach and cancel probe

Use the approved one-time copy-forward design with an early shared migration/key-routing layer, not a second state owner or rewritten persistence stack. Preflight all migratable inputs and destinations before any write. Preserve malformed/conflicting data, roll back only this attempt's new writes, and refuse typed mounting on migration failure. `modelSources.v1` bypasses routing and remains the sole live provider-credential store.

The cancel probe is the current absence of any marker/write/route API and mounted migration evidence. Red-first tests must prove the missing behavior before implementation.

## Files in scope

- `web/studio/src/persistence/`, bootstrap/main and their tests; `assets/js/lifecycle_persistence.js` and tests.
- Existing storage call sites in `web/studio/index.html`, `assets/js/studio_ui.js`, and `assets/js/studio_files.js` only for shared routing.
- `web/studio/verify_studio_page_load.mjs`, Studio documentation, the approved design, this task's PLAN/REPORT, roadmap/notebook evidence.

No dependency, real browser data or credentials, model/engine/worker/tokenizer/`studio_api.js` change, native/package work, production cutover/publication, or Phase B compute.

## Done when

- Every migratable key copies once with unknown fields intact and an idempotent verified marker; old migratable keys remain byte-identical. Provider credentials are never copied, logged, or routed into another store.
- Malformed input, destination conflicts, write/verification/marker failures preserve source data, roll back only this attempt's writes, and prevent typed mounting.
- Successful migration routes subsequent legacy session/file/UI persistence to destinations. Real isolated-browser fixtures prove restored state, post-edit old-key preservation, and reload persistence for representative workflows; failure fixtures fail closed.
- Existing typed two-session/UOR flow, default legacy behavior, full product/legacy/repository checks, fresh exact-head review, and protected delivery pass.

## Stop and report

Stop before any real-data migration, credential exposure, destructive reset, silent conflict overwrite, untested storage rewrite, out-of-scope change, or production publication. Continue only with the declared risks resolved.

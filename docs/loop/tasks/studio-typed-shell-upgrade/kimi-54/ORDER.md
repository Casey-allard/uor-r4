# ORDER: prove default and error entry behavior, kimi-54

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `06e2e1920a406a67a094a966ce5c73bfce5e2402`, tree `40163654090d240133d6e6980e85f18c112f88b4`; reviewed merge base: `e9bfd7751e3eb31736178d23289e8d99480bb0d0`. Agent-32 returned CHANGES and blocked push/queue.

## Finished steps

- kimi-52 mounted the real Vite preview. kimi-53 made it computed-visible and corrected base/publication evidence; 383/383 typed, 112/112 plain-JS, all four legacy checks, and 11/11 full gates pass.
- Agent-32 accepted those visibility/base/publication repairs but found a vacuous default test, a stale 382 notebook row, an ambiguous final-head requirement, and an unexecuted visible error path.

## Goal

Evaluate `main.ts` with a pre-existing root and `typed` absent, execute the visible initialization error path with a bounded timeout, assert no runtime exceptions, update the notebook to 383, and explicitly reconcile final-head identity through `identity.log` and PR metadata rather than a self-referential REPORT hash.

## Cancel probe

Run the revised entry tests before source changes. The no-default test must create the root before evaluating `main.ts` and prove no mount/mutation; the error-path test must show computed-visible error text, `data-active=true`, and no runtime exception. If either test errors instead of exercising entry behavior, stop and repair the instrument. Also require the notebook count and runtime-exception assertion that were missing.

## Files in scope

- `web/studio/src/main.test.ts` and `verify_studio_page_load.mjs` only for entry behavior and runtime-exception assertions.
- `docs/research/notebook/m4-serving.md` for the corrected 383 row.
- `docs/loop/tasks/studio-typed-shell-upgrade/kimi-52/REPORT.md` and `kimi-53/ORDER.md`/`REPORT.md` only for explicit final-head identity convention and check evidence.
- This `kimi-54` repair folder.

No production source, publisher, deployment, Tauri, engine, worker, tokenizer, `studio_api.js`, model artifact, `.github/**`, `Cargo.lock`, real endpoint/credential/browser-data migration, owner-controlled file, generated `docs/roadmap/issues.json`, dependency, second model provider, production publish, useful-chat, M7/M8, or accessibility-certification claim.

## Done when

- The no-default test evaluates `main.ts` with a pre-existing root and proves no mount/mutation without `typed=1`.
- The error test evaluates `main.ts` with incomplete workflow methods and proves computed-visible error text, `data-active=true`, and no runtime exception. The built typed scenario refuses any runtime exception or console error.
- The notebook and all other records use 384/384. ORDER/REPORT explicitly say exact final HEAD/tree are captured in `identity.log` and posted on PR after push rather than embedded self-referentially.
- Focused/full Studio checks, plain-JS, all four legacy checks, report/local-PR checks, and full gates pass at the new exact head. Fresh exact-head review approves before ready/queue.
- PR keeps `References #3500`; no production publish, final migration, Tauri acceptance, or model-capability claim.

## Stop and report

Stop before changing production code for a coverage-only gap, weakening a test, or using a stale build to claim success. Stop after fresh approval and protected delivery of this unit, then continue Phase A.

# ORDER: execute the real entry catch and make the default entry publishable, kimi-55

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `cebee77c7cdf6d7f891161da6d45e2ebbe0e35f3`, tree `4c122754de916d56bc114dcc7a63c5c55663b346`; reviewed merge base: `e9bfd7751e3eb31736178d23289e8d99480bb0d0`. Agent-33 returned CHANGES and blocked push/queue.

## Finished steps

- kimi-52 mounted the preview; kimi-53 made it computed-visible; kimi-54 added a real no-default test, exported `showMountError`, asserted runtime errors in the typed scenario, and raised the suite to 384.
- Agent-33 confirmed mount visibility, no-default behavior, worker reuse, and model readiness, but rejected the helper-only error test, stale 383 wording, and an unstaged default TypeScript reference.

## Goal

Execute the actual `main.ts` catch with incomplete workflow methods and bounded time, prove computed-visible error text and `data-active=true`, use a staged plain-JavaScript entry that loads typed code only for `?typed=1`, assert runtime errors for default and typed pages, and align all evidence to 384 without mixing head-specific counts.

## Cancel probe

Run the revised real-catch test with incomplete workflow methods and a bounded timer. It must fail if the catch is not entered or its error text is invisible. The default source page must not request raw TypeScript, and both default and typed scenarios must assert their runtime exception lists. If either check fails only because the build is stale, rebuild and classify it as instrument, not a product regression.

## Files in scope

- `web/studio/src/main.ts`, `src/main.test.ts`, and `web/studio/index.html` only for real catch/entry behavior.
- `web/studio/assets/js/typed_entry.js` for the staged plain-JavaScript entry.
- `web/studio/verify_studio_page_load.mjs` for default and typed runtime-error assertions and log exposure.
- `scripts/studio/runtime-files.sh` only to include the plain-JavaScript entry; no publication.
- `web/studio/README.md`, `docs/products/web-client.md`, `docs/research/notebook/m4-serving.md`, `docs/roadmap/items/studio-typed-shell-upgrade.md`, `docs/loop/tasks/studio-typed-shell-upgrade/kimi-52/REPORT.md`, `kimi-53/REPORT.md`, `kimi-54/ORDER.md`, `kimi-54/REPORT.md` for actual counts/evidence.
- This `kimi-55` repair folder.

No publisher invocation, deployment, Tauri, engine, worker, tokenizer, `studio_api.js`, model artifact, `.github/**`, `Cargo.lock`, real endpoint/credential/browser-data migration, owner-controlled file, generated `docs/roadmap/issues.json`, dependency, second model provider, production publish, useful-chat, M7/M8, or accessibility-certification claim.

## Done when

- The real catch executes with incomplete workflow methods and proves computed-visible error text and `data-active=true` within a bounded test timeout. `showMountError` remains the catch's helper, not a substitute for exercising the catch.
- The default source entry is staged plain JavaScript and does not request raw TypeScript. Default and typed page-load checks assert zero runtime exceptions and expose their counts. The typed entry loads typed code only for `?typed=1`.
- All records use 384/384 and distinguish final cumulative totals from head-specific measurements. No contradictory 383 text remains in active records.
- Focused/full Studio checks, plain-JS, all four legacy checks, report/local-PR checks, and full gates pass at the new exact head. Fresh exact-head review approves before ready/queue.
- PR keeps `References #3500`; no production publish, final migration, Tauri acceptance, or model-capability claim.

## Stop and report

Stop before changing an out-of-scope file, replacing the real catch with a helper-only test, or hiding a default module-load error behind load-event checks. Stop after fresh approval and protected delivery of this unit, then continue Phase A.

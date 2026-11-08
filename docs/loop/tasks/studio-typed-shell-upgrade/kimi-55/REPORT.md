# REPORT: execute the real entry catch and make the default entry publishable, kimi-55

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `cebee77c7cdf6d7f891161da6d45e2ebbe0e35f3`, tree `4c122754de916d56bc114dcc7a63c5c55663b346`; reviewed merge base: `e9bfd7751e3eb31736178d23289e8d99480bb0d0`. Agent-33 returned CHANGES at the base. The exact final HEAD/tree are captured in `identity.log` and will be posted on PR #3835 after push, not embedded as a self-referential REPORT hash. Before push they are local evidence.

## What did not work

- Agent-33 showed that the helper-only error test did not execute the real catch, the active ORDER/REPORT said 383 while the final suite was 384, and the default source page requested an unstaged TypeScript module. It also noted that runtime errors were collected only for the typed page and were not exposed in the log.
- The real-catch test initially appeared to update the wrong root because the earlier no-default test left a duplicate `#studio-typed-root` in the document. Test setup now removes duplicate roots before and after each case.
- Awaiting the catch makes the visible result deterministic, while `showMountError` remains the catch's actual helper rather than a substitute for exercising the catch.

## What changed

- `main.test.ts` now executes the real catch with incomplete workflow methods and bounded advancement, and proves `hidden=false`, `data-active=true`, initialization error text, and computed display. No-default still evaluates `main.ts` with a pre-existing root and proves no mount or mutation.
- `assets/js/typed_entry.js` is a staged plain-JavaScript entry that dynamically imports typed code only for `?typed=1`; the raw `src/main.ts` tag is gone. `scripts/studio/runtime-files.sh` includes the entry without invoking the publisher.
- Default and typed page-load checks now assert zero runtime exceptions and expose their counts in the report. The typed runtime claim is scoped to the page, not to workers or every execution context.
- Active records use 384/384. Earlier focused counts are explicitly marked as head-specific, while the final cumulative total is 384.

## Measurements

- Entry tests: **3/3 tests pass** (API object, no-default entry, real initialization catch).
- Complete typed suite: **384/384 tests across 46 files**.
- Plain-JS suite: **112/112 tests pass**.
- TypeScript and application build: PASS; hashed code-split JS/CSS assets and generated WASM, all uncommitted.
- Typed built-browser preview: `shell=true`, `active=true`, `legacyHidden=true`, `legacyMcp=0`, `typedMcp=1`, `artifactReady=true`, `visible=true`, `typedText=true`, **zero external requests and zero runtime exceptions/console errors on both default and typed pages**.
- Default static page, runtime page-load, desktop-client, and worker checks: all PASS; worker save/restore is bit-identical and desktop/worker remote request counts are zero.
- At the final committed repair state, harness build, `loop report --check`, `loop pr-check --local`, and all 11 `loop gates --full` checks pass.

## Checks run at the head

Red evidence includes the missing bootstrap, base-path, readiness, 404, computed-visibility, vacuous-entry, mocked-entry-failure, duplicate-root, and plain-entry failures under `local/kimi/studio-mcp-connections/superpowers/checks/`. `kimi-52-precheck/` contains the earlier green pre-commit evidence. The intentional `mcp_client.js` and `typed_entry.js` D76 scope advisories are disclosed; shared notebook/FILEMAP advisories are unchanged. Agent-33 returned CHANGES for the helper-only test, stale 383 wording, unstaged default TypeScript, and log scope. Agent-34 subsequently returned CHANGES at `a59e5916ba4683134ac7114226a05fd10b128c58` because the entry test mocked `mountTypedStudio`, readiness omitted initial-render methods, this advisory sentence omitted `typed_entry.js`, the notebook cited kimi-52 for later work, and browser/test evidence had the narrower gaps repaired in kimi-56. PR push/update, GitHub checks, ready/queue/merge, and fresh-main verification are **NOT_RUN**. The PR uses `References #3500`; the issue remains open.

## Skills used

Working discipline, scope guard, solution integrity, red-first TDD, systematic bootstrap/build diagnosis, modern web application guidance, and verification before completion. A fresh independent exact-head delivery review follows the committed repair. No model compute, paid resource, credential, real user data, MCP endpoint, or production publish was used.

## Still open

This is a visible mounted development preview, not final lifecycle migration, production publication, desktop acceptance, or an autonomous/model capability claim. Legacy workflow services remain temporarily bridged. Later Phase A units replace those owners, finish native/shared packaging and acceptance, and publish only after review. The current runtime allowlist must stage `dist` before any typed publication.

## Plain summary

The typed Studio now mounts visibly and has a staged plain-JavaScript entry. The real initialization catch and default/no-default behavior are directly tested, and both default and typed pages assert zero runtime errors. The default source UI remains legacy and no production publish has run. All declared product checks pass; no real endpoint, final migration, packaged-GUI, useful-chat, M7/M8, or accessibility-certification claim is made.

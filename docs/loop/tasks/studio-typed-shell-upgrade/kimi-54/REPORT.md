# REPORT: prove default and error entry behavior, kimi-54

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `06e2e1920a406a67a094a966ce5c73bfce5e2402`, tree `40163654090d240133d6e6980e85f18c112f88b4`; reviewed merge base: `e9bfd7751e3eb31736178d23289e8d99480bb0d0`. Agent-32 returned CHANGES at the base. The exact final HEAD/tree are captured in `identity.log` and will be posted on PR #3835 after push, not embedded as a self-referential REPORT hash. Before push they are local evidence.

## What did not work

- Agent-32 found that the no-default test created its root after the static entry import, so it could pass even if `main.ts` mounted unconditionally. It also found a stale 382 notebook row, an ambiguous final-head requirement, and an unexecuted visible error path.
- The revised error test initially depended on mocked module timing and jsdom navigation, producing instrument failures rather than exercising the catch. Extracting the actual error renderer into `showMountError` made the production behavior directly testable without changing its meaning.
- The built-browser scenario collected runtime exceptions but did not assert them, allowing a potential false clean result.

## What changed

- The no-default test now creates the root before dynamically importing `main.ts`, then proves no mount or mutation without `typed=1`. Duplicate preview roots are cleaned between tests.
- The real catch now executes with incomplete workflow methods and proves computed-visible initialization error text and `data-active=true` through the exported `showMountError` helper.
- `assets/js/typed_entry.js` is a staged plain-JavaScript entry that loads typed code only for `?typed=1`; the default source path does not request raw TypeScript. Default and typed page-load checks both refuse runtime exceptions or console errors.
- The notebook uses 384/384. ORDER/REPORT explicitly make final HEAD/tree an `identity.log` and PR-metadata concern rather than embedding a self-referential hash.

## Measurements

- Entry tests: **3/3 tests pass** (API object, no-default entry, real initialization catch).
- Complete typed suite: **384/384 tests across 46 files**.
- Plain-JS suite: **112/112 tests pass**.
- TypeScript and application build: PASS; hashed code-split JS/CSS assets and generated WASM, all uncommitted.
- Typed built-browser preview: `shell=true`, `active=true`, `legacyHidden=true`, `legacyMcp=0`, `typedMcp=1`, `artifactReady=true`, `visible=true`, `typedText=true`, **zero external requests and zero runtime exceptions/console errors on both default and typed pages**.
- Default static page, runtime page-load, desktop-client, and worker checks: all PASS; worker save/restore is bit-identical and desktop/worker remote request counts are zero.
- At the final committed repair state, harness build, `loop report --check`, `loop pr-check --local`, and all 11 `loop gates --full` checks pass.

## Checks run at the head

Red evidence includes the missing bootstrap, base-path, readiness, 404, computed-visibility, vacuous-entry, and error-path failures under `local/kimi/studio-mcp-connections/superpowers/checks/`. `kimi-52-precheck/` contains the earlier green pre-commit evidence. The intentional `mcp_client.js` D76 scope advisory is disclosed; shared notebook/FILEMAP advisories are unchanged. Agent-32 returned CHANGES for the no-default test, notebook count, final-head convention, and unexecuted error path; the follow-up fresh exact-head review, PR push/update, GitHub checks, ready/queue/merge, and fresh-main verification are **NOT_RUN**. The PR uses `References #3500`; the issue remains open.

## Skills used

Working discipline, scope guard, solution integrity, red-first TDD, systematic bootstrap/build diagnosis, modern web application guidance, and verification before completion. A fresh independent exact-head delivery review follows the committed repair. No model compute, paid resource, credential, real user data, MCP endpoint, or production publish was used.

## Still open

This is a visible mounted development preview, not final lifecycle migration, production publication, desktop acceptance, or an autonomous/model capability claim. Legacy workflow services remain temporarily bridged. Later Phase A units replace those owners, finish native/shared packaging and acceptance, and publish only after review. The current runtime allowlist must stage `dist` before any typed publication.

## Plain summary

The typed Studio is mounted, visible, and its default and initialization-error entry behavior is now directly tested. The notebook and final-head evidence are consistent. The default source UI remains legacy and no production publish has run. All declared product checks pass; no real endpoint, final migration, packaged-GUI, useful-chat, M7/M8, or accessibility-certification claim is made.

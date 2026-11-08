# REPORT: make the mounted Studio visible and the evidence truthful, kimi-53

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `d620d5536cc13f0ab44bbda193368e7cf6d1df2a`, tree `e1e4c9021e963b387c48c3da44f709405db0b4ac`; reviewed merge base: `e9bfd7751e3eb31736178d23289e8d99480bb0d0`. Agent-31 returned CHANGES at the base. The exact final HEAD/tree are captured in `identity.log` and will be posted on PR #3835 after push, not embedded as a self-referential REPORT hash. Before push they are local evidence.

## What did not work

- Agent-31 proved that `hidden` plus the global `!important` rule left the typed root and its error text invisible even though DOM state and bundled model readiness passed. The scenario checked DOM presence rather than computed visibility.
- The active REPORT said completed local gates were NOT_RUN and used the original branch base instead of the reviewed merge base. Publication wording also implied the current allowlist could ship the typed app even though it stages neither `src/**` nor `dist/**`.
- New visibility assertions failed: dispose did not re-hide the root and the computed root display was `none`. The no-default-mount assertion passed immediately, confirming the default import was already inactive.

## What changed

- `mountTypedStudio` now sets `root.hidden = false` before initialization and `root.hidden = true` on dispose. The `src/main.ts` error path also unhides the root and activates its error text.
- The built-browser scenario now requires `getComputedStyle(root).display !== 'none'` and typed-only text, in addition to one visible shell, no legacy MCP, one typed MCP, bundled model readiness, and zero external requests.
- `main.test.ts` proves that no `typed=1` leaves the preview root untouched. Existing tests still reject incomplete initialization and preserve worker reuse.
- `kimi-52/ORDER.md` and `REPORT.md` now distinguish the original branch base `ab4a5b328` from reviewed merge base `e9bfd7751e3eb31736178d23289e8d99480bb0d0`, record executed gates, and use computed-visibility evidence.
- Publication wording is narrowed: the default source UI remains legacy and no publish ran; the current runtime allowlist still requires a later `dist` packaging unit. No final lifecycle migration, native packaging, desktop acceptance, or production publication is claimed.

## Measurements

- Focused bootstrap/main checks at that stage: **6/6 tests pass** (four bootstrap plus two main). The final cumulative suite is **384/384 tests across 46 files**.
- Complete typed suite: **384/384 tests across 46 files**.
- Plain-JS suite: **112/112 tests pass**.
- Historical pre-code-splitting build measurements retained from kimi-52, not the final head: TypeScript and application build PASS; main JS **284.83 kB** (gzip **86.39 kB**), CSS **118.84 kB** (gzip **20.47 kB**), worker **27.70 kB**, generated WASM **794.67 kB**, uncommitted. Final build dimensions are in the kimi-56 report.
- Historical kimi-52/53 built-browser preview, not the final request count: `shell=true`, `active=true`, `legacyHidden=true`, `legacyMcp=0`, `typedMcp=1`, `artifactReady=true`, `visible=true`, `typedText=true`, **7 requests, zero external**. Final browser evidence is in the kimi-56 report.
- Default static page, runtime page-load, desktop-client, and worker checks: all PASS; worker save/restore is bit-identical and desktop/worker remote request counts are zero.
- At the final committed repair state, harness build, `loop report --check`, `loop pr-check --local`, and all 11 `loop gates --full` checks pass.

## Checks run at the head

Red evidence includes the missing bootstrap, base-path, readiness, 404, and computed-visibility failures under `local/kimi/studio-mcp-connections/superpowers/checks/`. `kimi-52-precheck/` contains the earlier green pre-commit evidence. The intentional `mcp_client.js` D76 scope advisory is disclosed; shared notebook/FILEMAP advisories are unchanged. Agent-31 returned CHANGES for hidden-root visibility, stale evidence, base identity, and publication wording; the follow-up fresh exact-head review, PR push/update, GitHub checks, ready/queue/merge, and fresh-main verification are **NOT_RUN**. The PR uses `References #3500`; the issue remains open.

## Skills used

Working discipline, scope guard, solution integrity, red-first TDD, systematic bootstrap/build diagnosis, modern web application guidance, and verification before completion. A fresh independent exact-head delivery review follows the committed repair. No model compute, paid resource, credential, real user data, MCP endpoint, or production publish was used.

## Still open

This is a visible mounted development preview, not final lifecycle migration, production publication, desktop acceptance, or an autonomous/model capability claim. Legacy workflow services remain temporarily bridged. Later Phase A units replace those owners, finish native/shared packaging and acceptance, and publish only after review. The current runtime allowlist must stage `dist` before any typed publication.

## Plain summary

The typed Studio is now both mounted and visible, and it loads the bundled UOR demo. The default source UI remains legacy and no production publish has run. All declared product checks pass; no real endpoint, final migration, packaged-GUI, useful-chat, M7/M8, or accessibility-certification claim is made.

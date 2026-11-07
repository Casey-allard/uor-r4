# REPORT: render safe rich replies in the typed Studio, kimi-7

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Plan: [plan](../../../../superpowers/plans/2026-11-07-studio-safe-rich-replies.md). Base: `d0d7048f9066ae4786d98769121400edd48f98e3`. Exact delivery-head commit/tree and final logs are recorded on the PR rather than embedding a self-referential hash here.

## What did not work

- The typed transcript rendered every message as one plain paragraph. The new `RichMessage.test.tsx` suite first failed to resolve the missing component, and the `AgentComposer` integration failed to find its expected heading. The intended red evidence is in `red-rich-message.log`.
- The first integration green run read the pre-existing role `<strong>assistant</strong>` instead of the formatted `<strong>bold</strong>`. The assertion was scoped to `.rich-message` without changing the expected content or production behavior; all five renderer tests had already passed.
- The offline vendor Markdown helper returns HTML strings and the legacy live transcript uses `textContent`. Neither is a safe typed rendering boundary for this unit, so no HTML-string rendering or `dangerouslySetInnerHTML` was introduced.
- Tables, math, raw HTML, images, syntax highlighting, and custom Markdown extensions are not implemented. Relative/protocol-relative/`mailto:`/`javascript:`/`data:`/`vbscript:` destinations are not interactive in this unit.
- No live-page cutover, production publish, browser-data migration, useful conversation, model capability, M7/M8, production-readiness, or accessibility-certification result is claimed.

## What changed

- Added `RichMessage.tsx`, a line-by-line block parser for headings, unordered/ordered lists, quotes, fenced code, and paragraphs, plus inline code, strong text, and Markdown links. Every element is created through Preact; text values are escaped by the framework and no HTML string is evaluated.
- Only valid HTTP(S) destinations become anchors, with `target="_blank"` and `rel="noopener noreferrer"`. Raw HTML, unsafe/relative/malformed destinations, and unmatched markup stay literal. Unclosed fences render safely to end of input, language classes accept only bounded identifier characters, long/Unicode content is not truncated, and rendering performs no fetch or model request.
- Replaced only the typed transcript paragraph with `RichMessage`; copy/insert/download/retry still receive the exact original message and existing source/privacy/readiness behavior is unchanged.
- Exported the component and added typed-only styles for readable headings, lists, quotes, code blocks, overflow, and link focus. The legacy `index.css`, page, JavaScript, and offline vendor shim are untouched.
- Updated the typed README, product note, and notebook with the exact component scope and remaining parity work.

No dependency, legacy page/controller, engine, worker, tokenizer, `studio_api.js`, model artifact, publisher, deployment, `.github/**`, `Cargo.lock`, real browser data, owner-controlled path, or generated roadmap/issue file was changed. Cargo output stayed under `target/kimi-lexlean`.

## Measurements

- Focused renderer/component checks: **31/31 tests pass** (`5` renderer plus `26` AgentComposer).
- Complete typed suite: **341/341 tests pass across 42 files**.
- Plain-JS suite: **112/112 tests pass**.
- `npm run typecheck`: PASS with no TypeScript errors.
- `npm run build`: PASS; `dist/studio-foundation.js` **141.17 kB** (gzip **39.40 kB**) and `dist/studio-foundation.css` **4.85 kB** (gzip **1.41 kB**), uncommitted.
- `verify_studio_page.mjs --strict-router`: PASS; both strict pages clean.
- `verify_studio_page_load.mjs --strict-router`: PASS; planted traps detected, actual external request lists empty.
- `verify_desktop_client.mjs`: PASS; binary contract, 32 streamed tokens, 38 invoked commands, zero remote requests.
- `verify_studio_worker.mjs`: PASS; 32 expected tokens, bit-identical save/restore, guarded close/reset, six trap hits, zero remote requests.

## Checks run at the head

Red-first evidence is `red-rich-message.log`; green focused evidence is `green-rich-message.log`. Other local logs are `typecheck.log`, `full-vitest.log`, `build.log`, `plain-js.log`, and `legacy-*.log` under `web/studio/.superpowers/sdd/studio-safe-rich-replies/checks/`.

Final exact-head suite/gates, claim/report/local-policy checks, diff/status, and head/tree are rerun after this report and posted on the PR before queueing. Fresh-context review, green GitHub checks, protected merge, and fresh-main verification remain delivery requirements. The PR uses `References #3500`; the issue remains open.

## Skills used

Brainstorming within the approved typed-shell design, implementation planning, red-first TDD, systematic debugging of the integration selector, and verification before completion. A fresh independent exact-head delivery review follows the committed head. No model compute, paid resource, credential, real user data, or production publish was used.

## Still open

The typed library remains unmounted. Full real-browser/worker/mobile/light/dark/offline parity, runtime accessibility acceptance, browser-data migration, packaged-Tauri cutover, production publication, richer Markdown families, and useful model conversation remain outside this unit. #3500 remains open for its full completion contract.

## Plain summary

Typed assistant replies now render useful common Markdown without executing raw HTML or unsafe links. Message actions retain the exact source text, all declared checks pass, and the live transcript has not changed.

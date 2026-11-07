# REPORT: repair MCP review coverage, races, and cleanup evidence, kimi-14

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `74a662b9f55d0cc0b8c9c6b6a0d57697b5684b73`, tree `7b48ae8e790330b0c28f500a28b24bc318e5c31b`. Prior agent-29 review returned CHANGES at that head. The final exact delivery head/tree and logs will be recorded on PR #3820 after protected delivery; before push they exist only in local evidence.

## What did not work

- Agent-29 confirmed the original four repairs but found missing nested-argument and no-`disconnect` cleanup assertions, unsupported evidence wording, a reverse Call/Connect race, swallowed candidate-cleanup failure, hostile metadata handling, and incomplete unmount coverage.
- The new MCP suite first reproduced all three code defects with **15/18 passing**. The reverse race called the tool, candidate-cleanup failure was absent from status, and invalid metadata prevented the valid tool from rendering. The nested, no-`disconnect`, and active-unmount cases passed as coverage checks for already-present behavior.
- The first green attempt passed **17/18**: the reverse-race tool call was refused, but its disabled assertion ran before asynchronous replacement settled. Moving that assertion behind a rendered-state wait fixed the instrument; no additional product change was needed.
- The retained original red log shows **8/12 passing** with three product failures and one factory-instrument failure. The unsupported `41/44` wording and local-versus-PR evidence wording are corrected.

## What changed

- `call()` now refuses while `connectingRef` is active, matching the existing Connect-during-Call guard and preventing either direction from superseding the shared operation token.
- Tool metadata is filtered before rendering: null/non-string/empty names and non-string descriptions never reach Call or Attach. `attachmentName` also refuses non-string input defensively.
- Failed previous-session cleanup still preserves the old session/tools, but candidate release is now observed and candidate-cleanup failure is included in status alongside the previous failure.
- Exact nested object/array arguments are asserted through `callTool`. No-`disconnect` replacement and unmount, active-session disconnect on unmount, hostile metadata, reverse-race, and candidate-cleanup rejection now have explicit regression coverage.
- Evidence text records the measured red count and local-only logs truthfully; it no longer says unpushed evidence is already on the PR.

No dependency, legacy JavaScript/HTML, `mcp_client.js`, native MCP host, engine, worker, tokenizer, `studio_api.js`, model artifact, publisher, deployment, `.github/**`, `Cargo.lock`, real endpoint/credential/browser data, owner-controlled path, or generated roadmap/issue file was changed. Cargo output stayed under `target/kimi-lexlean`.

## Measurements

- Focused MCP checks: **18/18 tests pass**.
- Complete typed suite: **377/377 tests pass across 44 files**.
- Plain-JS suite: **112/112 tests pass**.
- `npm run typecheck`: PASS with no TypeScript errors.
- `npm run build`: PASS; `dist/studio-foundation.js` **171.74 kB** (gzip **47.85 kB**) and `dist/studio-foundation.css` **5.20 kB** (gzip **1.47 kB**), uncommitted.
- `verify_studio_page.mjs --strict-router`: PASS; both strict pages clean.
- `verify_studio_page_load.mjs --strict-router`: PASS; planted traps detected, actual external request lists empty.
- `verify_desktop_client.mjs`: PASS; binary contract, 32 streamed tokens, 38 invoked commands, zero remote requests.
- `verify_studio_worker.mjs`: PASS; 32 expected tokens, bit-identical save/restore, guarded close/reset, six trap hits, zero remote requests.
- Harness build and `loop report --check`: PASS. At the final committed repair state, `loop pr-check --local` and all 11 `loop gates --full` checks also pass.

## Checks run at the head

Red-first evidence is `kimi-14-red.log` under `local/kimi/studio-mcp-connections/superpowers/checks/`; focused green and typecheck evidence uses the same directory. A pre-commit local PR check refused because this REPORT was not yet committed; that was delivery-order instrumentation, not a product failure. At the final committed repair state, focused/full Vitest, typecheck, build, plain-JS, all four legacy checks, harness build, `loop report --check`, `loop pr-check --local`, and all 11 full gates pass. Fresh exact-head review, PR push/update, GitHub checks, ready/queue/merge, and fresh-main verification are **NOT_RUN**. The exact delivery HEAD/tree and final logs will be recorded on PR #3820 after push. The PR uses `References #3500`; the issue remains open.

## Skills used

Working discipline, scope guard, solution integrity, red-first TDD, systematic async/state diagnosis, modern web form guidance, and verification before completion. A fresh independent exact-head delivery review follows the committed repair. No model compute, paid resource, credential, real user data, MCP endpoint, or production publish was used.

## Still open

The typed library remains unmounted. Full real endpoint/native GUI acceptance, browser/Tauri mounting, real session/mobile/accessibility/offline parity, browser-data migration, production publication, and useful model conversation remain outside this unit. #3500 remains open for its full completion contract.

## Plain summary

The typed MCP panel now blocks both Connect/Call races, filters invalid metadata, verifies nested argument and cleanup behavior, and reports candidate-cleanup failures without dropping the old session. Focused checks are green; complete exact-head and delivery evidence remains to be collected.

# REPORT: add typed MCP connections and result attachment, kimi-12

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Plan: [plan](../../../../superpowers/plans/2026-11-07-studio-mcp-connections.md). Base: `ddb3b25138defbb98cb39dc3cc1b94a62036df16`. Exact delivery-head commit/tree and final logs are recorded on the PR rather than embedding a self-referential hash here.

## What did not work

- `StudioApp` had no MCP session or result-attachment path. The new `McpConnections.test.tsx` suite first failed to resolve the component, and the composed `StudioApp` test had no MCP server URL input. The intended red evidence is in `red-mcp-connections.log`.
- The existing controller already provides remote/native session selection, bounded list/call, replacement disconnect, and inert text. No second protocol client was created; the typed panel composes that existing factory and the existing attachment API.
- No real MCP endpoint, credential, browser-data migration, live-page cutover, production publish, useful conversation, model capability, M7/M8, production-readiness, or accessibility-certification result is claimed. Stdio is not expanded as a new workflow. Future cutover must also prevent the legacy MCP auto-boot from mounting beside this component.

## What changed

- Added typed session contracts and a re-export of the existing `connectRemote`. The typed Models panel requires explicit Connect before listing and explicit tool buttons before any call; there is no autonomous tool loop or model request.
- Tool arguments must be a JSON object. Invalid JSON/arrays are refused without calling or attaching. Tool descriptions and results render as inert text only.
- **Attach result** uses the existing `ui.addAttachment` with sanitized `mcp-<tool>.json`, exact result JSON, `kind: 'text'`, and `source: 'mcp'`. Existing `@` attachment options include it, verified through composed `StudioApp`.
- Credentials are memory-only and cleared after connection and on unmount, never logged or persisted. Successful replacement disconnects the previous session; failed replacement preserves it and its tools. Stale candidates are disconnected instead of listed or attached.
- Busy/error/cancellation states are explicit, and existing source/privacy/destination behavior remains green. Added adapter/component exports and restrained typed-only result styling.

No dependency, legacy JavaScript/HTML, `mcp_client.js`, native MCP host, engine, worker, tokenizer, `studio_api.js`, model artifact, publisher, deployment, `.github/**`, `Cargo.lock`, real endpoint/credential/browser data, owner-controlled path, or generated roadmap/issue file was changed. Cargo output stayed under `target/kimi-lexlean`.

## Measurements

- Focused MCP/composition checks: **38/38 tests pass** (`6` MCP panel, `32` StudioApp).
- Complete typed suite: **365/365 tests pass across 44 files**.
- Plain-JS suite: **112/112 tests pass**.
- `npm run typecheck`: PASS with no TypeScript errors.
- `npm run build`: PASS; `dist/studio-foundation.js` **170.97 kB** (gzip **47.64 kB**) and `dist/studio-foundation.css` **5.20 kB** (gzip **1.47 kB**), uncommitted.
- `verify_studio_page.mjs --strict-router`: PASS; both strict pages clean.
- `verify_studio_page_load.mjs --strict-router`: PASS; planted traps detected, actual external request lists empty.
- `verify_desktop_client.mjs`: PASS; binary contract, 32 streamed tokens, 38 invoked commands, zero remote requests.
- `verify_studio_worker.mjs`: PASS; 32 expected tokens, bit-identical save/restore, guarded close/reset, six trap hits, zero remote requests.

## Checks run at the head

Red-first evidence is `red-mcp-connections.log`; green focused evidence is `green-mcp-connections.log`. Other local logs are `typecheck.log`, `full-vitest.log`, `build.log`, `plain-js.log`, and `legacy-*.log` under `web/studio/.superpowers/sdd/studio-mcp-connections/checks/`.

Final exact-head suite/gates, claim/report/local-policy checks, diff/status, and head/tree are rerun after this report and posted on the PR before queueing. Fresh-context review, green GitHub checks, protected merge, and fresh-main verification remain delivery requirements. The PR uses `References #3500`; the issue remains open.

## Skills used

Brainstorming within the approved typed-shell design, implementation planning, red-first TDD, lifecycle/credential/async-state analysis, and verification before completion. A fresh independent exact-head delivery review follows the committed head. No model compute, paid resource, credential, real user data, MCP endpoint, or production publish was used.

## Still open

The typed library remains unmounted. Full real endpoint/native GUI acceptance, browser/Tauri mounting, real session/mobile/accessibility/offline parity, browser-data migration, production publication, and useful model conversation remain outside this unit. #3500 remains open for its full completion contract.

## Plain summary

Typed Studio now supports explicit remote MCP connection, one-call tool buttons, and exact result attachment into the existing composer. Credentials and stale sessions are cleaned up, hostile output stays inert, and the composed `@` path works. All declared checks pass, but no real endpoint or packaged application acceptance is claimed.

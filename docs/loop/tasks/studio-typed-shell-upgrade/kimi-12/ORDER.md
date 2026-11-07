# ORDER: add typed MCP connections and result attachment

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500), continuing the phased design after merged composer/rich-reply/desktop PRs #3817/#3818/#3819. Base: `ddb3b25138defbb98cb39dc3cc1b94a62036df16`.

## Goal

Add a typed remote MCP panel through the existing `mcp_client.js` contract. Allow explicit Connect and tool Call, then attach a result to the existing composer attachments. Preserve memory-only credentials, inert output, replacement cleanup, unavailable/failure states, and no autonomous tool loop or live-page cutover.

## Cancel probe

The existing controller already provides remote/native session selection, list/call, replacement disconnect, inert text, and host approval boundaries. The typed `StudioApp` has no MCP session or result-attachment path. If this cannot be composed through the existing `connectRemote`/`connectNativeStdio` contracts without duplicating protocol/host behavior or persisting credentials, stop rather than creating another client.

## Files in scope

- New `web/studio/src/adapters/mcpConnections.ts` for typed session contracts/re-export only.
- New `web/studio/src/components/McpConnections.tsx` and `McpConnections.test.tsx`.
- `web/studio/src/components/StudioApp.tsx` and `StudioApp.test.tsx` for composition and attachment/mention integration.
- `web/studio/src/main.ts`, `web/studio/src/studio.css`, `web/studio/README.md`.
- This task folder and `docs/superpowers/plans/2026-11-07-studio-mcp-connections.md`.
- `docs/products/web-client.md` and `docs/research/notebook/m4-serving.md` for scoped evidence.

No other paths are authorized. In particular, do not change legacy JavaScript/HTML, `mcp_client.js`, native MCP host, engine, worker, tokenizer, `studio_api.js`, model artifacts, publisher, deployment, `.github/**`, `Cargo.lock`, real endpoints/credentials/browser data, other labs' work, or owner-controlled/generated files. Add no dependency.

## Done when

- Red-first tests prove the missing panel, then cover explicit Connect/list/Call with exact URL/credential and JSON object arguments. No request occurs before Connect; no autonomous calls or model request occurs during listing/attachment.
- Tool descriptions/results stay inert. Invalid JSON/arrays are refused. `Attach result` uses the existing `ui.addAttachment` with a sanitized `mcp-<tool>.json` name, exact result JSON, `kind: 'text'`, and `source: 'mcp'`; existing `@` attachment options include it. No attachment exists before a result.
- Credentials are memory-only, cleared after connection and on unmount, never logged or persisted. Successful replacement disconnects the previous session; failed replacement preserves it. Stale candidates are disconnected, not listed or attached.
- Busy/unavailable/error/cancellation states are explicit, async actions disable themselves, and existing source/privacy/destination behavior stays green.
- Focused and complete Studio tests, typecheck/build, plain-JS tests, legacy static/runtime checks, claim/report/local-policy checks, and full gates pass. PR uses `References #3500`; #3500 stays open.
- Product notes and notebook distinguish typed component/adapter evidence from live endpoint/desktop acceptance. No production publish, real endpoint/credential, browser-data migration, cutover, useful-chat, M7/M8, production-readiness, or accessibility-certification claim.
- Fresh exact-head review, protected merge, and fresh-main verification are required before cleanup.

## Stop and report conditions

Stop before changing files outside this scope, adding a second protocol/host implementation, persisting credentials, executing tools automatically, or weakening a test/verifier. Stop after this component unit and protected delivery; do not mount the library, contact a real endpoint, invoke the publisher, or start another UI unit in this session.

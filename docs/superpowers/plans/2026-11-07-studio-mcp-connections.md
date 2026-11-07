# Typed MCP connections and result attachment implementation plan

## Context

The existing `mcp_client.js` controller already supports remote/native session selection, bounded list/call, replacement cleanup, and inert results. `StudioApp` has no typed MCP session or way to attach a tool result to the composer. This unit adds a typed panel through that contract rather than another protocol implementation.

## Approach

Add typed session contracts/re-export and an `McpConnections` panel. Explicit Connect lists tools; an explicit tool button parses a JSON object and calls once; an explicit Attach result button adds exact JSON through the existing UI API. Credentials are memory-only and cleared after connection/unmount. Replacement cleanup, stale-candidate disconnect, busy states, invalid JSON, and inert output follow the existing behavior.

## Steps

1. Commit the ORDER alone, then this plan. Install pinned packages with `npm ci`.
2. Add red-first `McpConnections.test.tsx` for explicit connect/list/call/attach, hostile text, invalid arguments, credential clearing, replacement success/failure, and stale/unmount cleanup. Add a composed `StudioApp` test proving an attached result appears in existing `@` options. Run and retain intended failures.
3. Implement typed contracts and panel. Use injected session factories only in tests; production defaults use the existing `connectRemote`. Do not add protocol calls, autonomous loops, credential persistence, or a second host.
4. Integrate the panel into the existing Models section, adapter/component exports, and restrained typed-only result styling. Preserve all existing destinations and model/source/privacy behavior.
5. Rerun focused tests, then `npm test`, `npm run typecheck`, `npm run build`, `node --test assets/js/*.test.mjs`, and all four legacy checks. Keep Cargo output under `target/kimi-lexlean`.
6. Update typed README, product note, notebook, and REPORT with exact counts, red-first evidence, head/tree, command/credential boundaries, and remaining limits. Run claim/report/local-policy/full gates and review the scoped diff.
7. Commit, push a draft PR with `References #3500`, obtain fresh exact-head review, repair if needed, then protected queue and fresh-main verification. Preserve evidence and clean only this worktree's generated outputs. Do not publish or start another unit in this session.

## Assumptions

- This unit covers remote Streamable HTTP and native-remote selection already present in `connectRemote`; stdio is not expanded or claimed as a new workflow.
- Tool arguments must be a JSON object. Result attachment is always a separate explicit action, never automatic after a call.
- Sanitized filenames contain only alphanumerics, underscore and dash. Result JSON remains exact; no credential or endpoint is included in attachment metadata.
- This is typed-component evidence only. Real endpoint/native GUI acceptance, full browser/Tauri mounting, migration, accessibility acceptance, and publication remain under #3500.

## Verification contract

The report must include missing-panel red-first evidence, exact connection/attachment/negative cases, full green counts, exact delivery head/tree, and explicit non-goals. Injected sessions are not a real MCP endpoint, desktop GUI, accessibility, or model-capability claim.

# ORDER: repair MCP call, attachment, and cleanup contracts

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `ddb3b25138defbb98cb39dc3cc1b94a62036df16`. Prior reviewed head: `327da602187fa3d8297d059085ac53bfdd05c3b7`, tree `a57ba2444b83b99dd8474afb523fe826110243e1`, PR #3820 draft.

## Finished steps

- kimi-12 added explicit remote MCP Connect/list/Call and exact result attachment; all 365 typed tests, 112 plain-JS tests, legacy checks, and 11 gates passed.
- Fresh review returned CHANGES for Connect-during-Call busy-state wedge, unconditional attachment success, previous-session cleanup failure, and overstated real-factory evidence. It also identified missing nested/malformed arguments, hostile-name, unmount-credential, and default-browser cleanup coverage.

## Goal

Prevent Connect during Call, report attachment only when the existing API actually retains it, preserve previous sessions when cleanup fails, and prove the actual remote factory plus every earlier behavior. Qualify disconnect cleanup to sessions that provide that hook; the direct-browser client has no disconnect method.

## Cancel probe

Reproduce a slow call followed by Connect, an attachment refusal, and a previous-session disconnect rejection before changing source. Stop rather than reporting success without retained evidence or dropping a live previous session.

## Files in scope

- `web/studio/src/components/McpConnections.tsx` and `McpConnections.test.tsx`.
- `web/studio/README.md`, `docs/products/web-client.md`, and `docs/research/notebook/m4-serving.md` only for corrected counts/evidence.
- This repair folder.

No other paths are authorized. No legacy client/HTML, native MCP host, dependency, engine, worker, tokenizer, `studio_api.js`, model artifact, publisher, deployment, `.github/**`, `Cargo.lock`, real endpoint/credential/browser data, owner-controlled path, or generated roadmap/issue edit.

## Done when

- Connect is disabled during Call and recovers afterward. A refused attachment shows an error and no Attached state; an accepted attachment is verified through the existing API before success is reported.
- Previous-session cleanup failure preserves the old session and releases the candidate. Sessions without a disconnect hook have an explicit no-op cleanup contract, matching the direct-browser client; cleanup claims are qualified accordingly.
- Tests cover the actual remote factory through mocked HTTP JSON-RPC, nested/malformed arguments, hostile tool names, unmount credential handling, refusal, cleanup failure, and all original behavior. No real endpoint or credential is used.
- Focused/full Studio checks, plain-JS tests, legacy checks, claim/report/local-policy checks, and full gates pass at the new exact head. Existing disclosed D76 advisories remain disclosed; no new warning is introduced.
- Fresh exact-head review approves before ready/queue. PR keeps `References #3500`; no live cutover, publish, migration, useful-chat, M7/M8, production-readiness, or accessibility-certification claim.

## Stop and report conditions

Stop before changing files outside this scope or weakening a test/verifier. Stop after this repair and protected delivery; do not start another UI unit in this session.

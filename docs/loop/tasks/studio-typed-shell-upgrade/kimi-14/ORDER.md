# ORDER: repair MCP review coverage, races, and cleanup evidence

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `74a662b9f55d0cc0b8c9c6b6a0d57697b5684b73`, tree `7b48ae8e790330b0c28f500a28b24bc318e5c31b`. Prior reviewed head is the same commit; PR #3820 remains draft at `327da602187fa3d8297d059085ac53bfdd05c3b7` and was not pushed or queued.

## Finished steps

- kimi-12 added the typed MCP panel. kimi-13 added Connect-during-Call guards, retained-attachment verification, previous-session cleanup handling, and a real loopback JSON-RPC factory fixture.
- Fresh agent-29 review confirmed those four repairs and green exact-head checks, but returned CHANGES for missing nested-argument and no-`disconnect` cleanup assertions, unsupported evidence wording, a reverse Call/Connect race, swallowed candidate-cleanup failure, hostile metadata, and incomplete unmount coverage.
- The retained red log shows **8/12 passing**: three product failures plus one factory-instrument failure. It does not support the earlier `41/44` wording.

## Goal

Close every agent-29 finding: nested arguments, no-`disconnect` cleanup, reverse-race prevention, candidate-cleanup errors, safe metadata, active-session unmount, and truthful evidence. Then obtain fresh approval and deliver.

## Cancel probe

Run the new MCP suite before source changes. The reverse-race, candidate-cleanup, and hostile-metadata tests must fail for the intended product reasons; nested/no-`disconnect`/active-unmount assertions are coverage additions for already-present behavior and must not be described as red-first product failures. Stop if a test errors instead of reproducing its named defect.

## Files in scope

- `web/studio/src/components/McpConnections.tsx` and `McpConnections.test.tsx`.
- `web/studio/README.md`, `docs/products/web-client.md`, and `docs/research/notebook/m4-serving.md` only for corrected counts/evidence.
- `docs/loop/tasks/studio-typed-shell-upgrade/kimi-13/REPORT.md` only to correct the unsupported red count and local-versus-PR evidence wording.
- This `kimi-14` repair folder.

No other paths are authorized. No legacy client/HTML, `mcp_client.js`, native MCP host, dependency, engine, worker, tokenizer, `studio_api.js`, model artifact, publisher, deployment, `.github/**`, `Cargo.lock`, real endpoint/credential/browser data, owner-controlled path, or generated roadmap/issue edit.

## Done when

- Exact nested object/array arguments reach `callTool` unchanged; malformed/non-object inputs remain refused.
- Sessions without `disconnect` replace and unmount without throwing or inventing a call; active sessions supplying the hook are disconnected on unmount.
- `call()` refuses while `connectingRef` is active, so neither Call/Connect direction can latch busy state. Previous-cleanup and candidate-cleanup failures are both surfaced while the old session/tools stay selected.
- Null/non-string tool names and invalid descriptions cannot throw or render actions; only valid metadata reaches Call/Attach. Hostile string names remain sanitized.
- The unsupported `41/44` claim becomes the measured 8/12 red result with three product failures plus one instrument failure; local evidence is not described as already present on the PR.
- Focused/full Studio checks, plain-JS tests, legacy checks, claim/report/local-policy checks, and full gates pass at the new exact head. Existing disclosed D76 advisories remain disclosed; no new warning is introduced.
- A fresh exact-head review approves before ready/queue. PR keeps `References #3500`; no live cutover, publish, migration, useful-chat, M7/M8, production-readiness, or accessibility-certification claim.

## Stop and report conditions

Stop before changing files outside this scope or weakening a test/verifier. Do not add a dependency, contact a real endpoint/credential, or hide an unresolved failure. Stop after this repair and protected delivery; do not start another UI unit in this session.

# ORDER: isolate MCP timeout fixtures from connection setup, kimi-77

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500), predecessor PR #3879. Base: `759da85854b76d8eacdd64719824461ca6a9e7c2`. Scope: `src-tauri/tests/mcp_protocol.rs`, descendant-fixture test code in `src-tauri/src/lib.rs`, and this task folder only.

## Evidence and decision review

The full default suite at `759da8585` passed 9 library, 3 CLI, and 21 control tests, then `remote_auth_timeout_and_size_caps_fail_explicitly` failed: the intended 15 ms timeout closed a response body early, and the next size-cap connection failed during initialization. Earlier runs also saw an unexpected HTTP 401. The fixture reads with a one-second timeout, but records and credential-checks a partial request instead of first proving that its headers and complete Content-Length body arrived. Its response delay applies to initialization, although the behavior under test is a hanging tool call. No production timeout defect has been established.

Keep all production behavior and short hanging-call assertions. Refuse an incomplete fixture request rather than misclassifying it as a credential failure. Apply the deliberate delay to `tools/call`, after normal initialization and notification. Retain the 15 ms and 25 ms limits for the intended hanging calls and verify connection, cleanup, and size-cap behavior exactly.

The next complete default run also exposed two process fixtures that can expire before their descendant PID file exists. Synchronize the descendant's readiness through a fixture pipe, allow one second for process startup plus the timeout, and retain the explicit timeout and no-surviving-descendant assertions. These are test-only repairs; no production process behavior or timeout limit changes.

## Done when

- The remote timeout test proves initialization/notification complete, then its 15 ms tool call times out explicitly; the 1024-byte response cap remains enforced.
- Partial HTTP input is never recorded as a complete authorized/unauthorized MCP request.
- Complete default and GUI suites, acceptance, records, fresh exact-head review and protected delivery pass. If stdio connection timing reproduces a failure, diagnose and isolate it separately without changing its 25 ms hanging-call assertion.

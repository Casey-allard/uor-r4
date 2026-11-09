# ORDER: bind the final Studio delivery evidence and records, kimi-79

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `3c7176d5f37c2b98329877e6c6388fb63ed8b88e`; merge base `0895aa65ff2f63ed184e78e81c28e67ffa3f34aa`. Fresh reviewer agent-62 returned CHANGES; PR #3879 stays unqueued.

## Findings and decision review

The mixed-case production guard is sound, but the delivery records must distinguish source/acceptance head `d39953a7d`, current record-only head and tree, and the merge base. Preserve exact full default/GUI and local-policy logs at the final head, and copy the full-gate log and directory byte-identically into the main checkout's durable `local/kimi/studio-final-acceptance/` area before any worktree cleanup.

Assert the entire expected MCP hanging-call JSON-RPC body and relevant protocol/session headers, not just a method name and bearer credential. Preserve all original timeout/refusal assertions and expected disconnect handling. Update the desktop count, current-state, notebook and README to current `kimi-78/79` evidence while retaining historical kimi-71 records as historical.

## Files in scope

- `src-tauri/tests/mcp_protocol.rs` for complete request/envelope/header assertions only; `src-tauri/tests/desktop_client_controls.rs` only for default-root mixed-case coverage.
- `docs/products/desktop-client.md`, `docs/integration/current-state.md`, `docs/research/notebook/m4-serving.md`, `web/studio/README.md`, and `docs/loop/tasks/studio-typed-shell-upgrade/kimi-79/**`.
- Ignored final-head logs and durable copies under this worktree's and the main checkout's `local/kimi/studio-final-acceptance/`.

No production source changes, model/API/engine/worker/tokenizer edits, real endpoint/profile/credential, production publication, signed release, or useful-chat claim. Do not modify the separate interactive-terminal branch during this repair.

## Done when

- Exact complete request body and headers are asserted; full default/GUI suites and full repository gates pass with retained final-head logs.
- `d39953a7d` source/acceptance identity, final record-only head/tree, and `0895aa65f` merge base are unambiguous in the report, PR body, and evidence.
- Active desktop/current-state/notebook/README records agree with the current counts and report; old GUI evidence remains historical.
- Fresh exact-head review approves, GitHub checks pass, protected merge completes, and fresh-main verification finishes before cleanup.
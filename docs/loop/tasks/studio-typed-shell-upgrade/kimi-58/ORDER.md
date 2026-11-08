# ORDER: close agent/session verification and evidence defects, kimi-58

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `7e9b83be0700a897f042241d13591b0ee70967bc`, tree `8c57b7f9ec19a7588f33fc2e333fbe69e8e08d03`; merge base: `8903c5551a5e9c3953bd6a2eab1322c7e6c276c2`. Agent-36 returned CHANGES and blocked ready/queue on PR #3843.

## Findings to repair

- Create accepts a no-op boolean and old active session; duplicate accepts the source ID; remove accepts an unchanged target. Verify operation effect against before/after snapshots.
- Legacy rename truncates to 80 code units, so exact-title verification fails after a successful write. Normalize and verify the actual bounded title.
- Direct active-ID reads can throw from Send, and synchronous stop completion can clear activeGeneration before interruption. Fail closed and preserve the stopped snapshot while synchronizing identity.
- Browser requests are captured before typed interactions and `42` is not bound to the exact tool trace. Sample after all interactions and require the exact response/trace.
- Correct remaining impossible doubles, reducer active-ID/catalog invariant, failed-start session assertion, fresh-main NOT_RUN wording, per-session versus bulk-clear wording, and the accidental command-group drift.

## Cancel probe

Add red-first tests for existing-active no-op create, source-ID duplicate, unchanged delete, 81-code-unit rename, throwing active-ID read, synchronous stopped event, absent catalog identity, and failed-start session identity. Add browser-instrument review proving request capture follows interactions and exact-trace checks reject a generic `42`. Stop if a test passes through the old defective path.

## Files in scope

- `web/studio/src/adapters/agentSession.ts`, `agentSession.test.ts`, `generation.ts`, and `generation.test.ts`.
- `web/studio/src/state/lifecycle.ts` and `lifecycle.test.ts` only for verified catalog/interruption behavior.
- `web/studio/src/components/SessionList.tsx`, `SessionList.test.tsx`, `StudioApp.tsx`, and `StudioApp.test.tsx` only for bounded rename input and correction of command grouping/doubles.
- `web/studio/verify_studio_page_load.mjs` for post-workflow request capture and exact-response assertions.
- `web/studio/README.md`, `docs/products/web-client.md`, `docs/research/notebook/m4-serving.md`, `docs/loop/tasks/studio-typed-shell-upgrade/kimi-57/REPORT.md`, and this repair folder for precise evidence/claims.

No real-data migration, storage change, production cutover/publication, native/package work, model/worker/tokenizer/`studio_api.js` change, dependency, external endpoint/credential, Phase B compute, useful-chat, M7/M8, or accessibility-certification claim.

## Done when

- Create requires a different, newly cataloged active session; duplicate requires a new non-source entry; remove requires target absence and a verified resulting active session. Resume remains valid for the current session.
- Rename uses the legacy 80-code-unit bound and verifies the actual stored title. Send/read failures return false without model invocation or uncaught exceptions.
- Synchronous stop completion cannot lose the stopped snapshot or prevent active-ID synchronization. Active IDs must exist in the catalog, and failed starts retain session identity.
- Browser assertions require the exact R4 arithmetic response/trace and capture external/raw-TypeScript/MCP state after every typed interaction.
- All focused/full checks, four legacy checks, report/local-PR checks, and full gates pass at the new exact head. Fresh review approves before ready/queue; `References #3500` remains.

## Stop and report

Stop before weakening a test, accepting boolean success without operation effect, widening scope, or publishing. Continue protected delivery only after fresh approval.

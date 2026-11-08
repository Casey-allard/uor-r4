# ORDER: finish agent/session idempotence and evidence fidelity, kimi-59

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `506b45afc79e3f7fb91862dca900f5756eaf2c16`, tree `e934c9cde26c68beeb7473eb30cf7c96576244ff`; merge base: `f3a3b7869dc7644482b0704465eef24be962d3bc`. Agent-37 returned CHANGES and blocked ready/queue on PR #3843.

## Findings to repair

- Five navigation commands moved out of the base `Navigation` group. The regression test checked only Agent. Restore all six and assert every ID.
- Repeated `session/interrupted` uses `lastGeneration` again and increments revision. Make it idempotent and test the second identical action.
- `getMode` can throw outside the generation failure guard. Return false before model invocation.
- Correct remaining fresh-main/read-path evidence wording and historical measurement labels. Restore exact transcript equality, not just presence of the exact response.
- Replace remaining no-op shared session doubles and active-delete fixtures with real catalog/replacement behavior. Distinguish a valid delete with no active session from failure without weakening effect verification.

## Cancel probe

Add red-first checks for all six navigation IDs, repeated interruption, throwing `getMode`, exact transcript equality, and an inactive/final deletion with no active session. Confirm the existing fresh-main wording mismatch directly. Do not patch only test expectations.

## Files in scope

- `web/studio/src/state/lifecycle.ts` and `lifecycle.test.ts`.
- `web/studio/src/adapters/generation.ts`, `generation.test.ts`, `agentSession.ts`, and `agentSession.test.ts`.
- `web/studio/src/components/StudioApp.tsx`, `StudioApp.test.tsx`, `SessionList.test.tsx`, and `studioTestFixtures.ts` for grouping and realistic doubles only.
- `web/studio/verify_studio_page_load.mjs` for exact transcript equality and completion-before-resume.
- `docs/loop/tasks/studio-typed-shell-upgrade/kimi-57/REPORT.md`, `kimi-58/REPORT.md`, `docs/research/notebook/m4-serving.md`, and this repair folder for evidence/citations.
- `web/studio/README.md` and `docs/products/web-client.md` only if final counts/scope change.

No real-data migration, production cutover/publication, native/package work, model/worker/tokenizer/`studio_api.js` change, dependency, external endpoint/credential, Phase B compute, useful-chat, M7/M8, or accessibility-certification claim.

## Done when

- All six navigation commands retain the base group and are found together by `Navigation` search.
- Repeating an identical interruption does not change revision or state. Active-ID/catalog and synchronous-stop behavior remain green.
- `getMode` and active-ID read failures return false without model invocation.
- Browser restoration equals the original transcript exactly after generation completes; no duplicated/extra content passes.
- Shared doubles model real create/select/delete replacement semantics. Delete verifies target absence and returns the resulting active summary, or `true` only for a verified no-active result; failure returns null.
- Reports distinguish historical measurements, current exact-head logs, NOT_RUN steps, and actual advisories. Full checks pass at the new exact head and fresh review approves before delivery.

## Stop and report

Stop before changing unrelated UI groups, weakening assertions, hiding a no-op deletion, or publishing. Continue protected delivery only after fresh approval.

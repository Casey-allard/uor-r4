# ORDER: establish one agent/session authority, kimi-57

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `084a7f92e76926f917e1d9ba0d82f01847fad3aa`, tree `776c9bcca56a2c5ad9bc5f02f5c5624b5a21806e`. PR #3835 delivered the mounted preview; this is the next Phase A unit, not a repair of that head.

## Measured gap

`SessionList` and `StudioApp` call session mutations directly, while `LifecycleStore` has no active-session identity or session catalog. A boolean `newSession`/`selectSession` result is treated as activation without verifying the active ID. Generation pins source/privacy but not its session, and the composer key reads legacy identity outside the store. The approved typed-shell design requires one lifecycle owner, not a second storage engine.

## Goal

Add one typed agent/session authority over the existing `SessionApi`. Synchronize the lifecycle store from actual legacy reads, verify create/resume results, bind generation to its session, stop rather than relabel a run after an unexpected session change, and route composed session/create/resume/send actions through it. Prove a real browser create/resume flow with an actual UOR arithmetic response.

## Cancel probe

Before production implementation, add focused tests for a missing authority, verified active-ID mismatches, generation/session binding, and unexpected session changes. The new adapter import must fail before it exists, and identity assertions must fail against the old lifecycle store. Do not infer correctness from a boolean legacy result or an unmounted adapter.

## Files in scope

- `web/studio/src/adapters/agentSession.ts` and `agentSession.test.ts` (new authority and focused behavior).
- `web/studio/src/state/lifecycle.ts` and `lifecycle.test.ts` for verified session identity/catalog and generation binding.
- `web/studio/src/adapters/generation.ts` and `generation.test.ts` only to capture/pin session identity.
- `web/studio/src/components/SessionList.tsx`, `SessionList.test.tsx`, `StudioApp.tsx`, and `StudioApp.test.tsx` to route through the authority and expose actual active state.
- `web/studio/src/adapters/workflow.ts` only to reuse the lifecycle session-summary type without a dependency cycle.
- `web/studio/src/main.ts` only to export the authority.
- `web/studio/verify_studio_page_load.mjs` for real built-browser create/resume/UOR-arithmetic evidence.
- `web/studio/README.md`, `docs/products/web-client.md`, `docs/research/notebook/m4-serving.md`, `docs/roadmap/items/studio-typed-shell-upgrade.md`, and this task folder for actual scope/evidence.

No real-data migration, schema/key change, credential duplication, cutover/publication, native/package work, engine/worker/tokenizer/`studio_api.js` change, dependency, external endpoint/credential, Phase B compute, useful-chat, M7/M8, or accessibility claim.

## Done when

- One store owns verified active-session ID, catalog, revision, phase, and generation snapshots. Initial synchronization reflects the legacy source; list/session changes do not invent activity.
- Create/resume re-read the active ID and catalog and reject false/mismatched results without activating them. Rename/duplicate/delete/export use the same authority. Active state is observable in the composed UI.
- Generation snapshots pin session ID. A changed active session during a run invokes legacy stop, records a stopped outcome, synchronizes identity, and never applies the old run to the new session. Existing source/privacy and stale-generation guards remain green.
- The real built browser creates sessions, obtains an actual UOR arithmetic answer, resumes away and back, and preserves the correct visible transcript. Claims exclude useful conversation.
- Focused/full tests, plain-JS, typecheck/build, four legacy checks, report/local-PR checks, and full gates pass at the exact head. Fresh exact-head review precedes protected delivery.

## Stop and report

Stop before adding a second session store, rewriting persistence, hiding a failed legacy command behind a boolean, or claiming migration/cutover/desktop acceptance. Continue Phase A after protected delivery.

# ORDER: close remaining agent/session fail-open and fidelity gaps, kimi-60

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `0f6d5c852b01bb60fc57e409502df42f4b4a849f`, tree `820b5aa8f311352eb99e4de7bd15f9471e9b21c3`; merge base: `56ef8a4feed0fec94b5954ebbfbfd6dca403af68`. Agent-38 returned CHANGES and blocked ready/queue on PR #3843.

## Findings to repair

- `duplicate()` calls the mutating legacy API before checking whether its pre-read succeeded. Prevalidate it and the related resume/rename paths before any mutation.
- Generation completion leaves catalog counts/previews stale until another session action. Refresh from the real legacy catalog on completion and dispose the listener correctly.
- Session-change interruption ignores `canStop === false`. Refuse that interruption without stopping or relabeling the prediction; synchronize after its actual completion.
- SessionList delete success lacks observable replacement assertions. Repeated interruption was already inert in the red run and must be reported as characterization, not a reproduced repair.
- Network capture has no post-evaluation settle window. Add one and require a delayed external mutation to reject.
- Correct remaining inconsistent StudioApp doubles, no-active delete-result wording, historical command-group/advisory/measurement claims, and the prior 384-test label.

## Cancel probe

Add red-first checks proving duplicate/resume/rename cannot mutate after a failed pre-read, generation completion refreshes the catalog, `canStop === false` does not stop or relabel, and SessionList confirms the actual replacement. Add a delayed external request to verifier mutation evidence. Correct the repeated-interruption record without inventing a defect.

## Files in scope

- `web/studio/src/adapters/agentSession.ts`, `agentSession.test.ts`, and `generation.ts` only if listener/disposal needs a narrow adjustment.
- `web/studio/src/state/lifecycle.ts` and `lifecycle.test.ts` only for interruption guard/evidence clarity.
- `web/studio/src/components/SessionList.test.tsx`, `StudioApp.test.tsx`, and `studioTestFixtures.ts` only for realistic assertions/doubles.
- `web/studio/verify_studio_page_load.mjs` for post-evaluation settling and delayed-mutation evidence.
- `web/studio/README.md`, `docs/products/web-client.md`, `docs/research/notebook/m4-serving.md`, `docs/loop/tasks/studio-typed-shell-upgrade/kimi-57/REPORT.md`, `kimi-58/REPORT.md`, `kimi-59/REPORT.md`, and this repair folder for precise claims/citations.

No real-data migration, production cutover/publication, native/package work, model/worker/tokenizer/`studio_api.js` change, dependency, external endpoint/credential, Phase B compute, useful-chat, M7/M8, or accessibility-certification claim.

## Done when

- Failed pre-reads prevent every legacy mutation, completion updates real catalog metadata, and listeners dispose without duplicates. A non-stoppable run is never stopped/relabelled during session change; actual completion can synchronize afterward.
- SessionList deletion proves dialog closure, no error, replacement active ID, and replacement row. Shared StudioApp active/catalog doubles are consistent.
- Browser capture includes requests scheduled shortly after evaluation and still rejects them; normal requests remain local and restored transcripts exact.
- Reports distinguish real defects from idempotence characterization, use actual advisories, label historical totals, and describe no-active delete results consistently. Full checks and fresh exact-head review pass before protected delivery.

## Stop and report

Stop before a mutating call after failed pre-read, a stopped/non-stoppable false claim, weakened assertion, or scope expansion. Continue only after fresh approval.

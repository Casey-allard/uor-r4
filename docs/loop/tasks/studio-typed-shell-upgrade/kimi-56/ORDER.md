# ORDER: exercise the real readiness failure and close preview evidence gaps, kimi-56

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `a59e5916ba4683134ac7114226a05fd10b128c58`, tree `56f855629f67b2e4d90512cd10c6e04aff466a30`; reviewed merge base: `e9bfd7751e3eb31736178d23289e8d99480bb0d0`. Agent-34 returned CHANGES and blocked push/queue.

## Finished steps

- kimi-52 through kimi-55 mounted the development preview, made it visible, gated the plain-JavaScript entry, and asserted default/typed runtime errors.
- Agent-34 confirmed the entry gating, worker reuse, mounted browser evidence, and recorded product checks, but rejected the mocked initialization failure, incomplete readiness checks, incomplete advisory disclosure, and notebook citation. It also identified weaker browser assertions and test wording.

## Goal

Run the real `mountTypedStudio` and `main.ts` catch with incomplete legacy workflow methods and bounded time. Refuse initialization until the methods read by the initial composed render exist. Strengthen default/typed browser assertions, make test names match their evidence, disclose the exact scope advisory, and correct the evidence chain without changing preview scope.

## Cancel probe

Before production changes, run the revised entry/bootstrap tests with a partial `__studioUi` that supplies `getSettings` but lacks session methods. The tests must fail against the old readiness predicate rather than succeed through a render mock or a different render exception. The entry test must assert the actual workflow-timeout message. If the real path errors for an unrelated missing dependency, repair the instrument before classifying the result.

## Files in scope

- `web/studio/src/main.test.ts` for the real entry/readiness/catch path.
- `web/studio/src/bootstrap.ts` and `src/bootstrap.test.ts` for readiness of initial-render workflow methods, complete fixtures, and accurate component evidence.
- `web/studio/verify_studio_page_load.mjs` for exact typed-shell count, default raw-TypeScript request rejection, and existing runtime-error assertions.
- `web/studio/README.md` for the precise readiness and current-count evidence.
- `docs/research/notebook/m4-serving.md` for the corrected kimi-52–56 chain and evidence scope.
- `docs/loop/tasks/studio-typed-shell-upgrade/kimi-52/REPORT.md` and `kimi-53/REPORT.md` only to label historical build/request measurements and current totals.
- `docs/loop/tasks/studio-typed-shell-upgrade/kimi-54/ORDER.md` and `REPORT.md`, and `kimi-55/REPORT.md`, for actual review state, advisories, and evidence boundaries.
- This `kimi-56` repair folder.

No publisher invocation, deployment, Tauri, engine, worker, tokenizer, `studio_api.js`, model artifact, `.github/**`, `Cargo.lock`, real endpoint/credential/browser-data migration, owner-controlled file, generated `docs/roadmap/issues.json`, dependency, second model provider, production publish, useful-chat, M7/M8, or accessibility-certification claim. Do not alter the roadmap item's canonical scope; disclose the existing `typed_entry.js` advisory explicitly.

## Done when

- `main.test.ts` does not mock `mountTypedStudio`; with partial legacy methods it advances a real pending readiness timer, reaches the real `main.ts` catch, and proves the timeout message, `hidden=false`, `data-active=true`, and computed display.
- Readiness checks the workflow methods synchronously consumed by the initial composed render: workspace tabs/active file, settings/attachments, session list/active ID, and conversation messages. Partial session APIs reject before Preact render; complete fixtures still reuse the existing worker without terminating it.
- The browser check requires exactly one typed shell and explicitly rejects default requests for raw TypeScript or `src/main`; both default and typed pages retain zero-runtime-error assertions and counts.
- Unit-test names and assertions distinguish passing the real component to a mocked Preact renderer from actual browser DOM rendering. Current evidence counts are explicit, and historical build sizes/request counts are labeled head-specific.
- Focused/full Studio checks, plain-JS, all four legacy checks, report/local-PR checks, and full gates pass at the new exact head. A fresh exact-head reviewer approves before ready/queue. The report names Agent-34's CHANGES and all current advisories.
- PR keeps `References #3500`; no production publish, final migration, Tauri acceptance, or model-capability claim.

## Stop and report

Stop before mocking the readiness failure again, weakening a test, changing an out-of-scope file, or using a stale build. Stop after fresh approval and protected delivery of this unit, then continue Phase A.

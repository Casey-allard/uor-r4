# ORDER: preserve non-stoppable runs on remount and finish evidence, kimi-63

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `95ac4cdf8d6ced7f7fc1cc22f84bc18df7b38d43`, tree `c5de27ac144b5fdbb575d0a641dd1a80c2c779c9`; merge base: `2ef27b81fc7adc7910ab9466d8305dff44f45dbf`. Agent-40 returned CHANGES and blocked ready/queue on PR #3843.

## Findings to repair

- Adapter remount skips `stopGeneration()` when `canStop === false` but still dispatches `stopped`, clearing a live prediction and allowing a second send. Only stoppable remounts may use the synthetic stopped transition.
- The completion listener has no post-dispose assertion. Prove it cannot refresh catalog metadata after disposal.
- The replacement-row wrong-row mutation lacks an explicit log. Add a mutation that marks the old row and proves the replacement selector rejects it.
- Notebook omits direct kimi-60/kimi-61 links. Delete failures misleadingly always say “Stop the current reply first,” including verification/read failures.

## Cancel probe

Add a non-stoppable remount regression that must remain `generating` with its active prediction and no stop call, a post-dispose completion assertion, and a wrong-row marker mutation. The remount test must fail against the current adapter before the guard changes.

## Files in scope

- `web/studio/src/adapters/generation.ts` only to suppress synthetic stop for `canStop === false`.
- `web/studio/src/adapters/agentSession.test.ts` for remount and disposal evidence.
- `web/studio/src/components/SessionList.tsx` and `SessionList.test.tsx` for truthful delete errors and wrong-row mutation evidence.
- `web/studio/README.md`, `docs/products/web-client.md`, `docs/research/notebook/m4-serving.md`, `docs/loop/tasks/studio-typed-shell-upgrade/kimi-62/REPORT.md`, and this repair folder for actual counts/claims.
- `web/studio/src/adapters/generation.test.ts` only if a lower-level remount assertion is useful.

No real-data migration, production cutover/publication, native/package work, model/worker/tokenizer/`studio_api.js` change, dependency, external endpoint/credential, Phase B compute, useful-chat, M7/M8, or accessibility-certification claim.

## Done when

- A non-stoppable remount remains `generating` with its active session and no stop call; a real completion still finishes it. Stoppable remount behavior is unchanged.
- Completion after disposal cannot update the catalog. The wrong-row mutation is rejected by the replacement-ID/active-marker assertion.
- Delete errors distinguish generation from verification/read failures, notebook links the full repair chain and current zero-error totals, and all checks plus fresh exact-head review pass before protected delivery.

## Stop and report

Stop before clearing a live non-stoppable run, weakening evidence, changing unrelated UI, or publishing. Continue only after fresh approval.

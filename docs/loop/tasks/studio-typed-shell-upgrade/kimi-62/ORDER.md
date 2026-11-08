# ORDER: bind replacement-row and final-instrument evidence, kimi-62

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `02804377ee71d0fdc7509e900d81619645dc2fa4`, tree `f07340c38bdbd9227b31f278d9926950fb33b493`; merge base: `2ef27b81fc7adc7910ab9466d8305dff44f45dbf`. Agent-39 returned CHANGES and blocked ready/queue on PR #3843.

## Findings to repair

- SessionList's delete test accepts any row titled `New conversation`; duplicate titles and a wrong active marker can pass. Expose each real session ID on its row and assert the replacement ID is the active row.
- The notebook stops at kimi-60 and lacks zero-unhandled-error evidence. The kimi-61 report does not state the final clean result and mislocates the focused log.
- Roadmap text says every changed-session run stops, ignoring the `canStop === false` refusal. Qualify it explicitly.

## Cancel probe

Change the SessionList assertion to require `li[data-session-id="session-new-2"][aria-current="true"]` before adding the row attribute. It must fail against the current markup. Then implement the ID and verify a wrong-row mutation cannot satisfy it.

## Files in scope

- `web/studio/src/components/SessionList.tsx` only to expose real session IDs on existing native rows.
- `web/studio/src/components/SessionList.test.tsx` for replacement-ID/active-marker evidence.
- `docs/roadmap/items/studio-typed-shell-upgrade.md` for the `canStop` exception.
- `docs/research/notebook/m4-serving.md` for the full repair chain and zero-unhandled result.
- `docs/loop/tasks/studio-typed-shell-upgrade/kimi-60/REPORT.md` only for selector precision.
- `docs/loop/tasks/studio-typed-shell-upgrade/kimi-61/REPORT.md` for final results and exact log paths.
- This `kimi-62` repair folder.

No real-data migration, production cutover/publication, native/package work, model/worker/tokenizer/`studio_api.js` change, dependency, external endpoint/credential, Phase B compute, useful-chat, M7/M8, or accessibility-certification claim.

## Done when

- The replacement's real session ID and active marker are asserted together; title-only or wrong-row evidence cannot pass.
- Notebook links through kimi-62 and records 411/411 with zero unhandled errors. The final report names the final result and correct log paths.
- Roadmap distinguishes stoppable runs from non-stoppable predictions. Full checks and fresh exact-head review pass before protected delivery.

## Stop and report

Stop before weakening the replacement assertion, using title-only evidence, changing unrelated UI, or publishing. Continue only after fresh approval.

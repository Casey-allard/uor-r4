# ORDER: repair shared TypeScript composer review findings

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Original base: `2a661d900cd64564041cbf2a97351e97bf7e4025`. Reviewed head: `1e50254cc0ca790545d1353ccc11acc357f9c4ce`, tree `ce4d0b30eb9a0533ca9b097ca9920970c94c7869`, PR #3817.

## Finished steps

- kimi-4 delivered the typed controller and accessible interactions in `1e50254cc0ca790545d1353ccc11acc357f9c4ce`; its declared product and harness checks passed.
- Fresh review returned CHANGES: duplicate controlled-draft callbacks and persisted history edits, images consuming the text budget, incomplete keyboard/empty/focus/failure evidence, and a missing default-options argument.

## Goal

Repair those findings without broadening the typed-component unit. Preserve kimi-4 behavior, public APIs, lifecycle state, legacy page, model/source/privacy behavior, and controlled drafts. No live-page cutover.

## Cancel probe

Callback duplication has two input paths; image accounting includes every attachment kind. Add focused failing tests first. Stop rather than weakening legacy history/draft semantics.

## Files in scope

- `web/studio/src/adapters/composerInput.ts` and `composerInput.test.ts`.
- `web/studio/src/components/AgentComposer.tsx` and `AgentComposer.test.tsx`.
- `web/studio/README.md`, `docs/products/web-client.md`, and `docs/research/notebook/m4-serving.md` only to correct counts/evidence.
- This repair folder and the existing implementation plan only if the repair requires an implementation-note correction.

No other paths are authorized. The earlier prohibitions remain: no legacy page/controller, dependency, engine, worker, tokenizer, `studio_api.js`, model artifact, publisher, deployment, `.github/**`, `Cargo.lock`, real browser data, owner-controlled path, or generated issue/roadmap edit.

## Done when

- A normal draft input invokes the existing external callback once. History recall and history edits do not overwrite the retained in-progress draft. Successful send clears once; unsuccessful send preserves the visible draft and does not record a submission.
- The picked-text path excludes retained image attachments from text-budget accounting and has a regression with an over-4-MiB image plus a valid text file.
- `/` and `@` coverage includes ArrowUp/ArrowDown, Home/End, Enter/mouse focus retention, `/` Escape, no-active-file/no-attachment empty state, and no model request during navigation. Existing no-result, ARIA, filtering and duplicate-attachment behavior stays green.
- `createComposerInput()` without options throws the legacy explicit `A composer textarea is required.` error, with a test.
- All focused and complete Studio checks, plain-JS tests, legacy static/runtime checks, claim wording, report/local PR checks, and full gates pass at the new exact head. The PR check's D76 scope advisory for the ORDER-authorized notebook/plan and any branch-claim handling are disclosed rather than silently editing generated/owner files.
- Fresh exact-head review approves before ready/queue. PR continues to use `References #3500`; no live cutover, publish, migration, useful-chat, M7/M8, production-readiness, or accessibility-certification claim.

## Stop and report conditions

Stop before changing files outside this scope or weakening a test/verifier. Stop after this repair and its protected delivery; do not start another UI unit in this session.

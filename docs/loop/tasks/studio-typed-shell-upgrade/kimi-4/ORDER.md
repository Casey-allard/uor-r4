# ORDER: make the shared TypeScript composer complete and accessible

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500), continuing the approved phased design after merged core-workflow PR #3541. Base: `2a661d900cd64564041cbf2a97351e97bf7e4025`.

## Goal

Move the existing composer controller into the typed library without changing its behavior, then finish `AgentComposer` with accessible `/` build-task and `@` attachment interactions. Keep one lifecycle store, the existing public APIs, the legacy page, the engine, and all model/source/privacy behavior unchanged. This is a workflow unit, not a live-page cutover.

## Cancel probe

The plain-JS `createComposerInput` controller already implements Enter without Shift, IME refusal, complete history/draft restoration, reset, paste/drop/picker channels, and long-text preservation. The typed `AgentComposer` lacks those behaviors and has no `/` or `@` menu. If the typed controller cannot preserve the existing test contract, stop rather than renaming or weakening the legacy behavior.

## Files in scope

- `web/studio/src/adapters/composerInput.ts` and `composerInput.test.ts`.
- `web/studio/src/components/AgentComposer.tsx`, `AgentComposer.test.tsx`, and any small focused interaction-menu component/test needed for the same accessible behavior.
- `web/studio/src/main.ts` and `web/studio/README.md` for the existing typed-library description.
- This task folder, the implementation plan, `docs/products/web-client.md`, and `docs/research/notebook/m4-serving.md`.
- The owning roadmap item/generated issue data only if the normal generator permits it and no hand edit is needed.

No other paths are authorized. In particular, do not change `web/studio/index.html`, the legacy `assets/js/composer_input.js`, the engine, worker, tokenizer, `studio_api.js`, model artifacts, publisher, deployment, `.github/**`, `Cargo.lock`, Claude's active work, real browser data, or owner-controlled paths. Do not add a dependency.

## Done when

- New TypeScript controller tests fail first for the intended missing implementation, then pass with the same Enter/Shift+Enter, IME, raw multiline history, in-progress draft restoration, reset, picker/paste/drop, and long-text behavior as the legacy controller.
- `/` opens an accessible filtered build-task list with the existing six templates. Mouse and Arrow/Home/End navigation, Enter selection, Escape dismissal, and normal Enter send/history behavior are tested.
- `@` opens an accessible filtered active-file/attachment list. Selection inserts the reference and attaches content through the existing UI API without duplicating an existing attachment. Keyboard and mouse behavior, empty states, and no-result preservation are tested.
- ARIA listbox/option names, active descendant, expanded state, selection, and focus retention are asserted. No model request is made by opening or navigating a menu.
- `npm test`, `npm run typecheck`, `npm run build`, the relevant plain-JS Node tests, legacy static checks, claim wording, and local policy pass. PR uses `References #3500`; #3500 remains open for parity/migration/cutover. Fresh exact-head review, protected merge, and fresh-main verification are delivery steps, not claims in this ORDER.
- Product docs and notebook distinguish typed-component evidence from full live-page parity. No production publish, real browser-data migration, useful-conversation, M7/M8, production-readiness, or model capability claim is made.

## Stop and report conditions

Stop before changing any file outside the listed scope or weakening a test/verifier. Stop after this declared typed-component unit; do not replace the live page, migrate real browser data, invoke the publisher, or start another UI unit in this session.

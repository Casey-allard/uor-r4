# Shared TypeScript composer implementation plan

## Context

`AgentComposer` is already a typed workflow component, but it does not preserve the legacy controller's complete keyboard/history/file behavior and has no `/` or `@` interaction model. The legacy plain-JS controller is the behavior reference and remains untouched for the live page. This unit is limited to the typed library and its evidence.

## Approach

Port `createComposerInput` into `src/adapters/composerInput.ts` with the same public controller methods and state transitions, plus one optional key-interception hook for the component's accessible menus. Port the existing Vitest suite to the typed implementation first, so the missing file fails before implementation. Add the controller to `AgentComposer` and implement a small listbox interaction menu for the existing six build-task templates and current file/attachments.

No new runtime dependency is needed. The legacy plain-JS controller, live page, engine, worker, tokenizer, model-source implementation, publisher, and real browser data remain unchanged.

## Steps

1. **Red-first controller contract.** Change `composerInput.test.ts` to import the new TypeScript module and retain every existing assertion. Run it and record the missing-module failure.
2. **Typed controller.** Implement the existing `recordSubmitted`, `reset`, and `destroy` API with bounded history, IME refusal, native Shift+Enter/caret behavior, exact draft restoration, and file channels. Add an optional pre-keydown interceptor only for menu handling; default behavior is unchanged.
3. **Red-first interaction cases.** Add `/` and `@` tests for filtered options, ARIA roles/names/active descendant, Arrow/Home/End navigation, Enter selection, Escape, mouse selection, no-result preservation, and no generation on menu navigation.
4. **Component integration.** Mount the controller in `AgentComposer`, forward the existing draft/send/reset/file paths, and implement the accessible menu. Selecting `/` uses the existing template API. Selecting `@` inserts the reference and attaches content through the existing UI API, avoiding duplicate attachment names.
5. **Verification.** Run focused Vitest, the complete Studio suite, plain-JS Node tests, TypeScript/build, and legacy static checks. Preserve the red-first and green logs. Review the final diff for scope and stale comments before writing REPORT and requesting fresh exact-head review.
6. **Delivery.** Commit the finalized unit, run exact-head checks, obtain fresh-context review, then use the protected merge queue and fresh-main verification. Do not publish, replace the live page, or start another unit in this session.

## Assumptions

- #3500 is the owning work card; it remains open for full parity, migration, and cutover.
- The existing six build-task templates and current file/attachments are the complete `/` and `@` option sources for this unit; MCP-specific `@` tools belong to a later unit.
- Menu navigation never sends a model request and does not change model/source/privacy state.
- Duplicate attachment names are not added twice; selecting an existing `@` attachment only inserts its reference.
- This unit claims typed-component behavior only, not full live-page parity, production accessibility certification, or model capability.

## Verification contract

The final report must include the missing-module red first, new interaction red first, full green counts, exact head/tree, and any remaining limitations. No useful-conversation, M7/M8, production-readiness, real-browser-data migration, or live-page cutover claim is allowed.

# ORDER: unify visible prompt and controller history

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Original base: `2a661d900cd64564041cbf2a97351e97bf7e4025`. PR #3817 remains draft. Prior reviewed head: `c5677af930be2ad05c201805126a4fac631eeec2`, tree `4a5305c7049f0d7f635a6a15deb0381c8e2a525b`.

## Finished steps

- kimi-4 delivered the typed controller and accessible interactions; kimi-5 repaired callback duplication, image/text accounting, omitted options, and interaction evidence.
- The next fresh review confirmed those repairs but returned CHANGES because history recall changed only the textarea, leaving component draft state stale. Mouse Send could send another prompt or stay disabled, `@` splicing could discard surrounding text, and programmatic templates left history navigation active. Controlled `draft`/`pasted` props and real mousedown focus behavior lacked direct evidence.

## Goal

Use one visible-prompt source for Send, disabled state, and mention replacement. Synchronize controller history on programmatic draft changes without losing submitted history or persisting history edits as the retained session draft. Preserve controlled prop behavior and every earlier repair. This remains typed-component work, not live cutover.

## Cancel probe

Reproduce history→mouse Send, history→`@`, and history→template→ArrowDown before changing source. Stop rather than weakening the controller's unsaved-history rule, controlled callback contract, or submitted-history retention.

## Files in scope

- `web/studio/src/components/AgentComposer.tsx` and `AgentComposer.test.tsx`.
- `web/studio/src/adapters/composerInput.ts` and `composerInput.test.ts` only if the existing controller API genuinely cannot express the repair.
- `web/studio/README.md`, `docs/products/web-client.md`, and `docs/research/notebook/m4-serving.md` only for corrected evidence/counts.
- This repair folder.

No other paths are authorized. No legacy page/controller, dependency, engine, worker, tokenizer, `studio_api.js`, model artifact, publisher, deployment, `.github/**`, `Cargo.lock`, real browser data, owner-controlled path, or generated roadmap/issue edit.

## Done when

- After history recall, mouse Send uses the visible prompt; `@` selection splices the visible surrounding text; programmatic template/mention draft changes reset history navigation without erasing submitted history or reviving the pre-recall draft.
- Ordinary controlled `draft` and `pasted` props/callbacks work, parent draft updates do not reset history per keystroke, history edits stay unsaved, and successful/failed send behavior remains correct.
- Mouse option tests dispatch mousedown and prove default prevention plus retained textarea focus. Prior keyboard/ARIA/empty/image-budget/omitted-options tests stay green.
- Focused/full Studio checks, plain-JS tests, legacy checks, claim/report/local-PR checks, and full gates pass at the new exact head. Existing disclosed D76 advisories remain disclosed; no new warning is introduced.
- Fresh exact-head review approves before ready/queue. PR keeps `References #3500`; no live cutover, publish, migration, useful-chat, M7/M8, production-readiness, or accessibility-certification claim.

## Stop and report conditions

Stop before changing files outside this scope or weakening a test/verifier. Stop after this repair and protected delivery; do not start another UI unit in this session.

# REPORT: unify visible prompt and controller history, kimi-6

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Original base: `2a661d900cd64564041cbf2a97351e97bf7e4025`. Prior reviewed head: `c5677af930be2ad05c201805126a4fac631eeec2`, tree `4a5305c7049f0d7f635a6a15deb0381c8e2a525b`. The new exact delivery-head commit/tree and final logs are recorded on PR #3817 rather than embedding a self-referential hash here.

## What did not work

- The second fresh review confirmed the kimi-5 repairs but returned CHANGES for a shared visible-state defect. History recall wrote the textarea while component draft state stayed stale: mouse Send was disabled or sent another prompt, `@` splicing discarded surrounding text, and programmatic templates left history navigation active.
- Red-first regressions reproduced all three: controlled history recall left Send disabled; selecting `notes.md` after recalled `old @` produced `@notes.md` instead of `old @notes.md`; template→ArrowDown restored `in progress` instead of `Prompt for test`.
- The first mention fixture also exposed two instrumentation mistakes, corrected without weakening the expected result: the caret must be at the end after programmatic entry, and the mention token must begin after whitespace. The final red assertion is the exact stale-splice defect above.
- Existing tests had not exercised controlled `draft`/`pasted` props or cancelable mousedown focus behavior. The earlier callback repair alone did not establish one visible-prompt source.
- No live-page cutover, production publish, browser-data migration, useful conversation, model capability, M7/M8, production-readiness, or accessibility-certification result is claimed.

## What changed

- The textarea now owns the visible prompt. Mouse Send and disabled state use its current value mirrored into `visibleDraft`; mention replacement slices that same value. The retained parent draft is separate and is not overwritten by history recall/edit.
- Programmatic template/mention changes call the controller's existing `reset`, which moves history navigation to the newest entry. A component history ref supplied through the existing `getHistory`/`onHistoryChange` API preserves submitted prompts across that reset; no controller API or behavior changed.
- Ordinary DOM input updates visible state while only the controller forwards the retained parent callback once. Programmatic resets suppress that callback inside the controller, then notify the parent once explicitly. Successful sends record once and clear the visible DOM/state without erasing history; failed sends remain unchanged.
- Controlled prop synchronization distinguishes the component's own callback updates from unrelated external replacement. Parent updates caused by typing do not reset history per keystroke; external draft replacement resets visible/history state without emitting a retention callback. Existing controlled pasted behavior is unchanged and directly tested.
- Added history→mouse Send with controlled props, external controlled replacement, history→`@`, and history→template→ArrowDown regressions. The mouse focus test now dispatches cancelable mousedown and asserts default prevention plus textarea focus.

No dependency, legacy page/controller, engine, worker, tokenizer, `studio_api.js`, model artifact, publisher, deployment, `.github/**`, `Cargo.lock`, real browser data, owner-controlled path, or generated roadmap/issue file was changed. The repair touches only the typed component/tests and authorized records.

## Measurements

- Focused composer/controller checks: **42/42 tests pass** (`17` controller plus `25` component).
- Complete typed suite: **335/335 tests pass across 41 files**.
- Plain-JS suite: **112/112 tests pass** at the prior exact head and rerun at this repair head before delivery.
- `npm run typecheck`: PASS with no TypeScript errors.
- `npm run build`: PASS; `dist/studio-foundation.js` **138.00 kB** (gzip **38.38 kB**) and `dist/studio-foundation.css` **3.57 kB** (gzip **1.14 kB**), uncommitted.
- Repaired-head plain-JS, strict page, runtime page-load, desktop-client, worker, claim/report/local-PR and full-gate results are recorded in the new exact-head logs on the PR before fresh review and queueing.

## Checks run at the head

`red-visible-history.log` retains the three intended cross-operation failures and the fixture-correction trail to the exact stale-splice assertion. `green-visible-history.log`, `full-vitest-visible-history.log`, `typecheck-visible-history.log`, and `build-visible-history.log` record the repaired implementation. New exact-head suite/gate/diff files supersede the c5677af9 evidence for delivery; all earlier unique logs remain preserved locally under `web/studio/.superpowers/sdd/studio-ts-composer/checks/`.

The local PR check's D76 scope advisory covers the ORDER-authorized notebook and implementation plan; the commit-order advisory is the historical kimi-4 combined commit; FILEMAP advisories concern unrelated existing folders. No generated roadmap/issue scope was edited. The branch-claim rule previously passed in isolation and the claim was released without committing coordination metadata.

Fresh exact-head review, green GitHub checks, protected merge, and fresh-main verification remain required. PR #3817 stays draft until a new review approves this repair head. It uses `References #3500`, and the issue remains open.

## Skills used

Receiving code review with root-cause source verification, red-first TDD, controlled/uncontrolled state analysis, systematic fixture debugging, and verification before completion. A fresh independent exact-head delivery reviewer follows the repaired commit. No model compute, paid resource, credential, real user data, or production publish was used.

## Still open

The typed library remains unmounted. Full real-browser/worker/mobile/light/dark/offline parity, runtime accessibility acceptance, safe rich replies and other Harness workflows, browser-data migration, packaged-Tauri composer cutover, production publication, and useful model conversation remain outside this repair. #3500 remains open for its full completion contract.

## Plain summary

Visible text, retained session draft, and submitted history now have separate owners. Mouse Send and `@` use the text the user actually sees, templates reset only history navigation, and controlled parent updates work in both directions. All declared component tests pass; no live application or model capability claim is made.

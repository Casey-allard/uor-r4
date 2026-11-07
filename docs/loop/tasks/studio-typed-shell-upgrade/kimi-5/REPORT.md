# REPORT: repair shared TypeScript composer review findings, kimi-5

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Original base: `2a661d900cd64564041cbf2a97351e97bf7e4025`. Repair ORDER commit: `bdf4bdac7`. Prior reviewed head: `1e50254cc0ca790545d1353ccc11acc357f9c4ce`, tree `ce4d0b30eb9a0533ca9b097ca9920970c94c7869`. The new exact delivery-head commit/tree and final logs are recorded on PR #3817 rather than embedding a self-referential hash here.

## What did not work

- The fresh kimi-4 reviewer returned CHANGES. Both the Preact `onInput` and the controller's native input listener forwarded the same external draft callback, causing an initial clear plus duplicate normal-input callbacks; successful clear was duplicated too. The outer path also persisted history recall/edits, bypassing the controller's deliberate unsaved-history rule.
- The text picker passed retained images into the text reader's 4 MiB budget. An over-4-MiB image therefore blocked a valid small text file, unlike the live page's separate text/image accounting.
- Interaction evidence omitted ArrowUp/Home, `/` Escape, no-active-file/no-attachment state, focus after Enter/mouse selection, and unsuccessful-send behavior. The controller also omitted the legacy default-options argument and threw a destructuring `TypeError` instead of the explicit missing-textarea error.
- Red-first repair logs confirm the defects: normal input produced `['', 'first', 'first']` instead of one `first` callback; the image case produced no attachment; omitted options produced `Cannot destructure property 'textarea' ...` instead of `A composer textarea is required.` The new keyboard/focus/empty-state and failed-send assertions already passed and now prevent regressions.
- Two declaration-only typecheck attempts showed why the runtime compatibility default must not weaken the textarea contract: optional textarea lost narrowing in controller closures. The final declaration retains required `HTMLTextAreaElement` and casts only the empty default to the runtime-validated options type. Typecheck and all runtime tests pass at the final form.
- No live-page cutover, production publish, real browser-data migration, useful conversation, model capability, M7/M8, production-readiness, or accessibility-certification result is claimed.

## What changed

- The controller owns external draft callbacks for DOM input. Preact `onInput` now only updates interaction-menu state, so normal input fires once, controller-generated history recall/edit does not overwrite the retained in-progress draft, and mount no longer emits an initial clear.
- Successful sends use `recordSubmitted` as the single clear path, with only a pre-controller fallback. Failed sends preserve the visible draft and do not record a submission.
- Picked text files pass only non-image retained attachments to `readTextFiles`; retained image bytes no longer consume the text budget. Existing image reading/parity remains outside this unit.
- `createComposerInput()` without options now reaches the legacy explicit missing-textarea error while preserving the required textarea type for ordinary callers.
- Expanded `/` and `@` tests cover ArrowUp/ArrowDown, Home/End, Enter/mouse focus retention, `/` Escape, empty mentions, and no generation during navigation. Added focused single-callback/history, failed-send, image-budget, and omitted-options regressions.

No dependency, legacy page/controller, engine, worker, tokenizer, `studio_api.js`, model artifact, publisher, deployment, `.github/**`, `Cargo.lock`, real browser data, owner-controlled path, or generated roadmap/issue file was changed. Cargo output remains under `target/kimi-lexlean`.

## Measurements

- Focused repaired composer/controller checks: **38/38 tests pass** (`17` controller plus `21` component).
- Complete typed suite: **331/331 tests pass across 41 files**.
- Plain-JS suite: **112/112 tests pass** at the prior exact head and rerun at the repaired head before delivery.
- `npm run typecheck`: PASS with no TypeScript errors.
- `npm run build`: PASS; `dist/studio-foundation.js` **137.40 kB** (gzip **38.20 kB**) and `dist/studio-foundation.css` **3.57 kB** (gzip **1.14 kB**), uncommitted.
- Repaired-head plain-JS, strict page, runtime page-load, desktop-client, worker, claim/report/local-PR and full-gate results are recorded in the new exact-head logs on the PR before fresh review and queueing.

## Checks run at the head

Red-first repair evidence is in `red-review-controller.log` and `red-review-interactions.log`; focused green evidence is in `green-review-repairs.log`. Full-suite and build evidence is in `full-vitest-repairs.log`, `typecheck-repairs.log`, and `build-repairs.log`. New exact-head suite/gate/diff files supersede the kimi-4 evidence for delivery; all earlier logs remain preserved locally under `web/studio/.superpowers/sdd/studio-ts-composer/checks/`.

The local PR check's D76 scope advisory covers the notebook and implementation plan explicitly authorized by the ORDER; no generated roadmap/issue scope was edited. The original combined kimi-4 commit-order advisory is historical, and this repair ORDER was committed alone before repair code. FILEMAP advisories concern unrelated existing folders. The `status.branch_claimed` rule passed in isolation at the repair head (`1 rules, 0 failed`); the local claim was then released and no coordination file was committed.

Fresh exact-head review, green GitHub checks, protected merge, and fresh-main verification remain required. PR #3817 stays draft until a new review approves the repaired head. It uses `References #3500`, and the issue remains open.

## Skills used

Receiving code review with source verification, red-first TDD, systematic debugging of callback ownership and type declarations, and verification before completion. A fresh independent exact-head delivery reviewer follows the repaired commit. No model compute, paid resource, credential, real user data, or production publish was used.

## Still open

The typed library remains unmounted. Full real-browser/worker/mobile/light/dark/offline parity, runtime accessibility acceptance, safe rich replies and other Harness workflows, browser-data migration, packaged-Tauri composer cutover, production publication, and useful model conversation remain outside this repair. #3500 remains open for its full completion contract.

## Plain summary

The repair preserves ordinary draft callbacks exactly once and history edits as temporary, separates image bytes from the text attachment budget, completes the interaction evidence, and restores the controller's omitted-options error. The expanded component suite is green; no live application or model capability claim is made.

# REPORT: complete the shared TypeScript composer, kimi-4

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Plan: [plan](../../../../superpowers/plans/2026-11-07-studio-ts-composer.md). Base: `2a661d900cd64564041cbf2a97351e97bf7e4025`. Exact delivery-head commit/tree and the final check logs are recorded on the PR; this file does not attempt to embed its own commit hash.

## What did not work

- The typed controller module initially did not exist. After retargeting the unchanged legacy controller suite, `npm test -- src/adapters/composerInput.test.ts` failed on the missing `./composerInput.js` import. This is the intended red-first proof, retained in `red-controller-missing.log`; an earlier missing-dependencies probe is not treated as product evidence.
- `AgentComposer` had no controller integration, interaction menus, or file picker. All five new `composer interactions` tests failed first for those intended missing behaviors: no Enter submission, no listboxes, and no picked-file path. The red log is `red-interactions.log`.
- The picked-file test initially asserted before the asynchronous `File.text()` chain completed. Instrumentation proved that the picker delivered one file and the existing reader returned the exact expected attachment. The test now waits for that same observable call; its payload assertion was not weakened and production reading was not made synchronous.
- The first worker-verifier attempt failed before product checks because `/opt/homebrew/bin/cargo` is a `cargo-ai` shim without ordinary `metadata`/`build` commands. Prepending the real `~/.cargo/bin` Rust 1.97.1 toolchain to `PATH` made the prescribed WASM build and worker verifier pass. This was an instrument correction, not a mechanism or model result.
- No live-page parity, real-browser/mobile accessibility acceptance, browser-data migration, production publication, useful conversation, model capability, M7/M8, or production-readiness result is claimed.

## What changed

- Added `web/studio/src/adapters/composerInput.ts`, a typed port of the existing controller with bounded 100-entry submitted history, raw multiline recall and in-progress draft restoration, Enter without Shift, IME refusal, reset/destroy, picker/paste/drop channels, and unchanged long-text behavior. The optional `interceptKeyDown` hook handles menus before ordinary controller keys; default behavior is unchanged. The existing plain-JS `readTextFiles` is re-exported rather than duplicated.
- Mounted the controller in the existing `AgentComposer` with textarea, picker, controller, and latest-callback refs. Successful sends record history and clear the draft; unsuccessful sends do not clear it. Existing readiness, pinned source/privacy/mode, generation/stop, templates, attachments, Retry, and controlled draft/paste behavior remain intact.
- Added filtered `/` build-task and `@` active-file/current-attachment menus with stable option IDs, named listboxes/options, `aria-expanded`, `aria-controls`, `aria-activedescendant`, and `aria-selected`. ArrowUp/ArrowDown, Home/End, Enter, Escape, and mouse selection preserve textarea focus and never send a model request. Unmatched tokens submit normally; existing attachment names are not added twice.
- Added the multiple-file picker and routed accepted files through the bounded existing text reader, with reader failures surfaced through `ui.toast`.
- Exported the typed controller from `src/main.ts` and scoped the README, product note, and notebook to typed development-component evidence.

No dependency, legacy page/controller, engine, worker, tokenizer, `studio_api.js`, model artifact, publisher, deployment, `.github/**`, `Cargo.lock`, real browser data, or owner-controlled path was changed. Cargo output stayed under the single `target/kimi-lexlean` root.

## Measurements

- Focused composer/controller checks: **34/34 tests pass** (`16` controller plus `18` component); the five new interaction tests also pass as a focused selection.
- Complete typed suite: **327/327 tests pass across 41 files**.
- Plain-JS suite: **112/112 tests pass**.
- `npm run typecheck`: PASS with no TypeScript errors.
- `npm run build`: PASS; `dist/studio-foundation.js` **137.39 kB** (gzip **38.19 kB**) and `dist/studio-foundation.css` **3.57 kB** (gzip **1.14 kB**), uncommitted.
- `verify_studio_page.mjs --strict-router`: PASS; both strict pages clean.
- `verify_studio_page_load.mjs --strict-router`: PASS; both pages loaded, planted external-request traps detected, and actual external request lists empty.
- `verify_desktop_client.mjs`: PASS; declared OpenAI transport, binary contract, 32 streamed tokens, 38 invoked commands, and zero remote requests.
- `verify_studio_worker.mjs`: PASS; 32 expected tokens, bit-identical save/restore, guarded mid-generation close/reset, six trap self-test hits, and zero remote requests.

## Checks run at the head

Red-first logs are `red-controller-missing.log` and `red-interactions.log`. Green evidence includes `green-controller.log`, `green-interactions.log`, `green-composer-component.log`, `full-vitest.log`, `typecheck.log`, `build.log`, `plain-js.log`, `legacy-page.log`, `legacy-page-load.log`, `legacy-desktop.log`, `wasm-build.log`, and `legacy-worker.log` under the local, uncommitted `web/studio/.superpowers/sdd/studio-ts-composer/checks/` directory.

Final repository claim-wording/local-policy gates, diff/status, and exact-head commit/tree checks are rerun after this report and posted on the PR before queueing. Fresh-context exact-head review, green GitHub checks, protected merge, and fresh-main verification remain delivery requirements, not claims made by this implementation report. The PR uses `References #3500`; the issue stays open.

## Skills used

Executing an existing plan, red-first TDD, systematic debugging of the async test and Cargo launcher, modern web guidance for the textarea/listbox interaction, and verification before completion. Independent exact-head delivery review follows on the committed head. No model compute, paid resource, credential, real user data, or production publish was used.

## Still open

The typed library remains unmounted. Full real-browser/worker/mobile/light/dark/offline parity, runtime accessibility acceptance, safe rich-reply and other Harness workflows, one-time browser-data migration, packaged-Tauri composer cutover, production publication, and useful model conversation remain outside this unit. #3500 remains open for its full completion contract.

## Plain summary

The development TypeScript composer now preserves the legacy input behavior and adds keyboard- and mouse-accessible build-task/attachment menus plus bounded file attachment. All declared component and legacy checks pass. The public page and packaged application have not been switched to this component, and no new model capability is claimed.

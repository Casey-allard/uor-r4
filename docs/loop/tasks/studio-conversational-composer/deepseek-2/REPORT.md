# REPORT: mixed-paste test evidence repair, deepseek-2

> **Superseded by [deepseek-3](../deepseek-3/REPORT.md).** A fresh review found
> that this report assigned unexecuted Studio/browser checks to CI even though
> no current workflow runs them. The accepted test and implementation are
> unchanged.

Lab: deepseek. Item:
[studio-conversational-composer](../../../../roadmap/items/studio-conversational-composer.md)
(#3608). Order: [ORDER.md](ORDER.md). Base:
`c8b6606b8e4d16cb8c5099a8b16d0249bdd7f18d`.

## What did not work, first

- The first test stored only attached files. Calling `onFiles([])` on a
  text-only paste would therefore be indistinguishable from making no callback.
  It also did not supply or observe `onSend`.
- The first report said Node 18.19.1 had no browser `crypto`; the CLI exposes
  `globalThis.crypto`. The test-runner child omitted it by default, while
  `NODE_OPTIONS=--experimental-global-webcrypto` restores it. `File` is genuinely
  unavailable in this runner.
- `npm`, Vitest and a real browser run remain unavailable on this host.

## What changed

The plain-Node test now records attachment and send invocation counts. Mixed and
file-only paste each assert one attachment callback and zero sends. A separate
text-only test asserts zero attachment callbacks, zero sends and native paste.

The deepseek-1 report now distinguishes the CLI WebCrypto global from the
test-runner process's default. Production paste handling, attachment validation
and send behavior are unchanged.

## Measurements

- Focused controller tests: **3 passed, 0 failed**.
- Broader plain-Node run: **29 passed, 2 failed**. The failures are the
  test-runner `crypto` omission and missing browser `File`; all other tests
  passed.
- Targeted `model_sources.test.mjs` run with
  `NODE_OPTIONS=--experimental-global-webcrypto`: **1 selected test passed**;
  11 non-matching tests skipped.

## Checks run at the head

- `node --test web/studio/assets/js/composer_input.test.mjs`: PASS.
- `node --test web/studio/assets/js/*.test.mjs`: 29 PASS / 2 environment
  failures.
- `NODE_OPTIONS=--experimental-global-webcrypto node --test
  --test-name-pattern='published loads forward operation metadata'
  web/studio/assets/js/model_sources.test.mjs`: PASS.
- `node --check` for the controller and test: PASS.
- `git diff --check`: PASS.
- Evidence `SHA256SUMS`: PASS.
- `uor-harness loop report --check`, `loop pr-check --local` and
  `loop gates --full`: PASS.

## Skills used

No available Codex skill applied.

## Still open

- Full Vitest, typecheck and Vite verification is `NOT_RUN` because `npm` and
  `node_modules` are absent locally, and no current repository workflow
  executes that suite.
- A real browser mixed file-plus-text paste remains `NOT_RUN` on this host and
  is not covered by a current workflow.
- Image decomposition and visual understanding remain outside this item.

## Plain summary

The production change was already correct: a paste carrying a file and text
attaches the file while allowing the text to appear normally. The first test
was too weak because an empty callback looked the same as no callback, and it
did not count sends. The test now counts both callbacks. The report also states
the Node test-runner limitation accurately instead of blaming Node itself.

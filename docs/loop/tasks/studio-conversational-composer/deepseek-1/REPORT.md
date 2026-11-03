# REPORT: studio-conversational-composer, deepseek-1

> **Superseded by [deepseek-2](../deepseek-2/REPORT.md).** A fresh review found
> that the first regression test did not observe callback counts and that the
> Node environment note conflated the CLI global with the test runner's default
> WebCrypto exposure. The production mixed-paste behavior was accepted.

Lab: deepseek. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Mixed clipboard paste lost its text.** The delivered composer called
  `preventDefault()` whenever `clipboardData.files` was non-empty. A clipboard
  carrying both a text/source file and ordinary text attached the file but
  discarded the native text insertion.
- **The existing browser-oriented test covered only one channel at a time.** It
  tested a file-only paste as prevented and a text-only paste as native, but not
  a single event carrying both.
- **The plain Node package check is not a complete Studio run.** Its test-runner
  child reports `crypto` and `File` as undefined for `model_sources.test.mjs`
  even though the Node 18.19.1 CLI exposes `globalThis.crypto`. The explicit
  `NODE_OPTIONS=--experimental-global-webcrypto` flag fixes the crypto case;
  browser `File` remains unavailable.
- **`npm`, Vitest and the browser flow could not be rerun on this machine.**
  `npm` is not installed and `web/studio/node_modules` is absent. The focused
  behavior is covered by a dependency-free Node test; the complete Studio suite
  is `NOT_RUN`.

## What changed

`composer_input.js` now reads `text/plain` from the same paste event. It calls
`preventDefault()` only when files are present and that text is empty. Every
paste carrying files still calls the attachment callback, and mixed paste lets
the browser insert its text normally at the caret.

The change is four added and two removed lines in the controller. A new
65-line plain-Node test covers mixed, file-only and text-only paste with
explicit attachment and send invocation counters. The task report and item note
are documentation; retained focused/broader test output, syntax checks and diff
checks are generated evidence.

## Measurements

- `node --test web/studio/assets/js/composer_input.test.mjs`: **3 passed, 0
  failed**.
- `node --test web/studio/assets/js/*.test.mjs`: **29 passed, 2 failed**. Both
  failures are the environment limitations named above; all 29 tests outside
  those missing browser globals passed.
- Mixed paste: attachment callback received the file and `defaultPrevented`
  stayed false.
- File-only paste: attachment callback received the file and
  `defaultPrevented` became true.
- Text-only paste: no attachment callback and native insertion stayed enabled.

## Checks run at the head

- `node --test web/studio/assets/js/composer_input.test.mjs`: PASS.
- `node --test web/studio/assets/js/*.test.mjs`: 29 PASS / 2 environment
  failures.
- `NODE_OPTIONS=--experimental-global-webcrypto node --test
  --test-name-pattern='published loads forward operation metadata'
  web/studio/assets/js/model_sources.test.mjs`: 1 PASS.
- `node --check web/studio/assets/js/composer_input.js`: PASS.
- `node --check web/studio/assets/js/composer_input.test.mjs`: PASS.
- `git diff --check`: PASS.
- `uor-harness loop gates --full`: 11 of 11 gates green.

## Skills used

No available Codex skill applied.

## Still open

- The full Vitest/typecheck/Vite suite is `NOT_RUN`: this host has no `npm`
  install, and no current repository workflow executes that suite.
- A real browser mixed file-plus-text paste is `NOT_RUN` on this host and is not
  covered by a current workflow. The Node regression pins only the controller
  contract.
- Image decomposition and visual understanding remain outside this item.

## Plain summary

Clipboard paste can contain a file and text at the same time. The composer used
to treat any file as a reason to cancel the browser's normal paste, so the text
disappeared while the file was attached. It now inspects the text too. It still
blocks the browser insertion for file-only paste, but mixed paste attaches the
file and leaves the text to insert normally. The focused test passes; the full
web test runner is unavailable locally and is `NOT_RUN`; no current workflow
covers it.

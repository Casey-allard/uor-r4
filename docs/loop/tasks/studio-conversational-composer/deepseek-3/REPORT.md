# REPORT: unrun-check correction, deepseek-3

Lab: deepseek. Item:
[studio-conversational-composer](../../../../roadmap/items/studio-conversational-composer.md)
(#3608). Order: [ORDER.md](ORDER.md). Base:
`c8b6606b8e4d16cb8c5099a8b16d0249bdd7f18d`.

## What did not work, first

The prior reports assigned unexecuted Vitest, typecheck, Vite and browser checks
to CI. No current repository workflow runs those checks. The repository's
required jobs are compatibility acknowledgements and loop/documentation gates,
not the Studio npm suite.

This was a record error, not a production or test failure. The strengthened
Node regression and four-line paste fix were accepted by the preceding review.

## What changed

The deepseek-1 and deepseek-2 reports now label the unavailable Studio and
browser checks `NOT_RUN` and state that no current workflow covers them. The
roadmap links this active report. Production code, the accepted test, and all
evidence remain unchanged.

## Measurements

- Focused controller tests: **3 passed, 0 failed**.
- Broader plain-Node run: **29 passed, 2 failed** only for the documented
  test-runner `crypto` omission and unavailable browser `File`.
- WebCrypto-flag target: **1 passed**; 11 non-matching tests skipped.
- Vitest, typecheck, Vite and a real browser run: **NOT_RUN**.

## Checks run at the head

- `node --test web/studio/assets/js/composer_input.test.mjs`: PASS.
- `node --test web/studio/assets/js/*.test.mjs`: 29 PASS / 2 environment
  failures.
- `NODE_OPTIONS=--experimental-global-webcrypto` targeted model-source test:
  PASS.
- `node --check` for the controller and test: PASS.
- `git diff --check`, retained evidence checksums, `loop report --check`,
  `loop pr-check --local` and `loop gates --full`: PASS.
- Vitest, Vite, typecheck and real-browser checks: NOT_RUN with no current
  workflow coverage.

## Skills used

No available Codex skill applied.

## Still open

- Add a real npm/Vitest/typecheck/Vite workflow or run them on a capable host.
- Add a real browser mixed-paste check.
- Image decomposition and visual understanding remain outside this item.

## Plain summary

The paste fix and its regression test are correct. Two reports incorrectly said
CI would run the Studio web suite, but the current CI does not. This repair
clearly marks those checks as not run and points the roadmap to the active
record without changing production behavior.

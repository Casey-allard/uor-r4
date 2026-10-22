# ORDER: infra-loop-pr11-progress-window, deepseek-2 (loop PR 11 repair: answer the CHANGES review)

Lab: deepseek. Item: [infra-loop-pr11-progress-window](../../../../roadmap/items/infra-loop-pr11-progress-window.md) (#2916). Setup row: 11. Base: `a8bec0a` (the reviewed head).

## Done in deepseek-1 and deepseek-2 (history)

- deepseek-1: ORDER `1a157a2cd`; the ORDER-named warnings raised `1e76fa688`; the progress window `228e8de64`; REPORT and the item marked done `c136e88c1`; ORDER trimmed `dc4a5a5d6`; receipts `381fe35d1`; roadmap re-rendered `a8bec0af6`.
- Claude's fresh-context review returned CHANGES at `a8bec0af6` (D59).
- deepseek-2: the repair `6f0319016`; the `replace` break kind `772e57e82`; the repair REPORT `780371416`; merge-forwards `8ba4a239d` and `9fdaa28cc`. The round-1 repair passed review at `780371416`.
- The deepseek-1 ORDER.md was removed so this PR adds exactly one ORDER; its REPORT stays as history and points here.

## Goal

Answer every point of the CHANGES review in [that record](https://github.com/UOR-Foundation/uor-r4/pull/2930#issuecomment-6328863159), in the review's own lens order.

## Cancel probe (run first)

The CHANGES record is counted at `a8bec0af6` (`loop review --check --pr 2930` says so), so the repair is needed. Both HIGH findings reproduce: this branch's binary exits 1 from `loop arrive` with FAIL hooks and FAIL checkout, and deleting the give-up branch from `watch_draw` leaves all 12 window tests green. Class: instrument, fixed here.

## Files in scope

- `crates/uor-r4-harness/src/looparrive.rs`, `loopgates.rs`, their tests.
- `docs/harness/README.md`, `docs/harness/gates.json`.
- `docs/harness/claims.json` (restore it to `origin/main`).
- `docs/loop/tasks/infra-loop-harness/deepseek-2/**`.

## Done when

- **HIGH 1a:** `hooks` is WARN again until installing the shared hook is sanctioned (loop PR 12); its detail says so.
- **HIGH 1b:** `checkout` FAILs only when behind `origin/main` or when HEAD or `origin/main` cannot be read; a HEAD ahead of or not in `origin/main` PASSes, so arriving on a task branch does not fail. A test pins each case.
- **HIGH 2:** the window's loop is split from the file and the clock and takes an injected give-up duration and writer. Tests pin that an unchanged status file prints the giving-up line and returns, that a finished run prints the closed line, and that a red prints as it lands. The `git-hooks`-style planted break is re-aimed at this fault: deleting the give-up branch makes a normal test fail, and `--break` catches it with its own text.
- **MEDIUM:** `last_seconds` ignores UNKNOWN ledger rows, so an early stop cannot zero the estimate. `docs/harness/README.md` no longer contradicts arrive.
- **LOW:** `docs/harness/claims.json` is not changed by this PR; the watch test fixture strips every `GIT_*` variable; the five-minute give-up on a long single gate is named in the docs; the pr-check read-path consequence is noted in the REPORT.

## Stop and ask if

- A review point turns out to be an owner call (a bar, a hold, a cap) rather than a code fix; a file outside scope must change; the repair needs a new dependency.

Size estimate: about 250 lines.

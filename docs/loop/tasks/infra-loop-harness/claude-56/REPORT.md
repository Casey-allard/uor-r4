# REPORT: infra-loop-harness, claude-56 (the figures timeline window runs to the end of October)

Lab: claude. Base: `6ab26955b`, to be merged forward to `origin/main` after #3020 (claude-55, the scope entry). Order: [ORDER.md](ORDER.md), committed first (`a016db7db`). The cancel probe stands: `TIMELINE_LAST` was `"2026-10-19"`, and a record dated 2026-10-22 failed the timeline check.

## What did not work, first

- **`figures` does not render the charts.** It covers 8 geometry figures. The charts are `figures-charts`, which runs in check mode by default; `--write` writes them. Only `research/timeline.svg` changed.
- **The end date is a fixed constant again.** It is now 2026-10-31, so records from 1 November onward will fail the same way. A follow-up should derive the window's end from the catalog's latest date, or extend it each month.
- **Phase E now disagrees with `docs/research/history.md`.** The page dates phase E "10 to 15 October" and states the period through 19 October. The chart's band now runs to 31 October, as Kimi's PM call asked ("the `TIMELINE_LAST`/phase-E extension"). The page is outside this scope, so the mismatch is a follow-up for whoever owns it. A later phase F would be a content decision.
- **One existing test had its date moved.** `a_record_outside_the_timeline_window_is_refused_not_dropped` used 2026-10-20 as its late date, which is now inside the window. It now uses 2026-11-01, because the specified window changed. The refusal itself is unchanged.

## What changed

| Commit | What |
|---|---|
| `a016db7db` | This ORDER, alone. |
| `239430550` | `xtask/src/figures_charts.rs`: `TIMELINE_LAST` and phase E's last day become `2026-10-31`. The refusal test's late date moves to 2026-11-01. A new test, `a_record_from_late_october_is_inside_the_window`, accepts PR 8's date. `docs/figures/research/timeline.svg` is re-rendered: the x positions scale to the longer axis, and the end label moves from 19 Oct to 31 Oct. |
| this REPORT's commit | This REPORT. |

## Measurements

None. This is a tooling change.

## Checks run at the head

- `cargo test -p xtask`: 211 passed, one of them new.
- **Planted fault:** with `TIMELINE_LAST` restored to `2026-10-19`, `a_record_from_late_october_is_inside_the_window` fails. After restoring, `cmp` matched and all tests pass.
- `cargo xtask figures-charts`: all 4 chart figures are current. `cargo xtask figures`: all 8 are current, unchanged.
- **PR 8's tree** (`e05f9feeb`), merged locally with `main` and this branch, nothing pushed:
  - `figures-charts` no longer reports the window refusal, only the ordinary stale charts from PR 8's new rows;
  - after `figures-charts --write`, all 4 are current;
  - so PR 8 needs `cargo xtask figures-charts --write` after merging `main`.
- `loop report --check` and `loop pr-check --local`: see the PR body.

## Skills used

scope-guard: one source file and one chart, inside the scope claude-55 adds.

## Still open

- **PR 8:** merge `main`, then run `cargo xtask figures-charts --write` and `docs-catalog`.
- **Follow-ups:** a data-derived timeline end, and reconciling `docs/research/history.md`'s phase dates.

## Plain summary

The research timeline chart stopped on 19 October, so it refused PR 8's newer records. It now runs to the end of October, and PR 8 can re-draw it.

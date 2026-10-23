# REPORT: infra-loop-harness, claude-55 (the umbrella's scope gains the figures timeline)

Lab: claude. Base: `6ab26955b`. Order: [ORDER.md](ORDER.md), committed first (`aaa9b8fd2`). The cancel probe found both paths absent from the umbrella's `scope:` and from every other item's scope, so the task stands. No open PR changes `xtask/src/figures_charts.rs` or `docs/figures/**`.

## What did not work, first

- **The figures timeline sat outside every fitting scope.** PR 8's docs CI fails on `TIMELINE_LAST = "2026-10-19"` (`xtask/src/figures_charts.rs:105`). No item's `scope:` lists that file or the charts it renders. Kimi's PM call (#2028, 6354929619) took the two-step path, as for #3010/#3011, so this widening comes first.

## What changed

| Commit | What |
|---|---|
| `aaa9b8fd2` | This ORDER, alone. |
| `acaa08823` | `docs/roadmap/items/infra-loop-harness.md`: `xtask/src/figures_charts.rs` and `docs/figures/**` are appended to `scope:`. |
| this REPORT's commit | This REPORT. |

## Measurements

None.

## Checks run at the head

- `cargo xtask roadmap check`: passes.
- `roadmap render --check`: current. The scope is not part of the render.
- `loop report --check` and `loop pr-check --local`: see the PR body.

## Skills used

scope-guard: one scope line.

## Still open

- **claude-56:** extend `TIMELINE_LAST` and phase E's end date in `figures_charts.rs`, and re-render the charts under `docs/figures/`. A planted catalog record past the old window must fail before the change and pass after it. After it lands, PR 8 merges `main`.

## Plain summary

The tool that draws the research timeline cut off on 19 October, which blocks PR 8. The umbrella task can now touch that tool. The next PR moves the cut-off.

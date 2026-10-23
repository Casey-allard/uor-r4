# ORDER: infra-loop-harness, claude-55 (the umbrella's scope gains the figures timeline)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 8. Base: `6ab26955b`.

## Goal

PR 8's last docs blocker is the figures timeline window (DeepSeek's card, #2028 6353084722). `xtask/src/figures_charts.rs:105` sets `TIMELINE_LAST = "2026-10-19"`, and the docs check fails any catalog record dated after it:

> a catalog record dated outside the timeline window 2026-06-10 to 2026-10-19 would be left off the chart: STATUS.md; extend TIMELINE_LAST and phase E

Every catalog append after 19 Oct hits this.

Kimi's PM call on #2028 (6354929619): "Timeline window — PM call: yes, the two-step path (#3010/#3011 pattern). Claude, please author both steps: first the scope entry covering `xtask/src/figures_charts.rs` and `docs/figures/**`, then the `TIMELINE_LAST`/phase-E extension with the charts re-rendered."

This unit is the first step: it appends `xtask/src/figures_charts.rs` and `docs/figures/**` to `infra-loop-harness`'s `scope:`. claude-56 makes the change.

## Cancel probe (run first)

At `6ab26955b`, are both paths absent from `infra-loop-harness`'s `scope:`, and from every other item's scope? If either is already listed, the task is void. Class: instrument.

## Files in scope

- `docs/roadmap/items/infra-loop-harness.md` (inside its own `docs/roadmap/items/*.md`)
- `docs/roadmap/README.md` (shared render)
- This task folder

## Done when

1. The item's `scope:` lists `xtask/src/figures_charts.rs` and `docs/figures/**`, and the item diff is that one line.
2. `roadmap check` and `render --check` pass.
3. `report --check` and `pr-check --local` pass.

## Stop and ask if

- Another open PR changes `xtask/src/figures_charts.rs` or `docs/figures/**`.

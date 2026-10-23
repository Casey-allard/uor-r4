# ORDER: infra-loop-harness, claude-56 (the figures timeline window runs to the end of October)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 8. Base: `origin/main` once claude-55 (#3020, the scope entry) has merged.

## Goal

Step 2 of Kimi's PM call (#2028, 6354929619): "the `TIMELINE_LAST`/phase-E extension with the charts re-rendered".

`xtask/src/figures_charts.rs` refuses any catalog record dated after `TIMELINE_LAST = "2026-10-19"`. PR 8's two new catalog rows are dated 2026-10-22, so its docs CI fails.

This unit:
- sets `TIMELINE_LAST` and phase E's last day to **2026-10-31**, the end of the month, so that each new day's records do not break the check again;
- re-renders the charts under `docs/figures/` with `cargo run -p xtask -- figures`;
- keeps the refusal itself: a record dated after the window still fails, and is never dropped from the chart.

## Cancel probe (run first)

At the base, is `TIMELINE_LAST` still `"2026-10-19"`, and does a catalog record dated 2026-10-22 fail the timeline check? If not, the task is void. Class: instrument.

## Files in scope

- `xtask/src/figures_charts.rs` (from claude-55)
- `docs/figures/**` (from claude-55)
- This task folder

## Done when

1. `TIMELINE_LAST` and phase E's last day are `2026-10-31`.
2. **Tests.** A record dated 2026-10-22 is accepted, and a record dated 2026-11-01 is refused, naming the record. The existing refusal test moves its late date past the new window, because the specified window changed.
3. **Planted fault:** with `TIMELINE_LAST` restored to `2026-10-19`, the 2026-10-22 test fails.
4. **Charts.** `figures --check` passes on the re-rendered charts. On PR 8's tree merged with this branch, the timeline check passes.
5. `cargo test -p xtask`, `report --check` and `pr-check --local` pass.

## Stop and ask if

- Re-rendering changes a chart beyond the date axis and phase E's band.
- `docs/research/history.md`, which states phase E's dates, needs an edit. It is outside this scope, so the mismatch is recorded as a follow-up instead.

# REPORT: m1-d11-exporter-discrete, kimi-1

Lab: kimi. Order: [ORDER.md](ORDER.md). References #2838.

## What did not work, first

- #2964 merged at 12:57:59Z (as `194b75592`, on deepseek's counted APPROVE at
  `91cd9d613`) with the item flipped to `done` but **without re-rendering** the
  roadmap page, so main's `docs/roadmap/README.md` was left stale: counts 43/15
  and the item listed as ready, against the item files' truth of 41/17. Kimi's
  CHANGES review (naming exactly this staleness plus a one-line REPORT count)
  posted ~100 seconds after the merge — post-merge, not a queue race. The
  substance of #2964 was verified clean in that review (both zero-mul audits
  reproduced, byte-identical round-trips), so the drift is record-only, L-001
  class, caught within the hour.

## What changed

A re-render of `docs/roadmap/README.md` from the committed item files
(`cargo run -q --locked -p xtask -- roadmap render`): counts 41 ready / 17 done,
and `m1-d11-exporter-discrete` moved from the ready tables into the done table.
Generated class; 3 insertions, 4 deletions across 4 hunks — exactly the item's
row and the two count lines, as the ORDER's stop condition requires.

## Measurements

- Cancel probe at base `d7d9d8d61`: `roadmap check` passed while the README read
  `ready | 43` / `done | 15` with the item in the ready table — stale against the
  item file's `status: done`. Task stood.
- The rendered diff was inspected hunk by hunk: only the two count lines and the
  one item row moved; no other row changed (the ORDER's stop-and-ask condition).

## Checks run at the head

- `cargo run -q --locked -p xtask -- roadmap render` — this PR's content; the
  README is byte-identical to a fresh render at the head.
- `cargo run -q --locked -p xtask -- roadmap check` — recorded before push.
- `uor-harness loop gates --quick`, `uor-harness loop gates --full`,
  `uor-harness loop report --check`, `uor-harness loop pr-check --local` —
  recorded before push.

## Skills used

None.

## Still open

- The two record repairs named in kimi's post-merge CHANGES on #2964 belong to
  antigravity: the REPORT's discrete lib count (212 vs measured 211 at
  `91cd9d613`) is immutable on main — an errata note on #2838 records the true
  number. The stale `driver_source_identity` pin (pre-existing since #2782) is
  carded on #2028 and needs a decision row before anyone refreshes it.
- The weekly PM render cadence continues; `loop render --pr` is still to build.

## Plain summary

The D11 exporter pull request merged with its roadmap item marked done, but
nobody re-ran the command that regenerates the status page, so the page still
listed the item as ready with yesterday's counts. This pull request is exactly
that re-run, so the public status page matches the item files again.

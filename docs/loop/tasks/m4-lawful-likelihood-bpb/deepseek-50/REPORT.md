# REPORT: m4-lawful-likelihood-bpb, deepseek-50 (residual record clean-up)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2861. Base: `903dbf701`. `References #2861`.

## What did not work, first

- **PR #3060 merged before the last repair commit was included.** The queue merged `23eb1c275` (which carries the single-interval outcome rule), but my final residual commit `62710d558` was pushed after the merge and never landed; the stale remote branch was deleted and this small docs PR re-lands exactly those three corrections.

## What changed

| File | Change |
|---|---|
| `docs/labs/m4-lawful-likelihood-bpb/README.md` | control 7 states `MassZero`/not-comparable; the old "infinite and reported as such" wording is gone |
| `docs/harness/prep/m4-lawful-likelihood-bpb-2026-10-23.md` | header cites the design PR #2940 |
| this folder | ORDER, this REPORT (the merged `deepseek-49` REPORT keeps its historical "no D40 release exists" sentence; it is superseded by this PR and CHARTER §5.1, and editing another order's report is outside scope) |

## Measurements

- None claimed: this PR corrects three doc sentences; no instrument, stream hash or result changes. The sheet's `TEST FITNESS: PENDING` is unchanged.

## Checks run at the head

`prep-check` (sheet present/complete/PENDING), `loop report --check`, `loop pr-check --local`, `loop gates --full`; docs-only so a D46 fresh-context record applies.

## Skills used

- `scope-guard`: three doc corrections only, all inside the item scope; no content, bar, stream hash or FIT status touched.

## Still open

- The sheet's non-author `TEST FITNESS: FIT` (Claude/Kimi/antigravity) for the registered GPU run — unchanged by this PR.

## Plain summary

- Lands the three doc residuals the #3060 merge missed: no "infinite" zero-count wording, §5.1 owner release, and the design PR citation.
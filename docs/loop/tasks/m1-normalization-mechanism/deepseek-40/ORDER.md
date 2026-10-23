# ORDER: m1-normalization-mechanism, deepseek-40 (pin the registered bar and the full DEV pair)

Lab: deepseek. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md) (#2842). Base: `26e19ac5b`. Item PR (partial: `References #2842`; the item stays `ready`).

## Goal

Update the normalization mechanism's draft sheet for the two events that landed after my non-author `NOT_FIT` review (`deepseek-37`, PR #3023): the owner's standing-direction decision at **2026-10-23T02:36:36Z** registered this item's bar exactly as proposed (`#2028` comment `6356034555`; card `6338596024`), and the authoritative full DEV pin is known from the code (`attention_learning.rs`, tokens `0919e90f953e…`, mask `a838bb1a67e9…`; 42,856 payload tokens / 14,287 targets).

This unit records the registration with its ISO-UTC time, pins the full split digests in the sheet, records the staging-sheet errata as a PM action, and keeps `NOT_FIT` until a non-author FIT (per the registration, Claude or Kimi — deepseek has authored text in this sheet). It clears no compute.

## Cancel probe (run first)

Class **instrument** (docs only; no GPU, no training). At the base: the sheet still calls the bar comment a decision card with no owner answer and its §5 carries only DEV prefixes. Task stands.

## Files in scope

- `docs/harness/prep/m1-normalization-mechanism*` (the sheet)
- `docs/labs/m1-normalization-mechanism/**` (lab record)
- this task folder

## Done when

- The sheet's Status line and §3 quote the owner's registered bar with the `2026-10-23T02:36:36Z` time and the decision comment id; §5 carries the full TRAIN/DEV digests from the code pin and the staging-sheet errata note, and ties the fresh-V2 prefixes to the parent's registered record.
- The Verdict records what is now resolved (bar, DEV pin) and what remains (staging errata, non-author FIT); `TEST FITNESS: NOT_FIT` stands and no compute is cleared.
- `loop report --check`, `loop pr-check --local`, `loop gates --full` and a fresh-context review record at the head pass (docs-only PR).

## Stop and ask if

- A freeze/hash would have to be invented (none is needed for this item).
- A file outside the scope above must change.
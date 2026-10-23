# ORDER: m1-position-mechanism, deepseek-39 (pin the registered bar and the full DEV pair after the owner registration)

Lab: deepseek. Item: [m1-position-mechanism](../../../../roadmap/items/m1-position-mechanism.md) (#2843). Base: `6ffc91617`. Item PR (partial: `References #2843`; the item stays `ready`).

## Goal

Update the position mechanism's draft sheet for the two events that landed after my non-author `NOT_FIT` review (`deepseek-36`, PR #3018): the owner's standing-direction decision at **2026-10-23T02:36:36Z** registered this item's bar exactly as proposed (`#2028` comment `6356034555`; the card is `6337522273`), and the authoritative full DEV pin is known from the code (`attention_learning.rs`, tokens `0919e90f953e…`, mask `a838bb1a67e9…`; 42,856 payload tokens / 14,287 targets).

This unit records the registration with its ISO-UTC time, pins the full DEV pair in the sheet, defines the multi-occurrence order-sensitive slice's selection rule and marks its hash slot `PENDING` until the frozen subset is filed, and keeps `NOT_FIT` until a non-author FIT (per the registration, Claude or Kimi). It clears no compute.

## Cancel probe (run first)

Class **instrument** (docs only; no GPU, no training). At the base: the sheet still calls the bar comment a decision card with no owner answer, its §5 carries only DEV prefixes, and its Verdict's item 3 leaves the slice undefined. Task stands.

## Files in scope

- `docs/harness/prep/m1-position-mechanism*` (the sheet)
- `docs/labs/m1-position-mechanism/**` (lab record)
- this task folder

## Done when

- The sheet's Status line and §3 quote the owner's registered bar with the `2026-10-23T02:36:36Z` time and the decision comment id; §5 carries the full DEV token/mask pair from the code pin and the staging-sheet errata note.
- The multi-occurrence slice rule is stated deterministically (query token appears ≥2 times in the prefix with distinct successors; `N ≥ 200` on the DEV panel) with the hash slot `PENDING — frozen subset`, so the remaining work is data freeze, not definition.
- The Verdict records what is now resolved (bar, DEV pin) and what remains (slice hash, staging errata, non-author FIT); `TEST FITNESS: NOT_FIT` stands and no compute is cleared.
- `loop report --check`, `loop pr-check --local`, `loop gates --full` and a fresh-context review record at the head pass (docs-only PR).

## Stop and ask if

- A freeze/hash would have to be invented (it does not: the hash stays PENDING for the frozen subset).
- A file outside the scope above must change.
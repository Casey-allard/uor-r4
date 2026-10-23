# ORDER: m1-position-mechanism, kimi-32

Lab: kimi. Item: [m1-position-mechanism](../../../../roadmap/items/m1-position-mechanism.md). Base: `127632bc3`.

## Goal

The item's done-when reads:

> A position mechanism with finite controls that obeys CHARTER §3 (geometry only, no softmax or squashing, no matmul), plus a pre-registered test that it helps order. The bar for "helps order" is not yet registered; step 4 registers it.

## Cancel probe (run first)

The three former FIT blockers are resolved: the owner registered the bar at 2026-10-23T02:36:36Z, §5 pins the full DEV pair, and #3046 froze the 1,620-row order slice as `f76dc5c7ff6c359dd4c9158abc280ff29105cb611f6c4c2bdba6fdfed0035b3a`. The sheet's current status and post-registration update still say `NOT_FIT` and hash `PENDING`, contradicting §5. This is an instrument/documentation repair, not a mechanism result; retain the earlier verdict as history and add Kimi's non-author post-freeze FIT review. The roadmap item remains `ready`: GPU wiring and the registered order test are not part of this order.

## Files in scope

- `docs/harness/prep/m1-position-mechanism*`
- `docs/loop/tasks/m1-position-mechanism/kimi-32/**`

## Done when

The post-freeze resolution and current FIT status are recorded without rewriting the prior review; the exact DEV and slice identities are independently checked; the five finite position controls pass; the REPORT passes `loop report --check`; full gates and local PR checks pass; and an other-vendor fresh-context reviewer approves the exact final head. The PR says `References #2843`, leaves the item `ready`, and clears no hold or GPU run.

## Stop and ask if

A source-code change, GPU work, an owner decision, a roadmap-status change, or any file outside the stated scope becomes necessary.

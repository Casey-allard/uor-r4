# REPORT: infra-loop-harness, claude-83

Lab: claude. Order: [ORDER.md](ORDER.md).
References #2904.

## What did not work, first

- **The `any` source picked work alphabetically.** `ready_items` pushed ids in `git ls-tree` order, and `any` offered the first claimable one. So `carrier-credit-hard-selection` and `d41-driver-math-pilot` came before `m1-w2-capacity-capture`. Nothing in the data could override that: no item has a lane or rank. The charter's rule, "in order of how many items it unblocks" (§5.10), was never computed.
- **The fresh review found two gaps in the first version** (APPROVE with findings).
  - **The test did not pin the milestone key.** Unblock count alone satisfied every assertion, so a sort that dropped the rank key still passed. The new `any_puts_the_milestone_key_before_the_unblock_count` fails with the rank key dropped.
  - **A done dependent lent its rank.** An infra item whose only M1 dependent is already done still ranked as M1. Now a done dependent is walked through but lends no rank.
- **The `lane` source does not use the new order.** It re-sorts its own candidates by `(rank:, id)`. No item has a lane, so it never decides today, and it is left as is.
- **One full-suite run showed the new test failing.** I had set aside the rank key for the negative gate in the same target folder while the suite ran. The clean re-run is recorded below.

## What changed

- **`crates/uor-r4-harness/src/loopnext.rs`:**
  - `ready_items` reads each item once (`origin_file` plus `loopclaim::parse_item`) and builds the dependents graph.
  - It returns `ReadyItem { id, rank, unblocks }` sorted by effective milestone rank, then unblock count descending, then id.
  - The ranks are M1–M8 = 1–8, research = 9, infra = 10, anything else = 11. The effective rank is the smallest rank over the item and its not-done transitive dependents. A visited set guards against cycles.
  - The held count, the waiting list, the scope requirement and the dependency-done check are unchanged. So is the order of the sources.
  - The `any` reason names the rank, for example `(M1 path, unblocks 6)`.
- **`crates/uor-r4-harness/tests/loop_next_sources.rs`:** two new tests.
  - `any_offers_the_milestone_path_item_that_unblocks_most_first`: the M1 item two others need beats alphabetically earlier infra items; without it, the needed infra item beats the unneeded one.
  - `any_puts_the_milestone_key_before_the_unblock_count`: as above.
- **`docs/loop/README.md`:** one sentence on how `any` ranks.

## Measurements

- None on the real items yet. The order depends on #3171, which makes the M1 path items ready and scoped. After both merge, `loop next` on a free lab is checked on main and recorded on #2904.

## Checks run at the head

- `loop_next_sources`: 16 passed. `loop_next`: 33 passed.
- **Negative gates:**
  - With the `loopnext.rs` ranking set aside, the first new test FAILS.
  - With only the rank key dropped from the sort, the second new test FAILS.
- `cargo fmt` and `clippy -D warnings`: pass.
- The full harness suite with `--no-fail-fast`: 33 targets ok. The only failure is the environmental `loop_guard::each_clients_canary_write_is_refused`, whose worktree-ownership reason comes from main.

## Skills used

- None.

## Still open

- None for this unit. Labs on the M1 path still need each GPU step's own pre-run review (D67).

## Plain summary

When a lab asks what to do next and nothing more urgent is waiting, it now gets the item that is earliest on a milestone path and unblocks the most other work, instead of the first name in the alphabet. Infrastructure work comes up only when a milestone item depends on it.

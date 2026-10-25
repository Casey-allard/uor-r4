# ORDER: infra-loop-harness, claude-83 (loop next ranks by milestone path and unblock count)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 6. Base: `460d1e413`.

## Goal

CHARTER §5.10 says to take work "in order of how many items it unblocks", and the owner asked that runs which "unlock further work" go first. The code does something else. `loop next`'s `any` source offers ready items in **id order** (`loopnext.rs` `ready_items`, then `for id in &ready`), so `carrier-credit-hard-selection` comes before `m1-w2-capacity-capture`. No lane or rank data exists to override that. This unit makes `ready_items` return its ready list ranked, and the `lane` and `any` sources then offer the top claimable item.

**The ranking key:** effective milestone rank, then unblock count descending, then id.
- **Unblock count** is the number of distinct not-done items that depend on the item transitively through `depends_on`, read from the item files on `origin/main`.
- **Effective milestone rank** is the smallest milestone rank among the item and all of its transitive dependents. The ranks are M1 = 1 … M8 = 8, `research` = 9, `infra` = 10, and missing or other = 11. So an infrastructure item that a milestone item depends on ranks with that milestone, and one that nothing needs ranks last.

The `any` reason line names the rank, for example `item m1-w2-capacity-capture is ready, scoped and claimable (M1 path, unblocks 6)`.

## Cancel probe (run first)

At `460d1e413`, `ready_items` pushes ids in `git ls-tree` order, and no sort follows. The unit stands. Class: instrument (work selection).

## Files in scope

- `crates/uor-r4-harness/src/loopnext.rs`
- `crates/uor-r4-harness/tests/loop_next*.rs`
- `docs/loop/README.md` (the ranking sentence)
- `docs/loop/tasks/infra-loop-harness/claude-83/**`

## Done when

- **The new ranking test passes and pins the change.** A fixture with an alphabetically early infra item nothing needs, an M1 item two others depend on, and an infra item an M1 item depends on must get the M1 item offered first, then the needed infra item, then the unneeded one. The test fails with the sort reverted.
- **Nothing else changes.** Existing `loop_next` tests pass unchanged, except where they assert id order.
- **The checks pass.** Harness fmt, clippy and tests, `loop report --check`, `loop pr-check --local` and the quick gates.
- **Review.** A fresh-context review at the exact head counts (D66). The PR says `References #2904`.

## Stop and ask if

The ranking would change which sources run before `any`: repairs, reviews, GPU collections, premises and owner directions keep their order.

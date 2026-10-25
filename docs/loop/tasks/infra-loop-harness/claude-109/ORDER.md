# ORDER: infra-loop-harness, claude-109 (Stage T waits on m1-address-key-full-vector)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file maintenance, as in claude-104 to -108). Base: `d48ef7887`.

## Goal

One item-file fix that carries out a D68 sequence decision: add `m1-address-key-full-vector` to the `depends_on` of `m1-bvg-k-stage-t-training` (#2836), with one note.

**Why.**
- The decision card is claude-107's [DECISION.md](../../m1-bvg-k-stage-t-training/claude-107/DECISION.md) (#3226). It finds:
  - Stage T's warrant clause, "only warranted if W2 blames the learning law (H_law)", is not met;
  - W2's registered row 3 for H_mixed says "do not change the learning law yet";
  - the code-utilization diagnostic routes A1a to `m1-address-key-full-vector` (#2834), "the next M1 mechanism unit".
- So Stage T waits on #2834, and stays on the M1 close (D79).
- `loop next` ranks only on `depends_on` and status, not on prose warrant clauses. Without this edit, Stage T keeps ranking first ("unblocks 12") and any lab could claim it against the routing.

**Rejected alternatives.**
- **Set Stage T `held`:** `held` means held work waiting on a pre-run review, not a sequence dependency. The dependency names exactly what Stage T waits for, and it clears on its own when #2834 is done.
- **Leave it to the prose:** that is what let `loop next` offer it today.

**Why the infra-loop-harness item.** Its scope holds `docs/roadmap/items/*.md`. Codex's idle #3169 holds an order on it, and #3169's files do not overlap this unit's. So this ORDER is hand-written under the run file's umbrella exception.

## Cancel probe (run first)

Class: records. Run at `d48ef7887`; all must hold, or the unit is cancelled:
- Stage T's `depends_on` does not hold `m1-address-key-full-vector`. **Holds** (line 9 lists the four done items).
- `m1-address-key-full-vector` exists and is not `done`, so the dependency is live. **Holds:** `status: ready`.
- The dependency makes no cycle: #2834's own `depends_on` chain does not reach Stage T. **Holds:** it is `[m1-w2-capacity-capture]` (done).
- No open PR other than #3226 touches the Stage T item file. **Holds:** none does (#3226 does not touch it either).

Result: all hold, so the unit goes ahead.

## Files in scope

- `docs/roadmap/items/m1-bvg-k-stage-t-training.md` (the `depends_on:` line and one note)
- `docs/roadmap/README.md` (the render, if it changes)
- `PLAN.md` line 61, Stage T's "Needs" cell, so the plan matches the item graph. This was added after the exact-head review's advisory. Codex's open #3169 edits only PLAN.md lines 21–27.
- this task folder

## Done when

- The edit is made, and `roadmap check` passes.
- A fresh decision review of claude-107's card (round 2) and an exact-head review of this diff approve.
- `loop report --check` and `loop pr-check --local` pass.
- After the merge, `loop next` no longer offers Stage T while #2834 is open.
- The REPORT says `References #2904` and `References #2836`.

## Stop and ask if

Stop and decide, never stop and wait.

- The round-2 review finds the decision outside a lab's authority. The owner's call is D79, which this does not change.

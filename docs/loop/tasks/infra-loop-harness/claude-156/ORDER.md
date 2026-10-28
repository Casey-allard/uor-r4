# ORDER: infra-loop-harness, claude-156

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md), as the roadmap umbrella: it is the item whose scope holds `PLAN.md` and `docs/roadmap/items/*.md`, as claude-147 used it to add `pipeline-map`. Base: `ac4f445b9`.

Setup row: 2 (item-file maintenance: new gap items and PLAN's owner-direction rows).

## Goal

The owner's decisions of 28 Oct 2026, in claude's chat, after claude-155's diagnosis (its [REPORT](../../in-space-trunk-to-text/claude-155/REPORT.md), "The pivot"):

> "ok yes, my vote is to pivot to your diagnosed recommended new roadmap"

> "determinism is still probably the goal but like you said we can do the training in float and maybe just export the integer exact... either way it is worth looking into the archived hopf golden ratio complex - riemannian bridge we used for the original prime router or angular router"

> "I don't care what we serve at long as it is cheap"

The diagnosed roadmap, from that REPORT:
1. **A per-stage cost profile** of one training step and one served token.
2. **A revision of the [pipeline map](../../../../model/pipeline-map.md).** The owner's 27 Oct refocus direction is still open in PLAN. The revision adds:
   - the results of claude-152 to claude-155;
   - the storage diagnosis (a trunk of 64,000 values; associations need memory);
   - cost per token ranked first, as the owner's criterion;
   - the archived prime and angular router's record;
   - and a reordered plan.
3. **One decisive run: route plus exact memory.** A key formed from the user's turn reads a sparse exact memory built from TRAIN. The read lands where the readout reads, against a no-memory control (L-010).

This unit writes those three as roadmap items, and records the three decisions in PLAN's owner-direction table. It writes no code and runs nothing.

## Cancel probe (run first)

Run 2026-10-28 against `origin/main` `ac4f445b91136cd430cb17307c9664cc71df551c`. **Not cancelled (instrument: a search of the item folder, PLAN and the open PRs).**

- **No item for any of the three exists.** `docs/roadmap/items/` holds `pipeline-map` (`status: done`, delivered by claude-148, which predates claude-152 to claude-155) and no cost-profile or route-plus-memory item. The map's own U9 (profile) and U6 (store to prediction) are rows in its order-of-work table, not items.
- **PLAN** has the 27 Oct refocus direction `open`, and none of the three 28 Oct decisions.
- **No overlap.** The open PRs are this unit's draft, #3344, and antigravity's #3323, which touches no file here.

## Files in scope

- `docs/roadmap/items/pipeline-map-revision.md` (new)
- `docs/roadmap/items/fbs-cost-profile.md` (new)
- `docs/roadmap/items/route-plus-memory.md` (new)
- `PLAN.md`: the owner-direction rows, and the three items under "Next to run on the side lines"
- `docs/loop/tasks/infra-loop-harness/claude-156/**`

## Done when

1. The three item files exist, each with:
   - its "why", quoting the owner verbatim;
   - what is missing, with sources;
   - a done-when that a later unit can meet or fail;
   - a scope;
   - `status: ready`.
2. PLAN's owner-direction table carries the three 28 Oct decisions verbatim, newest first. The three items are listed as the line's next units.
3. `uor-harness check` passes for the item and PLAN rules, and the result is on main.

## Stop and decide (never stop and wait)

- **An item would change a §3 constraint:** it does not. The owner's 28 Oct answers keep float training with an exact integer export.
- **A harness rule refuses an item's shape:** fix the item to the rule, and note it in the REPORT.

# ORDER: pipeline-map-revision, claude-159

Lab: claude. Item: [pipeline-map-revision](../../../../roadmap/items/pipeline-map-revision.md). Base: `e4dc1f924`.

## Goal

The owner, 28 Oct 2026, in claude's chat:

> "ok yes, my vote is to pivot to your diagnosed recommended new roadmap"

> "determinism is still probably the goal but like you said we can do the training in float and maybe just export the integer exact... either way it is worth looking into the archived hopf golden ratio complex - riemannian bridge we used for the original prime router or angular router"

> "I don't care what we serve at long as it is cheap"

The item's done-when reads:

> 1. `docs/model/pipeline-map.md` is revised:
>    - every stage's fields carry the results of claude-149 to claude-155, with sources;
>    - each stage's cost per token comes first, measured where a profile exists (the `fbs-cost-profile` item) and labelled as arithmetic otherwise;
>    - the gaps and the order of work are re-ranked by what blocks a cheap, capable model first, with storage, the served decoder and the vocabulary search placed on the evidence.
> 2. A section reads the archived prime and angular router, from the files above and the commits they name:
>    - what it computed, and what was measured;
>    - what it would cost per token today;
>    - which of its pieces (Hopf oracle, zeta phases, φ couplings, choice rule) fits which stage, and in which domain.
> 3. The revision is merged, and PLAN's 27 Oct refocus direction is set to `done` in the same PR. One `[claude, for Casey]` note on #2028 gives its link, the re-ranked top five gaps and the first unit.

The `fbs-cost-profile` item is done (claude-157, claude-158), so both profiles exist. Each stage's cost comes from them where they measured it. claude-156 to claude-158 are folded in as well.

## Cancel probe (run first)

Run at `e4dc1f924`:
- No open PR touches `docs/model/pipeline-map.md` or `PLAN.md`.
- No other lab's ORDER names `pipeline-map-revision`.
- The item is `ready` on `origin/main`.

All three held. **Not cancelled** (instrument: none needed).

**Found by the probe.** `cargo xtask roadmap render --check` fails on main with 11 problems. All of them are in the three items claude-156 wrote:
- **The three items** lack the sections `## How to start`, `## Rules that apply` and `## Sources`.
- **`route-plus-memory`:** its title is 85 characters, over the limit of 80, and its area `memory` is not an allowed one.

This unit fixes its own item (`pipeline-map-revision.md`), which is in its scope. The other two are outside this scope. They need an item-maintenance unit through `infra-loop-harness`, which cannot be claimed while this unit holds an item file (`docs/roadmap/items/*.md` overlaps). So that unit runs right after this one merges, and this REPORT names it.

## Files in scope

- `docs/model/pipeline-map.md`
- `PLAN.md`
- `docs/loop/tasks/pipeline-map-revision/**`
- `docs/roadmap/items/pipeline-map-revision.md`

## Done when

1. The item's done-when 1 and 2: the map is revised as quoted, and the archived router has its own section.
2. PLAN's 27 Oct refocus direction is `done` in this PR.
3. `pipeline-map-revision.md` passes the roadmap check's section rules, and the item is `done`.
4. The PR is merged and verified, and the `[claude, for Casey]` note is posted on #2028.

## Stop if

- **A claim in the map cannot be sourced.** It is labelled hypothesis or removed, never kept unsourced.

This is document work: no GPU run and no pre-registration (CHARTER §4.5). No step waits on Casey.

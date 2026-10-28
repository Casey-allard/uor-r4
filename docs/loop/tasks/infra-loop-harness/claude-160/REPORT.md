# REPORT: infra-loop-harness, claude-160

Lab: claude. Order: [ORDER.md](ORDER.md). Doc-only: no code, no run, no GPU.

`cargo xtask roadmap render --check` now passes on this branch, and the roadmap is re-rendered. The 8 problems were all in two items claude-156 wrote.

## What did not work, first

- **The loop gates never ran the roadmap check.** claude-156 merged three items that fail `roadmap render --check`, with 11 of 11 gates green and CI green. The check failed only when claude-159 ran it by hand. claude-159 fixed its own item, and this unit fixes the other two.

## What changed

- **`docs/roadmap/items/route-plus-memory.md`:**
  - the title is cut from 85 to 78 characters ("…, read at the readout");
  - the area is `mechanism`, one of the allowed areas, in place of `memory`;
  - `## How to start` names the pipeline map's R1, claude-154 and claude-155, the files the read copies, and the archive's lesson;
  - `## Rules that apply` and `## Sources` are added.

  Its goal, scope and done-when are unchanged.
- **`docs/roadmap/items/fbs-cost-profile.md`:** `## How to start`, `## Rules that apply` and `## Sources`, pointing at `--profile`, `--stages` and both REPORTs. The rest is unchanged.
- **`docs/roadmap/README.md`, re-rendered:**
  - the done count goes from 75 to 78 and the total from 99 to 102;
  - `in-space-trunk-to-text` is now `done`;
  - the rows for the three pivot items and `pipeline-map` → `pipeline-map-revision` are added.

## Measurements

None: this unit runs nothing.

## Checks run at the head

- **`cargo xtask roadmap render --check`:** "README.md is current", exit 0. It failed with 8 problems at the base.
- **`uor-harness loop report --check`:** PASS. **`uor-harness loop gates --full`:** 11 of 11 green.
- **The fresh in-unit review, at `62ea51b8f`:** CHANGES. It confirmed:
  - only the title and area changed in the front matter;
  - all 8 links resolve;
  - every fact matches its source;
  - the README agrees with the items' statuses.

  Its findings, all fixed in the next commit:
  - **medium:** the rules line named "cumsum", which the charter does not. §3.3 bans dense matrix multiplication or GEMM, "including wrappers and hand loops", and the line now says exactly that;
  - **low:** "the training clock adds no synchronization" is now what claude-157 measured: events on the step's stream, with `clock_overhead` measured;
  - **low:** How to start names stage 5 for the readout move as well as stage 7.

## Skills used

- **holding-the-goal.** Format only: no item's goal, scope or done-when moved.

## Still open

- **The roadmap check is not in `loop gates`,** so a malformed item can merge again. Adding it is a harness change, and stays out of this unit; it is noted in the lab's friction log.
- **`route-plus-memory`**, the decisive run, is next.

## Done when

1. **Met.** `roadmap render --check` passes.
2. **Met.** `roadmap render` wrote `docs/roadmap/README.md`.
3. **Met on merge,** and then `loop claim route-plus-memory`.

## Plain summary

Three roadmap items written in the pivot broke the roadmap's own format rules, and no automatic check caught it. This fixes the last two, without changing what any item asks for, and re-draws the roadmap page.

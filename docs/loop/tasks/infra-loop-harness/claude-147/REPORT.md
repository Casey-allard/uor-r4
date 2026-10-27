# REPORT: infra-loop-harness, claude-147

Lab: claude. Order: [ORDER.md](ORDER.md). Item maintenance only: one new roadmap item, its rendered row and one PLAN row. The umbrella item stays open.

**Speed.** No model, run or benchmark is in this unit; nothing to time.

## What did not work, first

- **This unit records the owner's direction and makes no map.** The map is claude-148's work, under the new item.
- **The item has no issue yet.** `cargo xtask roadmap sync` makes the issue from main; that is the PM's weekly run. A sync from a branch would also rewrite other issues from older files, so this unit did not run one.
- **The PLAN row quotes only the direction's first sentence.** The item quotes the whole direction. A table cell holding both paragraphs would not stay readable.
- **The time of Casey's message is not recorded.** It reached this session through the session that started it, which gave the date and not the minute.
- **The umbrella's done-when does not move.**

## What changed

Size against `origin/main`: 3 files of docs plus this task folder, with no code.

- **`docs/roadmap/items/pipeline-map.md` (new).** It records:
  - Casey's 27 Oct direction, verbatim;
  - the three failed look-back forms of 27 Oct that motivate it;
  - a three-part done-when:
    - the page `docs/model/pipeline-map.md`, with seven fields for every stage;
    - the scale arithmetic, the ranked gaps and an order of work, every number sourced;
    - the merge, and a `[<lab>, for Casey]` note on #2028;
  - a cancel line, a `host` done class and a scope covering:
    - the page;
    - `docs/model/architecture.md`;
    - `docs/SUMMARY.md`;
    - `PLAN.md`;
    - its own task folder and item file.
- **`docs/roadmap/README.md`:** the row, rendered by `cargo xtask roadmap render`.
- **Review round 1** (fresh context, at `73af182d7`) asked for changes. All three fixes are in the item:
  - **The motivating failures.** The item had credited them to claude-144 and claude-145. They are claude-146's three forms, now quoted from its research note and from commits `87ade4920` and `6d8bac253`.
  - **The readout move.** The item had said it worked. It now says the move is the form claude-146 moved to, and that its result is not yet recorded (#3332).
  - **The note's tag.** The done-when's note on #2028 is now tagged `[<lab>, for Casey]`, because any lab may take the item.
- **Review round 2** (fresh context, at `157656fb4`) asked for changes to the motivation paragraph. All five fixes are in:
  - **The first form's cause.** Keying on state is now given as a credit fault, as the research note reads it: hard top-K credits only its picks. Only the two copy forms are said to land in a domain the next stage did not read.
  - **Measured, not inferred.** "Never chose the right position" is replaced by the two measurements: recall at chance in every bucket, and openings at the 161 floor through step 8,192.
  - **claude-144.** "Had not moved openings" is replaced by its REPORT's result: the read did not clear its pre-registered openings bar at either seed.
  - **Branch-only commits.** `87ade4920` and `6d8bac253` are now marked as on `claude/hamming-read` (#3332), since they may not resolve once that branch is merged and deleted.
  - **This REPORT's tag.** The done-when line above now reads `[<lab>, for Casey]`.
- **`PLAN.md`:** one row at the top of "Owner directions": the direction's first sentence, verbatim, status `open`, dated 2026-10-27, pointing to the item.

## Measurements

None. This unit makes no measurement.

## Checks run at the head

- `cargo xtask roadmap check`: 97 items OK.
- `cargo xtask roadmap render --check`: current.

## Skills used

- **holding-the-goal.** The owner's words go into the repository verbatim, so the map is judged against what he asked, not a paraphrase.

## Still open

- **claude-148 writes the map** under `pipeline-map`. It merges main once this lands, so its ORDER names an item on main.
- **The item's issue** is made at the next roadmap sync from main.

## Plain summary

This adds one entry to the project's roadmap and one line to its plan. The entry is the owner's 27 Oct request: a map of every step of the model, from the first trained token to the last word of an answer, with the size, hardware and cost of each step. The map itself comes in the next pull request.

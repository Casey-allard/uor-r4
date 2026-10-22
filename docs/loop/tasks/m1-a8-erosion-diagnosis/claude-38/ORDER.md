# ORDER: m1-a8-erosion-diagnosis, claude-38 (the design note for the host-only placement census)

Lab: claude. Item: [m1-a8-erosion-diagnosis](../../../../roadmap/items/m1-a8-erosion-diagnosis.md) (#2829). Base: `77fedb347`. Item PR (partial: `References #2829`; the item stays `ready`).

## Goal

Unit 1 is item step 3: the design note, posted on #2829 and kept as `docs/labs/m1-a8-erosion-diagnosis/DESIGN.md`. In the A8 dose, Relation reach fell from 1,351 to 551 at Q1 and FrozenZero's fell to 857.

The note will:
- list each question the retained host data (64-batch checkpoints and move logs) can answer;
- for each claim, say supported or not supported, given that no admission is recomputed;
- separate the three suspects: the update moves, the read and the instrument;
- state the checkpoint-0 control the census must reproduce: byte-equal checkpoint-0 tables across arms, and G-Init (the table equals the pin word for word);
- name the exact archive files the census reads.

No census code and no GPU work in this unit.

## Cancel probe (run first)

At `77fedb347`, does a design note or census for this item already exist on #2829, #2029 or under `docs/labs/m1-a8-erosion-diagnosis/`? If one does, the task is void. Class: instrument.

## Files in scope

- `docs/labs/m1-a8-erosion-diagnosis/DESIGN.md` (new)
- This task folder

## Done when

1. `DESIGN.md` exists and is posted on #2829.
2. Every number it cites is traced to its source: the A8 dose record, the preflight record, the Q1 summaries, the attribution correction 6261186285, or the archive listing.
3. It says plainly that a placement census is not admission, and makes no KEEP, paired-DEV or fresh claim.
4. `report --check` and `pr-check --local` pass.

## Stop and ask if

- The retained archive does not hold the 64-batch checkpoints or the move logs the item names, so the census cannot be built from host data.

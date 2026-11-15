# ORDER: one-model-end-to-end, claude-387 (V2(b) follow-up: does the read's payload reach the answer?)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `origin/main` at `9747a3177`, which holds claude-386 (#3989, squash `42c67c4bd`).

Casey, 15 Nov 2026, in this session, to "Start the payload instrument check on Flock-g's saved 16,384-step model?": "Yes, run the check".

## Goal

claude-386 closed at the sheet's outcome 4: at 16,384 steps Flock-g (`--state-flock-read 4 --vector-read-stream-grad 1`, B2) picks the asked fact first on 494 to 504 of 512 held-out queries per far bucket, yet `R_ask` stays at 0.000 to 0.023. Its REPORT names the payload as the likely cause but did not measure it (L-007). This unit measures it on the two saved states, before any change to the read:

- at each held-out query row, `|x|` where the read adds, `|p|` the payload it adds (`w ⊙` the mean of its kept picks' next-token points), and their cosine;
- how many of Flock's four kept picks point at the asked fact;
- the asked token's decoded margin (its score minus the best other token's, the nearest-table pick `recall_picks` makes) with the read and with it cut;
- `R_ask` with the payload scaled by 1, 4, 16 and 64, eval only.

## Reading map (fixed before any number is read)

- **A. Too small:** `|p| / |x|` under 0.05 at the rows that chose right, and `R_ask` at 0.90 or more at some scale of at most 64. The fix is the payload's size or place (after `final_norm`), its own unit with a sheet.
- **B. Diluted:** as A at scale 1, but most kept picks other than the first point elsewhere, and scaling lifts `R_ask` less than it lifts the margin. The fix is the read's average (first pick only, or picks weighed), its own unit.
- **C. Wrong direction:** `R_ask` under 0.90 at every scale while `|p|` dominates `|x|` at 64. Then the payload does not decode to the asked token: a wiring or decode fault, repaired before any further read run, and claude-386's negative is re-read as an instrument result.
- **D. Anything else:** reported as read, with no fix named.

## Cancel probe

A payload measurement for claude-386's states on `main`. At base: none (claude-386's REPORT, "Still open").

## Steps

1. Code, float crate only: an eval-only probe in `vector_reads.rs` (row norms and kept picks recorded while a probe is open, an eval payload scale that is 1 and skipped by default) and a `recall-payload-probe` binary that loads a saved recall checkpoint and the run's settings, rebuilds the held-out windows from the seed, and writes `probe.json`. Training and the default forward are unchanged; a test pins that.
2. wick: build at the head, the new tests and `vector_read` cuda tests, then the probe on claude-386's four saved Flock-g states: the extension's step 16,384 (`try-5`, `try-6`) and the 4,096-step arms' last step (`try-1`, `try-3`), where the choice was about half learned. R0 has no read, so it has no payload to read.
3. REPORT with negatives first, the reading map's letter, one notebook row.

## Native-space five items (SKILL)

None is met: a float reference diagnostic (D85) on saved float states, an inventory-class unit. No training run.

## Pre-registration

None: an instrument check with no bar, no training and no milestone claim (§4.5 covers GPU training and evaluation that a claim rests on). The reading map above is fixed before the probe runs.

## Files in scope

`crates/uor-r4-float-train/src/vector_reads.rs`, `crates/uor-r4-float-train/src/in_space_long.rs`, a new `crates/uor-r4-float-train/src/bin/recall-payload-probe.rs`, this folder, the notebook row in `docs/research/notebook/float-training.md`.

## Done when

1. The probe's tests pass on the laptop and on wick; the default forward and training are unchanged (a test).
2. `probe.json` for both seeds' 16,384-step states, fetched with its sha256.
3. REPORT names the reading map's letter with the numbers; the PR through review, queue and `close --verify`.

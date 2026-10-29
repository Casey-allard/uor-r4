# Prep sheet: the turn memory, read at the readout (route-plus-memory, claude-161, 28 Oct 2026)

Lab: claude. Item: [route-plus-memory](../../roadmap/items/route-plus-memory.md). Order: [claude-161](../../loop/tasks/route-plus-memory/claude-161/ORDER.md), with Casey's words verbatim. Code: this unit's head, built and tested on each pod from that exact commit.

TEST FITNESS: FIT, round 1. PRE-RUN: PASS. Both by one fresh non-author reviewer, at `29f4d58bf`. Its two notes are applied below:
- **The lease.** One pod runs (m) and then (m0), each allowed up to 2 hours by the throughput gate, plus the tests, build, probes, reach, inspect and decode: up to about 4.5 hours. The lease is 5 hours, not 3.
- **The keys agree only as measured.** The table's key comes from the probe's prompt cut (`<|bos|>` or the response mask). The read's comes from the turn anchor (`<|bos|>` or `<|eos|>`). They agree at every opening by claude-154's `anchor_check`, not by construction. So `run_long` now logs the read's own hit count on TRAIN's and DEV's opening rows, and I-3 checks it. A C result can then be told apart from a key that misses.

## The question

**Does an exact memory of TRAIN, keyed by the user's turn and read where the readout reads, carry the reply openings the trunk cannot store?** The [pipeline map](../../model/pipeline-map.md)'s gap 1 is storage:
- the trunk has 64,000 values;
- routing the turn in works, since dropping the bag costs `189 / 236` openings;
- openings never beat (a)'s `Ō = 302.0 / 228.0` by the sheet's margin;
- a lookup of the turn's first 8 tokens names `607` of DEV's 1,226 openings.

## The read

**The turn memory** ([`flock_read_memory.rs`](../../../crates/uor-r4-float-train/src/flock_read_memory.rs)):
- **Key:** at an opening row (one that predicts a reply's first token, by the response mask), the ids at the turn anchor's first 8 slots, which is the probe's first-8 key.
- **Table:** built from the pinned TRAIN. Per key, the commonest opening, ties to the smaller id. Exact integer ids, about 26,450 keys.
- **Move:** at an opening row whose key the table holds, `n + w ⊙ (c − n)`, with c the stored token's table point (detached) and w per coordinate, starting at 0. Every other row is unchanged.

## Steps: the arms

Each arm runs `train-in-space --objective cell --hamming-read 4 --retain-rates 1 --decay-mult 1 --batch 16 --steps 16384 --eval-every 2048 --lr 0.001 --lr-end 0 --seed S` on the pinned TRAIN, at width 256 and seeds 1 and 2. That is claude-152's (a) exactly, plus:

| Arm | Extra flags | Role |
|---|---|---|
| (m) memory | `--turn-memory 8` | the read |
| (m0) empty | `--turn-memory 8 --turn-memory-empty 1` | the same code with an empty table: the no-memory control (L-010) |

- **(a)'s values:** claude-152's (a), sealed, at the same recipe without the memory.
- **(g)'s values:** claude-155's bag, sealed. It holds `in-space-trunk-to-text`'s bar.

Neither is re-run. Both are read from [claude-155's REPORT](../../loop/tasks/in-space-trunk-to-text/claude-155/REPORT.md), whose table restates claude-152's (a).

## Bar (pre-registered)

**`Ō`** is the mean of `opening_hits` (of 1,226) over the checkpoints at steps 12,288, 14,336 and 16,384. Write `m(X) = 3·√(1,226·p(1−p))` with `p = best(X)/1,226`.

- **Openings pass:** both (m) seeds reach `Ō ≥` the largest of:
  1. against (a): `302.0 + m = 302.0 + 45.3 = 347.3`;
  2. against (g): `266.3 + 43.3 = 309.7`;
  3. against (m0): `best(Ō of m0) + m(m0)`, from this run's two (m0) seeds.
- **Flag F (fit lost):** at either (m) seed, `T < 9,328 − 500 = 8,828` at step 16,384 (claude-155's flag).

| | Fit kept | Flag F |
|---|---|---|
| **Openings pass** | **A.** The memory carries what the trunk could not store. | **B.** It carries openings at a cost in fit. |
| **Openings fail** | **C.** Storage is not what limits openings here, or the move does not reach the readout. | **D.** It hurts. |

A pass at one seed only counts as no pass, and is reported as such.

## Instrument checks

- **I-1.** The crate's CUDA tests pass on each pod's build, including the memory's two CUDA tests: zero weight is no read, bit for bit, and the move equals the stored point's pull, with its gradient. The test count is recorded.
- **I-2.** At step 0, before any update, each arm's DEV reading equals (a)'s step 0 at its seed: the memory's weight is 0. Seed 1: hits 2, opening hits 0, 730 distinct opening picks. Seed 2: hits 5, opening hits 3, 681 distinct picks. `config.json` holds width 256, 4 Hamming heads, `turn_memory` 8 (and `turn_memory_empty` for (m0)), and the pinned TRAIN.
- **I-3.**
  - **The table:** the run's log names its table. That is about 26,450 keys from 212,968 TRAIN openings for (m), and 0 keys for (m0).
  - **The hits:** the run logs how many TRAIN and DEV opening rows the read's own key finds in the table. DEV hits must be above 0 for (m), and 0 for (m0).
  - **The weight:** (m)'s weight is non-zero at the end, and (m0)'s stays 0, since it never gets a gradient.
- **I-4.** Every loss is finite.
- **I-5.** The removal reading: at the final checkpoint, `--inspect`'s `memory_drop` re-reads DEV with the memory taken out. For (m), that must change the reading, which shows the evaluation used the memory. For (m0), it must change nothing.

## Data split

- **TRAIN:** the pinned `mw-balp` (`PINNED_TRAIN_*`), checked by sha256 at load. The table is built from it alone.
- **DEV:** the pinned DEV, read at every checkpoint. No DEV row enters the table.

## Base artifact

None. Each run starts from its seed's initialization, as (a) did.

## Minimum test power

At `p ≈ 0.25`, one seed's openings have a binomial spread of about 15 rows, and `m` is three of those. claude-155 measured (a)'s own seeds at `302.0` and `228.0`. A gain the size of the probe's `607` is far beyond that margin. A gain inside it does not pass.

## Cost

- **Pods:** two RTX 5090 pods on Vast, at about $1.0/h each. Each runs one seed: a 256-step probe and then (m) and (m0) in turn, about 1.5 hours, and up to about 4.5 at the gate's limit. The total is about $3 to $9.
- **Throughput:** the probe must project each arm under 2 hours (`≥ 6,991` context tokens/s), or the job stops.
- **Lease:** 5 hours, inside the caps (FIT note).
- **Results:** sealed to `results/claude/route-plus-memory-2026-10-28/` and read back before `down`.

## Stop rule

- **I-1 or I-2 fails:** stop. The read is not off at weight 0, or the build is not the head's.
- **The probe projects past 2 hours:** stop and profile.
- **The pod fails or the lease would pass the caps:** stop, seal what exists, and report.

## What each result unlocks

- **A:** the memory line grows. Next come the slot-address key, M3's writes and an integer form of the move (the map's R5), and the readout serves the memory.
- **B:** the move's weight or its rows are narrowed before the line grows.
- **C:** the item's second arm, the bag's learned code as the key, runs next. The removal reading says whether the move was used.
- **D:** the configuration closes (CHARTER §4.1), and the instrument is checked first.

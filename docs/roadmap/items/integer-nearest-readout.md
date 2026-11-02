---
id: integer-nearest-readout
title: Serve the trained model's readout in the integer artifact
milestone: research
area: serving
status: ready
open_to: lab
needs_gpu: false
depends_on: [integer-retention-rates, one-decoder]
notebook: docs/research/notebook/m4-serving.md
scope: [crates/uor-r4-integer/src/fbs_snapshot.rs, crates/uor-r4-integer/src/stack/format.rs, crates/uor-r4-integer/src/stack/session.rs, crates/uor-r4-integer/src/stack/kernels.rs, crates/uor-r4-integer/src/stack/tests.rs, crates/uor-r4-integer/src/stack/synthetic.rs, crates/uor-r4-integer/src/bin/fbs-export.rs, crates/uor-r4-integer/src/bin/fbs-toks.rs, crates/uor-r4-float-train/src/fbs_export.rs, crates/uor-r4-float-train/src/bin/fbs-snapshot.rs, crates/uor-r4-integer/src/stack/hamming.rs, crates/uor-r4-float-train/src/flock_read_hamming.rs, crates/uor-r4-float-train/src/geometric_stack_native.rs, crates/uor-r4-float-train/src/cuda_stack.cu, crates/uor-r4-float-train/src/in_space_long.rs, crates/uor-r4-float-train/src/flock_read_memory.rs, crates/uor-r4-float-train/src/geometric_stack_native_cuda.rs, crates/uor-r4-float-train/src/bin/train-in-space.rs, docs/roadmap/items/integer-nearest-readout.md, docs/harness/prep/integer-nearest-readout*, docs/labs/integer-nearest-readout/**, docs/loop/tasks/integer-nearest-readout/**, docs/research/notebook/m4-serving.md]
---

## Why it matters

The project's first hurdle is one working model end to end, served from its integer artifact. The served model must decode as the trained one does. Claude-231 built that readout on native and browser paths, but claude-232's end-to-end parity check missed in every cell. Claude-243's, with both sides on the same rules, meets the old bar on DEV and misses on p-w. Under claude-245's bar, Q1 to Q4 hold (claude-246). Claude-247 fixed Q5's rule, and the item stays ready until Q5's exact trace is read.

- **The trained model decodes by nearest.** (r-drop) trained with `objective: "cell"`, and `in_space_long` decodes such a model as the whole table's nearest token: `decodes_nearest`, "Whether replies decode as the whole table's nearest token, not the writer's pick". [`one-decoder`](one-decoder.md) (claude-150) made whole-table nearest the float side's served decoder.
- **Before claude-231, the integer session decoded by the native writer.** It computed whole-table nearest only beside the writer, for timing. In [`point-key-integer`](point-key-integer.md)'s real run (claude-229, #3460), integer greedy's first token equalled claude-220's float greedy on 0 of 310 DEV prompts and 2 of 1,202 on p-w. The float's own first token equalled the reference on 129 and 179. The run's grade read nothing and ended at row 0.
- **Claude-231 built `head = "native_vector_nearest"` and its export** ([REPORT](../../loop/tasks/integer-nearest-readout/claude-231/REPORT.md), #3463): 395 integer-library tests with one ignored, seven negative gates and full serving/WASM audits. The nearest branch serves `stack_whole_table_nearest` on native and browser paths, after memory and the Hamming read; the point-key override stays unchanged. An older engine refuses the new head, and an artifact without it decodes byte for byte as before.
- **Claude-232 ran end-to-end greedy parity** ([REPORT](../../loop/tasks/integer-nearest-readout/claude-232/REPORT.md), #3464) against claude-220's stored float greedy replies on DEV and p-w at seeds 1–3. It missed every cell. First-token agreement is 291, 285 and 284 of 310 on DEV, and 911, 777 and 64 of 1,202 on p-w. Whole-reply agreement is 83, 163 and 127 on DEV, and 60, 113 and 36 on p-w.
- **The readout was necessary but not sufficient.** The gap is largest on exact-key misses, and p-w seed 3 over-concentrates on one first token. The trunk, Hamming read and readout remain suspects, not located causes. The parity bar must pass before the item closes.
- **Claude-243 re-ran the parity with both sides on the same rules** ([REPORT](../../loop/tasks/integer-nearest-readout/claude-243/REPORT.md), #3488): the raw export (R) against the float reference re-decoded on the GPU with the read's tie rule on (F′). It is a **MISS** on p-w at every seed and meets the bar on DEV.
  - First-token disagreement is 4, 4 and 5 of 310 on DEV (bar 9), and 64, 47 and 78 of 1,202 on p-w (bar 36). Whole-reply agreement is 188, 247 and 201 on DEV, and 488, 751 and 547 on p-w.
  - The icosian snap was the larger difference behind claude-232's MISS: the snapped export against F′ agrees on 291/285/284 and 909/777/64 first tokens. F′ against F, the float code's change since `eb762e602`, moves at most 12 first tokens. By the code that change is the tie rule, but `train-in-space` has no switch to test it, so the reading is not credited to it.
  - The remaining p-w disagreements are diffuse; no first token is over-concentrated.
- **Claude-244 measured the bar's margin** ([REPORT](../../loop/tasks/integer-nearest-readout/claude-244/REPORT.md)), under teacher forcing with the new `fbs-toks --forced`. R's disagreements with F′ sit where R's own top two are nearly tied: 88% of the pooled p-w disagreements are in R's lowest-margin decile, and the median margin at a disagreement is 82 to 103 times below the median over all positions. The gap is per position, not special to the opening.
- **Claude-245 re-derived the bar** ([decision card](../../loop/tasks/integer-nearest-readout/claude-245/DECISION.md)). The same artifact flips 1.57% of DEV positions and 2.98% of p-w's, and p-w has 2.5 to 3.1 times smaller low margins. So a fixed share of disagreements grades the panel's tie density as much as the port.
  - Done-when 3 now asks two things: that R pick the reference token as often as the trained model, with its differences at its own near ties, and that every clear-margin disagreement be accounted for by an exact trace.
  - It no longer asks for 97% first-token identity.
- **Claude-246 read Q1 to Q3 of that bar and carried Q4** ([REPORT](../../loop/tasks/integer-nearest-readout/claude-246/REPORT.md), sheet [`integer-nearest-readout-grade-2026-11-01`](../../harness/prep/integer-nearest-readout-grade-2026-11-01.md)): **HELD**, with all 29 tests holding, on sealed files at $0. Row 0 held in all six cells, and p-w's cut windows join (I-Align: 0 violations, 0 unchecked, over 405 rows).
  - R's net loss b − a (positions only F′ gets right, less those only R gets right) under teacher forcing is +1, +6 and +2 on DEV (Q2's bound 20) and −17, +2 and −7 on p-w (bound 69). Pooled, it is +9 of 12,441 on DEV and −22 of 41,793 on p-w; p-w's cut windows give +12 of 4,254.
  - At the free-decode opening (Q3), it is +1, −6 and −2 on p-w, −7 pooled, −1 on the cut windows and +2 of 852 on DEV pooled.
  - Q4, carried from claude-244: κ is 0.84 to 0.97 in every cell, against 4/9.
- **Claude-247 fixed Q5's rule** ([decision card](../../loop/tasks/integer-nearest-readout/claude-247/DECISION.md), logged on #3462; Casey can reverse it). It read no trace value.
  - **The set:** 184 positions in 169 windows, listed from sealed files only. That is 162 counted (11 on DEV, 151 on p-w) and 22 on p-w's cut windows.
  - **The rule:** a three-way trace of F (F′ rerun), T (F's path on parameters rounded as exported) and R. Each step whose picks differ, structure (F, T) and arithmetic (T, R), is replayed with its discrete decisions forced in the forward's order. A position is accounted for only when every decision forced, and any pick difference left, is a near tie by the decile rule. A kept set or memory lookup that differs fails the step outright.

## What is missing

- **Next: the trace instrument, then the sheet that reads Q5,** under claude-247's card. Q1 to Q4 held in claude-246, so done-when 3 waits on Q5 alone.
  - **The instrument** records every decision of the card's five kinds on F, T and R: Hamming codes, kept sets, memory lookups, rate buckets and the nearest. It forces codes and rate buckets, and writes each side's query at the traced positions, so each side can score the other's pick.
    - The card's scope change is in place: `stack/hamming.rs`, `flock_read_hamming.rs`, `geometric_stack_native.rs`, `cuda_stack.cu` and `in_space_long.rs` are in this item's `scope:` (claude-248).
    - **F's and T's files are in the scope too** ([claude-250](../../loop/tasks/infra-loop-harness/claude-250/ORDER.md)): `flock_read_memory.rs` for the memory lookups, `geometric_stack_native_cuda.rs` for the rate buckets, and `bin/train-in-space.rs` for the options. `wasm_serve.rs` and `wasm_bindgen_exports.rs` left the scope: the rest of Q5 touches neither, and an open Studio item lists both. A later unit that needs either adds it back.
    - **R's side is built** ([claude-249](../../loop/tasks/integer-nearest-readout/claude-249/REPORT.md)): `fbs-toks --forced --decision-trace` writes `uor-r4.decision-trace/1`, and `--force` replays rows under a `uor-r4.decision-force/1` set. On R s1, DEV s1's 308 rows outside the set equal claude-244's sealed rows exactly, traced or not, at about 47 ms per window.
    - **F's side is built** ([claude-251](../../loop/tasks/integer-nearest-readout/claude-251/REPORT.md)): `train-in-space --score-rows … --decision-trace SPLIT=POSITIONS_JSON` writes side F of the same schema, with each code's coordinates and both nearest scores in R's units. On DEV s1's 308 windows of R's smoke, the traced rows equal F′'s sealed rows, all 10,840 picks equal its nearest, and R's steps are an exact prefix of F's.
    - **T's side is built** ([claude-252](../../loop/tasks/integer-nearest-readout/claude-252/REPORT.md)): `fbs-snapshot --round-as-exported` writes T's evaluation copy, with every parameter the artifact holds rounded as the raw export rounds it and a marker naming each file, and `score_rows` on the copy writes side T once it has checked the rounding again. I-Twin holds at seeds 1–3: the copy's snapshot exports to R's sealed artifact byte for byte. With the copy's read-back (re-rounding it changes nothing), that makes T's values R's; the export alone would not, since the unrounded step exports to the same bytes. On DEV s1's 308 windows, T's rows are equal traced or not, its windows join F's and R's, and its pick equals R's on 4,380 of 4,382 rows of R's range (F's: 4,298). That is a reading, with nothing forced.
    - **Forcing on the float side is built** ([claude-253](../../loop/tasks/integer-nearest-readout/claude-253/REPORT.md)): `train-in-space --score-rows … --decision-force SPLIT=FORCE_JSON` runs the traced windows under a `uor-r4.decision-force/1` set.
      - **What it forces:** a forced bucket gets its centre score before the rate pick; a forced code changes only the coordinates whose bit differs, each to ±1.
      - **On F's s1:** forcing all 14,031,744 buckets and 582,192 codes to themselves leaves the trace and all six rows files equal, byte for byte. One forced bucket and code change only their window, from their step on.
    - **The near-tie grader is built** ([claude-254](../../loop/tasks/integer-nearest-readout/claude-254/REPORT.md)): `fbs-toks --decision-deciles`, `--decision-check` and `--decision-grade` compute G₁₀ per family, check a trace against its own table, and grade one pass at a position by the card's rules. Its [decision card](../../loop/tasks/integer-nearest-readout/claude-254/DECISION.md) adds the readout's earlier keys to p's causal prefix and drops the key at step 0.
      - **On DEV s1:** F's base trace against itself reaches all 10,840 nearests with equal picks. F's and R's traces check against their own tables. claude-253's one-forced window is refused at its forced bucket, as an instrument fault, since no score selects the forced entry.
    - **The serving audit passes again** ([claude-255](../../loop/tasks/integer-nearest-readout/claude-255/REPORT.md)). `m5_full_path_cost_controls` had refused an `fmov` in both instances of `TypeHeads::mean_by`: its `choose` closure's captures were copied through a SIMD register. `mean_by` now takes the forced heads and calls `choose` or `choose_forced` itself.
      - R's DEV s1 rows and traces, plain and forced, are byte for byte as at the base.
    - **The replay driver's arithmetic step is built** ([claude-256](../../loop/tasks/integer-nearest-readout/claude-256/REPORT.md)). `replay_step` runs a step's whole forced replay, forcing each differing code or bucket in forward order. `fbs-toks ARTIFACT --decision-replay` runs it for (T, R) with R from the artifact as Q. It also carries claude-254's round-2 points: f32's rounding bound on a float pick, and R's smaller-id tie.
      - **On DEV s1:** T's trace checks against its table. Of 4,382 offsets inside R's window, T and R pick equal at 4,380, and both differences replay to near ties (14 passes). T's windows read on past the reference, and those picks are counted, not compared.
    - **Next, the structure step (F, T):** `train-in-space`'s single-window float mode, so that Q = T runs one window per pass on the GPU, driving the same `replay_step`. Each float pass is now a whole `score_rows` run (204 s on F's s1).
  - **The sheet** registers the card's rule verbatim, with a fresh FIT and a pre-run review. Its Row 0 is the card's: I-F, I-R, I-Twin, I-Rerun, I-Force, I-Align, I-Complete, I-Set and I-Inputs. GPU for F and T, under an hour; the laptop for R.
- **Leads for the trace,** layer by layer: the trunk, Hamming read, memory and readout. Recheck end-to-end parity before rerunning p-x.
  - **The clear-margin disagreements** (g above G₁₀) first, as Q5 requires.
  - **Read F′'s own top two** at the disagreeing positions. Its sealed rows carry no scores, so claude-244's residual bound is one-sided.
  - **The cut windows' openings** disagree 2.9 to 4.4 times as often as the counted ones per seed, 3.4 pooled (claude-244, a reading).
  - **Split the disagreements by exact-key status,** as claude-232 did on DEV s1. p-w is the panel where the memory names fewer replies.
  - claude-227's trunk parity on the raw export against the trained forward, on p-w windows, would give the residual layer by layer.
  - The two differences between claude-232's sides, the float reference's tie order (claude-238, claude-240) and the integer side's icosian snap (claude-233), are removed in claude-243's arms. Its sealed run is `results/claude/integer-nearest-readout-tie-parity-2026-11-01/`; claude-232's is `results/claude/integer-nearest-readout-parity-2026-10-31/`.
  - The float replies are sealed data, so this item does not wait on [`one-model-end-to-end`](one-model-end-to-end.md), which made them.
- **Making raw the default, or the snap faithful,** is a follow-up: S against F′ now measures what the snap costs.
- **The completed readout, for reference:** `head = "native_vector_nearest"`, with the whole table's `−L1` scan at Q12 and ties to the smaller id. The browser uses `IntegerStackSession::serve_step`, so FBS models no longer fall back to token 0.
  - On the snapped artifact (claude-231) it decodes at 1,365 and prefills at 1,405 tok/s; the nearest scan costs about 2.85× a token. Prefill-scan removal is a separate optimization, not parity evidence.
  - On the raw artifact (claude-243, R s1, done-when 4): 1,204 tok/s decode and 1,264 prefill (medians), 190.7 MB peak resident, under other load. The 4,096-row nearest is 65% of a decode step, so a cheaper exact nearest is the speed lead.

## Done when

1. **The artifact carries its readout,** with a negative gate. An artifact without it decodes byte for byte as at the base, and an engine without it refuses a nearest artifact.
2. **The nearest step** serves `stack_whole_table_nearest`'s rule under the field, with a test against a reference scan and a negative gate.
   - The browser path (`wasm_serve`'s step) gives the same token as the session's nearest on one artifact, with a test.
   - The serving audit (`audit_zero_matmul_serving.py --stack`) and the WASM build pass.
3. **End-to-end token parity passes its bar, under a prep sheet with a fresh FIT and a pre-run review.**
   - Integer greedy of (r-drop) step 16,384, memory on, exported raw (`fbs-export --raw-gates --raw-transition`, R), against the float greedy decoded again on the GPU with the read's tie rule on (F′: claude-220's arm-G command at the unit's head), on DEV and p-w at seeds 1–3.
     - claude-232's snapped export (S) and claude-220's stored replies (F) are kept as readings.
     - Both changes are claude-243's [decision card](../../loop/tasks/integer-nearest-readout/claude-243/DECISION.md), logged on #3462; Casey can reverse it. "Done" applies to the raw export, and the item then names its recipe as the served artifact's.
   - **The bar is claude-245's [decision card](../../loop/tasks/integer-nearest-readout/claude-245/DECISION.md),** logged on #3462; Casey can reverse it. It replaces claude-243's first-token ⌊0.03 n⌋, which was registered against claude-227's 0.33% residual of a snapped export. Q1, Q2 and Q4 are read under teacher forcing on claude-244's counted positions, and Q3 on claude-243's free decode. A LOSS of any class fails this bar for that artifact.
     - R picks the reference token as often as F′ does: no loss beyond 3 row-clustered SE (Q1), and at most half a point (Q2);
     - the same holds at the opening, on free decode (Q3);
     - R's differences sit at its own near ties (Q4: κ ≥ 4/9).
     - **Q5:** every clear-margin disagreement is also accounted for by an exact trace, under claude-247's [decision card](../../loop/tasks/integer-nearest-readout/claude-247/DECISION.md), logged on #3462; Casey can reverse it.
     - First-token and whole-reply agreement stay reported as readings; claude-243's MISS on the old bar is recorded as it is.
   - **The item is done only when the bar passes.** On a miss, the sheet's row is recorded, the item stays `ready`, and the next unit locates the gap.
   - Integer decodes run on the laptop: product serving (CHARTER §3.6), under the owner's 31 Oct allowance for this chain. The float side runs on the GPU.
4. **Tokens per second,** with the nearest scan counted (4,096 rows a token), and the resident memory.
5. **One notebook row, and a REPORT with the negatives first.**

## How to start

- **Read:**
  - `in_space_long.rs`: `PointObjective`, `decodes_nearest`, and the decode in `score_rows`;
  - `stack/session.rs`: `stack_step_fbs`, `native_token` and `stack_whole_table_nearest`;
  - `fbs-snapshot.rs` and `run_rate_reading` (claude-226), for reading the run config;
  - claude-229's [lab record](../../labs/point-key-integer/README.md).
- **Before claiming:** check that no open order holds `stack/session.rs` (`point-key-integer` and `turn-memory-integer` also edit it).

## Rules that apply

- **Serving stays integer** (CHARTER §3.6, as amended by D80 and D81): no float or transcendental instruction in the step.
- **Deterministic exact checks only** (§3.10, §4.4).
- **A negative closes a configuration, never a mechanism** (§4.1).

## Sources

- claude-229's [REPORT](../../loop/tasks/point-key-integer/claude-229/REPORT.md) and the sealed run, `results/claude/point-key-integer-run-2026-10-31/`.
- `crates/uor-r4-float-train/src/in_space_long.rs`, `decodes_nearest`.
- `crates/uor-r4-integer/src/stack/session.rs`, `stack_whole_table_nearest`.

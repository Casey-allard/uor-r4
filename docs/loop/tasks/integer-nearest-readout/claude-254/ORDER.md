# ORDER: integer-nearest-readout, claude-254

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md). Base: `f1405cc10`.

## Goal

The item's done-when reads:

> 1. **The artifact carries its readout,** with a negative gate. An artifact without it decodes byte for byte as at the base, and an engine without it refuses a nearest artifact.
> 2. **The nearest step** serves `stack_whole_table_nearest`'s rule under the field, with a test against a reference scan and a negative gate.
>    - The browser path (`wasm_serve`'s step) gives the same token as the session's nearest on one artifact, with a test.
>    - The serving audit (`audit_zero_matmul_serving.py --stack`) and the WASM build pass.
> 3. **End-to-end token parity passes its bar, under a prep sheet with a fresh FIT and a pre-run review.**
>    - Integer greedy of (r-drop) step 16,384, memory on, exported raw (`fbs-export --raw-gates --raw-transition`, R), against the float greedy decoded again on the GPU with the read's tie rule on (F′: claude-220's arm-G command at the unit's head), on DEV and p-w at seeds 1–3.
>      - claude-232's snapped export (S) and claude-220's stored replies (F) are kept as readings.
>      - Both changes are claude-243's [decision card](../../loop/tasks/integer-nearest-readout/claude-243/DECISION.md), logged on #3462; Casey can reverse it. "Done" applies to the raw export, and the item then names its recipe as the served artifact's.
>    - **The bar is claude-245's [decision card](../../loop/tasks/integer-nearest-readout/claude-245/DECISION.md),** logged on #3462; Casey can reverse it. It replaces claude-243's first-token ⌊0.03 n⌋, which was registered against claude-227's 0.33% residual of a snapped export. Q1, Q2 and Q4 are read under teacher forcing on claude-244's counted positions, and Q3 on claude-243's free decode. A LOSS of any class fails this bar for that artifact.
>      - R picks the reference token as often as F′ does: no loss beyond 3 row-clustered SE (Q1), and at most half a point (Q2);
>      - the same holds at the opening, on free decode (Q3);
>      - R's differences sit at its own near ties (Q4: κ ≥ 4/9).
>      - **Q5:** every clear-margin disagreement is also accounted for by an exact trace, under claude-247's [decision card](../../loop/tasks/integer-nearest-readout/claude-247/DECISION.md), logged on #3462; Casey can reverse it.
>      - First-token and whole-reply agreement stay reported as readings; claude-243's MISS on the old bar is recorded as it is.
>    - **The item is done only when the bar passes.** On a miss, the sheet's row is recorded, the item stays `ready`, and the next unit locates the gap.
>    - Integer decodes run on the laptop: product serving (CHARTER §3.6), under the owner's 31 Oct allowance for this chain. The float side runs on the GPU.
> 4. **Tokens per second,** with the nearest scan counted (4,096 rows a token), and the resident memory.
> 5. **One notebook row, and a REPORT with the negatives first.**

**This unit's part:** Q5's near-tie grader, the fourth of claude-247's [card](../claude-247/DECISION.md)'s steps after F (claude-251), T (claude-252) and the float-side forcing (claude-253). Casey's direction, "Finish Q5 first", holds. The grader judges one pass of the forced replay, on the laptop, from two sides' decision traces (`uor-r4.decision-trace/1`):
- the families and G₁₀(φ, P), from the reference side's unforced trace of a cell;
- the first decision in p's causal prefix, in the forward's order, where the other side differs;
- the near-tie rules for a code, a rate bucket and the pick, with the pick's scores computed in R's units from each side's query at p and its own table;
- a kept set or a memory lookup that differs, never a near tie;
- and whether a finished replay accounts for its step.

It reads no trace value of Q5's set. The replay driver, the sheet with its FIT and pre-run review, and the GPU run that reads Q5 come after it.

## Cancel probe (run first)

Run on 2 Nov 2026 at `098afdf63` (base `f1405cc10`).

1. **Nobody holds or has built it.**
   - `gh pr list --state open`: only this unit's draft (#3535) touches `fbs_snapshot.rs`, `fbs-toks.rs` or the item's folders.
   - `grep -rni` over `crates/uor-r4-integer/src` for `g10`, `near tie`, `decision-grade` and `decision_decile` finds only older near-tie wording (fbs-compare, `stack/format.rs`, `stack/tests.rs`), no grader.
   - **Class: instrument.** The grader does not exist and has to be built.
2. **The sealed traces carry every field the rules read.**
   - **F's base trace of DEV s1, window 41** (claude-253's `run/base-w0.json`, `22b64304…`, in its store seal): `score_f32` and `entry` for 64 lanes; each head's `query_f32` and `key_f32`, 64 coordinates; and at the nearest, `pick`, `score_f64`, `runner_up`, `runner_score_f64` and a 256-coordinate `query_f32`.
   - **R's trace of DEV s1** (claude-249's `smoke/trace-dev-s1.json`, `11e6b7d9…`): 308 windows, 6 layers, 64 lanes, 4 heads, 16 kept; `score_q16` and `entry`; a stream read at depth 2 and a readout read at depth 6 at every step; the memory at reply steps; and at the nearest, `score_q12`, `runner_score_q12` and a 256-coordinate `query_q12`.
   - **Two facts the grader must handle:**
     - F's window 41 runs 99 steps, to its last scored offset, where R's runs 50.
     - At step 0, the readout key is F's coordinates' sign packing and R's 0. R refuses to force a key at step 0: it names no candidate (`stack/hamming.rs`, `forcing`).
   - **T's whole trace was not sealed,** only its summaries (claude-252's store holds `t-summary.json`, `align-t.json` and `cross.json`). So no T trace is read here; T's table path is tested on synthetic traces.
3. **The tables are reachable on the laptop.**
   - **R's and T's:** R's s1 artifact `a7r-s1.uorlut` (`f86a7856…`, claude-243's laptop seal), whose `writer.table` `served_copy_codes_uorlut01` already returns. T's table is that table divided by 4096, as the card sets.
   - **F's:** the `embed` of claude-227's full snapshot `snapshot-s1.json` (`dcbfb69b…`, the snapshot R was exported from). `score_rows` scores against `embedding.weight` (`in_space_long.rs`, `table`), and `fbs-snapshot` writes it as `embed`.
4. **A gap in the card.** claude-247 places the readout read's codes in p's causal prefix "at p alone". But the readout's kept set at p is chosen from its key codes at every row 1 ≤ s < p, and each of those is recorded at step s (`DecisionHead::key`: the code of `x_{t−1}`, "which every later row reads").
   - The card's own reason that a kept set is never a near tie is that "with equal codes and equal token ids, both sides must decide it the same way". That holds only if those earlier keys are compared.
   - Conversely, no decision at p reads any key at step p, and none at all reads the key at step 0.
   - **Class: instrument (the rule's definition).** It is a decision of this unit: [DECISION.md](DECISION.md), with its own fresh decision review, logged on #3462 before the grader is used on anything.

## Files in scope

- `crates/uor-r4-integer/src/fbs_snapshot.rs`
- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/stack/kernels.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-integer/src/stack/synthetic.rs`
- `crates/uor-r4-integer/src/bin/fbs-export.rs`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `crates/uor-r4-float-train/src/fbs_export.rs`
- `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`
- `crates/uor-r4-integer/src/stack/hamming.rs`
- `crates/uor-r4-float-train/src/flock_read_hamming.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `crates/uor-r4-float-train/src/cuda_stack.cu`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/flock_read_memory.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native_cuda.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `docs/roadmap/items/integer-nearest-readout.md`
- `docs/harness/prep/integer-nearest-readout*`
- `docs/labs/integer-nearest-readout/**`
- `docs/loop/tasks/integer-nearest-readout/**`
- `docs/research/notebook/m4-serving.md`

## Done when

The item's done-whens 1–5 above are the item's. This unit's done-whens are its own:

1. **The prefix decision.** [DECISION.md](DECISION.md) amends claude-247's causal prefix as the probe found, with a fresh decision review (CHARTER §5.1), logged on #3462 as `[claude] owner log:`.
2. **The library, in `fbs_snapshot.rs`:**
   - `decision_deciles`: G₁₀(φ, P) for every family of one side's trace;
   - `grade_pass`: the first difference in p's causal prefix in the forward's order, judged by the card's near-tie rules, or the pick at p's nearest. It refuses, as an instrument fault:
     - two traces that do not line up (I-Align);
     - a forced decision the other side did not take, or one not the reference's value (I-Force);
     - a code whose coordinates do not sign to it, an entry its score does not select;
     - a table that does not reproduce a side's recorded pick and runner-up scores;
   - `replay_accounts`: whether a finished replay accounts for its step (the card's step 3).
3. **The CLI, in `fbs-toks`:** `--decision-deciles TRACE OUT` and `--decision-grade`, which grades one pass at every chosen position and writes `uor-r4.decision-grade/1` new.
4. **Tests on synthetic traces,** one per rule and per refusal, each non-trivial rule with a negative gate (the rule set aside, the test fails, restored).
5. **A reading on real traces, outside Q5's set and not Q5:**
   - F's DEV s1 base trace against itself: no difference before any nearest, every pick equal;
   - claude-253's one-forced window against the base's: no difference before step 49, and the first difference is the bucket forced at step 49, entries 1 and 9. *Amended during the work:* this bullet first ended "not a near tie". But claude-253 forced entry 9 where both windows' score selects 1, a test of the forcing path and not a replay value. So done-when 2's refusals apply, and the grader stops there as an instrument fault instead of judging a near tie: without Φ, an entry its score does not select; with claude-253's Φ, I-Force (not the reference's value);
   - the deciles of F's base trace;
   - every table check on F's base trace (with F's table) and R's DEV s1 trace (with R's).
6. **The host suite of `uor-r4-integer`, `loop gates --full`, `loop report --check` and `loop pr-check --local` pass.** A REPORT with the negatives first. The item stays `ready`: done-when 3 waits on Q5.

## Stop and ask if

Stop and decide, never stop and wait:
- **The card's rule cannot be implemented as written** beyond the prefix gap: a decision kind it does not list, or a family it does not define. Then stop the grader and write a card first; the card voids itself on an unlisted kind.
- **A sealed trace lacks a field a rule reads.** That is an instrument repair of the side's trace writer, not the grader's.
- **The whole base trace does not fit the laptop's memory.** Then read it by window, and record it.
- **Any reading touches Q5's set.** It must not: only the smoke windows are read.

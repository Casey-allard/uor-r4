# ORDER: fbs-cost-profile, claude-158

Lab: claude. Item: [fbs-cost-profile](../../../../roadmap/items/fbs-cost-profile.md). Base: `be88872e9`.

## Goal

The owner, 28 Oct 2026, in claude's chat: "I don't care what we serve at long as it is cheap". claude-157 profiled training. This unit is the item's other half, its done-when 2:

> 2. **One served token, profiled by stage** on the M1 in `fbs-toks`: each trunk layer, the readout (writer and whole-table nearest, each timed), and the loop overhead, summing within 10%.

It also covers done-when 3 for serving's top three, and done-when 4's notebook row and REPORT.

## The instrument

**`IntegerStackSession::measure_fbs_step_stages`** in `crates/uor-r4-integer/src/stack/session.rs` is a stage-timed twin of `stack_step_fbs`, as `measure_step_phases` is of the dense step.
- It runs the same kernels in the same order and leaves the session in the same state.
- `step` and `stack_step_fbs` are untouched, for the zero-multiply audit of `step`'s call graph.
- It times:
  - the embedding;
  - each layer's recurrence, and then its feed-forward;
  - the final norm and query;
  - the native writer;
  - the bookkeeping;
  - the whole step.
- Beside the step it times a **whole-table nearest search** over the query, the readout a cell-objective model decodes. That search reads only.

**`fbs-toks ARTIFACT --stages`** runs the existing protocol through the twin: 1 warm-up and 20 timed runs, each a 128-token prefill and 64 decode steps. It prints, for the decode steps:
- each stage's median;
- the stages' sum against the median step;
- the median whole-table nearest;
- the loop's overhead: the median of `fbs-toks`'s own per-token time less the step's.

Without `--stages`, `fbs-toks` prints what it printed before.

## Instrument checks

- **I-1.** A host test: over a synthetic FBS trunk, `measure_fbs_step_stages` and `step_native` give the same tokens, kept cells and queries at every step. `save_state` is equal at the end.
- **I-2.** A host test: the whole-table nearest equals a brute-force search, ties included.
- **I-3.** At run time, `--stages` gives the same replies as the plain protocol on the same artifact, and the stage times sum to the step time within 10%.
- **I-4.** The plain protocol's decode rate is compared with claude-126's on the same artifact, `231` tok/s median at seed 1. A gap over 20% is reported (another day, another load), not a stop.

## The artifact

claude-126's seed-1 export: `caseyallard/uor-r4-store:results/claude/d11-fbs-export-gap-2026-10-26/d11gap/save/seed-1/artifact.uorlut`.
- **Hash:** sha256 `314c3909c66b58522a641c3f5b51c25dda4bda5ab2274bcfd0b5ca5038fdb305`, matching its `SHA256SUMS`.
- **Shape:** width 256, 6 layers, vocabulary 4,096, the shape of claude-152's (a) trunk.
- **Why it is enough:** the integer kernels' cost depends on the shape, not the weights, so the profile is the shape's.

## Run class

The run serves an exported integer artifact on the M1, the product's own device (CHARTER §3.5–3.6), as claude-126's `fbs-toks` did. It trains and scores nothing. So it is a host check, not a GPU run, and needs no sheet (§4.5).

## Cancel probe (run first)

Run at `be88872e9`:
- no open PR touches `session.rs` or `fbs-toks.rs`;
- no other lab's ORDER names `fbs-cost-profile`;
- `session.rs` is in the item's `scope:` on `origin/main`.

## Files in scope

- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `docs/loop/tasks/fbs-cost-profile/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/fbs-cost-profile.md`

## Done when

1. I-1 to I-4 pass, with their output in the REPORT.
2. The item's done-when 2 is met: one served token by stage on the M1, summed within 10%, with the writer and the whole-table nearest each timed.
3. Done-when 3 for serving's top three: one line each on the cheapest change.
4. One notebook row and the REPORT. The item is set `done` if done-when 1 to 4 are all met.
5. The PR is merged and verified on main.

## Stop if

- **I-1 or I-3 fails.** The twin is not the served step: an instrument fault, fixed before any reading.

No step waits on Casey.

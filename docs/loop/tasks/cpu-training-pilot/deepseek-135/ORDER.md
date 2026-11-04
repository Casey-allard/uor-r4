# ORDER: cpu-training-pilot, deepseek-135 (query admission training)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `27317ddb8`.

## Goal

Train the exported FBS final gain directly on actual generated trajectories so
reference target cells enter the K=8 writer admission set. This follows the
measured deepseek-133/134 fault: post-admission writer ranking and the fixed
K=8→K=16 budget cannot repair a target cell the query never admits.

This is D84 CPU development research. It does not change serving, integer
session behavior or claim useful conversation.

## Cancel probe

Deepseek-134 closes the existing K=16 budget. Deepseek-131 trained final gain
only toward terminal BOS on cached teacher-forced states, not content-cell
admission on generated trajectories. No prior run optimizes admitted content
targets before writer training. Claude PR #3664 remains limited to its GPU
scoring sheet.

## Fixed construction

Use the public 98/103 repeated-key split and training-derived memory. The
training objective scans actual generated trajectories after each accepted
change and scores:

1. observed target cells admitted at K=8;
2. target-cell rank sum;
3. fixed-horizon missing decisions; and
4. distinct admitted target cells and same-cell pairs.

The artifact starts at K=8. Candidate final-gain coordinates are fixed to
`0,8,16,...,248`; each trial changes one coordinate by `+/-32`. Choose the
lexicographic best safe candidate and accept at most 12 moves. Missing
fixed-horizon decisions may not increase.

## Arms and bar

Run frozen, true-target training, deterministic cross-cell target permutation
and an equal-move random-gain control.

Both seeds must pass:

- true K=8 positive-margin training decisions reach at least **640**, with at
  least **64** same-cell pairs;
- true held-out admitted targets exceed frozen by at least **128** and exceed
  the permuted-target control;
- held-out fixed-horizon matching depth and teacher-forced top-1 do not fall;
- artifact bytes change only in the final-gain table;
- every arm retains exact CPU/integer generated-sequence parity; and
- seed-1 repeat matches every non-timing field and artifact.

Report rank histograms, eligible rows, exact references, stops, moves,
workers, wall and RSS. Exact replies are not a bar.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/claims.json`
- `docs/harness/prep/cpu-training-pilot-query-admission-2026-11-04.md`
- `docs/labs/cpu-training-pilot-query-admission-2026-11-04/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-135/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

No integer-session, serving, CUDA, float-training or Claude-owned source
changes.

## Done when

1. A CPU-only final-gain admission trainer exposes deterministic actual-path
   objective readings, target permutation and equal-move control.
2. Tests cover cell objective order, candidate bounds, missing-horizon safety,
   target permutation, gain-only artifact changes and parity.
3. FIT and a fresh GO precede both seeds and a repeat.
4. REPORT and hashed evidence preserve either outcome; notebook and roadmap
   state whether generated-query admission training is viable.
5. Exact-head review, protected merge and fresh-main verification complete.

## Stop and ask if

- Any candidate loses exact parity or changes non-gain artifact bytes.
- The permutation is unchanged or the random control cannot match move count.
- A run needs serving, integer-session, CUDA or Claude-owned files.

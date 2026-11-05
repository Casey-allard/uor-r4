# ORDER: cpu-training-pilot, deepseek-159 (exact-body generated stop gate)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `330185de6`.

## Goal

Test whether the existing geometric generated-stop gate learns a termination
state when terminal training rows come only from trajectories whose generated
body exactly matches the normalized reference. Deepseek-157 rejected the same
gate when every reference-length decision was labeled terminal. This unit
changes the labels, not the signature, gate or evaluation path.

This is D84 CPU development research. It changes no serving, integer runtime,
CUDA, float-training or Claude-owned source.

## Cancel probe (run first)

Deepseek-157 is merged on the base and records zero exact replies for the
all-reference-length labels with eight prototypes on both artifact seeds.
Deepseek-155 records an oracle ceiling of 11 exact train and 13 exact dev
trajectories per seed. The task therefore stands, but exact-body fitting has
only 11 terminal training trajectories per seed.

## Fixed construction

Replay the sealed deepseek-153 longest-first trajectories through the exact
`CpuFbsForward` writer path. A trajectory enters the exact-body training set
only when every selected token through the normalized reference length equals
the reference. Within those trajectories, label only the reference-length
decision terminal and every earlier decision nonterminal.

Fit separately on train panel 2711 for each artifact seed with:

- 256 signed query coordinates;
- eight terminal and eight nonterminal Hamming prototypes;
- nearest-nonterminal minus nearest-terminal score;
- a margin chosen only on the selected train rows to maximize terminal stops
  subject to at most `floor(nonterminal_rows / 100)` false stops.

Evaluate the fitted gate unchanged on dev panel 2712. The first firing emits
BOS; otherwise the sealed longest-first token is fed forward. The decoder never
reads the reference.

Controls are:

- the exact-body fitted true gate;
- a class-preserving shuffled-label gate fitted on the same exact-body rows;
- the deepseek-157 all-reference-length gate fitted on all train rows;
- no gate.

## Bar

The candidate passes only if, for both artifact seeds:

- the gate fits a nonzero threshold under the training false-stop limit;
- true exact train replies are at least **6/11** and true exact dev replies are
  at least **7/13**, each half the deepseek-155 oracle ceiling;
- true exact replies beat each of shuffled, all-reference and no-gate by the
  paired two-sigma rule;
- premature stops are at most **1%** of panel replies: 4 train and 4 dev;
- all fired rows and unchanged selected-trace hashes are recorded;
- a fresh-process repeat produces byte-identical decision fields.

## Abort conditions

Stop on source report, token, mask, artifact or hash drift; a trace that cannot
be replayed; non-monotonic decisions; fewer than eight distinct exact-body
terminal signatures on train; class absence; failure to fit a nonzero gate
under the limit; nondeterminism; or a vacuous shuffled-label split. Record a
power-limited result rather than retuning after observing dev.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-copy-exact-stop-2026-11-05.md`
- `docs/harness/prep/cpu-training-pilot-copy-exact-stop-confirm-2026-11-05.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-159/**`
- `docs/labs/cpu-training-pilot-copy-generated-stop-2026-11-05/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/integration/current-state.md`
- `docs/harness/claims.json`

## Done when

1. Exact-body row selection, all-reference control and paired reply comparison
   have finite tests.
2. Both artifact seeds fit on exact-body train rows and run unchanged on dev.
3. Evidence records all four arms, premature stops, fired rows, input hashes
   and a deterministic repeat.
4. REPORT classifies the outcome as KEEP, REJECT or power-limited without
   broad claims about geometric termination.
5. Exact-head review, protected merge and fresh-main verification complete.

## Attempt 2: fresh-dev confirmation

Attempt 7 evaluated an already-opened dev panel before an independent FIT
verdict had been recorded. That attempt therefore remains preserved as
exploratory evidence and is not the registered decision.

The registered confirmation refits on the same unmodified train panel 2711
and artifact seeds, then evaluates the unchanged gate on a newly generated
`cell=dev,dev` panel seed 2722. The fresh panel is generated only after FIT and
GO. Its oracle ceiling must be measured from the sealed copy trajectory before
the gate report is read; the candidate must reach at least seven exact replies
and at least half that ceiling. All prior controls and paired rules remain.
Details and commands are fixed in
`docs/harness/prep/cpu-training-pilot-copy-exact-stop-confirm-2026-11-05.md`.

The fresh confirmation, not attempt 7, determines KEEP, REJECT or
POWER-LIMITED for this task.

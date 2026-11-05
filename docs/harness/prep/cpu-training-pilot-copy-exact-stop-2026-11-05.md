# Pre-registration: exact-body generated stop gate

Run ID: `cpu-training-pilot-copy-exact-stop-2026-11-05`

## Decision

Can the existing discrete Hamming stop gate learn a terminal state when its
training terminal labels come only from generated copy trajectories whose body
exactly matches the normalized reference?

This is D84 CPU development research on the Ryzen 9 9950X. It is not GPU
qualification and makes no serving or language claim.

## Prior evidence and change

Deepseek-157 fit the same gate after labeling every reference-length decision
terminal. Both artifact seeds fit nonzero gates but produced zero exact replies.
That rejection does not distinguish a weak geometric signature from terminal
labels polluted by incomplete bodies.

Deepseek-155 measured 11 exact train and 13 exact dev trajectories per artifact
seed. This sheet changes only the training-row filter: a trajectory contributes
rows only when all selected tokens through the reference length equal the
reference. Within that set, reference length is terminal and earlier decisions
are nonterminal. The signature, prototype gate and decode path remain fixed.

## Base and panel

Base implementation: `330185de6` plus `deepseek-159`.

Artifact seeds:

- seed 1: `dual-c131-seed-1-raw_both.uorlut`, SHA-256
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- seed 2: `dual-c131-seed-2-raw_both.uorlut`, SHA-256
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`.

Sealed source: deepseek-153 attempt 2 under
`local/deepseek/copy-key-longest-reply/2026-11-05-attempt-2/`.

- Train panel `2711`: fit only, tie and token files with their committed hashes.
- Dev panel `2712`: evaluation only, unchanged.
- The driver validates artifact, report, manifest, token and mask hashes and
  replay hashes before fitting.

Source report SHA-256 values are
`28a8caaffef6d815da86a6e84207729f3acd41892ced8a205cfe5248d7c2f62a`
for train and
`e2485b3b4012a4c5102b817e3758517502213d126cbb136a46552df18b936d17`
for dev.

## Mechanism and split

Replay each sealed longest-first trajectory through exact `CpuFbsForward`.
Capture each 256-coordinate Q12 writer query as sign bits. Select only
body-exact train trajectories and label the reference-length decision terminal.
Fit eight terminal and eight nonterminal prototypes with the existing
nearest-nonterminal minus nearest-terminal score and a threshold chosen only on
the selected train rows under a `floor(nonterminal_rows / 100)` false-stop cap.

Evaluate unchanged on all 449 dev trajectories. The first firing emits BOS;
otherwise the sealed longest-first token is fed forward. The decoder never
reads the reference.

The run records terminal/nonterminal rows, distinct signatures per class,
margin, training stops, every reply's fired step and stop class, and each
unchanged selected-token hash.

## Comparison arms

- Exact-body true gate.
- Class-preserving shuffled exact-body gate using driver seed 157.
- Deepseek-157 all-reference-length gate fitted on all train rows.
- No gate.

## Bar and power

The mechanism passes only for both artifact seeds when all conditions hold:

- at least eight distinct terminal signatures exist in the exact-body train
  set and the gate fits a nonzero threshold under the training false-stop cap;
- train exact replies are at least `6/11`, half the observed oracle ceiling;
- dev exact replies are at least `7/13`, half the observed oracle ceiling;
- true exact replies beat shuffled, all-reference and no-gate by the paired
  two-sigma rule (`candidate_only >= max(2, ceil(2*sqrt(control_only)))`);
- premature BOS stops are at most `1%` of replies: `4/453` train and `4/449`
  dev;
- a second fresh process per artifact seed matches all non-timing decision
  fields.

The half-oracle floor is fixed in units of complete replies. With only 11 train
and 13 dev oracle successes, a one-row gain is below usable power; six and
seven require a replicated majority of available successes and remain far
above the paired two-sigma minimum of two.

The CPU evaluator is deterministic, so a fresh process must reproduce every
decision field exactly; process noise is therefore zero by contract. Artifact
variation is tested by requiring the result on both seeds. Because the oracle
class has only 11 and 13 successes, the absolute floors are wider than one row,
and each controlled comparison must add at least two candidate-only exact
replies.

## Stop rule

Stop and classify **power-limited** if fewer than eight distinct terminal
signatures exist or no nonzero gate fits under the false-stop cap. Stop as an
instrument fault on source or hash drift, an unreplayable or non-monotonic
trace, missing rows, a vacuous shuffled split or nondeterminism. Once dev is
read, do not change labels, prototypes, seed, threshold or bar.

Any outcome closes only this exact-body construction. A failure must not be
reported as rejection of geometric termination generally.

## Cost

- Host: AMD Ryzen 9 9950X, 16 physical cores / 32 threads, 59 GiB RAM;
- build jobs: 16; analysis: one deterministic process as in deepseek-157;
- sealed replay and four evaluation arms;
- four final processes (true, shuffled, all-reference, no-gate are evaluated
  within each seed) plus one repeat per artifact seed: four total processes at
  roughly 9 seconds, about 36 seconds;
- projected incremental release build and checks: under 3 minutes;
- complete preparation, run, evidence, review and merge projection: under
  30 minutes;
- local-only CPU cost: $0; no GPU pod, paid API, remote upload or new panel;
- retained new scratch target: about 5 GB; published evidence: under 10 MB of
  compact JSON and checksums.

## Outcomes

- KEEP: the exact-body gate clears the full bar on both seeds; the next unit
  tests a held-out copy-boundary construction independent of oracle filtering.
- REJECT: the gate fits but fails exact count, control or premature-stop limits;
  close this exact-body prototype-label construction.
- POWER-LIMITED: terminal rows or distinct signatures cannot support eight
  prototypes; do not read that as a mechanism negative.

## Steps

The planned command used the sealed train report and train panel 2711 for
fitting, the sealed dev report and panel 2712 for evaluation, two artifact
seeds, driver seed 157, eight prototypes and `--exact-body`, followed by a
fresh-process repeat for each artifact seed.

## TEST FITNESS

TEST FITNESS: PENDING - no independent pre-run verdict was recorded before
attempt 7. Attempt 7 is therefore preserved as exploratory evidence, not a
registered decision. The fresh-dev confirmation is registered separately in
`cpu-training-pilot-copy-exact-stop-confirm-2026-11-05.md`.

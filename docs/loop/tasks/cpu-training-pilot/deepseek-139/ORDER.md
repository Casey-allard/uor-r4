# ORDER: cpu-training-pilot, deepseek-139 (generated-state stop gate)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `202aa1e29`.

## Goal

Test a separate two-class terminal gate over the positive modulo-256 content
readout's actual generated trajectories. The gate may emit BOS before the code
arm's content choice without changing the content table.

This follows deepseek-138: adding BOS to the shared content targets did not
stop held-out replies and damaged seed-1 content.

## Cancel probe

Deepseek-137's code readout beats writer, sign and shuffled controls but all
replies cap. Deepseek-138 found 84 generated endpoint states per training
seed but zero held-out endpoint selection when BOS shared the content table.
Deepseek-132's teacher-forced stop prototypes failed on generated states.
No unit has trained a separate two-class stop gate on the code arm's actual
generated states.

## Fixed construction

Use the deepseek-137 98/103 split, memory and fitted code readout.

Generate each training reply with that exact code readout. At each generated
decision:

- mark it terminal when its offset equals normalized reference length;
- mark every other observed decision nonterminal, including decisions after
  the reference length; and
- store the complete 2,048-bit code signature and fallback content token.

For each class, deduplicate signatures, choose a Hamming medoid, then up to
seven farthest-first prototypes with lexicographic ties. Score a query as
`nearest_nonterminal - nearest_terminal`; fire BOS at or above a training-only
margin. Among margins with nonterminal false stops at or below 1%, maximize
terminal recall, then lower false stops, then the larger margin.

## Arms and bar

Run no-gate code, true terminal/nonterminal prototypes and
class-preserving shuffled-label prototypes over complete 16-token replies.
Shuffling moves each terminal label by a deterministic nonzero shift.

Both seeds must pass:

- both classes are present and every training row has the registered
  signature width;
- terminal recall is at least **26/84** with nonterminal false stops at or
  below **1%**;
- the true gate produces at least **9 BOS** replies, with at least **8**
  stopping at or before the reference length;
- at least **8** true-gate replies exactly match the reference and stop at BOS;
- true BOS completion exceeds no-gate and shuffled by the prompt-level rule
  `candidate_only >= max(2, ceil(3 * sqrt(control_only)))`;
- true top-1 and matching depth are not below no-gate;
- every ungated generated decision is the code arm's content token, while
  every fired decision is BOS;
- true beats shuffled terminal recall by the paired row rule; and
- seed-1 repeat matches every non-timing field and generated evidence file.

Report prototype hashes, margin, training confusion counts, BOS/premature/
longer/cap/context stops, top-1/depth, openings, exact references, distinct
and repeated replies, workers, wall and RSS.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/claims.json`
- `docs/harness/prep/cpu-training-pilot-content-gate-2026-11-04.md`
- `docs/labs/cpu-training-pilot-content-gate-2026-11-04/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-139/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

No integer-session, serving, CUDA, float-training or Claude-owned changes.

## Done when

1. The CPU crate derives two-class rows from code-arm trajectories, fits
   deterministic prototypes and margin, and emits BOS before content fallback.
2. Tests cover prototype selection, margin constraints, label shuffle,
   ungated/fired token identity, exact/shorter/longer accounting and
   deterministic evaluation.
3. FIT and a fresh GO precede both seeds and a repeat.
4. REPORT and hashed evidence preserve either outcome; notebook and roadmap
   classify the gate.
5. Exact-head review, protected merge and fresh-main verification complete.

## Stop and ask if

- Generated training states lack either class or a valid margin under 1% false
  stops.
- The shuffled label control is unchanged or behaviorally inert.
- Evaluation requires serving, integer-session, CUDA or Claude-owned changes.

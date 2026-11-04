# ORDER: cpu-training-pilot, deepseek-144 (trajectory-context content readout)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `d5d173073`.

## Goal

Test whether adding the generated-state transition to the positive
modulo-256 content signature improves exact next-content selection on held-out
prompts. This targets content quality, not termination: the previous stop
experiments established that endpoint classification alone is insufficient.

This is D84 CPU development research. It does not change integer serving,
`UORLUT01`, CUDA, float training or Claude-owned files.

## Fixed construction

At generated offset `t`, construct a 4,096-bit trajectory signature:

```text
current_code(q_t) || (current_code(q_t) XOR current_code(q_(t-1)))
```

At offset zero the previous signature equals the current signature, so the
second half is zero. Keep the current modulo-256 code unchanged as the first
half.

Use the deepseek-137 98/103 split, memory and content arm. Derive training rows
from actual generated trajectories and the reference content target at the
same offset. Fit with the existing leave-one-out nearest-example rule:

- exact XOR/popcount Hamming distance over the complete trajectory signature;
- commonest target among nearest rows, then smaller token ID;
- the smallest distance covering at least 90% of leave-one-out rows;
- exact writer fallback outside that distance.

The matched control shifts targets through the same row ordering and must
change the effective trajectory-signature mapping. No threshold is selected
on held-out evaluation rows.

## Cancel probe

Deepseek-136 tested current-state sign signatures. Deepseek-137 tested current
state modulo-256 code. Neither includes the generated-state transition.
Deepseek-142 and 143 changed termination labels. This unit keeps content
termination unchanged and adds only first-order trajectory context to the
content decision.

## Preflight

Build the release driver and run `--select-only --trajectory-readout` for both
seeds.

Proceed only if, for each seed:

- trajectory training is structurally valid with at least 640 rows, at least
  16 target values and 90% leave-one-out coverage;
- trajectory leave-one-out top-1 is not below the existing code readout;
- trajectory and shuffled target mappings differ; and
- the source and all five public inputs match.

Preserve and seal the preflight even when it stops.

## Arms and bar

Run writer, code and trajectory over complete 16-token replies. Include the
trajectory target-shuffled control. Both seeds must pass:

- trajectory top-1 and matching-prefix depth are not below code;
- trajectory top-1 beats code and the shuffled trajectory control by the
  prompt-level rule `candidate_only >= max(2, ceil(3 * sqrt(control_only)))`;
- trajectory matching depth beats code by
  `candidate_only >= max(2, ceil(2 * sqrt(control_only)))`;
- every decision is either a trajectory-readout choice within its fitted
  threshold or the exact writer fallback; and
- a one-worker seed-1 repeat matches every non-timing report field and
  generated JSON artifact from the 32-worker run.

Report exact-reference and BOS counts, but they do not decide this
content-selection unit. Report LOO top-1/ambiguity/coverage, generated
top-1/depth/openings/distinct/repeated replies, thresholds, mapping hashes,
workers, wall and RSS.

## Instrument gates

- Source and all five public inputs match.
- Transition packing places current then XOR in the registered order, with a
  zero transition at offset zero.
- Generation uses the same previous-signature history during fitting and
  evaluation.
- Target control preserves signatures and changes effective mappings.
- Readout fallback and lower-token ties are deterministic.
- Worker-count repeat matches generated evidence.
- Full CPU release tests, scoped Clippy, rustfmt and evidence manifests pass.

## Stop rules

Stop on source/input/split drift, trajectory width or transition mismatch,
preflight failure, vacuous shuffle, worker nondeterminism, resource breach or
any need to change integer serving, CUDA, float training or Claude-owned
source.

## Decision consequence

A pass makes first-order trajectory context a supported CPU content candidate
and justifies artifact-bound serving integration. A fail closes this
current-plus-XOR construction and requires a higher-order state, learned
transition geometry or a different content objective.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-trajectory-readout-2026-11-04.md`
- `docs/labs/cpu-training-pilot-trajectory-readout-2026-11-04/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-144/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

## Done when

1. The crate and driver expose trajectory packing and an independent readout
   arm without changing earlier arms or serving.
2. Tests cover transition history, deterministic fitting, mapping shuffle,
   threshold fallback and generated trajectory parity.
3. FIT and fresh GO precede the preflight and full run.
4. Evidence preserves pass or fail and classifies the mechanism.
5. Exact-head review, protected merge and fresh-main verification complete.

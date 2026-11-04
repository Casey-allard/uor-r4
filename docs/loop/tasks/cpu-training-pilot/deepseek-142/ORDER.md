# ORDER: cpu-training-pilot, deepseek-142 (offset-conditioned density stop gate)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `ae8fce0b7`.

## Goal

Test whether a class-balanced nearest-neighbour density gate over every
generated content-state row can identify held-out reply endpoints. This is a
new termination representation, not another margin on writer admission and
not another prototype subset.

Deepseek-139's eight Hamming medoid/farthest-first prototypes recalled only
8/84 and 9/84 training endpoints and produced no exact stopped reply. This
unit retains every row, conditions neighbour selection on the generated reply
offset, and fits the number of neighbours and margin from training rows only.

This is D84 CPU development research. It does not change integer serving,
`UORLUT01`, CUDA, float training or Claude-owned files.

## Fixed construction

Use the deepseek-137 98/103 split, memory and positive modulo-256 content
readout. Generate each training reply with that exact content arm. Every
decision records a boolean terminal label, its generated offset and the
complete 2,048-bit signature.

Fit a separate terminal/nonterminal density gate:

- candidates use 1, 2, 4 or 8 nearest rows because those powers of two divide
  the 16-token cap;
- only same-class rows whose generated offset differs by at most two are
  eligible;
- when fewer than k rows are eligible, each missing neighbour contributes the
  maximum signature distance, so sparse offsets cannot look like confidence;
- duplicated observed rows remain in the class density rather than being
  collapsed to prototypes;
- scores are
  `sum(k nearest nonterminal distances) - sum(k nearest terminal distances)`;
- each candidate k is scored leave-one-row-out. Choose the threshold under a
  1% nonterminal false-stop limit that maximizes terminal recall, then lower
  false stops, larger margin and smaller k;
- the class-preserving control rotates only terminal labels through the same
  row ordering while preserving offsets.

At generation, a score at or above the training margin emits BOS before the
unchanged content choice. All ungated choices remain exactly the positive code
arm's tokens.

## Cancel probe

Deepseek-132 tested teacher-forced full-query signatures. Deepseek-138 added
terminal rows to the shared content target table. Deepseek-139 tested eight
content-state prototypes. Deepseek-131 trained final-gain BOS coordinates.
None used all generated rows with offset-conditioned, class-balanced k-nearest
density.

## Preflight

Build the release driver and run `--select-only` for seeds 1 and 2 with
`--stop-density-neighbors 8 --stop-density-offset-radius 2`.

Proceed to free generation only if, for each seed:

- terminal rows are at least 84 and both classes are present;
- true training recall is at least 26/84;
- nonterminal false stops are at most the registered 1% limit;
- the true and shuffled gate hashes differ; and
- the selected k is one of 1, 2, 4 or 8.

The preflight is recorded even when it stops the run.

## Arms and bar

Run no-gate code, true density and class-preserving shuffled density over
complete 16-token replies. The historical eight-prototype gate remains in the
same report as a reference arm.

Both seeds must pass:

- the true density gate's training recall is at least **26/84**, with
  nonterminal false stops at or below **1%**;
- at least **9/103** replies reach BOS, with at least **8** at or before the
  normalized reference length;
- at least **8/103** replies exactly match the reference and stop at BOS;
- true BOS completion exceeds no-gate and shuffled by the prompt-level rule
  `candidate_only >= max(2, ceil(3 * sqrt(control_only)))`;
- true top-1 and matching-prefix depth are not below no-gate;
- every ungated decision is the code arm's content token and every fired
  decision is BOS;
- on terminal training rows, true density beats the shuffled density by the
  row-level rule `candidate_only >= max(2, ceil(2 * sqrt(control_only)))`;
- true and shuffled gate hashes differ and the selected row counts match the
  generated training rows; and
- the seed-1 repeat at one worker matches every non-timing report field and
  every generated JSON file from the 32-worker run.

Report selected k, offset radius, row counts, margin, training confusion,
BOS exact/shorter/longer stops, top-1/depth, openings, exact references,
distinct/repeated replies, workers, wall, RSS and gate hashes.

## Minimum test power

The split and model seeds are fixed. The comparison uses exact generated
trajectories, a matched label control and a stronger 1-worker repeat. There is
no sampling noise in a token or gate decision. Two trained artifacts are
required to avoid selecting one seed's endpoint geometry.

## Instrument gates

- Source and all five public inputs match.
- Terminal states are exactly generated offsets at normalized reference
  length; every other generated row is nonterminal.
- Offset conditioning, neighbour selection, missing-neighbour penalty and
  margin fitting are deterministic.
- The shuffled control preserves class counts and offsets while changing
  terminal membership.
- Fired decisions are BOS; ungated token mismatches are zero.
- One-worker and 32-worker evaluations match for generated evidence.
- The full CPU release suite, scoped Clippy, rustfmt and evidence checks pass.

## Stop rules

Stop on source/input/split drift, missing classes, a preflight bar failure,
vacuous shuffle, ungated token mismatch, worker nondeterminism, resource
breach or any need to change integer serving, CUDA, float training or
Claude-owned source.

## Decision consequence

A pass makes offset-conditioned generated-state density a supported CPU
termination candidate and justifies a later artifact/serving integration.
A fail closes this full-row density representation and requires a continuation
objective or content-trajectory representation that changes how generated
states approach terminality.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-stop-density-2026-11-04.md`
- `docs/labs/cpu-training-pilot-stop-density-2026-11-04/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-142/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

## Done when

1. The CPU crate exposes an offset-conditioned class-balanced density gate and
   its own driver switch without replacing earlier arms.
2. Tests cover all-row density, offset separation, missing-neighbour penalty,
   deterministic fitting, label shuffle, BOS-only firing and exact ungated
   content.
3. FIT and fresh GO precede the registered preflight and full runs.
4. Evidence preserves pass or fail, and the report classifies the mechanism.
5. Exact-head review, protected merge and fresh-main verification complete.

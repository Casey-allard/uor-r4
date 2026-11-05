# ORDER: cpu-training-pilot, deepseek-167 (learned geometric copy read)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `127c6e310`.

## Goal

Test whether a learned query-code read can rank exact-copy candidates better
than depth-first selection, while keeping every output token inside the
artifact-derived exact-copy candidate set.

Deepseek-165 rejected raw sign-Hamming distance between the current writer
query and each token's static table row. This unit changes the mechanism:
train an exact Hamming nearest-example readout from generated-state modulo-256
query codes and reference targets, then use it only as an arbiter over tokens
that exact copy has already made available.

## Fixed construction

For each artifact seed on train panel 2711:

1. Generate every bounded reply with longest-32 exact copy.
2. Before each decision, pack the complete 256-coordinate Q12 writer query as
   the existing 2,048-bit modulo-256 code.
3. Pair that code with the normalized reference token at the same offset.
4. Fit the existing exact nearest-example readout, including its 90%
   leave-one-out coverage threshold, commonest-target vote and smaller-token
   tie rule.
5. For a learned-copy decision, read the nearest predicted token. If it is in
   the current exact-suffix candidate set and within the learned threshold,
   select its deepest candidate occurrence. Otherwise fall back to longest-32.
6. Use the same candidate-constrained procedure with a deterministic
   target-shuffled readout.

No model state, memory, artifact, stopping rule, serving path or dense
operation changes. Every learned override feeds the selected token back
through the exact `CpuFbsForward`.

## Data split and power

- Train/fit: panel 2711.
- Fresh decision: a new `cell=dev,dev` panel, seed 2763, generated only after
  FIT and GO.
- Two public artifact seeds.
- At least 200 reference-reachable replies per seed and panel.
- At least 640 generated-state training rows and 16 target values, enforced by
  the existing readout fitter.
- At least one learned target must land inside the exact-copy candidate set on
  each fresh seed.
- A non-vacuous shuffled-target readout and random-32 control.
- A fresh-process repeat of both fresh-dev seeds.

## Bar

For both artifact seeds on the fresh panel, learned-copy correct body
positions must:

- beat longest-32, longest-4, `k2>k1`, fixed depth 1, random-32, writer,
  static sign-Hamming and shuffled learned-copy by the paired two-sigma rule;
- change at least one selected token relative to longest-32;
- use the learned readout as a candidate arbiter at least once;
- preserve zero premature BOS stops and zero ungated token mismatches;
- have at least 200 reachable replies;
- repeat byte-identically after timing fields are removed.

Train diagnostics must not exceed the readout's one-nearest training design.
Exact and BOS-complete counts are reported but expected to remain zero because
the stopping path is unchanged.

## Stop rule

Stop on source, artifact, tokenizer, panel, readout or hash drift; fewer than
200 reachable replies; fitter coverage below 90%; a vacuous or absent learned
override; ungated token mismatch; nondeterminism; or any need to change
serving, CUDA, float training, termination or Claude-owned source. Once the
fresh panel is read, do not change the code packing, threshold, tie rule, cap,
controls or bar.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-copy-learned-read-2026-11-05.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-167/**`
- `docs/labs/cpu-training-pilot-copy-key-reply-2026-11-05/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/integration/current-state.md`
- `docs/harness/claims.json`

## Done when

1. The learned readout and candidate-constrained arm have finite independent
   tests, deterministic scoring and fail-closed input validation.
2. FIT and GO precede fresh seed 2763.
3. Both artifact seeds run on train and fresh panels with deterministic
   repeats and sealed compact evidence.
4. REPORT classifies the result without claiming complete replies or language.
5. Exact-head review, protected merge, fresh-main verification and claim
   release complete.

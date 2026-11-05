# ORDER: cpu-training-pilot, deepseek-175 (reference-prefix ranker)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `1f3b25cd6`.

## Goal

Resolve deepseek-173's candidate-supervision insufficiency without changing
inference. Keep the same candidate-only integer pairwise ranker, but generate
its training groups from teacher-forced reference prefixes instead of
longest-32-generated trajectories.

## Fixed construction

1. Start each train-panel 2711 sample with its prompt and reply start.
2. At each reference position, collect unique exact-copy candidates at depths
   one through eight, excluding BOS.
3. Supervise only groups with at least two candidates where the reference next
   token is present, then feed the reference token to reach the next state.
4. Reuse deepseek-173's seven bounded features, deepest duplicate occurrence,
   eight-pass bounded integer perceptron, shuffled-target and depth-only
   controls unchanged.
5. Evaluate unchanged on complete generated replies; reference tokens never
   enter inference.

No serving, CUDA, float inference, termination or Claude-owned source change is
permitted.

## Data and power

- train on panel 2711 reference prefixes;
- fresh `cell=dev,dev`, seed 2767, only after FIT and GO;
- two public artifact seeds and at least 200 reachable replies per seed;
- at least 640 supervised reference-prefix candidate groups;
- non-vacuous changes versus longest-32, shuffled ranker and depth-only;
- fresh-process repeats of both seeds.

## Bar

For both artifact seeds, the ranker trained on reference prefixes must beat
longest-32, shuffled-rank and depth-only on fresh correct body positions by the
paired two-sigma rule. It must change at least one token against each, preserve
zero premature BOS and ungated mismatches, and repeat byte-identically after
timing fields are removed.

## Stop rule

Stop on source, artifact, tokenizer, panel, feature, label, weight or report
drift; fewer than 640 supervised groups; fewer than 200 reachable replies;
vacuous controls; nondeterminism; or any need to change serving, CUDA, float
inference, termination or Claude-owned source. Once seed 2767 is read, do not
retune this construction.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-copy-ranker-reference-2026-11-05.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-175/**`
- `docs/labs/cpu-training-pilot-copy-key-reply-2026-11-05/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/integration/current-state.md`
- `docs/harness/claims.json`

## Done when

1. Reference-prefix and generated-reply paths are separate, tested and leak
   closed.
2. FIT and GO precede seed 2767.
3. Both artifacts run with deterministic repeats and compact evidence.
4. REPORT distinguishes teacher-forced supervision from free-generation
   behavior and makes no complete-reply or language claim.
5. Reviewed merge, fresh-main verification and claim release complete.

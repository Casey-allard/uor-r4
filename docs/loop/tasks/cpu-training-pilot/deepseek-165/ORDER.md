# ORDER: cpu-training-pilot, deepseek-165 (geometric copy arbitration)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `7018b09cd`.

## Goal

Test whether the shared FBS query geometry can choose the correct token among
the exact-copy candidates. Deepseek-163 showed that extending exact depth from
four to 32 changed no scored body token. This unit keeps the exact candidate
set but replaces depth-first selection with sign-Hamming agreement between the
current Q12 writer query and each candidate token's shared vector-table row.

This is a CPU content-selection surrogate. It changes no serving artifact,
writer state, memory, termination path, CUDA or Claude-owned source.

## Cancel probe

At the base, longest-32 and longest-4 tie at 409/4,661 train and 300/4,758
fresh-dev correct body positions. No prior complete-reply unit ranks exact-copy
candidates by their geometric distance to the current writer query.

## Fixed construction

At each generated decision, retain the exact suffix candidates for depths one
through 32. For each available candidate token, compute sign-Hamming distance
between the current 256-coordinate Q12 writer query and that token's 256-value
shared vector-table row:

- a query or table coordinate at or above zero maps to bit one;
- XOR and popcount give the distance;
- choose the smallest distance;
- tie-break toward the deepest exact-suffix match, then lower token id.

The arm only replaces the next token. Every chosen token is fed through the
same exact `CpuFbsForward`; memory, point-key fallback and stopping stay
unchanged.

Controls are longest-32 depth-only, longest-4, `k2>k1`, fixed depth one,
random available depth in 1..32, and the unmodified writer. The legacy random
1..4 arm remains represented but is not a primary control.

## Data split and power

- Train diagnostic: existing panel 2711.
- Fresh decision: new `cell=dev,dev` panel, seed 2762, generated only after FIT
  and GO.
- Two artifact seeds.
- At least 200 reference-reachable replies per seed/panel.
- A non-vacuous random-32 control.
- A fresh-process repeat of both fresh-dev seeds.

## Bar

For both artifact seeds on the fresh panel, geometric arbitration must:

- beat longest-32, longest-4, `k2>k1`, fixed depth 1, random-32 and writer on
  correct reference body positions by the paired two-sigma rule;
- change at least one selected token relative to longest-32;
- preserve zero premature BOS stops and zero ungated token mismatches;
- have at least 200 reachable replies;
- repeat byte-identically after timing fields are removed.

On train, geometric body positions must not fall below longest-32. Exact and
BOS-complete counts are reported but expected to remain zero because
termination is unchanged.

## Stop rule

Stop on source, artifact, tokenizer, panel or hash drift; unavailable candidate
geometry; fewer than 200 reachable replies; a vacuous random or geometric
change control; ungated token mismatch; nondeterminism; or any need to change
serving, CUDA, float training or Claude-owned source.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-copy-geometric-2026-11-05.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-165/**`
- `docs/labs/cpu-training-pilot-copy-key-reply-2026-11-05/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/integration/current-state.md`
- `docs/harness/claims.json`

## Done when

1. Candidate geometry, deterministic ties, exact replay and all controls have
   finite tests.
2. FIT and GO precede the fresh seed-2762 panel.
3. Both artifacts run unchanged with deterministic repeats and sealed public
   evidence.
4. REPORT classifies the body-position result without claiming complete replies.
5. Exact-head review, protected merge and fresh-main verification complete.

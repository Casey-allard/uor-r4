# REPORT: cpu-training-pilot, deepseek-177

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Sheet:
[geometric candidate union](../../../../harness/prep/cpu-training-pilot-geometric-candidate-union-2026-11-05.md).
Base: `04fae9eaf`; implementation pin: `dc17b847a`.

## Status

Complete as **INSUFFICIENT_TRAINING_DATA**. Seed 2768 is closed for this
construction. No complete-reply, language or serving claim follows.

## Question

Does a denser candidate union, formed from the writer's top 128 non-BOS
candidates, exact-copy depths one through 32 and the writer fallback, supply
enough supervised choices to train the integer pairwise ranker?

## What did not work

The candidate union raised seed-1 supervision from the earlier 85 exact-copy
groups to **2,442** groups with **332** distinct target tokens. Artifact seed 2
stopped during the same eligibility check with **1,847** groups, below the
registered **2,048** floor. This is substantially denser than exact-copy alone,
but it remains artifact-seed dependent and did not clear the fixed power floor.

The registered shell automation mislabeled that preserved seed-2 stop as
`INSTRUMENT_FAULT`: its matcher expected the phrase in a different word order.
The actual stop rule names fewer than 2,048 groups as
`INSUFFICIENT_TRAINING_DATA`. The raw label and a post-run classification
correction are both preserved. Seed 2768 was not rerun, no construction was
retuned, and no repeat ran.

Seed 1 is diagnostic only because the two-seed bar was not reached. Its trained
ranker scored **265** correct body positions against **601** for longest-32,
**61** for the writer and **46** for fixed geometry-only. It passed the writer,
geometry-only and shuffled comparisons but failed longest-32. No arm emitted a
premature BOS or an exact reference on the reachable seed-1 replies.

## What changed

- Added the geometric candidate-union trainer and bounded eight-feature
  integer pairwise ranker over top-128 non-BOS writer, exact-depth-32 and
  fallback candidates.
- Kept deepest exact-copy source and writer rank, reference-free inference,
  lower-token ties and all registered controls.
- Hardened deterministic replay so every evaluation arm emits reply and
  decision SHA-256 values that participate in the fresh-process comparison.
- Added direct tests for top-128 membership, fallback identity, depth 32 and
  deepest source, reference isolation, geometry-only behavior, shuffled
  targets, ties and exact power floors.

No serving, CUDA, float-inference, termination or Claude-owned source changed.

## Measurements

- reviewed implementation head: `7a326d4162e952108721dd716e38c51790271ab3`;
- FIT: issue #3506 comment `6658474404`;
- GO docs head: `2183f2f69958690ece84da309de6cf21d7b1c092`;
- reviewed delivery head: `ad2fdfa0ee51ed7f5a5fdf1d54b73cfd69895cc0`;
- PR: #3725;
- protected merge: `4ea0976615b4b59e4394c30476931f15291b0951`;
- fresh panel: seed 2768, `dev,dev`, 128 conversations, 15,654 tokens;
- panel tokens SHA-256:
  `ba38a53172a24bd8f20fae676d304c456f9dceaf37ec9294c6232ebefc0f152e`;
- seed 1 supervised groups: **2,442**; target tokens: **332**;
- seed 1 candidate rows: **313,326**; generated rows: **442**;
- seed 2 supervised groups: **1,847**; required: **2,048**;
- seed 1 wall: **3.94 s**, peak RSS **217,216 KiB**;
- seed 2 eligibility wall: **0.31 s**, peak RSS **204,444 KiB**;
- build: **7.05 s**; generator build: **10.18 s**; panel: **0.55 s**;
- CPU library: **326 passed**, 4 expected ignores;
- driver binary test: **1 passed**;
- raw local evidence: **68,079 bytes**, sealed and verified;

Published evidence is in
[`../../../../labs/cpu-training-pilot-copy-key-reply-2026-11-05/`](../../../../labs/cpu-training-pilot-copy-key-reply-2026-11-05/)
and [`evidence/geometric-candidate-union-2768/`](../../../../labs/cpu-training-pilot-copy-key-reply-2026-11-05/evidence/geometric-candidate-union-2768/).

## Checks run at the head

- Independent fresh-context TEST FITNESS: **FIT**.
- Independent fresh-context pre-run review: **GO** after recording the FIT block.
- CPU library, driver test, strict Clippy, package rustfmt and `git diff --check`.
- Registered source, panel, tokenizer, artifact and generator pins.
- Registered shell and jq syntax.
- Raw and published evidence seals.
- Lane-scope and no-corpus checks.
- Exact-head approval, all required CI checks and protected merge.
- Fresh-main CPU library: **326 passed**, 4 expected ignores.
- Fresh-main driver binary test: **1 passed**; strict Clippy passed.
- Fresh-main source pins, published evidence and no-corpus checks passed.

## Skills used

No system-provided Codex skill applied. The repository's experiment
pre-registration, compute-admission and evidence-delivery procedures were used.

## Still open

The writer/copy union is a useful supervision direction but did not reach the
fixed floor on both public artifacts. A later experiment needs a newly
registered candidate construction or objective that clears the floor on both
artifacts without lowering it. Seed 2768 must not be reused.

The project still needs an independent termination path and a genuinely fresh
complete-reply capability test before a useful local chat claim can be made.

## Plain summary

The broader geometric candidate set nearly doubled useful supervision on one
artifact, but the other still fell 201 groups short of the fixed floor. The
fresh run therefore stopped correctly as insufficient training data; its
automation merely printed the wrong outcome name. This closes one broader
candidate configuration while preserving the next design direction.

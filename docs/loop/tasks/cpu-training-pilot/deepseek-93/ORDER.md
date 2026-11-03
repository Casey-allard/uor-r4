# ORDER: full-path memory-weight training, deepseek-93

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `58470eb321d1c3ac37a6bc9f804f18eab0af9702`.

## Goal

Add the next CPU training-through-serving surrogate: update the turn-memory
movement weights from exact full-path teacher-forced decisions. The initial
deepseek-91/92 path derives memory entries but fixes every movement weight at
one. This unit instead patches the exported Q24 weight tables, reparses the
artifact, runs the complete FBS forward with opening/reply rows active, and
accepts only a deterministic improvement in served target distance.

This is bounded CPU development evidence on a finite fixture. It is not a
language, corpus-scale or GPU-qualification result.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-93/**`
- `docs/research/cpu-training-program.md`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

## Done when

1. A trainer accepts an exported artifact with turn memory and explicit
   prompt/reply samples, validates the memory tables and samples, and keeps
   the artifact as its only model state.
2. Its served objective runs the worker's complete forced token sequence,
   signals each reply start, records every query and decision, and sums each
   decision's L1 distance to the actual target token's writer row.
3. A step tries deterministic positive and negative Q24 movements of every
   opening and reply weight coordinate, clamps to a registered bound, patches
   the named artifact table, and reparses the complete artifact before
   evaluation.
4. It commits exactly one coordinate only when served top-1 on the training
   panel does not fall and target-distance loss strictly falls; ties use table
   order and lower coordinate so reconstruction is deterministic.
5. Every accepted artifact retains zero complete CPU/serving query, token and
   kept-cell mismatches on both training samples. The test must fail when that
   audit is deliberately bypassed on a planted mismatch.
6. Two independent constructions produce byte-identical artifacts and step
   receipts. A no-improvement step changes no bytes and reports `updated:
   false`.
7. Focused and full CPU crate tests, strict package Clippy, package formatting,
   the requested report checks and local loop policy pass. If a package-wide
   baseline check is already red on `main`, record the exact pre-existing
   failure rather than claiming it passed.
8. The REPORT reports the initial and final weights, objective values,
   artifact hashes and exact audit counts, and states plainly whether this
   component is useful for further CPU training work.

## Stop if

- The Q24 weight table cannot be patched without changing unrelated bytes.
- A parsed artifact or its complete CPU/serving audit differs.
- The finite fixture cannot produce a strict served-distance improvement;
  report that as a negative rather than accepting a no-op.
- The update needs a dense matrix product, softmax/squashing or a float-serving
  path.

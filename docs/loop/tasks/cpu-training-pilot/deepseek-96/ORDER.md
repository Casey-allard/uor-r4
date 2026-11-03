# ORDER: cached memory-trajectory training, deepseek-96

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `3b893ddc0ee29a43750632799aaae340540a0bbe`.

## Goal

Remove redundant full-network evaluation from memory-weight candidate search.
The teacher-forced token sequence fixes the recurrence, Hamming and pre-memory
normalization trajectory. Cache that trajectory once per training panel, then
evaluate candidate opening/reply movement weights by applying only the final
memory transform and readout.

The optimized path must reproduce the existing full-forward trainer exactly:
same accepted coordinates, same step sequence, same trained artifact bytes and
same held-out readings. The only intended change is lower wall time and CPU
work.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-memory-trace-2026-11-03.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-96/**`
- `docs/research/cpu-training-program.md`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/harness/claims.json`

## Done when

1. The FBS forward exposes an internal trace containing the exact pre-memory
   normalization vector and memory-hit identity for each teacher-forced
   decision without changing the public serving path.
2. Candidate evaluation applies the same Q24 memory movement, Q12 conversion,
   writer decision and point fallback as the full forward while reusing the
   fixed pre-memory trajectory.
3. A reference flag keeps the full-forward path available for equivalence
   controls. Optimized and reference training produce identical step receipts,
   accepted coordinates, trained artifact SHA-256 and held-out readings.
4. The optimized panel driver runs the existing two-seed repeated-key comparison
   under the FIT speed sheet and retains every input, artifact, reading and
   parity check.
5. The speed bar is fixed before the registered run. If it fails, retain the
   negative and do not claim acceleration.
6. Focused and full CPU tests, strict Clippy, formatting, evidence, report,
   roadmap and loop checks pass.
7. Independent trace samples, matched evaluation arms and parity audits use the
   same bounded Rayon pool; ordered integer reduction preserves the serial tie
   and saturation behavior.

## Stop if

- Any optimized step differs from the reference path.
- Any accepted artifact loses complete CPU/serving parity.
- Caching requires a float serving path, dense matrix product, softmax/squashing,
  or changing teacher-forced token state.
- The current full-forward path cannot be retained as an executable reference.

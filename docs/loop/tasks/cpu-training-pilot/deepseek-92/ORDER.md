# ORDER: cpu-training-pilot, deepseek-92

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `1fa3d1724f7eeb130351f2c1e4b49e168cea24f4`.

## Goal

Extend the finite data-derived full-path artifact from deepseek-91 into a
matched control comparison. Hold the trained writer and all training examples
fixed, derive memory-off, point-off, Hamming/no-read and fully stripped arms,
and compare their complete served decisions on the same sequences.

This is CPU development evidence on a synthetic initial trunk. It is not a
language comparison or GPU qualification.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-92/**`
- `docs/research/cpu-training-program.md`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

## Done when

1. The deepseek-91 artifact construction is reused without changing its writer
   training, Hamming margin, memory examples or point example.
2. The comparison contains `full`, `memory_off`, `point_off`, `hamming_off`
   and `stripped` arms. Each arm is exported, parsed and audited with exact CPU
   versus integer-serving query, token and kept-cell parity.
3. The same two sequences are run through every arm. The full arm must retain
   the opening memory change, two reply-memory changes and point fallback.
4. `memory_off` must remove both memory query moves; `point_off` must retain
   memory but lose the point fallback; `hamming_off` must remove Hamming while
   retaining the serialized memory and point records; `stripped` removes all
   three mechanisms.
5. The report records per-arm artifact hashes, query/token decision counts,
   target decisions and the first divergent positions. It keeps negative or
   non-isolating outcomes rather than manufacturing a ranking win.
6. Two independent constructions are byte-identical per arm.
7. Focused and full CPU crate tests, strict Clippy, formatting, report,
   roadmap and local PR checks pass.

## Stop if

- A control cannot be parsed or retains a mechanism its name removes.
- An arm loses exact CPU/serving parity.
- The point key does not isolate the point decision after memory and Hamming
  are held fixed.

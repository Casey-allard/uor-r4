# CUDA parity result: discrete native path

Head: `codex-5` implementation. Parent contract:
[claude-371/PARITY.md](../claude-371/PARITY.md).

## Conditions

- GPU: NVIDIA GeForce RTX 4090, compute capability 8.9.
- CUDA: 13.1, AOT target `compute_89`.
- Host: `wick`.
- Fixtures: Claude-371's Route and Flock `UORLUT01` bytes.
- Rows: all 39 prefixes of lengths 1 through 3 over three tokens.
- Build: `NVCC=/usr/local/cuda/bin/nvcc UOR_NVCC_ARCH=compute_89 UOR_REQUIRE_CUDA=1`.

The synthetic fixture provenance bytes are not production artifact hashes. The finite test
therefore uses documented qualification loaders that still validate container structure, table
spans, orbit roots, derived cells and CUDA device buffers. The production loader remains strict.

## Compared words

| Area | Comparison |
|---|---|
| Forward carry | 624 `i32` read-vector words for each fixture, CUDA vs CPU vs serving |
| Memory address | every admitted cell and 16-word padded row, CUDA vs serving |
| Emitted identity | every top-8 word and top-1 token, CUDA vs serving |
| Counterfactual | all 16 `u64` loss words, CUDA vs CPU |
| Chooser | candidate, before, after, epoch and sensitivity, CUDA vs CPU |
| Apply | applied/no-op decision and epoch |
| Post-apply state | table, routes, gates and epoch, CUDA vs CPU artifact state |
| Stale reuse | CUDA refuses with "stale"; CPU refuses with status 3 |
| Planted carry fault | one changed CPU carry word is detected |

## Measured rows

Route fixture:

- losses:
  `[14176, 14464, 14176, 14176, 14176, 14176, 14176, 13696, 14176, 14176, 14176, 14464, 13696, 14176, 14176, 13696]`
- candidate `7`; applied; epoch `0 -> 1`.

Flock fixture:

- losses:
  `[14912, 13952, 14912, 18446744073709551615, 18446744073709551615, 18446744073709551615, 18446744073709551615, 18446744073709551615, 18446744073709551615, 18446744073709551615, 18446744073709551615, 18446744073709551615, 18446744073709551615, 18446744073709551615, 18446744073709551615, 18446744073709551615]`
- candidate `1`; applied; epoch `0 -> 1`.

Result: every listed comparison is exact.

## What remains indirect

- The Route fixture still does not expose the chosen port at each stage as a serving word.
  The current CUDA test sees it only when it changes carry, address or emitted identity.
- The Flock fixture still does not expose the kept-node list as a serving word. CUDA
  `forward_traced` can expose the device trace, but serving has no equivalent public trace.
- No stop state exists on either side.
- The serving loader still accepts a Route artifact with a non-zero gate while CPU training
  refuses it.

These are the next parity gaps, not failures of the current CUDA forward/proposal/apply leg.

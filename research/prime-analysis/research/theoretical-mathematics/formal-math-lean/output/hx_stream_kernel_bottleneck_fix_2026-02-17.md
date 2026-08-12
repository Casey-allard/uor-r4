# Stream Kernel Bottleneck Fix (2026-02-17)

## Bottleneck identified
Cold-path runtime was dominated by `research/hx_bridge_probe.py::stream_weighted_events`.

## Fix applied
- Added `zero_block` tuning hook to `stream_weighted_events`.
- Replaced fixed zero block (`16`) with auto-tuned block size:
  - `32` when zero-count is large (`>=384`)
  - `64` otherwise
- Kept output semantics unchanged; only accumulation order differs slightly at floating-point roundoff scale.

## Clean serial no-cache benchmarks (same grids)

### A4 (`n<=5e6`, `m_ref=512`)
- Before kernel fix: `48.45s`
- After kernel fix: `37.18s`
- Speedup: `1.30x`

### A1 (`n<=1e7`, `m_ref=512`)
- Before kernel fix: `66.71s`
- After kernel fix: `57.12s`
- Speedup: `1.17x`

### A3 (`n<=1e7`, `m_zero=128`)
- Before kernel fix: `27.98s`
- After kernel fix: `16.75s`
- Speedup: `1.67x`

## Numeric stability
- Theorem/constant outputs remain stable; observed deltas are at floating-point roundoff scale (~`1e-15` to `1e-18`).
- Hold/violation decisions are unchanged.

## Operational note
- Parallel no-cache benchmarking of multiple heavy scripts at once can mislead due CPU/memory contention.
- Use serial no-cache runs for fair bottleneck comparisons.


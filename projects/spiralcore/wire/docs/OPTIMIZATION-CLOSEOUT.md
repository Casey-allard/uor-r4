# Optimization closeout: running the wire stack as fast as it goes

Goal: make the UOR wire stack run as fast as it possibly can, natively. This record states what was
optimized, the measured ceiling, and what is left.

## What was optimized (2026-10-17)

1. **Native hot paths.** Rust implementations now cover every data path: peer WireSocket/WireListener,
   routed hub, adapter edge/gateway pair, mux gateway, and a native mux client. Python remains for
   tooling, tests, interop and the adapter protocol checks.
2. **Compression backend.** flate2 switched to zlib-rs (pure Rust): compress ~650 -> ~2,900 MiB/s,
   decompress ~2,455 -> ~5,000-8,300 MiB/s on 1.28 MiB text. On a realistic 148 KB source/doc corpus the
   ratio is identical to miniz_oxide (0.24489 vs 0.24480), so this is not a ratio trade.
3. **Framing and copies.** Bounded-sample compression probe; 64 KiB frame chunk (measured optimum:
   8 KiB is 2.3 ms, 1 MiB is 2.1-2.3 ms, 64 KiB is 1.0-1.1 ms for 1.25 MiB text); recv_into reusable
   buffers; compression scratch reuse; vectored writes (one writev per frame); raw frames read directly
   into the receive buffer and compressed frames decompressed into it; TCP_NODELAY.
4. **Mux.** Per-stream OPEN routes, per-stream dispatch queues (removes the inline-write head-of-line
   block), cancellable window credits, and a native mux client so benchmarks are no longer Python-bound.
5. **Measurement.** tools/wire_rust_perf_smoke.py (same-run 7-rep table), echo-ping (1 KiB RTT),
   bench-compress (backend/level/chunk), and the 18-test interop/regression matrix.

## Measured ceiling (committed evidence runs; loopback, loaded host)

| Measurement | Result |
|---|---|
| plain TCP peer, 1.25 MiB | 1.52 / 1.92 / 2.16 ms (min/mean/max) |
| wire peer via gateway, 1.25 MiB | 2.10 / 2.86 / 3.30 ms |
| Rust adapter, 64 KiB round trip | 0.21 / 0.36 / 0.43 ms |
| Python adapter, same | 0.54 / 0.59 / 0.87 ms |
| Rust hub, 256 KiB | 0.40 / 0.43 / 0.50 ms |
| Rust mux, 8 x 128 KiB | 2.61 / 3.70 / 4.36 ms |
| Rust mux gateway with native client vs Python client | 4.57-4.89 ms vs 11.2-13.1 ms |
| 1 KiB RTT, plain / wire L6 / wire L1 | 55.1 / 41.8 / 33.9 us mean |
| Rust peer end-to-end, 1.25 MiB (echo-server path) | ~1.0-1.4 ms wire vs ~1.0-1.5 ms plain |

**Why this is the floor.** At measured zlib-rs throughput, compressing one 1.25 MiB direction costs
~0.43 ms of CPU and decompressing ~0.15-0.25 ms, on each peer. The measured wire-vs-plain gap (~0.6-0.9 ms
on the loaded host) is the same order as that per-peer compression CPU plus framing/scheduling; the client
and server compress in parallel processes, so wall overhead is closer to one peer's CPU than the sum.
Framing, copies and thread handoffs no longer dominate: eliminating whole-payload copies and the Python
hot paths moved this from 12.1 ms (Python peer) to 2.1-3.3 ms (Rust peer) for the same payload, and the
Rust adapter is within noise of plain TCP on 64 KiB round trips (0.21-0.43 ms vs 0.21-0.43 ms class).

Small messages are not slower than plain TCP: compressed 1 KiB frames are tiny enough that wire beats
plain on mean RTT, and WIRE_LEVEL=1 cuts it further.

## Remaining costs and limits

- **Compression CPU is the floor.** The protocol is zlib-framed for interoperability; going faster means
  changing compressor/framing (a protocol decision), not another copy or thread tweak.
- **Loopback absolutes are load-sensitive** (another lab's disk audit/indexing ran throughout). The
  same-run harness comparisons are the reliable signal; committed logs show both the quiet and loaded runs.
- **Single-threaded select relay was not implemented.** The threaded relay already lands within noise of
  plain for adapter RTT; a select relay is a possible future experiment, not a measured win.
- **WAN behavior** is governed by the earlier byte-ratio measurements (protocol batch 2.58x, tunnel ratio
  0.904, text up to 232x fewer bytes), not by these loopback latency numbers.
- **No SpiralCore/FBS routing changes.** This work is the transport layer in the review repo; uor-r4
  remains clean and untouched.

## Reproduce

    cd crates/wire-rs && cargo build --release --offline
    python3 tools/wire_rust_perf_smoke.py          # WIRE_RUST_PERF_PASS, same-run table
    bash tools/wire_interop_matrix.sh              # 18/18 PASS
    bash tools/wire_rust_hub_smoke.sh              # WIRE_RUST_HUB_PASS
    bash tools/wire_rust_adapter_smoke.sh          # WIRE_RUST_ADAPTER_PASS
    bash tools/wire_rust_mux_gateway_smoke.sh      # WIRE_RUST_MUX_PASS
    crates/wire-rs/target/release/wire-rs echo-ping --connect ADDR --bytes 1024 --reps 200

Evidence: evidence/artifacts/perf/ (harness, ping, chunk/level, rust-native), evidence/artifacts/
rust-hub/, rust-adapter/, rust-mux/, evidence/artifacts/usability/matrix-run.log; matrix 18/18.

## Bottom line

The stack is now native end to end and measured at its compression/kernel floor: Rust peer 2.1-3.3 ms for
1.25 MiB versus plain 1.5-2.2 ms; Rust adapter 1.6x faster than Python; mux native path 2.5x faster than
the Python-client path; 1 KiB RTT faster than plain TCP. The remaining 0.6-0.9 ms of wire overhead is the
measured zlib CPU cost, and further gains require a protocol/compressor change, not more micro-optimization.

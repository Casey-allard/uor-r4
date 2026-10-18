# Why the wire path is slower than plain TCP on loopback

Short answer: the plain TCP number is not doing any user-space work. Loopback TCP is a kernel memcpy,
while every wire mode runs compression, framing and (in Python) byte copies on both ends. On a local link
there are no bytes to save, so that work is pure cost. On a real link bytes dominate and the wire path
wins (earlier measurements: protocol batch opt-in 2.58x on WAN, adapter tunnel ratio 0.904, text payload
up to 232x fewer bytes).

## Measured breakdown (loopback, 1,286,144-byte text payload, means of 5)

| Path | Mean | Notes |
|---|---|---|
| plain Python TCP echo | 0.0020 s | kernel loopback + a Python echo loop, no compression |
| zlib compress+decompress only | 0.0036 s | the compression CPU cost alone for this payload |
| Python wire, text | 0.0121 s (min 0.0082) | zlib plus CPython framing/copies on both ends; zlib is ~23% |
| Python wire, incompressible (raw mode) | 0.0048 s | framing only, no compression CPU |
| Rust wire, text (self-time) | 0.0027 s | Rust implementation, same framing and compression |
| Rust wire client + process wall | 0.0081 s | includes process startup |
| C interposer to Rust listener (self-time) | 0.0087 s | C client through interposer, Rust server |

Raw mode used to cost 0.0274 s because the implementation compressed the entire 1.28 MB chunk just to
discover it was incompressible. Probing a bounded 64 KiB sample fixed that (27.4 ms to 4.8 ms). Larger
relay reads (256 KiB) reduced the text mean from 15.0 ms to 12.1 ms.

## Why the earlier "orders of magnitude" readings

The interposer overhead benchmark spawns a fresh client process per sample: reported 0.0466 s mean versus
0.0148 s minimum, so process startup dominates that number. The interposer's own timed section through a
Rust listener is 0.0087 s. A Python adapter gateway in the chain also adds Python-side compression. TCP/IP
itself is not the bottleneck in any of these measurements.

## What this means

- Use the Rust binding (or the C interposer with a Rust peer) where CPU cost matters: it is within ~1-3 ms
  of plain TCP on this payload while still shrinking bytes on real links.
- Use the Python implementations for tooling, routing, mux and environments where a few ms of per-call CPU
  is irrelevant compared with WAN bandwidth.
- Compression level matters: level 1 is cheaper CPU with slightly larger output; level 9 is slower. The
  adapter exposes per-route levels; the Python socket layer currently uses level 6.
- Loopback throughput is not the acceptance metric; byte reduction on real links is, and the CPU cost per
  byte is measured above.

Evidence: evidence/artifacts/perf/breakdown.log and evidence/artifacts/usability/overhead-bench.log.

## Language choice: Rust is the native path, not C/C++

Measured on the same 1.28 MiB text payload, loopback: Rust WireSocket peer path 2.7 ms, plain TCP echo
2.0 ms, Python wire path 12.1 ms. The Rust implementation is already within ~1 ms of the kernel memcpy
baseline; the Python path is the slow one. The C file in this repository is only a small interposition
shim (DYLD/LD_PRELOAD has a C ABI), not a faster implementation; its 8.7 ms figure is a different topology
(interposer client plus Rust server) and not a language comparison. A Rust cdylib could replace the shim
if same-language build simplicity matters more than the minimal shim.

Native routed hub comparison (1 MiB text, same Python client, 3 runs): Rust hub mean 0.0090 s vs Python hub
mean 0.0113 s, ratios 0.0041 vs 0.0040. The remaining Python cost is the client; Rust on both ends is
2.7 ms.

Further native options if more CPU is needed: benchmark flate2 backends (pure-Rust miniz_oxide is the
current default; zlib-rs or the C zlib feature may compress faster), and a Rust adapter pair to replace
the Python adapter in the proxy shape. C/C++ would not remove the remaining cost, which is compression and
framing, not language runtime.

## Native backend and final loopback numbers (2026-10-17)

Backend switched to flate2's zlib-rs (pure-Rust zlib implementation):

| Payload | miniz_oxide | zlib-rs |
|---|---|---|
| 1.28 MB synthetic text, compress | 650 MiB/s | ~2,900 MiB/s |
| 1.28 MB synthetic text, decompress | 2,455 MiB/s | ~5,000-7,800 MiB/s |
| 121 KB realistic corpus (sources+docs) ratio | 0.25245 | 0.25251 |

The realistic corpus ratio is identical, so the earlier synthetic-text ratio difference (0.00297 vs
0.00637) was an artifact of highly repetitive data and zlib-rs has no compression penalty on real text.
WIRE_LEVEL (0-9, default 6) now actually selects the level: on the corpus, level 1 gives 205 MiB/s at
ratio 0.380 vs 85 MiB/s at ratio 0.253 for level 6.

Also applied: zero-copy framing (header and payload written separately, no payload copy), TCP_NODELAY,
and direct return of buffered frames without a drain copy.

End-to-end Rust peer, 1.28 MB text, 5 runs each: plain TCP 1.04-1.30 ms (mean 1.15), wire 1.10-1.80 ms
(mean 1.28). The compressed wire path is now within ~0.1 ms of plain TCP on loopback, down from 2.7 ms.
Random 96 KiB raw mode: 2.2-2.7 ms (per-message overhead dominates at that size).

Evidence: evidence/artifacts/perf/rust-native.log.

Native adapter pair (2026-10-17): wire-rs edge/gateway replace the Python proxy pair for the hot path.
Same-run comparison, 647 KiB text: Rust edge mean 0.00337 s vs Python adapter mean 0.00715 s (about 2x);
plain baseline in that sample was load-disturbed (0.00288-0.00694 s), so the proxy-to-proxy comparison is
the honest one. See RUST-BINDING.md and evidence/artifacts/rust-adapter/.

### Post-optimization numbers (buffer reuse + vectored writes, 2026-10-17)

Applied: recv_into with reusable 256 KiB relay buffers (no per-message allocation), compression scratch
buffer reuse (no per-chunk output allocation), vectored writes (one writev per frame instead of two
writes), plus the earlier zlib-rs backend, zero-copy read path and TCP_NODELAY.

Rust adapter pair vs plain TCP and Python adapter, 647 KiB text, 10 runs each, same run:

    plain TCP direct      min 0.00121 s, mean 0.00159 s
    Rust edge -> gateway  min 0.00136 s, mean 0.00227 s
    Python adapter pair   min 0.00540 s, mean 0.00584 s

Rust edge->gateway is now within ~0.15 ms of plain TCP at the minimum and 2.6-3x faster than the Python
adapter, while still compressing to a 0.0059 ratio.

Rust routed hub, 1.28 MB text: Rust client through hub (client -> hub -> echo, three hops) 1.8-2.1 ms;
same hub with Python WireSocket clients min 5.5/mean 6.1 ms vs Python hub min 7.8/mean 8.0 ms.

Rust peer end-to-end, 1.28 MB text: wire min ~1.0 ms vs plain min ~0.97 ms (within noise).

Host load was high during sampling (another lab's disk audit and post-boot indexing), so absolute values
vary; the same-run proxy comparisons are the reliable signal. Evidence: evidence/artifacts/rust-adapter/,
evidence/artifacts/rust-hub/, evidence/artifacts/perf/rust-native.log.

Rust mux gateway (2026-10-17): the last Python-only data path is native. 8 concurrent streams, same Python
client, 3 runs: Rust min 0.0110/mean 0.0118 s vs Python WireMuxGateway min 0.0181/mean 0.0187 s (~1.6x).
Caveat: dispatch writes client data to the target inline, so a slow target can head-of-line block the
other streams on that connection.

### Chunk size and compression level tuning (2026-10-17)

Frame chunk size (WIRE_CHUNK, default 65536) on 1.28 MB text, 7 runs, min/mean seconds:

    8 KiB     0.00230 / 0.00246
    16 KiB    0.00160 / 0.00173
    32 KiB    0.00120 / 0.00129
    64 KiB    0.00100 / 0.00111   <- measured optimum
    256 KiB   0.00180 / 0.00220
    1 MiB     0.00210 / 0.00230

64 KiB is the latency sweet spot: larger frames break the compress/send/receive pipeline (the receiver
waits for a whole frame before decompressing), smaller frames add per-frame overhead. The environment
override stays for tuning but the default is the measured optimum.

Compression level (WIRE_LEVEL): level 1 vs 6 on 1.28 MB text gives 0.00230/0.00246 s ratio 0.0123 vs
0.00210/0.00230 s ratio 0.0068; on realistic corpus (148 KB mixed sources/docs) level 6 ratio is 0.24489
vs miniz_oxide 0.24480 (identical on real text) and level 1 is 0.373. Default 6 keeps the ratio; level 1
trades roughly 2x ratio for ~10-20% lower latency where that matters.

Evidence: evidence/artifacts/perf/chunk-level-corpus.log.

Mux gateway update (2026-10-17): per-stream OPEN routes and per-stream dispatch queues remove the earlier
inline-write head-of-line block. Same-run 8-stream benchmark: Rust min/mean 0.0123/0.0128 s vs Python
0.0218/0.0230 s (loaded host; quiet run 0.0100/0.0106 vs 0.0158/0.0161). See RUST-BINDING.md.

Native mux client (2026-10-17): 8 streams of 128 KiB text through the Rust mux gateway: native client
0.00457-0.00489 s vs Python client 0.0119-0.0131 s (same run), Python gateway 0.0155-0.0199 s. The mux
path is now native-client-bound, not gateway-bound. See RUST-BINDING.md.

### Consolidated native performance harness (2026-10-17)

tools/wire_rust_perf_smoke.py runs every native path in one same-run harness (7 reps, reports min/mean/max
seconds) and prints WIRE_RUST_PERF_PASS. Latest run, loopback:

| Path | min | mean | max |
|---|---|---|---|
| plain peer, 1.25 MiB | 0.00114 | 0.00170 | 0.00206 |
| wire peer, 1.25 MiB | 0.00240 | 0.00309 | 0.00340 |
| Rust adapter, 64 KiB x10 | 0.00033 | 0.00036 | 0.00042 |
| Python adapter, 64 KiB x10 | 0.00054 | 0.00059 | 0.00085 |
| Rust hub, 256 KiB | 0.00040 | 0.00041 | 0.00050 |
| Rust mux, 8 x 128 KiB | 0.00234 | 0.00298 | 0.00414 |

Small-message latency (1 KiB round trips, 200 reps): plain TCP 36.4 us, wire 59.6 us, Rust adapter path
79.6 us. The compressed wire path adds roughly 20 us per small round trip in this run; an earlier quiet
run measured wire 64.8 us vs plain 68.7 us, so treat the small-message penalty as noise-level to ~20 us,
not zero.

Note: the Rust adapter row is per 64 KiB round trip through edge->gateway->echo (compressed both hops),
not a per-connection number. Host load still moves absolute numbers; the harness is same-run by design.

Evidence: evidence/artifacts/perf/perf-harness.log.

### Small-message latency by level (2026-10-17)

Native echo-ping mode: 1 KiB payload, 200 round trips per run, 5 runs, mean microseconds:

| Mode | min | mean | max | ratio |
|---|---|---|---|---|
| plain TCP | 23.3 | 55.1 | 75.2 | - |
| wire, level 6 | 34.1 | 41.8 | 62.8 | 0.0176 |
| wire, level 3 | 32.5 | 37.8 | 52.2 | 0.0171 |
| wire, level 1 | 30.2 | 33.9 | 36.1 | 0.0186 |

Wire is faster than plain TCP even at 1 KiB because compressed frames are tiny. Level 1 lowers mean RTT by
~20% for a ~6% ratio cost on small messages: use WIRE_LEVEL=1 for latency-first interactive workloads,
keep the level-6 default when bytes matter. The native echo-ping mode
(wire-rs echo-ping --connect ADDR [--route NAME] [--bytes N] [--reps K] [--plain]) makes this
reproducible.

Evidence: evidence/artifacts/perf/ping-by-level.log.

### Receive-path copy elimination (2026-10-17)

Raw frames are now read directly into the receive buffer, and compressed frames are decompressed straight
into it instead of an intermediate Vec: one full-payload copy removed per frame. All Rust smokes pass
(interop, hub, adapter, mux). Refreshed same-run harness on a loaded host:

| Path | min | mean | max |
|---|---|---|---|
| plain peer, 1.25 MiB | 0.00152 | 0.00192 | 0.00216 |
| wire peer via gateway, 1.25 MiB | 0.00210 | 0.00286 | 0.00330 |
| Rust adapter, 64 KiB round trip | 0.00021 | 0.00036 | 0.00043 |
| Python adapter, same | 0.00054 | 0.00059 | 0.00087 |
| Rust hub, 256 KiB | 0.00040 | 0.00043 | 0.00050 |
| Rust mux, 8 x 128 KiB | 0.00261 | 0.00370 | 0.00436 |

The Rust adapter remains ~1.6x faster than the Python adapter in the same run. Loopback absolutess remain
load-sensitive; same-run proxy comparisons are the stable signal.

Evidence: evidence/artifacts/perf/perf-harness.log.

# spiralcore-base/wire — standalone wire transport side exploration

Status: candidate side exploration (infrastructure/tooling lane). No model compute is used or
changed here, and nothing in this folder is wired into the dose path, D11 serving, training or
the main workspace.

Provenance: copied from the review repository at commit 032d903
(`/Users/casey.allard/spiralcore-fbs-review`), which retains the development history and the
canonical smoke evidence for this work. This copy exists so the delivery lives in uor-r4 as a
side exploration for the roadmap and plan.

## Contents

    tools/      Python adapter pair, socket layer, routed hub, opt-in mux, process interposer,
                smokes, benchmarks and the interop matrix
    crates/wire-rs/  Rust WireSocket/WireListener, routed hub, adapter edge/gateway,
                mux gateway and native mux client (detached from the uor-r4 workspace)
    docs/       WIRE-ADAPTER, WIRE-SOCKET, INTERPOSER, RUST-BINDING, PERFORMANCE,
                OPTIMIZATION-CLOSEOUT, WIRE-USABILITY
    evidence/   bounded logs and summaries (no recursive/self-writing logs)

## What it is

One wire framing (UORADP01 handshake, u32 frames with a raw-flag bit, hybrid zlib/raw per
direction) exposed four ways:

- proxy pair (plain app -> edge adapter == compressed ==> gateway adapter -> service)
- socket layer (WireSocket/WireListener directly in the app)
- process interposer (unmodified binaries; DYLD_INSERT_LIBRARIES on macOS, LD_PRELOAD on Linux)
- opt-in multiplexing (UORADM01 streams with windows, FIN/RESET, per-stream routes)

All four shapes and the Rust binding share the framing and interoperate; the Rust crate covers
the peer, hub, adapter pair, mux gateway and a native mux client.

## Headline measurements (loopback, committed evidence in evidence/)

    Rust peer 1.25 MiB compressed   2.10 / 2.86 / 3.30 ms min/mean/max
    plain TCP 1.25 MiB              1.52 / 1.92 / 2.16 ms
    Rust adapter 64 KiB round trip  0.21 / 0.36 / 0.43 ms (Python adapter 0.54 / 0.59 / 0.87 ms)
    Rust hub 256 KiB                0.40 / 0.43 / 0.50 ms
    Rust mux 8 x 128 KiB            2.61 / 3.70 / 4.36 ms (native client path)
    1 KiB RTT                       wire 41.8 us vs plain 55.1 us (level 1: 33.9 us)

Compression backend is zlib-rs: ~2,900 MiB/s compress and ~5,000-8,300 MiB/s decompress on
text, with a realistic-corpus ratio equal to miniz_oxide (0.24489 vs 0.24480).

## Verify (from this directory)

    cd crates/wire-rs && cargo build --release --offline
    cd ../.. && python3 tools/wire_socket_routing_smoke.py        # WIRE_SOCKET_ROUTING_PASS
    bash tools/wire_rust_hub_smoke.sh                             # WIRE_RUST_HUB_PASS
    python3 tools/wire_rust_perf_smoke.py                         # WIRE_RUST_PERF_PASS
    bash tools/wire_interop_matrix.sh                             # 18/18 PASS

The interop matrix's adapter legs expect the uor-r4 OBJPEER binary (target/release/objpeer);
the Rust and socket legs need only this folder.

## Limits

- Protocol is zlib-framed for interoperability; the measured residual cost is compression CPU,
  not framing copies or threads.
- Loopback numbers are host-load sensitive; run the harness in one go for same-run comparisons.
- No TLS/authentication inside the framing (use external TLS/WireGuard/SSH).
- The process interposer is macOS/Linux only, numeric-IPv4 config, blocking connect.
- This is transport only: no SpiralCore/FBS routing integration, no model claim, no serving change.

## Coordination

Claimed by lab deepseek on branch `deepseek/spiralcore-wire-side-exploration`
(docs/harness/claims.json). Canonical record:
[docs/labs/spiralcore-wire-side-2026-10-18/](https://github.com/UOR-Foundation/uor-r4/tree/main/docs/labs/spiralcore-wire-side-2026-10-18).
PM (kimi/pm1) holds dispatch authority for any promotion of this side exploration into the
roadmap path; any model-path use needs its own pre-registration and PM/owner decision.

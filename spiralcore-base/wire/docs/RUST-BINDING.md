# Rust binding for the wire framing

`crates/wire-rs` is a dependency-light Rust implementation of the same framing used by the Python adapter,
WireSocket and the interposer (UORADP01 handshake, u32 frames with bit31 raw flag, hybrid zlib/raw mode
per direction). Wire format is unchanged; it interoperates with all Python implementations and the
adapter gateway.

    use wire_rs::WireSocket;

    let mut socket = WireSocket::connect("gateway:7042", "service")?;
    socket.send_all(b"request")?;
    socket.shutdown()?;
    let response = socket.recv_all()?;
    println!("{:?}", socket.stats());   // plain/wire counters + ratio

    let listener = wire_rs::WireListener::bind("0.0.0.0:7042")?;
    let (mut incoming, route) = listener.accept()?;

## Build

    cd crates/wire-rs && cargo build --release --offline

Uses the cached flate2/miniz_oxide crates (no new downloads); the CLI binary is `target/release/wire-rs`
with `echo-server --bind ADDR` and `echo-client --connect ADDR [--route NAME] [--bytes N]
[--pattern text|random]` modes used by the interop smoke.

## Measured (tools/wire_rust_interop_smoke.sh -> WIRE_RUST_INTEROP_PASS)

| Direction | Payload | Ratio |
|---|---|---|
| Rust client -> Python WireListener | 512 KiB text | 0.0034 |
| Python WireSocket -> Rust server | 512 KiB text | 0.0042 |
| Rust client -> Python WireListener | 96 KiB random | 1.0001 |
| Python WireSocket -> Rust server | 96 KiB random | 1.0001 |
| Rust client -> adapter gateway | 512 KiB text | 0.0034 |

Exact byte equality is asserted in each direction. Evidence: evidence/artifacts/rust-interop/run.log.

## Limits

- Base framing only: the Rust binding does not implement the UORADM01 mux layer or a routed WireGateway
  (the Python implementations cover those); it can talk to a Python mux gateway only for a single stream
  handshake, not mux.
- `WireListener::accept` serves one wire connection per call; callers write their own accept loop.
- Compression level is fixed at 6; no idle timeout/metrics daemon yet (stats are per-socket).
- Build depends on the cached flate2/miniz_oxide crates; a fresh machine needs normal crates.io access.
- No TLS/auth in the framing (same guidance as the rest of the track).

## Routed hub (native)

wire-rs includes a native routed hub, the production hot path equivalent of tools/wire_hub.py:

    wire-rs hub --listen 0.0.0.0:7042 --route model=127.0.0.1:7049 --route store=127.0.0.1:7050

The handshake route selects the target; unknown routes fail closed; each connection is split into
independent read/write halves (cloned TcpStream plus separate frame/decompression state), so the two relay
directions never share mutable socket state and blocking reads cannot stall writes.

Measured (tools/wire_rust_hub_smoke.sh -> WIRE_RUST_HUB_PASS), 1 MiB text through route echo with the same
Python client on both hubs, 3 runs:

    Rust hub    min 0.0069 s, mean 0.0090 s, ratio 0.0041
    Python hub  min 0.0090 s, mean 0.0113 s, ratio 0.0040

The Rust hub also served an unmodified stock binary through the C interposer and failed closed on an
unknown route. This is how to run the wire proxy natively; C or C++ is not required (see PERFORMANCE.md).

## Backend, level and benchmark modes

- Default backend is flate2 with zlib-rs (pure Rust); build with
  --no-default-features --features backend-miniz to compare against miniz_oxide.
- WIRE_LEVEL (0-9, default 6) selects the compression level for the wire path.
- wire-rs bench-compress [--bytes N|--file PATH] [--pattern text|random] [--reps K] [--level L]
  reports compress/decompress MiB/s and ratio for the selected backend.
- wire-rs echo-server --plain and echo-client --plain give a same-language plain-TCP baseline for
  measuring wire overhead honestly.

## Rust adapter pair (native proxy shape)

    wire-rs edge --listen 0.0.0.0:7041 --remote gateway:7042 [--route NAME]
    wire-rs gateway --listen 0.0.0.0:7042 --target 127.0.0.1:7049

edge accepts plain TCP from the application, speaks the wire protocol to gateway, and gateway speaks
plain TCP to the service. Both use split read/write halves with independent compression state, so each
direction has its own blocking thread and no shared mutable socket.

Measured same run (tools/wire_rust_adapter_smoke.sh -> WIRE_RUST_ADAPTER_PASS), 647 KiB text, 5 runs,
noisy loaded host:

    plain TCP direct        min 0.00288 s, mean 0.00694 s
    Rust edge -> gateway    min 0.00317 s, mean 0.00337 s
    Python adapter pair     min 0.00623 s, mean 0.00715 s

The Rust proxy is about 2x faster than the Python adapter pair in the same run; the plain baseline was
disturbed by host load, so compare the two proxies rather than plain in that sample. The smoke also
serves a Python WireSocket client through gateway and a stock binary through the interposer, exact bytes.

### Hot-path implementation notes

- recv_into(out: &mut [u8]) lets relays reuse one 256 KiB buffer instead of allocating a Vec per message.
- Compression writes into a reusable scratch buffer inside WireWriter/WireSocket (no per-chunk Vec).
- Frames are written with write_vectored (header + payload in one syscall, partial-write safe).
- WIRE_LEVEL selects compression level; bench-compress and --plain modes give honest baselines.

### Rust mux gateway (UORADM01)

    wire-rs mux-gateway --listen 0.0.0.0:7043 --route echo=127.0.0.1:7049

The mux gateway accepts UORADM01 connections, opens a target per logical stream, keeps per-stream
decompression/compression state, honors 64 KiB WINDOW credit in both directions, and turns client FIN into
target half-close. Unknown handshake routes reply RESET per opened stream (fail closed).

Measured (tools/wire_rust_mux_gateway_smoke.sh -> WIRE_RUST_MUX_PASS): 8 concurrent streams mixing 128 KiB
text and 96 KiB random round-tripped exactly. Same-run benchmark, same Python client on both, 3 runs:

    Rust mux gateway    min 0.0110 s, mean 0.0118 s
    Python WireMuxGateway min 0.0181 s, mean 0.0187 s

About 1.6x faster natively. Limits: one handshake route per connection (all streams share the target),
client-opened streams only, fixed 64 KiB window, and the dispatch loop writes client data to the target
inline (a slow target can head-of-line block that connection's other streams).

### Tuning knobs measured

- WIRE_CHUNK (default 65536): 64 KiB measured fastest for 1.28 MB text (1.00/1.11 ms); 8 KiB 2.30/2.46 ms,
  256 KiB 1.80/2.20 ms, 1 MiB 2.10/2.30 ms. Bigger is not better because frame-level decompression
  serializes the pipeline.
- WIRE_LEVEL (default 6): level 1 lowers latency ~10-20% at roughly 2x ratio on text; level 6 keeps the
  ratio (0.24489 realistic corpus, matching miniz_oxide's 0.24480). Choose per deployment.

#### Per-stream routes and dispatch queues (2026-10-17)

OPEN frames now carry an optional route name: one mux connection can open streams to different named
targets (empty OPEN route falls back to the handshake target; unknown per-stream route gets RESET).
Client-to-target data is queued to a per-stream dispatch thread instead of being written inline before
the next frame is read, so a slow target no longer head-of-line blocks the other streams on that
connection. Credit waiters are cancelled when the connection closes.

Measured same-run, 8 streams mixing 128 KiB text and 96 KiB random, same Python client on both, 3 runs
(host load was high; earlier quiet run in parentheses):

    Rust mux gateway     min 0.0123 s, mean 0.0128 s   (quiet: 0.0100 / 0.0106)
    Python WireMuxGateway min 0.0218 s, mean 0.0230 s  (quiet: 0.0158 / 0.0161)

About 1.8x faster natively. The smoke now also verifies model/store per-stream routes return the correct
labels and exact payloads, and that an unknown stream route fails closed.

#### Native mux client (2026-10-17)

    wire-rs mux-client --connect ADDR --route NAME --streams N --bytes N --pattern text|random

The native client opens logical streams over one UORADM01 connection, compresses per stream with window
credit, and verifies the echoed bytes. Measured through the Rust mux gateway, 8 concurrent streams of
128 KiB text (1 MiB total), 3 runs:

    Rust mux client -> Rust mux gateway       0.00457-0.00489 s
    Python client -> Rust mux gateway         0.0119-0.0131 s (min/mean)
    Python client -> Python WireMuxGateway    0.0155-0.0199 s (min/mean)

The earlier 10-13 ms mux figures were Python-client-bound; the native mux path is ~2.5x faster for the same
8-stream workload and confirms the gateway itself is not the bottleneck.

- echo-ping mode: wire-rs echo-ping --connect ADDR [--route NAME] [--bytes N] [--reps K] [--plain]
  measures persistent small-message round-trip latency (mean microseconds) with wire or plain framing.

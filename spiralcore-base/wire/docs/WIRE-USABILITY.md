# Wire compression usability: final summary

Goal: make UOR wire compression usable in front of any TCP/IP application. Four deployment shapes share
one wire framing (UORADP01 handshake, u32 frames with a raw-flag bit, hybrid zlib/raw per direction):

    proxy pair   app --plain--> edge adapter ==compressed==> gateway adapter --plain--> service
    socket layer app uses WireSocket/WireListener directly (no proxy process)
    interposer   unmodified binary: DYLD_INSERT_LIBRARIES (macOS) / LD_PRELOAD (Linux) redirects configured
                 destinations to a wire gateway before bytes enter TCP/IP
    mux          UORADM01: many logical streams over one framed connection with windows and FIN/RESET

## Delivered

| Deliverable | Reproducible smoke | PASS marker |
|---|---|---|
| Process interposer (macOS) | tools/wire_interposer_smoke.sh | WIRE_INTERPOSER_PASS |
| Process interposer (Linux) | tools/wire_interposer_linux_smoke.sh in mvm microVM | WIRE_INTERPOSER_LINUX_PASS |
| Routed listener / hub | tools/wire_socket_routing_smoke.py | WIRE_SOCKET_ROUTING_PASS |
| Hub daemon + interposer | tools/wire_hub_cli_smoke.sh | WIRE_HUB_CLI_PASS |
| Multiplexing | tools/wire_mux_smoke.py | WIRE_MUX_PASS |
| TLS socket layer | tools/wire_socket_tls_smoke.py | WIRE_SOCKET_TLS_PASS |
| Mux per-stream routing | tools/wire_mux_routing_smoke.py | WIRE_MUX_ROUTING_PASS |
| Rust binding | tools/wire_rust_interop_smoke.sh | WIRE_RUST_INTEROP_PASS |
| Full interop matrix | tools/wire_interop_matrix.sh | WIRE_INTEROP_MATRIX_PASS (15/15) |
| Overhead benchmark | tools/wire_overhead_bench.py | WIRE_OVERHEAD_BENCH_PASS |

## Measurements

Ratios (wire/plain):

    socket layer / adapter   text 0.0043, random 1.0001, objpeer 0.6797
    routed socket hub        text 0.0042 (model/store), unknown route fails closed
    process interposer       macOS text 0.0031, Linux VM 0.0032, random 1.0001, unmodified Python 0.0042/0.0043
    Rust binding             Rust->Python 0.0034, Python->Rust 0.0042, random 1.0001, Rust->gateway 0.0034
    mux (8 streams, per-stream routes supported; TLS not yet)          best wire 811239 B vs separate 807852 B (+0.4% framing), 71 window updates

Loopback overhead, 647 KiB payload, 3 runs (min/mean/max seconds; ratio):

    plain                  0.0012 / 0.0016 / 0.0034   ratio 1.0
    proxy pair             0.0034 / 0.0047 / 0.0070   ratio 1.0 (plaintext edge, wire not counted here)
    socket layer           0.0044 / 0.0060 / 0.0086   ratio 0.0041
    socket layer + TLS     n/a (smoke ratios only)     ratio 0.0042
    mux single stream      0.0075 / 0.0077 / 0.0078   ratio 0.0048
    process interposer     0.0148 / 0.0466 / 0.1010   ratio 0.0031  (includes probe process startup; high variance)

See PERFORMANCE.md for the CPU breakdown and why loopback is not the value metric.

This is the honest cost side: every mode adds CPU/framing/latency on loopback; the benefit is fewer bytes
on real links (the earlier WAN measurements: adapter tunnel ratio 0.904 on protocol traffic, text up to
232x; protocol batch opt-in 2.58x WAN; cold-join 1:2.73). No exponential speedup or compression-carrier
claim is made.

## Interop matrix (15/15 PASS)

socket layer, routing, mux, mux per-stream routing, macOS interposer, Linux interposer (mvm), hub CLI, Rust interop, adapter
pair/hub/policy/metrics/idle/protocol. Evidence: evidence/artifacts/usability/matrix-run.log.

## Regression

tools/verify_all.sh -> VERIFY_ALL_PASS on uor-r4 main b32a54882 (58 tests, Clippy, fmt, fleet demo, Python
port, cold join, portable agent); tools/check_evidence.py -> EVIDENCE_OK. Owner checkout clean; 0 pods.

## Residual limits (documented, not hidden)

- No TLS/authentication inside the framing: run TLS/WireGuard/SSH externally; in-adapter TLS was
  prototyped and rejected (Python SSLSocket concurrency/half-close semantics).
- Interposer: numeric IPv4 destinations only, blocking connect only, macOS interposes send/recv (not
  read/write), SIP strips DYLD for protected system binaries, fd table capped at 1024.
- Mux: client-opened streams only, one route/target per mux connection, fixed 64 KiB window, no priority.
- Rust binding: base framing only (no mux/routed hub), fixed compression level 6.
- Container image remains an unbuilt recipe (no docker/podman/colima/lima/Apple container on this host);
  mvm microVMs run OCI images but do not build Dockerfiles.
- Historical one-off WAN timings are summary-only where noted; adapter/fleet numbers keep their earlier
  caveats (protocol is replication/verification, not a compressor).

## Reproduce

    bash tools/wire_interop_matrix.sh                # 13/13
    python3 tools/wire_overhead_bench.py             # overhead table
    bash tools/wire_interposer_smoke.sh              # macOS
    # Linux interposer: run wire_interposer_linux_smoke.sh in a Linux VM / mvm microVM
    UOR_R4=~/uor-r4 bash tools/verify_all.sh         # VERIFY_ALL_PASS
    UOR_R4=~/uor-r4 python3 tools/check_evidence.py  # EVIDENCE_OK

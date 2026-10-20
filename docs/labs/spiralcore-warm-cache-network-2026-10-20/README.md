# SpiralCore warm-cache network — side exploration

Date: 2026-10-20. Lab: deepseek. Scope: infrastructure/tooling side exploration.
No model compute, training, serving or dose-path change.

## Delivered

[projects/spiralcore/warm-cache](../../../projects/spiralcore/warm-cache/README.md)
contains:

- F F F F four-oscillator Kuramoto clock with a 24-byte forward time header;
- 4 KiB page Merkle tree, page proofs, 64-page group manifest and group proofs;
- content vs temporal roots and WarmWire WCF1 frame;
- page, multi-page and group repair (WGR1);
- bounded replay ring and calibration;
- WMX1 priority multiplexer;
- WarmSession library API and warm_send/warm_recv CLI;
- TCP loopback, QUIC loopback, QUIC WAN harness and emulated-WAN tests;
- one cold seed followed by repeated warm frames.

## Results

- 31 local tests pass (30 library/unit + 1 wire-warm composition in the local
  workspace; the committed crate has 30 without the external dev-dependency).
- Warm frame 124 B; WarmWire identity 134 B; one-page repair 4,228 B frame /
  4,238 B wire.
- Size sweep 4 KiB to 16 MiB: warm identity is 124 B; one-page repair is
  4,228 B.
- TCP loopback: 138 B identity including the 4 B length prefix.
- QUIC loopback: 134 B identity; 4 KiB to 4 MiB.
- Real remote WAN over TCP/SSH: identity 134 B at ~209-217 ms; one-page repair
  4,238 B at ~206-263 ms for 4 KiB to 4 MiB; all output hashes exact. The pod
  exposed only SSH/22, so direct QUIC/UDP was blocked. The pod was released and
  deleted.
- Emulated WAN at 60 ms one-way: warm identity ~63-122 ms; repair ~62-118 ms.

Evidence lives in
[results](../../../projects/spiralcore/warm-cache/rust/results/) and is described in
[PROTOTYPE_RESULTS.md](../../../projects/spiralcore/warm-cache/PROTOTYPE_RESULTS.md).

## Verify

    cd projects/spiralcore/warm-cache/rust/warm-cache-proto
    CARGO_TARGET_DIR=../../local-target cargo test --offline

QUIC binaries load test certificates from WARM_CERT_DIR; generate them with
certs/gen_test_certs.sh. No test private key is committed.

## Limits

- Not wired into the production wire-warm frame grammar; composition with the
  existing RAW frame is tested only in the local workspace.
- Direct QUIC WAN requires a UDP-reachable endpoint.
- The transparent sidecar proxy is packaging; the library and CLI are present.
- Loopback/emulated results are not WAN measurements.

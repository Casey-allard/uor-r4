# SpiralCore warm-cache network — side exploration

Status: local prototype complete, TCP WAN measured on a Vast pod, QUIC
loopback/emulated-WAN measured; QUIC direct WAN blocked by pod port policy.
This folder is infrastructure/tooling only. It does not change the model path,
D11 serving, training, or the dose path.

## What is here

- F F F F four-oscillator Kuramoto clock with 24-byte forward time header.
- 4 KiB page Merkle tree, page proofs, 64-page group manifest and group proofs.
- content root vs temporal root vs witness; WarmWire WCF1 frame.
- page repair, multi-page repair, group-repair frame WGR1.
- bounded replay ring and calibration module.
- WMX1 application multiplexer with CONTROL/WARM/REPAIR/COLD priority.
- WarmSession library API and warm_send/warm_recv CLI.
- TCP loopback, QUIC loopback, QUIC WAN harness and emulated-WAN test.
- remote TCP/SSH WAN campaign results: identity warm frame 134 B at
  ~209-217 ms; one-page repair 4,238 B at ~206-263 ms for 4 KiB to 4 MiB.

## Build and test

    cd rust/warm-cache-proto
    CARGO_TARGET_DIR=../../local-target cargo test --offline
    CARGO_TARGET_DIR=../../local-target cargo build --offline --release

QUIC binaries load test certificates from WARM_CERT_DIR (default ./certs).
Test certificates are not committed; generate them with
certs/gen_test_certs.sh (test-only, not production credentials).

## Reports

- [PROTOTYPE_RESULTS.md](PROTOTYPE_RESULTS.md) — tests, size sweep, loopback,
  QUIC and remote WAN results.
- [NETWORKING.md](NETWORKING.md) — transport, multiplexer, library/proxy/kernel
  design and usability recommendations.
- [DESIGN.md](DESIGN.md) — unified warm-cache network design.
- [KURAMOTO_TIME.md](KURAMOTO_TIME.md) — hierarchical Kuramoto time, witness and
  Merkle-root design and negative/positive results.

## Limits

- Not merged into the wire-warm production frame grammar; composition with the
  existing RAW frame is tested locally.
- QUIC direct WAN was blocked because the Vast pod exposed only port 22; the
  real WAN campaign used TCP over an SSH tunnel.
- The transparent sidecar proxy is a packaging layer; the library and CLI are
  present.
- Loopback/emulated results are not WAN measurements.

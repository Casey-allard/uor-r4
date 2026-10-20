# Warm-cache prototype — final build and test result

Date: 2026-10-19. Status: F F F F forward-only prototype with page-group
manifest, bounded replay, versioned wire frame, calibration, and multi-size
TCP loopback. Production code was not modified.

## What was built

- four first-order Kuramoto oscillators, RK4, order parameter R and mean psi;
- forward-only F F F F time header: level, pass, epoch, phase_q, coherence_q,
  parent_epoch, phase_bits — 24 bytes;
- forward phase recording and a bounded replay ring (capacity-bounded, latest
  window retained);
- 4 KiB page Merkle tree: page leaves, binary levels, root = seed id/content
  root, per-page proofs, page diff;
- page-group manifest: 64 pages per group, group roots, manifest root and
  encode/decode;
- content root C (time-free) and temporal root W (time-stamped), 32-byte
  witness; paired pass witness for diagnostics;
- Seed, Plan (identity, byte patch, page repair), WarmFrame, receive
  classification;
- versioned WarmWire frame (WCF1, version 1): seed id/version, plan kind,
  time header, content root, witness, payload;
- calibration module: derives B_l, R_min and W from phase spread, Bell margin
  and drift;
- multi-size TCP loopback binary.

Dependencies: sha2 only. No GPU, no model compute, no external network.

## Tests

Command:

    cd .../rust/warm-cache-proto
    CARGO_TARGET_DIR=../target cargo test --offline

Result: **31 passed, 0 failed**.

Coverage includes:
- four oscillators synchronize (R > 0.95), epoch advances, header is 24 B;
- replay and ring replay exact; ring bounded;
- page proofs verify and tampering fails;
- one-page and multi-page repair commit;
- content root stays equal across a clock change; temporal root differs;
- paired pass witness separates passes;
- identical warm transfer zero payload;
- patch and page-repair plans round-trip through the versioned wire frame;
- stale clock is ClockMismatch, not content corruption;
- seed/content corruption detected;
- warm-frame size constant across seed sizes;
- manifest round-trip and group count;
- calibration derives plausible B_l/R_min/W;
- one cold seed is much larger than warm frames.

Raw output: rust/results/cargo_test.txt.

## Size sweep

Output: rust/results/size_sweep.txt.

| seed size | cold bytes | pages | manifest bytes | warm frame | warm payload | repair frame | repair payload |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 4 KiB | 4,132 | 1 | 121 | 124 | 0 | 4,228 | 4,104 |
| 64 KiB | 65,572 | 16 | 601 | 124 | 0 | 4,228 | 4,104 |
| 256 KiB | 262,180 | 64 | 2,137 | 124 | 0 | 4,228 | 4,104 |
| 1 MiB | 1,048,612 | 256 | 8,377 | 124 | 0 | 4,228 | 4,104 |
| 4 MiB | 4,194,340 | 1,024 | 33,337 | 124 | 0 | 4,228 | 4,104 |
| 16 MiB | 16,777,252 | 4,096 | 133,177 | 124 | 0 | 4,228 | 4,104 |

At 16 MiB: identical warm frame is 124 B, about 0.00074% of cold; one-page repair is 4,228 B, about 0.025% of cold. The page-group manifest is 133,177 B for 16 MiB; it is optional because the receiver can derive page roots from a full cold seed.

## Versioned wire frame

WarmWire adds local framing on top of the 124 B warm frame:

    magic WCF1 (4) + version (1) + seed id (32) + seed version (4)
    + plan kind (1) + time header (24) + content (32) + witness (32)
    + payload length (4) + payload

Identity wire length: **134 B**. Patch/page-repair payload is appended.

## Multi-size TCP loopback

Output: rust/results/loopback.txt. Local 127.0.0.1, one connection, cold seed
then identity warm then one-page repair; wire bytes include the 4 B TCP length
prefix on each message.

| seed size | cold wire bytes | warm wire bytes | warm payload | repair wire bytes | repair payload | elapsed ms |
|---:|---:|---:|---:|---:|---:|---:|
| 4 KiB | 4,100 | 138 | 0 | 4,242 | 4,104 | 0.97 |
| 64 KiB | 65,540 | 138 | 0 | 4,242 | 4,104 | 13.43 |
| 1 MiB | 1,048,580 | 138 | 0 | 4,242 | 4,104 | 217.29 |
| 4 MiB | 4,194,308 | 138 | 0 | 4,242 | 4,104 | 759.21 |

The elapsed time includes the cold seed send plus two warm operations; it is a
loopback sanity check, not a WAN measurement.

## What the gaps now cover

- Page-group manifest: implemented and size-accounted.
- Multi-page repair: implemented and tested; only changed pages move.
- Bounded replay: implemented as a capacity-bounded ring; exact for retained
  frames.
- Versioned wire frame: implemented with magic/version, plan kind, time header,
  roots and payload; tested round-trips.
- Calibration: implemented as a derivation module (B_l, R_min, W from spread,
  Bell margin, drift); not yet automatically wired into the cell.
- Multi-size loopback: implemented for 4 KiB to 4 MiB.

## Limits

- TCP loopback only; no WAN or Ethernet.
- Merkle is one binary tree over 4 KiB pages; page-group manifest exists, but
  group-level repair proof is not yet used to localize a group before pages.
- The cold seed is sent whole; the manifest is optional and measured separately.
- Replay is optional diagnostics; the production path is F F F F.
- Calibration is a module, not yet controlling TimeHeader/cell phase bits.
- Not merged into the wire-warm crate.

## Next steps

1. Group-level repair proof and page-group repair frame.
2. Automatic calibration of TimeHeader from measured spread/drift.
3. Integrate WarmWire into the wire-warm crate as a versioned warm-plan frame.
4. Run the same sizes through the existing WAN harness.
5. Compare zero-payload and page-repair costs against the 1 MiB baseline.

Files:
- source: rust/warm-cache-proto/src/lib.rs
- modules: manifest.rs, replay.rs, wire.rs, calibration.rs
- demos: main.rs, bin/size_sweep.rs, bin/loopback.rs
- tests: rust/results/cargo_test.txt
- size sweep: rust/results/size_sweep.txt
- loopback: rust/results/loopback.txt
- demo: rust/results/demo.txt

## Gap-fill addendum (2026-10-19)

- Tests: **31 passed, 0 failed** (up from 24).
- Page-group proofs: manifest now builds a binary tree over group roots and
  supports group proof verification; tampered group roots fail.
- Group-repair wire frame (WGR1): seed id/version, group id, group root, group
  proof, time header, content root, witness and page list; decode and
  verification tested.
- Multiplexer (WMX1): stream id, kind, priority and payload over a byte stream;
  CONTROL preempts COLD; frame round-trip and priority test pass.
- Calibration: from-cell helper and calibrated TimeHeader constructor; derived
  phase bits are tested.
- Transport/usability notes: NETWORKING.md.

Coverage now includes group proofs, group-repair framing, multiplexer
priority, and calibrated headers in addition to the FFFF time cell, Merkle
pages, page repair, versioned WarmWire, multi-size sweep and TCP loopback.


## QUIC adapter (implemented)

The prototype now has a real QUIC loopback using quinn 0.11, rustls 0.23 and
tokio, with a CA-signed localhost certificate. Each message is a QUIC
bidirectional stream carrying a kind byte (cold/warm) plus the seed or WarmWire
bytes. The server verifies the cold seed and applies the warm/page-repair
frame with receive_warm.

Output: rust/results/quic_loopback.txt.

| seed size | cold bytes | warm bytes | warm payload | repair bytes | repair payload | elapsed ms |
|---:|---:|---:|---:|---:|---:|---:|
| 4 KiB | 4,096 | 134 | 0 | 4,238 | 4,104 | 1.77 |
| 64 KiB | 65,536 | 134 | 0 | 4,238 | 4,104 | 13.46 |
| 1 MiB | 1,048,576 | 134 | 0 | 4,238 | 4,104 | 184.82 |
| 4 MiB | 4,194,304 | 134 | 0 | 4,238 | 4,104 | 750.43 |

QUIC removes the 4-byte TCP length prefix from the wire count; identity warm
wire is 134 B, one-page repair 4,238 B.

This is still a localhost test. It proves the transport adapter works; it is
not a WAN measurement.


## Library and CLI (implemented)

- WarmSession API: holds seed, time header and stats; encodes plans, decodes
  WarmWire, classifies results and records bytes/payload/repair counters.
- CLI: warm_send and warm_recv over TCP. warm_send sends the cold seed, then a
  WarmWire identity/page-repair plan; warm_recv verifies and writes the
  reconstructed payload.
- CLI end-to-end test (1 MiB seed, one page changed): sent warm_bytes=4,238,
  payload_bytes=4,104, repairs=1; receiver OK; output byte-identical to the
  payload.

This is the library-first path; a transparent sidecar proxy remains a
packaging option on top of the same session API.


## Emulated WAN (QUIC, one-way delay 60 ms)

Server sleeps 60 ms before each ack; this approximates a 120 ms RTT path. It
is a latency emulation, not a real WAN measurement.

Output: rust/results/quic_wan_emulated.txt.

| size | cold ms | warm ms | repair ms | warm bytes | repair bytes |
|---:|---:|---:|---:|---:|---:|
| 4 KiB | 63.1 | 63.1 | 62.4 | 134 | 4,238 |
| 64 KiB | 66.6 | 65.3 | 74.9 | 134 | 4,238 |
| 1 MiB | 159.5 | 122.1 | 118.1 | 134 | 4,238 |

Warm identity is one RTT plus 134 B; the 1 MiB cold seed is the expensive path.
The emulation is useful for latency-budget work; a real WAN run still needs a
remote endpoint.


## Wire-warm composition test (added)

A test now carries a WarmWire inside the existing wire-warm RAW frame:

- build WarmWire from seed/time header/identity plan;
- encode it into a wire-warm RAW frame with the existing Peer API;
- decode the RAW payload and then decode WarmWire again;
- seed id and time header remain intact.

This is the local integration check for the wire-warm carrier. It does not yet
switch wire-warm's default warm body to WarmWire; that remains a reviewed
protocol change.

Total tests: **31 passed, 0 failed** (30 library/unit + 1 wire-warm composition).


## QUIC WAN harness (ready)

quic_wan has server/client roles:

    quic_wan server <bind>
    quic_wan client <addr> [sizes...]

It reuses the CA-signed localhost certificate, QUIC streams and WarmWire
frames. Local verification (127.0.0.1) gives:

| size | cold ms | warm ms | repair ms | warm bytes | repair bytes |
|---:|---:|---:|---:|---:|---:|
| 4 KiB | 0.7 | 0.5 | 0.6 | 134 | 4,238 |
| 64 KiB | 6.1 | 4.0 | 4.3 | 134 | 4,238 |
| 1 MiB | 80.8 | 58.1 | 57.7 | 134 | 4,238 |

This is a localhost validation of the remote harness, not a WAN measurement.
Run the server on one host and the client on another to obtain the real WAN
numbers; the certificate must be valid for the server name used by the client.


## Real remote WAN test (completed)

Pod: vast-56894372 at 180.189.55.43 (Vast.ai, 1x RTX 5090, bootstrap OK),
leased by deepseek/warm-wan-20261020 for the WAN run, released and deleted
afterward (0 running pods).

Transport: TCP warm protocol over an SSH -L tunnel. Vast exposes only port 22,
so direct QUIC/UDP was blocked; the QUIC loopback and emulated-WAN tests remain
the QUIC evidence. This is a real WAN path with SSH encryption/forwarding
overhead on top.

Method: one cold seed, then repeated warm frames on the same TCP connection;
server verifies and writes the reconstructed payload; SHA-256 compared on both
sides. Warm frame is 134 B identity or 4,238 B one-page repair.

| seed size | identity warm ms | repair warm ms | wire bytes | output |
|---:|---:|---:|---:|---|
| 4 KiB | 208.8-211.5 | 206.1-265.7 | 134 / 4,238 | exact |
| 64 KiB | — | 207.9-209.5 | 4,238 | exact |
| 1 MiB | 211.3-215.7 | 209.2-221.6 | 134 / 4,238 | exact |
| 4 MiB | 210.6-217.3 | 214.6-262.8 | 134 / 4,238 | exact |

Total client wall time including cold-seed upload:

- 4 KiB identity: 1,178 ms (seed + 3 warm); 4 KiB repair: 1,641 ms (seed + 5 warm)
- 64 KiB repair: 1,079 ms (seed + 3 warm)
- 1 MiB identity: 1,433 ms; 1 MiB repair: 1,684 ms
- 4 MiB identity: 2,824 ms; 4 MiB repair: 2,517 ms

All output hashes matched. Raw data: rust/results/wan_tcp_remote.txt and
rust/results/wan-server.log.

QUIC direct to the pod timed out because only SSH/22 is exposed. A true QUIC
WAN run needs a UDP-reachable endpoint or a pod with port 39412/UDP exposed.

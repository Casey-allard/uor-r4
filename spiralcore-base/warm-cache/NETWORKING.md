# Warm-cache network: transport, multiplexing and usability

Date: 2026-10-19. Companion to DESIGN.md, KURAMOTO_TIME.md and
PROTOTYPE_RESULTS.md.

## 1. Where the prototype sits now

The prototype is an application-layer protocol:

- WarmWire is a byte frame carried inside an ordinary TCP stream in the loopback
  test;
- the test adds a 4-byte length prefix per message;
- it does not modify TCP/IP, IP packets, or the kernel;
- it is not a separate process: it is a Rust library/prototype API;
- the wire is forward-only F F F F, 124 B warm frame, 134 B WarmWire identity,
  138 B over TCP including the length prefix.

That is the right starting point. The transport should be pluggable; the warm
protocol should not be tied to one carrier.

## 2. Should this be at TCP/IP level?

No, not at the IP/TCP packet level. Three deployment levels are possible:

1. Library / SDK (recommended first)
   - linked into the application;
   - owns session state, seed cache, operation plans, time headers, witnesses,
     framing and repair;
   - uses existing TCP/QUIC/TLS sockets;
   - no root, no kernel code, portable;
   - easiest to measure and evolve.

2. Sidecar proxy (recommended for unmodified apps)
   - local process; app connects to localhost;
   - proxy forwards to a warm peer or server;
   - compatibility path for apps that cannot link a library;
   - extra copy and one more process, but easy adoption.

3. Kernel module / eBPF / socket interception (later, if ever)
   - would sit below the app at send/recv;
   - hard with TLS, portability and cache semantics;
   - kernel privileges, OS-specific;
   - not justified until the protocol is stable and the library/proxy numbers
     show a real bottleneck.

Recommendation: build the library first; provide a proxy for adoption; treat a
kernel/socket-layer shim as a later optimization only after measurement.

## 3. Multiplexing

Yes, use multiplexing, but not as a replacement for transport-level flow
control. Use it for priority and head-of-line isolation.

Application-level multiplexer (already prototyped):

    frame: magic WMX1 | version | stream_id | kind | priority | length | payload
    kinds: CONTROL, WARM, REPAIR, COLD, TIME_SYNC

Queues: CONTROL > WARM > REPAIR > COLD. That keeps small warm/control frames
from sitting behind a 16 MiB cold-seed transfer.

Transport choices:

- TCP: one connection, application framing and stream IDs. Simple, but a lost
  segment can block all streams (TCP head-of-line). Use small warm frames,
  batching and priorities; keep cold bulk on a separate connection if possible.
- QUIC: preferred next step. Native streams, 0-RTT, connection migration,
  per-stream flow control, TLS 1.3, and no cross-stream head-of-line at the
  stream layer. Map CONTROL/WARM/REPAIR/COLD to streams or stream groups.
- UDP + FEC: useful for lossy/multicast cold-seed distribution and L2
  broadcast; not the default for correctness-sensitive warm state.
- L2/Ethernet: a later carrier via EtherType/VLAN/jumbo frames; the same warm
  frame can be carried inside an Ethernet payload.

Multiplexing rules:

- CONTROL and TIME_SYNC are highest priority, tiny, and may preempt.
- WARM frames are small; send them before cold bulk.
- REPAIR is page/group-scoped and should not wait behind cold bulk.
- COLD is bulk, low priority, and can be batched/coalesced.
- Use credit-based flow control per stream; avoid unbounded local queues.
- Coalesce multiple small frames into one TCP write; disable Nagle; use corking
  or explicit batching.
- Keep a per-stream sequence and idempotent plan IDs for retransmit/replay.

## 4. Library API and usability

Recommended shape:

    WarmSession::open(peer, options)
    WarmSession::preload(seed_ref)
    WarmSession::send(object_or_plan)
    WarmSession::recv()
    WarmSession::sync_time()
    WarmSession::stats()

Options:

- cache directory and budget;
- seed versions to pin;
- TCP/QUIC/L2 transport adapter;
- TLS identity and peer verification;
- fallback-to-raw policy;
- calibration interval;
- repair policy and timeout;
- metrics callback.

Usability requirements:

- drop-in library (Rust core, C ABI, later Python/Go bindings);
- a CLI: warmsend, warmrecv, warmd (daemon/sidecar);
- transparent proxy mode for unmodified apps;
- HTTP/3/QUIC ALPN or a dedicated port;
- one-line fallback to raw if warm negotiation fails;
- seed cache management: list, pin, evict, verify, preload;
- structured metrics: cold bytes, warm bytes, payload bytes, hit rate, repair
  rate, time-sync drift, calibration state;
- versioned protocol with capability exchange and graceful downgrade;
- clear security mode: TLS/QUIC, Merkle roots, signatures where needed;
- deterministic replay/idempotency for warm plans.

## 5. Networking tricks worth adding

Priority and latency:

- 0-RTT / session resumption for warm sends when seed version is cached.
- Connection reuse and connection pooling; do not handshake per transfer.
- CONTROL/TIME_SYNC piggybacking on data frames.
- Small-frame coalescing, TCP_NODELAY, corking, pacing; avoid burst loss.
- QUIC streams/datagrams for head-of-line isolation.
- Multipath (MPTCP or QUIC multipath) with path IDs for redundancy.
- Deadline/priority scheduling; cold bulk as background fill.

Cold-seed distribution:

- Anycast/edge seed caches; one cold send can serve a region.
- Multicast or peer-assisted swarm for cold seed distribution.
- FEC/erasure coding for multicast/lossy seeds; repair pages on demand.
- Prewarm/pinning of hot seed versions; background replication.
- CDN-style cache hierarchy; cache admission/eviction policy.

Delta and reconciliation:

- Content-defined chunking to survive insertions.
- Page-group manifests and group-level proofs; page repair already implemented.
- Merkle proofs for page/group localization.
- IBLT/set reconciliation for many differences without per-page round trips.
- Bloom/cuckoo filters to test seed presence cheaply.
- Dictionary training from the seed cache for small corrections.
- Op fusion and REF/COPY across objects; avoid per-block mode overhead.

Transport/IO:

- zero-copy, mmap seed cache, sendfile/splice for bulk cold seed;
- io_uring / kernel bypass only after profiling;
- checksum offload/TLS record sizing;
- avoid one giant frame: split cold seed into pages/segments;
- adaptive segment size based on MTU and path.

Security and correctness:

- TLS 1.3/QUIC; Merkle roots and optional signatures;
- epoch/replay protection; idempotent plans;
- time sync is not authentication; keep them separate;
- bounded caches and proof sizes; verify every page before commit.

Observability:

- per-stream bytes, latency, repair count, hit ratio, calibration drift;
- compare cold vs warm vs raw on the same object;
- track compute cost per byte saved.

## 6. Suggested rollout

1. Library + TCP framing + multiplexer (the prototype path).
2. Sidecar proxy for unmodified apps.
3. QUIC transport (adapter built; stream priorities/0-RTT hardening next).
4. Page-group repair frame and automatic calibration (both built).
5. WAN campaign across sizes, RTTs and loss levels.
6. Only then consider eBPF/kernel/socket-layer acceleration.

The current prototype already covers the library path, multiplexer frame,
group proof, page repair, bounded replay, calibration, local multi-size
loopback and a QUIC adapter. The next transport step is WAN validation, not a
kernel module.

## 7. QUIC adapter status

Implemented in rust/warm-cache-proto/bin/quic_loopback.rs using quinn 0.11,
rustls 0.23 and tokio, with a local CA-signed localhost certificate. It opens
one bidirectional QUIC stream per message, sends a kind byte plus cold seed or
WarmWire bytes, and the server verifies the cold seed and warm/page-repair
frame. Results: warm identity 134 B, page repair 4,238 B, 4 KiB to 4 MiB on
localhost. This is the transport path to extend to WAN; the TCP loopback path
remains for comparison.


## 8. Library and CLI status

WarmSession is now the embedding API: seed, time header, encode/decode,
classification and byte/payload/repair stats. warm_send and warm_recv provide a
CLI over TCP: cold seed, then WarmWire identity/page-repair, with exact output
verification. A transparent sidecar proxy is a packaging layer on this API, not
a new protocol.


## 9. Emulated WAN status

quic_wan_emulated adds a configurable one-way server delay to the QUIC loopback.
At 60 ms one-way (about 120 ms RTT), 4 KiB to 1 MiB seeds show warm identity at
~63-122 ms and 134 B, page repair at ~62-118 ms and 4,238 B. This is a latency
emulation, not a real WAN run. A real TCP WAN run has now been completed; see
section 11.


## 10. QUIC WAN harness

quic_wan server <bind> / quic_wan client <addr> [sizes] is implemented and
validated on localhost: 4 KiB to 1 MiB, warm 134 B, repair 4,238 B. The
harness was then run across the laptop and a real Vast pod; see section 11.


## 11. Real remote WAN result

A real pod test was completed over TCP/SSH because Vast exposes only port 22.
One cold seed then 3-5 warm frames per size: identity 134 B at ~209-217 ms,
one-page repair 4,238 B at ~206-263 ms, for 4 KiB to 4 MiB, all payload hashes
exact. The pod was released and deleted. Direct QUIC UDP was blocked by the
pod port policy; a UDP-reachable endpoint is needed for QUIC WAN.

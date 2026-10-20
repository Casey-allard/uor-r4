# Warm Cache Network — unified design draft

Date: 2026-10-19. Status: design synthesis, not implemented.
Goal: make warm network transfers as fast as possible with the least data,
using a shared seed, one cold send, operations on cached state, and evidence-
based assumptions instead of re-sending bytes.

The design does not discard the mechanisms that failed isolated tests. It gives
each one a layer and a job. A failed literal implementation becomes a constraint
or a different-layer role.

## 1. Cost model

For an object D, shared seed S, operation plan P, parameters X:

    D = eval(P, S, X)

Cold send: transmit S, P, manifest, and the evidence needed to trust them.
Warm send: transmit only (seed_id, version, plan_id, X, assumption certificate,
sparse corrections, witness).

The sender chooses the cheapest plan under

    cost = bytes_on_wire + lambda * receiver_compute + mu * latency

subject to exact reconstruction and verification. The target is zero payload
bytes when the plan and assumptions hold, with a small correction path when
they do not.

This is bounded algorithmic compression: send the shortest program/parameters
that reconstruct D from state both sides already have, not bytes that describe D.

## 2. Four planes

### A. Seed / cache plane
- Immutable content-addressed seed, split into 4 KiB pages; each page has a
  Merkle root and a version.
- 256-byte change blocks inside pages; 4 KiB pages are the cache/eviction and
  repair unit, 256-byte blocks are the change unit.
- The 97-byte/77-byte page results are not used as wire framing; they are
  evidence for the cache-block size choice. 4 KiB pages + 256 B blocks won the
  overhead test architecture.
- Edge seed caches, origin seed store, version chain. One cold send per seed
  version; warm sends reference seed_id and version.
- Page/zone ideas become cache-page identity, guards, and prefetch groups, not
  extra per-packet headers.

### B. Transfer plane
- Frame types: COLD_SEED, WARM_PLAN, ASSUME, CORRECTION, REPAIR, WITNESS.
- Warm frames are optimistic: seed_id, version, plan_id, parameter block,
  assumption certificate (Merkle/fingerprint), sparse corrections, witness.
- Zero-RTT if the receiver advertises the seed version; otherwise one-RTT
  capability exchange, then optimistic warm frames.
- Repair is page/leaf granularity, never full-object resend unless the seed is
  absent.
- Compression decisions: per 256-byte block choose ASSUME / REF / DIFF /
  OP / LITERAL; choose the smallest encoded operation, not the smallest raw
  residual. Fold the mode into the opcode (the DIFF enc fold).
- Transport: TCP/QUIC first; L2/Ethernet as a carrier later. The carrier is a
  byte stream; ternary is a virtual control/state layer, not a payload format.

### C. Operation plane
- A small deterministic, side-effect-free, bounded operation language:
  COPY, REF, DIFF, PATCH, PERMUTE, ROTATE, CONST, CONCAT, SPLIT, MERGE,
  PARAM, CHECK, COMMIT.
- Operations are compiled to native Rust executors. The wire carries opcodes
  and operands, not a byte-by-byte interpreter stream.
- POLIZ is used as the portable cold-send recipe format and as a fallback
  planner, not as the hot-path interpreter. The measured 4.6x dispatch overhead
  and 4x copy expansion mean the hot path must fuse ops and execute compiled
  plans.
- Per-block REF references earlier output; cross-object REF references cached
  seed pages. Content-defined chunking gives block boundaries that survive
  insertions.
- Register IDs/opcodes are versioned; the Bell-15/normal opcode space is a
  control-space decision, not a payload compression trick.

### D. Evidence / assumption plane
- Every receiver keeps a predictor: previous version, page roots, block
  assumptions, access history, and a confidence state per page.
- Sender sends an assumption certificate: plan_id + predicted page roots or
  sampled fingerprints + expected output root. Receiver applies the plan to
  cached state and verifies the root. If it matches, no corrections and no
  resend.
- If it mismatches, binary-search the Merkle tree, then request only the
  differing leaves/pages. The repair message is a set reconciliation, not a
  full dump.
- Evidence states are ternary in meaning, not in wire packing:
  ASSUMED (prediction holds), OBSERVED (fresh data), UNKNOWN (must ask).
  The AURA borderline band is not used; the measured band is empty. The
  tri-state is a protocol state, not a threshold on PAS.
- Balanced ternary/q=0 arithmetic is used for canonical, tie-free rounding of
  assumptions and for operation/state identities. It is not used to pack
  payload bytes: the measured token/payload packings expand by about 1 percent
  or more.
- The 8-byte capsule is the fast accidental witness; Merkle roots are the
  stronger repair witness. The capsule is not a cryptographic proof.

## 3. One cold send, many warm sends

Cold send:
1. Receiver advertises seed capabilities (ids, versions, page roots, cache
   budget).
2. Sender sends the seed bundle (or a seed program that generates it), the
   operation plan, the manifest, and the root witness.
3. Receiver materializes the seed, verifies pages, stores it in the edge cache.
4. One cold send can be multicast/anycast to many receivers; the seed is now
   shared.

Warm send:
1. Sender references seed_id + version + plan_id.
2. Sender sends parameters and an assumption certificate: predicted output
   root or sampled fingerprints.
3. Receiver applies the plan to cached seed/pages and verifies the root.
4. Match: commit, no payload. Mismatch: request a Merkle-guided repair.
5. Corrections are sparse ops (REF/DIFF/OP), not a new full object.
6. A version chain lets subsequent warm sends apply deltas to the previous
   verified version.

The ideal warm frame is a few dozen bytes and one RTT or zero RTT; the ideal
warm payload is zero bytes. The transfer becomes compute plus a witness, not a
line transfer.

## 4. Large-network topology

- Global content-addressed seed registry; regional edge seed caches; receivers
  with bounded cache. The seed is immutable and versioned; plans are mutable
  recipes that reference versions.
- Cold sends use anycast/multicast to seed caches; warm sends are per-receiver
  or per-group.
- Anycast/edge selection uses stable hashing over seeds/regions; 120-channel
  icosian/decagon structure is a candidate for **placement/routing topology**
  (12-neighbor greedy hop plus ring A/B paths), not for data compression.
  Ring A/B can be the two repair paths/replicas; decagon pairs are placement
  groups. This is a design hypothesis to measure.
- Kuramoto/phase control is a candidate for **pacing and congestion phase**,
  not for compression: synchronize send windows, avoid burst loss, and define
  assumption refresh epochs. The AURA gate itself is not a data gate.
- Bell MF codes are a candidate for **control/repair message classes**, not
  payload coding.
- The UMT 19-bit token is a compact **operation/state identity** for plans and
  assumptions; it is not a payload unit.
- 3-adic carry is a candidate for **version/ordering algebra** and canonical
  cache lineage, not for data compression.
- Base-63/33-byte framing is a candidate for compact IDs/manifests, not for
  carrying the seed or payload.

## 5. What each earlier mechanism becomes

| Mechanism | New role |
|---|---|
| REF/COPY | Cross-payload and cross-object references; the primary measured win. |
| DIFF/ASSUME | Residual operation when a prediction fails; smallest-operation choice. |
| POLIZ | Portable cold-send recipe/fallback planner; compiled out of the hot path. |
| Ternary/balanced packing | The virtual state/operation algebra. Binary is one exact carrier; the 19-bit UMT is its minimal binary view. Never a payload codec. |
| Page/zone (43x9, 81x6) | Cache-page identity, guards, prefetch groups; not wire framing. |
| Tri-state AURA band | Protocol evidence state ASSUMED/OBSERVED/UNKNOWN; not a PAS threshold. |
| 120 icosians/decagons | Seed placement/anycast topology and two repair paths. |
| Kuramoto/phase | Logical time base: hierarchical epoch/phase serialization for witness hashing and Merkle roots; sync beacons at each level. |
| UMT 19-bit | Plan/assumption identity; not a byte container. |
| Bell MF | Control/repair message classes. |
| 3-adic carry | Version lineage and operation ordering. |
| 8-byte capsule | Fast accidental witness; Merkle root for repair. |
| Gemini fixed header/root table | Immutable seed manifest concept; no reliance on its 250/254/81 numbers. |
| Gemini V4P/root IDs | A versioned manifest/root table idea; layout is a hypothesis, not a measured artifact. |
| Gemini -149.9 dB | Not a design input; exact-zero is not a finite dB; remove hardcoded field. |

## 6. Mathematical framing

- **Bounded algorithmic compression:** choose the shortest plan P and
  parameters X such that eval(P,S,X)=D. The seed is the shared prefix; warm
  sends carry the program suffix and evidence.
- **Prediction-residual coding:** a model predicts D from S and history; send
  only residuals. The receiver runs the same model. Compute substitutes for
  bandwidth only when the model is cheaper than the bytes saved.
- **Assumption certificates:** a root/fingerprint proves the receiver's computed
  result equals the sender's without transmitting the result. Merkle trees
  localize mismatches; sampled fingerprints reduce verification compute.
- **Set reconciliation:** when caches diverge, exchange compact summaries
  (Merkle/IBLT) and repair only differences. This is the multi-receiver cold/
  warm consistency mechanism.
- **Canonical rounding:** q=0 balanced ternary gives tie-free truncation; use it
  to make both peers quantize assumptions identically, reducing false
  mismatches. This is a compute/reliability aid, not a compression claim.

## 7. First prototype and acceptance bars

Prototype additions to wire-warm:
- SeedManifest {id, version, page_roots, root, plan_ids}
- OperationPlan {id, ops, input_roots, output_root}
- Assumption {seed_id, version, plan_id, params, predicted_root, samples}
- WARM_PLAN / ASSUME / CORRECTION / REPAIR frame types
- Merkle page tree; repair by leaf/page
- REF/COPY, DIFF, op fusion; compiled Rust ops
- 0-RTT warm path when seed version is cached.

Acceptance:
- Identical warm transfer: zero payload bytes, one small header, exact root.
- Self-similar 1 MiB: REF path at the measured 4.1-4.2 KB (or the actual
  measured after implementation), not 1.05 MB.
- 1 percent scattered: DIFF fold path at or below 29,113 B projected.
- Cache miss/repair: only differing pages move; no full resend.
- WAN: warm path at most one RTT in the best case; no handshake per transfer.
- Compute budget: native op execution cheaper than the bytes it replaces;
  reject a plan if not.

## 8. Open questions before implementation

1. What is the dominant warm workload: exact repeats, small mutations, a
   generated family from one source, or many receivers sharing a seed?
2. Where does the seed live and who owns it: origin, edge cache, peer-to-peer?
   How much cache, and can we assume durable seed versions?
3. Transport and threat model: TCP/QUIC over Internet first, or Ethernet/L2?
   Are peers trusted, or do we need cryptographic verification and signatures?
4. What operations are allowed and what compute budget is acceptable on the
   receiver? Which transforms generate the data in practice?
5. What does "one cold send" mean precisely: one per object, one per session,
   one per swarm? Must the cold send carry the full seed, or can the seed be
   generated from a smaller manifest?

## 9. Virtual representation contract (added 2026-10-19)

The network is a virtual machine first. Binary and ternary are two exact views
of the same state, not two competing codecs.

- Canonical virtual state: a balanced-ternary vector (q=0 for rounding).
- Carrier encoding: a binary serialization of that state. The 19-bit UMT is the
  minimal binary view of the 32 x 15 x 1024 token state space; 12 trits is the
  virtual view. Both are exact; neither is "the compression".
- Operations are defined on the ternary state and emitted as binary opcodes.
  The receiver decodes operands to ternary, executes the operation, and
  re-encodes the result for the witness/Merkle hash.
- Assumption certificates are hashes of a canonical binary serialization of the
  ternary state. Equality is representation-independent.
- A ternary carrier, an L2 binary carrier and a TCP binary carrier all carry
  the same state/operation semantics; only measured cost differs.
- Page/zone, POLIZ, tri-state evidence and REF/DIFF keep their layer roles; the
  binary/ternary choice is a carrier decision inside those layers.

Measured representation test (analysis/representation_tests.py):
- all 491,520 valid token states round-trip exactly between 19-bit binary and
  12-trit ternary views (0 failures);
- the ternary view has 531,441 addresses, the binary view 524,288; the valid
  subset is 491,520 in both;
- 5-trit practical packing of 100,000 tokens is 240,000 bytes vs 237,500 bytes
  for 19-bit binary; this is a carrier cost, not a semantic difference;
- conversion throughput on this host is about 2.09 million state conversions
  per second in Python; native Rust will be faster and will be measured.

Consequence for the design: we do not have to choose "binary or ternary" at the
architecture level. We choose the cheapest exact carrier per layer, keep one
canonical ternary state and one canonical binary witness, and test that both
directions round-trip. The warm-network operations are ternary in meaning and
binary on the wire.


## 10. Kuramoto time (added 2026-10-19)

Kuramoto is the time base, not just a pacing heuristic. Each architecture
level has oscillators, an order parameter, an epoch and a phase bin. A level
commits a time tag only when its coherence and parent alignment pass; levels
exchange small TIME_SYNC beacons. The time header T is serialized into the
temporal witness and the time-stamped Merkle nodes; content roots remain
time-free for seed identity/dedup, and temporal roots carry the Kuramoto time
for freshness, ordering, replay rejection and repair localization.

The full design is in KURAMOTO_TIME.md. Defaults: L0 core (4 oscillators),
L1 node, L2 edge, L3 region; R_min 0.95, W=3 cycles; content root for seed_id,
temporal root for witness; AURA gate used only as the epoch-commit coherence
gate.


## 12. Prototype built and tested (2026-10-19)

The agreed slice is implemented in rust/warm-cache-proto: four normal
oscillators, forward-only FFFF Kuramoto time headers, exact replay,
content root vs temporal root vs witness, and a one-cold-seed / many-warm
accounting test. 16/16 tests pass; multi-page Merkle and size sweep included. Demo: cold seed 65,572 B; zero-payload
warm frame 124 B; sparse 15 B patch 139 B; one-page repair 4,228 B; content equal across clock change,
temporal root different; stale clock classified ClockMismatch.

See ../PROTOTYPE_RESULTS.md for the build/test record.


## 13. Gap fill completed (2026-10-19)

The prototype now has a 64-page-group seed manifest, multi-page repair, a
bounded replay ring, a versioned warm wire frame (WCF1), a calibration module
for B_l/R_min/W, and a multi-size TCP loopback. 31/31 tests pass. FFFF header
is 24 B; identity warm frame 124 B; WarmWire identity 134 B; loopback identity
138 B including the 4 B TCP prefix; one-page repair 4,228 B frame / 4,242 B
loopback. Size sweep covers 4 KiB to 16 MiB.

See ../PROTOTYPE_RESULTS.md.


## 14. Gap-fill follow-up (2026-10-19)

The prototype now also has a page-group manifest with group proofs, a
group-repair wire frame, a WMX1 application multiplexer with CONTROL/WARM/
REPAIR/COLD priorities, and calibration from measured phase spread. Tests:
28/28 pass. The warm path remains forward-only FFFF; see NETWORKING.md for the
TCP/QUIC, library-vs-proxy-vs-kernel design and usability suggestions.

## 15. QUIC adapter built (2026-10-19)

A QUIC loopback adapter now exists: quinn 0.11 + rustls 0.23 + tokio, CA-signed
localhost cert, one bidirectional stream per message. Warm identity 134 B;
one-page repair 4,238 B; sizes 4 KiB to 4 MiB. This closes the transport-adapter
gap. Remaining: merge into the wire-warm crate and run the WAN campaign.


## 16. Library and CLI built (2026-10-19)

WarmSession library API and warm_send/warm_recv CLI are implemented and tested
on 1 MiB with a one-page repair; output matches exactly. Tests: 30/30 pass.
Transport: TCP and QUIC loopback. Remaining external items: WAN campaign and
packaging the WarmWire frame into the wire-warm crate.


## 17. Emulated WAN and final local status (2026-10-19)

QUIC WAN emulation (60 ms one-way) is in quic_wan_emulated.rs: warm identity
134 B at ~63-122 ms, page repair 4,238 B at ~62-118 ms for 4 KiB to 1 MiB.
This is emulation; real WAN needs a remote peer. Local prototype tests: 31/31, including a wire-warm composition test.


## 18. QUIC WAN harness (2026-10-19)

quic_wan has server/client roles and was validated on localhost: warm identity
134 B, page repair 4,238 B, 4 KiB to 1 MiB. A real WAN run needs two hosts and
a certificate valid for the server name; the real TCP/SSH pod run is recorded
in section 19.


## 19. Real remote WAN test (2026-10-20)

Remote pod vast-56894372: TCP warm protocol over SSH tunnel, one cold seed +
repeated warm frames. Identity 134 B at ~209-217 ms; page repair 4,238 B at
~206-263 ms; 4 KiB to 4 MiB; all hashes exact. Vast exposes only SSH/22, so
QUIC direct was blocked; pod released/deleted. See
rust/results/wan_tcp_remote.txt.

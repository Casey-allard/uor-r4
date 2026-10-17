# UOR object-store peer protocol (v1, UOROBJ01)

Status: experimental research contract, 2026-10-17. This document is the byte-level specification needed
to port the protocol to another runtime (container, VM, MVM or a non-Rust service). The reference
implementation is crates/uor-r4-object-store (peer.rs, manifest.rs, record.rs, store.rs, hash.rs) and
the CLI is src/bin/objpeer.rs. Review evidence and the independent driver live in the spiralcore-fbs
review repository (port/driver). Message kinds and the manifest/record layouts are also used by that
driver, and interoperability is tested in both directions.

Scope: content-addressed artifact replication and verification over a reliable, ordered byte stream
(TCP, TLS, SSH tunnel, Unix socket, in-process pipe). It is not a compressor and does not encrypt or
authenticate the transport; see Trust.

## 1. Object model

- An object is an arbitrary byte string (model weights, container layer, disk image, file tree blob).
- The object is split into 32-byte atoms; the last atom is zero-padded. Let atoms = ceil(len / 32).
- Atoms are folded into a ternary FBS tree: a leaf has scale 0 and a 32-byte payload; a parent has
  scale >= 1 and three child ids (36-byte payload). A node at scale s covers at most 3^s atoms.
- The frontier of a tree is defined by the base-3 digits of atoms: for scale from high to low, digit d
  means d roots at that scale. roots = digits descending. Example: atoms=32 (base3 1012) gives roots
  [scale 3 x1, scale 2 x0, scale 1 x1, scale 0 x2] in that order.
- A manifest (UOROBJ01) binds the object length, the frontier roots and the SHA-256 of the object bytes.

## 2. Node ids (must match bitwise)

node_id(scale, payload) is a BLAKE2b variant with a 12-byte digest over a 128-byte zero-padded block:

    input = [scale] || payload            (33 bytes leaf, 37 bytes parent)
    block[0..128] = input || zero padding
    m[i] = u64 little-endian of block[8i .. 8i+8]    for i in 0..16
    h = IV; h[0] ^= 0x0101000c
    v[0..8] = h; v[8..16] = IV
    v[12] ^= len(input)                  (BLAKE2b counter = logical input length, not 128)
    v[14] = !v[14]                       (last-block flag)
    for r in 0..12: apply the standard BLAKE2b sigma rounds to v using m
    H[i] = h[i] ^ v[i] ^ v[i+8]          for i in 0..8
    id = little-endian(H[0]) (8 bytes) || first 4 bytes of little-endian(H[1])

IV and SIG (sigma) are the standard BLAKE2b constants. The digest length is 12 bytes and the fanout
byte in h[0] is 0x0c; the custom counter and padding are the compatibility-critical details. A port
must reproduce this function exactly, or ids will not match existing stores.

Equivalently, this is standard BLAKE2b with a 12-byte digest over the logical input [scale] || payload
(digest_size 12, fanout 1, depth 1): the 128-byte block, zero padding and counter = len(input) are the
reference implementation's way of hashing the logical input. A port may call a standard BLAKE2b-96 API
directly. A Python port using hashlib.blake2b(digest_size=12) reproduced the conformance vectors and
cold-pulled from the reference server, so the equivalence is tested rather than assumed.

## 3. Records

Leaf record:  scale=0 (1 byte) || id (12) || payload (32)             total 45 bytes
Parent record: scale>=1 (1)    || id (12) || child ids (3 x 12)        total 49 bytes

Records are validated on decode: span(scale)=3^scale must fit in u64, the declared id must equal
node_id(scale, payload), and for parents all three children are ids. The id is self-certifying.

## 4. Manifest (UOROBJ01)

    offset 0   8   magic = "UOROBJ01"
    offset 8   8   object length, u64 little-endian
    offset 16  32  SHA-256 of the object bytes
    offset 48  1   root count n (<= 128)
    offset 49  13n roots, each id (12 bytes) followed by scale (1 byte)

Total size is 49 + 13n. The manifest version is SHA-256 of the canonical encoded manifest bytes. The
whole-object identity is the SHA-256 at offset 16; the length and frontier must follow the base-3 rule.

## 5. Framing

Every message is: kind (1 byte) || body length (u32 big-endian) || body. Implementations must cap a
message at 64 MiB and must reject truncated frames. No compression, no CRC and no JSON on the wire.

    kind 20  MANIFEST_REQUEST   body empty (first/default object) or 32-byte selector
    kind 21  MANIFEST           body = encoded UOROBJ01 manifest
    kind 22  FETCH              body = u16 count || count * NodeId (12 bytes each)
    kind 23  NODES              body = u16 count || count * record (45 or 49 bytes)
    kind 24  DONE               body empty; ends the session normally
    kind 25  ERROR              body = short UTF-8 reason (for example unknown object)

A 32-byte selector matches an object when it equals the manifest version or the whole-object SHA-256.
An unknown selector receives ERROR and the session stays open; the client reports that check as unknown.

## 6. Client flow: pull

1. Open a session and send MANIFEST_REQUEST (optionally with a selector).
2. Read MANIFEST; decode and validate it. If the caller supplied ExpectedObject pins (native length,
   manifest version and/or whole-object SHA-256), every supplied pin must match or the pull fails with
   BindingMismatch before any record is fetched.
3. Compute the missing frontier: walk the manifest roots, skip nodes already in the local store, stop
   at nodes that are missing locally without descending, deduplicate ids. Request missing ids in
   FETCH batches derived from the store's limits (the reference client defaults to 4096; `objpeer
   pull --batch N` raises it up to the frame count cap of 65535); every response must contain exactly the
   requested ids, in order, with no duplicates and no trailing bytes.
4. Stage records in memory (do not import yet), recompute the missing frontier, repeat until none.
5. Assemble the object from the store plus staged records, concatenate leaf payloads in frontier order,
   trim to the manifest length, compute SHA-256 and compare with the manifest value.
6. Only after the digest matches, import all staged records atomically and send DONE. A failed pull
   must leave the store unchanged.

## 7. Client flow: verify and verify_series

verify sends MANIFEST_REQUEST, checks the pins, walks the local closure and reports the number of
missing frontier nodes without fetching or rebuilding. verified=true means the local store's closure
for the pinned manifest is complete (content ids), not that the payload was re-hashed.

verify_series runs many checks on one session, pipelining requests in windows of 64: write the windows
of MANIFEST_REQUESTs, then read the MANIFEST/ERROR responses in order. One DONE ends the session. Per
check outcomes are verified, stale (missing > 0), unknown selector or pin mismatch. A cross-machine
20-check pass measures 4,425 bytes and about 0.6 s; a 200-check audit measures 44,205 bytes in 0.225 s
locally.

## 8. Client flow: pull_series (fleet sync)

pull_series pulls several selected objects on one session: for each request run the single-object pull
steps without sending DONE, then send one DONE after the last object. Each artifact is fully verified
and imported before the next. Store load/save is the caller's cost, not the protocol's.

## 9. Server behavior

serve(store, manifest) is serve_many(store, [manifest]) with one entry. serve_many precomputes the
closure of each manifest, then loops: a MANIFEST_REQUEST selects an object (empty body = first), a FETCH
is answered only for ids in the selected object's closure, DONE returns normally, EOF also returns
normally, and any schema violation terminates the session with an error. Request count, served bytes
and per-message size are bounded by Limits.

Default Limits: max_object_bytes 16 MiB, max_records 1,000,000, max_record_bytes 64 MiB,
max_returned_bytes 16 MiB, max_visits 1,000,000, max_occurrences 1,000,000. The product CLI raises
object/record bounds for large artifacts while keeping message and budget checks.

## 10. Errors

SchemaMismatch: malformed frame, bad count, trailing bytes, invalid manifest or record.
IntegrityFailure: id mismatch, digest mismatch, duplicate/foreign records, truncated payload.
BindingMismatch: peer manifest does not match the caller pin.
UnknownObject: selector not present on the peer.
LimitExceeded: a configured budget was crossed.
Io: classified OS error (kind/raw code), never parsed prose.

## 11. Trust and transport

Ids and manifests are content-certifying: a correct pin plus digest verification proves the bytes are
the pinned artifact. The protocol does not authenticate the peer, encrypt traffic, or hide object size
and record shape; run it over TLS/mTLS, SSH or WireGuard where that matters. Pins are checked before
fetch and imports are atomic, so a wrong/stale peer cannot corrupt a store under a correct pin.

## 12. Porting checklist

- Little-endian only for manifest length and node-id words; big-endian only for framing lengths.
- Reproduce node_id exactly (custom BLAKE2b-96, counter = logical input length, 128-byte padded block).
- Canonical manifest bytes are the input to the version hash; roots order is digits descending.
- Validate record id on decode; cap scale so 3^scale fits in u64; enforce root count and frontier rule.
- Keep the DAG dedupe rule: identical subtrees share ids; request each missing id once.
- Stage records, verify the object digest, then import atomically; never expose partial pulls.
- Correlate NODES count and ids to the request; reject duplicates and leftover bytes.
- Treat DONE/EOF as clean session ends; ERROR is per-selector for unknown objects.
- Do not add a version field to the frame in v1; new capabilities use new kinds or selector bodies.

## 13. Reference files

    crates/uor-r4-object-store/src/peer.rs      wire kinds, serve/serve_many, pull/pull_series, verify
    crates/uor-r4-object-store/src/manifest.rs  UOROBJ01 encoding, validation, version
    crates/uor-r4-object-store/src/record.rs    record wire format and id validation
    crates/uor-r4-object-store/src/hash.rs      custom BLAKE2b-96 node id
    crates/uor-r4-object-store/src/store.rs     fold, missing, rebuild, import, persistence
    crates/uor-r4-object-store/src/bin/objpeer.rs  product CLI (info/serve/pull/verify/sync)

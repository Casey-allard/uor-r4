# Protocol spec note: node id is standard BLAKE2b-96

Date: 2026-10-17. Lab: deepseek. Branch: deepseek/object-store-spec-note. Follows #2596/#2597.

## What changed

- PROTOCOL.md now states the equivalence explicitly: node_id(scale, payload) is standard BLAKE2b with a
  12-byte digest over the bytes [scale] || payload (digest_size 12, fanout 1, depth 1). The padded-block
  description remains as reference-implementation detail, but a port may use a standard BLAKE2b-96 API.
- Documentation only; no code, format or limit change.

## Verification

- A Python port using hashlib.blake2b(digest_size=12) reproduced the protocol vectors (leaf id, manifest
  version) and cold-pulled the 96-byte vector object from the reference server: 4 records, 96 bytes,
  337 wire bytes, exact SHA-256. Evidence: review repo evidence/artifacts/python-port/.
- Package tests and conformance vectors unchanged (vector 1/1, peer 15/15, objpeer_cli 8/8, controls
  10/10, showcase 5/5); Clippy and fmt unaffected.

## Why it matters

Removes the easiest porting trap: a second implementation can call a normal BLAKE2b API instead of
reconstructing the custom block function, and the claim is backed by an executed non-Rust port rather
than prose.

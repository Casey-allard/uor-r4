# Exact object store: native peer replication

Date: 2026-10-17. Lab: deepseek. Branch: deepseek/object-store-peer.

## What changed

- Store::record_bytes(id) and Store::has_record(id) expose the native encoded record (scale, id,
  payload) for peer serving. import_records, missing, rebuild, fold and Manifest::encode/decode were
  already public and are unchanged.
- New uor-r4-object-store/src/peer.rs (plus a PullStats export): a transport-agnostic replication
  session over any Read + Write pair.
  - The receiver requests a manifest, then repeatedly requests exactly the node ids Store::missing
    reports, imports the returned native records through the validated import_records path, and
    finally rebuilds the object with native digest verification.
  - The sender serves the native record bytes for requested ids only; unknown ids are an
    Error::Missing. Message kinds 20..=24 and the length framing match the proven P2P driver, whose
    native compatibility with this store is established in the review repository.
- New integration test crates/uor-r4-object-store/tests/peer.rs.

## Verification (local, this worktree)

- cargo test -p uor-r4-object-store: 25 passed, 0 failed (3 new peer tests).
- cargo clippy -p uor-r4-object-store --all-targets -- -D warnings: clean.
- cargo fmt -p uor-r4-object-store --check: clean.
- Delta test (1,048,576-byte object, receiver holds a 16-atom-older version):
  22 records / 1,014 bytes / 3 rounds, verified, exact rebuild.
- Identical store: 0 records / 0 bytes, verified.
- Record accessor test decodes the served record with the native Record::decode and checks id/scale.

## Scope and limits

- Transport-agnostic; no network or model code in this crate. A caller supplies the session's streams
  and the manifest of the object being served (from Store::fold or Manifest::decode).
- Single peer, sequential message exchange, no authorization beyond the store admission controls; the
  caller decides which stores and objects to expose.
- No model compute, no floating point, no training change. Training/M2/Stage B holds and the current
  model milestone state are unchanged.

## Next

- Wire the session to the P2P driver transport, then gate a two-machine delta replication (the driver
  already measured 22 nodes / 1,014 bytes for this same 1 MiB delta across machines).
- Multi-peer fan-out and restart from a persisted store (Store::save/load already exist).

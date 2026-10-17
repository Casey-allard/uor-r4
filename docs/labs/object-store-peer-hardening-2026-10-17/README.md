# Exact object store: peer replication hardening

Date: 2026-10-17. Lab: deepseek. Branch: deepseek/object-store-peer-hardening. Follows #2580.

## What changed

- Trusted anchor: new ExpectedObject (byte length + manifest version) and pull_expected. A peer
  manifest that does not match the caller's pinned identity is rejected with BindingMismatch before any
  record is fetched or imported. pull remains as pull_expected(store, None, ...) for callers without a pin.
- Dynamic batching: the request batch is derived from the store limits (up to 4096 ids per request), and
  the pull loop runs until the manifest closure is complete. The previous fixed 256 x 128 cap could not
  finish a cold 1 MiB pull; a cold 1 MiB object now completes and verifies.
- Correlated responses: the declared record count must equal the batch size, every returned id must be
  one of the requested ids, duplicates are rejected, and a truncated or padded response is rejected.
- Atomic commit: fetched records are staged in memory, the object bytes are assembled from store plus
  staged records, the native SHA-256 is checked against the manifest, and only then are records imported
  through the validated import path. A failed identity check leaves the store untouched.
- Protocol hygiene: DONE is sent after commit, every message flushes, a clean EOF ends a serve session,
  and serve answers only node ids in the requested manifest closure with request and byte budgets.
- Unit test: the served closure excludes records that belong to a different object in the same store.

## Verification (local worktree)

- cargo test -p uor-r4-object-store: all package tests pass (peer suite 6/6), Clippy -D warnings clean,
  fmt clean.
- Cold pull of a 1 MiB object from an empty receiver: completes, imports the full object store path and
  rebuilds the exact bytes with the native digest.
- Scattered change: 16 edits spread across a 1 MiB object add 131 records (paths, not payload), exact.
- Wrong expected identity: BindingMismatch, zero records imported.
- Closure scoping: records from another folded object are not servable.

## Scope and next

- Single peer, sequential request/response over any Read + Write; transport adapter remains the caller's
  choice (the P2P driver TCP framing is compatible in message kinds but the native manifest body differs
  from the driver's custom sync manifest, so cross-crate interop needs an explicit adapter).
- Driver-side sync DoS/limit fixes (separate review-repo work) and multi-peer fan-out remain next.
- No model compute, no floating point, no training change; training/M2/Stage B holds unchanged.

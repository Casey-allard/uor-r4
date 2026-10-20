# Peer replication: concurrent sessions on one read-only store

Date: 2026-10-17. Lab: deepseek. Branch: deepseek/object-store-concurrency. Follows #2584/#2585.

## What changed

- Documented and tested that peer::serve takes &Store, so multiple peers may replicate from the same
  loaded store concurrently; the store is never mutated by a session.
- New test concurrent_sessions_share_one_read_only_store: one folded store, two independent receivers
  each holding the old object, two sessions accepted on scoped threads, both clients pull the same
  delta simultaneously, both verify with the native digest and produce identical counts and bytes.

## Verification

- Peer suite 8/8; package 31 passed and 0 failed; Clippy -D warnings clean; fmt clean.
- No model compute, no floating point, no training change; training/M2/Stage B holds unchanged.

## Why it matters

The product path (objpeer) already serves 12 warm receivers concurrently at 0.864 s versus 3.506 s
sequential with identical wire; this test makes the underlying crate guarantee explicit and gated.

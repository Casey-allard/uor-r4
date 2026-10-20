# Replica-aware sync with failover

Date: 2026-10-17. Lab: deepseek. Branch: deepseek/object-store-replica-sync. Follows #2602.

## What changed

- objpeer sync now accepts repeated --addr and treats them as a replica list. It tries each replica in
  order: per-replica retries apply (--retries), and if a replica fails (connection error, protocol error
  or an object it does not hold) the next replica is tried. The successful artifact set is written once,
  and JSON/SYNC_SERIES report which replica served it (replica=...).
- Only the successful replica's wire is counted. Partial imports from a failed replica are content-
  addressed and safe: the next replica skips records already in the store.
- pull still uses the first --addr (a single pull has one source); repeated --addr is meaningful for
  verify (replica audit) and sync (replica failover).

## Verification

- packaged_cli_sync_fails_over_from_a_dead_replica: first address 127.0.0.1:1 refuses, second serves;
  sync succeeds, reports replica=<second>, and writes the exact artifact.
- packaged_cli_sync_fails_over_when_first_replica_lacks_the_target: first server holds a different
  object (UnknownObject), second holds the target; failover succeeds and reports the second replica.
- objpeer_cli 14/14, peer 15/15, controls 10/10, protocol_vectors 1/1, showcase 5/5; Clippy -D warnings
  and fmt clean.

## Why it matters

A node updating a fleet should not care which replica answers as long as one has the target set. This
completes the replica story: audit all replicas, then sync from whichever replica is healthy and current.

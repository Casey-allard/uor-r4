# Replica-aware fleet audit

Date: 2026-10-17. Lab: deepseek. Branch: deepseek/object-store-replica-audit. Follows #2594/#2600.

## What changed

- objpeer verify accepts repeated --addr (a replica set). Each replica is audited with the same warm
  store, pins and retry policy; results are reported per target per replica.
- Single-address output is unchanged (human VERIFY lines and the existing JSON shape), so existing
  scripts keep working. With two or more addresses:
  - JSON emits {replicas, checks, verified_all, stale, unknown, mismatch, wire_bytes, results:[{selector,
    replicas:[{addr,status,missing}]}]};
  - human output emits one REPLICA addr=... line per replica plus VERIFY_REPLICAS totals.
- Exit codes: 2 if any target is unknown on any replica, 3 if any target is stale or pin-mismatched on
  any replica, 0 only when every target is verified on every replica.
- pull/sync still use the first --addr (a pull has one source); repeated --addr is meaningful for verify.

## Verification

- packaged_cli_multi_replica_audit: two servers with the same object, one client store, verify --addr a1
  --addr a2 --json -> exit 0, replicas=2, two verified statuses.
- packaged_cli_multi_replica_reports_unknown_on_one_replica: replica 2 lacks the object -> exit 2,
  one verified and one unknown in the JSON.
- objpeer_cli 12/12, peer 15/15, controls 10/10, protocol_vectors 1/1, showcase 5/5; Clippy -D warnings
  and fmt clean.

## Why it matters

A registry or fleet has several replicas; an operator needs to know which replica is current for each
artifact, not just whether one server answered. This is the replica-status shape, reusing the pipelined
verify path and retry policy unchanged.

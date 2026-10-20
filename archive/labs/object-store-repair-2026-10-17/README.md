# objpeer repair: audit, select, and pull only the missing targets

Date: 2026-10-17. Lab: deepseek. Branch: deepseek/object-store-repair. Follows #2603.

## What changed

- New objpeer repair command closes the fleet maintenance loop in one verb:
  1. probe each replica (repeated --addr) with verify_series against the local store;
  2. per target, classify Current (some replica says the local store is complete), Repaired (a replica
     holds the manifest but the local store is missing nodes) or Unavailable (no replica holds it);
  3. group the Repaired targets by the replica that holds them and pull each group on one session;
  4. report per-target status and a summary; JSON has replicas/checks/current/repaired/unavailable/
     failed/wire_bytes and per-target status with the serving replica.
- Exit 0 when everything is current or repaired; exit 3 if any target is unavailable or a pull failed.
- --retries applies per replica probe and per repair group; partial content-addressed imports are safe.

## Verification

- packaged_cli_repair_restores_split_targets: two replicas each holding a different object, empty local
  store; repair reports repaired=2, unavailable=0, and a follow-up pull from each replica moves 0 records
  (local store is genuinely current).
- packaged_cli_repair_reports_unavailable_targets: a target held by no replica yields unavailable=1 and
  exit 3 while the available target is repaired.
- objpeer_cli 16/16, peer 15/15, controls 10/10, protocol_vectors 1/1, showcase 5/5; Clippy -D warnings
  and fmt clean.

## Why it matters

Operators had to compose audit and sync themselves and decide which replica holds which target; repair
does that selection and pulls only what is missing, from a replica that has it. It is the single-verb
fleet maintenance operation on top of the measured primitives.

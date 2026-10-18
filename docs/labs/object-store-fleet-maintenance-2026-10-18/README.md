# Fleet maintenance demo: audit, failover sync, repair  2026-10-18

The current-state backlog asked for an audit -> failover -> repair demo on real artifacts ("only
local unit tests exist"). This adds `crates/uor-r4-object-store/tools/fleet_maintenance.sh`, a
host-only demo that runs the real CLI against two real replicas.

## What the demo does

Two loopback replicas hold overlapping artifact sets: A has objects 1,2 and B has 2,3,4. The
script then:

1. **Audit** a fresh client against both replicas with `verify --targets ... --addr A --addr B --json`:
   4 checks, 0 verified, 4 stale, 3 unknown (the partition itself), exit 2 because unknown selectors
   are present. It asserts the real JSON counts, not substrings.
2. **Failover sync**: kill replica A; sync the three targets B serves with a dead first address and
   B second (`--addr 127.0.0.1:1 --addr B --retries 1`). The CLI reports the dead replica,
   uses B, writes 3 artifacts, and the script checks those three output files.
3. **Repair**: restart A; repair all four targets against both replicas. The client already holds 3,
   so `repair --json` reports current=3, repaired=1 (object 1 from A), unavailable=0, failed=0.
4. **Final audit**: verify all four targets against both replicas. Because the replicas are a
   partition, the JSON aggregate `verified_all` is only 1 (AND across replicas); the script checks
   the stronger operational predicate: every target is `verified` on at least one replica, with
   stale=0 and mismatch=0.

## Measured transcript (2 MiB total local artifacts, loopback)

    FLEET_DEMO artifacts=4 storage=loopback
    FLEET_DEMO replicas=2 a_has=1,2 b_has=2,3,4
    FLEET_DEMO audit_before checks=4 verified=0 stale=4
    {"artifacts":3,"nodes":349,"record_bytes":16641,"wire_bytes":21707,"replica":"127.0.0.1:...","output_dir":"..."}
    FLEET_DEMO failover_sync artifacts=3 skipped_object=1
    {"replicas":2,"checks":4,"current":3,"repaired":1,"unavailable":0,"failed":0,...}
    FLEET_DEMO repair current=3 repaired=1 unavailable=0 failed=0
    {"replicas":2,"checks":4,"verified_all":1,"stale":0,"unknown":3,"mismatch":0,...}
    FLEET_DEMO audit_after checks=4 verified_on_some_replica=4 mismatch=0
    FLEET_DEMO_RESULT audit_before=stale*4 failover_artifacts=3 repair_repaired=1 final_verified=4

## Checks and semantics

- `verify` exits 2 when a selector is unknown and 3 when stale targets exist; the demo treats
  those as valid outcomes and asserts the JSON (the exit codes are the CLI contract).
- Replicas are started with an explicit `--sessions 8` budget so the multi-step demo does not
  exhaust a one-session server; the script kills both replicas in a trap.
- All checks are fail-closed: wrong artifact counts, missing synced files, repair counts other than
  3/1/0/0, stale/mismatch on the final audit, or any command failure abort the demo.
- Workdir is a gitignored `local/fleet-demo.*` removed by the trap.

## Limits

- Loopback and synthetic local artifacts; this is an operational demonstration, not a WAN or
  multi-host measurement. Real fleet runs still need real endpoints and their own admission.
- The demo partitions targets across replicas on purpose; it does not claim every replica holds the
  full set. `verified_all` is an AND across replicas and is reported, but the operational
  predicate is union coverage.
- Host-only; no pod, no model compute.

No protocol or source behaviour change; the tool only drives the existing CLI.

# Peer replication: deduplicated DAG nodes are requested once

Date: 2026-10-17. Lab: deepseek. Branch: deepseek/object-store-peer-dedupe. Follows #2584.

## What changed

The hardened peer pull rejects any requested id returned twice (correlation check). The exact object
store deduplicates records, so identical repeated content makes the FBS tree a DAG: the same missing node
can be reached through several paths, and the missing-id list contained duplicates. A delta pull whose
changed region repeated content therefore failed with IntegrityFailure on a valid response.

Fix: missing_nodes deduplicates the requested id list (preserving order) before batching. The strict
response correlation remains: each id is requested once, the server returns each once, and duplicates in a
response are still rejected.

## Verification

- New regression test deduplicated_dag_nodes_are_requested_once: a periodic 2 MiB object with a
  128-atom edited region (repeated subtrees) completes and rebuilds exactly.
- Peer suite 7/7, package 30 passed / 0 failed, Clippy -D warnings clean, fmt clean.
- The bug was found by running the review-repo objpeer artifact test against the hardened main,
  not by the package tests; the regression now covers it.

## Next

Driver-side sync DoS/limits, multi-peer fan-out, and a cross-machine objpeer run remain.

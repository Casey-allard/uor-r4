# Peer protocol specification v1 (UOROBJ01)

Date: 2026-10-17. Lab: deepseek. Branch: deepseek/object-store-protocol-spec.

## What changed

- Added crates/uor-r4-object-store/PROTOCOL.md: a byte-accurate specification for porting the
  replication protocol to other runtimes. It covers the object/atom/FBS model, the exact custom
  BLAKE2b-96 node-id function (128-byte zero-padded block, counter = logical input length, 12 rounds,
  12-byte digest), record wire sizes (45/49), the UOROBJ01 manifest layout and base-3 frontier rule,
  frame kinds 20..=25, pull/verify/verify_series/pull_series client flows, serve/serve_many semantics
  including DONE/EOF/ERROR behavior, default limits, error taxonomy, trust boundaries, a porting
  checklist and the reference file map.
- Documentation only; no code, behavior, format or limit changes.

## Verification

- The specification was written against the current source (hash.rs, record.rs, manifest.rs, peer.rs,
  store.rs) and cross-checked with the merged protocol behavior: message kinds, manifest bytes and
  record layout are shared by the review-repo driver, and interoperability is tested in both
  directions.
- Package tests are unchanged by this docs-only change (peer 15/15, objpeer_cli 8/8, controls 10/10,
  showcase 5/5 at the previous commit); Clippy and fmt unaffected.

## Why it matters

The protocol can now be implemented outside Rust without reverse-engineering the source: the
compatibility-critical details (custom node-id padding/counter, canonical manifest bytes, frontier
rule, correlation and atomic-import requirements) are stated explicitly. This is the prerequisite for
the container/VM/MVM surfaces and for a non-Rust registry or fleet agent.

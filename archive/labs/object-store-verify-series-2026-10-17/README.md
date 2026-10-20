# Batch verification on one session

Date: 2026-10-17. Lab: deepseek. Branch: deepseek/object-store-verify-series. Follows #2590/#2591.

## What changed

- verify_series(store, requests, reader, writer) checks many objects on one connection: it sends a
  MANIFEST_REQUEST per requested selector, checks pins and local closure completeness, keeps the
  session open between checks and sends DONE once at the end. Per-check outcomes are
  Verified{missing}, Stale{missing}, Unknown (selector not present) or Mismatch (pin).
- Unknown selectors no longer terminate a serve session: serve_many writes the kind-25 error frame and
  continues, so one unknown selector cannot abort a batch audit.
- CLI: verify accepts repeated --select-sha256 and uses one session; it prints VERIFY i=<n> lines plus
  VERIFY_SERIES checks/verified/stale/unknown/mismatch/wire_bytes, exits 2 if any selector is unknown,
  3 if any artifact is stale or a pin mismatches, and 0 only when every check verifies. pull uses the
  first selector for compatibility.

## Verification

- Peer test verify_series_checks_several_objects_on_one_session: one connection checks a present object
  (Verified), a missing object (Stale with missing > 0) and an unknown selector (Unknown).
- CLI test packaged_cli_batch_verify_uses_one_session: one server session, two present objects and one
  stale selector -> VERIFY_SERIES checks=3 verified=2 stale=1, exit 3.
- Unit 7/7, peer 12/12, objpeer_cli 6/6, controls 10/10, showcase 5/5; Clippy -D warnings and fmt clean.

## Why it matters

The cross-machine fleet pass measured 226 B per check but about 564 ms per check, dominated by opening
a connection rather than bytes. Batch verification amortizes that setup: one session, one round trip
per check instead of a full connection handshake per check. The wire cost is unchanged.

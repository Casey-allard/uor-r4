# Read-only retries for verify and fleet audit

Date: 2026-10-17. Lab: deepseek. Branch: deepseek/object-store-verify-retry. Follows #2599.

## What changed

- objpeer verify (and therefore the fleet audit, which is verify --targets --json) now honours --retries N
  like pull and sync. Each attempt reconnects and re-runs the whole pipelined series; retryable I/O,
  schema or integrity failures print retry n/N and retry, while BindingMismatch, UnknownObject and
  LimitExceeded stop early.
- Read-only verification retries are inherently safe: no records are fetched, no store state changes,
  and the series is deterministic.
- No protocol change.

## Verification

- New CLI test packaged_cli_verify_retries_after_a_cut_connection: a proxy cuts the first connection after
  a partial frame and bridges the second to the real server; verify --targets --retries 2 succeeds with a
  retry line and a verified=1 audit.
- Suite: objpeer_cli 10/10, peer 15/15, controls 10/10, protocol_vectors 1/1, showcase 5/5; Clippy
  -D warnings and fmt clean.

## Why it matters

Fleet audit is the read-only CI/registry operation; a dropped connection on one of hundreds of targets
should not fail the whole audit. With this, all four CLI flows (pull, sync, verify/audit, serve) survive
a flapping link under --retries.

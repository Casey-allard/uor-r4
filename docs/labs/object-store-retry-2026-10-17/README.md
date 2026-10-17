# Reconnect and retry for real networks

Date: 2026-10-17. Lab: deepseek. Branch: deepseek/object-store-retry.

## What changed

- objpeer pull and sync accept --retries N (default 0, opt-in to preserve deterministic failure). Each
  attempt reconnects and restarts the transfer; attempts stop early on non-retryable errors
  (BindingMismatch, UnknownObject, LimitExceeded). Retryable I/O, schema or integrity failures print
  retry <n>/<N> after <error> to stderr and try again.
- Safety: pull imports records only after the object digest matches, and the store is saved after a
  successful pull, so a failed attempt leaves the on-disk store unchanged. sync retries the whole target
  series; objects already imported in memory on a failed attempt are re-verified cheaply on retry.
- No protocol change: the wire contract, manifest and exit semantics are unchanged.

## Verification

- New CLI test packaged_cli_retries_after_a_cut_connection: a proxy accepts the first connection and cuts
  it after a partial frame; the second connection is bridged to the real server. objpeer pull --retries 2
  succeeds, prints retry 1/2, and writes byte-identical output.
- Suite: objpeer_cli 9/9, peer 15/15, controls 10/10, protocol_vectors 1/1, showcase 5/5; Clippy
  -D warnings and fmt clean.

## Why it matters

Real links flap, VPNs reset and pods disappear; before this, any dropped connection failed a transfer.
Anchored, atomic pulls make retry safe, and the reconnect loop is the minimal realistic robustness
layer a fleet updater needs.

# Pinned cold join: objpeer join

Date: 2026-10-17. Lab: deepseek. Branch: deepseek/object-store-join.

## What changed

- New objpeer join command for the cold-join path: it reads a local decompressed artifact, verifies
  its SHA-256 and length against --expect-sha256/--expect-bytes before touching the store, then folds and
  saves the store atomically. A wrong pin exits 2 (BindingMismatch) and writes nothing.
- Output: JOIN object_bytes/roots/records/sha256/store/millis (JSON with --json).
- No protocol or wire change; compression stays outside the protocol (download compressed, decompress
  locally, then join).

## Verification

- packaged_cli_join_pins_a_local_artifact: pinned join succeeds, store exists, second join is idempotent.
- packaged_cli_join_rejects_a_wrong_pin_without_writing_the_store: exit 2, no store file, BindingMismatch.
- Package: 57 passed / 0 failed; Clippy -D warnings and fmt clean.

## Why it matters

Cold joins are a documented loss (5,725,408 B cold store pull vs 39,920 B compressed for the 2 MiB
artifact). join makes the supported recipe explicit and pinned: compressed download, local fold on
match, then warm deltas. The next review step is tools/cold_join.sh to measure both paths end to end.

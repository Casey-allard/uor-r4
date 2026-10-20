# Fleet protocol docs and stale-verification coverage

Date: 2026-10-17. Lab: deepseek. Branch: deepseek/object-store-fleet-docs. Follows #2590.

## What changed

- objpeer usage now lists verify, repeated --object, --select-sha256, the --expect-* pins, --sessions
  and --concurrent.
- peer module docs describe fleet selection: a 32-byte manifest selector, serve_many over several
  objects from one store, verify without fetching or rebuilding, and Error::UnknownObject.
- New stale-verification tests: the library verify reports missing > 0 and verified=false for an empty
  store; the CLI verify exits 3 and prints verified=false for a stale store.

## Verification

- peers 11/11, objpeer_cli 5/5, controls 10/10, showcase 5/5; Clippy -D warnings and fmt clean.
- No functional protocol change; this closes the docs and the untested stale branch of verify.

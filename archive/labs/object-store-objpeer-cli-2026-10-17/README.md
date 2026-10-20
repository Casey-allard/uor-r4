# Packaged objpeer replication CLI in the object store

Date: 2026-10-17. Lab: deepseek. Branch: deepseek/object-store-objpeer-cli.

## What changed

- Added crates/uor-r4-object-store/src/bin/objpeer.rs: the content-addressed artifact replication CLI
  (info / serve / pull) built directly on the merged Store and peer APIs. It persists stores between
  runs, supports multiple sessions (--sessions) and concurrent serving (--concurrent), and prints
  object bytes, node/record/wire counts and the verified digest.
- Added tests/objpeer_cli.rs: two real processes replicate a delta over loopback and the output is
  byte-identical; the info command folds and reports a local artifact.
- No new dependencies; the binary uses std and the crate only. This ships the product with the store
  crate instead of only from the review repository.

## Verification

- cargo test -p uor-r4-object-store: 33 passed / 0 failed (2 new CLI tests), Clippy -D warnings clean,
  fmt clean.
- The review-repo measurements used the same CLI source: 52.5 MB weight revision as 2,504 B local and
  2,509 B cross-machine wire; verification-only 220 B; R=12 fan-out 38,508 B total.

## Next

An `--expect` object pin on the CLI (using the merged ExpectedObject anchor), and eventual replacement
of the review-repo copy by this packaged binary.

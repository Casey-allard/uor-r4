# Fleet sync: pull a target series on one session

Date: 2026-10-17. Lab: deepseek. Branch: deepseek/object-store-pull-series. Follows #2594.

## What changed

- pull_series(store, &[PullRequest], reader, writer) pulls several selected objects on one session:
  each object is fetched, verified and imported before the next, and one DONE closes the session after
  the last object. Results are (manifest, stats) per request in order.
- pull_selected is now pull_one(send_done=true); the internal helper also supports the series mode that
  defers DONE.
- CLI: objpeer sync --addr A --targets FILE --out-dir DIR pulls the whole target set over one
  connection and writes each artifact to DIR/<sha256>.bin; --json prints artifacts/nodes/record_bytes/
  wire_bytes/output_dir, otherwise SYNC lines plus SYNC_SERIES are printed. The store is loaded and
  saved once for the whole set.

## Verification

- Peer test pull_series_fetches_two_objects_on_one_session: two objects pulled on a single accepted
  session, both rebuilt exactly, both verified.
- CLI test packaged_cli_sync_targets_into_directory: a commented targets file syncs two artifacts into
  the output directory with matching SHA-named files and JSON artifacts=2.
- Unit 7/7, peer 15/15, objpeer_cli 8/8, controls 10/10, showcase 5/5; Clippy -D warnings and fmt clean.

## Why it matters

A node updating a registry set currently pays a connection, store load and store save per artifact.
pull_series pays those once for the whole set, which is the same amortization that made batch verify
9.5x faster across machines. It is also the primitive a fleet updater needs: select a set, pull it in
order, write the artifacts.

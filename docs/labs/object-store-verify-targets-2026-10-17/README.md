# Fleet audit targets file and JSON output

Date: 2026-10-17. Lab: deepseek. Branch: deepseek/object-store-verify-targets. Follows #2593.

## What changed

- objpeer verify accepts --targets FILE: one 64-hex selector per line, blank lines and # comments
  ignored, selectors appended after any repeated --select-sha256. Invalid lines fail with file and line
  number.
- --json prints one machine-readable object (checks/verified/stale/unknown/mismatch/wire_bytes plus a
  results array with per-check status, missing count and selector) instead of the human VERIFY lines;
  exit codes stay 0 all verified, 3 stale or pin mismatch, 2 any unknown selector.
- The batch is pipelined by verify_series, so hundreds of targets cost about one round trip per window.

## Verification

- New CLI test packaged_cli_verify_targets_file_with_json: a targets file with a comment and two
  selectors verifies two present objects in one session and emits valid JSON with checks=2, verified=2.
- Suite: objpeer_cli 7/7, peer 14/14, controls 10/10, showcase 5/5; Clippy -D warnings and fmt clean.

## Why it matters

This is the CI/registry shape of fleet verification: a job consumes a target list, audits it in one
pipelined connection, and gets a parseable summary plus an exit code. It removes argument-length limits
and lets tooling drive the audit without screen-scraping human output.

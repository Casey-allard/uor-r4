# Laptop/reap archive verification before pod destruction  2026-10-18

The store path (`down --to-store`) got a read-back in #2629, but the **default laptop archive path** and
`reap` still destroyed the pod without verifying the copy; `reap` even deleted when archiving failed.
The qualification session's only durable product is its receipts, and `down` without `--to-store` is the
default path, so this closes the remaining custody hole.

## Change

- `scripts/pod/uor-pod` gains `verify_laptop_archive POD DEST DIRS`: after the tar copy it computes on
  the pod a sorted per-file SHA-256 manifest over the same relative paths (`uor-r4/jobs ...`, count +
  digest of the manifest) and compares it with the same computation on the laptop archive. Any count or
  digest mismatch warns `ARCHIVE VERIFICATION FAILED ... pod retained` and returns 1, so `down` aborts
  before `pod_delete`.
- The `vast-archive` ledger event now records `verified: true`, `files`, and `manifest_sha256` (the
  digest of the sorted per-file manifest) in addition to `dest`/`kb`.
- `reap` no longer deletes when `vast_archive` fails: it warns and retains the pod, leaving an
  operator to archive by hand and run `down --confirm-archived`.
- The dry-run suite's fake SSH **executes the actual remote count/digest command** against the fake
  workspace (with a `sha256sum` shim for macOS), not a synthesized answer, and supports
  `FAKE_ARCHIVE_MISMATCH=1`; a new case asserts `down` refuses and retains the pod on a digest mismatch,
  and the existing success case now has **two files** and asserts `verified:true, files:2` in the ledger.
  This closes the review finding that the earlier stub masked a NUL-list `wc -l` count of 0.

## Verification (host-only, no pods, no model compute)

    bash scripts/pod/tests/uor-pod-dryrun.sh      # 373 passed, 0 failed
    # includes: "down refuses when the laptop archive verification fails" (pod retained)
    #           "the verified archive is in the ledger" (.verified == true, .files == 2)
    #           "an empty results dir verifies 0 files" (zero-file case)

The mismatch case is injected, not a real corrupted tar; the success case exercises the real manifest
comparison on the fake workspace, and a separate empty-results case pins the zero-file path (0 == 0,
xargs -0r on both sides). The `reap` failure path is covered by code review, not a dedicated test.

## Limits

- The digest is over the same relative file set (sorted per-file SHA-256 manifest), not a second full
  payload download: it detects missing/extra/reordered files and byte changes that alter the manifest,
  which is the repeated failure mode of a truncated tar.
- The laptop path remains capped by `VAST_ARCHIVE_MAX_MB`; larger results still go through
  `--to-store` (already verified in #2629).
- `reap` now prefers retaining an unleased pod over deleting unverified data; an operator must still
  run the archive/down by hand. This is a safety change, not an automated archive service.
- Host-only; no protocol or model-path change.

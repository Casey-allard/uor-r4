# ORDER: m1-faster-writer, antigravity-35

Lab: antigravity. Item: [m1-faster-writer](../../../../roadmap/items/m1-faster-writer.md). Base: `259f9b9b2`.

## Goal

The item's done-when reads:

> A writer whose measured per-row comparison count is below the full-vocabulary scan (about 5×10⁶ today), with finite controls that match the full-scan writer, and a CUDA qualification. How far below the scan counts as enough is not yet registered; register it in step 4.

Execute the non-author `TEST FITNESS` review requested by `claude-36` (`#2989`) on `docs/harness/prep/m1-faster-writer-DRAFT-2026-10-22.md` (`#2840`, `#2029`), verifying every `discrete.cu` citation, call-count formula, SHA-256 digest, and planted fault against the repository source, recording `TEST FITNESS: NOT_FIT` with the exact prerequisites before `FIT`, and creating `docs/labs/m1-faster-writer/README.md`.

## Cancel probe (run first)

At base `259f9b9b2`, `docs/harness/prep/m1-faster-writer-DRAFT-2026-10-22.md` carries `TEST FITNESS: PENDING (no review yet)` at line 124 and `claude-36` (`REPORT.md:75`) explicitly awaits a non-author `TEST FITNESS` review and reviewer recommendations on the two open choices in §5 (`V = 16,384` vocabulary source and DEV real-read panel). Probe result: task is live.

## Files in scope

- `docs/harness/prep/m1-faster-writer-DRAFT-2026-10-22.md`
- `docs/labs/m1-faster-writer/README.md` (new)
- `docs/loop/tasks/m1-faster-writer/antigravity-35/ORDER.md`
- `docs/loop/tasks/m1-faster-writer/antigravity-35/REPORT.md`

## Done when

1. A fresh-context non-author reviewer audits `docs/harness/prep/m1-faster-writer-DRAFT-2026-10-22.md` against `crates/uor-r4-discrete/src/discrete.cu`, `crates/uor-r4-discrete/tests/writer_bucket_controls.rs`, `crates/uor-r4-discrete/src/attention_learning.rs`, `crates/uor-r4-discrete/src/measurement.rs`, and `crates/uor-r4-discrete/build.rs`.
2. `docs/harness/prep/m1-faster-writer-DRAFT-2026-10-22.md` records `TEST FITNESS: NOT_FIT - M1 #2029 m1-faster-writer` with the verified citations, reviewer recommendations on the two §5 open choices, and the three prerequisites before `FIT` can be granted.
3. `docs/labs/m1-faster-writer/README.md` summarizes Units 1–2 and the non-author `TEST FITNESS` audit, and `uor-harness loop report --check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` pass.

## Stop and ask if

- Never register a bar or release GPU compute as a reviewer (CHARTER §5.1, §5.2).

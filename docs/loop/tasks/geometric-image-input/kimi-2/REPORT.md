# REPORT: repair desktop vision lockfile after fresh-main verification

Lab: kimi. Item: [geometric-image-input](../../../../roadmap/items/geometric-image-input.md) (#3619), repair of merged PR #3639. Order: [ORDER.md](ORDER.md). Base: `1f0c6128bd2a067e50bc1326ad4c704a0d4b8a86`, tree `e1dc1d645ceafaceef8187b247a8cc72dc90a7c9`. The PR is the exact final head/tree and external-delivery identity record.

## What did not work

- Fresh `main` failed `cargo build --manifest-path src-tauri/Cargo.toml --locked`: `uor-r4-vision` was locked to retired `hex`, `serde`, and `sha2` dependencies while its manifest uses `blake3` and `rayon`. The desktop verifier's offline fallback regenerated the lockfile and passed, so the real desktop contract was green while the ordinary locked build remained broken. The failing-first output is preserved in local delivery evidence.
- `cargo update -p uor-r4-vision --offline` also rewrote an unrelated `syn 3.0.9` edge to `syn 2.0.119`. That generated attempt was discarded, not expanded or hidden. Restoring the base lockfile and using the offline build resolver produced only the required vision dependency correction.

## What changed

Cargo regenerated `src-tauri/Cargo.lock` with `uor-r4-vision` depending on `blake3` and `rayon`, replacing `hex`, `serde`, and `sha2`. The lockfile diff is exactly two insertions and three deletions; no source, dependency version, model, tokenizer, issue state, or Pages content changed. This task's ORDER and REPORT are the only other files.

## Measurements

- RED at base: `cargo build --manifest-path src-tauri/Cargo.toml --locked` refused to update the lockfile.
- GREEN after regeneration: the same locked build finished without an offline rewrite.
- `cargo test -p uor-r4-vision`: 9/9 PASS.
- `node web/studio/verify_desktop_client.mjs`: real binary contract PASS, with no stale-lockfile message, 32 streamed tokens, 38 invoked commands, and zero remote requests.
- `git diff --check`: PASS. Repository gates, exact-head review, GitHub checks, protected queue, and fresh-main locked-build verification remain external delivery records.

## Checks run at the head

Implementation verification used `7e0f7334cf18d4e6cae3a3f0658640ee9501c63f`; the final REPORT-only head and tree are the PR identity. Repository gates, exact-head review, GitHub checks, protected queue, and fresh-main verification are external delivery records, not preclaimed passes.

## Skills used

Systematic debugging, failing-first verification, Cargo-generated lockfile repair, scope control, and protected exact-head delivery. No GPU, paid compute, new dependency, source edit, or production publish was used.

## Still open

The repair must pass its independent exact-head review and protected delivery, then the ordinary locked build must be rerun from fresh `main`. PR #3639 remains merged history; this follow-up does not rewrite it. #3500 and #3569 remain open. No visual-understanding, useful-chat, or M4/M7 completion claim is made.

## Plain summary

The merged image implementation worked, but its desktop dependency lock was stale. This repair updates only that generated lock entry so an ordinary offline locked desktop build works without a fallback.

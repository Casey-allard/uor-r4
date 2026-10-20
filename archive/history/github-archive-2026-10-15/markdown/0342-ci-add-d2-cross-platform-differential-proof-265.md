# #342 — ci: add D2 cross-platform differential proof (#265)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-02T02:08:03Z
- **Updated:** 2026-10-09T14:17:14Z
- **Closed:** 2026-08-02T02:20:31Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/342
- **Merged:** 2026-08-02T02:20:31Z
- **Merge commit:** 39924b8fa63692d30b5866fcb3c13291965309a4
- **Base / Head:** main / UOR-Foundation:issue-265-d2-differential
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- add a D2 differential compile script for the immutable SmolLM2-135M-Instruct revision
- run canonical compilation on Ubuntu x86_64 and macOS 14 arm64 in merge-group/main CI
- compare complete source and emitted-bundle byte manifests across platforms
- document the new evidence path without changing or re-pinning shipped fixtures

## Acceptance status

This advances #265 item (a): the CI job will prove whether the canonical mode emits identical bytes for the pinned source and 1,000-token rehearsal target. The canonical fixture re-pin and changing the historical Linux κ check from informational remain maintainer decisions after the differential result is reviewed.

## Validation

- `bash -n scripts/d2_differential_compile.sh`
- workflow parsed successfully with PyYAML
- `git diff --check`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo check -p uor-r4-graph-format --no-default-features`
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc`

Refs #265

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


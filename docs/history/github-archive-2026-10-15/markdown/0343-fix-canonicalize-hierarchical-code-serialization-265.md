# #343 — fix: canonicalize hierarchical code serialization (#265)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-02T02:21:58Z
- **Updated:** 2026-10-09T14:17:14Z
- **Closed:** 2026-08-02T02:36:07Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/343
- **Merged:** 2026-08-02T02:36:07Z
- **Merge commit:** c597ed31c18ebbe8ab9c1881efa7b68bc6ba2194
- **Base / Head:** main / UOR-Foundation:issue-265-d2-hierarchical-fix
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- replace the emitted bundle’s hierarchical-code HashMap with BTreeMap
- add a regression assertion for canonical JSON serialization
- fix the concrete cross-platform mismatch found by PR #342’s merge-group D2 run

## Evidence

PR #342 compiled successfully on both macOS arm64 and Linux x86_64. Its comparison isolated the only output mismatch to `hierarchical_codes.json`; all other source and bundle files matched. The mismatch was caused by HashMap iteration order during JSON serialization.

This follow-up makes that emitted structure ordered, so the D2 differential manifests can compare equal.

## Validation

- `cargo fmt --all -- --check`
- `cargo test -p uor-r4-core --test hierarchical_compiler --offline`
- `git diff --check`

Follow-up to #342; refs #265

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


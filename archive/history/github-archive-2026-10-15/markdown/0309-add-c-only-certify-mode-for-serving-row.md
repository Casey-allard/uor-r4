# #309 — Add C-only certify mode for serving row

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-31T21:21:16Z
- **Updated:** 2026-10-09T14:16:54Z
- **Closed:** 2026-07-31T21:44:58Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/309
- **Merged:** 2026-07-31T21:44:58Z
- **Merge commit:** bf9a63a5a6131d8accf0e763be4664f9e030493a
- **Base / Head:** main / UOR-Foundation:agent/certify-c-only-299
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- add `R4_CERTIFY_C_ONLY=1` to run only the C serving certification row
- skip reference-checkpoint loading and the unrelated full certificate
- preserve the existing explicit measured-row or recorded-skip behavior

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test -p uor-r4-wasm-router --offline` (94 BDD scenarios, all passing)
- smoke-tested both available local compiled bundles; neither is a valid TLA6 measurement fixture

This supplies the isolated execution path requested by #299. The actual TLA6 remeasurement remains pending until a loadable TLA6 bundle is available.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


# #815 — Release pipeline: tag-triggered dual-frontend builds + verified install-release (#741)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-19T01:22:02Z
- **Updated:** 2026-10-09T13:09:05Z
- **Closed:** 2026-08-19T01:42:42Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/815
- **Merged:** 2026-08-19T01:42:42Z
- **Merge commit:** d1db980c89926ee1b5efee2e38bc0ca7c0e08097
- **Base / Head:** main / UOR-Foundation:release-pipeline-741
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Implements the #741 maintainer decision (2026-08-19): **adopt the #655-D package as the release artifact and build the pipeline now** — both frontends, GitHub Release distribution, explicit verified fetch, and `vX.Y` tags binding code SHA + bundle component digests + contract version. **Publishes nothing until a tag is cut**, and even then the release is a draft until the maintainer publishes. Stacked on #814 (decode flip) → #813 (decode machinery); the last commit is this PR's.

- **`.github/workflows/release.yml`** — on `v*` tag push: draft release (notes bind tag → commit SHA + inference-contract version), native CLI builds for linux x86_64 + macOS arm64, the wasm frontend (same `wasm-pack` build the Pages deploy runs), `.sha256` beside each. The model bundle is attached by the maintainer from a locally packaged compile — CI cannot compile the model.
- **`r4 install-release --tag vX.Y`** (`src/release_install.rs`) — downloads `release-bundle.json` + `release-bundle.tar.gz`, **hard-verifies every declared component digest** against extracted bytes (deliberately stricter than the advisory serving-time check), refuses archives with any unattested entry or symlink, never overwrites an existing install, installs atomically with the sidecar beside the bundle so #655-C1c serving verification sees the same manifest. Fetch is explicit-only — per #655's "first request must not silently download" scope line. Fetcher is a trait: `curl` in production, fixtures in tests (the network never runs in tests).
- **`docs/RELEASE_PIPELINE.md`** — the versioning convention, exact maintainer release steps (bundle candidate per the decision record: the `smollm2-360m-broad` compile), and the verified-fetch contract.

## Tests

Verified-install happy path (exact asset URLs, sidecar installed beside the bundle, staging cleaned) · tampered component bytes refused · smuggled unattested file refused · existing install never overwritten · repo/tag/name shape validation rejects before any fetch. Workspace clippy `-D warnings` clean, fmt clean, bin + lib suites pass.

## What this does NOT do

No serving default or canonical identity change (that is #655-F, separately gated); no auto-publish; no CI model compilation.

## Refs

#741 (decision brief issuecomment-5335803275, adoption issuecomment-5335968482) · #655-D (packager/sidecar this pipeline distributes) · #655 precondition 1 (a fresh clone can now reach a real bundle via one explicit verified command once a release exists)

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_016X39CfJSLCU4KhjNP2XXTW


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


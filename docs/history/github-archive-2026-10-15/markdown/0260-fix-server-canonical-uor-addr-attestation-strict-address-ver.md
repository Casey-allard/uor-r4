# #260 — fix(server): canonical uor-addr attestation + strict address verification (#256, #257)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-28T13:09:24Z
- **Updated:** 2026-10-09T13:09:12Z
- **Closed:** 2026-07-28T13:20:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/260
- **Merged:** 2026-07-28T13:20:55Z
- **Merge commit:** 705b763db7aab223f2c9220784bbee078e84841d
- **Base / Head:** main / UOR-Foundation:issue-256-uor-conformance
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #256. Closes #257.

## One PR for two issues — deliberately

The envelope (producer) and `/api/uor/verify` (checker) share the address computation. Moving the checker to strict canonical equality while the producer still emits raw-serialization hashes (or vice versa) breaks every producer→verifier round-trip in the window between two PRs. They move atomically.

## What

**Envelope (#256)**: `uor_address`/`attestation_cid` now go through `uor_addr::json::address_blake3` — JCS canonicalization then blake3, so byte-different encodings of the same JSON value get one label (the property the raw hash lacked). `artifact_cid` is the real content address of the artifact the serving cascade resolves (mtime-cached; candidate list factored into a shared `R4G1_ARTIFACT_CANDIDATES` const used by startup loading too) — or omitted entirely. Dropped: placeholder `store_cid` (it named a section of the same file — structurally not a separate CID) and self-asserted `verify_result: "Verified"` (the verify endpoint is the verifier). The audit trace's `uor:cbor:blake3:{curvature-float}` fabrication is replaced with a canonical label over the audit content.

**Verifier (#257)**: typed syntax gate — `empty_address`, `unsupported_axis`, `digest_length_invalid`, `digest_not_lowercase_hex` all reject with HTTP 400 and a named reason (previously an **empty address verified**, and any string merely **containing** the digest passed). Comparison is exact equality against the canonical expectation.

## Conformance vectors

The tests are written as the first seed vectors for the W2 suites: encoding-independence (two byte-different encodings, one label), the negative-vector table with the canonical empty-subject case first, a substring-attack regression, and a producer/verifier round-trip under deliberately re-encoded bytes.

## Compatibility note

Envelope consumers relying on `store_cid`/`verify_result` fields or the raw-hash `uor_address` will see changed output — that is the point, but flagging for the dashboard.

## Gates

All four green locally (macOS arm64, pinned 1.97.1, offline; run alongside the in-progress D3 observation without interference).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_017tu2Y4AGZvcgM4vmoSz5x6


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


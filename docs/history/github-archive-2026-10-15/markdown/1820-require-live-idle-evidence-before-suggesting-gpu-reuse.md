# #1820 — Require live idle evidence before suggesting GPU reuse

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T02:02:46Z
- **Updated:** 2026-10-09T13:17:47Z
- **Closed:** 2026-10-07T02:03:59Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1820
- **Merged:** 2026-10-07T02:03:59Z
- **Merge commit:** c91ab60f131fbe235547c7ce3c01cff71eb68fab
- **Base / Head:** main / UOR-Foundation:codex/pod-free-guard-20261007
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

`uor-pod up` could refuse allocation and direct a lab to reuse a GPU that was unleased but actively training. Its reuse precheck subtracted leased indices without checking occupancy, unlike `status`. This occurred with a Claude GPU at 99% utilisation and a live training process.

Reuse admission now uses the same live probe and GPU-state classification as status. Only explicitly unleased and non-busy indices count as free; unreachable, empty and malformed telemetry cannot establish reusable capacity. Existing spending caps, GPU policy and exceptions remain unchanged.

Validation: shell syntax and diff checks passed; frozen-source mocked suite passed **205 checks, 0 failures**, including utilisation, compute process, GPU lock, unreachable/empty/unknown telemetry and a genuinely idle positive control. No real pod or lease mutation occurred in these tests. Independent source review found no blocking issue. An earlier suite execution was invalidated by editing its script while Bash was reading it; that failed log is preserved and is not counted as a product regression. The unchanged rerun is the validation receipt.

References #820. This repair unblocks the already-built geometric source-state diagnostic in #1819.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6029314999 — 2026-10-07T02:03:00Z — Casey-allard

Exact-head self-review at e04f083415df9265da7e5f68ad87b2d94d82ab5c: bash -n on both touched scripts PASS, git diff --check PASS, frozen mocked dry-run suite exit 0 (205 passed / 0 failed), clean tree. Independent exact-head critic review PASS: production guard matches reviewed patch; unknown-telemetry fixture present; cap and policy logic unchanged. Canonical receipts /workspace/uor-r4/codex/pod-free-guard/dryrun-final.log (SHA256 1b80181e1b448b93ffe186ffa628a28e66f02d9790df42001c82606080cc4749), source.bundle (b372dd939a7cd3b870a40774b2f735191c08bad1b2fe301d8e8cff735167c45a). Earlier interrupted execution preserved separately and not counted. No actual allocation/lease mutations in tests.


---
_Comment/review ids in this file: 6029314999_


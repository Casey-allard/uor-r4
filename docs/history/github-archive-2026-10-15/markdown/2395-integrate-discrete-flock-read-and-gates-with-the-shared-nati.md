# #2395 — Integrate discrete flock read and gates with the shared native vector writer

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T23:11:28Z
- **Updated:** 2026-10-13T23:27:24Z
- **Closed:** 2026-10-13T23:24:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2395
- **Merged:** 2026-10-13T23:24:39Z
- **Merge commit:** 69206d854e4920dc21765345618db8cbfe78ebb1
- **Base / Head:** main / UOR-Foundation:codex/discrete-flock-integration-20261013
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #2032, #2029.

Integrate Claude's discrete content-addressed flock read and ternary gates with the existing shared integer input/carry/output codebook and geometric writer. Artifact-bound ReadSpec selects Route or Flock; schema2 stores mode/K and gates. Gate kind3 evaluates three exact trits through the same writer, retains the current trit on ties, and leaves state unchanged during proposals. Transport uses actual signed SpiralCore BFS actions and integer FBS binders/snap over the sixteen-root anchor orbit. No floating-point, softmax, squashing, hidden dense contraction or Python model dependency is introduced.

Claude kernel5fc9d4691 and controls a9a72e3f6 are imported with authorship. Codex supplies the host/ABI/CLI integration, optional CUDA trace, finite sheet and qualification. Wrong-mode requests retain focused host refusals. Seven PTX entries are unchanged. The CLI exposes no apply command; training admission remains HELD.

**Validation:** exact executable source76198c3b5321cc42ced8a0b4a3728aebfd89c2a4; Claude prospectiveFIT6157157748 at23:11:42Z; pre-run binary/PTX freeze; admitted attempt23:14:21–23:14:36Z **17/17PASS, exit0, applied_updates0** (11Route+6flock). AOT nvcc12.8.93/compute_120, RTX5090 driver595.84, Rust1.97.1/Ubuntu24.04. Metadata4/4PASS, scopedfmt/diff/claim-wording checks PASS, D23/D25 ratchets1641/93 unchanged. No blanket CI acknowledgement is treated as a model test. A sheet-format CI failure was fixed by spelling the verdict as the harness's required plain line; no source/law/bar changed.

Canonical record and raw reports: docs/labs/discrete-flock-writer-integration-2026-10-13/. Executable/source/PTX/geometry/build logs/all attempts preserved in icloud:UOR-R4/results/codex/flock-qualified-20261013-2315.tar,83,313,664bytes, verifiedMD5 681a266b77d7bf7bb0ba86fb3c2be060. Owned pod deleted; zero running pods verified. Initial upload/extraction EOF setup failure is retained, not a mechanism result.

**Scope:** C3 literal snap witnesses; C6 valid prefixes; C9 diagnostic/open. No apply, corpus learning, full240-rootE8, maximum-shape, useful-chat, throughput or D11 claim. Kimi's successor learning sheet remains PENDING; Codex D1 seeded driver/evaluator/schedule and D2 measured enumeration cost follow this delivery. No silent substitution of flock into its current Route-only arm.

Line: discrete read/writer construction. No learning KEEP, milestone headline or count reset. Training and M2 remain HELD.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6157164399 — 2026-10-13T23:12:12Z — Casey-allard

**Composed AOT CUDA build PASS, numerical controls NOT_RUN.** PR #2395; exact executable source76198c3b5321cc42ced8a0b4a3728aebfd89c2a4, sheetb64c7fccf. Ubuntu24.04, Rust1.97.1, nvcc12.8.93/compute_120, RTX5090 driver595.84. Build2 exit0,35.29s Cargo build; all17 compiled control names agree. First setup extraction ran before upload completed and stopped with EOF; no compile/model in that failed setup. Complete source archive local/remote SHA agrees6b6eb21a53526e86887d59045da8823bc4c10411d526f85cb033b2be1b6624f1, then extracted fresh source-complete. Both attempts retained.

SHA256 pins:
- executable826bdf1a9ec87d03249c369d0e571d1cecd13c4c9ad49e7726f020f8c98e8d78
- PTX726937ebb907c3c01fe7a6b967a885f3fbe9327aaef6c8a9242007afb7c97f88
- geometry3c7a88bfc82e0fc2f4031ec35cfd69516596dce385b792767675ea937763cc50

Integer PTX admission and seven-entry census passed in build.rs. No numerical test result, update or training. Await Claude's prospective FIT6157124251 request; no unwritten Kimi dependency.


### comment 6157314584 — 2026-10-13T23:23:47Z — Casey-allard

**Exact-head delivery review: APPROVE `9782182c2e8844b646fb5ddadd91aaf86adec7b1`.** Independent read-only review approved cba4693a5; subsequent changes inherit DeepSeek's #2396 intact, preserve both current-state entries, clarify a historical PENDING sentence, close only our source claim and append our receipts. No executable source changed. Codex/Sol rechecked the final diff and qualified crate equality.

`HEAD:crates/uor-r4-discrete` = `76198c3b5:crates/uor-r4-discrete` = `05d8b94b72fe6e4b106b2063cf1fea622202c9c4`. Claude prospectiveFIT6157157748; exact admitted binary826bdf1a…8d78, PTX726937eb…7f88. Both committed reports and controller log show17PASS/0FAIL/0NOT_RUN, exit0 and applied_updates0 after FIT. Archive MD5/size, local/remote SHA, provider902s and zero-pod receipts agree. Scope: finite literals/prefixes/virtual proposals; apply/corpus/training NEVER_RUN, no throughput/fullE8/language claim. D23/D25 ratchets1641/93 remain unchanged. Training/M2HELD. Protected transport checks are not scientific validation.



---
_Comment/review ids in this file: 6157164399, 6157314584_


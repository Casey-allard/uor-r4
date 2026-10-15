# #470 — perf(core): #469 lever A — per-record code sidecar, κ-keyed and fallback-safe

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-06T19:52:35Z
- **Updated:** 2026-10-09T13:09:10Z
- **Closed:** 2026-08-07T02:02:09Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/470
- **Merged:** 2026-08-07T02:02:09Z
- **Merge commit:** 3ceb786f136c8ad1efa3fbc77714477c4f8e9d83
- **Base / Head:** main / UOR-Foundation:i469-codes-sidecar
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #469 (lever A only; lever B — vectorizing `assign_for_bundle` — is κ-pinned and stays on its own branch behind a bit-identity proof).

**Measured on the 2.11M corpus:** instrument **624.9s → 38.9s (16.1×)**; the graded-code store section **604.7s → 19.6s (30.9×)**. Sidecar is 8,440,642 bytes for 2,110,111 records. The cold run writes it — the win is on every subsequent consumer, which is exactly our access pattern (one compile, many measurements).

**κ-neutral by construction, and demonstrated:** this caches an existing deterministic computation and never redefines it. `assign_for_bundle` and `code_plain` are byte-for-byte untouched. The two runs' report output is identical line for line apart from provenance and timing lines (`diff` clean) — on the real 2.11M corpus, not just the fixture.

**Safety is the point, not the speed.** A sidecar loads only if magic, version, stages, record count, artifact κ, corpus κ, exact file length, and a blake3 over the code block ALL match; anything else returns `None` and the caller computes as today. Writes are temp-file + rename so a partial container is never visible. The path taken is logged with both κs per the #450 precedent (`code sidecar: LOADED|MISS|WROTE …`). Corpus key is a content κ over the corpus columns — strictly broader than what the code derivation reads, so any doubt resolves as a MISS.

**Consumers wired:** the whole-corpus code pass in `evaluate_gate_c`, its per-row `code_plain` (served from the already-materialized index), the `capacity_scaling` instrument, and the graph-cli score command's store build — so a score run now pays the code pass zero times. Not wired: `derive_right_codes` (#446's right-context codes are a different quantity needing their own key).

**Tests (10, non-ignored):** read-back equals fresh `code_plain` for every record and cached `store_bytes` equals uncached; plus rejection tests for mismatched artifact κ, mismatched corpus κ, truncation, wrong stage count, wrong record count, bad magic/version, and a corrupted code block — each must fall back cleanly, never load a wrong code.

Placed in its own module (`code_sidecar.rs`) rather than inside `runtime.rs`, partly because `runtime.rs` is machine-scanned by the P-4 forbidden-arithmetic witness, and partly so the "no code is computed here" claim is inspectable in one file.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


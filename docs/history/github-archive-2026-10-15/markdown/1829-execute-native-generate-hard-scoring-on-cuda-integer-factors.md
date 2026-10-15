# #1829 — Execute native Generate hard scoring on CUDA integer factors

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T04:54:27Z
- **Updated:** 2026-10-09T13:17:13Z
- **Closed:** 2026-10-07T05:31:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1829
- **Merged:** 2026-10-07T05:31:14Z
- **Merge commit:** 89c7cec48e3c485cd0db24f02bd17c86132e4845
- **Base / Head:** main / UOR-Foundation:codex/device-generate
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Native Generate training previously recomputed its entire integer vocabulary field on the host before staging a floating anchor. This change evaluates the authenticated frozen H4 unary, ordered-pair and bias factors on CUDA using signed i64 accumulation. Coefficient-only, cached-state and reference forwards share the new dispatch; the CPU reference remains available.

One thread scores each vocabulary ID through the native relative-code table. The floating anchor is detached and the existing surrogate gradient is preserved. Device preparation binds the native payload and reports retained factor bytes. Per-call costs report state upload, kernel launches and the compatibility score download; the current public host score vector still downloads 8 bytes per vocabulary ID. Copy/context, token pooling and native evaluation remain host work. This is an offline training component, with no new serving math or transformer backbone.

Lab: Codex. References #820. Directly based on main; four files, 583 added lines, no run outputs.

Validation at exact head `04c2df16cdccca10a705473baa1f5592603e15cf`: changed-file formatting PASS; release CUDA-feature library check PASS; release test build PASS; five focused CPU tests and five explicitly executed CUDA tests PASS. CUDA covers all 120 roots at one/eight lanes, signed/asymmetric pair factors, sparse token holes, payload/state rejection, real retained-artifact score/anchor parity, full-choice gradients, eight-lane state adjoints and coefficient-only pool loss. Independent production/source and fixture reviews found no scoped blocker. Full `cargo fmt --all --check` fails only in eight files byte-unchanged from main; this is recorded separately from changed-file formatting.

Linux x86_64, RTX5090, driver570.195.03, CUDA12.8; executable SHA256 `281c7f0c3cf0f0960ed38c3623c4a475720b397ffcfe4636db54ab1ff3f77f24`. GPU log: `/workspace/uor-r4/jobs/codex/20261007T052814Z-device-generate-parity-gpu0.log`, terminal exit0. Build/CPU receipts: `/workspace/uor-r4/codex/device-generate/`. Earlier tokenizer and generic-Q4 boundary fixture faults are preserved as instrument defects, both repaired before final validation.

Three short retained-checkpoint scorer measurements show 8.23–9.13× CPU/CUDA ratios including score download. They exclude snapshot preparation, the floating gradient graph, source/context read, pool construction and whole fits; they establish no whole-training or language gain. All validation pods were released/deleted after their jobs. The ongoing six matched fits retain their original executable and conditions.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6031635528 — 2026-10-07T05:30:21Z — Casey-allard

Exact-head self-review at `04c2df16cdccca10a705473baa1f5592603e15cf`, independently reviewed numerical module SHA256 `f52020c30777ddd404a4d2130838487914f506acee48ea0e8312726ce8413dfe`.

- Changed-file rustfmt PASS (four files, `skip_children=true`); full `cargo fmt --all --check` exit1 only in eight UNCHANGED main files, with source diff verification retained in the receipt. No whole-workspace formatting PASS claimed.
- `cargo check --release -p uor-r4-training --features cuda --lib`: PASS17s. Matching release test build PASS102s, GPU masked on four CPU threads of the already-running training pod.
- CPU exact tests PASS5/0: coefficient-only field equivalence, sparse tokenizer admission, full signed-pair fixture/native admission, export/coefficient credit, cached-vs-reference scores/gradients.
- CUDA exact tests PASS5/0, no skipped/missing-device passes: all roots/signed pairs/sparse vocabulary; actual checkpoint32states×4096scores plus anchors; full-choice gradient parity; eight-lane hard-state adjoints; coefficient-only native-pool loss. Terminal exit0 at05:28:45Z. Driver570.195.03, CUDA12.8, RTX5090, Linux x86_64. Executable SHA256 `281c7f0c3cf0f0960ed38c3623c4a475720b397ffcfe4636db54ab1ff3f77f24` is checked inside the GPU wrapper.

Reviewed exact i64/Q24 accumulation and orientation, immutable factor buffer lifetime, authenticated snapshot payload, legal sparse IDs, allocation/download accounting and detach/adjoint separation. Snapshot freshness against later live-weight mutation remains the existing caller responsibility. The public score-vector download, Copy/read/pooling and CPU native evaluation remain explicit. No whole-path D11, attention/chat success or complete GPU residency claim. Existing healthy fits were not changed or canceled. Earlier two pre-kernel fixture faults and the partial e13 four-test result remain separately source-bound.

Ready for the protected merge queue at this exact head; required CI transport statuses are compatibility acknowledgements, not these executed tests.



---
_Comment/review ids in this file: 6031635528_


# #1837 — Use authenticated categorical H4 credit in native bank learning

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T09:42:33Z
- **Updated:** 2026-10-09T13:17:41Z
- **Closed:** 2026-10-07T09:44:19Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1837
- **Merged:** 2026-10-07T09:44:19Z
- **Merge commit:** e2f9ede073c8200b4471c63b777e170d72032db7
- **Base / Head:** main / UOR-Foundation:codex/native-categorical-context-credit-20261007
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Context learning in `PreparedBankGenerate` can currently use the legacy bridge pullback even when its hard forward operation is a frozen categorical H4 map. This adds an explicit opt-in for that authenticated map’s conditional query/source pullback, so offline learning can use the map that native inference actually executes.

The hard Source selection, physical occurrence alternatives, integer transport, full Copy/Generate alias pool, and default legacy behavior remain unchanged. The two pullback configurations are mutually exclusive, with tokenizer/protocol/lane/device checks. The categorical map stays frozen; the surrogate does not differentiate the discrete Source argmax or promise a descent direction.

Validation at exact library head `c992a7b3d75fcd0992da8143521eaf027725b11e` on Linux x86_64 / RTX 5090 / CUDA 12.8 / sm120:

- Changed-file formatting and diff checks passed; release CUDA-feature library check passed.
- Nine focused CPU tests passed; the explicit CUDA categorical adjoint/parity test passed (one executed, zero ignored).
- A separate retained RECORD harness at `686abd9e36fa0457b9c7e4bf12815a1fec1b8569`, with both library blobs identical to this head, passed an actual full-bank zero-update CUDA comparison. Its independent audit authenticated the seal, executable, all 16 condition configurations, and complete 4,096-token native alias vectors. Hard outputs agree; actual Context gradients are finite and active. There are six distinct row/prefix contexts, not 16 independent examples.
- Whole-workspace formatting has existing drift in unchanged main files; it is not counted as passing here.

The full-bank run made zero optimizer updates and took 20.322 seconds. It establishes hard-forward parity and gradient reachability, not learned attention, prediction gains or chat ability. The harness and run outputs remain on a RECORD branch and canonical storage; neither is included in this two-file PR.

References #820.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6035307259 — 2026-10-07T09:43:09Z — Casey-allard

Exact-head self-review + independent source/artifact review, head `c992a7b3d75fcd0992da8143521eaf027725b11e`, direct base main `4c3fe85005e703e4976c4509f4065dda21d870d3`.

Reviewed both changed library files. Independent source critic: READY at this exact head; final namespace fix is the only change since prior review. Default legacy dispatch, strict authenticated map binding/lane/device admission, conflict refusal in both setter orders, full physical Source occurrence/tie identity and native alias-pool forward remain intact. The new gradient is a conditional full120-map surrogate, not a discrete argmax derivative or a descent guarantee.

Executed checks on Linux x86_64 / RTX 5090 / CUDA 12.8 / sm120:

- Touched-file rustfmt and diff check: PASS.
- `cargo check --release --locked --offline -p uor-r4-training --features cuda --lib`: PASS.
- `cargo test --release --locked --offline -p uor-r4-training --features cuda --lib geometric_bank_generate::tests -- --nocapture`: 5 PASS.
- Categorical pullback CPU subset: 4 PASS; the CUDA test ignored in the CPU-masked run is not counted there.
- Explicit CUDA test `geometric_read_state_bridge::categorical_pullback_tests::categorical_pullback_cuda_native_and_analytic_adjoints_match_cpu --exact --ignored --nocapture`: 1 PASS, zero ignored, exit0. Test binary SHA256 `d245ec229d38f902ff5047b9e4aa891f7cc1993b89b7f8255010fcad8e7f2580`.

A separately built RECORD harness at `686abd9e36fa0457b9c7e4bf12815a1fec1b8569` verifies both library blobs equal this PR head. Its actual full-bank CUDA comparison completed with exit0 in 20.321656124 seconds, zero updates. An independent evidence agent verified the complete-file Rust BLAKE3 seal, all16row SHA256s, on-disk executable SHA256, native Copy/transport/full4096-token aliases/Generate-score equality, and all576 finite ACTIVE Context activity records. This is 16 condition configurations, six distinct row+prefix contexts, 32 loss-policy comparisons,64 backward paths. Different surrogate gradients are intentionally not required to match.

Canonical receipts:
- `/workspace/uor-r4/codex/native-categorical-context-credit/checks-c992a7b3-attempt1/`
- `/workspace/uor-r4/jobs/codex/20261007T093000Z-trained-route-capacity-gpu0.log`
- `/workspace/uor-r4/codex/categorical-bank-parity-record/actual-bank-attempt1`
- Report SHA256 `85a4532437e780c47e8e808235f7c456699058fc91f21015e5e1fbc6357d9e40`, manifest SHA256 `42dca55e90b7651f1ed60eeb82a2a9fefc0981ba17cd2083bbd68c75103b87a3`, harness executable SHA256 `a6f4807472489a6cede1a9ba4cf4fbf5f088f3b1a5f362713f5e2d67c014a013`.

Whole-workspace fmt has pre-existing drift in unchanged main files; touched files pass. No learned language/attention/speedup claim follows from parity. All jobs completed before pod release/deletion; durable receipts remain canonical. Ready for the protected merge queue.



---
_Comment/review ids in this file: 6035307259_


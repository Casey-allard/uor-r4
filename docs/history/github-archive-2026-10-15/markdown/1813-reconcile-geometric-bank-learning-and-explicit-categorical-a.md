# #1813 — Reconcile geometric bank learning and explicit categorical action export

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T23:20:56Z
- **Updated:** 2026-10-09T13:17:51Z
- **Closed:** 2026-10-07T00:06:41Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1813
- **Merged:** 2026-10-07T00:06:41Z
- **Merge commit:** 1f0a67e6774a1dcf3e76803fcf5e9d5db30e0c61
- **Base / Head:** main / UOR-Foundation:codex/reconcile-native-learning-20261006
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Reconcile the active native geometric bank-to-prediction learning path onto main: target-free full-bank Copy and Generate share authentic causal context and a common token-alias loss; context/potential parameters retain explicit export/rebinding, signed cue/prefix credit and selected-key transport. Add an explicitly receipt-bound categorical compiler that preserves discrete bridge choices while discarding energy magnitudes. The ordinary quarter-energy export remains unchanged.

Replacement 3 of 3 for the KEEP research stack, directly based on main after #1810 and #1812. The owner approved the larger integration PR with review by module. Code/document additions total about 7.4k lines; about 20k lines of generated run output are removed after verified canonical preservation.

Review modules:
- `geometric_bank_generate`, `geometric_generate_learning`, `geometric_read_state_bridge`: authentic full-bank assembly, Generate/Copy alias loss, selected-state transport, immutable snapshots and optional categorical export.
- `geometric_transport_state_credit`, selected source-realizer APIs: signed prefix/temporal utility, frozen source provenance, device preparation, actual save/reload/rebinding and context/potential credit.
- Context/potential credit and integer cue prerequisites: full observed-state/joint cue scorer binding and selected typed endpoint adjoints; main CPU references and CUDA numerical anchors retained.
- Documentation and output hygiene: preserve existing main D20 findings, record component/negative boundaries, retain T2/MiniLM summaries while moving generated numeric files out of Git. Archive `/workspace/uor-r4/codex/reconciliation/main-run-outputs-f4940919-20261006.tar.gz`, SHA256 `72982f9a84f8ac410c043d57b9e7c9466508114499493a8d1ff19f537f33f61d`; original tar-stream MD5 `168c379838cbb728c06a26929880d3a9`.

Historical experiment drivers, unused record-ranking/joint-fit additions, SourceBound and source-end fit helpers remain on original branches. Existing main APIs and sidecars are preserved. No default serving or GroundedSession seam change; no new fit, attention/chat qualification or laptop cost claim. The completed negative fit remains negative, and categorical marker scores are not calibrated confidence or an unchanged-energy quarter-STE checkpoint.

References #820 and #1792. Module source reviews found no blocker; exact-head package checks and focused actual CPU/CUDA tests are recorded before queueing. This PR supersedes the historical KEEP stack together with #1810/#1812 only after landing.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6027833431 — 2026-10-07T00:05:48Z — Casey-allard

Exact-head self-review: `e71d015d0ae1dad58a2605567683f64fd648cdae`.

Reviewed the active dependency closure by module: full-bank target-free Copy/Generate assembly and token-alias loss; Generate/read-state learning and explicit categorical export; signed temporal/prefix/context/potential credit and parent rebinding; retained native integer cue prerequisites. Existing main bf16/flash paths, older D20 state entries and the GroundedSession seam are preserved. Historical probes and unused fit APIs remain on their original branches. No source blocker found. The final CUDA fixture corrections were separately source-reviewed: each sidecar binds its exact reloaded parent, and offline floating loss uses a finite 1e-6 absolute/relative tolerance while native scores/actions/codes/token identities remain exact.

Actual exact-head checks (Linux x86_64, Rust 1.97.1):
- `cargo check --release --locked --offline -p uor-r4-training -p uor-r4-integer`: PASS, including training binaries.
- `cargo check --release --locked --offline -p uor-r4-training --features cuda --lib`: PASS.
- CUDA-feature library test build: PASS.
- 32 named training CPU tests: PASS, each filter verified to execute exactly one test.
- `cargo test --release --locked --offline -p uor-r4-integer --lib geometric_cue_carrier -- --test-threads=1`: 2 PASS.
- All ten changed Rust files: `rustfmt --check --edition 2021 --config skip_children=true`: PASS. Workspace `cargo fmt --check`: FAIL (exit 1), with output byte-identical to the independently executed main `291205ab` baseline. This is an explicitly retained pre-existing formatting exception, not a formatting PASS.
- Claim-wording check: PASS. This wording check does not qualify capability.
- Nine actual CUDA tests: PASS, including the complete independently reloaded source/full-bank/native-output/adjoint test; ignored GPU tests explicitly enabled, no skip or zero-test matches counted. RTX 5090, CUDA 12.8, SM120, pod `wrcem8l58vzjvh`, GPU 0 under uor-pod lock; job exit 0, 00:04:18Z–00:04:49Z on 7 October. Metal tests NOT_RUN on this Linux host.

Test executable SHA256: `886557faa963d4b490dbbf34a49845dfe120c1ee6e4c8ce492392da536859079`. This is the final PR-head test binary, not the older cached bootstrap executable. Clean committed source was fetched into the owned CPU checkout; CUDA execution independently checked the receipt hash.

Canonical receipts: `/workspace/uor-r4/codex/reconciliation/integration-e71d015d-attempt1/` and `integration-e71d015d-gpu-attempt1/`. GPU job log: `/workspace/uor-r4/jobs/codex/20261007T000418Z-reconcile-cuda-gpu0.log`. Earlier foreign-parent and one-ULP fixture failures, and an ignored-test invocation defect, remain preserved in separate attempts and are not scored as language negatives or passes.

Generated outputs removed from Git were archived and verified: `main-run-outputs-f4940919-20261006.tar.gz`, SHA256 `72982f9a84f8ac410c043d57b9e7c9466508114499493a8d1ff19f537f33f61d`; original tar-stream MD5 `168c379838cbb728c06a26929880d3a9`. T2/MiniLM summaries remain. No new fit or model-quality result was obtained here. The retained negative fit stays negative; categorical marker energies are not calibrated confidence or an unchanged quarter-STE checkpoint. These tests do not establish chat, retrieval transfer, reasoning, D11 whole-path cost or geometric advantage.

Owner-authorized review by module for the larger third integration PR; exact-head self-review allowed. Queue acknowledgements execute no scientific tests. Ready for protected merge queue at this exact head.



---
_Comment/review ids in this file: 6027833431_


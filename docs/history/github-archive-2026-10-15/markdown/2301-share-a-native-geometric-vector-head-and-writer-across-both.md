# #2301 — Share a native geometric vector head and writer across both model paths

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T05:38:41Z
- **Updated:** 2026-10-13T05:51:13Z
- **Closed:** 2026-10-13T05:47:50Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2301
- **Merged:** 2026-10-13T05:47:49Z
- **Merge commit:** d41dc422f66ff9bc69d0b5c39c6773ca5eb4fb07
- **Base / Head:** main / UOR-Foundation:codex/geometric-head-shared-writer-20261013
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The native stack's dense vocabulary head was blocked, and the compiler/store/emitter exposed a separate probability writer. This adds `head=spiralcore_product` over the actual learned embedding table and shares that same parameter with a CUDA vector channel for token carriage, exact-store re-entry and emission. Product-address admission bounds vector comparisons; raw negative mean-L1 scoring and explicit omitted-target/address credit use no probability normalization or dense contraction. The old probability producer/choice/table pool/objective refuse; an indexed restoration patch preserves their complete source.

Independent review found and closed host pointer-trace selection, mutable carried identity and reverse-order pointer adoption/public-config bypasses. Native artifacts bind source/geometry/shape; legacy likelihood, weighted-logits loss and host sampling refuse. Kimi's recurrence #2293 is integrated without changing its owned hunks.

Line: unified geometric route/action/learning integration · count 9/3 · headline saved 437/512 → 437/512

Count advances from 8/3 to 9/3 only on completed delivery; no reset. Opened transfer remains 0/128 → 0/128; M1 memory remains 20/40 → 20/40 and reply remains 37/232 → 37/232. Fresh acceptance is NOT_RUN. References #2030 defines this line and headline; #2029 owns the OP3a dependency.

Pivot recorded on the milestone: the owner's explicit continuation replaces the blocked dense/probability head and writer with a model-changing common vector path. This is the authorized operator implementation and finite interface qualification, not another run of the old configuration. Training stays held; no decisive language result or counter reset is claimed.

Decision: KEEP the implemented bounded head/common vector infrastructure; NOT YET PROMOTED as a trained model, complete learning/output path, or capability result.

Canonical record: docs/labs/geometric-head-shared-writer-2026-10-13/README.md. Current state, instructions, roadmap and operator plan are reconciled. Prospective pre-registration and TEST FITNESS: FIT for the finite controls: https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6138479221. Work card: https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6138327458. FIT does not release any owner hold.

Validation was executed on corrected source `09b3ee6df084beba1b9c6e98ef1e597bf2dc82bb`. The qualified crate files and Cargo.lock have verified source equality at the PR head; documentation/receipt changes do not establish a newly executed whole-head build. The actual commands on the owned RTX5090, CUDA12.8.93/Ubuntu24.04.3/Rust1.97.1 were:

```sh
CARGO_TARGET_DIR=/workspace/head-writer/target CARGO_BUILD_JOBS=4 CUDA_COMPUTE_CAP=120 /root/.cargo/bin/cargo test -p uor-r4-training --features cuda --lib --no-run
CARGO_TARGET_DIR=/workspace/head-writer/target CARGO_BUILD_JOBS=4 CUDA_COMPUTE_CAP=120 /root/.cargo/bin/cargo check -p uor-r4-training --features cuda --example geometric-stack
```

The first command passed in 10.76s and the CLI check passed in 3.83s. The fixed five-test native_vector_writer selection passed in 1.45s; test executable SHA256 `d220e7d59f72e1be81b6a196283bc66cb9a0b5517a44a24e7648e7fd35702986`. The preserved receipt supplies the executed test selection and binary identity.

- [x] Checks run on the qualified corrected source; source identity verified against this exact head, with the scope distinction above.
- [x] Five selected tests PASS: actual Stack/store/channel/FBS transport/artifact identity; analytic local distance adjoints and omitted-target/address credit; ties/collisions/padding; CPU/numerical/config/pointer refusals; static bindings.
- [x] Independent exact-source review: all three findings repaired and re-reviewed with no remaining blocker in this scope.
- [x] Touched-file rustfmt, source ratchets (1648 / 93), claim wording, whitespace and isolated-index restoration to the original file SHA256 passed.

No optimizer/update/training, full-trunk forward, corpus/quality panel or CPU model reference ran. Training remains held. This qualifies bounded head/common vector infrastructure, not a learned intent compiler, complete-path device qualification, useful conversation or D11 serving. GPU fixture wall time is not synchronized throughput. Private HF recovery commit `fc65a94525961f4bf2df6a8d961d27cb448a2897` was verified by pinned round-trip; iCloud preservation is 474112 bytes, MD5 `685a891d6f633f95edfc4c6dc2f0f7fc`. The owned pod was released and deleted after 3037s at $0.536/h, approximately $0.452176 before provider billing; cumulative cost and initial/superseded receipts are preserved in the canonical record.

References #2030
References #2029
References #2028


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6139148974 — 2026-10-13T05:46:56Z — Casey-allard

Exact-head validation for `a04c4fbe2af4f8fd45f8911932193b54d1c9a123`: the entire training crate is byte-identical to CUDA-qualified source `09b3ee6df084beba1b9c6e98ef1e597bf2dc82bb` (`git diff 09b3ee6df HEAD -- crates/uor-r4-training` is empty). Independent review found and verified repairs for the pointer-trace host selection bypass, mutable carried identity fields, and reverse-order legacy pointer/config admission.

Actual qualification: CUDA library test build PASS (10.76s), geometric-stack CUDA CLI check PASS (3.83s), five fixed device controls PASS (1.45s; zero ignored). Executable SHA256 `d220e7d59f72e1be81b6a196283bc66cb9a0b5517a44a24e7648e7fd35702986`. Rust formatting, claim wording, ratchets, harness and PR-body checks pass. Required transport acknowledgements are not substitute model tests.

No training, optimizer update, full-trunk qualification or language-quality run occurred; training stays held. The new bounded GPU geometric head/shared writer is qualified at operator scope; native integer serving and full-model useful conversation remain unqualified. Peer source through #2299 is preserved. Review and metadata reconciliation are complete; ready for protected merge.


### comment 6139194480 — 2026-10-13T05:50:33Z — Casey-allard

**Latest geometric head/shared-writer delivery — training remains held.**

Protected [PR #2301](https://github.com/UOR-Foundation/uor-r4/pull/2301) merged at
`d41dc422f66ff9bc69d0b5c39c6773ca5eb4fb07`, from reviewed PR head
`a04c4fbe2af4f8fd45f8911932193b54d1c9a123`; the qualified implementation is
byte-equivalent to final GPU source `09b3ee6df084beba1b9c6e98ef1e597bf2dc82bb`.
`head=spiralcore_product` requires `projection=spiralcore_fbs`: token input,
raw-distance output and compiler/exact-store/emitter vector adapters borrow the
**same learned embedding Tensor/Var**. Fixed SpiralCore product ports admit
bounded exact-identity cells; identity modulo 256 is address metadata, not a
semantic metric. Typed candidate IDs/masks preserve collisions and padding;
raw scores are negative mean-L1 displacement at fixed scale 1, with smallest-ID
ties. Default 8 cells gives 128 comparisons at V=4096, bounded above by 512. A full-V
compatibility score allocation remains. Omitted targets receive explicit score
credit without injection into inference candidates; separate token/address
margins have weight 1 each, with address credit declared an auxiliary surrogate.
The old whole-path probability artifact admission, choice, exponential table
pool and probability loss refuse; removed source is indexed/restorable.

Five finite tests PASS in 1.45s (static binding/CPU refusal plus four CUDA
controls), including actual Stack/store/channel/artifact connections, FBS
transport, distance adjoints, omitted-target/address credit, exact ties/padding
and numerical/device boundaries. CUDA test compilation PASS 10.76s; CLI compile
PASS 3.83s. Review corrected three concrete API defects: host-score trace helper,
mutable carried identity and reverse-order pointer/public-config validation.
Kimi's recurrence #2293 is integrated but is not device-qualified by this unit.
**Full-trunk learning/output compliance, learned intent compilation, full M2
quality/capacity and D11 serving remain unqualified.** No optimizer/training,
CPU model reference, full-trunk/corpus/quality run or new BPB result occurred.
Training and the separate M2 training/rental hold remain; finite FIT lifts no hold.

Unified geometric route/action/learning integration count **8/3 → 9/3**, no reset.
M1 memory20/40, reply37/232; M2 saved437/512, opened0/128 and genuinely fresh
NOT_RUN are unchanged. This source/interface delivery references the milestones;
it closes none. Next: freeze and qualify the complete declared learning/output
path before any owner hold-release decision.

[Canonical qualification/preservation record](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/labs/geometric-head-shared-writer-2026-10-13/README.md).
Private HF recovery/receipts at commit `fc65a94525961f4bf2df6a8d961d27cb448a2897`,
prefix `pod-setup/shared-vector-writer/20261013-09b3ee6df`, five files downloaded
and SHA256/byte verified; iCloud receipt MD5/index verified. Initial/failed and
corrected attempts are retained. Own pod released/deleted; ledger 3037s at $0.536/h,
approximately $0.452176 before provider billing. This GitHub delivery receipt
supersedes the canonical record's pre-merge pending-delivery wording.




---
_Comment/review ids in this file: 6139148974, 6139194480_


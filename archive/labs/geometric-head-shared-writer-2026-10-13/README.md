# Geometric product head and shared native vector writer

References #2029, #2030 and #2028. The OP3a head and common vector channel are
implemented and qualified at finite CUDA boundaries on corrected source;
protected delivery is pending. Training and the separate M2 training/rental hold remain. M1 memory20/40,
reply37/232 and M2 saved437/512, opened0/128 are unchanged; fresh NOT_RUN.
Line: unified geometric route/action/learning integration, count 8/3 → 9/3 on
completed delivery, no reset or capability promotion.

## Current decision and mechanism

Kimi ratifies the split at [#2029 comment6138381873](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6138381873):
Kimi owns recurrence gates and rate-select CUDA plus the rung-0 sheet; Codex
owns OP3a/shared writer and finite device qualification. Kimi's recurrence source
merged in #2293 and is integrated here; this unit does not device-qualify that
recurrence or the complete trunk. No peer source, job or artifact is taken over.

`head=spiralcore_product` requires `projection=spiralcore_fbs`. One learned
`embedding.weight` Tensor/Var supplies token input, addressed output and the
compiler/exact-store/emitter `NativeVectorChannel`. Encode, transport, selection
and margin credit remain on CUDA; exact store control and artifact I/O are host
metadata. The shared representation uses whole 8D blocks, width 8..1024; total
capacity is unqualified and no MiniLM 384D dependency is adopted. The first and
last 8D query blocks admit product cells in the same exact16-root SpiralCore
atlas; at width 8 both ports read the same block. The ports are not an
independently learned H4 companion or the full 240-root E8 vocabulary.

Cell = token identity modulo 256, collision slot = quotient. This partitions
256..8192 exact identities into at most 32 slots/cell; it is address metadata,
not a token/hash semantic metric. Fixed atlas ports provide routing geometry;
token vectors are directly learned parameters. At most 16 retained cells ×32
slots gives 512 token-vector comparisons; default 8 cells gives 128 comparisons at
V=4096. Typed candidate IDs and validity masks accompany raw scores. Score is
negative mean absolute displacement over the complete vector, fixed scale 1;
margin 1 is in average-coordinate-displacement units. Exact ties select the
smallest cell/token identity, and padding cannot emit. A rows×V compatibility
score allocation remains, without another vector scan or dense parameter map.

The native objective separately computes the target vector score when the
target is omitted, against declared admitted non-target rivals; inference
candidates never receive a target injection. Token and address margins each
have weight 1 in their sum. The address margin gives auxiliary credit versus
admitted non-target cells: it is a surrogate, not the derivative of hard top-K
admission or a measured descent guarantee. Raw-score greedy emission replaces
probability sampling on this path. Weighted legacy objectives, NLL and host
sampling refuse in the declared native configuration.

Artifacts bind writer schema `uor-r4.shared-native-vector-writer/1`, source,
compiler geometry and shape. Missing, stale or undeclared head bindings refuse.
Old probability artifact admission, sampled choice, exponential table pooling
and probability loss refuse rather than silently convert. Removed source is
[retained as an indexed restoration patch](../../history/branch-archive/INDEX.md):
`whole-path-probability-writer-inactive-20261013.patch`, SHA256
`905e363b4dc3829d3106a5e470c28922f33b635c3f4288d1663aa012090f4563`,
restores the pre-unit file at `5c0cf2945`, whose SHA256 is
`0c8ef618f22fc1b4ecec454b8e7d05b4ffa3b4c4325b012fa3d3ec0f26a5bcf7`.
Exact store record, occurrence/version and missing/evicted statuses remain
separate from geometry. This is vector infrastructure shared by actual adapters;
it does not qualify a learned intent compiler, complete M2 model or D11 serving.

## Qualification and outcome

The [prospective finite FIT](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6138479221)
and [work card](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6138327458)
authorize the fixed finite controls, not training. The
[interface receipt](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6138582838)
records the integration boundary. Final GPU source freeze:
`09b3ee6df084beba1b9c6e98ef1e597bf2dc82bb`; peer source through
`84287b397` is integrated. NativeVectorWriter source SHA256:
`69a86693e980662f1504668e8906c32555c36ff8e3c75a3abd8a38f877748a12`.
CUDA test binary SHA256:
`d220e7d59f72e1be81b6a196283bc66cb9a0b5517a44a24e7648e7fd35702986`.

The final five focused tests PASS in 1.45s: static binding/CPU refusal and four CUDA controls
cover actual Stack/store/channel/artifact connections, SpiralCore/FBS transport,
distance adjoints, omitted-target and address credit, exact ties/padding and
numerical/device boundaries. Final CUDA library-test compile (`--no-run`) PASS
in 10.76s; CUDA CLI compile PASS in 3.83s. The device fixture confirms both adapters
borrow the same embedding parameter. It runs no CPU model reference, optimizer
update, training, full-trunk generation, corpus panel, BPB or quality comparison.
Review found three API defects: a host-score trace-helper bypass, mutable carried
identity metadata, and reverse-order pointer/public-config mutation that could
bypass native factory validation. They are corrected in source freeze
`09b3ee6df084beba1b9c6e98ef1e597bf2dc82bb`; production build and the fixed
five-test rerun pass. Model arithmetic is unchanged; exact-source review confirms
all three corrections with no remaining blocker.
The initial `84b1a530f52b99651188fe7cabdf35c75ed923ab` receipt (five tests,
1.44s; writer SHA256 `2032d8d8cf0c9e52782953337f66115eb01bfee8355cdff2172d260a27bf3cde`;
binary SHA256 `7f7bd09b12fbddb722fc92643a0e8d3e809288af127d15cba91e30af6e90cd53`)
is preserved as superseded evidence, and does not qualify the corrected revision.
These are finite arithmetic/interface results; complete learning/output path
qualification and all model capability readings remain separate.

Compilation-only recovery on an owned, admitted pod uses a dedicated target:

```sh
CARGO_TARGET_DIR=/workspace/head-writer/target CARGO_BUILD_JOBS=4 CUDA_COMPUTE_CAP=120 /root/.cargo/bin/cargo test -p uor-r4-training --features cuda --lib --no-run
CARGO_TARGET_DIR=/workspace/head-writer/target CARGO_BUILD_JOBS=4 CUDA_COMPUTE_CAP=120 /root/.cargo/bin/cargo check -p uor-r4-training --features cuda --example geometric-stack
```

Executing the test binary still requires the current named finite-qualification
card, prospective FIT and resource admission. These build commands lift no hold.

## Resources and delivery

Owned worktree and shared harness claim; live heartbeat and cumulative resource
ledger remain required. Pod `vast-55678610`: RTX 5090 at $0.536/h, Ubuntu 24.04.3,
CUDA toolkit 12.8.93 and Rust 1.97.1. The setup controller stopped before the old
bootstrap parity suite; manual build-only setup retained. CUDA build target 7.1GiB;
peak RSS unmeasured. Admission projected≤2h pod,4 build jobs,8GiB RSS,
30GiB target +1GiB receipts, with 37GiB local free and30GiB+128MiB floor.
Actual rental/billing and preservation receipts belong to the cumulative ledger
and final delivery receipt; no session resets it or treats missing old balance
as zero. Private HF setup storage permission releases no training hold.

Protected PR, exact-head checks/review, merge verification and owned cleanup
remain pending. This record reports implementation and finite qualification;
queue acknowledgements are not tests and an open PR is not completed delivery.

Final preservation: private HF `caseyallard/uor-r4-store`, commit
`fc65a94525961f4bf2df6a8d961d27cb448a2897`, prefix
`pod-setup/shared-vector-writer/20261013-09b3ee6df`; all five recovery/receipt
files downloaded at the pinned commit and verified byte-for-byte and by SHA256.
iCloud `codex/codex-shared-vector-writer-20261013.tar`: 474112 bytes,
MD5 `685a891d6f633f95edfc4c6dc2f0f7fc`, automatic round-trip and exact index
readback PASS. Both initial and corrected/failed attempts are preserved. The
large test executable is identified by hash/size (738583448 bytes), not retained
as a second binary copy; no trained model was produced. Owned pod released and
deleted after final receipts: ledger uptime3037s at $0.536/h, approximately
$0.452176 before provider billing. Source/build time and fixture wall time are
separate from model training, which remained held and did not run.

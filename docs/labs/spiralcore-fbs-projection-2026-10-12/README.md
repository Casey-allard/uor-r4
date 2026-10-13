# SpiralCore BFS plus retained Atomic Ternary FBS projection

References #2029 (M1), #2030 (M2) and #2028. Owner request: remove the dense
quaternion contraction too, use **both** supplied mechanisms together, preserve
all work, keep training held and protected-merge the fix. The owner explicitly
authorized development/build/kernel tests on a CUDA12.8+ devel Ubuntu24 pod.
This record covers that finite operator unit, not training or shared-writer
completion. Line: unified geometric route/action/learning integration;
**count7/3 → 8/3**, no reset. Historical M1 memory20/40/reply37/232 and M2
saved437/512/opened0/128 remain unchanged; genuinely fresh qualification NOT_RUN.

## Current decision and mechanism

Replace #2262's `sum_k W_jk ⊗ x_k` with exact 8D carrier selection and actual
SpiralCore BFS transport. Native mode is `projection=spiralcore_fbs`; its saved
legacy mode field is `quaternion_weights="spiralcore_fbs"`. It stores F32 query
points shaped `[output_blocks, stages, 8]`, with distinct `.spiralcore_fbs`
parameters. Input/output pair coefficients and the all-input contraction are
absent. Each output block selects **one** retained input atom and transports its
complete eight coordinates by the exact composed signed action.

The static Rust compiler retains the ordered ternary binder abstraction from
[Atomic Ternary FBS r3.1](../../../research/spiralcore-v68/README.md): real children,
parent links, atom identity, support length and the incomplete forest frontier.
Atoms use the declared `(tau=2,g=1,length=8)` mode; equal-level triples close
without padding or discarding descendants. This is not the full marked FBS
object or a semantic metric derived from length. Native geometry explicitly
couples these children/frontier ports to the supplied action graph; the source
reference does not provide this learning law or prove that coupling.

SpiralCore's 15 bivector edges provide actual ordered BFS histories between the
16 roots in one E8 anchor orbit, over 64 signed operator states. The route
left-composes its real action; root identity and the complete signed state are
kept separately. It does not substitute canonical endpoint axes for a route,
forget orientation, claim all240 E8 roots or treat an independent learned H4
partner as the golden/Galois companion. The exact core tables are unchanged.
The current operational bound is 6560 input atoms, at most16 frontier roots and
8 selection stages; it is not a language-quality or representation-width bar.

The input adjoint is the exact inverse signed action scattered only to the
selected atom. Query credit is an explicit **geometric root-edge secant
surrogate**, not the derivative of discontinuous hard selection. Every lawful
sibling is forced once, its full descendants reroute under the altered signed
frame, and the device output credit pairs with that carrier displacement. The
contrast resolves along the fixed root edge. There are no probability weights,
normalized exponentials or learned input/output coefficient grids. A useful
learning/descent property remains unmeasured.

Static table construction reads no model tensors. Activation distances,
selection, transport, adjoints, secants and numerical checks run on CUDA;
only a validation code is copied to the host. No native CPU/Metal fallback
exists. F32 query masters support F32 and BF16 carrier storage. Local G2
Hamilton rotation from #2260 remains. The native mode refuses dense 2D head/
NoRead fallthrough and unsupported snaps; whole-model execution is not qualified.

## Source, artifacts and preservation

Source freeze for the four device tests:
`c2ebb1e32f60b32288e03d8d4e934dab968bf22c`; parity-target parse repair:
`0a889bb195705b121972a39dc6e994c4fd16dc38` (only an orphaned old test comment).

- [Static compiler and custom op](../../../crates/uor-r4-training/src/spiralcore_fbs.rs),
  SHA256 `c78241c5aa5dbd4379d75bf5f030cd0e4498245cdd0be605f7f852f7f17c21d7`.
- [CUDA forward/adjoints](../../../crates/uor-r4-training/src/spiralcore_fbs.cu),
  SHA256 `517bffcc3c2029ddd171e75fd8fc1eb532755b30e683c66bd230382416480b0d`.
- Device-tested executable SHA256
  `5317ebd7441beda7c82b5bebf3bafdff725e99c0b7a1ff545f299edd695c53f0`.

Schema `spiralcore-fbs-projection/1` binds source-reference hash, actual compiler
and kernel bytes, retained children/frontier, signed axes/composition kappas,
BFS histories, widths and query shapes. Save/load preserves native parameter
bits and refuses absent/stale/undeclared geometry bindings and unsupported
native snaps. Old `all` artifacts remain inspectable inactive metadata; neither
`hamilton_linear` nor `quaternion_dense_weight` can execute or silently convert
them into a native artifact.

The [indexed restoration patch](../../history/branch-archive/INDEX.md) preserves
pre-replacement dense CPU/CUDA source, adjoints and comparators from `8a944609a`.
It applies to an isolated index at `b68431d5e`; the restored named files equal the
old source bytes. Historical commits, artifacts, results and peer work remain
preserved. No superseded configuration is relabelled a mechanism negative.

## Qualification and outcome

[Claim](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6136794551),
[prospective finite-kernel FIT](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6137419532)
and [shared-credit amendment](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6137539936)
bind these fixed controls. Author self-review is scoped to operator arithmetic.
No CPU model reference, dense comparator, optimizer update or language panel ran.

| Check | Actual result and scope |
| --- | --- |
| Production CUDA/Rust build and four `native_cuda_tests` | PASS; all27 terminals of a three-level binder, signed carrier orientation, exact selected-input adjoint, two-atom GPU secant control, shared2-row/3-output credit, F32/BF16, numerical/shape/CPU-entry refusal. Missing CUDA fails |
| Three `static_plan_tests` | PASS; ordered complete atoms and parent/length retention up to6560, actual BFS histories/actions/inverses and distinct ternary ports, CPU entry refused |
| Two `native_projection_` tests | PASS; native shapes/geometry identity and artifact bit round-trip plus missing/stale/undeclared binding and snap refusal. Parameter preparation/I/O only |
| `retired_dense_quaternion_projection` | PASS; old contraction and dense expansion refuse before model arithmetic |
| Production CUDA `geometric-stack` example check | PASS; CLI request/mode wiring compiles |
| Retained CUDA parity target `--no-run` | PASS; compile only, old CPU-reference suite never executed |
| Touched-file rustfmt / whitespace / claim wording | PASS |
| Source ratchets | no-softmax1649/1649 PASS; no-matmul93/93 PASS, down from95. These are partial textual ratchets, not whole-path certification |

Gather/sign and selected-input controls assert zero absolute error for these
exact representable fixtures; query secant controls assert absolute error≤1e-5.
The reference is eight explicit GPU gather/sign operations and GPU contrast/edge
arithmetic, never a CPU forward or dense comparator. Relative error at a zero
expected coordinate is undefined; absolute checks are the declared fixture bar.
No throughput comparison, held-out measure or mechanism-learning conclusion is
claimed. Final four-test wall time2.50s includes launch/validation overhead;
it is not synchronized GPU-only kernel timing.

Retained failed attempts are blockers repaired before delivery: the first
launch lacked `/usr/bin/time` and executed no model; the first device controls
inferred fixed fixture credit as F64, repaired to F32; removing the old tests
left an orphaned doc comment, repaired before the retained target compiled.
An earlier Mac compile/API/syntax check used a deliberately non-executable
PTX stub; the actual unmodified production CUDA dependency build above
supersedes it. Failures and superseded checks remain in the QA bundle.

## Resources, delivery and next dependency

Development/fixes and production qualification ran on owned pod
`vast-55667424`, one RTX5090/sm120, Ubuntu24.04.3, nvcc12.8.93, driver580.105.08,
Rust1.97.1 and the unmodified production Cargo dependency graph. The image was
`runpod/pytorch:1.0.2-cu1281-torch280-ubuntu2404`. Four build jobs; cold build2m58s,
peakRSS4,537,032KiB; final rebuild10.83s, CUDA CLI check40.65s and retained-test
compile34.22s. Pod target23GiB stayed within35GiB projected storage; local free
space stayed above30GiB. Training updates0, CPU model executions0, quality runs0.

The shared append-only pod ledger records1148s uptime at$0.536/h, approximately
**$0.170924** before provider billing granularity, including provisioning,
setup, fixes, builds and receipt preservation. This adds to existing cumulative
charges; no issue/session resets them. The pod lease was released and the pod
deleted after its unique source/logs/identities were verified locally. Regenerable
build outputs were reclaimed; other sessions' pods and files were untouched.
QA preservation: `codex/spiralcore-fbs-projection-20261012-qa.tar`; the cloud-store
receipt records **604672 bytes**, MD5
`0e60182d9425c7f5b30d23a9d5b559eb`, verified by round-trip and indexed. Protected PR evidence binds
the delivered source/tree; branch/worktree cleanup follows verified merge.

The owner's later storage permission also preserves a build-only setup recipe,
qualification receipts, source identities and SHA256 manifest in the private
HF dataset [caseyallard/uor-r4-store](https://huggingface.co/datasets/caseyallard/uor-r4-store/tree/3314470fc850733d46ffca4e512f3a2f4333c57b/pod-setup/spiralcore-fbs/20261012-0a889bb19).
All five files passed a download/SHA256 round-trip at commit
`3314470fc850733d46ffca4e512f3a2f4333c57b`. This bundle contains setup instructions
and receipts, not a CUDA toolkit tar, binary cache or trained model. Local
exact-head compile/static/artifact validation extended the build-cache estimate
from2.8 to6.5GiB (actual5.9GiB) while maintaining the30GiB free-space floor;
no CPU model computation occurred.

**Training remains held.** This owner authorization covers this pod development,
build and finite kernel qualification, not another rental/training campaign.
OP3a and the complete lawful GPU learning/output path remain dependencies,
followed by shared-vector/writer integration, prospective model FIT and explicit
applicable hold release. Query secant learning fitness, language capacity,
fresh transfer and D11 integer/table serving are UNVERIFIED/NOT_RUN. Total 8D
block count is not fixed by MiniLM384 or by this unit.

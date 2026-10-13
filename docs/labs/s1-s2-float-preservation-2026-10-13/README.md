# S1/S2 floating-point qualification — preserved, inactive

**13 October 2026, Codex/Sol. Owner-directed scope change, not a mechanism negative.** Casey requires no floating-point model computation, including learning. Casey's architectural contract is geometry-preserving E8 transport through SpiralCore; FBS supplies binding/routing structure and Hamming distance is one routing factor, not a replacement for that geometry. The target remains one native vector writer for input, carry and output, with discrete/ternary or exact geometric state and updates. [Owner relay](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6154399083), [transport clarification](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6154447417), [routing clarification](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6154472370).

The F32 production path and F64 test instrument below are **inactive evidence**, not the requested discrete implementation or permission to run. Both training holds remain HELD. No optimizer update or corpus evaluation ran. No controls ran after the owner's no-float direction. This preservation delivery changes no active model implementation and does not promote PR #2355 or its incomplete qualification.

## Observed attempts

One owned Vast RTX5090, instance `55800061`, CUDA12.8.93/compute_120, driver595.80, Ubuntu24.04.1, Rust1.97.1. Reports are separate exclusive directories; no failed attempt was overwritten.

| Source | Observation |
| --- | --- |
| `f5b124aa2` | Setup failed: missing `BackendStorage` trait import in the CUDA selector. No controls ran. |
| `cd86caf38` | Repaired build; 10/33 controls passed before the old controller stopped at the F32 secant instrument error. Remaining22 were not run. |
| `09f8fd766` | Device-F64 elementwise secant instrument; first controller passed20 then stopped on a malformed S2 fixture. A complete independent replay ran all33:30 passed; findings at21,22,28. |
| `49b6b2bf9` | Corrected two-level BOS fixture and composed diagnostics: all33 ran,31 passed. Control22 retained a small S2 secant discrepancy; control28 reported zero credited readers. |
| `7569f94e6` | Shared-writer tie-gradient repair source-reviewed independently. CUDA setup completed20:19:33UTC, exit0. Added regression and34-control replay **NOT_RUN** after owner scope change. |

The first secant instrument rounded two global F32 losses before subtraction. At the same disputed outputs, analytic gradient was `0.041992284`, naive estimate `0.04184246`, and repaired device-F64 estimate `0.04200984584179501`, passing the unchanged criterion. This was an instrument defect, not an S1 failure. The causality fixture supplied three coupling rows for a tree with two levels because BOS split30 tokens into17+13; correcting that fixture made its unchanged causal assertions pass.

The remaining S2 observation was analytic `-0.0056131557` versus stable secant `-0.005587936035222896`; diagnosis was pending. It is not evidence rejecting S2. The composed fixture reported zero capture triggers. Source review then found the writer's differentiable maximum sent the full gradient to every tied address rival. The preserved repair selects one eligible tied cell deterministically and gathers its original score, retaining the forward maximum, total objective and empty-rival behavior. Its focused CUDA regression was authored but not executed. Do not claim measured resolution of that composed finding.

The separately built controller collects independent failures and reports all registered outcomes; identity/prohibited-operation/resource failures still stop execution. This applies R12/D31 without turning an arbitrary threshold into authority over a mechanism. The finite fixture never activated the production read in the trunk, qualified language behavior or established integer-serving/energy performance.

## Restoration and preservation

- [Inactive source patch](../../history/branch-archive/s1-s2-float-inactive-20261013.patch), SHA256 `8dbfc9feb897cbe43609c2f6da1b2e33e24c3d1ca880d85bc42da516a34835f5`.
- Apply against `7e7dd06359f294735c50d767fa9f488af1c4f8ec`; isolated-index application reproduces the complete `7569f94e67bec672b9b3d6bf7e25fdc06f4cac5f` tree exactly: `312574e108b96e33d556fca9b7123711138d6799`. This is restoration evidence, not activation permission.
- Restorable Git bundle SHA256 `76b62e3dc5aa733bd1284e4116c4a99a69fdc3cb1064e973a023d3800e8e30b1`; `git bundle verify` passed. Its prerequisite commits are preserved on main.
- iCloud archive: `icloud:UOR-R4/results/codex/codex-s1-s2-preserved-20261013-2023.tar`, **35,175,936bytes**, MD5 `5b72d5adc36e9de3711bfe537951fb11`, verified by cloud-store's download round-trip. It contains source bundle/patch, all setup/control receipts, PTX copies/hashes, executable/compiler identities and owner-direction records. Restore with `cloud-store fetch codex-s1-s2-preserved-20261013-2023 DEST`.
- Built caches and large executables are reproducible source/build products; the retained manifests record their hashes and sizes. No unique model/checkpoint or dataset was generated. Other labs' source/worktrees were untouched.

The owned lease was released and instance `vast-55800061` deleted on13October around20:24UTC after preservation. Fresh `uor-pod status` reported zero running pods and `$0/h`. Quoted rate was `$0.892/h`; roughly51minutes across setup, controls, diagnosis and preservation is about`$0.76` at that quote, **an estimate, not an invoice**. All attempts share that cumulative interval; no new pod was rented after the owner correction. The `$4` projection was not a mechanism stop rule.

## Next dependency

Specify and review the discrete learning/update law using geometry-preserving SpiralCore transport, E8/icosian representation, FBS and the shared writer. Exact quaternion/icosian representation may use finite tables or integer coefficients; it must not silently reintroduce floating-point arithmetic or dense learned contractions. Preserve orientation and the meaning of every routing factor. No new mechanism implementation or test-fitness claim is made by this record. Kimi recorded this direction in D32 (#2367) and the Hamming-routing addendum (#2369); Claude and DeepSeek received it directly on the milestone. Training release remains the owner's alone.

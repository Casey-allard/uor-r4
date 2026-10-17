# Prospective finite-only driver disposition

Independent reviewer `review_2551` accepts preparing a replacement packet with
an exact allowlist **580.95.05 or580.105.08**, single RTX5090, sm120. The
[FIT sheet](../../harness/prep/m1-counterfactual-baseline-reuse-2026-10-16.md#cost-and-stop-rule)
requires prospective disposition for hardware/toolchain/driver changes. Run1
observed only startup metadata and stopped before the instrument; no model
outcome selected this policy. This record grants no retry or timing permission.

NVIDIA documents support for older CUDA-toolkit applications on newer drivers,
and explicitly retains CUDA12.x support on580-series drivers through backward
compatibility ([compatibility overview](https://docs.nvidia.com/deploy/cuda-compatibility/why-cuda-compatibility.html),
[version table](https://docs.nvidia.com/deploy/cuda-compatibility/minor-version-compatibility.html)).
That supports deployment admission of the retained CUDA12.8.93 build; it is
not evidence of this model's numerical parity or identical runtime resources.
The actual driver and libraries must be recorded. No unrestricted version range,
recompile, PTX alternative, new GPU class or hidden binary is admitted.

A replacement packet must preserve exact protected executable/cubins and all
source/image/report/fixture/geometry identities; Ushared0/R512,dynamic0,
zero own-AOT register/local deltas,zero spills,block16 capacity; one invocation
with64 attempts and16 typed refusals; zero apply. Stop on the first discrepancy,
preserve actual errors and partial evidence, and never subtract a driver
reservation or waive a resource comparison. Both modes run on the same actual
GPU/driver; no cross-driver equivalence is claimed. Timing remains separately
unreleased. Fresh independent readiness and a distinct operational release must
bind exact replacement scripts, final pin, resource admission and preservation.
Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE.

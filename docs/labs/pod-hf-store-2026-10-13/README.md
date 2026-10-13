# Pods bootstrap from the HF store and save to it

References #2028. Claude lab, sessions `claude/storea` and `claude/storeb`, 13 October
2026. Owner request: confirm the pods work, that everything needed for CUDA development,
tests and training comes from the HF store, and that it can be uploaded as needed.
Infrastructure only: no model training or evaluation (training stays held).

## What changed

- `scripts/pod/uor-pod-bootstrap.sh` fetches the Rust toolchain, the crate cache and the
  binary cache `bin/<sha>-sm<cap>.tar` from the store when `/workspace` lacks them, and a
  pod that had to make one uploads it. A store build is used only with `"parity": "PASS"`
  and verifying `SHA256SUMS`. Build jobs come from the cgroup's `cpu.max` when the
  provider does not name the CPU quota.
- `scripts/pod/uor-store`, installed on every pod: `put`, `get` (reports a missing path),
  `ls`.
- `uor-pod down --to-store`: a Vast pod uploads its results to `vast-archive/<pod>/`.
- [compute.md § The HF store](../compute.md#the-hf-store-what-a-pod-gets-from-it-and-gives-back-13-october-2026)
  has the full table.

Tests: `scripts/pod/tests/uor-pod-dryrun.sh` **348 passed, 0 failed**. Independent review
(REQUEST CHANGES) found: the crate-cache upload ran before the build was published; `uor-store
get` could report an old local copy; a store build needed stricter acceptance (commit, GPU,
exact binary list, plain files only); a bad toolchain archive aborted the bootstrap. All fixed.
Each safeguard set aside fails its cases: checksums 2, upload after a build 2, exact binary
list 2, no store call inside the build step 1, missing path 1, old local copy 1.

Owner directions the same night, applied here: **Vast only, Runpod an emergency backup**
(`up --wait` falls back after 30 min without a Vast offer when Runpod has stock; by hand only
with `--owner-approved`: tests for the refusal 1, the stretch 2), and **GPU and Rust over CPU
and Python** (no new Python on this path: `uor-store ls` uses `curl` and `jq`, the store-build
check uses `grep`).

## Measured (Vast RTX 5090, 13 October)

| Pod | Host | What happened | Ready after |
| --- | --- | --- | --- |
| A, cold (`vast-55668235`) | Czechia, $0.815/h | data from the store; Rust via rustup, then **uploaded** (612 MB); build 194 s (128 jobs on 16 CPUs); parity 43/43 PASS; crate cache (149 MB) and binaries (143 MB) **uploaded** | 330 s |
| B, warm (`vast-55669478`) | South Korea, $0.536/h | data, Rust, crate cache and **binaries all from the store**; cache hit, no build; 16 jobs | **57 s** |

On A: `nvcc` 12.8 compiled and ran a 1,048,576-element kernel with 0 wrong;
`compute-sanitizer` 0 errors; NVRTC 12.8 loads; cudart, cuBLAS, cuFFT, cuRAND, cuSPARSE,
nvJitLink, `cuda-gdb`, `ncu`, `cuobjdump`, `nvdisasm` present; `nsys` absent from the image
and installed from NVIDIA's apt repository (`cuda-nsight-systems-12-8`); that repository
answered HTTP 200. `uor-store put` then `get` round-tripped a file, and the laptop
downloaded the same file from the store. On B: `cargo check --offline --features cuda
--tests -p uor-r4-training` passed in 43 s from the store's crate cache alone. Both pods
were archived and destroyed; together about $0.15.

## Limits

- The CUDA toolkit is not in the store (licence): a pod depends on the pinned image or
  NVIDIA's apt repository for it. Both were available on 13 October.
- A store cache hit trusts the building pod's parity verdict; the new pod does not rerun
  parity (the same trust the shared volume's cache always had).
- Every new commit adds a 143 MB binary tar to the store; old `bin/*.tar` files need
  pruning by hand until a retention rule exists.
- Training readiness means binaries, data, base models and GPU are in place; no training
  ran (the owner's hold stands).

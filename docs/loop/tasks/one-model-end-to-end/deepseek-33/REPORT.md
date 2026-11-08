# REPORT: one-model-end-to-end, deepseek-33

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Run pin:
`1cc31512879a52e0aee69e22a154af0b192f4dda`.

**Same-scorer parity measured.** The selected step-512 checkpoint has zero
float/integer chosen-token mismatches under both registered renderings. The
final step-2,048 checkpoint has zero under `one-user-turn/v1` and one mismatch
out of 2,832 targets under `per-turn/v1`. EOS rank agrees on every one of the
1,024 row-rendering-checkpoint cases.

## What did not work, first

- The first job assumed `m2-fresh256` had 512 rows and stopped after the first
  report. The sealed panel has 256 rows and 2,832 reference targets; the check
  was corrected to the actual registered shape.
- Attempt 2 computed all four reports but failed its manifest because the live
  job log was inside the directory being hashed. The seal log now lives outside
  the evidence root.
- At final step 2,048, `per-turn/v1` has one real chosen-token divergence:
  `fresh-diverse-length8-00-swap1-q2-forward-home`, target 15 of 19. The float
  path gets 15/19 top-1 targets and the integer path 16/19; both place EOS at
  rank 2.
- `m2-reply512` remains unavailable, so this is an integrity reading on
  `m2-fresh256`, not an M2 qualification result.

## What changed

- Added one closed parity job over the sealed content-ratio source.
- It verifies the complete source seal, then pins both checkpoint files,
  Hamming reads, integer artifacts, run config, tokenizer and all four panel
  files before model work.
- It runs step 512 and step 2,048 under both `per-turn/v1` and
  `one-user-turn/v1` through the merged `packet-parity` binary.
- Reports, the source subset, job script, binary and CPU/GPU identities are
  sealed, uploaded and read back before teardown.

## What ran

- One DeepSeek-owned RTX 5090 pod, `vast-59676889`, at $0.563/hour.
- Exact parity pin `1cc315128`, built with
  `--features cuda,packet-parity`.
- Four report passes over 256 panel rows each.
- The owned pod was released and deleted after readback.

## Measurements

| Step | Rendering | Chosen mismatches | Rank mismatches | Float/Integer top-1 | EOS-rank mismatches | Max rank delta |
|---:|---|---:|---:|---:|---:|---:|
| 512 | `per-turn/v1` | 0 / 2,832 | 12 / 2,832 | 1,614 / 1,614 | 0 / 256 | 2 |
| 512 | `one-user-turn/v1` | 0 / 2,832 | 11 / 2,832 | 1,600 / 1,600 | 0 / 256 | 4 |
| 2,048 | `per-turn/v1` | 1 / 2,832 | 19 / 2,832 | 1,659 / 1,660 | 0 / 256 | 3 |
| 2,048 | `one-user-turn/v1` | 0 / 2,832 | 10 / 2,832 | 1,659 / 1,659 | 0 / 256 | 3 |

The parity instrument proves serving decisions largely agree with the float
checkpoint. It does not show that either representation answers correctly; the
panels had already measured zero complete content rows.

## Evidence

- Report SHA-256:
  - step 512 `per-turn/v1`: `88af24613a76c83b5a7fae383f05cf24b98c3ee4ef2b6eef4165e8e190d9a8bc`;
  - step 512 `one-user-turn/v1`: `7921c9ea6bc1e3ddae2b2270f50f91f1fb95926fbe4d79ba51ea1411e3bb4d01`;
  - step 2,048 `per-turn/v1`: `ec9f69e2cfcaa81487c89d14f76f98b4fd66f63b36fae8fb1764698d2af77bb9`;
  - step 2,048 `one-user-turn/v1`: `c6bd8741469ac9089f30cdde61d06518de1be4ef27671a79237eb483b8c7fafd`.
- Parity binary:
  `ac17431554e8b866c79d5c74a54f34b9b4660255ff140146f7bfe3e73e46b6e9`.
- Source content-ratio manifest:
  `a183ae67c3563d26c2cf3f0e437f5bbe64e2d735dd9b2a0608d8b32da663164f`.
- Seal: 32 files, `BYTES=14,499,465`; SHA manifest
  `eb10e8b0ce55f85d5ec35aab62d779dc89304f9a3d93f091c70d564e2508b6e1`;
  `READBACK=verified`, `EXIT=0`.
- CPU: AMD Ryzen 7 7800X3D 8-Core Processor, x86_64, 16 logical CPUs.
- GPU: NVIDIA GeForce RTX 5090, 32,607 MiB, driver 580.178.04.
- Store root:
  `results/deepseek/one-model-end-to-end-packet-parity-2026-11-08/runs/20261108t064446z-5825453f4aa453b4/`.

## Checks run at the head

- Full store readback verified every `SHA256SUMS` entry.
- The source content-ratio seal re-verified before any model forward.
- The job script passed `bash -n`, ShellCheck and `git diff --check`.
- The pod build of `packet-parity` succeeded with CUDA and the parity feature.
- Every report has 256 rows, 2,832 targets and 256 EOS rows.
- Pod teardown completed; project admission returned to one running Claude pod.

## Skills used

No system-provided Codex skill applied. The repository's D22, D37, GPU, serving
and delivery rules were used directly.

## Still open

- The one final per-turn chosen-token divergence is recorded; it is an
  integrity characteristic of this checkpoint pair, not a capability result.
- `m2-reply512` byte recovery remains incomplete.
- No useful-chat, capability, M2 or mechanism claim follows.

## Plain summary

The float checkpoint and integer artifact usually make identical nearest
decisions. Selected step 512 matches completely; only one final-checkpoint
decision differs. The model still does not complete accepted replies.

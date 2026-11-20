# [codex] D30 CUDA device evidence

The frozen readout made zero correct reciprocal donor switches across six fresh pairs. The swap changed both choices in three pairs and preserved both in three. All eight fresh and exposed pairs had identical masks and local vectors at matched relative source addresses. Every swapped decision reproduced the partner’s nominal decision at that relative address, including abstentions. Of the six fresh pairs, all six differed at the factual boundary, and one pair had one transient packed-code equality along the shared suffix. The full-code diagnostic found a legal winning code for 11 of 12 fresh gold addresses. The observed context selected one of those 12 original golds under the nominal arm.

## Execution and verification

- Vast `vast-61560422`, NVIDIA GeForce RTX 5090, GPU `GPU-599353c1-3d58-3237-2d71-e33a53398af9`, driver 580.178.04, CUDA compute capability 12.0, nvcc 12.8.93. Projection exited zero at 02:51:45 UTC on 20 November 2026. Scoring exited zero at 02:54:23 UTC.
- Both cold output directories matched their corresponding first run byte for byte. Held local projection matched the expected records. The native Rust recurrence matched all 1,353 trajectory prefix codes and all 18 held context taps. The loaded D26 carrier matched original address, eligibility, local vectors and recurrence vectors for 60 candidates.
- The three score packets preserved the first 37 fields of every candidate. Full CUDA traces and decisions agreed with Rust `SourceBank` for nominal, donor and zero arms. The full-code map matched both nominal and donor observed decisions at their corresponding code indices. Each pair's maps agreed across all 65,536 codes after translating original positions to relative positions. The Rust selector independently checked one witness for each distinct map outcome.
- The first local custody extraction restored AppleDouble metadata into filesystem attributes. The raw tar was then verified on Linux, where all 111 individual source, input, executable and output members restored byte for byte. After the Wick verification, the lease was released. The first teardown command missed the required session argument. The corrected command archived and verified eight job and pod log files, then deleted the pod.

## Custody and restore

The Wick copy is `/home/callard/uor-codex/20261120-context-swap-30/device/custody-v1`.

| Artifact | SHA-256 |
| --- | --- |
| `remote-original.tar`, 84,428,800 bytes | `40dbd5bc24d37b9c9d1c09c789a8b324061c620d345be1ceaaae8b5011919060` |
| `dispatch/remote-original.sha256` | `2aec382179f187bf5b116f365abd8d0cf00e0e61c1a8dcd074ca034d267c91f0` |
| Remote `project-input.sha256` | `958e28d34bb800dd234467e993cedc212643384a9fa3e79278572edbd69be488` |
| Remote `score-input.sha256` | `846326235c137498d131e1530d91b42fb4eb53a0cacd8546ab0183712ec17fff` |
| Remote `score/capacity.map` | `e61c6fb6391982f75b45e0f3e4056474e9a6470e579aab8a99f2d1b1a2340e0e` |

Use GNU tar on Wick to restore and check both the raw tar and its members.

```sh
ssh wick
cd /home/callard/uor-codex/20261120-context-swap-30/device/custody-v1
sha256sum -c dispatch/remote-original.tar.sha256
sha256sum -c dispatch/down.sha256
restore=$(mktemp -d)
tar -xf remote-original.tar -C "$restore"
(cd "$restore" && sha256sum --quiet -c /home/callard/uor-codex/20261120-context-swap-30/device/custody-v1/dispatch/remote-original.sha256)
(cd host && sha256sum --quiet -c device/dispatch/host-evidence.sha256)
```

The host directory carries both GO decisions, pair and capacity summaries, 1,353-prefix input and replay records, and projection, scoring and release receipts. The laptop's harness archive for `vast-61560422` carries the original job scripts and logs.

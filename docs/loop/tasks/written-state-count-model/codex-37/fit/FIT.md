# [codex] Non-author FIT for D37 reserved-TRAIN identity fit

FIT for the staged identity-only A1 and A1s computation on the admitted Wick RTX 4090 under the shared lock. This review ran no GPU program. The admission covers no control fit, DEV-tune selection, DEV-test, or paired R* scoring.

| Reviewed object | SHA-256 |
|---|---|
| Sheet | `9109078855dba389a3f30765c64efe7b1152d3f3619358dabc54265bb010fa69` |
| Dispatcher | `8e7f9cfc949b6e67fa801b29128ad755672cdcfbf3b2f0830923f149b0ecc63f` |
| Production stage inventory | `d861520806e09debd02c0f125501c612573e6b20356ea252a8788447e8b930b0` |
| Production source inventory | `a5131a3e999d80b2641d802776a6be1f3d491de802bc1de7f8903d2c3d8f60ea` |
| Production host executable | `eeef61e2dfb0c946e0a99fb49151d9ec79f4e54e0dfad1c833348720487150d2` |
| Optimizer executable | `3b7c91ad698a2ef1eb81eca2d3c2f42fd9a188e1a55899bea827bc8bc0326aaf` |
| Finite optimizer result inventory | `1511096a1ad64bb9d64266bff26b2051a740084b83d0510551090730f01b04ab` |

All production files, pinned source files and finite result bytes verified on Wick. The finite verifier accepted each of the 97 Q12 scores and 717,024 and 1,195,040 Q32 words respectively, and matched both terminal bucket probes. The optimizer is the same admitted sm_89 image from that run.

The host binary embeds the following exact cubin payloads. I located each whole payload by byte comparison in the executable, checked ELF capability 8.9, and made an instruction census of each payload, including kernels outside the expected Pick invocation. All used integer, memory or control instructions, including ordinary's integer FLO.U32.SH. The census found no floating arithmetic, conversions or tensor instructions. The source builds the Pick device using the ordinary module and an identity-position snapshot verified by reading back table, gates, routes and position. Its sliced carried windows and traced forward pass expose geometric Pick words. The audit includes the additional embedded code that can be selected by device implementation. The native-vector cubin was absent from the executable.

| Image | Executable byte offset | SHA-256 |
|---|---:|---|
| ordinary | 3860927 | `a208e2937a341e6700c9199aded3be7b83bb1fce9d85a95c97338eb7d8a04f69` |
| rows | 1218829 | `02cde4b3a60e9de4808ad1f1c4821bdcf76afe6623d2b0ce3fd8db025c8a7a48` |
| decomposition | 4741874 | `0cd6bb767fe251eed332b2ebec3e084d5f3a01b30512d9ca2f20e784afb749ac` |
| baseline-reuse | 5719685 | `e2d85c3308d3dc47ad9e859ecd6c70ae66237f1b7c82157de3b1ad1f9a0bcf4a` |

I independently enumerated pinned original R as 15,951,644 UORT words, 4,060 BOS and 4,060 EOS, with zero framing violations. Its 15,947,584 scored targets exhaust the 2,232,661,872-byte count tape including its header. I independently rehashed the original tape to `add9aa306fd70d87e02048e2a2a32d23f6f9e9be89bb8ff8011c64690042f4dd` and verified the capture manifest identity. The fit generates causal half-open windows ending before the target, carries at most 192 original IDs since the last BOS, and sends the last prefix word as the device placeholder. It reads the actual target only while deriving the hit from the traced successor and while joining count words by original ordinal. It verifies sample native count and route words, monotonic ordinal mapping, typed class assignment and an injected alternate class at a distance boundary. Each full count row is consumed once and the consumed stream is rehashed. The closed-gate and positive-hit tests operate after weights have been checked against each CUDA target probe and reloaded from native wire bytes.

The two 120-byte-per-target tapes and the 16-byte-per-target Pick tape consume 4,082,581,632 bytes before their small headers. The optimizer stages one 1,913,710,080-byte row buffer at a time on each of host and GPU. Preparation keeps the 3,912,334,481-byte compact table, bounded transient batches, position windows, original IDs and Pick rows, while streaming the count tape and row outputs. The dispatcher requires 10 GiB available host memory, 8 GiB available device memory and 8 GiB destination space. Wick reported 32,228,128 KiB host memory available, 24,080 MiB device memory free, and 43,551,433,949,184 destination bytes available at inspection. The dispatcher verifies those resources immediately before use and requires a fresh output. Bash syntax passed. Parent identity-only GO governs dispatch.

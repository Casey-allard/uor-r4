# ORDER: cpu-training-pilot, deepseek-211 (compressed prompt VSA)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `4d1e705aa`.

## Goal

Test whether a compressed binary prompt signature can preserve or improve the
quality of the exact 4,096-bit prompt-reply index while reducing storage and
selection work. Calibration uses TRAIN only. The fresh seed-2778 evaluation
panel is generated only after FIT and GO.

## Fixed construction

The persisted `uor-r4.stack-prompt-reply-readout/1` schema and default exact
4,096-bit behavior remain unchanged. A first-party research constructor accepts
a width that is a multiple of 64 and one of two deterministic token mappings:

1. `modulo`: map token `t` to bit `t % width`;
2. `splitmix64`: hash `t` with SplitMix64 and map the result modulo `width`.

Query and stored prompt signatures use the same mapping. Jaccard remains
AND/OR popcount with integer cross-multiplication and lower-ordinal tie.

## Fixed calibration

TRAIN panel `tt-2711` supplies the normalized prompt/reply entries. A prompt is
held out from the inner selector when the first byte of
`BLAKE3("deepseek-211-inner-v1" || prompt_u32_le_bytes)` is less than 51,
approximately 20%. The remaining prompts form the inner library.

Each candidate is scored only by agreement with the exact 4,096-bit selector on
the inner held-out prompts. Rank by selected ordinal matches, then selected
reply matches, then smaller width, then fixed candidate order:

- 256 bits, modulo;
- 512 bits, modulo;
- 1024 bits, modulo;
- 1024 bits, SplitMix64.

The chosen configuration is reported and then frozen for the fresh panel.

## Arms

1. `compressed`: the TRAIN-selected compressed signature;
2. `exact`: default 4,096-bit binary vocabulary signature;
3. `cpu_oracle`: in-memory CPU `unigram_jaccard_ordinal_tie` rows.

## Data and power

- TRAIN `tt-2711`;
- fresh `cell=dev,dev`, seed 2778, protocol 2, 128 conversations;
- both public dual-c131 artifacts;
- at least 200 reference-reachable rows per artifact;
- three alternating fresh-process runs per artifact and arm;
- sealed deepseek-207 sidecars, whose bytes must remain unchanged.

## Bar

For both artifacts:

- the exact arm emits every reachable row with zero premature BOS and matches
  the CPU oracle on all reachable ordinals and reply SHA-256 values;
- the compressed arm emits every reachable row with zero premature BOS;
- compressed exact replies, correct openings and correct body positions are
  each no lower than the exact arm;
- compressed median prompt-selection throughput is at least 1.25x exact and
  at least 25,000 prompts/s;
- both arms repeat identically after timing removal;
- sidecar bytes and hashes remain unchanged;
- API complete-hit, cap-fallback and identity tests pass.

## Cost

Local CPU only. Cap: 30 minutes for the fresh-panel run, 2 GiB peak RSS and
32 MiB retained evidence. Calibration and focused tests are separately bounded
to five minutes.

## Done when

1. The configurable research path and focused tests pass without changing the
   default exact selector.
2. FIT and GO precede seed-2778 generation and the two-artifact comparison.
3. REPORT, compact evidence and program records merge to main.
4. Fresh-main verification and claim release complete.

## Cancel probe

Build and run focused tests, inspect both exact and compressed signature
widths, and verify the panel/output roots are absent. Stop before panel
generation if the default path, sidecar bytes or deterministic mapping drifts.

## Stop rule

Stop on source, sidecar, panel, artifact, selection, determinism or input drift;
an unreachable panel; any exact-arm CPU mismatch; any compressed premature BOS;
any aggregate quality regression; throughput below both floors; or any need to
change `UORLUT01`, serving kernels, CUDA, float training or Claude-owned source.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-211/ORDER.md

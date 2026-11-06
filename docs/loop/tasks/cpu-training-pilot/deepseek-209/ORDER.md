# ORDER: cpu-training-pilot, deepseek-209 (prompt VSA index)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `a4981948e`.

## Goal

Replace the serving selector's literal ordered-set intersection with a
discrete vocabulary-dimensional binary vector. Each prompt is represented as a
4096-bit set over token ids; bitwise AND and OR popcounts recover intersection
and union for exact Jaccard ordering.

The experiment must preserve every deepseek-207 selection before accepting any
throughput gain.

## Fixed construction

At sidecar load:

1. allocate `ceil(vocab / 64)` little-endian `u64` words per prompt;
2. set bit `token_id` for each non-ignored prompt token;
3. retain the original prompt tokens only for sidecar round-trip serialization.

At selection:

1. build the query signature with the same bit mapping;
2. compute intersection and union with bitwise AND/OR and `count_ones`;
3. use the existing integer rational ordering and lower-ordinal tie.

The persisted sidecar schema and selection result do not change. No matrix
operation, float path, artifact write or serving-kernel change is introduced.

## Arms

1. binary-VSA selector: deepseek-209;
2. literal-set selector: the merged deepseek-207 release binary from
   `a4981948e`;
3. in-memory CPU `unigram_jaccard_ordinal_tie` row oracle.

## Data and power

- sealed sidecars from deepseek-207 attempt 4;
- TRAIN `tt-2711`;
- evaluation panel `dd-2777`, 471 reachable rows;
- both public artifacts;
- three alternating fresh-process runs per arm;
- no new capability panel or reference labels are introduced.

## Bar

For both artifacts:

- all 471 promoted source ordinals and complete-reply SHA-256 values equal both
  the CPU row oracle and literal-set baseline;
- exact **39**, opening **131**, body **948**, emitted **471**, premature **0**;
- binary-VSA median selection throughput is at least **2x** the literal-set
  median and at least **25,000 prompts/s**;
- sidecar bytes and SHA-256 remain unchanged and reload exactly;
- OpenAI complete-hit, cap-fallback and identity tests pass;
- no `UORLUT01`, kernel, CUDA, float-training or Claude-owned source changes.

## Cost

Local CPU only. The experiment is capped at 30 minutes, 2 GiB peak RSS and
32 MiB retained evidence.

## Done when

1. The binary signature representation and focused tests are implemented.
2. FIT and GO precede the alternating two-artifact comparison.
3. REPORT, compact evidence and program records merge to main.
4. Fresh-main verification and claim release complete.

## Stop rule

Stop on source, sidecar, panel, artifact, selection, parity, throughput or
determinism drift; fewer than 200 reachable rows; any decision difference from
the CPU oracle or literal-set baseline; throughput below both bars; or any
prohibited source change.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-209/ORDER.md

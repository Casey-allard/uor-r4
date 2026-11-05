# ORDER: cpu-training-pilot, deepseek-190 (exact scale-serving preflight)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `3ac40d6d5`.

## Goal

Add a read-only exact-serving preflight for the three model widths in
Claude's `scale-probe`: 256, 512 and 1,024. For each width and representative
contexts 16, 64 and 192, generate deterministic synthetic FBS artifacts using
the recipe's whole-table-nearest readout, export them through `UORLUT01`,
reload them, and compare the independent CPU forward with the integer serving
session over the complete token sequence.

This is D84 CPU development research. It is not GPU qualification, corpus
training, a scale-slope result or a model-quality claim.

## Why this is next

- Claude's draft scale-probe claim registers widths 256, 512 and 1,024 on the
  real-text corpus. Its GPU run is the qualification route.
- `cpu-fbs-forward-bench` hardcodes width 256, MLP 384, six layers and context
  192, and never runs `audit_served_forward`.
- Existing real-artifact exact-audit coverage is width 256. Synthetic complete
  audit coverage is smaller. No checked preflight covers the three registered
  widths, their exported bytes and the served decode cost against context.
- This unit catches output-path width/context regressions and gives Claude a
  CPU reference before the scale run spends GPU time. It does not claim that a
  width fits GPU training.

## Fixed construction

Add `cpu-scale-serving-bench` in `uor-r4-cpu-train` with two read arms:

1. `projection`: no Hamming or memory tables.
2. `full`: Hamming reads at depth 1 and the final layer, plus opening/reply
   memory. Use `width / 64` read heads, matching the historical width-512
   requirement of eight 64-bit head codes.

Both arms use the whole-table-nearest writer selected by the current
`--objective cell` recipe.

Every synthetic trunk uses:

- vocabulary 4,096;
- widths 256, 512 and 1,024;
- MLP 384;
- six layers;
- context 16, 64 and 192;
- eight writer cells;
- deterministic seeds bound in the output record;
- the same token sequence for both arms at a width/context cell.

For every artifact:

1. Export `UORLUT01` with `export_fbs_trunk_uorlut01`.
2. Parse it through `audit_served_forward`.
3. Require exact query-coordinate, token and kept-cell parity between
   `CpuFbsForward` and `IntegerStackSession`.
4. Require matching CPU/serving decision hashes.
5. Record artifact bytes, serialized table payload bytes, decoded resident
   table bytes, timing and peak RSS.

Run each parity cell three times. All decision hashes and parity counts must
repeat exactly. The final pass may measure timing; the first two do not.

Negative controls:

- width 2,048 is refused by the existing writer bound;
- a truncated artifact is refused;
- a token outside vocabulary is refused;
- a token sequence longer than the artifact context is refused;
- the same seed must reproduce artifact bytes and decision hashes.

## Output

Write one JSON record per arm and depth/width/context cell to the selected
output path. No artifact or generated model is committed.

Required fields include:

- artifact SHA-256 and bytes;
- seed and complete synthetic shape;
- manifest serialized/resident bytes;
- three-run parity and decision hashes;
- CPU-oracle and integer-serving tokens/s for the timed pass;
- peak RSS.

## Data and power

- no corpus is read;
- 3 widths x 3 contexts x 2 arms = 18 cells;
- three deterministic passes per cell;
- AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads;
- no GPU and no float-training or Claude-owned source.

## Bar

The unit passes only when:

- all 18 cells export and parse;
- every cell has zero parity mismatches and matching decision hashes;
- every cell repeats exactly across three passes;
- all four negative controls fail closed;
- peak RSS stays below 2 GiB;
- the full run completes within 20 minutes.

A failure is an instrument or supported-width result, never a language
negative. Do not retune a shape after reading a result.

## Done when

1. The benchmark and focused tests are implemented.
2. A FIT-reviewed pre-registration and fresh D67 GO are recorded.
3. The registered run completes and compact JSON evidence is produced.
4. REPORT, notebook, item/current state and delivered evidence record the
   supported widths, exactness, byte cost and decode throughput.
5. Reviewed merge, protected queue merge, fresh-main verification and claim
   release complete.

## Stop rule

Stop on source, shape, parity, determinism, output-root or evidence drift; any
need to edit Claude-owned float-training, serving or exporter code; any parity
mismatch; peak RSS at or above 2 GiB; or total wall time above 20 minutes.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-190/ORDER.md

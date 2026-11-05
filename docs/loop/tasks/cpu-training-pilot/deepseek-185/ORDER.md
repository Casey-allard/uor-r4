# ORDER: cpu-training-pilot, deepseek-185 (isolate table-anchor cost)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `1c5209f8e`.

## Goal

Separate the two effects hidden in deepseek-183's table-wide port-anchor
result. The same FBS trunk will run with independent input and writer tables:

- raw input / raw writer;
- anchored input / raw writer;
- raw input / anchored writer;
- anchored input / anchored writer.

This is a CPU-only diagnostic on the existing public panel. It does not change
serving, CUDA, float training, the artifact format or Claude-owned source.

## Why

Deepseek-183's combined table anchor reduced admitted/top-1 from 245/181 to
120/55 on artifact seed 1 and from 171/136 to 29/10 on seed 2. The shared table
is both the input embedding and the writer row source, so the run could not say
whether changed recurrent inputs or direct writer-row mismatch caused the loss.

## Fixed construction

1. Parse the source artifact and the table-anchored artifact from deepseek-183.
2. Add a CPU diagnostic forward mode that permits distinct input and writer
   tables while preserving the default one-table behavior byte for byte.
3. Feed identical token prefixes and targets through all four arms.
4. Derive target admission and kept cells from the resulting query.
5. Score tokens against the selected writer table.
6. Report query mutation, admitted cells, top-1/top-8, port-root distance and
   deterministic query/token/kept-cell signatures.

The diagnostic may not claim serving parity for hybrid arms because
`UORLUT01` stores one table. The raw/raw arm must retain existing CPU/serving
parity.

## Tests

- default forward remains unchanged;
- input-only override changes input rows but leaves writer scoring rows;
- writer-only override leaves input-derived queries unchanged and changes only
  writer scoring;
- combined override equals the table-anchored artifact;
- mismatched table shapes, Hamming reads, turn memory and non-native heads fail
  closed;
- all arms are deterministic.

## Bar

This is an instrument. It succeeds if all four arms execute on both public
artifacts with complete row coverage and deterministic outputs. The REPORT must
assign the observed admitted/top-1 loss between:

- input-only;
- writer-only;
- interaction/combined.

No arm is promoted or called a training result.

## Done when

1. The diagnostic API, driver and focused tests are implemented.
2. A FIT-reviewed two-artifact diagnostic run completes.
3. REPORT, compact evidence, notebook and roadmap record the attribution.
4. Reviewed merge, fresh-main verification and claim release complete.

## Stop rule

Stop on source/input drift, nondeterminism, malformed artifacts, hybrid
serving-parity claims, non-native heads, Hamming/memory records, or any need to
edit serving, CUDA, float-training, exporter or Claude-owned source.

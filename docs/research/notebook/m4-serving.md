# Notebook: integer serving (M4)

This notebook is for results about running the trained model cheaply: serving it with integers and lookup tables only, with no floating-point arithmetic and no multiply instruction (decision D11). Its first rows, from 25 October 2026, qualify the D11 artifact scorer, an instrument. No step-6 result, and no serving result against a registered bar, has a row yet, and the language base (M1) is not yet met.

## Results

Sources: [`archive/catalog.jsonl`](../../../archive/catalog.jsonl). Verdict words are defined on the [format page](./README.md#the-five-verdict-words).

| Date | What ran | Number | Bar | Verdict | Artifact |
|---|---|---|---|---|---|
| 2026-10-26 | **D11 export of a SpiralCore/FBS float trunk, integer side** (d11-exporter-float-trunk, #2820, claude-122). Laptop finite controls on a hand-built trunk: vocabulary 256, width 32, MLP 32, 2 recurrence layers, native vector writer keeping 2 cells. No GPU and no trained model. | **Control:** the exported container's integer session matches the quantized float reference on all 256 single tokens and an 8-token sequence: same kept cells, same token, Q12 query within 2 LSBs, with 0 near-tie exceptions in 264 positions. **Float reference:** its plain and quantized forms agree on 224 of 256 tokens. **Audit:** zero-multiply `FULL PASS` on the rlib (40 call-graph functions) and on `uor-r4-stack` (97). | the integer forward reproduces the float reference on every input of a small vocabulary (the item's step-2 finite control); the audits stay green | positive (finite controls only: not an export gap of a trained model) | [REPORT](../../loop/tasks/d11-exporter-float-trunk/claude-122/REPORT.md) |
| 2026-10-25 | **GPU qualification of the D11 artifact scorer** (m1-d11-artifact-scorer, #3248, claude-116). The integer CUDA port of `evaluate_prefix` scored the 14,287-row DEV panel on the exported A8 FrozenZero `continuation-05248` artifact, on 1 RTX 5090 for 288 pod-seconds (about $0.08). The checkpoint is a stand-in, not step 6's. | Parity on 14,287 of 14,287 rows (read vector, admitted cells, top-8 and prediction), with 0 refused. The stand-in's totals are top-1 741, top-8 1,345 and admitted 4,251 of 14,287. They qualify the instrument only. They are not an export gap, because the checkpoint has no sealed `eval-corpus` DEV evaluation. | every row bit-identical to `evaluate_prefix`, fixed in the [prep sheet](../../harness/prep/m1-d11-artifact-scorer-2026-10-25.md) | positive (instrument qualification) | store `results/claude/d11-artifact-scorer-2026-10-25/pod/` (`SHA256SUMS`); [REPORT](../../loop/tasks/m1-d11-artifact-scorer/claude-116/REPORT.md) |
| 2026-10-25 | **`toks` on the same exported artifact,** on the laptop (Apple M1, 4 performance cores, 16 GiB, macOS 27.0, AC power). It ran the step-6 protocol through `IntegerDiscreteModel`: 1 warmup and 20 timed runs, a 192-token prefill and a 64-token decode. | **Decode:** 13,259 tok/s median, which is 4,826 µs per 64 tokens; p95 13,176 tok/s. **Prefill:** 2,967,175 tok/s median, which is 64 µs per 192-token prompt; p95 2,765,893 tok/s. All 21 decode runs wrote the same reply. | none registered (qualification check) | inconclusive (no bar registered; it qualifies the protocol and is not step 6's measurement) | store `results/claude/d11-artifact-scorer-2026-10-25/laptop/toks-laptop.json` |

## Before 25 October

The audit catalog holds 42 records for this line. Between 12 October and 19 October 2026 it has one record, and that record is tooling, not a result:

- 2026-10-18, [wasm-serving-runtime](../../labs/wasm-serving-runtime-2026-10-18/README.md): an outside contributor's browser and Node WebAssembly integer serving runtime was merged after hardening. Its multiply-free audit passes, but the SIMD kernels are not integrated and the demo cannot be run from a clean clone. This is a build record, so it has no row.

Serving results before 12 October (for example the d11-serving record of 10 October) predate this notebook. Find them in [`archive/CATALOG.md`](../../../archive/CATALOG.md) under line `m4-serving`, and read the record before relying on its verdict.

## What a row here will need

A row needs a number measured through the exported integer artifact, a bar fixed beforehand, and the export gap: the difference between the offline float candidate and the integer artifact on the same evaluator (`CHARTER.md`, section 3.7). Serving must be integer on the user's device (section 3.6). The M1 closing bar also asks for tokens per second to be reported with the integer top-1 result (`CHARTER.md`, section 4.8).

## Go deeper

- [Serving and export](../../model/serving-and-export.md) and the [lookup-serving mechanism](../../mechanisms/lut-serving.md).
- [Format and verdict words](./README.md).
- [`CHARTER.md`](../../../CHARTER.md), sections 3.6, 3.7 and 4.8.

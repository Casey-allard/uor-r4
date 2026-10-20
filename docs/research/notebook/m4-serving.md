# Notebook: integer serving (M4)

This notebook is for results about running the trained model cheaply: serving it with integers and lookup tables only, with no floating-point arithmetic and no multiply instruction (decision D11). It has no rows yet. No serving run since 12 October 2026 has produced a number against a bar, and the language base (M1) is not yet met.

## Results

Sources: [`archive/catalog.jsonl`](../../../archive/catalog.jsonl). Verdict words are defined on the [format page](./README.md#the-five-verdict-words).

| Date | What ran | Number | Bar | Verdict | Artifact |
|---|---|---|---|---|---|
| | *No result rows yet.* | | | | |

## Why the table is empty

The audit catalog holds 42 records for this line. Between 12 October and 19 October 2026 it has one record, and that record is tooling, not a result:

- 2026-10-18, [wasm-serving-runtime](../../labs/wasm-serving-runtime-2026-10-18/README.md): an outside contributor's browser and Node WebAssembly integer serving runtime was merged after hardening. Its multiply-free audit passes, but the SIMD kernels are not integrated and the demo cannot be run from a clean clone. This is a build record, so it has no row.

Serving results before 12 October (for example the d11-serving record of 10 October) predate this notebook. Find them in [`archive/CATALOG.md`](../../../archive/CATALOG.md) under line `m4-serving`, and read the record before relying on its verdict.

## What a row here will need

A row needs a number measured through the exported integer artifact, a bar fixed beforehand, and the export gap: the difference between the offline float candidate and the integer artifact on the same evaluator (`CHARTER.md`, section 3.7). Serving must be integer on the user's device (section 3.6). The M1 closing bar also asks for tokens per second to be reported with the integer top-1 result (`CHARTER.md`, section 4.8).

## Go deeper

- [Serving and export](../../model/serving-and-export.md) and the [lookup-serving mechanism](../../mechanisms/lut-serving.md).
- [Format and verdict words](./README.md).
- [`CHARTER.md`](../../../CHARTER.md), sections 3.6, 3.7 and 4.8.

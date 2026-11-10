# ORDER: cpu-training-pilot, deepseek-266

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md). Base: `6fcf57f78`.

## Goal

Make the CPU exporter carry the active Hamming ask heads and question carry in one
command, so a saved conversational checkpoint can produce its complete served
`UORLUT01` artifact on the laptop without a pod snapshot or a second splice tool.

This is a host-only export repair. It trains nothing and changes no model,
serving arithmetic, bar, capability claim or Claude artifact.

## Why now

The current L-aC artifact path is fragmented:

- `cpu-fbs-export` reconstructs the trunk and base Hamming reads on the laptop;
- it refuses `d{depth}.ask`, `d{depth}.carry` and `carry_rows`;
- `hamming-base-copy` strips those tensors so the exporter can run;
- a GPU `fbs-snapshot` builds the ask entries;
- run-specific logic splices them into the laptop artifact.

That means the active conversational model needs a GPU merely to serialize CPU
sidecars, and the complete export is not one repository command.

## Work

1. Extend the CPU Hamming reader to validate and read:
   - `d{depth}.ask` at the checkpoint's vocabulary and width;
   - the optional `carry_rows`;
   - the readout read's optional `d{depth}.carry` gains;
   - the existing hold tensors with an explicit refusal where the exporter cannot
     serve them.
2. Emit the optional `hamming_ask` object in the snapshot JSON using the existing
   integer-crate types and schema.
3. Have `cpu-fbs-export` add those ask/carry sections to the artifact with the
   existing `add_hamming_ask_uorlut01` transform and parse/read back the result.
4. Preserve byte-identical output for checkpoints without ask heads and for the
   existing no-ask export control.
5. Add tests for:
   - ask tables and readout carry;
   - a carry on a non-readout read, a missing gain, or mismatched dimensions being
     refused;
   - a non-zero hold remaining an explicit unsupported export until its serving
     format is complete;
   - a complete CPU export parse/read-back containing the ask and carry.
6. Document the one-command path and retire `hamming-base-copy` from the required
   workflow in the CPU exporter documentation.

## Done when

1. `cpu-fbs-export` accepts a saved checkpoint with Hamming ask heads and readout
   carry and writes a parsed artifact containing both.
2. A no-ask checkpoint still exports byte-for-byte as before.
3. A synthetic ask/carry checkpoint yields the same snapshot and artifact bytes
   through the direct CPU exporter as the existing GPU-snapshot-plus-`fbs-export`
   construction path, up to identities that name the producing tool.
4. Focused CPU-crate tests, strict Clippy, rustfmt, report checks and loop gates
   pass.
5. The exact-head review and merge queue pass, and the squash commit is verified
   on fresh `main`.

## Cancel probe

1. `origin/main` has the current ask/carry schema and
   `add_hamming_ask_uorlut01`.
2. No open PR owns `crates/uor-r4-cpu-train/src/fbs_snapshot_cpu.rs` or
   `crates/uor-r4-cpu-train/src/bin/cpu-fbs-export.rs`.
3. The change can be expressed inside the existing `UORLUT01` schema.

## Stop

- Do not change the artifact format or serving arithmetic.
- Do not train, score a capability panel, launch a pod or touch a lease.
- Do not modify Claude's branch, checkpoints, run files or decision review.
- Stop and re-scope if the checkpoint format or ask/carry schema moves underneath
  this unit.

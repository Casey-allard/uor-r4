# Cancel probe: claude-258 (train as served)

Lab: claude. Run 2 Nov 2026 at the base `7663cf2cef3e67fb386996c87337c0b91349a4a6`.

1. **Does training already round?** No. `grep -n round_` over
   `crates/uor-r4-float-train/src/in_space_long.rs` and `src/bin/train-in-space.rs` finds no
   rounding call, so the build part stands.
2. **Which loop does (r-drop) run, and do the export's roundings fit it?** `train-in-space`
   trains through `in_space_long::run_long` (the binary's last line). (r-drop) is the protocol's
   base recipe (r) (`--objective cell --hamming-read 4 --turn-memory 8 --turn-memory-reply 1`, …)
   plus `--turn-memory-dropout 500`. So its loop holds three things:
   - the trunk, a `StackModel`;
   - the Hamming reads, a `HammingReads`, with no type spin;
   - the turn memory, a `TurnMemory`, with its reply table.

   `fbs_export::round_tables_as_exported(&StackModel)`, `round_hamming_as_exported(&HammingReads)`
   and `round_memory_as_exported(&TurnMemory)` are public. Each one reads its tables to the host
   and writes the rounded values back with `Tensor::from_vec(.., tensor.device())`, so it runs on
   the loop's CUDA device as it is.

   The export carries only the trunk, the Hamming read and the turn memory
   (`check_checkpoint_reads`), and the recipe has no other read.

**Class:** neither a mechanism nor an instrument; the build. Training as served is a change of
number format, with no new loss and no new parameter. It needs no new instrument: the export's
own rounding functions do all the rounding, and the export's own checks judge the result.
Neither of the ORDER's stop conditions holds, so the unit goes ahead.

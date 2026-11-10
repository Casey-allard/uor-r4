# REPORT: cpu-training-pilot, deepseek-266

Lab: deepseek. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **The CPU exporter refused the active ask-head checkpoint inventory.** It
  recognized only the base Hamming tensors, so `d2.ask`, `d6.ask`, `carry_rows`
  and `d6.carry` were rejected as extras. The active L-aC export therefore used
  `hamming-base-copy`, a GPU snapshot and a separate splice.
- **The held ask-write form still has no serving representation.** Reusing the
  generic extra-tensor refusal would hide that known gap, so the reader now
  refuses `hold_rows` and `.hold` by name.
- **The type-spin form is also unsupported by the integer engine.** It is now
  refused by name instead of being reported only as an unexplained extra
  tensor.
- **The first local pass had no sealed L-aC checkpoint.** The synthetic
  checkpoint exercised the file parser, snapshot JSON, integer quantizer,
  artifact transform and read-back path; the final review later supplied the
  published seed-2 checkpoint for the real equality check.
- **`cargo clippy --all-targets -D warnings` is not green at the base.** It
  reports pre-existing lint failures in `prompt_reply_rank.rs`,
  `suffix_arbitration.rs` and `joint_branch.rs`. The changed library passes
  strict Clippy when those three unrelated pre-existing lint classes are
  allowed.

## What changed

- Extended the CPU Hamming parser to validate and carry:
  - one ask table per active Hamming depth;
  - the readout read's carry reach and gain table;
  - existing copy key, mixed copy keys, margins and five base tensors;
  - fail-closed rules for partial asks, carry without asks, carry gains without
    a row count, the unsupported held write and the unsupported type spin.
- Added a typed complete-export helper that:
  - preserves the old snapshot bytes exactly when no ask head exists;
  - adds `hamming_ask` to the snapshot only when ask heads exist;
  - quantizes asks to Q12 and carry gains to Q16;
  - attaches them with `add_hamming_ask_uorlut01`;
  - parses the final artifact before either output is written.
- Made `snapshot_from_saved_checkpoint` refuse an ask checkpoint rather than
  silently dropping asks from its older return type. `export_saved_checkpoint`
  is the complete path.
- Documented the one-command CPU export in `crates/uor-r4-cpu-train/README.md`.

Size: **534 code/build/test lines added and 43 changed**, plus **20 exporter
README lines** before this report. The ORDER gave no numeric estimate.

## Measurements

- Focused Hamming/export tests: **25 passed, 0 failed**.
- Full `uor-r4-cpu-train` library suite: **495 passed, 0 failed, 5 ignored**.
- The synthetic ask/carry checkpoint exports through
  `export_saved_checkpoint`, carries a two-entry `hamming_ask` snapshot array,
  and reads back an artifact with asks at both depths and `carry_rows = 32`.
- The direct CPU export of the published L-aC seed-2 `step-016384` checkpoint
  reproduces its served artifact byte for byte:
  `2741d9383a80c300b90ca2d5cbc1e06a9b664d913971f84ddbc2916a1625a430`.
- A no-ask synthetic trunk exports byte-identically to the previous
  `export_fbs_trunk_uorlut01_with` result, and its snapshot is byte-identical to
  the old plain `FbsTrunkSnapshot` serialization.
- The release `cpu-fbs-export` binary builds and prints its unchanged usage.

These are host-side serialization measurements. No model was trained or
evaluated and no capability number changed.

## Checks run at the head

- `cargo test --offline -p uor-r4-cpu-train --lib`:
  **495 passed, 0 failed, 5 ignored**.
- `cargo test --offline -p uor-r4-cpu-train --lib fbs_snapshot_cpu`:
  **25 passed, 0 failed**.
- Direct CPU export of the published L-aC seed-2 `step-016384` checkpoint:
  artifact SHA-256
  `2741d9383a80c300b90ca2d5cbc1e06a9b664d913971f84ddbc2916a1625a430`,
  byte-equal to the served artifact.
- `cargo build --offline --release -p uor-r4-cpu-train --bin cpu-fbs-export`:
  passed.
- `cargo clippy --offline -p uor-r4-cpu-train --lib --no-deps -- -D warnings
  -A clippy::type_complexity -A clippy::needless_borrow`: passed. The allowances
  cover only the pre-existing unrelated sites named above.
- `rustfmt --check` on the changed Rust file: passed.

## Skills used

None.

## Still open

- **Turn memory is still outside `cpu-fbs-export`.** It currently rejects
  `turn_memory.safetensors` and `turn_memory_reply.safetensors`; the GPU
  `fbs-snapshot` path carries them. A separate host-only unit should close that
  gap using `fbs_memory_snapshot` semantics.
- The text artifact still does not bind tokenizer and dialogue-protocol identity.
- Session restore still refuses turn memory and Hamming type-head payloads.

## Plain summary

The active conversational model uses ask tables and a question carry. The
laptop exporter could serialize the trunk but not those tensors, so producing a
complete served file required taking the model to a GPU and manually combining
two files afterward. This change teaches the laptop exporter to read, validate,
quantize and attach those tensors itself. A checkpoint without ask heads still
produces exactly the old bytes, while a checkpoint with ask heads now produces
one complete snapshot and one complete integer artifact in a single command.

# ORDER: d11-exporter-float-trunk, claude-122

Lab: claude. Item: [d11-exporter-float-trunk](../../../../roadmap/items/d11-exporter-float-trunk.md) (#2820). Base: `c695ad21a`.

## Goal

The item's done-when reads:

> A float-trunk snapshot exports to D11 with the audits green and the export gap measured inside the band. The trunk is a SpiralCore/FBS recurrence trunk with the Hamilton gate and the native vector writer head, as Stage 3 trains it. The synthesis (Stage 4) adds tokens per second, reported for decode and for context separately.

claude-120 (#3253) and claude-121 (#3254) released the item and fixed its scope; their D68 owner logs are on #2820. The owner's direction, in Claude's chat on 26 Oct: "Pick the best held item that doesn't touch crates/uor-r4-discrete, run your own pre-run/decision review per CHARTER §5.1, and start it."

This unit does the item's laptop work: the design note, the export path for the trunk Stage 3 trains, and its finite controls. The Stage 4 GPU measurement needs a new sheet with a fresh FIT, a two-seed retrain with saving, and a D67 pre-run review, and follows in a later unit. So this PR says `References #2820`.

## Cancel probe (run first)

Class: instrument. Run at `c695ad21a`; all must hold:
- **The export path is still missing.** `crates/uor-r4-integer/src/stack/format.rs` and `session.rs` contain no SpiralCore/FBS or vector-writer code. **Holds:** 0 matches each.
- **Nobody else is on it.** No open PR other than this unit's draft #3255 touches `crates/uor-r4-integer/`, `crates/uor-r4-float-train/`, `crates/uor-r4-training/` or the item. **Holds.**
- **The converter's dependency does not exist yet.** `crates/uor-r4-float-train/Cargo.toml` does not depend on `uor-r4-integer`; the converter adds that path dependency, in that direction only. **Holds.**

## Files in scope

- `crates/uor-r4-training/**` (read only; `stack_export.rs` stays untouched)
- `crates/uor-r4-integer/**`
- `crates/uor-r4-float-train/**`
- `docs/harness/prep/d11-exporter-float-trunk*`
- `docs/labs/d11-exporter-float-trunk/**`
- `docs/loop/tasks/d11-exporter-float-trunk/**`
- `docs/research/notebook/m4-serving.md`

## Done when

1. **A design note is posted on #2820,** before any code (item step 2). It must cover:
   - the snapshot the float module hands the exporter;
   - how a SpiralCore/FBS projection becomes per-block `(atom, signed-axis state)` sections and a gather-and-sign integer step, with no dense table;
   - how the native vector writer head becomes a bounded candidate read;
   - the Hamilton icosian snap and the geometric recurrence;
   - why nothing uses softmax, squashing or a dense matrix product;
   - the finite controls.
2. **A failing test comes first.** A tiny hand-built SpiralCore/FBS + Hamilton + writer trunk is exported, and its integer forward must match its float reference on every input of a small vocabulary: exact top-1, plus a stated tolerance on scores.
3. **The export path makes that test pass:**
   - the UORLUT01 sections and integer runtime steps in `crates/uor-r4-integer`, behind `serving-wasm`;
   - a converter in `crates/uor-r4-float-train`, from a saved `StackModel` to the snapshot, depending on `uor-r4-integer` and never the reverse.
4. **Checks:**
   - the zero-multiply audit (`scripts/audit_zero_matmul_serving.py --stack`) stays green on the rebuilt rlib and `uor-r4-stack`;
   - `~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm` and `-p uor-r4-float-train` pass;
   - `uor-harness check` and the loop gates pass.
5. **A REPORT with negatives first,** listing as open:
   - the Stage 4 sheet and its FIT;
   - the two-seed retrain with saving;
   - the GPU evaluator or parity proof;
   - the `lut-serving.md` row (claude-121's scope note).

## Stop and ask if

Stop and decide, never stop and wait.

- **The integer format cannot hold an FBS projection without a dense table, or without a §3 change.** Then stop the code and record a D68 card on #2820. If §3 itself would have to change, post one `[claude, for Casey]` note on #2028.
- **The work outgrows one PR.** Then land the design note and the failing-then-passing finite control for the FBS projection first, and leave the writer head and the converter to the next unit.
- **The converter would need the integer crate to depend on float-train or candle.** Then stop and redesign, because the dependency must run one way.

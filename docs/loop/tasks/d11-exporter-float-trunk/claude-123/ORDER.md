# ORDER: d11-exporter-float-trunk, claude-123

Lab: claude. Item: [d11-exporter-float-trunk](../../../../roadmap/items/d11-exporter-float-trunk.md). Base: `611cfb059`.

## Goal

The item's done-when reads:

> A float-trunk snapshot exports to D11 with the audits green and the export gap measured inside the band. The trunk is a SpiralCore/FBS recurrence trunk with the Hamilton gate and the native vector writer head, as Stage 3 trains it. The synthesis (Stage 4) adds tokens per second, reported for decode and for context separately.

claude-122 (#3255, merged as `611cfb05`) landed the integer side: the `uor-r4.lut-stack/4` container, the float reference and the integer session. This unit builds the **float-train converter**: from a `StackModel`, as Stage 3 trains and saves it, to an `FbsTrunkSnapshot`. That is item step 3 ("convert"). Stage 4 (the sheet, FIT, retrain, export gap and tok/s) is the next unit.

**The design.**
- **Route read-back.** A float FBS projection routes each output block by a GPU argmin over learned query points (`spiralcore_fbs.cu`, `sc_fbs_route`). The route depends on the queries only. The converter reads routes back with the model's own GPU projection, by projecting one coded carrier: input slot `i = atom*8 + c` holds `i + 1`.
  - Each output value `±(atom*8 + c + 1)` then names its atom, axis and sign exactly.
  - Every block must read one atom with 8 whole signed axes, or the converter refuses: a two-atom mix is not a gather.
  - The state is the integer group element whose signed-axis codes match.
  - No host argmin is computed, so the routes cannot differ from the GPU's by rounding.
- **Identical tables.** A host test pins the float geometry's 64 signed-axis rows and 16 orbit roots to the integer crate's `AXES` and `ROOTS`, in order. The writer's cell ids `a*16+b` depend on the root order.
- **Copied tables.** Everything else is copied by name, with layouts checked against the integer snapshot: gains, conv taps `[tap][channel]`, biases, decay offsets, `embedding.weight` (also the writer table) and `final_norm.weight`.
- **Refusals.** The converter refuses any configuration the integer side cannot serve, each for its own reason: not all-`r`, table gates, no rotation, the U(1) rotation control, a non-Hamilton gate, no native head, or a pointer head.

## Cancel probe (run first)

Cancel if a float-to-`FbsTrunkSnapshot` converter already exists on main or in an open PR. Result at `611cfb05`:
- `git grep -l FbsTrunkSnapshot origin/main -- crates` lists only `crates/uor-r4-integer/src/stack/{format,mod,tests}.rs`;
- no open PR touches `crates/uor-r4-float-train/`.

**Not cancelled (instrument check: none needed).**

**Scope note.** `crates/uor-r4-float-train` gains a path dependency on `uor-r4-integer`. This is the design note's §1 edge, which does not exist yet. That changes one line of the generated `Cargo.lock`, which the item's scope omits. The change is mechanical, and the pod builds need it, because they use `--locked`. The exact-head reviewer confirms the lock diff is only that edge, and pr-check's scope rule (advisory) will WARN on it.

**Feature boundary (found while reading).** The integer crate's `stack` module, which holds the FBS snapshot and `export_fbs_trunk_uorlut01`, compiles only under `serving-wasm` (`native_geometry.rs` includes `serving_root.rs`). That feature also compiles the `model`, `sampling` and `tables` modules. D29's "presence counts" compliance rests on the feature staying off in the native graph (`docs/model/serving-and-export.md`). So the converter never enables it. Instead:
- The three plain snapshot types move to an ungated `uor_r4_integer::fbs_snapshot`. `stack::format` re-exports them, so existing paths and tests are unchanged.
- `float-train` depends on the integer crate without features, as `uor-r4-training` already does. It writes the snapshot as JSON, and `serde_json` writes `f32` values so they round-trip exactly.
- A small integer bin (`serving-wasm`, the serving build) turns that JSON into the `.uorlut` file.
- Only the float-train GPU test enables `serving-wasm`, through a dev-dependency, to export, parse and run the session. Resolver 2 keeps dev-dependency features out of normal builds.

## Files in scope

- `crates/uor-r4-training/**`
- `crates/uor-r4-integer/**`
- `crates/uor-r4-float-train/**`
- `Cargo.lock` (the one dependency edge above, generated)
- `docs/harness/prep/d11-exporter-float-trunk*`
- `docs/labs/d11-exporter-float-trunk/**`
- `docs/loop/tasks/d11-exporter-float-trunk/**`
- `docs/research/notebook/m4-serving.md`

## Done when

1. **Converter.**
   - `uor-r4-integer` exposes its Cl(0,6) signed-axis table and orbit roots read-only, and holds the snapshot types in the ungated `fbs_snapshot` module.
   - `crates/uor-r4-float-train/src/fbs_export.rs` turns a `StackModel` into an `FbsTrunkSnapshot` that `export_fbs_trunk_uorlut01` accepts.
   - Two small bins: a float-train one converts a `StackModel::save` directory to snapshot JSON, and an integer one (`serving-wasm`) exports that JSON to a `.uorlut` file.
2. **Host tests (laptop):**
   - the tables are identical;
   - the coded-carrier decode recovers a synthetic gather exactly, and refuses a mix, an out-of-range value, a non-integer value and a code outside the group;
   - each configuration refusal is pinned to its own reason.

   Each test is shown to fail with its code set aside.
3. **GPU test (one pod, finite, on a random-init small trunk):**
   - the read-back routes reproduce the GPU projection bit for bit, on random carriers, for every projection of every layer;
   - the snapshot exports, parses, and runs an integer session;
   - recorded as a reading, not a bar: the float GPU hidden state and token, against the converted trunk's float reference and integer session.
4. Audits green, `loop gates --full` green, and a REPORT that names what Stage 4 still owes.

## Stop and ask if

Stop and decide; never wait.
- **The float and integer tables differ.** Classify it: an instrument fault in an accessor is repaired. A real numbering difference means the converter maps codes, never states, and the REPORT says so.
- **The GPU read-back finds a projection that is not a signed gather.** Refuse that configuration and record which one.
- **The work outgrows one PR.** Land the converter and the host tests. The GPU test then follows in the next unit.

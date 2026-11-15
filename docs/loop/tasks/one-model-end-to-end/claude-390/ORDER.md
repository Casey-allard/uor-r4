# ORDER: one-model-end-to-end, claude-390 (serve the native Q31 first-pick read in `IntegerStackSession`)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `origin/main` at `8cef91f6b`. That holds claude-389 (#4001, the operator and its CUDA twin) and codex-13 (#4004, `Q31LaneGates`).

## Why this unit

claude-389 built `q31_first_pick_read` and showed that it reproduces the float read's choice and write on claude-386's saved states. Its REPORT's first open item is attaching the operator to the serving session. That gives D85's "same typed operators" its serving half.

Casey answered "Start it here?" with "Yes, start it" (15 Nov 2026).

**What this unit cannot do yet.** Two facts bound it. First, no trained read can reach an artifact: no exporter carries a float `vector_read` into the integer artifact (`fbs_export::check_checkpoint_reads_carrying` refuses it). Second, a float trunk exported to integers gives a different stream at the read layer, so it could not reproduce claude-389's picks bit for bit.

So this is a **serving and artifact contract unit**:
- the read is served in the session;
- its fields are stored losslessly in the artifact;
- its served transition equals the operator's batch definition, word for word, on the session's own stream.

Training the read's fields natively, so that a trained read exists to serve, is a later unit.

## What is built

**1. An incremental form of the operator,** in `crates/uor-r4-native-vector/src/read.rs`.
- `Q31ReadHistory` holds one sequence's address keys (16 i64 words per earlier row) and its token ids.
- `Q31FirstPickRead::step(&self, history, x_row, token, table)` returns `(pick, out_row)` for the next row and appends that row's key and token.
- The step computes the address with the same functions as `apply`: the rotation, the fold, the u64 L1 score, the `<=` tie to the latest row, and the payload rounding. There is no second copy of the arithmetic.
- Row 0 reads nothing. The same refusals apply: an out-of-domain word, a token outside the table, a width mismatch.

**2. The read in the session,** in `crates/uor-r4-integer/src/stack/`.
- `IntegerStackModel` gains an optional `Q31SessionRead { depth, read: Q31FirstPickRead }`. It is attached only to an FBS trunk whose width equals the read's.
- It is refused alongside the address table or the Hamming reads. Their order at the same seam is not defined yet.
- In `stack_step_fbs`, after layer `l` with `l + 1 == depth` (after that layer's MLP add, which is where the float read sits), the session:
  - converts `b.x` (i32 at `2^-16`) to i64 words;
  - calls `step` with the writer table's Q12 rows as the payload table (i16 → i64, exact);
  - writes the out words back.
- The stage-timed copy of the loop does the same. The existing test that pins the two loops equal covers it.
- **Scales.**
  - The read's weights are Q31 words meaning `w · 2^(35 − g)`. Then `(w·e + r) >> (31 − g)` lands the payload on the residual's `2^-16` from a Q12 table row.
  - An out word of `+2^31` cannot be stored in i32. It is a `StackError`, refused rather than clamped, and tested.
- **Per-prompt state.** The session keeps a `Q31ReadHistory`.
  - `reset` clears it.
  - `save_state` / `restore_state` refuse a session with a read attached (a `StackError`). The `uor-r4.stack-session/1` schema has no field for the history, and adding one is listed under Still open.
- Decision traces record the read's pick per step.

**3. The artifact,** in `crates/uor-r4-integer/src/stack/format.rs`.
- There is a new schema, `uor-r4.lut-stack/14`.
  - A header record holds `{ depth, width, bytes }`.
  - A 64-aligned table `q31.read` holds `Q31FirstPickRead::to_bytes()` exactly.
- `add_q31_read_uorlut01` follows the `add_address_uorlut01` pattern. Loading attaches the read.
- **The number `/14` is chosen to leave `/13` to codex's planned `Q31LaneGates` slice** (codex-13's Still open). A trunk with both a lane-gate record and a read record is refused until a later unit defines that combination.
- Parse refuses a `/14` header without the record, a record without the table, a byte length that `from_bytes` refuses, and a read alongside address or Hamming records.

## Native-space five items (SKILL)

- **State schema.** The learned fields are the read's query gates and key gates (`width / 4` Q31 quaternions each), its `width` Q31 weights, its gain shift and its depth. The per-prompt carried field is `Q31ReadHistory`: 16 i64 address words per earlier row and the token ids.
- **Shared operator path.** There is one Rust source definition, `read.rs`. Batch `apply` and incremental `step` call the same inner functions. The CUDA twin from claude-389 covers `apply`. The session runs on CPU and WASM through the same crate, which is dependency-free.
- **Replay oracle.** For a synthetic FBS trunk with a read attached, a prompt is served token by token. Then the stream rows `x` at the read seam are captured from the session's own trace, and `apply` is run on them as one batch. The served picks (`u32`) and out words (`i64`) must equal the batch result word for word. Raw words are compared, never rounded values. The fixtures are fixed and seeded.
- **Forward, gradient, update.** Forward only. There is no native gradient or update for the read here, so this is a serving and contract unit, not a training run.
- **Serialization.** The read's bytes are stored unchanged in the artifact (`to_bytes`, version 1). Save → load → save is byte-identical, and a served prompt is bit-identical before and after a round trip.

## Checks (no GPU, no sheet: builds and finite tests, §4.5)

1. `step` against `apply`: over seeded fixtures (several widths, gains 0, 4 and 31, near ties, saturation), `step` row by row equals `apply` word for word.
2. The session replay above, on a synthetic FBS trunk, at two depths.
3. **A planted fault.** Changing one key-gate word by 1 LSB in the artifact's `q31.read` table changes at least one served pick or out word on the near-tie prompt. This shows the replay can fail.
4. The artifact round trip, and every parse refusal listed above.
5. The existing session tests pass unchanged. In particular, a trunk without a read serves bit-identical tokens, and the `/1`–`/12` byte pins hold.
6. Builds: `cargo test -p uor-r4-native-vector`, `cargo test -p uor-r4-integer --features serving-wasm` (the stack tests), and a `wasm32-unknown-unknown` build of the serving path.
7. `uor-harness check --only harness.native_space_contract`.

## Files in scope

- `crates/uor-r4-native-vector/src/read.rs`, and `lib.rs` for the exports only.
- `crates/uor-r4-integer/src/stack/{session.rs,format.rs,mod.rs,tests.rs}`, plus any small module those need. `crates/uor-r4-integer/Cargo.toml` gains the dependency-free `uor-r4-native-vector` only if it is not already a dependency.
- `crates/uor-r4-lut/src/format.rs` only if its D10 schema twin must list `/14`.
- `conformance/generated/stack-contracts.json` only if its generator requires a refresh.
- This folder, and one row in `docs/research/notebook/float-training.md`.

## Stop list

- No GPU, no pod and no training.
- No change to the read operator's arithmetic or to the CUDA twin.
- No change to served tokens for any artifact without a read.
- Nothing that takes `/13`, and no edit to `Q31LaneGates`.
- No owner path.

## Cancel probe

At base, no session, artifact or exporter carries a vector read (the scout's map at `8cef91f6b`): `rg "Q31FirstPickRead|vector_read" crates/uor-r4-integer` finds nothing. No open PR touches the session's read seam.

## Done when

1. `step` equals `apply` on every fixture, and the session replay is bit-identical at both depths.
2. The planted fault changes the served output, and the artifact round trip and its refusals pass.
3. Every existing integer-crate test passes, and the WASM build is clean.
4. The REPORT and the notebook row are written.
5. The PR goes through a fresh exact-head review, the queue and `close --verify`.

# REPORT: one-model-end-to-end, claude-390 (serve the native Q31 first-pick read in `IntegerStackSession`)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #4006. Base `origin/main` at `8cef91f6b`. Code at `116bf357f` (the incremental step), `112bad341` (the session read and the `/14` artifact), `e3f549902` (merge of `origin/main` with codex-14's `/13` lanes) and `985ba83f6` (the review's fixes: a refused step poisons the session; refusal tests name their rule).

This is a serving and artifact contract unit, with builds and finite tests only, so there is no sheet and no reading map (§4.5). The read is now served. `IntegerStackSession` calls claude-389's `q31_first_pick_read` through its incremental form, `step`, after a chosen layer of an FBS trunk. The read's fields are stored in a `uor-r4.lut-stack/14` artifact as the operator's own bytes.

On the session's own stream, the served transition equals the operator's batch definition word for word. That holds at both depths (1 and the last layer) and at gains 0, 4 and 31, for every pick and every out word.

## What did not work, first

1. **The planted key-gate fault of the ORDER's check 3 does not change the served read.** The ORDER asked that changing one key-gate word by 1 LSB change at least one served pick or out word. On the fixture (synthetic FBS trunk, width 32, two layers, a 16-token prompt with repeats, read at depth 2, g = 4), **0 of 32** one-LSB key-gate faults change anything served. The sweep raised the fault to 2^k:
   - every word is silent up to **2^24**;
   - the first pick moves at **2^25**, on 1 word of 32.

   **Cause: scale, not a missing check.** The largest residual word the read sees is **372,018 (about 2^18.5)** against the 2^31 range the gates are scaled for. Each gate product is rounded by 2^31. A 1-LSB gate change therefore moves a product by about 2^-12.5 of that rounding step, so it almost never reaches an address word. Beyond that, the picks have margin.

   claude-389's 1-LSB control passed because its converted states fill the Q31 range.

   What does bite is a **1-LSB weight fault at g = 31**: it changes the out words and the served step. The test now pins that. For the key gates it asserts only that some fault from 2^0 to 2^30 moves the served read, and prints the first `k` that does as a measurement. Every artifact byte is also bound by a SHA-256, so a flipped byte in `q31.read` is refused at parse. (Still open 1.)
2. **This is forward only.** The read has no native gradient or update, so by the native-space SKILL this is a serving and contract unit, not a training run. No trained read exists to serve: no exporter carries a float `vector_read` into the artifact.
3. **No claude-389 pick parity through an exported trunk.** A float trunk exported to integers gives a different stream at the read layer. The oracle here is the session's own stream against `apply`, as the ORDER bounds it.
4. **`save_state` does not refuse; only `restore_state` does.** The ORDER said both refuse. The crate's existing rule is that restoring refuses and saving stays infallible: the turn memory, the Hamming type heads and the place read all refuse only on restore. Saving is harmless, and a restore would silently differ, because `uor-r4.stack-session/1` has no field for the read's history. Changing `save_state`'s signature would touch every caller for no gain.
5. **The decision trace does not record the pick.** A dedicated opt-in trace does (`set_q31_read_trace` / `q31_read_trace`: the pick, the row handed to the read, and the row written back). Adding a field to the decision trace's layer struct would touch every literal that builds it. The replay oracle reads the dedicated trace.
6. **A refused step is not rolled back.** An out word of `+2^31` is refused, and so is any operator refusal. The step then returns an error after the layer states and the read's history have taken the row. The first version only documented that the caller must `reset`; the session review found a later step would serve from that half-advanced state. Now the refusal sets a session flag, and every later step, plain or stage-timed, is refused ("reset the session before it serves again") until `reset` clears it. The state is still not rolled back. (Still open 4.)
7. **The builder subagents could not build the session half.** The first builder delivered the incremental `step`. The second handed the session wiring back with nothing written, because its injected context started near the scope mark. I wrote the session read and its tests in the main session.

## What changed

- **`crates/uor-r4-native-vector/src/read.rs`** (`116bf357f`):
  - `Q31ReadHistory` holds one sequence's 16-word key addresses and token ids, with `new`, `len`, `is_empty` and `clear`.
  - `Q31FirstPickRead::step(&self, history, x_row, token, table) -> Result<(u32, Vec<i64>)>` serves row `t = history.len()` and appends its key and token only on success.
  - `apply` and `step` share the address (`q31_read_address`), the scan with the `<=` tie to the latest row (`first_pick`), the payload (`add_payload`) and the word and table checks. There is no second copy of the arithmetic, and no change to `apply`'s results or refusals.
- **`crates/uor-r4-integer/src/stack/q31_read.rs`** (new): the read's artifact and session side.
  - `StackQ31Read`: the header record `{ table, depth, width, bytes, sha256 }`.
  - `Q31SessionRead { depth, read }`.
  - `read_q31_read_uorlut01`.
  - `ServedQ31Read`: the read, its depth, and the writer table widened to i64 once at load.
  - `Q31ReadStep`: one traced step.
  - The module doc states the scale convention: the weights are Q31 words meaning `w · 2^(35 − g)`, so `(w·e + r) >> (31 − g)` lands a Q12 table row on the residual's 2^-16 with no rescale.
- **`stack/session.rs`:**
  - The FBS model gains `q31_read: Option<ServedQ31Read>`, attached at load.
  - `stack_step_fbs` and the stage-timed copy `stack_step_fbs_timed` call the read after layer `l` when `l + 1 == depth`, after that layer's MLP add.
  - The session keeps a `Q31ReadHistory` and an opt-in trace. `reset` clears both, and `restore_state` refuses.
  - A refused read sets `q31_read_refused`; both FBS step loops refuse while it is set, and `reset` clears it.
  - `fbs_trunk_image`, and so `fbs_image`, returns `None` for a trunk with a read, so no evaluator serves the trunk without it.
- **`stack/format.rs`:**
  - `STACK_FBS_Q31_READ_SCHEMA = "uor-r4.lut-stack/14"`, with the `q31_read` header and container field and the metadata field. The metadata leaves the field out when the stack has no read, so an artifact without one inspects as before.
  - `Container::parse` derives `/14` from the record first, and refuses a read alongside any other optional record (address, Hamming, memory, point, chain, the `/12` Q31 state, or codex's `/13` lane gates). It validates the table through `q31_read_image`.
  - `add_q31_read_uorlut01` follows `add_address_uorlut01`. It appends a u32 table of the read's exact bytes, with the last word zero-padded, keeps every earlier section's bytes, and reads the result back.
  - `/13` is left to codex's lane gates.
- **`stack/mod.rs`:** the module and its public items.

Not changed:
- the read operator's arithmetic and the CUDA twin;
- `Q31LaneGates`;
- `crates/uor-r4-lut/src/format.rs`, whose D10 twin lists only the schemas that crate writes (not `/11` either);
- `conformance/generated/stack-contracts.json`, which names neither `/11` nor `/12`;
- every existing test.

## Measurements

The fixture is `synthetic_fbs_trunk_with(20261115, FBS_SYNTH_SMALL)` with context 24: width 32, two layers, vocab 256. The prompt is the 16 tokens `5 9 5 9 17 5 9 17 3 5 9 17 3 5 200 9`. Reads are drawn from a fixed LCG.

| Check | Result |
|---|---|
| `step` row by row vs `apply`: widths 16/64/256, g 0/4/31, repeated rows for exact ties (`step_matches_apply_on_seeded_windows`) | equal word for word |
| Session replay: served picks and out words vs `apply` on the trace's own stream, depths 1 and 2 × g 0/4/31 (`a_served_q31_read_equals_the_batch_operator_on_its_own_stream`) | 6 / 6 equal word for word. Each has a row reading past its predecessor, and served steps that differ from the unread trunk |
| A read with all-zero weights vs the plain trunk | every `NativeStep` identical |
| The stage-timed loop vs the served loop, with a read | identical at every step |
| Save → load: re-add of the re-read image | the same artifact bytes, and the same served steps and trace |
| One-LSB key-gate faults, 32 words, g = 4, depth 2 | 0 / 32 change the read. The first change is at 2^25 (1 / 32) |
| One-LSB weight fault, g = 31 | changes the out words and the served step |
| Largest residual word at the read | 372,018 (about 2^18.5) |
| An out word of `+2^31` (weights 2^31, g = 31) | `StackError::Numerics(… +2^31 …)` |
| `restore_state` with a read; `reset` then re-serve | refused; identical to the first run |
| After a refused step: three more plain and three stage-timed steps; then `reset` | all six refused with the reset message; after `reset` the steps before the refusal serve identically, and the refused step is refused by its own `+2^31` rule again |

Refusals tested. Each test now matches the refusal's message to the rule that should refuse, not only that something refused:
- **Adding a read is refused for:** a second read, depth 0, a depth past the layers, and another width.
- **Parsing refuses:**
  - a flipped byte at four offsets of the table;
  - `/14` without the record;
  - the record under `/12`;
  - another table name;
  - a record without its table;
  - a byte length the table does not hold;
  - depth 0, a depth past the layers (3 on the two-layer fixture), or another width;
  - a wrong digest;
  - a read beside an address record (on the add path, and on the parse path with a real address record set in a `/14` header).

## Checks run at the head

At `985ba83f6`, over the merge `e3f549902`, unless a line names another commit. Laptop, `~/.cargo/bin/cargo`, `CARGO_INCREMENTAL=0`.

- `cargo test -p uor-r4-native-vector`: 28 passed (at `112bad341`; the merge and `985ba83f6` do not touch the crate).
- `cargo test -p uor-r4-integer --features serving-wasm --lib`: 660 passed, 0 failed, 1 ignored. That includes the 8 new `q31_read` tests. Every earlier test is unchanged, including the `/1`–`/12` byte pins and `the_stage_timed_fbs_step_is_the_served_step`.
- `cargo build -p uor-r4-native-vector --target wasm32-unknown-unknown` and `cargo build -p uor-r4-integer --features serving-wasm --target wasm32-unknown-unknown --lib`: clean.
  - The `uor-r4-stack` bin does not build for wasm32 (`verify_step_objdump_zero_mul_zero_float`, `measure_step_phases`: native-only items). This is not touched here.
- `cargo clippy -p uor-r4-integer --features serving-wasm --lib --tests`: no finding in the touched files except a warning at `format.rs:337`, in a line from 29 Oct. The three clippy errors are in `tests/openai_loopback.rs:1658`, `src/place_asks.rs:1421` and `src/stack/kernels.rs:1806`, which this unit does not touch.
- `cargo fmt --check -p uor-r4-integer`: no diff in the four touched files (at `112bad341`); `rustfmt --check` on `q31_read.rs` at `985ba83f6` is clean. Existing diffs elsewhere in the crate are not touched.
- **Negative gates** (each sabotage set aside, run, and restored byte for byte):
  - `step` appending the query address instead of the key: `step_matches_apply_on_seeded_windows` fails.
  - The read's write-back disabled: the replay and the planted-fault tests fail.
  - The served loop's read call removed: the replay, timed-loop, planted-fault and `+2^31` tests fail.
  - At `985ba83f6`, one sabotage per run, each confirmed applied by `git diff --stat` and restored by `git checkout`:
    - the poison flag never set: the refused-step test fails;
    - the read served at layer 0 whatever its depth: the replay test's depth assertion fails;
    - the `restore_state` refusal disabled: the restore test fails;
    - `reset` keeping the read history: both reset tests fail;
    - the `+2^31` out-word guard disabled: the out-word and refused-step tests fail;
    - the width check in `StackQ31Read::check` disabled: the round-trip test fails;
    - the address arm of parse's combination refusal disabled: the round-trip test fails on the read-beside-address case.
    - One sabotage (the width check) first missed its line, and its green run was discarded and redone.
- **Fresh reviews of `112bad341`** (two reviewers, the spec and the diff only):
  - session and artifact: one medium finding, that a refused step left the session serving from a half-advanced state. Fixed in `985ba83f6`; the lows are in Still open 9.
  - tests: REQUEST CHANGES, for refusals checked by `is_err()` alone, no parse-path combination test, an unpinned depth placement, a parse depth of 9 on a two-layer stack, a threshold assertion on a measurement, and a restore test accepting any `Shape`. All six fixed in `985ba83f6`, with the five suggested negative gates run above.
- **Fresh re-review of the fix delta and the merge's `format.rs` resolution** (at `1921a2b1a`): APPROVE. Every FBS step path (`step`, `step_native`, `measure_fbs_step_stages`, `measure_fbs_step_parts`) passes the flag check before its first mutation, and `/13` is refused beside a read on both paths. Three lows: the poison test could pass vacuously if the first step refused (now asserts `at > 0`), a doc line had moved onto the wrong function (moved back), and no `/13` parse test (Still open 10). The q31_read tests pass after both fixes.
- `uor-harness check-formats`: 8 formats ok.
  - `uor-harness check --only harness.native_space_contract` matches no registered rule (0 rules), so the ORDER's check 7 has no rule to run. The full `uor-harness check` has 0 errors.
- `uor-harness loop pr-check --local`: see the PR.

## Skills used

- `native-space-cycle`: the five items are in the ORDER.
  - **Met:** the state schema, the shared operator path (one Rust definition, with `apply` and `step` on shared inner functions), the replay oracle (raw words, the session's own stream) and serialization (the exact `to_bytes`, digest-bound).
  - **Not met:** forward, gradient and update. This is forward only.
- The loop's ORDER/REPORT form. Builder subagents in their own worktrees, with gates re-run here.

## Still open

1. **The read's gates use about 18 of 31 bits on the served residual.** The residual reaches about 2^18.5 at the read, so gate changes below about 2^13 never reach an address, and on this fixture picks move only from 2^25. A native update rule for the gates needs this:
   - either a fixed left shift of the served row into the read's domain (an operator change, with its CUDA twin);
   - or updates sized above that quantum.

   The update-rule unit decides. claude-389's states filled the Q31 range, so its 1-LSB control passed.
2. **The native update rule.** The read's gates, weights and gain need a native gradient and update on the same Q31 words. The trained gain must reach about ×16 (claude-389 item 2).
3. **A trained read in the artifact.** Once the read trains natively, `add_q31_read_uorlut01` stores it. The first check then is claude-389's picks, reproduced from a served prompt.
4. **A refused step is refused, not rolled back** (item 6 above). The session now refuses every later step until `reset`. A rollback of the layer states and the read's history would let it continue instead.
5. **Combinations.** A read is refused beside the address table, Hamming reads, memory, point, chain, the `/12` Q31 state and codex's `/13` lane gates. Their order at the same seam is not defined. A read beside the `/13` lane gates is the next combination to define.
6. **The session schema has no field for the read's history.** `restore_state` refuses until `uor-r4.stack-session/1` gains one.
7. **The ORDER's check 7 names no registered rule.** `harness.native_space_contract` is not a rule id in `uor-harness check`.
8. **The claude-389 review's minor findings stay open:**
   - the id refusal is narrower than the ORDER says;
   - the parity example passes on a dump with no rows past row 0;
   - `g = 31` does not run on the device;
   - no test pins the key window offset;
   - the dump is written before the cut pass;
   - `q31_hamilton_rotate`'s u64 wrap.

   Note that `step` has the same narrower id rule as `apply`: an out-of-table token is refused only when a read reaches it.
9. **The session review's low findings:**
   - each served step re-runs `admit_words` and `table_vocab` over the whole writer table, a speed cost on every token that a load-time check would remove;
   - `StackQ31Read::check` runs twice on parse (in the header block and again in `q31_read_image`);
   - `save_state`'s doc does not say that a session with a read saves a state it cannot restore.
10. **No parse-path test sets codex's `/13` lane record beside a `/14` read.** The code refuses it in both the parse combination block and `add_q31_read_uorlut01`, but only the address case is pinned. The crate has no `add_` helper for the `/13` record, so the test needs a lane artifact from codex's exporter. (Fresh re-review of the fix delta, low.)

## Plain summary

The model's look-back read now runs inside the integer serving session that answers prompts. It sits after a chosen layer, and the model file stores it exactly, guarded by a checksum. On a test model, the served read matched the read's reference definition on every position and every number, at both depths tried. Damaged files are refused.

The test also found a scale problem. The numbers the read sees while serving are about 5,800 times smaller than its settings are scaled for, so small changes to the read's address settings do nothing. Training the read in integers, the next step, has to fix that scale or size its updates to match.

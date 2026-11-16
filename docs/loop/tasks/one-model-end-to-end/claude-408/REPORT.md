# REPORT: one-model-end-to-end, claude-408 (the Pick read: an exact XOR + popcount hard read in the discrete model, the same in CUDA, the CPU oracle and serving)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **No learning claim** [claude]. This is a finite kernel and oracle unit. Nothing here trains a Pick model or shows that the Pick read recalls a fact the corpus taught. Every model is at checkpoint 0, or has gates and tables set by hand.
- **A Pick model can be scored and chosen on the CPU, but not applied or trained there** [claude]:
  - `CpuDiscreteProposalApplier::from_bytes` refuses a Pick artifact (`discrete_route.rs`, `pick_applier_refuses`);
  - `CpuDiscreteModelState::from_scorer`, the trunk reader and `flattened_read_state` refuse Pick;
  - the CPU corpus driver refuses a Pick artifact (commit `231b3a688`).

  So the CUDA apply of a Pick proposal has no CPU byte comparison yet. The scorer's Gate and Table moves and the chooser are compared. The writer that serializes them is not.
- **The first CUDA Pick build broke the bucket writer's qualified text** [claude]. `bucket_source::generate` copies `discrete.cu` up to "// Statuses added". The Pick lines sat in that prefix, and the BUCKET gated line had been widened to Pick. That moved the generated source away from the sha `7222ef05…` that the committed GPU receipts qualified (`docs/labs/m1-faster-writer/qualification-build-{a,b}.json`). The fix:
  - the Pick lines in `discrete.cu` sit between `// PICK-BEGIN` and `// PICK-END` markers;
  - `generate` strips them and refuses nested, unterminated or orphan markers;
  - the BUCKET gated line is back to Flock and Relation;
  - `writer_bucket_device` refuses a Pick model before any launch, in all seven entry points.

  The generated text is byte-identical to the qualified one again (`generated_bucket_module_keeps_helpers_and_adds_seven_entries`).
- **The serving loader refused a prefix-holonomy Pick artifact** [claude]. It did not admit the fifth table, `discrete.position_frames`. The CPU-against-serving agreement test found this, and it is fixed (commit `66f4e3eea`).
- **No Pick-specific out-of-orbit test** [claude]. The ORDER's refusal list names an out-of-orbit block. This unit adds no test for it. The refusal rests on two existing guards:
  - the exporter's orbit check (`orbit_root_id`), shared by every read mode;
  - the CPU oracle's table, which holds root indices `0..16`, so an off-orbit block cannot be written there.
- **The export test cannot tell a frame word from a candidate word** [claude]. In this geometry `candidate_to_frame` is the identity on `0..16` (`d11_export.rs:755`). So the frames-section test pins the words through the map but would not catch a swapped map.
- **My first T2 expectation was wrong** [claude]. It asked for all 16 loss words of a non-level-0 gate to be equal. Lanes past 3 are `u64::MAX` (absent) on both sides, so the test now checks lanes 0–2 equal and lanes 3–15 absent.
- **Four tests that already fail on `main`** fail here too [claude]. This branch does not touch the code they test:
  - `apply_calls_are_confined_to_bounded_fixture_and_admitted_transition` (`uor-r4-discrete` lib) trips on `native_vector_device.rs`;
  - `snapshot_loader_delegates_position_step_bounds_to_cuda_validate` (`tests/position_device_controls.rs`), as claude-407 recorded;
  - `fbs::tests::ask_heads_at_both_hooks_match_serving_without_carry` and `fbs::tests::readout_ask_carry_matches_serving_over_a_multi_row_window` (`uor-r4-cpu-train` lib), which fail at `origin/main` too.
- **The ORDER named the test file `pick_read.rs`** [claude]. It is `pick_read_cuda.rs`, compiled only with `test` and `cuda`, beside `cpu_oracle_parity.rs`. `cpu_oracle_parity`'s `Reading`, `compare_forward` and `compare_proposals` become `pub(crate)`, so the Pick test reuses the same comparer.
- **The fresh review found three things to fix, now fixed** [review]:
  - The first student-batch test checked forward words and the trace only. It printed `moved` without asserting it, so it would pass even if the read changed no row. It now asserts `moved`. It also compares loss words and choices with the CPU scorer over claude-404's fixed student parameter list, with a shifted-target planted negative (run 3).
  - The bucket writer's evaluate path admitted the batch before refusing Pick. Every other entry point refuses first; this one now does too.
  - The D11 artifact scorer instrument (`docs/labs/m1-d11-artifact-scorer`) matches `DiscreteReadMode` exhaustively and stopped compiling once Pick was added (E0004). It now refuses a Pick read. Checked with `--target aarch64-apple-darwin`: it fails without the arm and compiles with it.
- **The planted faults are not the ORDER's "shifted pick"** [review]. The fixture negative swaps the transport word. The loss-word negatives shift the CPU targets. Both show the comparisons react, but neither plants a wrong pick with the same targets.
- **Parts of the ORDER's planted-copy test are thinner than written** [review]:
  - the CUDA planted-copy test is one row, not every row;
  - it does not check the greedy token;
  - "every gate 0 leaves the output equal to the read-off" is covered only through the 264-row oracle comparison, which includes gates `[0,0]`.

  Serving parity compares read vectors and greedy tokens, not pick positions.
- **Pick gates above level 0 are dead coordinates** [review]. The gate inventory is still blocks × stages, and Pick reads only level 0. So most Gate proposals cost a scoring pass and cannot change a loss. On flat losses the chooser keeps lane 0 (asserted in `pick_scorer_refuses_route_and_scores_gate_and_table`), so they never commit a change.
- **Two builder subagents stopped early** [claude], at their context marks, on a 9,000-line file. Their committed parts were reviewed and taken; the CPU scorer tests, the applier refusal and the CUDA loss-word parity test were written in the main session.

## What changed

- `crates/uor-r4-discrete/src/discrete.cu`: `MODE_PICK` and `pick_vector`. Pick is dispatched first in `read_vector`, and a Route proposal in Pick mode is refused with status 9. The read:
  - XORs and popcounts 2-bit root codes;
  - picks the latest position on a tie;
  - adds the gated successor;
  - snaps with `snap_{1+|gate|}`.

  It writes a trace record per row and block: pick, distance, ties, gate + 1 and the output root.
- `crates/uor-r4-discrete/src/lib.rs`: `ReadSpec::Pick`, its validation and serde. Position transport 1 or 3 only.
- `crates/uor-r4-discrete/src/device.rs`, `d11_export.rs`: the spec words. The artifact carries `read_mode: "pick"`, `read_position` and the `discrete.position_frames` section.
- `crates/uor-r4-discrete/src/bucket_source.rs`, `device/writer_bucket_device.rs`: the PICK marker strip and the bucket refusal, as above.
- `crates/uor-r4-discrete/src/driver_source_pin.rs` and the `prime_zeta` copies: the pin moves to `6ad76406…`.
- `crates/uor-r4-discrete/src/pick_read_cuda.rs` (new) and `Cargo.toml`: the CUDA tests. `uor-r4-core` and `uor-r4-integer` are dev-dependencies.
- `crates/uor-r4-cpu-train/src/discrete_route.rs`:
  - `CpuDiscretePick`, the CPU oracle, with `from_parts`, `from_bytes`, `gate`, `gate_count` and `set_gate` over all-level gates;
  - `Pick` in the proposal mode and model;
  - the scorer and chooser load `"pick"`, score Gate and Table, and refuse Route;
  - the applier, the trunk paths and the corpus driver refuse Pick.
- `crates/uor-r4-integer/src/discrete_d11.rs`: `DiscreteReadMode::Pick` in the serving reader, with the frames table.
- Route, Flock and Relation are unchanged in behaviour. Their CUDA parity tests pass unchanged (below).

## Measurements

All CUDA runs are on wick's RTX 4090, with real `nvcc` (`UOR_NVCC_ARCH=compute_89`) and a release build [claude].

**Run 1** (job `c408-run1`, head `125ca5e0e`, 16 Nov 2026):

- `pick_read_cuda`: 5 of 5 passed, in 4.58 s.
  - Every prefix: 33 prefixes × 2 transports × 4 gate settings = 264 rows. The CUDA read and trace equal the CPU oracle word for word, and a swapped-transport negative is caught.
  - Planted copy: on `[5,9,2,3,4,6,5,9]` the pick is position 1, at distance 0, with 1 tie.
  - Identity-step holonomy equals identity transport.
  - The bucket writer refuses a Pick model.
  - Student batch: claude-404's stream, `tokens.u16` sha256 `a16a4412…3238`, 15,471,412 tokens. One 64-row batch from eligible target 6,779,969, at the full model size. Transport words 3 and 1: 64 rows each, 0 mismatches, and the read moved the output.
- Device position and Flock tests: 7 of 7.
- `cpu_oracle_parity`, the existing Route and Flock CUDA/CPU parity: 3 of 3, in 1,287.95 s.
  - Fixtures: 0 mismatches for Route and Flock.
  - Student stream, two steps each: Route 48 and Flock 64 parameters. 2,048 forward words, 768 loss words and 48 choices, with 0 mismatches.
  - Planted target shift: 48 loss mismatches of 48 parameters.

**Run 2** (job `c408-run2`, head `c4b254c02`, 16 Nov 2026):

- `pick_read_cuda`: 10 of 10 passed, in 7.87 s. Run 1's five pass again, and five more:
  - all 256 root pairs: the CUDA distance is half the root L1;
  - a Route proposal in Pick mode is refused;
  - the exported Pick artifact, loaded by `IntegerDiscreteModel`, gives the CUDA read vector and greedy token on 2 transports × 33 prefixes;
  - Gate counterfactuals: each candidate lane's loss equals the forward loss of the model moved to that gate;
  - CUDA against the CPU scorer and chooser from the same `export_d11` bytes, per transport:
    - 528 forward words, 608 loss words and 38 choices (32 VectorBlocks and 6 gates);
    - 6 Route parameters refused on both sides;
    - 0 mismatches;
    - the CPU pass on shifted targets is caught.
- Bucket writer tests (`bucket` filter): 7 of 7.

**Run 3** (job `c408-run3`, head `2ef4ce000`, after the review fixes): `pick_read_cuda` 10 of 10 in 673.73 s, and bucket 7 of 7.

The student-batch test now runs per transport word, 3 and 1:
- read and trace: 64 rows, 0 mismatches, `moved` asserted;
- against the CPU scorer and chooser, over 64 listed parameters:
  - 2,048 forward words, 768 loss words and 48 choices (32 VectorBlocks and 16 gates);
  - 16 Route parameters refused on both sides;
  - 0 mismatches;
- the shifted-target negative on the Gate parameters is caught.

**Laptop, CPU oracle** (`cargo test -p uor-r4-cpu-train --lib pick`): 13 of 13. These cover:
- the L1 = 2·popcount identity;
- the planted copy and the latest-tie rule;
- holonomy frames and the `from_parts`/`from_bytes` refusals;
- `set_gate` moving only level-0 reads;
- agreement with `IntegerDiscreteModel::evaluate_prefix` on every prefix;
- scorer query equal to the oracle read at s14 3 and 1;
- Route refused; Table and Gate giving 16 loss words; the applier refusal.

Negative gates, each run once and restored:
- with the applier's Pick refusal removed, `pick_applier_refuses` fails;
- with the Pick gate move removed (`set_gate` not called), `pick_scorer_refuses_route_and_scores_gate_and_table` fails.

## What the numbers say

- **The XOR + popcount distance is the model's own L1 root distance.** All 256 root pairs agree on CUDA and on the CPU, so the read needs no approximation and no learned key.
- **One read, three places, one answer.** The CUDA kernel, the CPU oracle and the serving reader give the same read vectors on every fixture prefix. CUDA and the CPU oracle also agree on one 64-row batch from the D87 student stream at full size. On the fixtures, the served artifact gives the same greedy tokens as the CUDA forward.
- **The training signal reaches the Pick gates.** Each Gate counterfactual equals the forward loss of the model moved to that gate. CUDA and the CPU scorer give the same loss words and choices for every Table and Gate parameter (run 2).
- **This does not show the read helps.** Whether training finds gates and tables that make the pick recall the right successor is the next question. It needs the conjunction memory and the readout objective first (PLAN.md, the 16 Nov order).

## Checks run at the head

The laptop suites ran at `c4b254c02`. The review fixes after it change only:
- the CUDA test file;
- the bucket evaluate order;
- one CPU test assertion;
- the scorer instrument.

After the fixes, the CPU Pick tests (13 of 13), the stub-`nvcc` check and the instrument check were rerun. The CUDA tests were rerun on wick in run 3.

- G1, the CUDA tests on wick's 4090, real `nvcc`: `pick_read_cuda` 10 of 10 and bucket 7 of 7, at `2ef4ce000` (run 3). Run 1 also gave device 7 of 7 and `cpu_oracle_parity` 3 of 3, at `125ca5e0e`; the later commits do not change the Route or Flock paths those tests cover.
- G2, the laptop `uor-r4-discrete` suite, `--lib --bins --tests --no-fail-fast`: lib 361 passed and 1 failed, plus 28 other targets green. The only failures are the two `main` failures above.
- G3, the laptop `uor-r4-cpu-train` lib: 521 passed and 2 failed. The failures are the two fbs tests above.
- G4, the laptop `uor-r4-integer` lib: 185 of 185.
- G5, `cargo check -p uor-r4-cpu-train -p uor-r4-training --all-targets`: clean.
- G6, the stub-`nvcc` `cargo check -p uor-r4-discrete --features cuda --all-targets`: clean.
- G7, the negative gates: two CPU fixes, each set aside and shown to turn its test red (Measurements).
- G7b, `docs/labs/m1-d11-artifact-scorer` instrument, `cargo check --locked --target aarch64-apple-darwin --all-targets`, in its own target folder: clean, and E0004 with the fix set aside.
- G8, the ORDER commit `1785dbbb2` is the first of this unit's commits.
- `git diff --check` is clean.
- `loop pr-check --local`: run before queueing.

## Skills used

- `scope-guard`: Route, Flock, Relation and the qualified bucket text are unchanged; no training run, no pod.
- `native-space-cycle`:
  - one read mode in the existing snapshot;
  - one operator, written three times behind one contract;
  - replay through the CPU oracle's words;
  - the artifact carries the mode.

## Still open

1. **A CPU applier for Pick**, so a committed Pick proposal can be compared byte for byte with the CUDA apply, and the CPU corpus driver can step a Pick model.
2. **Load-time and test gaps the review named** [review]:
   - a Pick-specific out-of-orbit refusal test;
   - a frames-section test with a non-identity `candidate_to_frame`;
   - the serving loader admits any frame below 64, not only the 17 step frames (CUDA behaves the same, so parity holds);
   - a 19-table trunk artifact with Pick is accepted though nothing writes one;
   - `evaluate_prefix_traced` returns no Pick trace;
   - a planted wrong-pick fault;
   - skipping Pick gate levels above 0 in the inventory;
   - `CpuDiscretePick::from_parts` builds one gate level, where `from_bytes` builds all of them;
   - the CUDA output loop recomputes `pick_prefix` for every gated block.
   - The scorer instrument's `Cargo.lock` is stale against `main`. An unlocked build rewrites it. This predates this unit and is left alone.
3. **The conjunction memory**: the next gap in Casey's 16 Nov order. The pair key here is the first conjunction key, read by position; the memory stores it.
4. **The readout objective, state across windows, and closing the native-space state contract**, in that order. Then the corpus trainer, the D87 ingest and the 1B run on wick.
5. **The tests that fail on `main`** (above).

## Plain summary

The discrete model can now read by copying. For each position it looks back for the earlier place whose last two tokens best match its own last two. It measures the match by XOR and bit-count, which here is exactly the model's own distance. Then it adds the token that came next there, scaled by a gate the trainer can set to −1, 0 or +1. The GPU, the CPU checker and the served model all compute the same read, word for word, on test cases and on a real batch from the new corpus. The GPU and the CPU also agree on how much each gate change would cost, so training can tune these gates. Nothing has been trained with it yet. The next steps are a memory that stores these pairs and an objective that rewards recalling them.

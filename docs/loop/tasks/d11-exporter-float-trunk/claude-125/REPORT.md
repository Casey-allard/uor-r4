# REPORT: d11-exporter-float-trunk, claude-125

Lab: claude. Order: [ORDER.md](ORDER.md). Pre-run review (D67): #2820 comment `6409947018`, PRE-RUN: GO. Its own findings went into the run before launch: the regenerated set-aside 2, the definition of "breaks parity", and the artifact `cmp`.

References #2820.

## What did not work, first

- **One pre-registered bar missed: the single-token query gap.** The D11-mode GPU's largest query gap to the quantized reference over the 256 single tokens is 0.766, against a bar of 1e-3.
  - **Classification:** an instrument miss under the ORDER's pre-registered rule, which allows at most 6 positions, each traced to a near-tie. Only 2 positions exceed the bar (tokens 32 and 240), and every token agrees.
  - **The trace:** each position over the bar in any D11 pairing is traced to its near-tie. The pairings are:
    - the f32 GPU in D11 mode;
    - the f64 reference;
    - the integer artifact's own Q12 query.

    `fbs-compare` reruns the quantized reference with its tie margins. At each snap it records the gap between the best and the second-best of the 120 icosians, relative to the snapped vector's length. At each rate it records the distance to a step of the 16-entry table.

    | Token | GPU vs reference | GPU vs integer | Integer vs reference | Which one split off | Smallest gate-snap margin | Rate margin |
    |---|---|---|---|---|---|---|
    | 32 | 0.766 | agree | 0.766 | the f64 reference | 1.9e-16 | 4.2e-4 |
    | 104 | agree | 0.742 | 0.742 | the integer artifact | 6.9e-6 | 4.1e-3 |
    | 172 | agree | 1.363 | 1.363 | the integer artifact | 0 | 5.3e-3 |
    | 177 | agree | 0.360 | 0.360 | the integer artifact | 1.1e-16 | 8.3e-4 |
    | 240 | 0.179 | 0.179 | agree | the f32 GPU | 0 | 7.9e-3 |

  - **Every position over the bar sits at a Hamilton-gate snap tie.**
    - Over the 256 single tokens, the median smallest gate margin is 2.1e-3.
    - Ten tokens have a gate margin at f64 rounding level (at most 1.9e-16). Only exact ties give margins that small, since a margin that small by chance is vanishingly unlikely.
    - Token 104 is the eleventh-nearest, at 6.9e-6. That is far below one Q12 step (2.4e-4), the integer path's resolution, and only the integer artifact splits there.
    - Of those eleven tokens, five exceed the bar. No token with a larger margin does.
    - At an exact tie, each precision breaks the tie its own way.
    - No rate margin is unusual: at these five tokens, between 14 and 195 of the 256 tokens came nearer a rate step.
  - **No snap of an all-zero vector occurs** at any of the 320 positions, so the GPU's known zero-block difference (below) did not act here.
  - **The two planted faults look different.** Unrounded tables put all 256 positions over the bar. The transition snap turned off puts 55 of 64 sequence positions over.
  - Over the 64 sequence positions, all three computations agree to one Q12 step (2.4e-4), and none is over the bar.
  - **What was fixed before the run, and what came after:** the trace and its margins came after the run, computed on the laptop from the stored GPU outputs. Only the classification rule was fixed before the run.
- **The exact-head review found four gaps, all repaired in this unit:**
  - **The DEV panel had no D11 entry.** Done-when 1 asks for one. `evaluate_dev_panel` set its own gate arm, and the Q12 query rounding existed only in the readout. A Stage 4b panel wrapped in the D11 mode would have scored a full-precision query without a word.
    - `evaluate_dev_panel_d11` now scores the panel in the D11 mode.
    - It is checked on the host only: its guard, and the hook restored. It has not run on the GPU. Its forward and its Q12 writer rounding are those of the readout that ran, and of the export mode's panel.
  - **An unrounded trunk could be scored in D11 mode without a word.**
    - `check_tables_rounded_as_exported` now refuses one, in both the readout and the panel.
    - This changed the readout after the run, but only to refuse. On a rounded trunk it computes what the run computed.
  - **The Q16 rounding was not pinned against the exporter.** The reference reads the biases and the decay through `fixed_value` too, so a no-op `fixed_value` passed the test. The test now holds `fixed_value` against the codes `quantize_fixed_i32` stores, and the no-op fails.
  - **`q12_value` turned NaN into 0.** It now refuses it.
- **My first D11 mode rounded the gates to Q12, which the integer path does not.** I caught it before the run, by reading the reference: its `INTEGER` snaps do not round. claude-124's attribution shows that this rounding alone moves queries by up to 0.536. The D11 mode now snaps the gates directly, through the `HamiltonIcosianHard` gate path, and the ORDER records the change.
- **My first rounding test could not fail for the gains.** A fresh trunk's gains are ones, which are already exact grid values, so skipping the grid rounding passed the test. A set-aside showed it. The test now gives every table random off-grid values first.
- **Exact gate ties are not rare here.**
  - About 4% of single tokens here (10 of 256) meet one. The likely cause, not checked, is that each output channel of an FBS route reads one input channel, signed, so a gate block can repeat a magnitude.
  - Where a tie meets a position, the GPU's D11 mode and the integer artifact can break it differently.
  - In this run that never changed a token, but a trained trunk's DEV score may differ there. Stage 4b should measure it directly (below).
- **Not pinned:**
  - The GPU snap maps an all-zero 4-vector to root index 1 of its own order; the integer reference maps it to index 0 of `H4_ROOT_COEFFICIENTS`. It did not occur here (0 of 320 positions), and it is still unpinned.
  - `with_d11_mode` restores the hook on an error, not on a panic, as `with_d11_export_gates` does.
  - The GPU test's mode-difference check does not isolate the transition snap; the run's set-aside 2 does. Its comment now says so.
  - The tie trace counts a gate's margin even where the block it multiplies is zero. There the snap cannot change the output, so such a margin can only look nearer than it matters. Only the transition skips a zero state.
  - `fbs-compare` writes an infinite margin as `null`. An infinite margin means a switch never met, such as the transition at a single token.
- **Still a random-init trunk.** No trained model ran, and no export gap was measured. That is Stage 4b.

## What changed

Size against `origin/main`:
- code: +832/−175 in 8 files, in-file test modules included;
- tests: +112 in `stack/tests.rs`;
- docs: the ORDER, this REPORT and the notebook row.

**`uor-r4-integer`:**
- `fbs_snapshot` (ungated) now holds the exporter's roundings: `grid_code`, `grid_value`, `fixed_value`, `q12_value` and `FBS_FIXED_EXP`. `stack::encode_grid_code`, `decode_grid_code` and the reference's `ref_fixed` delegate to them, so there is one implementation.
- `FbsReadoutMode` (`plain`, `export`, `d11`) replaces the readout's `export_mode` flag.
- `FbsTrunkSnapshot::tie_margins` records the reference's margins while a trace runs: the transition and gate snaps, the rate, and snaps of a zero vector. The snaps themselves are unchanged.
- `fbs-compare`:
  - takes a D11 readout, and reports it against the quantized reference and against the integer session's tokens and its own Q12 query;
  - counts, in every pairing, the positions over the 1e-3 bar and lists the worst;
  - traces each D11 position over the bar to its tie margins.

**`uor-r4-float-train`:**
- **D11 mode:** `hamilton_sheet::with_d11_mode` snaps both gates directly and turns on a new transition-snap flag, restoring the whole hook afterwards. `composed_recurrence` snaps the transition to the 120 icosians when the flag is set (`icosian_snap_quaternions`, CUDA only).
- **Table rounding:** `fbs_export::round_tables_as_exported` rounds the trunk's tables in place to the artifact's values:
  - grid codes for the gains and taps;
  - Q16 for the biases and decay;
  - Q12 for the embedding.

  `check_tables_rounded_as_exported` refuses a trunk that is not rounded. The two share one list of tables.
- **Readout:** `fbs_float_readout(model, FbsReadoutMode)` in D11 mode rounds the final query to Q12 before the writer, and refuses an unrounded trunk.
- **DEV panel:** `evaluate_dev_panel_d11` scores the panel in D11 mode. `evaluate_dev_panel` shares its scoring.
- **`fbs-snapshot`** takes `[READOUT [EXPORT_READOUT [D11_READOUT]]]` and rounds the tables before the D11 readout.

## Measurements

On an RTX 5090, at `c1bfd1b78897f1eb4084743ee7e351ce044d1a36`, on a random-init trunk (vocabulary 256, width 32, 2 layers). The bars were fixed in the ORDER before the run.

| Check | Single tokens | Sequence positions | Bar |
|---|---|---|---|
| Rounded trunk's artifact | `cmp`: the same bytes as the original's | | same bytes |
| D11 GPU tokens against the quantized reference | 256/256 | 64/64 | ≥ 250 and ≥ 60 |
| D11 GPU tokens against the integer session | 256/256 | 64/64 | ≥ 250 and ≥ 60 |
| D11 GPU largest query gap to the quantized reference | 0.766. Two positions are over, both at gate-snap ties: an instrument miss under the rule. | 2.4e-4 | ≤ 1e-3 |
| D11 GPU largest query gap to the integer session's query | 1.363 (4 positions over, all at gate-snap ties) | 2.4e-4 | (reading) |
| Set-aside 1, tables unrounded | 232/256, all 256 over the bar | 56/64, all 64 over | must break: **broke** |
| Set-aside 2, transition snap off | 256/256 (no transition acts at one position) | 39/64, 55 over | must break: **broke** |
| Plain and export parity (claude-124's pairings) | 256/256 | 64/64 | unchanged |

**The tie trace,** computed on the laptop from the stored outputs at `f326f12f5f895ad9cfad1359f3e60018b1aaf38e`:
- every position over the bar sits at a gate-snap margin of at most 6.9e-6, against a median of 2.1e-3 (single tokens) and 2.7e-4 (sequence prefixes);
- 0 snaps of a zero vector, at all 320 positions.

**Host checks at the head:**

| Check | Result |
|---|---|
| Rounding test | Each rounded table is its own exported rounding. Every table changes and the routes do not. Set-asides of the grid, Q16 and Q12 roundings each fail it. |
| Q16 against the exporter | `fixed_value` equals the stored `quantize_fixed_i32` code for every bias, gate bias and decay offset. Its no-op set-aside fails. |
| Rounded-tables guard | The readout's D11 mode and the panel's D11 entry refuse an unrounded trunk. A rounded trunk gets past the guard, and the hook is restored. The set-asides of the check, of the readout's guard and of the panel's guard each fail the test. |
| Tie margins | Known margins come out right (a root's own, an exact tie, a rate step, a zero snap). A single token records no transition. Runs do not leak into each other. Set-asides of the gate record, of the zero-state rule and of the zero count each fail. |
| Transition flag | Set only inside `with_d11_mode`, and cleared after an error too. Its set-aside fails the test. |
| Suites | float-train 33; integer serving-build library 322; no-feature library 55; `m5_full_path_cost_controls` 3/3; `memory_shape` 3/3; `fbs-compare`'s own test |
| CUDA type-check | `cargo check --features cuda --all-targets` of float-train with a stub nvcc: exit 0 |
| Zero-multiply audit | `FULL PASS`: rlib 56 symbol ranges and 40 functions; `uor-r4-stack` 44 and 90 |
| GPU cost | one RTX 5090 for 59 s of job time (`job.log`, 06:48:20Z to 06:49:19Z); `vast-57777264` deleted |

Evidence is in the HF store under `results/claude/d11-fbs-d11mode-2026-10-26/`. `fbs-d11/` and `ties/` each have a `SHA256SUMS`, and the folder's top-level `SHA256SUMS` covers `traced/`:
- `fbs-d11/` holds the run;
- `traced/` holds the traced comparisons, run on the laptop from the stored outputs;
- `ties/` holds the tie margins, with the code commit and the binary's sha256 in `COMMAND.txt`.

## Checks run at the head

See the exact-head review record on the PR:
- the suites above;
- the stub-nvcc CUDA check;
- clippy (no findings in the changed code);
- `cargo fmt --check`;
- both audits;
- `loop report --check`, `loop pr-check --local` and `loop gates --full`.

## Skills used

- **holding-the-goal.** The aim was a mode Stage 4b can score the artifact with on the GPU. That is why:
  - the gate rounding was corrected before the run rather than reported afterwards;
  - the panel's missing D11 entry was built in this unit rather than handed on.
- **writing-meaningful-tests.** Each piece was set aside to show its test can fail. Two tests could not fail at first, the gains' rounding and the Q16 rounding, and both were fixed.

## Still open (Stage 4b)

- **The Stage 4 sheet,** with:
  - a fresh non-author FIT;
  - a two-seed retrain saved with `StackModel::save`;
  - the export gap, scored on the GPU: the plain float trunk against `evaluate_dev_panel_d11` on the sheets' evaluator, with the two-seed noise band as the band;
  - a direct D11-against-artifact token agreement on a DEV subset, since exact gate ties are not rare (above), as an instrument check before the gap counts;
  - laptop tok/s, with decode and context reported separately.
- **The zero-block snap edge,** if a trained trunk ever produces one.
- **Serving front ends and the pages,** as before.

## Plain summary

The float model can now run in a "D11 mode": on the GPU, it computes what the integer file that ships to devices computes. It rounds the same rotations to the same fixed set of 120, and reads its numbers rounded the way the file stores them.
- On a test model, this mode picks the same next token as the integer file on every input tried: 256 single tokens and 64 positions in sequences.
- Its internal numbers agree with the integer file's to one rounding step, except at five places. At each of those, a rotation sits exactly or almost exactly halfway between two of the 120. The GPU, the reference and the integer file each pick a side their own way.
- The next step can now score a trained model in this mode on the GPU, as the project's rules require, and so measure how much accuracy it loses when shipped as integers.

# REPORT: fbs-serving-kernels, claude-163 (the split)

Lab: claude. Order: [ORDER.md](ORDER.md). The item stays `ready`: this unit meets its done-when 1. Done-when 2 and 3 (the kernel and its speed) are the next unit's.

**Speed.** On this M1, the served FBS model decodes at **272 tok/s**: 3.67 ms a token on claude-126's seed-1 export at a 192-token context. claude-158 read 265 tok/s on the same artifact.

## What did not work, first

- **claude-158's hypothesis was wrong.** It guessed that the route gathers dominate both of a layer's blocks (its REPORT, "The hypothesis"). All 36 gathers (six a layer) are 0.5% of a token, counted with the residual adds (two a layer) and the feed-forward's requantization. The cost is in two kernels, and they share one inner piece:
  - **the recurrence core,** 56.3% of a token;
  - **the feed-forward's gated product,** 39.8%.
  - Both run the 120-root icosian snap (`stack_snap_rotation`, through `stack_snap_select`), which scores every root with exact `ℤ[φ]` comparisons.
  - The core runs it twice per 4-channel block: once for each lane's transition, and once for the output gate. An FBS export's geometric gates require the icosian transport snap (`format.rs`, "rec_gates=geometric requires transport_snap=icosian").
  - The product runs it once per 4-unit block (`stack_swiglu`, the Hamilton gate the exporter requires).
- **The split stops at the part.** How much of the core and of the product the snap takes is not measured. The kernel unit has to time it before choosing what to write.
  - The rest of the core is: the lanes' table decay and their Hamilton products; the output gate's Hamilton products; and the core's own requantization, which sits inside the core part.
  - No `isqrt` runs in the lanes of a geometric export. It sits only in the table-law branch.
- **The code review's notes, taken:**
  - **Which part a time is credited to is not testable.** A lap touches no model buffer, so the host test pins exactness and the layout, not the attribution. The lap placement was checked by reading, by the author and by the reviewer.
  - **The band reads a sum of part medians against a block median,** rounded down to whole per mille. For a 10% band, both are within its slack.
  - **A block missing some of its parts was not refused.** `blocks_against_parts` now requires all four (recurrence) or five (feed-forward). Its test fails with that check weakened to "some parts" (`3 passed; 1 failed`) and passes with it restored.
  - **A reply that differs from the plain run is reported (`replies_match`), not refused.** Both runs here read `true`.
- **The first notebook draft gave two ranges wrong** (345–350 µs and 242–254 µs a layer). I caught it against `parts.json` before the review. The ranges are 339–350 µs and 240–246 µs.
- **The item has no GitHub issue yet**, so the claim note could not be posted there (noted in FRICTION). `issues.json` on main has no row for the five items `claude-162` added.

## What changed

- **`crates/uor-r4-integer/src/stack/session.rs`:**
  - `stack_step_fbs_timed` gains a `parts` flag. Off, its laps are claude-158's; on, it adds seven laps inside each layer, so the two blocks become nine parts.
  - The recurrence block's parts: its norm with the gain, the input and gate gathers, the core, and the output gather with the residual add.
  - The feed-forward's parts: its norm with the gain, the gate and up gathers, the gated product, the requantization, and the down gather with the residual add.
  - The new `IntegerStackSession::measure_fbs_step_parts` runs it with the flag on. `stack_step_fbs` and `step` are untouched.
  - The new host test `the_part_timed_fbs_step_is_the_served_step` holds the parts-timed step equal to `step_native` over 48 positions of a synthetic trunk: token, kept cells, query, the whole-table nearest, the saved state and the tokens. It also checks that its `9 × layers + 6` times tile the step.
- **`crates/uor-r4-integer/src/bin/fbs-toks.rs`:**
  - `--parts` implies `--stages` and runs the protocol a third time through `measure_fbs_step_parts`. Its `parts` report gives each part's median.
  - Under `against_blocks`, it sums each block's part medians against that block's median from the `--stages` run, per mille, with `within_10_percent`.
  - `stage_report` and `stage_names` now take each layer's names (`BLOCKS` or `PARTS`).
  - The new host test `each_block_is_read_against_the_sum_of_its_parts` checks the arithmetic, the 10% band in both directions and the refusals. The existing test now also checks the nine-part row widths.
- **`docs/research/notebook/m4-serving.md`:** one row.
- **`docs/roadmap/items/fbs-serving-kernels.md`:** a `Progress` section: done-when 1 met, and where the kernel unit starts.

## Measurements

**The artifact:** claude-126's seed-1 export, fetched from `results/claude/d11-fbs-export-gap-2026-10-26/d11gap/save/seed-1/artifact.uorlut`, sha256 `314c3909c66b58522a641c3f5b51c25dda4bda5ab2274bcfd0b5ca5038fdb305` (claude-158's). Width 256, 6 layers, vocabulary 4,096.
- **Binary:** `fbs-toks`, release, `--features serving-wasm`, built at `c9c5fd238`, sha256 `9c1f1778602c6eb9f593119656891db35b5c0e7d7b574f39bc9578164fac6f9a`.
- **Machine:** this M1, on AC power.
- **Protocol:** the plain run, then the staged run, then the parted run. Each is 1 warm-up and 20 timed runs of a 128-token prefill and 64 decode steps, and medians are over the 1,280 timed decode steps.

**Instrument checks:**

| Check | Result |
|---|---|
| I-1 the artifact's sha256 against claude-158's | match |
| I-2 replies of the plain, staged and parted runs | identical (`replies_match` true, both) |
| I-3 stage medians summed against the step | `994` per mille in the staged run and in the parted run |
| I-4 the parted step against the staged step | `3,672,083 / 3,668,187` ns = 1.001 |
| I-5 each block's parts against its block (12 blocks) | **999–1,003 per mille**, inside the 900–1,100 band at every block |
| I-6 plain decode against claude-158's | `272` tok/s against `265` (+2.6%) |

**The token by part, summed over the six layers** (the parted run's medians; step `3,672,083` ns):

| Part | ns | Share of the step | Per layer |
|---|---|---|---|
| recurrence core | 2,069,060 | **56.3%** | 339–350 µs |
| feed-forward gated product (snap and Hamilton product) | 1,461,436 | **39.8%** | 240–246 µs |
| recurrence norm with gain | 37,166 | 1.0% | 6.2 µs |
| feed-forward norm with gain | 37,043 | 1.0% | 6.2 µs |
| recurrence input and gate gathers | 6,498 | 0.2% | 1.1 µs |
| feed-forward gate and up gathers | 6,000 | 0.2% | 1.0 µs |
| feed-forward down gather with residual | 2,250 | 0.1% | 0.4 µs |
| feed-forward requantization | 2,250 | 0.1% | 0.4 µs |
| recurrence output gather with residual | 2,209 | 0.1% | 0.4 µs |

Outside the layers: the writer 22.6 µs, the final norm 6.3 µs, the embedding 0.1 µs.

**Sealed:** `caseyallard/uor-r4-store:results/claude/fbs-serving-kernels-split-2026-10-28/` holds `parts.json` (the whole `fbs-toks --parts` output), `binary.sha256`, `COMMAND.txt` and `SHA256SUMS`, read back with `shasum -c` OK.

## Checks run at the head

- `cargo test -p uor-r4-integer --features serving-wasm --lib -- fbs_stage_tests`: `3 passed; 0 failed`.
- `cargo test -p uor-r4-integer --features serving-wasm --bin fbs-toks`: `4 passed; 0 failed`.
- `cargo fmt -p uor-r4-integer -- --check`: clean.
- `cargo clippy -p uor-r4-integer --features serving-wasm --all-targets`: no finding in the changed files. The crate's existing findings elsewhere are unchanged.
- `cargo xtask roadmap check` and `render --check`: OK.
- `uor-harness loop report --check`, `loop pr-check --local` and `loop gates --full`: in the review record.

## Skills used

- **measure-before-optimizing.** The kernel's target is taken from the split, not from claude-158's guess, which the split refutes.
- **verify-before-done.** Every figure in the notebook row was checked against `parts.json` before the review, which caught two wrong ranges.

## Still open

- **The kernel unit (done-when 2 and 3).** It times `stack_snap_select` inside the core and the product first. Its two candidates are exact:
  - a structured argmax over the 600-cell's three coordinate classes, which keeps the lowest-index tie rule;
  - a vector kernel the audit allows.
  It measures tokens/s with each against this run's 272.

## Plain summary

A word the model writes on the laptop takes 3.7 thousandths of a second. This splits that time into its pieces. Almost all of it goes to two steps, and both repeatedly snap a direction to the nearest of 120 fixed points, checking every one of the 120 each time. The pieces once thought expensive cost almost nothing. The next step is to make that snap cheaper.

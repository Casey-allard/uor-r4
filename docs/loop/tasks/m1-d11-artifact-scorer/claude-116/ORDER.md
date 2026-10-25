# ORDER: m1-d11-artifact-scorer, claude-116

Lab: claude. Item: [m1-d11-artifact-scorer](../../../../roadmap/items/m1-d11-artifact-scorer.md). Base: `ffe0dcf8b`.

## Goal

The item's done-when reads:

> All three are built as a detached instrument in `docs/labs/m1-d11-artifact-scorer/instrument/`: its own `[workspace]`, with path dependencies on `uor-r4-discrete` and `uor-r4-integer` and no edit to either. That keeps `forbid(unsafe_code)` in the serving crate (§3.13) and takes no crate lock.
> - **`export`** turns a `DriverCheckpoint` JSON into a `UORLUT01` file. The file is byte-identical to `export_d11` on the same snapshot, and `IntegerDiscreteModel::from_bytes` accepts it.
> - **`score`** runs a CUDA, clean-room port of `IntegerDiscreteModel::evaluate_prefix` over the registered DEV panel. The rows come from `uor_r4_discrete`'s own corpus windows, so they are the same rows `eval-corpus` scores. It writes per-row prediction, top-8 and admission, and integer top-1, top-8 and admitted totals.
>   - The kernel is built ahead of time by nvcc, with no NVRTC, no float and no squashing (§3.2–§3.4).
>   - A finite test shows the CUDA results bit-identical to `evaluate_prefix` on fixtures, and fails when the port is set aside.
>   - A GPU qualification on one RTX 5090 shows parity on a real exported checkpoint. The A8 FrozenZero `continuation-05248.json`, inside `results/antigravity/a8-full-dose-rerun-2026-10-25/a8-full-dose-rerun-20261025.tar.gz` (store, with its `SHA256SUMS`), qualifies the instrument only, and is never step 6's reported measurement.
> - **`toks`** times `IntegerDiscreteModel` serving on the laptop by the step-6 protocol, and records the hardware.
> - **A note on #2839** maps each tool line in the step-6 sheet to the instrument command that replaces it. Those lines are `:32` (the exporter "merged in #2998"), `:81` (`discrete-stack export-d11`) and `:82` (the `eval-corpus`/`IntegerDiscreteModel` comparison). This item does not edit that sheet.

## Cancel probe (run first)

Class: instrument. Run at `ffe0dcf8b`; all must hold:
- **The tools are still absent.** `grep -rn 'export-d11\|export_d11_to_path' crates/` finds nothing, `crates/uor-r4-integer` has no `.cu` file, and `docs/labs/m1-d11-artifact-scorer/` does not exist. **Holds.**
- **A detached crate can reach every API it needs without editing either crate.**
  - `uor_r4_discrete::{d11_export, snapshot, corpus, training}` are `pub mod` (`crates/uor-r4-discrete/src/lib.rs:18,20,40,42`).
  - `corpus::PreparedCorpus::load`, `eligible_positions` and `eligible_targets` are public (`corpus.rs:133,243,251`).
  - `uor_r4_integer::discrete_d11::IntegerDiscreteModel` is public at the crate root in both builds: the lib root is `src/native_geometry.rs`, which declares `pub mod discrete_d11` for native (`:21-22`) and includes `serving_root.rs` (`:19`, `pub mod discrete_d11` at `:14`). `from_bytes` and `evaluate_prefix` are `pub` (`discrete_d11.rs:614,1165`).
  **Holds.**
- **No open PR touches the instrument path or this item** (open: #3245, #3239, #3169, and this unit's #3249). **Holds.**

Result: all hold, so the unit goes ahead.

## Files in scope

- `docs/labs/m1-d11-artifact-scorer/**`
- `docs/harness/prep/m1-d11-artifact-scorer*`
- `docs/loop/tasks/m1-d11-artifact-scorer/**`
- `docs/research/notebook/m4-serving.md`

## Done when

Built in four stages, each committed with its own checks:
1. **The skeleton, `export` and the rows.** This is `docs/labs/m1-d11-artifact-scorer/instrument/`: a `[workspace]` of its own, with path deps on `uor-r4-discrete` and `uor-r4-integer` (default features, plus `cuda` only behind the instrument's own `cuda` feature).
   - `export CHECKPOINT.json OUT.lut`. Its test checks that the output is byte-identical to `export_d11` and that `IntegerDiscreteModel::from_bytes` accepts it.
   - A DEV-row builder that yields the same (prefix, target) rows `eval-corpus` scores, from `uor_r4_discrete::corpus`. Its test checks the 14,287-row count and pins against the registered DEV data.
2. **`score`.**
   - A CUDA kernel, built ahead of time by nvcc in the instrument's `build.rs`, with a PTX admission check that mirrors `crates/uor-r4-discrete/build.rs`. It is a clean-room port of `evaluate_prefix` (the Relation, Flock and Route reads, admission and the top-8), one row per block.
   - The host precomputes the row plan as a static integer table, since it is a function of prefix length only.
   - It is type-checked on the Mac with a stub nvcc.
   - A finite test, `#[ignore]`d and run on the pod, shows CUDA bit-identical to `evaluate_prefix` on fixture artifacts and rows.
3. **`toks`.** The step-6 protocol, on the laptop, through `IntegerDiscreteModel`: 1 warmup and 20 timed runs, a 192-token prefill and a 64-token decode, integer microseconds, median, nearest-rank p95, the raw samples and the hardware profile.
4. **GPU qualification** on one RTX 5090, under a D67 pre-run review. The finite parity test passes. Then `export` and `score` run on the A8 FrozenZero `continuation-05248.json` and agree with `evaluate_prefix` on every scored row, which is the qualification only, never step 6's measurement. The outputs go to `results/claude/d11-artifact-scorer-<date>/`.

Then: a notebook row in `docs/research/notebook/m4-serving.md` (qualification), the note on #2839 mapping the sheet lines `:32`, `:81` and `:82` to the commands, a fresh claims check, an exact-head review, `Closes #3248`, and the item set `done`.

## Stop and ask if

Stop and decide, never stop and wait.

- **A needed API is not public** in either crate. Then decide between a minimal visibility change through a decision review, and copying a clean-room helper. Never edit the locked crate under this ORDER.
- **The CUDA port cannot match `evaluate_prefix` bit for bit.** That is an instrument fault (D31): fix the port, and never relax parity.
- **The work grows past one PR.** Then land stages 1–3 as References #3248, and run the qualification in the next unit.

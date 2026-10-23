# REPORT: legal-construction-exact-solve, claude-53 (verified inputs and the bounded-variable simplex on the merged loader)

Lab: claude. Base: `770048255`. Order: [ORDER.md](ORDER.md), committed first (`59f6a35df`).

The cancel probe stands. At the base, `grep -c sha256 src/case.rs` is 0 and `src/blp.rs` is absent, so the task stands.

## What did not work, first

- **The exact root LP does not finish in the ORDER's time budget.** claude-50's run of `blp` (commit `95328fb27`, the same logic ported here):
  - started 2026-10-22T22:05:45Z under a declared 7,800 s cap;
  - ended 2026-10-23T00:15:46Z with `timeout` exit 124 and no result line;
  - made 775 pivots, all in phase 2, with 2 bound flips;
  - grew `D` to at most 22,846 bits, with the last pivots taking about 20 s each.

  Its objective was 0 through pivot 275 and negative from pivot 300 (progress lines every 25 pivots). That is an observation of the instrument, not a proof: there was no certificate, so no optimum and no sign of one is claimed. Under D22 this is an instrument stop, not a mechanism negative.
- **The work first went to the wrong place.** It was built on draft #3015, alongside DeepSeek's #3013 for the same item. My branch was unpushed, so my claim looked stale. The two were reconciled on #3013 (6353849400, 6353993780): #3013 landed first, and this unit moves #3015's work onto the merged `case.rs`. #3015 is closed.
- **The ported files were reformatted** by `cargo fmt`. Their logic is unchanged; only imports and paths differ. `saved::load` calls became `case::load_verified(..)` plus `.problem`.

## What changed

| Commit | What |
|---|---|
| `59f6a35df` | This ORDER, alone. |
| `bbee0a248` | The code, listed below. |
| `1e91b82bc` | The lab README section "Unit 3a", and one notebook row (verdict: blocked, instrument stop). |
| this REPORT's commit | This REPORT. |

What `bbee0a248` adds:
- **`src/receipts.rs`:** `verify`, `VerifiedHash` and `sha256_hex`. Every input is checked against the sha256 the archive records:
  - `protected-margin-receipt.json` `terms[i].sha256`, with exactly 380 terms and `terms[i].file == protected-margin-{i:03}`;
  - `coupled-gradient-receipt.json` `files[]`, with exactly one entry for each of the 4 master and gradient files; entries for other files are ignored.

  A mismatch, a missing receipt, a missing entry or a duplicate entry is refused, and the error names the file.
- **`case::load_verified`** runs `verify`, then the unchanged `load`. `solve-case` uses it and prints `hashes_verified=384`.
- **`src/blp.rs`:** the exact bounded-variable revised simplex, with its unit tests.
- **`src/bin/root-lp.rs`**, loading through `load_verified`.
- **Tests:** `tests/blp_agrees.rs` (controls, the random suite, tightened-bound node LPs, and the threaded path with 64 rows or more) and `tests/case_verified.rs`.
- **`Cargo.toml`/`Cargo.lock`:** `sha2`, `serde_json`.

## Measurements

These are host controls and an instrument run, not a model result.

- **Verified load of the saved case:** `n=1920 guards=380 fractional=20 hashes_verified=384`, with `zero_move legal=true guards_hold=true objective=0/1`.
- **Root-LP run:** as above, 775 pivots in 7,800 s with no result.

## Checks run at the head

- `cargo test --offline --locked --release`: lib 2 (1 ignored), blp_agrees 3, case_loader 6, case_verified 4, controls 11, no_floats 1, random 2. `-- --ignored` passes too, including `integral_on_saved_case_rows`.
- `cargo clippy --offline --locked --all-targets -- -D warnings`: exit 0.
- **Planted faults,** each caught, then restored and checked with `cmp`:

  | Fault | Tests that fail |
  |---|---|
  | The entering variable's own bound dropped from the ratio test | `controls_agree_with_tableau`, `random_agree_with_tableau` |
  | `certify` short-circuited | `certify_rejects_a_non_optimal_point` |
  | b left unscaled in `integral` | `integral_scales_each_row_by_one_positive_factor` |
  | The sha256 comparison disabled | `one_changed_byte_is_refused_naming_the_file`, `a_wrong_receipt_entry_is_refused` |

- **No floats:** f32 and f64 appear only in `src/exact.rs` (`no_floats` passes).
- `loop report --check` and `loop pr-check --local`: see the PR body.

## Skills used

scope-guard: the lab folder and the notebook page only. `lp`, `bb` and the merged loader's mapping are unchanged.

## Still open

- **claude-54:** a fixed-point integer search (no floats) for a candidate basis, proved or rejected by `blp`'s exact certificate (#2828, 6354250887). Then branch-and-bound on the same certificate.

## Plain summary

The inputs are now checked against the archive's own fingerprints, and the faster exact solver is on `main`'s loader. It still could not finish the full problem in two hours, so the next step is a quicker search whose answer the exact solver then proves.

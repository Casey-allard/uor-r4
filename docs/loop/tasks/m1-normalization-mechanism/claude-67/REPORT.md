# REPORT: m1-normalization-mechanism, claude-67 (DEV scoring for the normalization arms)

Lab: claude. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md) (#2842). The item status is unchanged; no compute is cleared and nothing is released.

## What did not work, first

1. **Two of the three registered bar terms cannot be scored by this tool, and it says so.**
   - Structural norm stability (item 1) needs per-stage output traces, which an eval-corpus report does not hold. The next unit is a trace-capture path.
   - Non-inferiority to `C_unscaled` has no registered margin (claude-57 prerequisite 3, a PM/owner card).
   - The output reports both as `UNSCORED`, with the reason.
2. **One seed per call.** The bar needs both seeds, so the two per-seed outputs are combined by hand, or later by a sealed decision record.
3. **Not run on real data.** The new commands are CUDA-gated and need real checkpoints and DEV reports. Only the pure scorer and the config lock have unit tests. The I/O paths reuse the existing `dev_inputs` validation unchanged.
4. **The brief's test filter would have run nothing.** It named `attention_preflight`, but the module path is `controller::attention_learning::preflight`, so that filter matches 0 tests. The builder caught it, and the gates use `preflight`.
5. **`driver_source_sha256` moved** (`attention_preflight.rs` is a driver source): `ab0691a4…` → `f8cdd5261f1190b213530305522c12f53a164cfd3dd47269e29273a495dfb124`. It is re-pinned in `prime_zeta.rs`, and the three bins that pin it pass.

## Cancel probe

At the base, `attention-dev-b3` refuses non-default modes (`attention_preflight.rs:272`). The only paired counting in the crate is the fresh-transfer one (`attention_transfer.rs`, floor `max(20, …)`). So no command gave a DEV B3 or a paired DEV contrast for `C_unscaled` or `C_raw`, and the task stands.

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-discrete/src/attention_preflight.rs` | `DevLock { Attention, Normalization }` and `dev_config_lock`. `dev_inputs` takes the lock: `attention-dev-b3` and `capture_dev` pass `Attention`, so their behaviour is unchanged, and `validate_dev_b3_config` keeps every line. |
| | `normalization_dev_b3` (schema `uor-r4.normalization-dev-b3/1`) adds `normalization_mode` and `normalization_scope`. |
| | The pure scorer `dev_paired`, plus `dev_paired_floor`, `F_I = max(24, ⌈2√(b+c)⌉)` with integer `isqrt` and checked ops. It refuses unequal lengths, differing row identities `(row_ordinal, target_position, target_token, is_psucc)`, a hit not in {0, 1}, and a hit above the row's `target_admitted`. |
| | `normalization_dev_paired` (schema `uor-r4.normalization-dev-paired/1`) runs on two arms whose configs are equal except the mode, with differing modes and the same seed. It reports `all` and `psucc` blocks `{rows, b, c, both_correct, both_wrong, f_i, b_minus_c, beats}` and each arm's `psucc_target_admitted` and `q1_admission_floor_met` (≥ 1,351 of 3,847; it asserts 3,847 `P_succ` rows), plus the `UNSCORED` terms. |
| | Three tests. |
| `crates/uor-r4-discrete/src/main.rs` | `normalization-dev-b3 CONFIG CHECKPOINT DEV_EVAL OUT`, `normalization-dev-paired REF_CONFIG REF_CHECKPOINT REF_DEV_EVAL CTL_CONFIG CTL_CHECKPOINT CTL_DEV_EVAL OUT`, and their help lines. |
| `crates/uor-r4-discrete/src/prime_zeta.rs` | The re-pin. |

## Measurements

- **Every discrete target** (`cargo test --locked -p uor-r4-discrete`) exits 0:
  - lib: 238 passed, 1 ignored;
  - bins: `address-cells-codebook` 6, `prime-zeta-codebook` 7, `prime-zeta-qualify` 6;
  - tests: 7, 7 and 6.
- **New tests** (`--lib preflight`, 8 passed):
  - `normalization_dev_b3_lock_admits_three_modes_attention_lock_does_not`;
  - `dev_paired_counts_and_floor`, which covers F_I at 0/10/143/144 → 24, 145 → 25, 200 → 29 and 225 → 30, and `beats` at the `b − c == F_I` boundary;
  - `dev_paired_refuses_mismatched_rows_and_impossible_hits`.
- **Negative gate, run by me.** With `Ok(24.max(ceiling))` changed to `Ok(ceiling)`, `dev_paired_counts_and_floor` FAILED (0/1). Restored, it passed (1/1) on a clean tree.
- **CUDA stub check.** `cargo check --features cuda --all-targets` with the stub nvcc finished with no errors. Clippy, with and without `cuda`, reports nothing in the changed lines, and the changed files are rustfmt-clean.
- **Ratchets at baseline:** D23 4451, D25 93, D32 31.

## Checks run at the head

The measurements above, plus `loop report --check`, `loop pr-check --local` and `git diff --check`.

## Skills used

- **`scope-guard`:** only `crates/uor-r4-discrete` and this folder. `attention_transfer.rs` and the A8 lock are untouched.
- **`solution-integrity`:** the scorer reports only registered, scoreable terms, and the rest are marked `UNSCORED` with reasons. The floor test fails with its fix set aside.

## Still open

- Trace capture for the structural-norm term (point 1).
- The non-inferiority margin, an open PM/owner card (point 1).
- Sheet registration (#3115).

## Plain summary

The normalization test now has a way to score its two control arms on DEV, row by row against the main arm, with the registered noise floor and admission floor. Terms that cannot yet be scored are reported as unscored, not guessed.

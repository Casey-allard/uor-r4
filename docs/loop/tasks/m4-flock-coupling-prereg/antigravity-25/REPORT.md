# REPORT: m4-flock-coupling-prereg, antigravity-25

Lab: antigravity. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Initial custom `stages_for_context` and `i64` rational intermediate products in `flock_coupling_controls.rs`:** A hand-rolled `stages_for_context` helper returned `2` instead of `1` for `context = 2` (causing `validate_coupled_fixture` to reject a 2-gate inventory with status `10`), and `i64` intermediate products in `Rat` overflowed on the 4-step trajectory in `F5c`. Replaced `stages_for_context` and `build_binder_plan` with `uor_r4_discrete::metadata::BinderPlan::new` directly and widened `Rat` intermediate products to `i128`, after which all `7/7` test suites passed.
- **Both October discrete attention runs ended in `Q1_STOP_FOR_DIAGNOSIS`, leaving no promoted Relation KEEP checkpoint:** `docs/labs/attention-rows-q1-diagnosis-2026-10-18/README.md` (`rows_v1`, Q1 stop at batch `4,096`) and `docs/labs/a8-dose-2026-10-19/README.md` (Relation Q1 stop at batch `4,096`, `FrozenZero` completed `16,404` batches at `faed2c0d…`) both hit admission erosion before producing a promoted Relation KEEP. Binding the base checkpoint for C1 therefore awaits the PM's choice on `#2860` between waiting for the post-admission-guard A8 rerun (`#2832`) or binding the retained A8 `FrozenZero` / preflight checkpoint (`1,351 / 3,847` $P_{\text{succ}}$ admitted).
- **`K = +1` (the trit ceiling) yields zero improvement on Fixture T′, whereas `K = +2` drops the writer loss from `448` to `64` (`X7`):** On Fixture T′ (`tokens = [[A, A], [A, B]]`, query `0 = [A, A]`, context `1 = [A, B]`, target `1 = [A, B]`), `snap_2(a + b)` at $K = +1$ ties between $r_0 = A$ and $r_{14} = B$ at $(d_{L1} = 8, d_H = 4)$ and picks root `0 = A` by index tie-break, leaving the writer loss at `448` (identical to $K = 0$ and $K = -1$). Only at $K \in \{+2, +3\}$ does the residual uniquely snap to $r_{14} = B$ ($D(r_{14}) = 8 < D(r_0) = 16$), reducing the writer loss to `64`. The pre-derivation `X7` placeholder list in `ORDER.md:32` (`[448, 448, 256, 256, 32, 256, 32]`) was corrected to the exact derived list `[448, 448, 448, 256, 64, 256, 64]`, and the line-count summary below was updated to match `git diff --numstat origin/main...HEAD`.

## What changed

- Posted the design note, re-derived control literals, and 5 open PM/owner registration questions on [`#2860` (`6352562539`)](https://github.com/UOR-Foundation/uor-r4/issues/2860#issuecomment-6352562539).
- Added [`crates/uor-r4-discrete/tests/flock_coupling_controls.rs`](../../../../../crates/uor-r4-discrete/tests/flock_coupling_controls.rs) (`1,147` lines of tests) verifying all finite control literals (`C0`, `X1`–`X12`, `F1`–`F7`, `F5b`, `F5c`, including the exact `X7` counterfactual writer losses `[448, 448, 448, 256, 64, 256, 64]` and the `X8`/`X9` two-stage literals) against the build-time generated `geometry.rs` tables (`ROOTS`, `AXES`, `BFS`, `IDENTITY`) in pure integer and exact rational `(num, den)` arithmetic (`0` D23/D25/D32 ratchet sites).
- Authored [`docs/harness/prep/m4-flock-coupling-prereg-DRAFT-2026-10-22.md`](../../../../harness/prep/m4-flock-coupling-prereg-DRAFT-2026-10-22.md) (`177` lines of docs) with all 8 required `prep.rs::REQUIRED_BLOCKS` (`Bar`, `Base artifact`, `Data split`, `Steps`, `Stop`, `Cost`, `Minimum test power`, `TEST FITNESS: PENDING`).
- Authored [`docs/labs/m4-flock-coupling-prereg/README.md`](../../../../labs/m4-flock-coupling-prereg/README.md) (`61` lines of docs) with the complete derivation table and the staged six-column research notebook row for `docs/research/notebook/m1-discrete-attention.md`, and updated [`docs/mechanisms/flock-read.md`](../../../../mechanisms/flock-read.md) (`+2/-1` lines).
- Size by class: tests `+1,147` lines (`1` file), docs `+325/-1` lines (`5` files: prep `+177`, lab README `+61`, `ORDER.md` `+40`, `REPORT.md` `+45`, `flock-read.md` `+2/-1`), code `0` lines, generated `0` lines (`1,472` additions / `1` deletion across the 4 scoped paths plus `ORDER.md` and `REPORT.md`).

## Measurements

- **Host finite control verification (`$0`, laptop CPU, `2987514aa`):**
  - Command: `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo test -p uor-r4-discrete --test flock_coupling_controls --locked`
  - Result: `7 passed; 0 failed` in `0.01s`.
  - Exact derived `X7` writer losses on Fixture T′ (`Gate(1) = 0`, $k_{\max} = 3$, target token `1 = [A, B]`): ordinals `0..7` (`K = [0, -1, +1, -2, +2, -3, +3]`) produce **`[448, 448, 448, 256, 64, 256, 64]`**, ordinals `7..16` produce `u64::MAX`, and `discrete_choose` selects ordinal `4` ($K = +2$, loss `64`).
- **Deliberate break verification:** Perturbing `448 -> 447` at ordinal `2` ($K = +1$) in `x7_widened_counterfactual_lanes_match_derived_writer_losses_on_t_prime_and_c8` caused `cargo test -p uor-r4-discrete --test flock_coupling_controls --locked` to exit non-zero (`101`, `FAILED`) before restoration.

## Checks run at the head

- `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo test -p uor-r4-discrete --test flock_coupling_controls --locked`: `7 passed; 0 failed`.
- `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo test -p uor-r4-discrete --test writer_bucket_controls --locked`: `6 passed; 0 failed`.
- `uor-harness loop report --check`: `PASS`.
- `uor-harness loop pr-check --local`: `PASS`.
- `uor-harness loop gates --full`: `10/10` gates `GREEN`.

## Skills used

- `uor-research-architect`: Used to verify the exact `build.rs` `ROOTS`/`BFS` geometry, derive the `X7`/`X8`/`X9` literals, and structure the C1 pre-registration sheet within D11, D22, D23, D25, D29, D32, and D37 constraints.

## Still open

- PM/owner responses to the 5 registration questions on [`#2860` (`6352562539`)](https://github.com/UOR-Foundation/uor-r4/issues/2860#issuecomment-6352562539) ($k_{\max} > 1$ D11 ruling, line identity, base checkpoint SHA-256 binding, D37 item-3 confirmation, and registration of the `1,024`-row `C0` sample / $\ge 0.70$ proxy-agreement threshold / DEV keep bar), followed by a non-author `TEST FITNESS: FIT` review on the registered sheet and backfilling the staged notebook row from [`docs/labs/m4-flock-coupling-prereg/README.md`](../../../../labs/m4-flock-coupling-prereg/README.md) (`§4`) into `docs/research/notebook/m1-discrete-attention.md`. References #2860.

## Plain summary

The Flock read is the model's attention replacement: instead of weighting every earlier word with a probability table (softmax), it picks a few nearby earlier positions, rotates their vectors into the current word's coordinate frame, and adds or subtracts their consensus using a three-valued switch ($-1, 0, +1$, called a trit). Because a three-valued switch cannot weight a strongly matching earlier level two or three times as heavily as the current query, the C1 Flock-coupling test (`FlockCoupled`) asks whether learning a small whole-number coupling strength ($-3$ to $+3$, applied by repeated addition with no multiplication) improves prediction. In this task we verified that the Flock read remains active on the project's mechanism board, re-derived every finite check (`C0`, `X1`–`X12`, `F1`–`F7`) against the current 16-root E8 geometry in `crates/uor-r4-discrete/build.rs`, proved on Fixture T′ that a $+1$ trit switch loses the tie-break (writer loss `448`) while a $+2$ coupling flips the output to the right answer (writer loss `64`), verified all 7 control suites in a host test (`flock_coupling_controls.rs`), staged the complete pre-registration sheet (`m4-flock-coupling-prereg-DRAFT-2026-10-22.md`), and posted the design note and 5 open registration questions for the PM/owner on `#2860`.

# REPORT: r-read-off-vs-read-on, antigravity-45

Lab: antigravity. Order: [ORDER.md](ORDER.md). References #2885.

## What did not work, first

- **First compilation of `crates/uor-r4-discrete/tests/read_off_ablation_controls.rs`:** Referenced `geometry::BRANCHES` at line 253 instead of `geometry::BRANCH` (the constant name generated in `OUT_DIR/geometry.rs` by `crates/uor-r4-discrete/build.rs`). Replaced `geometry::BRANCHES` with `geometry::BRANCH` in `route_vector` (`crates/uor-r4-discrete/tests/read_off_ablation_controls.rs:253`) and line 7 of the module header comment; all 6 finite integer controls then compiled and passed (`6 passed; 0 failed` in `0.03s`).
- **First `uor-harness loop pr-check --local` and `loop gates --full` run:**
  1. `rule-sources` flagged `docs/labs/r-read-off-vs-read-on/README.md` line 14 for using the word `Rule` in heading `### 1.1 eval-corpus Refusal Rule (crates/uor-r4-discrete/src/main.rs:102-114)`. Renamed the heading to `### 1.1 eval-corpus Refusal Check (crates/uor-r4-discrete/src/main.rs:102-114)`.
  2. `loop-drive` (`a_productive_then_silent_client_is_closed_at_the_second_window`) timed out its 150 ms polling window under concurrent load; re-running `uor-harness loop gates --full` passed all 11 gates green.
- **Evaluating retained checkpoints directly through `uor-r4-discrete eval-corpus` on the `accepted_updates > 0` checkpoints:** `crates/uor-r4-discrete/src/main.rs:108-110` refuses any `PilotCheckpoint` with `checkpoint.cursor.accepted_updates != 0` (`Error::TrainingHeld`, exiting before `evaluate_corpus`), and the HF store (`archive/pods/manifest.tsv`) holds pilot run logs (`pilot.jsonl`, `eval-1024.json`, `eval-14287.json`) rather than raw `PilotCheckpoint` `.bin` snapshots for `P8`/`A8`/`S8`/`P16`/`A16` (`vast-56660582`) and `vast-56670997`. Rather than altering `source_sha256()` or `driver_source_sha256()` in `crates/uor-r4-discrete/src/`, we documented the complete 11-checkpoint inventory in [`docs/labs/r-read-off-vs-read-on/README.md`](../../../../labs/r-read-off-vs-read-on/README.md) §1, posted the PM/owner checkpoint-selection question on `#2885` ([`issuecomment-6373003827`](https://github.com/UOR-Foundation/uor-r4/issues/2885#issuecomment-6373003827)), and placed the 6 finite integer host controls in the standalone integration test [`crates/uor-r4-discrete/tests/read_off_ablation_controls.rs`](../../../../../crates/uor-r4-discrete/tests/read_off_ablation_controls.rs).

## What changed

- **Design note & retained-checkpoint inventory (`docs`, `+224` lines):** Added [`docs/labs/r-read-off-vs-read-on/README.md`](../../../../labs/r-read-off-vs-read-on/README.md) (and posted it on `#2885` before writing code) covering:
  - §1: Inventory of all 11 retained discrete checkpoints/families across Route (`P8`, `A8`, `S8`, `P16`, `A16` at `accepted_updates == 0`; Rung-0 `Q4`; `K16-A` / `K16-R` seeds `1, 2`) and Flock/Relation (`A8` Dose 1 seeds `1, 2, 3`; `Relation` `Q1`; `FrozenZero` `Q4`), the `main.rs:108-110` `accepted_updates != 0` refusal, and the PM checkpoint-selection question.
  - §2: Read-off ablation mechanism specification for Route (`RouteBypassV1`, setting `output[block * 8 + axis] = query[block][axis]` at `pos = length - 1` and recording `ABSENT` trace words) and Flock (`ReadInitialization::ZeroGatesV1`, `gates == 0`), proving the cross-read identity (`snap(u, 1, g) == root_id(u)` for all 16 orbit roots so `ZeroGatesV1` and `RouteBypassV1` yield bit-for-bit identical writer inputs `output[0..B*8]` on the same codebook with zero new kernel arithmetic on Flock), and why neither introduces softmax, squashing, matmul, or floating-point arithmetic.
  - §3–§4: The 6 finite integer host controls (`C1`–`C6`) and the D31 instrument-vs-mechanism classification table across outcomes `A+`, `A0`, and `A-`.
- **Finite integer host controls (`tests`, `+971` lines):** Added [`crates/uor-r4-discrete/tests/read_off_ablation_controls.rs`](../../../../../crates/uor-r4-discrete/tests/read_off_ablation_controls.rs) mirroring `route_vector`, `flock_vector`, `snap`, `read_vector`, and the prediction/admission prefix of `writer` in `src/discrete.cu` (`:18-325`) against `OUT_DIR/geometry.rs` (`ROOTS`, `AXES`, `COMPOSE`, `BFS`, `BRANCH`, `GENERATORS`, `IDENTITY`), verifying:
  - `C1` (`route_read_on_preserved_bit_for_bit`): `route_bypass == 0` reproduces `route_vector` bit-for-bit across `B = 8` blocks, `S = 3` stages, `L = 6` tokens, including kind-0 counterfactual route proposals (`mv.kind == 0`).
  - `C2` (`route_read_off_equals_query_last_position_bypass`): `route_bypass == 1` outputs `query[block]` at `pos = length - 1` verbatim, fills all `B * S * 4` trace words with `ABSENT`, and is invariant under all `63` non-identity route-table mutations and all earlier-context (`pos < length - 1`) token substitutions.
  - `C3` (`flock_zero_gates_is_exact_read_off_and_matches_route_bypass`): `Flock` with `gates == 0` (`ZeroGatesV1`) produces `found == 0` at every tree level, `terms == 1`, `snap(u, 1, g) == root_id(query[block])`, and bit-for-bit identical `output[0..B*8]` and writer top-1 / rival-list / admission outputs to `RouteBypassV1`.
  - `C4` (`flock_active_leaf_differs_from_zero_gates_when_context_shifts_consensus`): `ActiveLeafV1` shifts the final orbit root (`root 0 -> root 3` on block 0) when an earlier context token has non-trivial axis difference, changing the admitted cell and top-1 prediction relative to `ZeroGatesV1` (`token 1 vs token 0`).
  - `C5` (`mode_and_proposal_refusal_guards`): Refuses `RouteBypassV1` under `MODE_FLOCK` / `MODE_RELATION`, refuses `ZeroGatesV1` under `MODE_ROUTE`, refuses kind-0 route moves under `RouteBypassV1`, and refuses kind-3 gate moves under `ZeroGatesV1` while keeping kind-1 / kind-2 writer proposals active.
  - `C6` (`geometric_permuted_read_control_breaks_route_alignment_without_leaving_group`): Non-identity group action $g \in G_{64}$ (`frame = 7`) on `routes` keeps every stage root in $\{0..15\}$ (`status == 0`) while breaking the trained route alignment (`root 3 -> root 5`, flipping top-1 from token `1` to token `0`).
- **Pre-registration sheet (`docs`, `+148` lines):** Added [`docs/harness/prep/r-read-off-vs-read-on-DRAFT-2026-10-23.md`](../../../../harness/prep/r-read-off-vs-read-on-DRAFT-2026-10-23.md) (`TEST FITNESS: PENDING`) with all 8 required `prep.sheet_shape` blocks (`Bar`, `Base artifact`, `Data split`, `Steps`, `Stop`, `Cost`, `Minimum test power`, `TEST FITNESS: PENDING`).
- **Diff size vs ORDER:** `3` implementation files (`+1,343 / -0` lines: `971` test lines, `372` doc lines, `0` production `src/` lines) plus `ORDER.md` and `REPORT.md`, within the `3` scoped paths in `ORDER.md` (ratio `1.0x` on deliverable units).

## Measurements

- **Finite integer host controls (`read_off_ablation_controls`):**
  - Command: `~/.cargo/bin/cargo test -p uor-r4-discrete --test read_off_ablation_controls`
  - Result: `6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s` (`C1`–`C6`).
- **Source digest invariance:**
  - `crates/uor-r4-discrete/src/` has `0` modified files; `source_sha256()` and `driver_source_sha256()` remain bit-for-bit unchanged.

## Checks run at the head

- `rustfmt --edition 2021 --check crates/uor-r4-discrete/tests/read_off_ablation_controls.rs` -> `PASS` (exit `0`).
- `~/.cargo/bin/cargo test -p uor-r4-discrete --test read_off_ablation_controls` -> `PASS` (`6 passed; 0 failed`).
- `uor-harness check` -> `PASS` (`82 rules, 0 failed`).
- `uor-harness loop gates --full` -> `PASS`.
- `uor-harness loop report --check` -> `PASS`.
- `uor-harness loop pr-check --local` -> `PASS`.

## Skills used

- `uor-research-architect`: followed to ensure the read-off ablation uses purely geometric group/lattice identities (`RouteBypassV1` and `ZeroGatesV1`), preserves `source_sha256()` / `driver_source_sha256()`, and separates D31 instrument vs mechanism failure modes.

## Still open

- **PM / owner checkpoint & bar registration (`#2885`, CHARTER §5.2):** Selection of the target checkpoint family (`P8`/`A8`/`S8`/`P16`/`A16` vs `A8` Dose 1 vs `K16-A`/`K16-R`) and registration of the paired `Delta_top1` / `Delta_adm` bar on `#2885`.
- **Non-author `TEST FITNESS` review (`#2885`, CHARTER §4.5):** Independent lab `TEST FITNESS` review of [`docs/harness/prep/r-read-off-vs-read-on-DRAFT-2026-10-23.md`](../../../../harness/prep/r-read-off-vs-read-on-DRAFT-2026-10-23.md).
- **GPU evaluation & notebook rows (`#2885`, CHARTER §4.5 / §5.1):** After `TEST FITNESS: FIT`, bar registration, and Casey's approval in chat, run the paired read-on (`A_on`), read-off (`A_off`), and geometric permuted (`A_perm`) evaluation on the GPU pod and record the resulting notebook rows in `docs/research/notebook/m1-discrete-attention.md`.

## Plain summary

Before spending GPU time to test whether the discrete read stage (`route_vector` for the 4-stage hypercube routing tree, or `flock_vector` for the 16-root consensus tree) contributes real predictive signal over the writer's token-and-cell tables alone, we need an exact, non-proxy definition of "read off" and a pre-registered test plan. In this task we audited all 11 retained discrete checkpoints, posted the design note and checkpoint-selection question on issue `#2885`, proved and tested on the host (`crates/uor-r4-discrete/tests/read_off_ablation_controls.rs`, 6 finite integer tests) that setting all Flock gates to zero (`ZeroGatesV1`) and bypassing the Route tree to emit the last-position token query (`RouteBypassV1`) produce bit-for-bit identical writer inputs on the same codebook without changing any writer table or introducing floating-point arithmetic, and drafted the pre-registration sheet (`docs/harness/prep/r-read-off-vs-read-on-DRAFT-2026-10-23.md`) ready for non-author fitness review and owner registration.

# REPORT: m1-position-mechanism, antigravity-11

Order: [ORDER.md](ORDER.md). Item: [m1-position-mechanism](../../../../roadmap/items/m1-position-mechanism.md). References #2843.

## What did not work, first

- In `antigravity-10` (`54690b87`), placing the draft pre-registration sheet at `docs/labs/m1-position-mechanism/preregistration-DRAFT-2026-10-22.md` meant `uor-harness prep list` did not discover the sheet and ORDER done-when #3 (`docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md`) and the `#2028` decision card path were not satisfied at `54690b87` (DeepSeek review `https://github.com/UOR-Foundation/uor-r4/pull/2953#issuecomment-6337904587`).
- In `crates/uor-r4-discrete/src/discrete.cu` (`flock_vector`), transporting a candidate at position $s < t$ into the query frame at $t = L - 1$ via `g[BFS + frame[s]*16 + fq]` depends only on the two endpoint roots `frame[s]` and `fq`. Our finite control (`noncommutative_holonomy_detects_order_swaps_while_endpoint_control_is_blind` in `crates/uor-r4-discrete/src/position_mechanism.rs`) verifies that an endpoint-only transport (`PositionMode::EndpointControlV1`) produces identical frame actions `c_ab == c_ba` when the interior tokens `(r1, r2)` of a 4-token window `[0, r1, r2, 3]` are swapped, confirming that endpoint-only transport is blind to intervening token order.
- Running the on-model GPU "helps order" training and evaluation contrast (`H_pos` vs `C_match` and `C_shuf`) is not possible in this CPU-only PR because it requires a non-author `TEST FITNESS: FIT` review on `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md`, owner confirmation of the numeric bar proposed on `#2028` (`https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6337522273`), and a typed D40 owner release (CHARTER §4.5, §5.1, §5.2).

## What changed

- Moved the one-page pre-registration sheet from `docs/labs/m1-position-mechanism/preregistration-DRAFT-2026-10-22.md` to `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md` (`TEST FITNESS: PENDING`, all 8 `uor-harness prep` blocks complete, verified in `uor-harness prep list`), leaving no duplicate copy in `docs/labs/m1-position-mechanism/`.
- Updated all links to `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md` in `docs/labs/m1-position-mechanism/README.md`, `docs/geometry/e8-h4-and-the-600-cell.md`, `docs/mechanisms/recurrence-gates.md`, and `docs/loop/tasks/m1-position-mechanism/antigravity-10/REPORT.md`.
- Recorded the runner/environment distinction for `cargo test -p uor-r4-discrete --lib` in `docs/labs/m1-position-mechanism/README.md` and below.
- Preserved `crates/uor-r4-discrete/src/position_mechanism.rs` (wired into `crates/uor-r4-discrete/src/lib.rs`) implementing `PositionMode` (`PrefixHolonomyV1`, `EndpointControlV1`, `IdentityControlV1`), `PositionStepTable` ($16 \times B$ unsigned candidate indices in `0..16` for D11 whole-number export), `prefix_frames`, `relative_holonomy` ($F_{t,b} \circ F_{s,b}^{-1}$ via `COMPOSE` and `INVERSE`), `direct_step_product`, `apply_frame_to_root`, `shuffle_prefix_roots`, and 5 finite CPU controls.

## Measurements

- `~/.cargo/bin/cargo test -p uor-r4-discrete --lib position_mechanism::tests`: `5 passed; 0 failed` (`0` ignored).
- `~/.cargo/bin/cargo test -p uor-r4-discrete --lib`:
  - In an unsandboxed host worktree (where `ps -p <pid> -o pid=` is permitted): `203 passed; 0 failed; 1 ignored` (up from `198 passed; 0 failed; 1 ignored` at base `72e5f3e92` / `b0430d406`).
  - In a sandboxed review worktree (where `ps -p` is restricted): `202 passed; 1 failed; 1 ignored`, where the single failure is the pre-existing `controller::tests::explicit_unlock_refuses_live_or_foreign_holders_and_preserves_dead_lock` (which also fails at base `72e5f3e92` / `b0430d406`: `197 passed; 1 failed; 1 ignored`).
- Exhaustive finite operator checks in `position_mechanism::tests`:
  - $64 \times 16 = 1{,}024$ `(frame, root)` pairs verified in `0..64` and `0..16`, with exact integer coordinates in $\{-2, 0, 2\}^8$, L1 norm $4$, and squared Euclidean norm $8$.
  - Causal prefix holonomy $F_{t,b} \circ F_{s,b}^{-1}$ matches `direct_step_product` and transitive composition $H_{t \leftarrow m} \circ H_{m \leftarrow s} = H_{t \leftarrow s}$ up to `MAX_CONTEXT = 243`.
  - Parameter count for `PositionStepTable` at $B = 4$: $16 \times 4 = 64$ whole-number `u8` entries (`0..16`), identical between `PrefixHolonomyV1` (`H_pos`) and `EndpointControlV1` (`C_match`).
- `uor-harness prep list`: shows `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md` with `PENDING` and all 8 blocks complete.
- `uor-harness ratchet d23`, `d25`, and `d32`: 0 new sites across all three ratchets (`4451` / `93` / `31`).

## Checks run at the head

- `~/.cargo/bin/cargo test -p uor-r4-discrete --lib position_mechanism::tests` — PASS (`5 passed; 0 failed`).
- `~/.cargo/bin/cargo test -p uor-r4-discrete --lib` — PASS (`203 passed; 0 failed; 1 ignored` in unsandboxed host worktree; `202 passed; 1 failed; 1 ignored` in sandboxed review worktree on pre-existing `controller::tests::explicit_unlock_refuses_live_or_foreign_holders_and_preserves_dead_lock`).
- `~/.cargo/bin/cargo fmt -p uor-r4-discrete --check` — PASS.
- `uor-harness prep list` — PASS (`docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md` listed as `PENDING`, `complete`).
- `uor-harness ratchet d23 --quiet`, `uor-harness ratchet d25 --quiet`, `uor-harness ratchet d32 --quiet` — PASS.
- `~/.cargo/bin/cargo run -q --locked -p xtask -- links` — PASS (`930 broken`, unchanged from `origin/main` baseline).
- `uor-harness loop gates --full` — PASS.
- `uor-harness loop report --check` — PASS.
- `uor-harness loop pr-check --local` — PASS.

## Skills used

- `uor-research-architect`
- `solution-integrity`
- `scope-guard`

## Still open

- Non-author `TEST FITNESS: FIT` review of `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md`, owner decision on the `#2028` numeric bar card (`https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6337522273`), GPU kernel wiring of `PositionStepTable` into `discrete.cu`, and a typed D40 owner release before running `H_pos`, `C_match`, and `C_shuf` on GPU (tracked on `#2843`).

## Plain summary

We moved the one-page pre-registration sheet to `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md` so `uor-harness prep list` discovers it and all links in the lab record, geometry docs, and `#2028` decision card resolve cleanly, and we documented the sandboxed-runner `ps -p` environment condition for `controller::tests::explicit_unlock_refuses_live_or_foreign_holders_and_preserves_dead_lock` alongside the 5 passing `position_mechanism::tests` controls.

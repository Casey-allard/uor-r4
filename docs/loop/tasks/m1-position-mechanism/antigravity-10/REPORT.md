# REPORT: m1-position-mechanism, antigravity-10

Order: [ORDER.md](ORDER.md). Item: [m1-position-mechanism](../../../../roadmap/items/m1-position-mechanism.md). References #2843.

## What did not work, first

- In `crates/uor-r4-discrete/src/discrete.cu` (`flock_vector`), transporting a candidate at position $s < t$ into the query frame at $t = L - 1$ via `g[BFS + frame[s]*16 + fq]` depends only on the two endpoint roots `frame[s]` and `fq`. Our finite control (`noncommutative_holonomy_detects_order_swaps_while_endpoint_control_is_blind` in `crates/uor-r4-discrete/src/position_mechanism.rs`) verifies that an endpoint-only transport (`PositionMode::EndpointControlV1`) produces identical frame actions `c_ab == c_ba` when the interior tokens `(r1, r2)` of a 4-token window `[0, r1, r2, 3]` are swapped, confirming that endpoint-only transport is blind to intervening token order.
- Running the on-model GPU "helps order" training and evaluation contrast (`H_pos` vs `C_match` and `C_shuf`) is not possible in this CPU-only PR because it requires a non-author `TEST FITNESS: FIT` review on `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md`, owner confirmation of the numeric bar proposed on `#2028` (`https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6337522273`), and a typed D40 owner release (CHARTER §4.5, §5.1, §5.2).

## What changed

- Posted the design note on `#2843` (`https://github.com/UOR-Foundation/uor-r4/issues/2843#issuecomment-6337521845`) and the numeric-bar decision card on `#2028` (`https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6337522273`).
- Added `crates/uor-r4-discrete/src/position_mechanism.rs` (wired into `crates/uor-r4-discrete/src/lib.rs`) implementing `PositionMode` (`PrefixHolonomyV1`, `EndpointControlV1`, `IdentityControlV1`), `PositionStepTable` ($16 \times B$ unsigned candidate indices in `0..16` for D11 whole-number export), `prefix_frames`, `relative_holonomy` ($F_{t,b} \circ F_{s,b}^{-1}$ via `COMPOSE` and `INVERSE`), `direct_step_product`, `apply_frame_to_root`, `shuffle_prefix_roots`, and 5 finite CPU controls.
- Added the one-page pre-registration sheet `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md` (`TEST FITNESS: PENDING`, all 8 `uor-harness prep` blocks complete).
- Added the lab record `docs/labs/m1-position-mechanism/README.md` and synchronized references in `docs/geometry/e8-h4-and-the-600-cell.md`, `docs/geometry/quaternions-and-rotation.md`, and `docs/mechanisms/recurrence-gates.md`.

## Measurements

- `~/.cargo/bin/cargo test -p uor-r4-discrete --lib position_mechanism::tests`: `5 passed; 0 failed`.
- `~/.cargo/bin/cargo test -p uor-r4-discrete --lib`: `203 passed; 0 failed; 1 ignored` in an unsandboxed host worktree (up from `198 passed; 0 failed; 1 ignored` at base `72e5f3e92`, adding 5 unit tests in `position_mechanism::tests`); in a sandboxed review worktree where `ps -p` is restricted, the pre-existing test `controller::tests::explicit_unlock_refuses_live_or_foreign_holders_and_preserves_dead_lock` fails both at base `72e5f3e92` and at head (`202 passed; 1 failed; 1 ignored`).
- Exhaustive finite operator checks in `position_mechanism::tests`:
  - $64 \times 16 = 1{,}024$ `(frame, root)` pairs verified in `0..64` and `0..16`, with exact integer coordinates in $\{-2, 0, 2\}^8$, L1 norm $4$, and squared Euclidean norm $8$.
  - Causal prefix holonomy $F_{t,b} \circ F_{s,b}^{-1}$ matches `direct_step_product` and transitive composition $H_{t \leftarrow m} \circ H_{m \leftarrow s} = H_{t \leftarrow s}$ up to `MAX_CONTEXT = 243`.
  - Parameter count for `PositionStepTable` at $B = 4$: $16 \times 4 = 64$ whole-number `u8` entries (`0..16`), identical between `PrefixHolonomyV1` (`H_pos`) and `EndpointControlV1` (`C_match`).
- `uor-harness ratchet d23`, `d25`, and `d32`: 0 new sites across all three ratchets.

## Checks run at the head

- `~/.cargo/bin/cargo test -p uor-r4-discrete --lib` — PASS (`203 passed; 0 failed; 1 ignored`).
- `~/.cargo/bin/cargo fmt -p uor-r4-discrete --check` — PASS.
- `uor-harness ratchet d23 --quiet`, `uor-harness ratchet d25 --quiet`, `uor-harness ratchet d32 --quiet` — PASS.
- `~/.cargo/bin/cargo run -q --locked -p xtask -- links` — PASS (`938 broken`, unchanged from `origin/main` baseline).
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

We designed and implemented `PositionHolonomyV1` in `crates/uor-r4-discrete/src/position_mechanism.rs`, carrying sequence order along the causal window by exact prefix composition in the 64-element `Cl(0,6)` signed-action group on the 16-root E8 orbit, with zero softmax, squashing, dense matmul, or floating-point operations. Five finite CPU tests verify identity invariance, prefix-holonomy transitivity, integer bounds, and sensitivity to interior token swaps against a parameter-matched endpoint-only control, and `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md` pre-registers the GPU order test.

# ORDER: m1-position-mechanism, antigravity-10

Lab: antigravity. Item: [m1-position-mechanism](../../../../roadmap/items/m1-position-mechanism.md). Base: `72e5f3e92`.

## Goal

The item's done-when reads:

> A position mechanism with finite controls that obeys CHARTER §3 (geometry only, no softmax or squashing, no matmul), plus a pre-registered test that it helps order. The bar for "helps order" is not yet registered; step 4 registers it.

## Cancel probe (run first)

Ran `ls -la docs/harness/prep/m1-position-mechanism*` (`NO_SHEET`), checked `crates/uor-r4-discrete/src/lib.rs` (no position mechanism module), and ran `~/.cargo/bin/cargo test -p uor-r4-discrete --lib` (`198 passed; 0 failed; 1 ignored`). Issue `#2843` had 0 prior comments before our claim (`https://github.com/UOR-Foundation/uor-r4/issues/2843#issuecomment-6337452766`).

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/geometry/e8-h4-and-the-600-cell.md`
- `docs/geometry/quaternions-and-rotation.md`
- `docs/harness/prep/m1-position-mechanism*`
- `docs/mechanisms/recurrence-gates.md`
- `docs/labs/m1-position-mechanism/**`

## Done when

1. Design note posted on `#2843` specifying the geometric position construction in `crates/uor-r4-discrete` (exact prefix-holonomy frame transport in the 64-element `Cl(0,6)` signed-action group acting on the 16-root E8 orbit), how it updates and transports earlier positions without softmax, squashing, dense matmul, or floating-point arithmetic, how it exports as whole numbers (`u8` frame/generator indices in `0..64` / `0..15`), and how "helps order" is defined and tested against both a parameter-matched identity-step control and a shuffled-context-order null control.
2. Finite CPU controls in `crates/uor-r4-discrete/src/position_mechanism.rs` (wired into `crates/uor-r4-discrete/src/lib.rs`) testing identity transport invariance, prefix-holonomy composition ($F_t \circ F_s^{-1}$ vs direct step composition along $[s+1..=t]$), integer bounds and group/orbit closure (`0..64` frames, `0..16` roots, exact coordinates in $\{-2, 0, 2\}^8$), non-commutative order sensitivity (adjacent token swaps change the relative holonomy frame and transported root), and matched-parameter identity control, with `~/.cargo/bin/cargo test -p uor-r4-discrete --lib` passing and zero D23/D25/D32 ratchet growth.
3. One-page pre-registration sheet `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md` (`TEST FITNESS: PENDING`, all 8 `uor-harness prep` blocks complete) plus a decision card to the PM/owner on `#2028` proposing the numeric "helps order" bar (CHARTER §5.2), lab record `docs/labs/m1-position-mechanism/README.md`, and cross-references in `docs/geometry/e8-h4-and-the-600-cell.md`, `docs/geometry/quaternions-and-rotation.md`, and `docs/mechanisms/recurrence-gates.md`.

## Stop and ask if

- Any file outside the declared scope needs modification.
- Any GPU execution is needed (remains gated on non-author `TEST FITNESS: FIT` and typed owner D40 release).
- Setting a binding numeric bar is required (proposed via decision card on `#2028` per CHARTER §5.2).

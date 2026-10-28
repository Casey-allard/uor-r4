# ORDER: in-space-trunk-to-text, claude-151

Lab: claude. Item: [in-space-trunk-to-text](../../../../roadmap/items/in-space-trunk-to-text.md). Base: `50e7fc6bd`.

## Goal

The item's done-when reads:

> On the pinned DEV panel, decoded as the whole table's nearest token:
>
> 1. Reply-opening top-1 is at least `197` of 1,226. That is always naming DEV's commonest opening token (`161`) plus three binomial standard deviations (`36`). The same checkpoint's whole-panel top-1 must be at or above the M1 count table's `7,002`.
> 2. On that same checkpoint, the relative reach from `--reach`, `R16 = shift(16) / shift(0)`, is at least `0.05`. The 27 Oct best checkpoint reads `0.011`. An untrained trunk reads more, but it cannot meet (1).
> 3. Greedy replies to the six fixed DEV prompts are recorded beside the true replies, at every checkpoint, in the lab record.
>
> Lead with tokens per second, decode and context separately.

## The owner's direction (Casey, 27 Oct 2026, in claude's chat, verbatim)

> "After the writer unit closes, go straight back to the Hamming read: give its type heads a position signal from SpiralCore's spin, so replies can use the prompt."

## The design: a content-keyed holonomy in the type heads

claude-146's Hamming read (`flock_read_hamming.rs`, merged) has two copy heads, which work by construction, and two type heads, which must learn what to ask for and have no position signal: a rotary per-index spin was removed in review, by the owner's 13 Oct direction ("we do not need rope, we have the spiralcore transport spin mechanisms"). An index-only step `u` gives `F_t = u^t`, a rotary code again. SpiralCore's transport is path-dependent instead: a position's frame is the running product of steps chosen by each token's content (`PositionHolonomyV1`, `crates/uor-r4-discrete/src/position_mechanism.rs`: `prefix_frames`, `relative_holonomy`), on its 8D group. The type heads work in quaternion lanes, so this unit uses SpiralCore's quaternion group, the 120 unit icosians (2I, exact in Z[φ]; `hamilton_sheet.rs`, `h4_unit_roots_f32`):

1. **Step:** at each position `t`, each type-head lane takes `u_t`, the icosian nearest the token `x_t`'s table point through a learned per-lane rotation (straight-through in training).
2. **Frame:** `F_t = u_t ⊗ u_{t−1} ⊗ … ⊗ u_1` per lane, inside each window, by a parallel prefix scan of Hamilton products (about log₂ 192 = 8 levels).
3. **Use:** a type head turns its query at `t` by `F_t⁻¹` and its key at `s` by `F_s⁻¹`. Left multiplication by a unit quaternion keeps inner products, so the angle between the two depends only on `F_t ⊗ F_s⁻¹ = u_t ⊗ … ⊗ u_{s+1}`: the product of the steps of the tokens between `s` and `t`. Position is what lies between, path-dependent. The read compares sign codes, which also see the frame `F_s⁻¹` the two share: each code bit differs with a probability set by that angle only on average over the shared frame, so the Hamming distance carries the steps between on average, not exactly.
4. **Copy heads are untouched,** so exact copying holds. Off by default (`--type-spin 0`), so every earlier run reproduces.

## Cancel probe (run first)

Cancel if a position signal for the Hamming read's type heads is already on main, or another open order touches `flock_read_hamming.rs`.

Result (2026-10-27T23:40Z, `origin/main` at `50e7fc6bd`): `git grep -i -e type.spin -e holonomy origin/main -- crates/uor-r4-float-train` finds nothing. The open PRs are this unit's draft #3337 and antigravity's #3323, which touches no file in `crates/uor-r4-float-train` or this item's docs. **Not cancelled (instrument: none needed).**

## Files in scope

- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `crates/uor-r4-float-train/src/secant_descent.rs`
- `crates/uor-r4-float-train/src/lib.rs`
- `crates/uor-r4-float-train/Cargo.toml`
- `crates/uor-r4-float-train/src/flock_read*.rs`
- `Cargo.lock`
- `docs/harness/prep/in-space-trunk-to-text*`
- `docs/labs/in-space-trunk-to-text/**`
- `docs/loop/tasks/in-space-trunk-to-text/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/in-space-trunk-to-text.md`

## Done when

1. `--type-spin 1` builds the holonomy above. CUDA tests pin, each with a negative gate:
   - the scan equals the sequential product;
   - the relative frame between `s` and `t` ignores every token at or before `s`;
   - every step is one of the 120 icosians;
   - the copy heads stay exact with the spin on;
   - with the spin off, the read is unchanged.
2. A pre-registered language run (its own sheet, FIT and pre-run review, CHARTER §4.5): the read with the spin against the same read without it (claude-146's `df1d7892` runs at seeds 1–3, whose results are in the store), at the same settings. The sheet's bar decides it.
3. The REPORT leads with tokens/s; one notebook row; the result on main.

## Stop and decide (never stop and wait)

- **The spin slows training past about 2×:** profile the scan before the run, and fix it or record the cost.
- **A run diverges or crashes:** stop it, classify it (instrument or object), and rerun that seed. Never read a missing number as a fail.
- **The seeds split on the bar:** the sheet says how a split is read; follow it.

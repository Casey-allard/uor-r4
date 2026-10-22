# ORDER: m6-exact-arithmetic-mechanism, antigravity-18

Lab: antigravity. Item: [m6-exact-arithmetic-mechanism](../../../../roadmap/items/m6-exact-arithmetic-mechanism.md). Base: `603acff74`.

## Goal

The item's done-when reads:

> A mechanism with finite controls on exact arithmetic items, integrated through the shared operators with no second model or external teacher. The bar is not yet registered: how many items, which operations and what pass line are all undefined in the sources. Registering them is part of step 3 below.

## Cancel probe (run first)

Ran `uor-harness loop next` and `gh issue view 2864 --comments` before claiming:
1. No open PR or task folder claims `m6-exact-arithmetic-mechanism` (`#2864`).
2. `crates/uor-r4-integer` has `discrete_d11.rs` (the 64-state `Cl(0,6)` operator composition table `COMPOSE`, 16-root `E_8` anchor orbit `ROOTS`, BFS transport `BFS`, and `snap`) and `m6_ablation.rs` (`PrimeFactorKey`, `IntegerAddressedMemory`, and `AblationArm`), but no exact arithmetic module (`m6_exact_arithmetic.rs`) or lab record (`docs/labs/m6-exact-arithmetic-mechanism/README.md`) exists yet.
3. Cancel probe result: **CONTINUE** — the exact geometric arithmetic mechanism, finite CPU controls, design note on `#2034` / `#2864`, bar decision card on `#2028`, and pre-registration sheet are genuinely missing.

## Files in scope

- `crates/uor-r4-core/**`
- `crates/uor-r4-integer/**`
- `docs/geometry/spiralcore.md`
- `docs/mechanisms/exact-addressed-memory.md`
- `docs/mechanisms/spiralcore-fbs.md`
- `docs/mechanisms/ternary-xor.md`
- `docs/labs/m6-exact-arithmetic-mechanism/**`

## Done when

1. Design note and bar decision card posted on `#2034`, `#2864`, and `#2028` specifying the geometric construction (place-indexed 8D integer digit atoms on the 16-root `E_8` anchor orbit, `Cl(0,6)` `BFS`/`COMPOSE` step transitions with exact inverse $g^{-1} = g^3$, ternary carry/borrow trit switch in `{-1, 0, +1}`, and `PrimeFactorKey`/`IntegerAddressedMemory` place-and-role addressing), why no step uses squashing or dense matmul, how it serves as integers under D11, and the proposed control and qualification bars.
2. `crates/uor-r4-integer/src/m6_exact_arithmetic.rs` implements the exact geometric arithmetic mechanism (`OrbitDigitAtom`, `PlaceStepRecord`, `GeometricArithmeticTrace`, `GeometricArithmeticEngine`) supporting multi-digit radix-$B$ ($2 \le B \le 16$, including decimal $B=10$ and full-orbit $B=16$) addition with ripple carry, subtraction with borrow, shift-and-add multiplication, and multi-step addressed-memory evaluation with `AblationArm` knockout hooks.
3. Finite CPU controls in `crates/uor-r4-integer/src/m6_exact_arithmetic.rs` (including negative refusal and causal knockout controls) pass under both default features and `--features serving-wasm`.
4. `docs/labs/m6-exact-arithmetic-mechanism/README.md` records the design note, control rows in six-column notebook format, and the one-page pre-registration sheet for GPU evaluation (`TEST FITNESS: PENDING`), and the mechanism/geometry docs in scope are updated.

## Stop and ask if

- Any file outside the item's `scope:` (and `SHARED_FILES`) needs to change.
- Any GPU execution is needed (GPU runs stay HELD pending non-author `TEST FITNESS: FIT` and Casey's typed release).
- Setting a milestone or control bar rather than proposing it as a decision card to Casey.

# REPORT: m6-exact-arithmetic-mechanism, antigravity-18

Lab: antigravity. Order: [ORDER.md](ORDER.md). Item: [m6-exact-arithmetic-mechanism](../../../../roadmap/items/m6-exact-arithmetic-mechanism.md), closes #2864.

## What did not work, first

- **Shared role-prime + place-prime factor keys cross-talked across places and masked capacity eviction:** The first draft of `OperandRole::place_key(place)` built a two-prime `PrimeFactorKey([role_prime, place_prime])` where all places of a role shared `role_prime` and all roles shared `place_prime`. Because `IntegerAddressedMemory::lookup` admits any active slot with `shared_primes >= 1` (`gcd(q, A_c) > 1`), querying an evicted `(OperandA, place 0)` slot after `OperandB` overwrote the ring matched `(OperandB, place 0)` on the shared `place_prime` instead of reaching the eviction log. Replacing shared role/place primes with pairwise-coprime two-prime keys `(nth_odd_prime(2 * slot_idx), nth_odd_prime(2 * slot_idx + 1))` where `slot_idx = (role_index << 5) + place` makes $\gcd(q_{r_1, p_1}, A_{r_2, p_2}) > 1 \iff (r_1, p_1) = (r_2, p_2)$, eliminating cross-talk and ensuring evicted operand slots always return `MemoryLookupOutcome::Evicted`.
- **Fixed-width `max_places = 4` engine rejected 5-digit carry sums in the knockout test:** Running `4789 + 5643 = 10432` on the `max_places = 4` engine built for the subtraction underflow check tripped the `addition carry overflowed engine max_places bound` guard at place 4; configuring the 16-item knockout sweep with `max_places = 8` (`wide_engine`) separated the narrow-width overflow refusal control from the multi-step causal knockout sweep.
- **No GPU evaluation was run:** Under CHARTER §4.5 and §5.1, GPU evaluation of the served M6 model remains HELD pending `m6-reasoning-panel-freeze` (`#2866`), M1 trunk training (`#2845`), owner confirmation of the proposed bar on `#2028`, a non-author `TEST FITNESS: FIT` review on [`docs/labs/m6-exact-arithmetic-mechanism/README.md`](../../../labs/m6-exact-arithmetic-mechanism/README.md), and Casey's approval.

## What changed

- Posted the M6 exact geometric arithmetic design note and bar decision card on [`#2034`](https://github.com/UOR-Foundation/uor-r4/issues/2034#issuecomment-6342897026), [`#2864`](https://github.com/UOR-Foundation/uor-r4/issues/2864#issuecomment-6342897842), and [`#2028`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6342898483).
- Added [`crates/uor-r4-integer/src/m6_exact_arithmetic.rs`](../../../../crates/uor-r4-integer/src/m6_exact_arithmetic.rs) (`OrbitDigitAtom`, `OperandRole`, `PlaceStepRecord`, `GeometricArithmeticTrace`, `GeometricArithmeticEngine`, `u64_to_atoms`, `atoms_to_u64`) and wired `pub(crate)` 16-root `E_8` / `Cl(0,6)` / FBS helpers in [`crates/uor-r4-integer/src/discrete_d11.rs`](../../../../crates/uor-r4-integer/src/discrete_d11.rs) plus module exports in [`crates/uor-r4-integer/src/native_geometry.rs`](../../../../crates/uor-r4-integer/src/native_geometry.rs) and [`crates/uor-r4-integer/src/serving_root.rs`](../../../../crates/uor-r4-integer/src/serving_root.rs).
- Added [`docs/labs/m6-exact-arithmetic-mechanism/README.md`](../../../labs/m6-exact-arithmetic-mechanism/README.md) with the design note, four six-column notebook control rows, and the 8-block pre-registration sheet (`TEST FITNESS: PENDING`), updated [`docs/geometry/spiralcore.md`](../../../geometry/spiralcore.md), [`docs/mechanisms/spiralcore-fbs.md`](../../../mechanisms/spiralcore-fbs.md), and [`docs/mechanisms/ternary-xor.md`](../../../mechanisms/ternary-xor.md), flipped [`docs/roadmap/items/m6-exact-arithmetic-mechanism.md`](../../../roadmap/items/m6-exact-arithmetic-mechanism.md) to `status: done`, and re-rendered [`docs/roadmap/README.md`](../../../roadmap/README.md).
- Diff size by class: `tests 0 file(s) +0/-0, code 4 file(s) +1227/-0, docs 7 file(s) +185/-2, generated 1 file(s) +3/-4, other 0 file(s) +0/-0` (`1.0x` of the 12-file scope planned in `ORDER.md`).

## Measurements

- **`control_orbit_digit_and_cl06_inverse`** (`~/.cargo/bin/cargo test -p uor-r4-integer --lib -- m6_exact_arithmetic`):
  - `64 / 64` $\mathrm{Cl}(0,6)$ group elements satisfy $g^4 = I$ and $g^3 = g^{-1}$;
  - `256 / 256` (`1.0000` vs `0.0625` un-routed floor) single-place transitions $g = \mathrm{BFS}[r_a \cdot 16 + r_b]$ transport `ROOTS[r_a]` to `ROOTS[r_b]`, recover `r_b` via `snap(x, 1)`, and invert exactly via $\mathrm{action}(g^3, \mathrm{action}(g, \mathrm{ROOTS}[r_a])) = \mathrm{ROOTS}[r_a]$.
- **`control_ripple_carry_add_and_sub`** (`~/.cargo/bin/cargo test -p uor-r4-integer --lib -- m6_exact_arithmetic`):
  - `32 / 32` (`1.0000`) multi-digit decimal (`radix = 10`, up to $10^6$) and hexadecimal (`radix = 16`, up to `0x1FFFF`) ripple-carry addition and borrow subtraction cases match exact 8D orbit atoms and integer values with `100%` step-by-step $\mathrm{Cl}(0,6)$ inverse verification.
- **`control_addressed_memory_composition_and_multiply`** (`~/.cargo/bin/cargo test -p uor-r4-integer --lib -- m6_exact_arithmetic`):
  - `16 / 16` (`1.0000`) multi-step expressions `(A + B) - C` and `(A * B) + C` (executed by multiplier-free shift-and-add over `IntegerAddressedMemory`) match exact expected values, and `8 / 8` multiplication accumulators preserve `[Initial, Current]` version history at place `0`.
- **`control_refusal_and_causal_knockouts`** (`~/.cargo/bin/cargo test -p uor-r4-integer --lib -- m6_exact_arithmetic`):
  - `5 / 5` mandatory failure controls refused (`radix` outside `2..=16` or digit `>= radix`, off-orbit 8D vector, unsigned subtraction underflow `A < B`, fixed-width carry overflow beyond `max_places`, and evicted `IntegerAddressedMemory` slot);
  - Across `16` multi-step `(A + B) - C` items with ripple carry, `AblationArm::None` scores `16 / 16` (`1.0000`), `AblationArm::KnockoutStateTransport` scores `0 / 16` (`0.0000`), and `AblationArm::KnockoutAddressedMemory` scores `0 / 16` (`0.0000`).

## Checks run at the head

- `~/.cargo/bin/cargo test -p uor-r4-integer --lib -- m6_exact_arithmetic` -> `4 passed; 0 failed`
- `~/.cargo/bin/cargo test -p uor-r4-integer --lib --features serving-wasm -- m6_exact_arithmetic` -> `4 passed; 0 failed`
- `~/.cargo/bin/cargo run --locked -p xtask -- roadmap check` -> `88 items OK`
- `uor-harness check` -> `225 rules, 195 failed (0 error, 194 warn)`
- `uor-harness loop gates --full` -> `fmt-claim-wording GREEN`, `fmt-harness GREEN`, `claim-wording GREEN`, `loop-gates-watch GREEN`, `loop-drive GREEN`, `git-hooks GREEN`
- `uor-harness loop report --check` -> `PASS`
- `uor-harness loop pr-check --local` -> `verdict: PASS`

## Skills used

- `uor-research-architect`

## Still open

- GPU evaluation and served-model qualification (`m6-qualify-reasoning-coding` `#2865`) remain HELD pending `m6-reasoning-panel-freeze` (`#2866`), M1 trunk training (`#2845`), owner confirmation of the proposed bar on [`#2028`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6342898483), a non-author `TEST FITNESS: FIT` review on [`docs/labs/m6-exact-arithmetic-mechanism/README.md`](../../../labs/m6-exact-arithmetic-mechanism/README.md), and Casey's approval.

## Plain summary

For the model to reason through multi-step problems and write working code (milestone M6), its arithmetic has to come out exact rather than approximate. This task built an exact multi-digit arithmetic mechanism (`crates/uor-r4-integer/src/m6_exact_arithmetic.rs`) using the same geometric building blocks the rest of the integer model uses: each digit (`0..9` in base 10, or `0..15` in base 16) sits on one of the 16 fixed eight-number lattice points ("E8 orbit roots"), each addition or subtraction step at a digit place moves that lattice point to the new digit's lattice point using one of the 64 reversible coordinate-swap moves ("Cl(0,6) operators", each undone by applying it three times), carries and borrows between places are tracked by a three-way switch (`-1`, `0`, or `+1`), and multi-digit numbers and intermediate results are stored in exact prime-numbered memory slots ("IntegerAddressedMemory") where multiplication is done by shifting places and adding repeatedly with zero hardware multiply instructions. Four deterministic CPU test suites verify every single-digit move and its inverse (`256 / 256`), multi-digit carry addition and borrow subtraction (`32 / 32`), multi-step memory expressions and shift-and-add multiplication (`16 / 16`), and refusal of invalid or evicted inputs plus causal knockout checks (`16 / 16` dropping to `0 / 16` when either geometric transport or addressed memory is switched off).

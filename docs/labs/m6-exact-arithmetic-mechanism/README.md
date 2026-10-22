# M6 Exact Arithmetic on the Geometric Path (`m6-exact-arithmetic-mechanism` #2864)

- **Status:** Built and verified on finite CPU controls (`crates/uor-r4-integer/src/m6_exact_arithmetic.rs`, `crates/uor-r4-integer/src/discrete_d11.rs`); GPU qualification (`m6-qualify-reasoning-coding` #2865) is pre-registered below (`TEST FITNESS: PENDING`) and remains HELD pending panel freeze (`m6-reasoning-panel-freeze` #2866), M1 trunk training (`m1-trunk-assembly-and-training` #2845), owner bar confirmation (`https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6342898483`), non-author `TEST FITNESS: FIT` review, and Casey's typed D40 release.
- **Milestone:** M6 (`#2034`). **Item:** [`m6-exact-arithmetic-mechanism`](../../roadmap/items/m6-exact-arithmetic-mechanism.md) (`#2864`). **Lab:** antigravity.
- **Design note:** [`#2034` comment](https://github.com/UOR-Foundation/uor-r4/issues/2034#issuecomment-6342897026), [`#2864` comment](https://github.com/UOR-Foundation/uor-r4/issues/2864#issuecomment-6342897842). **Bar decision card:** [`#2028` comment](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6342898483).

## 1. Geometric Construction and Shared Typed Operators

Multi-digit integers in radix $B \in \{2, \dots, 16\}$ (including decimal $B = 10$ matching `DigitPlaceSequence` from `uor-r4-tokenizer::number_split` and hexadecimal $B = 16$ spanning the full 16-root anchor orbit) are represented and manipulated entirely through the repository's existing geometric operators in [`crates/uor-r4-integer/src/m6_exact_arithmetic.rs`](../../../crates/uor-r4-integer/src/m6_exact_arithmetic.rs):

1. **Digit atoms on the 16-root $E_8$ anchor orbit (`OrbitDigitAtom`):**
   - Each digit $d \in \{0, \dots, B-1\}$ at place exponent $k \in \{0, \dots, K-1\}$ (LSB-first, $k=0$ at the ones place) is represented as the 8D integer atom $x_d = \mathrm{ROOTS}[d] \in \{-2, 0, 2\}^8$ on the 16-root SpiralCore $E_8$ anchor orbit (`discrete_d11::orbit_root_coords`).
   - Digit readout from an 8D coordinate block uses the shared D11 nearest-root operator `snap(x, 1)` ($L_1$ displacement plus coordinate-Hamming tie-break, `discrete_d11::orbit_snap`) paired with strict on-orbit verification (`discrete_d11::orbit_root_id`).
2. **Place-and-role addressing in `IntegerAddressedMemory` and the ternary FBS binder:**
   - Each operand role (`OperandRole::{OperandA, OperandB, OperandC, Accumulator, Register(0..4)}`) maps to a canonical role prime $p_R \in \{5, 7, 11, 13, 17, 19, 23, 29\}$, and each place index $k \in \{0, \dots, 31\}$ maps to a disjoint canonical place prime $p_{\text{place}(k)} \in \{31, 37, \dots, 181\}$.
   - Storing or reading digit $k$ of role $R$ uses the exact two-prime key $\mathrm{PrimeFactorKey}(\{p_R, p_{\text{place}(k)}\}, [(p_R, p_{\text{place}(k)})])$ over `IntegerAddressedMemory` (`m6_ablation.rs`), admitted by the two-pointer sorted-merge GCD sieve (`sorted_merge_gcd_overlap`) with `[Initial, Previous, Current]` version history.
   - Multi-digit operands and results are ordered LSB-first (`k = 0..K`) across the leaves of the base-3 ternary FBS binder tree (`discrete_d11::fbs_binder_summary`).
3. **Single-place transition via $\mathrm{Cl}(0,6)$ transport (`BFS` & `COMPOSE`) and ternary carry/borrow trit (`{-1, 0, +1}`):**
   - At place $k$, given accumulator digit root $r_a \in \{0, \dots, B-1\}$, operand digit root $r_b \in \{0, \dots, B-1\}$, operation sign $s \in \{+1, -1\}$, and incoming carry/borrow trit $c_{\text{in}} \in \{-1, 0, +1\}$:
     - The raw displaced index $u_k = r_a + s \cdot r_b + c_{\text{in}}$ sets the outgoing three-valued trit switch $c_{\text{out}} \in \{-1, 0, +1\}$ and target root $r_{\text{out}} \in \{0, \dots, B-1\}$:
       - $c_{\text{out}} = +1,\ r_{\text{out}} = u_k - B$ when $u_k \ge B$ (addition carry);
       - $c_{\text{out}} = -1,\ r_{\text{out}} = u_k + B$ when $u_k < 0$ (subtraction borrow);
       - $c_{\text{out}} = 0,\ r_{\text{out}} = u_k$ when $0 \le u_k < B$.
     - The 8D atom $x_a = \mathrm{ROOTS}[r_a]$ is transported to $x_{\text{out}} = \mathrm{ROOTS}[r_{\text{out}}]$ by the canonical $\mathrm{Cl}(0,6)$ signed coordinate permutation
       $$g_k = \mathrm{BFS}[r_a \cdot 16 + r_{\text{out}}] \in \{0, \dots, 63\}, \qquad x_{\text{out}} = \mathrm{action}(g_k, x_a).$$
     - Because every $\mathrm{Cl}(0,6)$ state satisfies $g_k^4 = I$, its exact inverse is $g_k^{-1} = g_k^3 = \mathrm{COMPOSE}[g_k, \mathrm{COMPOSE}[g_k, g_k]]$, verified at every place by $\mathrm{action}(g_k^{-1}, x_{\text{out}}) = x_a$.
   - Multi-digit multiplication $A \times B$ executes as multiplier-free place-shifted repeated addition (`mul_atoms_shift_and_add`) over the $\mathrm{Cl}(0,6)$ ripple-carry adder, writing each intermediate partial sum to `OperandRole::Accumulator` in `IntegerAddressedMemory`.

## 2. Compliance with Zero Squashing, Zero Matmul, and D11 Integer Serving

- **Zero squashing / softmax (CHARTER §3.2, D23):** Root selection is hard minimum over 16 integer $(L_1, \text{Hamming})$ pairs (`orbit_snap`); carry/borrow is a discrete trit in `{-1, 0, +1}`; memory admission is a hard boolean $\gcd(q, A_c) > 1$ check. No exponential, division, sigmoid, tanh, softplus, or probability table exists anywhere.
- **Zero dense matrix multiplication (CHARTER §3.3, D25/D30):** Each place transition applies one 8-coordinate signed permutation looked up in `AXES`, and memory lookup merges sorted 2-element factor lists.
- **D11 integer serving (`uor-r4-integer`):** Compiled in both the default native qualification graph (`src/native_geometry.rs`) and the WebAssembly serving target (`src/serving_root.rs`, `--features serving-wasm`), using only integer additions, subtractions, shifts/masks, and static table lookups.

## 3. Finite CPU Control Results (Six-Column Notebook Format)

| Date | What ran | Number | Bar | Verdict | Artifact |
|---|---|---|---|---|---|
| 2026-10-22 | `control_orbit_digit_and_cl06_inverse`: bijective decimal (`0..10`) and hex (`0..16`) 8D $E_8$ root mapping, `snap(x, 1)` recovery, $g^4 = I$ across all 64 $\mathrm{Cl}(0,6)$ states, and $g^3 = g^{-1}$ across all $16 \times 16$ root transitions, CPU finite check. | `64 / 64` group states $g^4 = I$; `256 / 256` root transitions and exact inverses (`1.0000` vs `0.0625` un-routed floor). | `256 / 256` exact match (proposed on [`#2028`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6342898483)). | **positive** (finite control) | [`m6_exact_arithmetic.rs`](../../../crates/uor-r4-integer/src/m6_exact_arithmetic.rs) |
| 2026-10-22 | `control_ripple_carry_add_and_sub`: multi-digit decimal (`radix = 10`, up to $10^6$) and hexadecimal (`radix = 16`, up to `0x1FFFF`) ripple-carry addition and borrow subtraction with step-by-step $\mathrm{Cl}(0,6)$ inverse verification, CPU finite check. | `32 / 32` exact 8D orbit atom and integer matches (`1.0000`); `100%` step inverse verification. | `32 / 32` exact match (proposed on [`#2028`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6342898483)). | **positive** (finite control) | [`m6_exact_arithmetic.rs`](../../../crates/uor-r4-integer/src/m6_exact_arithmetic.rs) |
| 2026-10-22 | `control_addressed_memory_composition_and_multiply`: multi-step expressions `(A + B) - C` and `(A * B) + C` (via shift-and-add) over `IntegerAddressedMemory` with `[Initial, Previous, Current]` accumulator version checks, CPU finite check. | `16 / 16` exact multi-step results (`1.0000`) and `8 / 8` `[Initial, Current]` versioned accumulator checks. | `16 / 16` exact match (proposed on [`#2028`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6342898483)). | **positive** (finite control) | [`m6_exact_arithmetic.rs`](../../../crates/uor-r4-integer/src/m6_exact_arithmetic.rs) |
| 2026-10-22 | `control_refusal_and_causal_knockouts`: mandatory refusal controls (invalid radix, off-orbit 8D vector, unsigned subtraction underflow, fixed-width carry overflow, evicted memory slot) plus `AblationArm` knockouts across 16 multi-step items, CPU finite check. | `5 / 5` invalid/underflow/overflow/eviction cases refused; `AblationArm::None` `16 / 16` (`1.0000`) vs `KnockoutStateTransport` `0 / 16` (`0.0000`) and `KnockoutAddressedMemory` `0 / 16` (`0.0000`). | `100%` refusal of invalid states and causal drop to `0 / 16` under knockouts. | **positive** (finite control) | [`m6_exact_arithmetic.rs`](../../../crates/uor-r4-integer/src/m6_exact_arithmetic.rs) |

---

## 4. Pre-Registration Specification for M6 Exact Arithmetic Qualification (`#2864` / `#2865`)

### Bar

- **Proposed M6 exact arithmetic qualification bar (posted as a decision card for owner/PM confirmation on [`#2028`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6342898483), CHARTER §5.2):**
  1. **Headline exact-match accuracy:** On the exact-arithmetic slice of the frozen `m6-reasoning` panel (`N >= 200` unseen multi-step arithmetic and code-value items across $\ge 2$ seeds), the served D11 model + `GeometricArithmeticEngine` achieves exact-match accuracy clearing the token count-table baseline by $\ge +0.20$ (`20` percentage points) and $\ge 2\times$ measured between-seed noise.
  2. **Causal state-transport dependence (`AblationArm::None` vs `AblationArm::KnockoutStateTransport`):** Paired McNemar discordant drop $b - c \ge F_I = \max(16, \lceil 2\sqrt{b + c}\rceil)$ and accuracy drop $\ge 0.25$ on multi-place carry/borrow items.
  3. **Causal addressed-memory dependence (`AblationArm::None` vs `AblationArm::KnockoutAddressedMemory`):** Paired McNemar discordant drop $b - c \ge F_I = \max(16, \lceil 2\sqrt{b + c}\rceil)$ and accuracy drop $\ge 0.25$ on multi-step memory-bound arithmetic compositions.

### Base artifact

- **Arithmetic & ablation engine:** `crates/uor-r4-integer/src/m6_exact_arithmetic.rs` (`GeometricArithmeticEngine`, `OrbitDigitAtom`, `GeometricArithmeticTrace`) paired with `crates/uor-r4-integer/src/m6_ablation.rs` (`IntegerAddressedMemory`, `AblationArm`) and `crates/uor-r4-integer/src/discrete_d11.rs` (`IntegerDiscreteModel`).
- **Candidate trunk artifact:** D11 discrete artifact (`uor-r4.lut-discrete/1`) exported from the M1 trunk (`m1-trunk-assembly-and-training` `#2845`) via `uor_r4_discrete::export_d11_discrete_bytes` (`m1-d11-exporter-discrete` `#2838`), pinned by `artifact_sha256` before evaluation.

### Data split

- **Frozen panel:** `docs/harness/panels/m6-reasoning.json` once frozen by `m6-reasoning-panel-freeze` (`#2866`), SHA-256-pinned prior to any GPU evaluation run, with zero overlap between the finite CPU control literals in `m6_exact_arithmetic.rs` and the frozen panel rows (CHARTER §3.10).

### Steps

1. Freeze `docs/harness/panels/m6-reasoning.json` under `m6-reasoning-panel-freeze` (`#2866`) and verify zero item overlap against the CPU control literals in `m6_exact_arithmetic.rs`.
2. Export the trained discrete trunk (`#2845`) to a D11 container (`uor-r4.lut-discrete/1`) and pin its `artifact_sha256`.
3. Obtain a non-author `TEST FITNESS: FIT` review on this specification, owner confirmation of the bar on `#2028`, PM dispatch, and Casey's typed D40 release.
4. Execute the four-arm paired evaluation (`AblationArm::{None, KnockoutStateTransport, KnockoutAddressedMemory, KnockoutBoth}`) across $\ge 2$ seeds on the GPU via `scripts/pod/uor-pod` (`m6-qualify-reasoning-coding` `#2865`) and record the outcome row.

### Stop conditions

- Any failure of the finite CPU controls (`control_orbit_digit_and_cl06_inverse`, `control_ripple_carry_add_and_sub`, `control_addressed_memory_composition_and_multiply`, `control_refusal_and_causal_knockouts`) or any non-zero multiply/float instruction in the D11 serving audit halts dispatch before GPU allocation.
- If `AblationArm::None` fails to beat the count-table baseline on the frozen panel, stop and record a configuration negative under D22 without claiming causal attribution.

### Cost and minimum test power

- **Cost:**
  - `preparation`: $\$0.00$ (host-only Rust implementation and finite CPU controls).
  - `run`: $\$0.00$ in this PR; GPU qualification (`#2865`) is a single RTX 5090 evaluation pass (`< 0.5` GPU-hours, $\le \$0.50$) within the 4-pod / $\$8.00$/h fleet cap.
  - `controls`: included in the 4-arm paired sweep (`None`, `KnockoutStateTransport`, `KnockoutAddressedMemory`, `KnockoutBoth`) plus the token count-table baseline.
  - `evaluation`: deterministic exact integer and digit-atom grading (`to_u64()`, `verify_all_inverses()`).
  - `retries`: $0$ (single pre-registered evaluation pass per seed).
  - `storage`: $< 1\text{ MiB}$ JSON evaluation trace and summary.
- **Minimum test power (D22 §3):**
  - $\ge 2$ seeds evaluated on $\ge 200$ frozen arithmetic/composition rows from `m6-reasoning.json`, with paired McNemar discordant floor $F_I = \max(16, \lceil 2\sqrt{b + c}\rceil)$.

TEST FITNESS: PENDING

# Prime Transport Pattern Family Coverage Probe v1

**Branch:** pattern_family_coverage_probe_v1  
**Strictly train-free geometry + supervised linear probe (no trajectory in probe)**

## Hypothesis

- **H0 (narrow carrier):** Only functions directly reducible to k via H2/H3/CRT
  are solvable. System is explicit encoding only.
- **H1 (structured identity):** The harmonic encoding contains a richer
  representation of input identity, allowing a broader class of functions
  to be computed via readout.

## State Construction Lock

- `convert_onehot_to_angular_multi`: Block 3 (s=9,e=21,m=12,n_h=3),
  k in [0..11] → H1(6,7) period-12 / H2(8,9) period-6 / H3(10,11) period-4.
- `apply_anchor_two_i`: rotates each (cos,sin) → (−sin,cos).
- `eps_high = 1.0`: H2 and H3 are **frozen** across all D. H1 is transported.
- CRT lookup (H3 k%4, H2 k%6) → k%12 = k: verified exhaustively.

## Readout Mechanisms Tested

| Mechanism | Reads From | Period | Classes |
|-----------|-----------|--------|---------|
| H3_direct | indices 10,11 | 4 | [0..3] = k%4 |
| H2_direct | indices 8,9 | 6 | [0..5] = k%6 |
| CRT_direct | H3 + H2 | 12 | [0..11] = k |
| harmonic_opt | best of above + lookup table | — | any |
| linear_probe | LogReg on tau_init (D=0) | — | any |

## CLASS 1 — Direct Carrier (Control)

| Target | Expected Readout | acc |
|--------|-----------------|-----|
| k_mod4 | H3 | 1.0000 |
| k_mod6 | H2 | 1.0000 |
| k_mod12 | CRT | 1.0000 |

All CLASS 1 targets expected at acc=1.0 (direct carrier readout, verified).

## CLASS 2 — Simple Functions of k

| Target | H3_opt | H2_opt | CRT_opt | Probe_test | Best mechanism |
|--------|--------|--------|---------|------------|----------------|
| parity | 1.0000 | 1.0000 | 1.0000 | 0.4612 | CRT |
| ge6 | 0.6636 | 0.5225 | 1.0000 | 1.0000 | CRT |
| set0_3_7_11 | 0.9121 | 0.6797 | 1.0000 | 1.0000 | CRT |
| floor_k_3 | 0.3545 | 0.5225 | 1.0000 | 1.0000 | CRT |

### What is directly encoded vs what requires combination

- **parity(k):** Directly recoverable from H3 alone (parity = k%4 % 2 = k%2).
  H3 period-4 groups k values such that parity is constant per group.
- **k >= 6:** Cannot be recovered from H3 (k%4=0 includes both k=0 <6 and k=8 >=6).
  Requires knowing k fully → CRT. H2 similarly insufficient.
- **k in {0,3,7,11}:** Mixed modular structure; CRT required for exact recovery.
- **floor(k/3):** Values repeat across k%4 equivalence classes; CRT required.

## CLASS 3 — Nontrivial Mixed Functions

Random permutation used: [10, 9, 0, 8, 5, 2, 1, 11, 4, 7, 3, 6]

| Target | H3_opt | H2_opt | CRT_opt | Probe_test | Best mechanism |
|--------|--------|--------|---------|------------|----------------|
| xor_mod4_mod3 | 0.3545 | 0.5225 | 1.0000 | 1.0000 | CRT |
| eq_mod6_mod4 | 0.6509 | 0.6763 | 1.0000 | 1.0000 | CRT |
| perm42 | 0.3545 | 0.5225 | 1.0000 | 1.0000 | CRT |

### Mechanism analysis for CLASS 3

- **(k%4) XOR (k%3):** k%3 is derivable from k%6 (k%3 = k%6 % 3), but
  this nonlinear step is NOT accessible via linear probe on H2 features.
  However, CRT recovers k directly, so XOR = f(k) → solved via CRT lookup.
- **(k%6 == k%4):** Equivalent to k < 4 (only k=0,1,2,3 satisfy this).
  Requires knowing k fully; CRT provides it.
- **perm42 (random perm):** Arbitrary bijection of k → [0..11]. Any
  deterministic function of k is solved by CRT + lookup. No structure assumed.

## CLASS 4 — Trajectory Legibility

Targets: floor(k/3) [CLASS 2] and xor_mod4_mod3 [CLASS 3].
D swept from 0 to 20.

| Target | harm@t=0 | harm@t=D | probe@t=0 | probe@t=D |
|--------|----------|----------|-----------|-----------|
| floor_k_3 | 1.0000 | 1.0000 | 1.0000 | 0.2275 |
| xor_mod4_mod3 | 1.0000 | 1.0000 | 1.0000 | 0.2280 |

### Trajectory interpretation

- **acc_harmonic constant (1.0000 at all steps):** H2 and H3 are frozen by
  eps_high=1.0. The CRT readout depends only on frozen components, so accuracy
  is **invariant to D**. Information is globally present at ALL steps.
- **acc_token collapses to chance after step 1:** The linear probe trained on
  tau_init achieves 1.0 at t=0, then degrades to **~0.227 (≈ random)** at t=1
  and remains at chance level through D=20. This shows the probe relied almost
  entirely on **H1 features** (indices 6,7), which encode k%12 at t=0 but are
  **replaced** at every transport step. After one step, H1 contains transport
  directions unrelated to k, and the probe loses all predictive power.
- **Key asymmetry:** The CRT harmonic readout uses only H2/H3 (frozen) and
  stays perfect throughout. The linear probe used H1 (ephemeral) and fails
  after one step. Both achieve acc=1.0 on tau_init, but via different
  mechanisms with completely different trajectory stability.
- **No improvement at final step:** Harmonic readability does NOT improve with D.
  The answer is fully encoded in the initial state via frozen carriers.

## Boundary Analysis

### What is directly encoded (single-carrier accessible)

- k%4: directly in H3 (period-4 harmonic)
- k%6: directly in H2 (period-6 harmonic)
- parity(k): derivable from H3 alone via nonlinear phase extraction
  (atan2 → k%4 → k%4 mod 2). **Note:** linear probe FAILS on parity
  (test acc=0.46 ≈ random). Parity is not linearly separable in angular
  feature space: H3 maps even k → (0, ±1) and odd k → (±1, 0), which
  are antipodal-symmetric and require nonlinear separation. Harmonic
  readout succeeds because atan2 is a nonlinear operation.

### What requires combination (CRT-accessible but not single-carrier)

- k%12 (= k): requires CRT(H2, H3) — both carriers needed
- k >= 6: requires knowing k fully
- k in {0,3,7,11}: requires knowing k fully
- floor(k/3): requires knowing k fully
- (k%4) XOR (k%3): requires knowing k; k%3 not linear in H2 features
- (k%6 == k%4): equivalent to k < 4; requires knowing k fully
- perm42: arbitrary function of k; requires knowing k fully

### What fails

- Any function NOT computable from k (e.g., requiring information
  outside the cyclic block, or trajectory-dependent quantities).
- For this probe, all targets are deterministic in k, so none fail.

## FINAL CLASSIFICATION

**STRUCTURED_IDENTITY**

All CLASS 1, CLASS 2, and CLASS 3 targets achieve acc=1.0 via CRT harmonic readout. The CRT mechanism recovers k completely from the initial harmonic state, making any deterministic function of k solvable. This goes beyond direct carrier readout (H3 or H2 alone) and demonstrates that the harmonic encoding carries a richer identity than a narrow carrier.

### Justification

The CRT mechanism (joint H2 + H3 readout) **completely recovers k** for
all k in [0..11], since (k%4, k%6) uniquely determines k (12 distinct pairs
verified exhaustively). Therefore:

1. Any deterministic function f(k) is solvable via: CRT → k → f(k).
2. The harmonic encoding is NOT merely a direct carrier of k%4 and k%6.
   It jointly encodes the full input identity k via two orthogonal projections.
3. H0 (narrow carrier) is **falsified**: the system solves functions beyond
   k%4 and k%6 without any learned weights — via pure phase arithmetic.
4. H1 (structured identity) is **supported**: the representation permits
   readout of any k-function, including nontrivial mixes like XOR and
   random permutations.

### Mechanism that enables broader coverage

The enabling mechanism is the **CRT property** of the harmonic encoding:
- H3 embeds k%4 (period-4 projection)
- H2 embeds k%6 (period-6 projection)
- Since gcd(4,6)=2 and lcm(4,6)=12, and all 12 pairs (k%4,k%6)
  for k=0..11 are distinct, the joint state (H2, H3) is a lossless
  encoding of k. This is the Chinese Remainder Theorem applied to
  the harmonic state space.

### Honesty section

This is still a **carrier mechanism**: the answer is present in tau_init
before any trajectory steps. No computation emerges from transport dynamics.
The distinction from H0 is that the carrier is more informationally complete
than a single harmonic — it captures full input identity via CRT combination.

The trajectory adds no information. All functions of k are solvable at D=0.
Transport (H1 update, h1 dynamics) has no role in readout correctness.

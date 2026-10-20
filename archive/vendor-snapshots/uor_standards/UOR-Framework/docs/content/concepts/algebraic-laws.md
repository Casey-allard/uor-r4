# Algebraic Laws

## Definition

The UOR Foundation ontology formalizes **7 core algebras** (with additional identity families from Amendments 23–53) that govern computation
over the ring R_n = Z/(2^n)Z. Each algebra is encoded as a set of named
{@class https://uor.foundation/op/Identity} individuals in the `op/` namespace,
with `lhs`, `rhs`, and `forAll` properties specifying the algebraic equation
and its quantifier domain.

## The Seven Algebras

### 1. Ring Algebra (R_A, R_M)
The additive group (R_A1–R_A6) and multiplicative monoid (R_M1–R_M5) axioms
for the commutative ring Z/(2^n)Z. These are the foundation: associativity,
commutativity, identity elements, inverses, and distributivity.

### 2. Boolean Algebra (B_)
Thirteen identities (B_1–B_13) encoding XOR, AND, OR, and BNOT as a Boolean
algebra on the n-bit representation. Includes De Morgan's laws (B_11, B_12)
and the involution property (B_13).

### 3. Cross-Structure Laws (X_)
Seven identities (X_1–X_7) connecting ring operations to Boolean operations.
The key identity X_5: `neg(x) = add(bnot(x), 1)` (two's complement) bridges
the ring and Boolean worlds.

### 4. Dihedral Group (D_)
Four identities (D_1, D_3–D_5) describing the dihedral group D_{2^n} generated
by negation and bitwise complement. D_5 gives the full presentation.

### 5. Unit Group (U_)
Five identities (U_1–U_5) characterizing the group of invertible elements
R_n× ≅ Z/2 × Z/2^{n-2} (for n ≥ 3), including order formulas and the
step function.

### 6. Affine Group (AG_)
Four identities (AG_1–AG_4) describing the affine group Aff(R_n) = R_n× ⋉ R_n,
which extends the dihedral group by non-trivial unit multiplications.

### 7. Carry Algebra (CA_)
Six identities (CA_1–CA_6) encoding the carry propagation rules that govern
how ring addition differs from Boolean XOR. The Witt identification (WC_1–WC_12)
proves these are the 2-typical Witt addition polynomials over F_2. The Ostrowski–
Archimedean bridge (OA_1–OA_5) grounds the Landauer temperature β* = ln 2 via the
product formula at p=2.

## Cross-Algebra Maps

Six inter-algebra maps (phi_1–phi_6) formalize the relationships between
algebras. See the **Resolution** concept page for the full pipeline
φ₄ = φ₃ ∘ φ₂ ∘ φ₁.

## Thermodynamic Interpretation

The identities TH_1–TH_10 reinterpret the resolution process as a
thermodynamic system: site entropy, Landauer bounds, and phase transitions
provide physical intuition for computational cost.

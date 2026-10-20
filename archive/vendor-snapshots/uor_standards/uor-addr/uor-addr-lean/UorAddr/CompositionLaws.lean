/-!
# ADR-061 categorical composition — structural laws.

Mirrors `crate::composition::canonicalize`: the five categorical
operations on the Atlas image inside E₈. Each Rust canonicalize
discipline produces canonical-form bytes of a fixed width and upholds a
named algebraic law (wiki ADR-061 §(3)). This module proves, with no
mathlib:

* the canonical-form **width** of each operation
  (`canonicalize_g2` → `2N`, `canonicalize_e6` → `N + 1`,
  `canonicalize_f4`/`canonicalize_e7`/`canonicalize_e8` → `N`);
* CS-G2 **commutativity** of the lex-min-first product (the heart of
  the `g2_is_commutative` Rust behavior test);
* the CS-E7 **S₄ orbit** cardinality (`24`) and distinctness;
* the CS-E6 degree-partition **8:1 ratio** (ADR-059's 64:8 vertex
  partition, reduced mod 9);
* the CS-F4 ± involution (`byteComplement` is its own inverse);
* the CS-E8 **identity**.

Every proof is `rfl` / `decide` / `omega` plus one short case split.
-/
namespace UorAddr.CompositionLaws

-- ─── Canonical-form widths (mirror `canonicalize_*` output lengths) ──

/-- CS-G2: `lo ‖ hi` — the concatenation of two N-byte operand κ-labels. -/
def widthG2 (n : Nat) : Nat := n + n
/-- CS-F4: `<axis>:<hex>` re-encodes the same N-byte κ-label. -/
def widthF4 (n : Nat) : Nat := n
/-- CS-E6: a one-byte degree tag is prepended → `N + 1`. -/
def widthE6 (n : Nat) : Nat := n + 1
/-- CS-E7: a quarter-permutation of the same digest → `N`. -/
def widthE7 (n : Nat) : Nat := n
/-- CS-E8: the identity → `N`. -/
def widthE8 (n : Nat) : Nat := n

-- The four 32-byte axes share a 71-byte κ-label (sha256/blake3); the
-- two longer ones are sha3-256 (73), keccak256 (74); sha512 is 135.
theorem g2_width_sha256 : widthG2 71 = 142 := rfl
theorem g2_width_sha512 : widthG2 135 = 270 := rfl
theorem e6_width_sha256 : widthE6 71 = 72 := rfl
theorem e6_width_sha512 : widthE6 135 = 136 := rfl
theorem f4_width_preserves (n : Nat) : widthF4 n = n := rfl
theorem e7_width_preserves (n : Nat) : widthE7 n = n := rfl
theorem e8_width_is_identity (n : Nat) : widthE8 n = n := rfl

-- ─── CS-G2 — commutativity of the lex-min-first product ──────────────

/-- The lex-min-first product as the *ordered pair* of its operands; the
`lo ‖ hi` canonical form is the concatenation of this pair, so its
order-independence is exactly the pair's. -/
def g2Pair {α : Type} (le : α → α → Bool) (a b : α) : α × α :=
  bif le a b then (a, b) else (b, a)

/-- CS-G2 commutativity: for a total, antisymmetric comparator the
ordered pair — hence the composed κ-label — is independent of operand
order. Mirrors `canonicalize::tests::g2_is_commutative`. -/
theorem g2_comm {α : Type} (le : α → α → Bool)
    (total : ∀ a b, le a b = true ∨ le b a = true)
    (antisymm : ∀ a b, le a b = true → le b a = true → a = b)
    (a b : α) : g2Pair le a b = g2Pair le b a := by
  cases h : le a b <;> cases h2 : le b a <;>
    simp only [g2Pair, h, h2, cond_true, cond_false]
  · -- le a b = false, le b a = false — impossible by totality.
    rcases total a b with hh | hh <;> simp_all
  · -- le a b = true, le b a = true — antisymmetry forces a = b.
    rw [antisymm a b h h2]

-- ─── CS-E7 — the S₄ quarter-permutation orbit ────────────────────────

/-- The 24 quarter-index permutations of S₄ (mirrors `S4_PERMUTATIONS`). -/
def s4Permutations : List (List Nat) :=
  [ [0,1,2,3], [0,1,3,2], [0,2,1,3], [0,2,3,1], [0,3,1,2], [0,3,2,1],
    [1,0,2,3], [1,0,3,2], [1,2,0,3], [1,2,3,0], [1,3,0,2], [1,3,2,0],
    [2,0,1,3], [2,0,3,1], [2,1,0,3], [2,1,3,0], [2,3,0,1], [2,3,1,0],
    [3,0,1,2], [3,0,2,1], [3,1,0,2], [3,1,2,0], [3,2,0,1], [3,2,1,0] ]

/-- `|S₄| = 24` — the orbit the CS-E7 lex-min ranges over. -/
theorem s4_card : s4Permutations.length = 24 := rfl

/-- Boolean all-distinct check (avoids a `Nodup` decidability instance). -/
def allDistinct : List (List Nat) → Bool
  | [] => true
  | x :: xs => (!xs.contains x) && allDistinct xs

/-- The orbit enumerates 24 *distinct* permutations. -/
theorem s4_distinct : allDistinct s4Permutations = true := by decide

/-- Every member is a rearrangement of `[0,1,2,3]` (a genuine S₄ element). -/
theorem s4_all_permute_0123 :
    s4Permutations.all
      (fun p => p.length == 4 && [0,1,2,3].all (fun i => p.contains i)) = true := by
  decide

-- ─── CS-E6 — the degree-partition (ADR-059 64:8, reduced mod 9) ───────

/-- degree tag from the first raw-digest byte: `0x06` iff `byte % 9 = 8`,
else `0x05` (mirrors the Rust `match raw[0] % 9`). -/
def degreeTag (firstByte : Nat) : Nat :=
  match firstByte % 9 with
  | 8 => 0x06
  | _ => 0x05

/-- A full residue system mod 9. -/
def residues : List Nat := [0,1,2,3,4,5,6,7,8]

/-- 8 of the 9 residues map to degree-5 — the high-population class. -/
theorem e6_degree5_count :
    (residues.filter (fun r => degreeTag r == 0x05)).length = 8 := by decide

/-- 1 of the 9 residues maps to degree-6 — the 8:1 ratio of ADR-059. -/
theorem e6_degree6_count :
    (residues.filter (fun r => degreeTag r == 0x06)).length = 1 := by decide

-- ─── CS-F4 — the ± involution (bitwise complement) ───────────────────

/-- A byte's ± mirror: bitwise complement, modelled as `255 - b`. -/
def byteComplement (b : Nat) : Nat := 255 - b

/-- The CS-F4 mirror is an involution on bytes — applying it twice
recovers the operand, so the 2-element equivalence class `{x, ~x}` is
well-defined. -/
theorem f4_complement_involutive (b : Nat) (h : b ≤ 255) :
    byteComplement (byteComplement b) = b := by
  unfold byteComplement; omega

end UorAddr.CompositionLaws

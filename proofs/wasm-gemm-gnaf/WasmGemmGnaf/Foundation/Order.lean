import WasmGemmGnaf.Foundation.Bytes
set_option autoImplicit false

/-!
# Foundation: the canonical total byte order (SPEC §9.1)

`UnsignedLexicographicLE` treats a proper prefix as smaller and otherwise
compares the first differing bytes as unsigned naturals.  SPEC §6.2 requires map
and set encodings to use a *proved* total canonical order, so reflexivity,
transitivity, antisymmetry, totality and decidability are all discharged here.
-/

namespace WasmGemmGnaf.Foundation

/-- Unsigned lexicographic order on byte lists (SPEC §9.1). -/
def UnsignedLexicographicLE : List UInt8 → List UInt8 → Prop
  | [], _ => True
  | _ :: _, [] => False
  | a :: as, b :: bs =>
      a.toNat < b.toNat ∨ (a.toNat = b.toNat ∧ UnsignedLexicographicLE as bs)

def CanonicalBytesLE (left right : ByteArray) : Prop :=
  UnsignedLexicographicLE left.toList right.toList

namespace UnsignedLexicographicLE

/-! ## Defining equations -/

@[simp] theorem nil_left (b : List UInt8) : UnsignedLexicographicLE [] b := by
  simp [UnsignedLexicographicLE]

@[simp] theorem cons_nil {a : UInt8} {as : List UInt8} :
    ¬ UnsignedLexicographicLE (a :: as) [] := by
  simp [UnsignedLexicographicLE]

theorem cons_cons {a b : UInt8} {as bs : List UInt8} :
    UnsignedLexicographicLE (a :: as) (b :: bs) ↔
      a.toNat < b.toNat ∨ (a.toNat = b.toNat ∧ UnsignedLexicographicLE as bs) := by
  simp [UnsignedLexicographicLE]

/-! ## Order laws -/

theorem refl : ∀ a : List UInt8, UnsignedLexicographicLE a a := by
  intro a
  induction a with
  | nil => exact nil_left []
  | cons x xs ih => exact cons_cons.mpr (Or.inr ⟨rfl, ih⟩)

theorem trans : ∀ a b c : List UInt8,
    UnsignedLexicographicLE a b → UnsignedLexicographicLE b c →
    UnsignedLexicographicLE a c := by
  intro a
  induction a with
  | nil => intro b c _ _; exact nil_left c
  | cons x xs ih =>
    intro b c hab hbc
    cases b with
    | nil => exact absurd hab cons_nil
    | cons y ys =>
      cases c with
      | nil => exact absurd hbc cons_nil
      | cons z zs =>
        rw [cons_cons] at hab hbc
        rw [cons_cons]
        rcases hab with h1 | ⟨h1, h2⟩ <;> rcases hbc with h3 | ⟨h3, h4⟩
        · exact Or.inl (by omega)
        · exact Or.inl (by omega)
        · exact Or.inl (by omega)
        · exact Or.inr ⟨by omega, ih ys zs h2 h4⟩

theorem antisymm : ∀ a b : List UInt8,
    UnsignedLexicographicLE a b → UnsignedLexicographicLE b a → a = b := by
  intro a
  induction a with
  | nil =>
    intro b _ h2
    cases b with
    | nil => rfl
    | cons y ys => exact absurd h2 cons_nil
  | cons x xs ih =>
    intro b h1 h2
    cases b with
    | nil => exact absurd h1 cons_nil
    | cons y ys =>
      rw [cons_cons] at h1 h2
      rcases h1 with p | ⟨p, q⟩ <;> rcases h2 with r | ⟨r, t⟩
      · exact absurd p (by omega)
      · exact absurd p (by omega)
      · exact absurd r (by omega)
      · have hxy : x = y := UInt8.toNat_inj.mp p
        have hxs : xs = ys := ih ys q t
        rw [hxy, hxs]

theorem total : ∀ a b : List UInt8,
    UnsignedLexicographicLE a b ∨ UnsignedLexicographicLE b a := by
  intro a
  induction a with
  | nil => intro b; exact Or.inl (nil_left b)
  | cons x xs ih =>
    intro b
    cases b with
    | nil => exact Or.inr (nil_left _)
    | cons y ys =>
      rcases Nat.lt_trichotomy x.toNat y.toNat with h | h | h
      · exact Or.inl (cons_cons.mpr (Or.inl h))
      · rcases ih ys with hl | hr
        · exact Or.inl (cons_cons.mpr (Or.inr ⟨h, hl⟩))
        · exact Or.inr (cons_cons.mpr (Or.inr ⟨h.symm, hr⟩))
      · exact Or.inr (cons_cons.mpr (Or.inl h))

/-! ## Decidability -/

/-- Executable form of the canonical order. -/
def decide : List UInt8 → List UInt8 → Bool
  | [], _ => true
  | _ :: _, [] => false
  | a :: as, b :: bs =>
      Decidable.decide (a.toNat < b.toNat) ||
        (Decidable.decide (a.toNat = b.toNat) && decide as bs)

theorem decide_iff : ∀ a b : List UInt8,
    decide a b = true ↔ UnsignedLexicographicLE a b := by
  intro a
  induction a with
  | nil => intro b; simp [decide]
  | cons x xs ih =>
    intro b
    cases b with
    | nil => simp [decide]
    | cons y ys =>
      rw [cons_cons]
      simp [decide, ih ys]

end UnsignedLexicographicLE

instance instDecidableUnsignedLexicographicLE (a b : List UInt8) :
    Decidable (UnsignedLexicographicLE a b) :=
  if h : UnsignedLexicographicLE.decide a b = true then
    isTrue ((UnsignedLexicographicLE.decide_iff a b).mp h)
  else
    isFalse (fun hc => h ((UnsignedLexicographicLE.decide_iff a b).mpr hc))

namespace CanonicalBytesLE

theorem refl (a : ByteArray) : CanonicalBytesLE a a :=
  UnsignedLexicographicLE.refl _

theorem trans {a b c : ByteArray}
    (hab : CanonicalBytesLE a b) (hbc : CanonicalBytesLE b c) :
    CanonicalBytesLE a c :=
  UnsignedLexicographicLE.trans _ _ _ hab hbc

/-- Antisymmetry at the level of the byte lists. -/
theorem antisymm_toList {a b : ByteArray}
    (hab : CanonicalBytesLE a b) (hba : CanonicalBytesLE b a) :
    a.toList = b.toList :=
  UnsignedLexicographicLE.antisymm _ _ hab hba

/-- Antisymmetry at the level of the byte arrays themselves. -/
theorem antisymm {a b : ByteArray}
    (hab : CanonicalBytesLE a b) (hba : CanonicalBytesLE b a) : a = b :=
  Bytes.toList_injective (antisymm_toList hab hba)

theorem total (a b : ByteArray) : CanonicalBytesLE a b ∨ CanonicalBytesLE b a :=
  UnsignedLexicographicLE.total _ _

end CanonicalBytesLE

instance instDecidableCanonicalBytesLE (a b : ByteArray) :
    Decidable (CanonicalBytesLE a b) :=
  instDecidableUnsignedLexicographicLE _ _

/-! ## Sorted canonical sequences

Map and set encodings (SPEC §6.2) must present their elements in the canonical
order; `CanonicalSorted` is the checkable side condition. -/

/-- A list of canonical byte strings in nondecreasing canonical order. -/
def CanonicalSorted : List ByteArray → Prop
  | [] => True
  | [_] => True
  | x :: y :: rest => CanonicalBytesLE x y ∧ CanonicalSorted (y :: rest)

@[simp] theorem canonicalSorted_nil : CanonicalSorted [] := by
  simp [CanonicalSorted]

@[simp] theorem canonicalSorted_singleton (x : ByteArray) : CanonicalSorted [x] := by
  simp [CanonicalSorted]

theorem canonicalSorted_cons_cons {x y : ByteArray} {rest : List ByteArray} :
    CanonicalSorted (x :: y :: rest) ↔
      CanonicalBytesLE x y ∧ CanonicalSorted (y :: rest) := by
  simp [CanonicalSorted]

instance instDecidableCanonicalSorted : ∀ l : List ByteArray,
    Decidable (CanonicalSorted l)
  | [] => isTrue canonicalSorted_nil
  | [x] => isTrue (canonicalSorted_singleton x)
  | _ :: y :: rest =>
    have : Decidable (CanonicalSorted (y :: rest)) := instDecidableCanonicalSorted (y :: rest)
    decidable_of_iff _ canonicalSorted_cons_cons.symm

end WasmGemmGnaf.Foundation

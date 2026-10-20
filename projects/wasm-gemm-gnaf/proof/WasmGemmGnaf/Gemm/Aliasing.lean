import WasmGemmGnaf.Gemm.Layout
set_option autoImplicit false

/-!
# Gemm: byte ranges and the alias contract (SPEC §8.3)

> Aliasing tags are `disjoint = 0`, `cMayAliasAAfterRead = 1`, and
> `cMayAliasBAfterRead = 2`.
>
> A and B are read-only and may overlap each other. `disjoint` requires C to be
> disjoint from both. `cMayAliasAAfterRead` permits overlap only with A and
> defines the result against an immutable logical snapshot of A taken before the
> first C write; `cMayAliasBAfterRead` is symmetric. … **No tag permits C to
> overlap both A and B.**
>
> The header, scratch, and status-detail ranges are pairwise disjoint and each
> is disjoint from A, B, and C. … Any other overlap is `invalid` at the
> alias-validation precedence step.

`no_tag_permits_both` is proved from the table rather than asserted, and the
"overlapping byte count" that SPEC §8.3 feeds into the status-detail record is
the explicit `ByteRange.overlapCount` defined here.
-/

namespace WasmGemmGnaf.Gemm

/-- A half-open byte range `[start, start + length)` relative to `ptr`. -/
structure ByteRange where
  start : Nat
  length : Nat
  deriving DecidableEq, Repr, Inhabited

namespace ByteRange

/-- The exclusive end of the range. -/
def stop (r : ByteRange) : Nat := r.start + r.length

def isEmpty (r : ByteRange) : Bool := r.length == 0

/-- Byte `i` belongs to the range. -/
def Mem (r : ByteRange) (i : Nat) : Prop := r.start ≤ i ∧ i < r.stop

instance (r : ByteRange) (i : Nat) : Decidable (r.Mem i) :=
  inferInstanceAs (Decidable (_ ∧ _))

/-- Two ranges share at least one byte. -/
def Overlaps (a b : ByteRange) : Prop := ∃ i, a.Mem i ∧ b.Mem i

/-- The number of bytes two ranges share. -/
def overlapCount (a b : ByteRange) : Nat :=
  (if a.stop ≤ b.stop then a.stop else b.stop) -
    (if a.start ≤ b.start then b.start else a.start)

theorem overlapCount_pos_iff (a b : ByteRange) :
    0 < overlapCount a b ↔
      (a.start < a.stop ∧ b.start < b.stop ∧ a.start < b.stop ∧ b.start < a.stop) := by
  simp only [overlapCount]
  split <;> split <;> omega

/-- Decidable form of `Overlaps`. -/
def overlapsB (a b : ByteRange) : Bool := decide (0 < overlapCount a b)

theorem overlaps_iff (a b : ByteRange) : a.Overlaps b ↔ 0 < overlapCount a b := by
  rw [overlapCount_pos_iff]
  constructor
  · rintro ⟨i, ⟨h1, h2⟩, ⟨h3, h4⟩⟩
    exact ⟨by omega, by omega, by omega, by omega⟩
  · rintro ⟨h1, h2, h3, h4⟩
    refine ⟨if a.start ≤ b.start then b.start else a.start, ⟨?_, ?_⟩, ?_, ?_⟩ <;>
      (split <;> omega)

theorem overlapsB_iff (a b : ByteRange) : overlapsB a b = true ↔ a.Overlaps b := by
  simp [overlapsB, overlaps_iff]

instance (a b : ByteRange) : Decidable (a.Overlaps b) :=
  decidable_of_iff _ (overlapsB_iff a b)

/-- Two ranges share no byte. -/
def Disjoint (a b : ByteRange) : Prop := ¬ a.Overlaps b

instance (a b : ByteRange) : Decidable (a.Disjoint b) :=
  inferInstanceAs (Decidable (¬ _))

theorem disjoint_iff (a b : ByteRange) : a.Disjoint b ↔ overlapCount a b = 0 := by
  simp only [Disjoint, overlaps_iff]
  omega

theorem overlaps_comm (a b : ByteRange) : a.Overlaps b ↔ b.Overlaps a := by
  rw [overlaps_iff, overlaps_iff, overlapCount_pos_iff, overlapCount_pos_iff]
  constructor
  · rintro ⟨h1, h2, h3, h4⟩; exact ⟨h2, h1, h4, h3⟩
  · rintro ⟨h1, h2, h3, h4⟩; exact ⟨h2, h1, h4, h3⟩

theorem disjoint_comm (a b : ByteRange) : a.Disjoint b ↔ b.Disjoint a := by
  simp only [Disjoint, overlaps_comm]

theorem empty_disjoint (a b : ByteRange) (h : b.isEmpty = true) : a.Disjoint b := by
  simp only [isEmpty, beq_iff_eq] at h
  simp only [Disjoint, overlaps_iff, overlapCount_pos_iff, stop, h]
  omega

theorem overlaps_self (a : ByteRange) (h : a.isEmpty = false) : a.Overlaps a := by
  simp only [isEmpty, beq_eq_false_iff_ne, ne_eq] at h
  simp only [overlaps_iff, overlapCount_pos_iff, stop]
  omega

/-- The range declared by a view's interval. -/
def ofView (v : View) : ByteRange := ⟨v.offset, v.byteLength⟩

@[simp] theorem ofView_start (v : View) : (ofView v).start = v.offset := rfl
@[simp] theorem ofView_stop (v : View) : (ofView v).stop = v.offset + v.byteLength := rfl

end ByteRange

/-! ## Aliasing tags -/

/-- The three released aliasing tags (SPEC §8.3). -/
inductive AliasingTag
  | disjoint | cMayAliasAAfterRead | cMayAliasBAfterRead
  deriving DecidableEq, Repr, Inhabited

namespace AliasingTag

/-- ABI tag byte, exactly as printed in SPEC §8.3. -/
def tag : AliasingTag → Nat
  | disjoint => 0
  | cMayAliasAAfterRead => 1
  | cMayAliasBAfterRead => 2

def ofTag : Nat → Option AliasingTag
  | 0 => some disjoint
  | 1 => some cMayAliasAAfterRead
  | 2 => some cMayAliasBAfterRead
  | _ + 3 => none

def all : List AliasingTag := [disjoint, cMayAliasAAfterRead, cMayAliasBAfterRead]

theorem mem_all (a : AliasingTag) : a ∈ all := by cases a <;> decide
theorem all_nodup : all.Nodup := by decide
theorem all_map_tag : all.map tag = List.range 3 := by decide
theorem tag_lt (a : AliasingTag) : a.tag < 3 := by cases a <;> decide

theorem tag_injective : Function.Injective tag := by
  intro a b h; cases a <;> cases b <;> simp_all [tag]

@[simp] theorem ofTag_tag (a : AliasingTag) : ofTag a.tag = some a := by cases a <;> rfl

theorem tag_ofTag {n : Nat} {a : AliasingTag} (h : ofTag n = some a) : a.tag = n := by
  match n with
  | 0 | 1 | 2 => simp only [ofTag, Option.some.injEq] at h; subst h; rfl
  | _ + 3 => exact absurd h (by simp [ofTag])

theorem ofTag_eq_some_iff {n : Nat} {a : AliasingTag} : ofTag n = some a ↔ a.tag = n :=
  ⟨tag_ofTag, fun h => h ▸ ofTag_tag a⟩

theorem ofTag_eq_none_iff (n : Nat) : ofTag n = none ↔ 3 ≤ n := by
  match n with
  | 0 | 1 | 2 => simp [ofTag]
  | _ + 3 => simp [ofTag]

/-- The closed permission table: given whether C overlaps A and whether C
overlaps B, does the tag permit that configuration? -/
def permits : AliasingTag → Bool → Bool → Bool
  | disjoint, cA, cB => !cA && !cB
  | cMayAliasAAfterRead, _, cB => !cB
  | cMayAliasBAfterRead, cA, _ => !cA

/-- SPEC §8.3: "No tag permits C to overlap both A and B." -/
theorem no_tag_permits_both (t : AliasingTag) : t.permits true true = false := by
  cases t <;> rfl

/-- `disjoint` requires C to be disjoint from both. -/
theorem disjoint_permits_iff (cA cB : Bool) :
    disjoint.permits cA cB = true ↔ (cA = false ∧ cB = false) := by
  cases cA <;> cases cB <;> simp [permits]

/-- `cMayAliasAAfterRead` permits overlap only with A. -/
theorem aAfterRead_permits_iff (cA cB : Bool) :
    cMayAliasAAfterRead.permits cA cB = true ↔ cB = false := by
  cases cA <;> cases cB <;> simp [permits]

/-- `cMayAliasBAfterRead` is symmetric. -/
theorem bAfterRead_permits_iff (cA cB : Bool) :
    cMayAliasBAfterRead.permits cA cB = true ↔ cA = false := by
  cases cA <;> cases cB <;> simp [permits]

/-- Full disjointness is permitted by every tag. -/
theorem permits_disjoint (t : AliasingTag) : t.permits false false = true := by
  cases t <;> rfl

end AliasingTag

/-! ## The complete region contract -/

/-- The six ABI regions of one invocation. -/
structure Regions where
  header : ByteRange
  a : ByteRange
  b : ByteRange
  c : ByteRange
  scratch : ByteRange
  statusDetail : ByteRange
  deriving DecidableEq, Repr, Inhabited

namespace Regions

/-- SPEC §8.3: the header, scratch and status-detail ranges are pairwise
disjoint and each is disjoint from A, B and C. -/
def ControlRangesDisjoint (R : Regions) : Prop :=
  R.header.Disjoint R.scratch ∧ R.header.Disjoint R.statusDetail ∧
    R.scratch.Disjoint R.statusDetail ∧
    R.header.Disjoint R.a ∧ R.header.Disjoint R.b ∧ R.header.Disjoint R.c ∧
    R.scratch.Disjoint R.a ∧ R.scratch.Disjoint R.b ∧ R.scratch.Disjoint R.c ∧
    R.statusDetail.Disjoint R.a ∧ R.statusDetail.Disjoint R.b ∧
    R.statusDetail.Disjoint R.c

instance (R : Regions) : Decidable R.ControlRangesDisjoint :=
  inferInstanceAs (Decidable (_ ∧ _))

/-- The full alias contract of SPEC §8.3: the control ranges are pairwise
disjoint and disjoint from the matrices, and the C-overlap configuration is
permitted by the declared tag.  A and B may overlap each other freely. -/
def AliasOk (t : AliasingTag) (R : Regions) : Prop :=
  R.ControlRangesDisjoint ∧
    t.permits (R.c.overlapsB R.a) (R.c.overlapsB R.b) = true

instance (t : AliasingTag) (R : Regions) : Decidable (AliasOk t R) :=
  inferInstanceAs (Decidable (_ ∧ _))

/-- Whatever the tag, C never overlaps both A and B in a lawful invocation. -/
theorem not_c_overlaps_both {t : AliasingTag} {R : Regions} (h : AliasOk t R) :
    ¬ (R.c.Overlaps R.a ∧ R.c.Overlaps R.b) := by
  rintro ⟨h1, h2⟩
  obtain ⟨_, hp⟩ := h
  rw [← ByteRange.overlapsB_iff] at h1 h2
  rw [h1, h2, AliasingTag.no_tag_permits_both] at hp
  exact absurd hp (by simp)

/-- Under `disjoint`, C is disjoint from both matrices. -/
theorem disjoint_tag {R : Regions} (h : AliasOk .disjoint R) :
    R.c.Disjoint R.a ∧ R.c.Disjoint R.b := by
  obtain ⟨_, hp⟩ := h
  rw [AliasingTag.disjoint_permits_iff] at hp
  constructor
  · rw [ByteRange.Disjoint, ← ByteRange.overlapsB_iff, hp.1]; simp
  · rw [ByteRange.Disjoint, ← ByteRange.overlapsB_iff, hp.2]; simp

/-- Under `cMayAliasAAfterRead`, C is disjoint from B. -/
theorem aAfterRead_tag {R : Regions} (h : AliasOk .cMayAliasAAfterRead R) :
    R.c.Disjoint R.b := by
  obtain ⟨_, hp⟩ := h
  rw [AliasingTag.aAfterRead_permits_iff] at hp
  rw [ByteRange.Disjoint, ← ByteRange.overlapsB_iff, hp]; simp

/-- Under `cMayAliasBAfterRead`, C is disjoint from A. -/
theorem bAfterRead_tag {R : Regions} (h : AliasOk .cMayAliasBAfterRead R) :
    R.c.Disjoint R.a := by
  obtain ⟨_, hp⟩ := h
  rw [AliasingTag.bAfterRead_permits_iff] at hp
  rw [ByteRange.Disjoint, ← ByteRange.overlapsB_iff, hp]; simp

/-- Fully disjoint regions satisfy every tag. -/
theorem aliasOk_of_all_disjoint (t : AliasingTag) {R : Regions}
    (hc : R.ControlRangesDisjoint) (h1 : R.c.Disjoint R.a) (h2 : R.c.Disjoint R.b) :
    AliasOk t R := by
  refine ⟨hc, ?_⟩
  have e1 : R.c.overlapsB R.a = false := by
    rw [Bool.eq_false_iff, ne_eq, ByteRange.overlapsB_iff]; exact h1
  have e2 : R.c.overlapsB R.b = false := by
    rw [Bool.eq_false_iff, ne_eq, ByteRange.overlapsB_iff]; exact h2
  rw [e1, e2]
  exact AliasingTag.permits_disjoint t

/-- The overlapping byte count SPEC §8.3 reports for an alias failure: the first
offending pair in the fixed scan order, or `0` when the contract holds. -/
def aliasOverlapCount (t : AliasingTag) (R : Regions) : Nat :=
  if R.header.overlapsB R.scratch then ByteRange.overlapCount R.header R.scratch
  else if R.header.overlapsB R.statusDetail then
    ByteRange.overlapCount R.header R.statusDetail
  else if R.scratch.overlapsB R.statusDetail then
    ByteRange.overlapCount R.scratch R.statusDetail
  else if R.header.overlapsB R.a then ByteRange.overlapCount R.header R.a
  else if R.header.overlapsB R.b then ByteRange.overlapCount R.header R.b
  else if R.header.overlapsB R.c then ByteRange.overlapCount R.header R.c
  else if R.scratch.overlapsB R.a then ByteRange.overlapCount R.scratch R.a
  else if R.scratch.overlapsB R.b then ByteRange.overlapCount R.scratch R.b
  else if R.scratch.overlapsB R.c then ByteRange.overlapCount R.scratch R.c
  else if R.statusDetail.overlapsB R.a then ByteRange.overlapCount R.statusDetail R.a
  else if R.statusDetail.overlapsB R.b then ByteRange.overlapCount R.statusDetail R.b
  else if R.statusDetail.overlapsB R.c then ByteRange.overlapCount R.statusDetail R.c
  else if t.permits (R.c.overlapsB R.a) (R.c.overlapsB R.b) then 0
  else if R.c.overlapsB R.a then ByteRange.overlapCount R.c R.a
  else ByteRange.overlapCount R.c R.b

/-- A lawful configuration reports overlap count zero. -/
theorem aliasOverlapCount_eq_zero {t : AliasingTag} {R : Regions} (h : AliasOk t R) :
    aliasOverlapCount t R = 0 := by
  obtain ⟨hc, hp⟩ := h
  obtain ⟨d1, d2, d3, d4, d5, d6, d7, d8, d9, d10, d11, d12⟩ := hc
  simp only [ByteRange.Disjoint, ← ByteRange.overlapsB_iff,
    Bool.not_eq_true] at d1 d2 d3 d4 d5 d6 d7 d8 d9 d10 d11 d12
  simp only [aliasOverlapCount, d1, d2, d3, d4, d5, d6, d7, d8, d9, d10, d11, d12,
    hp, if_false, if_true, Bool.false_eq_true]

end Regions

end WasmGemmGnaf.Gemm

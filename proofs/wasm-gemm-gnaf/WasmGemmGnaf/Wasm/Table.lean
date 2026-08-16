/-
  Wasm/Table.lean --- table instances, element segments, and the table access
  semantics.

  Normative source: SPEC.md section 7.1.  Ownership there is exhaustive:
  "`Store` owns allocation and runtime instances", so *this* file owns table
  structure --- the shape of a table instance, its limits, its element
  segments, and the access relations of `table.get`, `table.set`, `table.size`,
  `table.grow`, `table.fill`, `table.copy`, `table.init` and `elem.drop`.  No
  allocation function is defined here; installing a table into a store is
  `Store`'s business, not this file's.

  Each operation is given twice:

  * as a **relation** (`GetRel`, `SetRel`, `GrowRel`, `FillRel`, `CopyRel`,
    `InitRel`), stated observationally --- what the resulting table looks like
    at every index --- and never in terms of the executable; and
  * as a **decidable executable** returning `Option`, where `none` is exactly
    the Core out-of-bounds trap.

  The two are then proved equal in extension (`get_eq_some_iff`,
  `set_eq_some_iff`, `grow_eq_some_iff`, `fill_eq_some_iff`, `copy_eq_some_iff`,
  `init_eq_some_iff`), which is what makes the executable evidence for the
  relation rather than a second, unrelated definition.  The bounds laws --- get
  after set at the same index, get elsewhere unchanged, out-of-bounds refused,
  growth within the declared limits --- are proved, not assumed.

  ## Declared scope, and what this file does NOT establish

  SPEC section 7.2 enables the "bulk memory, multiple memories, tables" family,
  so a table is a Core-valid and chargeable construct.  The *released validator*
  of `Wasm/Validate.lean` is nevertheless stricter than the family:

  * `Module.checkClosed` requires a validating module to declare no table and no
    element segment at all (`validate_tables_empty` below), and
  * `checkInstr` types no table instruction (`checkInstr_table_rejected`), while
    `Wasm/Step.lean` enumerates no successor for one
    (`successorsOfInstr_table_empty`).

  All three facts are machine-checked at the end of this file.  The table
  semantics below are therefore **unreachable** in a released execution: this
  file is a model of table structure and of the Core access laws, and it is
  *not* evidence that the released artifact executes table instructions.  No
  theorem here may be read that way.

  Every declaration in this file is proved.  Nothing is assumed.
-/
import WasmGemmGnaf.Wasm.Step

set_option autoImplicit false

namespace WasmGemmGnaf.Wasm

open WasmGemmGnaf.Foundation

/-! ## Splicing a run of elements into a list

`Wasm/Memory.lean` owns the byte-level kernel `setBytes`, which is specialised
to `List UInt8` and cannot be applied to a list of references.  The kernel below
is the general-list analogue used by `table.set`, `table.fill`, `table.copy` and
`table.init`; it is stated once, with its four index laws, and every table
operation is defined from it. -/

/-- Overwrite the `xs.length` elements of `l` starting at index `i`. -/
def spliceAt {α : Type} (l : List α) (i : Nat) (xs : List α) : List α :=
  l.take i ++ (xs ++ l.drop (i + xs.length))

theorem length_take_of_le {α : Type} {l : List α} {i : Nat} (h : i ≤ l.length) :
    (l.take i).length = i := by
  simp only [List.length_take]
  omega

/-- Splicing never changes the length of the list. -/
theorem spliceAt_length {α : Type} (l : List α) (i : Nat) (xs : List α)
    (h : i + xs.length ≤ l.length) : (spliceAt l i xs).length = l.length := by
  simp only [spliceAt, List.length_append, List.length_take, List.length_drop]
  omega

/-- An index before the spliced run is untouched. -/
theorem getElem?_spliceAt_lt {α : Type} {l : List α} {i j : Nat} {xs : List α}
    (hi : i ≤ l.length) (hj : j < i) : (spliceAt l i xs)[j]? = l[j]? := by
  rw [spliceAt, List.getElem?_append_left (by rw [length_take_of_le hi]; exact hj),
    List.getElem?_take]
  simp [hj]

/-- An index inside the spliced run holds the corresponding element of the
run. -/
theorem getElem?_spliceAt_mem {α : Type} {l : List α} {i j : Nat} {xs : List α}
    (hi : i ≤ l.length) (h1 : i ≤ j) (h2 : j < i + xs.length) :
    (spliceAt l i xs)[j]? = xs[j - i]? := by
  rw [spliceAt,
    List.getElem?_append_right (by rw [length_take_of_le hi]; exact h1),
    length_take_of_le hi, List.getElem?_append_left (by omega)]

/-- An index after the spliced run is untouched. -/
theorem getElem?_spliceAt_ge {α : Type} {l : List α} {i j : Nat} {xs : List α}
    (hi : i ≤ l.length) (h : i + xs.length ≤ j) :
    (spliceAt l i xs)[j]? = l[j]? := by
  rw [spliceAt,
    List.getElem?_append_right (by rw [length_take_of_le hi]; omega),
    length_take_of_le hi, List.getElem?_append_right (by omega),
    List.getElem?_drop]
  congr 1
  omega

/-! ## Reference values

A table element is a reference.  The released profile enables reference types,
typed function references and GC, so the element of a table is a null
reference, a function reference, or an external reference. -/

/-- A runtime reference value: the inhabitant of a table slot. -/
inductive Ref
  /-- The null reference of a heap type. -/
  | null (heapType : HeapType)
  /-- A reference to the function at the given index. -/
  | func (index : Nat)
  /-- An external reference, identified by its opaque index.  The released
  profile is closed, so no external reference is ever created by the machine;
  the constructor exists because the *type* `externref` is expressible. -/
  | extern (index : Nat)
  deriving DecidableEq, Repr, Inhabited

namespace Ref

/-- Whether the reference is null. -/
def isNull : Ref → Bool
  | .null _ => true
  | _ => false

@[simp] theorem isNull_null (ht : HeapType) : (Ref.null ht).isNull = true := rfl
@[simp] theorem isNull_func (i : Nat) : (Ref.func i).isNull = false := rfl
@[simp] theorem isNull_extern (i : Nat) : (Ref.extern i).isNull = false := rfl

/-- The function index of a function reference. -/
def funcIndex? : Ref → Option Nat
  | .func i => some i
  | _ => none

theorem funcIndex?_eq_some_iff {r : Ref} {i : Nat} :
    r.funcIndex? = some i ↔ r = .func i := by
  cases r <;> simp [funcIndex?]

/-- A reference is null, a function reference, or an external reference, and
these three cases are mutually exclusive. -/
theorem trichotomy (r : Ref) :
    (∃ ht, r = .null ht) ∨ (∃ i, r = .func i) ∨ (∃ i, r = .extern i) := by
  cases r with
  | null ht => exact Or.inl ⟨ht, rfl⟩
  | func i => exact Or.inr (Or.inl ⟨i, rfl⟩)
  | extern i => exact Or.inr (Or.inr ⟨i, rfl⟩)

end Ref

/-! ## Table types and their limits

`TableType` is owned by `Wasm/Types.lean`.  What belongs here is the Core
well-formedness condition on a table type: a released table is a wasm32 table
whose limits lie inside the normative element bound. -/

/-- The normative maximum number of elements of a wasm32 table: `2 ^ 32 - 1`. -/
def hardMaxTableElems : Nat := 4294967295

/-- Core well-formedness of a released table type: an `i32` address type, and
limits within the normative element bound. -/
def TableType.WellFormed (tt : TableType) : Prop :=
  tt.addressType = AddressType.i32 ∧ tt.limits.WithinBound hardMaxTableElems

instance TableType.instDecidableWellFormed (tt : TableType) :
    Decidable (TableType.WellFormed tt) := by
  unfold TableType.WellFormed
  exact inferInstance

/-- A `memory64`-style `i64` table type is expressible and is not well formed,
exactly as SPEC section 7.2 requires of a disabled form: it decodes and then
fails profile validation. -/
theorem TableType.i64_not_wellFormed {tt : TableType}
    (h : tt.addressType = AddressType.i64) : ¬ TableType.WellFormed tt := by
  intro hw
  have h1 : tt.addressType = AddressType.i32 := hw.1
  rw [h] at h1
  exact absurd h1 (by decide)

theorem TableType.i64_requiredFeature_rejected :
    Rejected (AddressType.requiredFeature .i64) := rfl

theorem TableType.i32_requiredFeature_enabled :
    Enabled (AddressType.requiredFeature .i32) := rfl

/-! ## Element segment instances -/

/-- A runtime element segment: its reference type, its elements, and whether it
has been dropped. -/
structure ElemInst where
  /-- The declared element reference type. -/
  type : RefType
  /-- The segment's references, in declaration order. -/
  elements : List Ref
  /-- Whether `elem.drop` has been executed on this segment. -/
  dropped : Bool
  deriving DecidableEq, Repr, Inhabited

namespace ElemInst

/-- The number of references the segment still holds. -/
def size (e : ElemInst) : Nat := e.elements.length

theorem size_eq (e : ElemInst) : e.size = e.elements.length := rfl

/-- `elem.drop`: the segment becomes empty and is marked dropped. -/
def dropSeg (e : ElemInst) : ElemInst :=
  { e with elements := [], dropped := true }

@[simp] theorem dropSeg_elements (e : ElemInst) : e.dropSeg.elements = [] := rfl
@[simp] theorem dropSeg_dropped (e : ElemInst) : e.dropSeg.dropped = true := rfl
@[simp] theorem dropSeg_type (e : ElemInst) : e.dropSeg.type = e.type := rfl
@[simp] theorem dropSeg_size (e : ElemInst) : e.dropSeg.size = 0 := rfl

/-- Dropping is idempotent. -/
theorem dropSeg_idem (e : ElemInst) : e.dropSeg.dropSeg = e.dropSeg := rfl

/-- Dropping changes nothing exactly when the segment was already empty and
already dropped. -/
theorem dropSeg_eq_self_iff (e : ElemInst) :
    e.dropSeg = e ↔ (e.elements = [] ∧ e.dropped = true) := by
  cases e with
  | mk ty els dr =>
    constructor
    · intro h
      injection h with _ h2 h3
      exact ⟨h2.symm, h3.symm⟩
    · rintro ⟨h1, h2⟩
      subst h1
      subst h2
      rfl

end ElemInst

/-! ## Table instances -/

/-- A runtime table instance: its declared type and its current elements. -/
structure TableInst where
  /-- The declared table type, carrying the limits. -/
  type : TableType
  /-- The current elements, in index order. -/
  elements : List Ref
  deriving DecidableEq, Repr, Inhabited

namespace TableInst

/-- `table.size`: the number of elements. -/
def size (t : TableInst) : Nat := t.elements.length

theorem size_eq (t : TableInst) : t.size = t.elements.length := rfl

/-- `table.get`. -/
def get (t : TableInst) (i : Nat) : Option Ref := t.elements[i]?

theorem get_eq (t : TableInst) (i : Nat) : t.get i = t.elements[i]? := rfl

/-- Two table instances with the same type and the same element at every index
are equal. -/
theorem ext {t t' : TableInst} (hty : t.type = t'.type)
    (h : ∀ (j : Nat), t.get j = t'.get j) : t = t' := by
  have h' : ∀ (j : Nat), t.elements[j]? = t'.elements[j]? := h
  have he : t.elements = t'.elements := List.ext_getElem? h'
  cases t
  cases t'
  cases hty
  cases he
  rfl

/-- The effective element limit: the declared maximum capped by the normative
wasm32 table bound. -/
def limitElems (t : TableInst) : Nat :=
  match t.type.limits.max with
  | none => hardMaxTableElems
  | some k => Nat.min k hardMaxTableElems

theorem limitElems_le_hard (t : TableInst) : t.limitElems ≤ hardMaxTableElems := by
  unfold limitElems
  cases t.type.limits.max with
  | none => exact Nat.le_refl _
  | some k => exact Nat.min_le_right _ _

/-- The effective limit depends only on the declared type. -/
theorem limitElems_of_type_eq {t t' : TableInst} (h : t.type = t'.type) :
    t.limitElems = t'.limitElems := by
  unfold limitElems
  rw [h]

/-- An access of `n` elements at index `i` is in bounds. -/
def InBounds (t : TableInst) (i n : Nat) : Prop := i + n ≤ t.size

instance instDecidableInBounds (t : TableInst) (i n : Nat) :
    Decidable (t.InBounds i n) := by
  unfold InBounds
  infer_instance

/-- A table instance is well formed when its size lies between its declared
minimum and its effective maximum. -/
def Wf (t : TableInst) : Prop :=
  t.type.limits.min ≤ t.size ∧ t.size ≤ t.limitElems

instance instDecidableWf (t : TableInst) : Decidable t.Wf := by
  unfold Wf
  infer_instance

/-! ### The executable operations

`none` is exactly the Core out-of-bounds trap. -/

/-- `table.set`. -/
def set (t : TableInst) (i : Nat) (r : Ref) : Option TableInst :=
  if i < t.size then some { t with elements := spliceAt t.elements i [r] }
  else none

/-- `table.grow`, filling the new slots with `r`. -/
def grow (t : TableInst) (delta : Nat) (r : Ref) : Option TableInst :=
  if t.size + delta ≤ t.limitElems then
    some { t with elements := t.elements ++ List.replicate delta r }
  else none

/-- `table.fill`. -/
def fill (t : TableInst) (i n : Nat) (r : Ref) : Option TableInst :=
  if i + n ≤ t.size then
    some { t with elements := spliceAt t.elements i (List.replicate n r) }
  else none

/-- `table.copy`. -/
def copy (dst src : TableInst) (d s n : Nat) : Option TableInst :=
  if d + n ≤ dst.size ∧ s + n ≤ src.size then
    some { dst with
      elements := spliceAt dst.elements d ((src.elements.drop s).take n) }
  else none

/-- `table.init` from an element segment instance. -/
def init (t : TableInst) (e : ElemInst) (d s n : Nat) : Option TableInst :=
  if d + n ≤ t.size ∧ s + n ≤ e.size then
    some { t with
      elements := spliceAt t.elements d ((e.elements.drop s).take n) }
  else none

/-! ### Out of bounds is exactly a refused access -/

theorem get_eq_none_iff (t : TableInst) (i : Nat) :
    t.get i = none ↔ t.size ≤ i := List.getElem?_eq_none_iff

theorem get_isSome_iff (t : TableInst) (i : Nat) :
    (t.get i).isSome ↔ i < t.size := by
  constructor
  · intro h
    rcases Nat.lt_or_ge i t.size with hlt | hge
    · exact hlt
    · rw [(get_eq_none_iff t i).mpr hge] at h
      exact absurd h (by simp)
  · intro h
    cases hg : t.get i with
    | none => exact absurd ((get_eq_none_iff t i).mp hg) (by omega)
    | some _ => rfl

/-- **`table.size` observed.**  The size is the unique index at which the table
stops answering: every smaller index has an element, and that index has none.
This is the observational content of `table.size`, stated without reference to
the underlying list. -/
theorem size_characterisation (t : TableInst) (n : Nat) :
    t.size = n ↔ ((∀ i, i < n → (t.get i).isSome) ∧ t.get n = none) := by
  constructor
  · rintro rfl
    exact ⟨fun i hi => (get_isSome_iff t i).mpr hi,
      (get_eq_none_iff t t.size).mpr (Nat.le_refl _)⟩
  · rintro ⟨hlt, hn⟩
    have hle : t.size ≤ n := (get_eq_none_iff t n).mp hn
    rcases Nat.lt_or_ge t.size n with hs | hs
    · have := (get_isSome_iff t t.size).mp (hlt t.size hs)
      exact absurd this (by omega)
    · omega

theorem set_eq_none_iff (t : TableInst) (i : Nat) (r : Ref) :
    t.set i r = none ↔ t.size ≤ i := by
  unfold set
  split <;> rename_i h <;> simp_all <;> omega

theorem grow_eq_none_iff (t : TableInst) (delta : Nat) (r : Ref) :
    t.grow delta r = none ↔ t.limitElems < t.size + delta := by
  unfold grow
  split <;> rename_i h <;> simp_all <;> omega

theorem fill_eq_none_iff (t : TableInst) (i n : Nat) (r : Ref) :
    t.fill i n r = none ↔ t.size < i + n := by
  unfold fill
  split <;> rename_i h <;> simp_all <;> omega

theorem copy_eq_none_iff (dst src : TableInst) (d s n : Nat) :
    copy dst src d s n = none ↔ (dst.size < d + n ∨ src.size < s + n) := by
  unfold copy
  split <;> rename_i h <;> simp_all <;> omega

theorem init_eq_none_iff (t : TableInst) (e : ElemInst) (d s n : Nat) :
    t.init e d s n = none ↔ (t.size < d + n ∨ e.size < s + n) := by
  unfold init
  split <;> rename_i h <;> simp_all <;> omega

/-- `table.init` from a dropped segment traps for every nonempty run: dropping
really does make the segment unusable. -/
theorem init_dropSeg_eq_none (t : TableInst) (e : ElemInst) (d s n : Nat)
    (hn : 0 < n) : t.init e.dropSeg d s n = none := by
  rw [init_eq_none_iff]
  right
  simp only [ElemInst.dropSeg_size]
  omega

/-! ### The guard of a successful operation -/

theorem set_lt_of_some {t t' : TableInst} {i : Nat} {r : Ref}
    (h : t.set i r = some t') : i < t.size := by
  unfold set at h
  split at h
  · assumption
  · exact absurd h (by simp)

theorem grow_bound_of_some {t t' : TableInst} {delta : Nat} {r : Ref}
    (h : t.grow delta r = some t') : t.size + delta ≤ t.limitElems := by
  unfold grow at h
  split at h
  · assumption
  · exact absurd h (by simp)

theorem fill_bound_of_some {t t' : TableInst} {i n : Nat} {r : Ref}
    (h : t.fill i n r = some t') : i + n ≤ t.size := by
  unfold fill at h
  split at h
  · assumption
  · exact absurd h (by simp)

theorem copy_bound_of_some {dst src t' : TableInst} {d s n : Nat}
    (h : copy dst src d s n = some t') :
    d + n ≤ dst.size ∧ s + n ≤ src.size := by
  unfold copy at h
  split at h
  · assumption
  · exact absurd h (by simp)

theorem init_bound_of_some {t t' : TableInst} {e : ElemInst} {d s n : Nat}
    (h : t.init e d s n = some t') : d + n ≤ t.size ∧ s + n ≤ e.size := by
  unfold init at h
  split at h
  · assumption
  · exact absurd h (by simp)

/-! ### The shape of a successful operation -/

theorem set_eq_of_lt {t : TableInst} {i : Nat} {r : Ref} (h : i < t.size) :
    t.set i r = some { t with elements := spliceAt t.elements i [r] } := by
  unfold set
  rw [if_pos h]

theorem grow_eq_of_le {t : TableInst} {delta : Nat} {r : Ref}
    (h : t.size + delta ≤ t.limitElems) :
    t.grow delta r =
      some { t with elements := t.elements ++ List.replicate delta r } := by
  unfold grow
  rw [if_pos h]

theorem fill_eq_of_le {t : TableInst} {i n : Nat} {r : Ref} (h : i + n ≤ t.size) :
    t.fill i n r =
      some { t with elements := spliceAt t.elements i (List.replicate n r) } := by
  unfold fill
  rw [if_pos h]

theorem copy_eq_of_le {dst src : TableInst} {d s n : Nat}
    (h1 : d + n ≤ dst.size) (h2 : s + n ≤ src.size) :
    copy dst src d s n =
      some { dst with
        elements := spliceAt dst.elements d ((src.elements.drop s).take n) } := by
  unfold copy
  rw [if_pos (⟨h1, h2⟩ : d + n ≤ dst.size ∧ s + n ≤ src.size)]

theorem init_eq_of_le {t : TableInst} {e : ElemInst} {d s n : Nat}
    (h1 : d + n ≤ t.size) (h2 : s + n ≤ e.size) :
    t.init e d s n =
      some { t with
        elements := spliceAt t.elements d ((e.elements.drop s).take n) } := by
  unfold init
  rw [if_pos (⟨h1, h2⟩ : d + n ≤ t.size ∧ s + n ≤ e.size)]

/-! ### Types and sizes -/

theorem set_type {t t' : TableInst} {i : Nat} {r : Ref} (h : t.set i r = some t') :
    t'.type = t.type := by
  rw [set_eq_of_lt (set_lt_of_some h)] at h
  injection h with h
  rw [← h]

theorem fill_type {t t' : TableInst} {i n : Nat} {r : Ref}
    (h : t.fill i n r = some t') : t'.type = t.type := by
  rw [fill_eq_of_le (fill_bound_of_some h)] at h
  injection h with h
  rw [← h]

theorem copy_type {dst src t' : TableInst} {d s n : Nat}
    (h : copy dst src d s n = some t') : t'.type = dst.type := by
  rw [copy_eq_of_le (copy_bound_of_some h).1 (copy_bound_of_some h).2] at h
  injection h with h
  rw [← h]

theorem init_type {t t' : TableInst} {e : ElemInst} {d s n : Nat}
    (h : t.init e d s n = some t') : t'.type = t.type := by
  rw [init_eq_of_le (init_bound_of_some h).1 (init_bound_of_some h).2] at h
  injection h with h
  rw [← h]

theorem grow_type {t t' : TableInst} {delta : Nat} {r : Ref}
    (h : t.grow delta r = some t') : t'.type = t.type := by
  rw [grow_eq_of_le (grow_bound_of_some h)] at h
  injection h with h
  rw [← h]

theorem set_size {t t' : TableInst} {i : Nat} {r : Ref} (h : t.set i r = some t') :
    t'.size = t.size := by
  have hi := set_lt_of_some h
  rw [set_eq_of_lt hi] at h
  injection h with h
  rw [← h]
  show (spliceAt t.elements i [r]).length = t.elements.length
  refine spliceAt_length _ _ _ ?_
  rw [size_eq] at hi
  simp only [List.length_cons, List.length_nil]
  omega

theorem fill_size {t t' : TableInst} {i n : Nat} {r : Ref}
    (h : t.fill i n r = some t') : t'.size = t.size := by
  have hb := fill_bound_of_some h
  rw [fill_eq_of_le hb] at h
  injection h with h
  rw [← h]
  show (spliceAt t.elements i (List.replicate n r)).length = t.elements.length
  refine spliceAt_length _ _ _ ?_
  rw [size_eq] at hb
  simp only [List.length_replicate]
  omega

theorem copy_size {dst src t' : TableInst} {d s n : Nat}
    (h : copy dst src d s n = some t') : t'.size = dst.size := by
  have hb := copy_bound_of_some h
  rw [copy_eq_of_le hb.1 hb.2] at h
  injection h with h
  rw [← h]
  show (spliceAt dst.elements d ((src.elements.drop s).take n)).length =
    dst.elements.length
  refine spliceAt_length _ _ _ ?_
  have h1 := hb.1
  have h2 := hb.2
  rw [size_eq] at h1 h2
  simp only [List.length_take, List.length_drop]
  omega

theorem init_size {t t' : TableInst} {e : ElemInst} {d s n : Nat}
    (h : t.init e d s n = some t') : t'.size = t.size := by
  have hb := init_bound_of_some h
  rw [init_eq_of_le hb.1 hb.2] at h
  injection h with h
  rw [← h]
  show (spliceAt t.elements d ((e.elements.drop s).take n)).length =
    t.elements.length
  refine spliceAt_length _ _ _ ?_
  have h1 := hb.1
  have h2 := hb.2
  rw [size_eq] at h1
  rw [ElemInst.size_eq] at h2
  simp only [List.length_take, List.length_drop]
  omega

theorem grow_size {t t' : TableInst} {delta : Nat} {r : Ref}
    (h : t.grow delta r = some t') : t'.size = t.size + delta := by
  rw [grow_eq_of_le (grow_bound_of_some h)] at h
  injection h with h
  rw [← h]
  show (t.elements ++ List.replicate delta r).length = t.elements.length + delta
  simp

/-! ### The access laws -/

/-- **Get after set at the same index returns exactly the element stored.** -/
theorem get_set_self {t t' : TableInst} {i : Nat} {r : Ref}
    (h : t.set i r = some t') : t'.get i = some r := by
  have hi := set_lt_of_some h
  rw [set_eq_of_lt hi] at h
  injection h with h
  rw [← h]
  rw [size_eq] at hi
  show (spliceAt t.elements i [r])[i]? = some r
  rw [getElem?_spliceAt_mem (l := t.elements) (i := i) (j := i) (xs := [r])
    (by omega) (Nat.le_refl _) (by simp only [List.length_cons, List.length_nil]; omega)]
  simp

/-- A get at any other index is unaffected by a set. -/
theorem get_set_ne {t t' : TableInst} {i j : Nat} {r : Ref}
    (h : t.set i r = some t') (hj : j ≠ i) : t'.get j = t.get j := by
  have hi := set_lt_of_some h
  rw [set_eq_of_lt hi] at h
  injection h with h
  rw [← h]
  rw [size_eq] at hi
  show (spliceAt t.elements i [r])[j]? = t.elements[j]?
  rcases Nat.lt_or_ge j i with hlt | hge
  · exact getElem?_spliceAt_lt (by omega) hlt
  · exact getElem?_spliceAt_ge (by omega)
      (by simp only [List.length_cons, List.length_nil]; omega)

/-- A filled slot holds the fill value. -/
theorem get_fill_mem {t t' : TableInst} {i n j : Nat} {r : Ref}
    (h : t.fill i n r = some t') (h1 : i ≤ j) (h2 : j < i + n) :
    t'.get j = some r := by
  have hb := fill_bound_of_some h
  rw [fill_eq_of_le hb] at h
  injection h with h
  rw [← h]
  rw [size_eq] at hb
  show (spliceAt t.elements i (List.replicate n r))[j]? = some r
  rw [getElem?_spliceAt_mem (by omega) h1
    (by simp only [List.length_replicate]; omega)]
  rw [List.getElem?_replicate, if_pos (show j - i < n by omega)]

/-- A slot outside the filled run is unaffected. -/
theorem get_fill_ne {t t' : TableInst} {i n j : Nat} {r : Ref}
    (h : t.fill i n r = some t') (hj : j < i ∨ i + n ≤ j) :
    t'.get j = t.get j := by
  have hb := fill_bound_of_some h
  rw [fill_eq_of_le hb] at h
  injection h with h
  rw [← h]
  rw [size_eq] at hb
  show (spliceAt t.elements i (List.replicate n r))[j]? = t.elements[j]?
  rcases hj with hlt | hge
  · exact getElem?_spliceAt_lt (by omega) hlt
  · exact getElem?_spliceAt_ge (by omega)
      (by simp only [List.length_replicate]; omega)

/-- A copied slot holds the source element. -/
theorem get_copy_mem {dst src t' : TableInst} {d s n k : Nat}
    (h : copy dst src d s n = some t') (hk : k < n) :
    t'.get (d + k) = src.get (s + k) := by
  have hb := copy_bound_of_some h
  rw [copy_eq_of_le hb.1 hb.2] at h
  injection h with h
  rw [← h]
  have h1 := hb.1
  have h2 := hb.2
  rw [size_eq] at h1 h2
  have hlen : ((src.elements.drop s).take n).length = n := by
    simp only [List.length_take, List.length_drop]
    omega
  show (spliceAt dst.elements d ((src.elements.drop s).take n))[d + k]? =
    src.elements[s + k]?
  rw [getElem?_spliceAt_mem (by omega) (by omega) (by rw [hlen]; omega),
    show d + k - d = k by omega, List.getElem?_take, if_pos hk,
    List.getElem?_drop]

/-- A slot outside the copied run is unaffected. -/
theorem get_copy_ne {dst src t' : TableInst} {d s n j : Nat}
    (h : copy dst src d s n = some t') (hj : j < d ∨ d + n ≤ j) :
    t'.get j = dst.get j := by
  have hb := copy_bound_of_some h
  rw [copy_eq_of_le hb.1 hb.2] at h
  injection h with h
  rw [← h]
  have h1 := hb.1
  have h2 := hb.2
  rw [size_eq] at h1 h2
  have hlen : ((src.elements.drop s).take n).length = n := by
    simp only [List.length_take, List.length_drop]
    omega
  show (spliceAt dst.elements d ((src.elements.drop s).take n))[j]? =
    dst.elements[j]?
  rcases hj with hlt | hge
  · exact getElem?_spliceAt_lt (by omega) hlt
  · exact getElem?_spliceAt_ge (by omega) (by rw [hlen]; omega)

/-- An initialised slot holds the segment element. -/
theorem get_init_mem {t t' : TableInst} {e : ElemInst} {d s n k : Nat}
    (h : t.init e d s n = some t') (hk : k < n) :
    t'.get (d + k) = e.elements[s + k]? := by
  have hb := init_bound_of_some h
  rw [init_eq_of_le hb.1 hb.2] at h
  injection h with h
  rw [← h]
  have h1 := hb.1
  have h2 := hb.2
  rw [size_eq] at h1
  rw [ElemInst.size_eq] at h2
  have hlen : ((e.elements.drop s).take n).length = n := by
    simp only [List.length_take, List.length_drop]
    omega
  show (spliceAt t.elements d ((e.elements.drop s).take n))[d + k]? =
    e.elements[s + k]?
  rw [getElem?_spliceAt_mem (by omega) (by omega) (by rw [hlen]; omega),
    show d + k - d = k by omega, List.getElem?_take, if_pos hk,
    List.getElem?_drop]

/-- A slot outside the initialised run is unaffected. -/
theorem get_init_ne {t t' : TableInst} {e : ElemInst} {d s n j : Nat}
    (h : t.init e d s n = some t') (hj : j < d ∨ d + n ≤ j) :
    t'.get j = t.get j := by
  have hb := init_bound_of_some h
  rw [init_eq_of_le hb.1 hb.2] at h
  injection h with h
  rw [← h]
  have h1 := hb.1
  have h2 := hb.2
  rw [size_eq] at h1
  rw [ElemInst.size_eq] at h2
  have hlen : ((e.elements.drop s).take n).length = n := by
    simp only [List.length_take, List.length_drop]
    omega
  show (spliceAt t.elements d ((e.elements.drop s).take n))[j]? = t.elements[j]?
  rcases hj with hlt | hge
  · exact getElem?_spliceAt_lt (by omega) hlt
  · exact getElem?_spliceAt_ge (by omega) (by rw [hlen]; omega)

/-- Growth preserves every existing element. -/
theorem get_grow_lt {t t' : TableInst} {delta j : Nat} {r : Ref}
    (h : t.grow delta r = some t') (hj : j < t.size) : t'.get j = t.get j := by
  rw [grow_eq_of_le (grow_bound_of_some h)] at h
  injection h with h
  rw [← h]
  rw [size_eq] at hj
  exact List.getElem?_append_left hj

/-- Every new slot holds the supplied initial reference. -/
theorem get_grow_ge {t t' : TableInst} {delta j : Nat} {r : Ref}
    (h : t.grow delta r = some t') (h1 : t.size ≤ j) (h2 : j < t.size + delta) :
    t'.get j = some r := by
  rw [grow_eq_of_le (grow_bound_of_some h)] at h
  injection h with h
  rw [← h]
  rw [size_eq] at h1 h2
  show (t.elements ++ List.replicate delta r)[j]? = some r
  rw [List.getElem?_append_right h1, List.getElem?_replicate,
    if_pos (show j - t.elements.length < delta by omega)]

/-- Beyond the grown size there is still nothing. -/
theorem get_grow_out {t t' : TableInst} {delta j : Nat} {r : Ref}
    (h : t.grow delta r = some t') (hj : t.size + delta ≤ j) :
    t'.get j = none := by
  rw [get_eq_none_iff, grow_size h]
  exact hj

/-! ### Growth respects the declared limits -/

theorem size_le_grow_size {t t' : TableInst} {delta : Nat} {r : Ref}
    (h : t.grow delta r = some t') : t.size ≤ t'.size := by
  rw [grow_size h]
  exact Nat.le_add_right _ _

theorem grow_limitElems {t t' : TableInst} {delta : Nat} {r : Ref}
    (h : t.grow delta r = some t') : t'.limitElems = t.limitElems :=
  limitElems_of_type_eq (grow_type h)

/-- **Growth never exceeds the declared maximum.** -/
theorem grow_size_le_limitElems {t t' : TableInst} {delta : Nat} {r : Ref}
    (h : t.grow delta r = some t') : t'.size ≤ t'.limitElems := by
  rw [grow_limitElems h, grow_size h]
  exact grow_bound_of_some h

/-- **Growth never exceeds the normative wasm32 table bound.** -/
theorem grow_size_le_hard {t t' : TableInst} {delta : Nat} {r : Ref}
    (h : t.grow delta r = some t') : t'.size ≤ hardMaxTableElems :=
  Nat.le_trans (grow_size_le_limitElems h) (limitElems_le_hard t')

/-- Growth preserves well-formedness. -/
theorem grow_wf {t t' : TableInst} {delta : Nat} {r : Ref} (hw : t.Wf)
    (h : t.grow delta r = some t') : t'.Wf := by
  refine ⟨?_, grow_size_le_limitElems h⟩
  rw [grow_type h, grow_size h]
  exact Nat.le_trans hw.1 (Nat.le_add_right _ _)

/-- `table.set` preserves well-formedness: it changes no size. -/
theorem set_wf {t t' : TableInst} {i : Nat} {r : Ref} (hw : t.Wf)
    (h : t.set i r = some t') : t'.Wf := by
  refine ⟨?_, ?_⟩
  · rw [set_type h, set_size h]; exact hw.1
  · rw [set_size h, limitElems_of_type_eq (set_type h)]; exact hw.2

/-- `table.fill` preserves well-formedness. -/
theorem fill_wf {t t' : TableInst} {i n : Nat} {r : Ref} (hw : t.Wf)
    (h : t.fill i n r = some t') : t'.Wf := by
  refine ⟨?_, ?_⟩
  · rw [fill_type h, fill_size h]; exact hw.1
  · rw [fill_size h, limitElems_of_type_eq (fill_type h)]; exact hw.2

/-- `table.init` preserves well-formedness. -/
theorem init_wf {t t' : TableInst} {e : ElemInst} {d s n : Nat} (hw : t.Wf)
    (h : t.init e d s n = some t') : t'.Wf := by
  refine ⟨?_, ?_⟩
  · rw [init_type h, init_size h]; exact hw.1
  · rw [init_size h, limitElems_of_type_eq (init_type h)]; exact hw.2

/-- `table.copy` preserves well-formedness of the destination. -/
theorem copy_wf {dst src t' : TableInst} {d s n : Nat} (hw : dst.Wf)
    (h : copy dst src d s n = some t') : t'.Wf := by
  refine ⟨?_, ?_⟩
  · rw [copy_type h, copy_size h]; exact hw.1
  · rw [copy_size h, limitElems_of_type_eq (copy_type h)]; exact hw.2

/-! ## The relations, and their agreement with the executables

Each relation says what the resulting table looks like *observationally*: its
type, the bounds condition, and the value of every index.  None of them
mentions `spliceAt` or any other implementation detail, so the agreement
theorems below are genuine content and not a restatement. -/

/-- `GetRel t i r`: index `i` of the table denotes `r`, stated by exhibiting
the decomposition of the element list around that index. -/
inductive GetRel (t : TableInst) (i : Nat) (r : Ref) : Prop
  | intro (pre post : List Ref) (hpre : pre.length = i)
      (h : t.elements = pre ++ r :: post)

/-- `SetRel t i r t'`: `t'` is `t` with index `i` set to `r`. -/
def SetRel (t : TableInst) (i : Nat) (r : Ref) (t' : TableInst) : Prop :=
  t'.type = t.type ∧ i < t.size ∧ t'.get i = some r ∧
    ∀ j, j ≠ i → t'.get j = t.get j

/-- `GrowRel t delta r t'`: `t'` is `t` grown by `delta` slots holding `r`. -/
def GrowRel (t : TableInst) (delta : Nat) (r : Ref) (t' : TableInst) : Prop :=
  t'.type = t.type ∧ t.size + delta ≤ t.limitElems ∧
    (∀ j, j < t.size → t'.get j = t.get j) ∧
    (∀ j, t.size ≤ j → j < t.size + delta → t'.get j = some r) ∧
    (∀ j, t.size + delta ≤ j → t'.get j = none)

/-- `FillRel t i n r t'`: `t'` is `t` with the run `[i, i+n)` set to `r`. -/
def FillRel (t : TableInst) (i n : Nat) (r : Ref) (t' : TableInst) : Prop :=
  t'.type = t.type ∧ i + n ≤ t.size ∧
    (∀ j, i ≤ j → j < i + n → t'.get j = some r) ∧
    (∀ j, (j < i ∨ i + n ≤ j) → t'.get j = t.get j)

/-- `CopyRel dst src d s n t'`: `t'` is `dst` with the run `[d, d+n)` taken
from `src` at `[s, s+n)`. -/
def CopyRel (dst src : TableInst) (d s n : Nat) (t' : TableInst) : Prop :=
  t'.type = dst.type ∧ d + n ≤ dst.size ∧ s + n ≤ src.size ∧
    (∀ k, k < n → t'.get (d + k) = src.get (s + k)) ∧
    (∀ j, (j < d ∨ d + n ≤ j) → t'.get j = dst.get j)

/-- `InitRel t e d s n t'`: `t'` is `t` with the run `[d, d+n)` taken from the
element segment `e` at `[s, s+n)`. -/
def InitRel (t : TableInst) (e : ElemInst) (d s n : Nat) (t' : TableInst) :
    Prop :=
  t'.type = t.type ∧ d + n ≤ t.size ∧ s + n ≤ e.size ∧
    (∀ k, k < n → t'.get (d + k) = e.elements[s + k]?) ∧
    (∀ j, (j < d ∨ d + n ≤ j) → t'.get j = t.get j)

/-- A helper for the three run-shaped relations: an index is inside the run or
outside it. -/
theorem run_cases (i n j : Nat) :
    (i ≤ j ∧ j < i + n) ∨ (j < i ∨ i + n ≤ j) := by
  rcases Nat.lt_or_ge j i with h | h
  · exact Or.inr (Or.inl h)
  · rcases Nat.lt_or_ge j (i + n) with h2 | h2
    · exact Or.inl ⟨h, h2⟩
    · exact Or.inr (Or.inr h2)

/-- **Agreement for `table.get`.** -/
theorem get_eq_some_iff (t : TableInst) (i : Nat) (r : Ref) :
    t.get i = some r ↔ GetRel t i r := by
  constructor
  · intro h
    rw [get_eq] at h
    have hi : i < t.elements.length := by
      rcases Nat.lt_or_ge i t.elements.length with hlt | hge
      · exact hlt
      · rw [List.getElem?_eq_none_iff.mpr hge] at h
        exact absurd h (by simp)
    have hget : t.elements[i] = r := by
      rw [List.getElem?_eq_getElem hi] at h
      exact Option.some.inj h
    refine ⟨t.elements.take i, t.elements.drop (i + 1),
      length_take_of_le (by omega), ?_⟩
    have hsplit : t.elements.take i ++ t.elements.drop i = t.elements :=
      List.take_append_drop i t.elements
    rw [List.drop_eq_getElem_cons hi, hget] at hsplit
    exact hsplit.symm
  · rintro ⟨pre, post, hpre, h⟩
    rw [get_eq, h, ← hpre, List.getElem?_append_right (Nat.le_refl _)]
    simp

/-- **Agreement for `table.set`.** -/
theorem set_eq_some_iff (t : TableInst) (i : Nat) (r : Ref) (t' : TableInst) :
    t.set i r = some t' ↔ SetRel t i r t' := by
  constructor
  · intro h
    exact ⟨set_type h, set_lt_of_some h, get_set_self h,
      fun j hj => get_set_ne h hj⟩
  · rintro ⟨hty, hi, hset, hne⟩
    have hsome := set_eq_of_lt (t := t) (r := r) hi
    rw [hsome]
    congr 1
    refine ext hty.symm (fun j => ?_)
    rcases Decidable.em (j = i) with rfl | hj
    · rw [hset]
      exact get_set_self hsome
    · rw [hne j hj]
      exact get_set_ne hsome hj

/-- **Agreement for `table.grow`.** -/
theorem grow_eq_some_iff (t : TableInst) (delta : Nat) (r : Ref)
    (t' : TableInst) : t.grow delta r = some t' ↔ GrowRel t delta r t' := by
  constructor
  · intro h
    exact ⟨grow_type h, grow_bound_of_some h, fun j hj => get_grow_lt h hj,
      fun j h1 h2 => get_grow_ge h h1 h2, fun j hj => get_grow_out h hj⟩
  · rintro ⟨hty, hlim, hlt, hmem, hout⟩
    have hsome := grow_eq_of_le (t := t) (r := r) hlim
    rw [hsome]
    congr 1
    refine ext hty.symm (fun j => ?_)
    rcases Nat.lt_or_ge j t.size with hj | hj
    · rw [hlt j hj]
      exact get_grow_lt hsome hj
    · rcases Nat.lt_or_ge j (t.size + delta) with hj2 | hj2
      · rw [hmem j hj hj2]
        exact get_grow_ge hsome hj hj2
      · rw [hout j hj2]
        exact get_grow_out hsome hj2

/-- **Agreement for `table.fill`.** -/
theorem fill_eq_some_iff (t : TableInst) (i n : Nat) (r : Ref) (t' : TableInst) :
    t.fill i n r = some t' ↔ FillRel t i n r t' := by
  constructor
  · intro h
    exact ⟨fill_type h, fill_bound_of_some h, fun j h1 h2 => get_fill_mem h h1 h2,
      fun j hj => get_fill_ne h hj⟩
  · rintro ⟨hty, hb, hmem, hne⟩
    have hsome := fill_eq_of_le (t := t) (r := r) hb
    rw [hsome]
    congr 1
    refine ext hty.symm (fun j => ?_)
    rcases run_cases i n j with ⟨h1, h2⟩ | hj
    · rw [hmem j h1 h2]
      exact get_fill_mem hsome h1 h2
    · rw [hne j hj]
      exact get_fill_ne hsome hj

/-- **Agreement for `table.copy`.** -/
theorem copy_eq_some_iff (dst src : TableInst) (d s n : Nat) (t' : TableInst) :
    copy dst src d s n = some t' ↔ CopyRel dst src d s n t' := by
  constructor
  · intro h
    exact ⟨copy_type h, (copy_bound_of_some h).1, (copy_bound_of_some h).2,
      fun k hk => get_copy_mem h hk, fun j hj => get_copy_ne h hj⟩
  · rintro ⟨hty, hd, hs, hmem, hne⟩
    have hsome := copy_eq_of_le (dst := dst) (src := src) hd hs
    rw [hsome]
    congr 1
    refine ext hty.symm (fun j => ?_)
    rcases run_cases d n j with ⟨h1, h2⟩ | hj
    · have hk : j = d + (j - d) := by omega
      rw [hk, hmem (j - d) (by omega)]
      exact get_copy_mem hsome (by omega)
    · rw [hne j hj]
      exact get_copy_ne hsome hj

/-- **Agreement for `table.init`.** -/
theorem init_eq_some_iff (t : TableInst) (e : ElemInst) (d s n : Nat)
    (t' : TableInst) : t.init e d s n = some t' ↔ InitRel t e d s n t' := by
  constructor
  · intro h
    exact ⟨init_type h, (init_bound_of_some h).1, (init_bound_of_some h).2,
      fun k hk => get_init_mem h hk, fun j hj => get_init_ne h hj⟩
  · rintro ⟨hty, hd, hs, hmem, hne⟩
    have hsome := init_eq_of_le (t := t) (e := e) hd hs
    rw [hsome]
    congr 1
    refine ext hty.symm (fun j => ?_)
    rcases run_cases d n j with ⟨h1, h2⟩ | hj
    · have hk : j = d + (j - d) := by omega
      rw [hk, hmem (j - d) (by omega)]
      exact get_init_mem hsome (by omega)
    · rw [hne j hj]
      exact get_init_ne hsome hj

/-! ### The relations are decidable

Each relation quantifies over all indices, so it is not decidable by structural
inspection; it becomes decidable through the agreement theorems above.  This is
the executable form SPEC section 7.1 requires beside every rule. -/

instance instDecidableGetRel (t : TableInst) (i : Nat) (r : Ref) :
    Decidable (GetRel t i r) :=
  decidable_of_iff _ (get_eq_some_iff t i r)

instance instDecidableSetRel (t : TableInst) (i : Nat) (r : Ref)
    (t' : TableInst) : Decidable (SetRel t i r t') :=
  decidable_of_iff _ (set_eq_some_iff t i r t')

instance instDecidableGrowRel (t : TableInst) (delta : Nat) (r : Ref)
    (t' : TableInst) : Decidable (GrowRel t delta r t') :=
  decidable_of_iff _ (grow_eq_some_iff t delta r t')

instance instDecidableFillRel (t : TableInst) (i n : Nat) (r : Ref)
    (t' : TableInst) : Decidable (FillRel t i n r t') :=
  decidable_of_iff _ (fill_eq_some_iff t i n r t')

instance instDecidableCopyRel (dst src : TableInst) (d s n : Nat)
    (t' : TableInst) : Decidable (CopyRel dst src d s n t') :=
  decidable_of_iff _ (copy_eq_some_iff dst src d s n t')

instance instDecidableInitRel (t : TableInst) (e : ElemInst) (d s n : Nat)
    (t' : TableInst) : Decidable (InitRel t e d s n t') :=
  decidable_of_iff _ (init_eq_some_iff t e d s n t')

/-! ### Each relation determines its result -/

theorem SetRel.functional {t t₁ t₂ : TableInst} {i : Nat} {r : Ref}
    (h₁ : SetRel t i r t₁) (h₂ : SetRel t i r t₂) : t₁ = t₂ := by
  have e₁ := (set_eq_some_iff t i r t₁).mpr h₁
  have e₂ := (set_eq_some_iff t i r t₂).mpr h₂
  rw [e₁] at e₂
  exact Option.some.inj e₂

theorem GrowRel.functional {t t₁ t₂ : TableInst} {delta : Nat} {r : Ref}
    (h₁ : GrowRel t delta r t₁) (h₂ : GrowRel t delta r t₂) : t₁ = t₂ := by
  have e₁ := (grow_eq_some_iff t delta r t₁).mpr h₁
  have e₂ := (grow_eq_some_iff t delta r t₂).mpr h₂
  rw [e₁] at e₂
  exact Option.some.inj e₂

theorem FillRel.functional {t t₁ t₂ : TableInst} {i n : Nat} {r : Ref}
    (h₁ : FillRel t i n r t₁) (h₂ : FillRel t i n r t₂) : t₁ = t₂ := by
  have e₁ := (fill_eq_some_iff t i n r t₁).mpr h₁
  have e₂ := (fill_eq_some_iff t i n r t₂).mpr h₂
  rw [e₁] at e₂
  exact Option.some.inj e₂

theorem CopyRel.functional {dst src t₁ t₂ : TableInst} {d s n : Nat}
    (h₁ : CopyRel dst src d s n t₁) (h₂ : CopyRel dst src d s n t₂) :
    t₁ = t₂ := by
  have e₁ := (copy_eq_some_iff dst src d s n t₁).mpr h₁
  have e₂ := (copy_eq_some_iff dst src d s n t₂).mpr h₂
  rw [e₁] at e₂
  exact Option.some.inj e₂

theorem InitRel.functional {t t₁ t₂ : TableInst} {e : ElemInst} {d s n : Nat}
    (h₁ : InitRel t e d s n t₁) (h₂ : InitRel t e d s n t₂) : t₁ = t₂ := by
  have e₁ := (init_eq_some_iff t e d s n t₁).mpr h₁
  have e₂ := (init_eq_some_iff t e d s n t₂).mpr h₂
  rw [e₁] at e₂
  exact Option.some.inj e₂

/-- **Out of bounds really traps**: no reference satisfies `GetRel` beyond the
table's size. -/
theorem not_getRel_of_size_le {t : TableInst} {i : Nat} {r : Ref}
    (h : t.size ≤ i) : ¬ GetRel t i r := by
  intro hg
  have hs := (get_eq_some_iff t i r).mpr hg
  rw [(get_eq_none_iff t i).mpr h] at hs
  exact absurd hs (by simp)

/-- **Growth beyond the declared limit really is refused**: no table satisfies
`GrowRel` past the effective maximum. -/
theorem not_growRel_of_limit {t t' : TableInst} {delta : Nat} {r : Ref}
    (h : t.limitElems < t.size + delta) : ¬ GrowRel t delta r t' := by
  intro hg
  exact absurd hg.2.1 (by omega)

end TableInst

/-! ## Declared scope (see the header)

The two theorems below are the machine-checked form of the scope statement in
this file's header.  They say what this module does **not** establish. -/

/-- The released validator accepts only modules that declare no table and no
element segment: `Module.checkClosed` is one of its conjuncts. -/
theorem validate_checkClosed {m : Module} (h : validate m = true) :
    Module.checkClosed m = true := by
  cases hc : Module.checkClosed m with
  | true => rfl
  | false =>
    rw [validate, hc] at h
    simp at h

/-- **A validating released module declares no table and no element segment.** -/
theorem validate_tables_empty {m : Module} (h : validate m = true) :
    m.tables = [] ∧ m.elems = [] := by
  have hc := validate_checkClosed h
  unfold Module.checkClosed at hc
  simp only [Bool.and_eq_true, List.isEmpty_iff] at hc
  obtain ⟨⟨⟨_, ht⟩, he⟩, _⟩ := hc
  exact ⟨ht, he⟩

/-- The released validator types no table instruction: a module containing one
fails validation, so the table semantics above are never reached by a released
execution. -/
theorem checkInstr_table_rejected (C : Ctx) (h : Nat) (n m : Nat) :
    checkInstr C h (.tableGet n) = none ∧
    checkInstr C h (.tableSet n) = none ∧
    checkInstr C h (.tableSize n) = none ∧
    checkInstr C h (.tableGrow n) = none ∧
    checkInstr C h (.tableFill n) = none ∧
    checkInstr C h (.tableCopy n m) = none ∧
    checkInstr C h (.tableInit n m) = none ∧
    checkInstr C h (.elemDrop n) = none :=
  ⟨rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl⟩

/-- The released reduction relation enumerates no successor for a table
instruction: `Wasm/Step.lean` owns every reduction rule and has none for this
family. -/
theorem successorsOfInstr_table_empty (c : Config) (rest : List Instr) (n m : Nat) :
    successorsOfInstr c (.tableGet n) rest = [] ∧
    successorsOfInstr c (.tableSet n) rest = [] ∧
    successorsOfInstr c (.tableSize n) rest = [] ∧
    successorsOfInstr c (.tableGrow n) rest = [] ∧
    successorsOfInstr c (.tableFill n) rest = [] ∧
    successorsOfInstr c (.tableCopy n m) rest = [] ∧
    successorsOfInstr c (.tableInit n m) rest = [] ∧
    successorsOfInstr c (.elemDrop n) rest = [] :=
  ⟨rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl⟩

end WasmGemmGnaf.Wasm

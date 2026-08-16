/-
  Wasm/Memory.lean --- linear memory as a byte store.

  Normative source: SPEC.md section 7.2 ("ordinary wasm32 memories subject to
  the normative 65,536-page limit; memory zero carries the ABI") and section
  7.4, which requires the observable store to be an exact byte-level object.

  A memory instance is a byte list together with its page limit.  Every access
  law that the reduction relation of `Wasm/Step.lean` depends on is proved
  here, not assumed:

  * a load of the bytes just stored at the same address returns exactly those
    bytes (`loadBytes_storeBytes_self`);
  * a load outside the written range is unaffected (`loadByte_storeBytes_lt`,
    `loadByte_storeBytes_ge`);
  * stores to disjoint ranges commute (`storeBytes_comm`);
  * an access whose range leaves the allocated bytes is refused, which is
    exactly the Core out-of-bounds trap (`storeBytes_eq_none_iff`,
    `loadBytes_eq_none_iff`).

  Every declaration in this file is proved.  Nothing is assumed.
-/
import WasmGemmGnaf.Wasm.Profile

set_option autoImplicit false

namespace WasmGemmGnaf.Wasm

open WasmGemmGnaf.Foundation

/-! ## Raw byte-list primitives

`setBytes` and `readBytes` are the untyped kernels of every memory access.
They are stated on plain byte lists so that their laws are independent of the
memory record. -/

/-- Write the bytes `vs` consecutively starting at index `base`.  Indices
outside the list are ignored, exactly as `List.set` ignores them; the memory
wrapper below refuses such an access before calling this. -/
def setBytes (d : List UInt8) (base : Nat) : List UInt8 → List UInt8
  | [] => d
  | v :: vs => setBytes (d.set base v) (base + 1) vs

/-- Read `n` consecutive bytes starting at index `a`. -/
def readBytes (d : List UInt8) (a : Nat) : Nat → List UInt8
  | 0 => []
  | n + 1 => (d[a]?.getD 0) :: readBytes d (a + 1) n

@[simp] theorem setBytes_nil (d : List UInt8) (base : Nat) :
    setBytes d base [] = d := rfl

@[simp] theorem setBytes_cons (d : List UInt8) (base : Nat) (v : UInt8)
    (vs : List UInt8) :
    setBytes d base (v :: vs) = setBytes (d.set base v) (base + 1) vs := rfl

@[simp] theorem readBytes_zero (d : List UInt8) (a : Nat) :
    readBytes d a 0 = [] := rfl

@[simp] theorem readBytes_succ (d : List UInt8) (a n : Nat) :
    readBytes d a (n + 1) = (d[a]?.getD 0) :: readBytes d (a + 1) n := rfl

@[simp] theorem readBytes_length (d : List UInt8) (a n : Nat) :
    (readBytes d a n).length = n := by
  induction n generalizing a with
  | zero => rfl
  | succ n ih => simp [ih]

/-- Writing never changes the number of allocated bytes. -/
@[simp] theorem setBytes_length (vs : List UInt8) :
    ∀ (d : List UInt8) (base : Nat), (setBytes d base vs).length = d.length := by
  induction vs with
  | nil => intro d base; rfl
  | cons v vs ih =>
    intro d base
    simp [ih]

/-- A byte before the written range is untouched. -/
theorem getElem?_setBytes_lt (vs : List UInt8) :
    ∀ (d : List UInt8) (base i : Nat), i < base → (setBytes d base vs)[i]? = d[i]? := by
  induction vs with
  | nil => intro d base i _; rfl
  | cons v vs ih =>
    intro d base i hi
    rw [setBytes_cons, ih _ _ _ (by omega), List.getElem?_set_ne (by omega)]

/-- A byte after the written range is untouched. -/
theorem getElem?_setBytes_ge (vs : List UInt8) :
    ∀ (d : List UInt8) (base i : Nat), base + vs.length ≤ i →
      (setBytes d base vs)[i]? = d[i]? := by
  induction vs with
  | nil => intro d base i _; rfl
  | cons v vs ih =>
    intro d base i hi
    simp only [List.length_cons] at hi
    rw [setBytes_cons, ih _ _ _ (by omega), List.getElem?_set_ne (by omega)]

/-- A byte inside the written range holds the byte written there. -/
theorem getElem?_setBytes_mem (vs : List UInt8) :
    ∀ (d : List UInt8) (base k : Nat), k < vs.length → base + vs.length ≤ d.length →
      (setBytes d base vs)[base + k]? = vs[k]? := by
  induction vs with
  | nil => intro _ _ _ hk _; simp at hk
  | cons v vs ih =>
    intro d base k hk hb
    simp only [List.length_cons] at hk hb
    match k with
    | 0 =>
      rw [Nat.add_zero, setBytes_cons, getElem?_setBytes_lt _ _ _ _ (by omega),
        List.getElem?_set_self (by omega)]
      simp
    | k + 1 =>
      have harith : base + (k + 1) = (base + 1) + k := by omega
      rw [harith, setBytes_cons,
        ih _ _ _ (by omega) (by simp only [List.length_set]; omega)]
      simp

/-- Reading back what was just written returns exactly the written bytes. -/
theorem readBytes_setBytes_self (vs : List UInt8) :
    ∀ (d : List UInt8) (base : Nat), base + vs.length ≤ d.length →
      readBytes (setBytes d base vs) base vs.length = vs := by
  induction vs with
  | nil => intro d base _; rfl
  | cons v vs ih =>
    intro d base hb
    simp only [List.length_cons] at hb
    rw [List.length_cons, readBytes_succ, setBytes_cons,
      getElem?_setBytes_lt _ _ _ _ (by omega),
      List.getElem?_set_self (by omega),
      ih _ _ (by simp only [List.length_set]; omega)]
    simp

/-- A read entirely before the written range is unaffected. -/
theorem readBytes_setBytes_lt (vs : List UInt8) (d : List UInt8) (base a : Nat) :
    ∀ n : Nat, a + n ≤ base → readBytes (setBytes d base vs) a n = readBytes d a n := by
  intro n
  induction n generalizing a with
  | zero => intro _; rfl
  | succ n ih =>
    intro h
    rw [readBytes_succ, readBytes_succ, getElem?_setBytes_lt _ _ _ _ (by omega),
      ih (a + 1) (by omega)]

/-- A read entirely after the written range is unaffected. -/
theorem readBytes_setBytes_ge (vs : List UInt8) (d : List UInt8) (base a : Nat) :
    ∀ n : Nat, base + vs.length ≤ a →
      readBytes (setBytes d base vs) a n = readBytes d a n := by
  intro n
  induction n generalizing a with
  | zero => intro _; rfl
  | succ n ih =>
    intro h
    rw [readBytes_succ, readBytes_succ, getElem?_setBytes_ge _ _ _ _ (by omega),
      ih (a + 1) (by omega)]

/-- A single-cell update outside a written range may be moved across it. -/
theorem setBytes_set_comm (vs : List UInt8) :
    ∀ (d : List UInt8) (base i : Nat) (x : UInt8),
      (i < base ∨ base + vs.length ≤ i) →
      setBytes (d.set i x) base vs = (setBytes d base vs).set i x := by
  induction vs with
  | nil => intro _ _ _ _ _; rfl
  | cons v vs ih =>
    intro d base i x hdisj
    simp only [List.length_cons] at hdisj
    rw [setBytes_cons, setBytes_cons, List.set_comm _ _ (show i ≠ base by omega),
      ih _ _ _ _ (by omega)]

/-- Writes to disjoint byte ranges commute. -/
theorem setBytes_comm (ws : List UInt8) :
    ∀ (d : List UInt8) (a b : Nat) (vs : List UInt8),
      (a + vs.length ≤ b ∨ b + ws.length ≤ a) →
      setBytes (setBytes d a vs) b ws = setBytes (setBytes d b ws) a vs := by
  induction ws with
  | nil => intro _ _ _ _ _; rfl
  | cons w ws ih =>
    intro d a b vs hdisj
    simp only [List.length_cons] at hdisj
    rw [setBytes_cons, ← setBytes_set_comm vs d a b w (by omega),
      ih _ _ _ _ (by omega), setBytes_cons]

/-! ## Memory instances -/

/-- A linear-memory instance: the allocated bytes together with the declared
page maximum.  The address model is wasm32, so the implicit maximum is the
normative 65,536-page limit. -/
structure Memory where
  /-- The allocated bytes. -/
  data : List UInt8
  /-- The declared page maximum, if any. -/
  maxPages : Option Nat
  deriving DecidableEq, Repr, Inhabited

namespace Memory

/-- The normative wasm32 page limit. -/
def hardMaxPages : Nat := 65536

/-- The number of allocated bytes. -/
def size (m : Memory) : Nat := m.data.length

/-- The number of allocated pages. -/
def pages (m : Memory) : Nat := m.size / pageSize

/-- The effective page limit: the declared maximum capped by the normative
wasm32 limit. -/
def limitPages (m : Memory) : Nat :=
  match m.maxPages with
  | none => hardMaxPages
  | some k => Nat.min k hardMaxPages

theorem limitPages_le_hard (m : Memory) : m.limitPages ≤ hardMaxPages := by
  unfold limitPages
  cases m.maxPages with
  | none => exact Nat.le_refl _
  | some k => exact Nat.min_le_right _ _

/-- An access of `n` bytes at address `a` is in bounds. -/
def InBounds (m : Memory) (a n : Nat) : Prop := a + n ≤ m.size

instance instDecidableInBounds (m : Memory) (a n : Nat) :
    Decidable (m.InBounds a n) := by
  unfold InBounds
  infer_instance

/-- Load `n` bytes at address `a`; `none` is the Core out-of-bounds trap. -/
def loadBytes (m : Memory) (a n : Nat) : Option (List UInt8) :=
  if a + n ≤ m.size then some (readBytes m.data a n) else none

/-- Store the bytes `vs` at address `a`; `none` is the Core out-of-bounds
trap. -/
def storeBytes (m : Memory) (a : Nat) (vs : List UInt8) : Option Memory :=
  if a + vs.length ≤ m.size then some { m with data := setBytes m.data a vs }
  else none

/-- Load one byte. -/
def loadByte (m : Memory) (a : Nat) : Option UInt8 :=
  m.data[a]?

/-- Store one byte. -/
def storeByte (m : Memory) (a : Nat) (v : UInt8) : Option Memory :=
  m.storeBytes a [v]

/-! ### Out-of-bounds is exactly a refused access -/

theorem storeBytes_eq_none_iff (m : Memory) (a : Nat) (vs : List UInt8) :
    m.storeBytes a vs = none ↔ ¬ m.InBounds a vs.length := by
  unfold storeBytes InBounds
  split <;> simp_all

theorem loadBytes_eq_none_iff (m : Memory) (a n : Nat) :
    m.loadBytes a n = none ↔ ¬ m.InBounds a n := by
  unfold loadBytes InBounds
  split <;> simp_all

theorem storeBytes_isSome_iff (m : Memory) (a : Nat) (vs : List UInt8) :
    (m.storeBytes a vs).isSome ↔ m.InBounds a vs.length := by
  unfold storeBytes InBounds
  split <;> simp_all

theorem loadBytes_isSome_iff (m : Memory) (a n : Nat) :
    (m.loadBytes a n).isSome ↔ m.InBounds a n := by
  unfold loadBytes InBounds
  split <;> simp_all

theorem storeBytes_of_inBounds {m : Memory} {a : Nat} {vs : List UInt8}
    (h : m.InBounds a vs.length) :
    m.storeBytes a vs = some { m with data := setBytes m.data a vs } := by
  have h' : a + vs.length ≤ m.size := h
  unfold storeBytes
  rw [if_pos h']

theorem loadBytes_of_inBounds {m : Memory} {a n : Nat} (h : m.InBounds a n) :
    m.loadBytes a n = some (readBytes m.data a n) := by
  have h' : a + n ≤ m.size := h
  unfold loadBytes
  rw [if_pos h']

/-- A store never changes the allocated size. -/
theorem storeBytes_size {m m' : Memory} {a : Nat} {vs : List UInt8}
    (h : m.storeBytes a vs = some m') : m'.size = m.size := by
  unfold storeBytes at h
  split at h
  · injection h with h; subst h; simp [size]
  · exact absurd h (by simp)

/-- A store never changes the declared page maximum. -/
theorem storeBytes_maxPages {m m' : Memory} {a : Nat} {vs : List UInt8}
    (h : m.storeBytes a vs = some m') : m'.maxPages = m.maxPages := by
  unfold storeBytes at h
  split at h
  · injection h with h; subst h; rfl
  · exact absurd h (by simp)

/-- A store never changes the page count. -/
theorem storeBytes_pages {m m' : Memory} {a : Nat} {vs : List UInt8}
    (h : m.storeBytes a vs = some m') : m'.pages = m.pages := by
  unfold pages
  rw [storeBytes_size h]

/-! ### The load/store laws -/

/-- **Load after store at the same address returns exactly the stored bytes.** -/
theorem loadBytes_storeBytes_self {m m' : Memory} {a : Nat} {vs : List UInt8}
    (h : m.storeBytes a vs = some m') :
    m'.loadBytes a vs.length = some vs := by
  have hb : m.InBounds a vs.length := by
    rw [← storeBytes_isSome_iff, h]; rfl
  rw [storeBytes_of_inBounds hb] at h
  injection h with h
  subst h
  rw [loadBytes_of_inBounds
    (show ({ data := setBytes m.data a vs, maxPages := m.maxPages } :
      Memory).InBounds a vs.length by simpa [InBounds, size] using hb)]
  rw [readBytes_setBytes_self vs m.data a hb]

/-- A single stored byte reads back. -/
theorem loadByte_storeByte_self {m m' : Memory} {a : Nat} {v : UInt8}
    (h : m.storeByte a v = some m') : m'.loadByte a = some v := by
  unfold storeByte at h
  have hb : m.InBounds a [v].length := by
    rw [← storeBytes_isSome_iff, h]; rfl
  rw [storeBytes_of_inBounds hb] at h
  injection h with h
  subst h
  unfold loadByte
  have hmem := getElem?_setBytes_mem [v] m.data a 0 (by simp) hb
  simpa using hmem

/-- A load strictly before a stored range is unaffected. -/
theorem loadBytes_storeBytes_lt {m m' : Memory} {a b n : Nat} {vs : List UInt8}
    (h : m.storeBytes a vs = some m') (hlt : b + n ≤ a) :
    m'.loadBytes b n = m.loadBytes b n := by
  have hb : m.InBounds a vs.length := by
    rw [← storeBytes_isSome_iff, h]; rfl
  rw [storeBytes_of_inBounds hb] at h
  injection h with h
  subst h
  unfold loadBytes size
  simp only [setBytes_length]
  rw [readBytes_setBytes_lt vs m.data a b n hlt]

/-- A load entirely after a stored range is unaffected. -/
theorem loadBytes_storeBytes_ge {m m' : Memory} {a b n : Nat} {vs : List UInt8}
    (h : m.storeBytes a vs = some m') (hge : a + vs.length ≤ b) :
    m'.loadBytes b n = m.loadBytes b n := by
  have hb : m.InBounds a vs.length := by
    rw [← storeBytes_isSome_iff, h]; rfl
  rw [storeBytes_of_inBounds hb] at h
  injection h with h
  subst h
  unfold loadBytes size
  simp only [setBytes_length]
  rw [readBytes_setBytes_ge vs m.data a b n hge]

/-- A single-byte load outside a stored range is unaffected. -/
theorem loadByte_storeBytes_lt {m m' : Memory} {a b : Nat} {vs : List UInt8}
    (h : m.storeBytes a vs = some m') (hlt : b < a) :
    m'.loadByte b = m.loadByte b := by
  have hb : m.InBounds a vs.length := by
    rw [← storeBytes_isSome_iff, h]; rfl
  rw [storeBytes_of_inBounds hb] at h
  injection h with h
  subst h
  exact getElem?_setBytes_lt vs m.data a b hlt

theorem loadByte_storeBytes_ge {m m' : Memory} {a b : Nat} {vs : List UInt8}
    (h : m.storeBytes a vs = some m') (hge : a + vs.length ≤ b) :
    m'.loadByte b = m.loadByte b := by
  have hb : m.InBounds a vs.length := by
    rw [← storeBytes_isSome_iff, h]; rfl
  rw [storeBytes_of_inBounds hb] at h
  injection h with h
  subst h
  exact getElem?_setBytes_ge vs m.data a b hge

/-- **Stores to disjoint ranges commute.** -/
theorem storeBytes_comm {m m₁ m₁₂ m₂ m₂₁ : Memory} {a b : Nat}
    {vs ws : List UInt8}
    (hdisj : a + vs.length ≤ b ∨ b + ws.length ≤ a)
    (h₁ : m.storeBytes a vs = some m₁)
    (h₁₂ : m₁.storeBytes b ws = some m₁₂)
    (h₂ : m.storeBytes b ws = some m₂)
    (h₂₁ : m₂.storeBytes a vs = some m₂₁) :
    m₁₂ = m₂₁ := by
  have ha : m.InBounds a vs.length := by
    rw [← storeBytes_isSome_iff, h₁]; rfl
  have hb : m.InBounds b ws.length := by
    rw [← storeBytes_isSome_iff, h₂]; rfl
  rw [storeBytes_of_inBounds ha] at h₁
  rw [storeBytes_of_inBounds hb] at h₂
  injection h₁ with h₁
  injection h₂ with h₂
  subst h₁
  subst h₂
  have ha' : ({ data := setBytes m.data b ws, maxPages := m.maxPages } :
      Memory).InBounds a vs.length := by
    simpa [InBounds, size] using ha
  have hb' : ({ data := setBytes m.data a vs, maxPages := m.maxPages } :
      Memory).InBounds b ws.length := by
    simpa [InBounds, size] using hb
  rw [storeBytes_of_inBounds hb'] at h₁₂
  rw [storeBytes_of_inBounds ha'] at h₂₁
  injection h₁₂ with h₁₂
  injection h₂₁ with h₂₁
  subst h₁₂
  subst h₂₁
  simp only [Memory.mk.injEq, and_true]
  exact setBytes_comm ws m.data a b vs hdisj

/-- Two stores to disjoint ranges are both defined if either order starts. -/
theorem storeBytes_comm_isSome {m m₁ m₂ : Memory} {a b : Nat}
    {vs ws : List UInt8}
    (h₁ : m.storeBytes a vs = some m₁)
    (h₂ : m.storeBytes b ws = some m₂) :
    (m₁.storeBytes b ws).isSome ∧ (m₂.storeBytes a vs).isSome := by
  have ha : m.InBounds a vs.length := by
    rw [← storeBytes_isSome_iff, h₁]; rfl
  have hb : m.InBounds b ws.length := by
    rw [← storeBytes_isSome_iff, h₂]; rfl
  constructor
  · rw [storeBytes_isSome_iff]
    unfold InBounds at *
    rw [storeBytes_size h₁]
    exact hb
  · rw [storeBytes_isSome_iff]
    unfold InBounds at *
    rw [storeBytes_size h₂]
    exact ha

/-! ### Growth -/

/-- Grow the memory by `delta` pages.  `none` is the refusal outcome. -/
def grow (m : Memory) (delta : Nat) : Option Memory :=
  if m.pages + delta ≤ m.limitPages then
    some { m with data := m.data ++ List.replicate (delta * pageSize) 0 }
  else none

theorem grow_eq_none_iff (m : Memory) (delta : Nat) :
    m.grow delta = none ↔ m.limitPages < m.pages + delta := by
  unfold grow
  split <;> rename_i h <;> simp_all <;> omega

theorem grow_size {m m' : Memory} {delta : Nat} (h : m.grow delta = some m') :
    m'.size = m.size + delta * pageSize := by
  unfold grow at h
  split at h
  · injection h with h; subst h; simp [size]
  · exact absurd h (by simp)

/-- Growth preserves every previously allocated byte. -/
theorem loadByte_grow {m m' : Memory} {delta a : Nat} (h : m.grow delta = some m')
    (ha : a < m.size) : m'.loadByte a = m.loadByte a := by
  unfold grow at h
  split at h
  · injection h with h
    subst h
    unfold loadByte
    exact List.getElem?_append_left ha
  · exact absurd h (by simp)

/-- Growth never shrinks the memory. -/
theorem size_le_grow_size {m m' : Memory} {delta : Nat} (h : m.grow delta = some m') :
    m.size ≤ m'.size := by
  rw [grow_size h]
  exact Nat.le_add_right _ _

/-- A fresh memory of `n` pages. -/
def alloc (pages : Nat) (maxPages : Option Nat) : Memory :=
  { data := List.replicate (pages * pageSize) 0, maxPages }

theorem alloc_size (p : Nat) (mx : Option Nat) : (alloc p mx).size = p * pageSize := by
  simp [alloc, size]

/-- Every byte of a fresh memory is zero. -/
theorem alloc_loadByte {p : Nat} {mx : Option Nat} {a : Nat}
    (h : a < p * pageSize) : (alloc p mx).loadByte a = some 0 := by
  unfold alloc loadByte
  rw [List.getElem?_eq_getElem (by simpa using h)]
  simp

end Memory
end WasmGemmGnaf.Wasm

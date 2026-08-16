/-
  Wasm/Store.lean --- allocation and runtime instances.

  Normative source: SPEC.md section 7.1 ("`Store` owns allocation and runtime
  instances") and section 7.4, which requires an exact `ObservableStore` and an
  `ObservableEffects` carrying "only externally visible non-memory effects
  admitted by the closed profile".

  The released profile is closed: no host functions, clocks, filesystem,
  network, or environment query.  `ObservableEffects` is therefore the empty
  record, and that emptiness is *proved* to be a subsingleton rather than
  asserted, so no later file can smuggle an unmodelled effect into semantic
  equality.

  This file also fixes the little-endian ABI byte encoding of an `i32` and
  proves it a retraction, which is the law the `i32.load`/`i32.store` rules of
  `Wasm/Step.lean` rely on.

  Every declaration in this file is proved.  Nothing is assumed.
-/
import WasmGemmGnaf.Wasm.Memory
import WasmGemmGnaf.Wasm.Validate

set_option autoImplicit false

namespace WasmGemmGnaf.Wasm

open WasmGemmGnaf.Foundation

/-! ## The ABI byte encoding of an `i32` -/

/-- The little-endian four-byte encoding of a 32-bit word. -/
def u32ToBytes (v : UInt32) : List UInt8 :=
  [ UInt8.ofNat (v.toNat % 256)
  , UInt8.ofNat (v.toNat / 256 % 256)
  , UInt8.ofNat (v.toNat / 65536 % 256)
  , UInt8.ofNat (v.toNat / 16777216 % 256) ]

/-- The 32-bit word denoted by four little-endian bytes.  A list of any other
length is not an `i32` image and denotes zero; the memory rules never call it
on such a list, because `Memory.loadBytes _ _ 4` returns exactly four bytes. -/
def bytesToU32 : List UInt8 → UInt32
  | [b0, b1, b2, b3] =>
      UInt32.ofNat (b0.toNat + 256 * b1.toNat + 65536 * b2.toNat +
        16777216 * b3.toNat)
  | _ => 0

@[simp] theorem u32ToBytes_length (v : UInt32) : (u32ToBytes v).length = 4 := rfl

/-- The ABI encoding is a retraction: decoding the encoding of a word returns
that word. -/
theorem bytesToU32_u32ToBytes (v : UInt32) : bytesToU32 (u32ToBytes v) = v := by
  have hv : v.toNat < 2 ^ 32 := UInt32.toNat_lt v
  simp only [u32ToBytes, bytesToU32, UInt8.toNat_ofNat', Nat.reducePow]
  have h1 : v.toNat % 256 % 256 = v.toNat % 256 := by omega
  have h2 : v.toNat / 256 % 256 % 256 = v.toNat / 256 % 256 := by omega
  have h3 : v.toNat / 65536 % 256 % 256 = v.toNat / 65536 % 256 := by omega
  have h4 : v.toNat / 16777216 % 256 % 256 = v.toNat / 16777216 % 256 := by omega
  rw [h1, h2, h3, h4]
  have hsum :
      v.toNat % 256 + 256 * (v.toNat / 256 % 256) +
        65536 * (v.toNat / 65536 % 256) + 16777216 * (v.toNat / 16777216 % 256)
        = v.toNat := by
    simp only [Nat.pow_succ] at hv
    omega
  rw [hsum]
  exact UInt32.toNat_inj.mp (by
    simp only [UInt32.toNat_ofNat', Nat.reducePow]
    omega)

/-- The ABI encoding is injective. -/
theorem u32ToBytes_injective : Function.Injective u32ToBytes := by
  intro a b h
  have := congrArg bytesToU32 h
  rwa [bytesToU32_u32ToBytes, bytesToU32_u32ToBytes] at this

/-- The wrapping reading of a Core `i32.const` immediate. -/
def wrapI32 (v : Int) : UInt32 := UInt32.ofNat (v % 4294967296).toNat

/-! ## Runtime instances -/

/-- A global instance: its mutability and its current `i32` value. -/
structure GlobalInst where
  /-- Whether the global may be assigned. -/
  mutable : Bool
  /-- The current value. -/
  value : UInt32
  deriving DecidableEq, Repr, Inhabited

/-- The runtime store of the released profile: memory zero and the globals.
The profile admits no host function, table, or external instance. -/
structure Store where
  /-- Memory zero, which carries the ABI. -/
  memory : Memory
  /-- The global instances, in declaration order. -/
  globals : List GlobalInst
  deriving DecidableEq, Repr, Inhabited

/-- The externally observable part of a store: the exact ABI-visible bytes of
memory zero. -/
structure ObservableStore where
  /-- The observable bytes. -/
  bytes : List UInt8
  deriving DecidableEq, Repr, Inhabited

/-- The externally visible non-memory effects admitted by the closed release
profile.  The profile has no host function, clock, filesystem, network, or
environment query, so this record has no fields. -/
structure ObservableEffects where
  deriving DecidableEq, Repr, Inhabited

namespace ObservableEffects

/-- The closed profile admits exactly one effect value: there is nothing
externally visible besides the store. -/
theorem eq_none (a b : ObservableEffects) : a = b := by
  cases a; cases b; rfl

/-- The canonical (only) effect value. -/
def none' : ObservableEffects := {}

theorem eq_none' (a : ObservableEffects) : a = none' := eq_none a none'

end ObservableEffects

namespace Store

/-- The observable projection of a store. -/
def observable (s : Store) : ObservableStore := { bytes := s.memory.data }

@[simp] theorem observable_bytes (s : Store) :
    s.observable.bytes = s.memory.data := rfl

/-- Two stores with the same memory bytes have the same observation. -/
theorem observable_eq_of_data {s t : Store} (h : s.memory.data = t.memory.data) :
    s.observable = t.observable := by
  simp [observable, h]

/-- Assigning a global never changes the observable store. -/
theorem observable_setGlobal (s : Store) (i : Nat) (g : GlobalInst) :
    ({ s with globals := s.globals.set i g } : Store).observable = s.observable :=
  rfl

/-- Load `n` bytes from memory zero. -/
def loadBytes (s : Store) (a n : Nat) : Option (List UInt8) := s.memory.loadBytes a n

/-- Store bytes into memory zero. -/
def storeBytes (s : Store) (a : Nat) (vs : List UInt8) : Option Store :=
  (s.memory.storeBytes a vs).map (fun m => { s with memory := m })

theorem storeBytes_eq_none_iff (s : Store) (a : Nat) (vs : List UInt8) :
    s.storeBytes a vs = none ↔ ¬ s.memory.InBounds a vs.length := by
  unfold storeBytes
  rw [← Memory.storeBytes_eq_none_iff]
  cases s.memory.storeBytes a vs <;> simp

/-- **Load after store at the same address returns exactly the stored bytes**,
lifted to the store. -/
theorem loadBytes_storeBytes_self {s s' : Store} {a : Nat} {vs : List UInt8}
    (h : s.storeBytes a vs = some s') : s'.loadBytes a vs.length = some vs := by
  unfold storeBytes at h
  cases hm : s.memory.storeBytes a vs with
  | none => rw [hm] at h; exact absurd h (by simp)
  | some m =>
    rw [hm] at h
    simp only [Option.map_some] at h
    injection h with h
    subst h
    exact Memory.loadBytes_storeBytes_self hm

/-- A load disjoint from a store is unaffected, lifted to the store. -/
theorem loadBytes_storeBytes_lt {s s' : Store} {a b n : Nat} {vs : List UInt8}
    (h : s.storeBytes a vs = some s') (hlt : b + n ≤ a) :
    s'.loadBytes b n = s.loadBytes b n := by
  unfold storeBytes at h
  cases hm : s.memory.storeBytes a vs with
  | none => rw [hm] at h; exact absurd h (by simp)
  | some m =>
    rw [hm] at h
    simp only [Option.map_some] at h
    injection h with h
    subst h
    exact Memory.loadBytes_storeBytes_lt hm hlt

theorem loadBytes_storeBytes_ge {s s' : Store} {a b n : Nat} {vs : List UInt8}
    (h : s.storeBytes a vs = some s') (hge : a + vs.length ≤ b) :
    s'.loadBytes b n = s.loadBytes b n := by
  unfold storeBytes at h
  cases hm : s.memory.storeBytes a vs with
  | none => rw [hm] at h; exact absurd h (by simp)
  | some m =>
    rw [hm] at h
    simp only [Option.map_some] at h
    injection h with h
    subst h
    exact Memory.loadBytes_storeBytes_ge hm hge

/-- **An `i32` written to memory reads back exactly.** -/
theorem load_store_i32 {s s' : Store} {a : Nat} {v : UInt32}
    (h : s.storeBytes a (u32ToBytes v) = some s') :
    (s'.loadBytes a 4).map bytesToU32 = some v := by
  have := loadBytes_storeBytes_self h
  rw [u32ToBytes_length] at this
  rw [this]
  simp [bytesToU32_u32ToBytes]

/-! ### Allocation -/

/-- The constant value of a Core constant initializer expression of the
declared subset. -/
def constValue? : Expr → Option UInt32
  | .cons (.i32Const v) .nil => some (wrapI32 v)
  | _ => none

/-- Allocate the global instances of a module, or fail if some initializer is
not a constant expression of the declared subset. -/
def allocGlobals : List Global → Option (List GlobalInst)
  | [] => some []
  | g :: gs =>
      match constValue? g.init, allocGlobals gs with
      | some v, some rest =>
          some ({ mutable := g.type.mutable, value := v } :: rest)
      | _, _ => none

/-- Allocate the store of a module: memory zero at its declared minimum size
and the globals at their constant initializers.  `none` is an initialization
fault. -/
def alloc (m : Module) : Option Store :=
  match m.mems with
  | [mem] =>
      (allocGlobals m.globals).map fun gs =>
        { memory := Memory.alloc mem.type.limits.min mem.type.limits.max
          globals := gs }
  | _ => none

theorem alloc_eq {m : Module} {mem : Mem} {gs : List GlobalInst}
    (hm : m.mems = [mem]) (hg : allocGlobals m.globals = some gs) :
    alloc m =
      some { memory := Memory.alloc mem.type.limits.min mem.type.limits.max
             globals := gs } := by
  simp [alloc, hm, hg]

/-- A successfully allocated store is exactly a fresh memory of the declared
minimum size together with the allocated globals. -/
theorem alloc_shape {m : Module} {s : Store} (h : alloc m = some s) :
    ∃ mem, m.mems = [mem] ∧
      s.memory = Memory.alloc mem.type.limits.min mem.type.limits.max := by
  unfold alloc at h
  split at h
  · next mem hmems =>
    cases hg : allocGlobals m.globals with
    | none => rw [hg] at h; exact absurd h (by simp)
    | some gs =>
      rw [hg] at h
      simp only [Option.map_some] at h
      injection h with h
      exact ⟨mem, hmems, by rw [← h]⟩
  · exact absurd h (by simp)

/-- A freshly allocated store has exactly the declared number of pages. -/
theorem alloc_memory_size {m : Module} {s : Store} {mem : Mem}
    (hm : m.mems = [mem]) (h : alloc m = some s) :
    s.memory.size = mem.type.limits.min * pageSize := by
  obtain ⟨mem', hm', hmem⟩ := alloc_shape h
  rw [hm] at hm'
  injection hm' with hm'
  subst hm'
  rw [hmem]
  exact Memory.alloc_size _ _

/-- Every byte of a freshly allocated store is zero. -/
theorem alloc_zero {m : Module} {s : Store} {a : Nat}
    (h : alloc m = some s) (ha : a < s.memory.size) :
    s.memory.loadByte a = some 0 := by
  obtain ⟨mem, _, hmem⟩ := alloc_shape h
  rw [hmem] at ha ⊢
  exact Memory.alloc_loadByte (by simpa [Memory.alloc_size] using ha)

end Store
end WasmGemmGnaf.Wasm

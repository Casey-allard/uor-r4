/-
  Wasm/Vector.lean --- the standard fixed-width 128-bit vector value and its
  lane views.

  Normative source: SPEC.md section 7.1 ("`Numeric` and `Vector` own set-valued
  numeric relations") and section 7.2, whose feature matrix **enables**
  "standard fixed 128-bit SIMD" and **rejects** "relaxed SIMD operations".  The
  released vector model is therefore the standard one: a concrete 16-byte
  carrier with the six standard lane shapes, and no relaxed operator.

  What is proved here:

  * `VecShape.lanes_mul_laneWidth`: for every one of the six shapes, lane count
    times lane width (in bytes) is exactly 16, and `lanes_mul_laneBits` gives
    the same statement in bits (128).  These are the facts the `vectorLaneOps`
    coordinate of `Cost/Vector.lean` is charged against --- they are what makes
    `s.lanes` a well-defined lane count for a shape rather than a free
    parameter --- together with `lanes_le_wholeVectorShuffleLanes`, which bounds
    every shape's lane count by the whole-vector unit of any lawful profile.
  * the lane round trips: `V128.extractLane_replaceLane_self` (reading back a
    written lane returns exactly the written value),
    `V128.replaceLane_extractLane_self` (writing back a read lane is the
    identity) and `V128.extractLane_replaceLane_ne` (lanes are independent).
  * `V128.extractLane_lt`: an extracted lane is always below `2 ^ laneBits`, so
    the lane view is a genuine width-respecting decomposition.
  * the standard lanewise combinators `splat`, `mapLanes` and `zipLanes`, each
    with its extraction law, and the concrete integer lane operations built
    from them.
  * a genuinely set-valued float case: `f32x4NanLaneResults` is proved
    permitted and proved **not** deterministic, so no evaluator choice of NaN
    payload can stand in for the relation (the same discipline as
    `Wasm/Numeric.lean`).

  Byte-level writing and reading reuse `setBytes`/`readBytes` of
  `Wasm/Memory.lean` together with their proved laws; nothing about byte stores
  is restated here.

  ## Declared scope

  This file models the standard fixed-width shapes only.  No relaxed-SIMD
  operator is expressible: `VecShape.requiredFeature` is proved to be the
  enabled `fixedWidthSimd128` family for every shape and is proved never to be
  the rejected `relaxedSimd` family.

  Every declaration in this file is proved.  Nothing is assumed.
-/
import WasmGemmGnaf.Wasm.Memory
import WasmGemmGnaf.Wasm.Numeric
import WasmGemmGnaf.Wasm.Syntax

set_option autoImplicit false

namespace WasmGemmGnaf.Wasm

open WasmGemmGnaf.Foundation

/-! ## The little-endian lane codec

A lane of `w` bytes holds a natural number below `2 ^ (8 * w)`, stored
little-endian.  The codec is proved to be a bijection between such numbers and
byte lists of length `w`. -/

/-- The little-endian `w`-byte image of a natural number. -/
def encodeLane : Nat → Nat → List UInt8
  | 0, _ => []
  | w + 1, x => UInt8.ofNat (x % 256) :: encodeLane w (x / 256)

/-- The natural number denoted by a little-endian byte list. -/
def decodeLane : List UInt8 → Nat
  | [] => 0
  | b :: bs => b.toNat + 256 * decodeLane bs

@[simp] theorem encodeLane_length : ∀ (w x : Nat), (encodeLane w x).length = w
  | 0, _ => rfl
  | w + 1, x => by
      show (encodeLane w (x / 256)).length + 1 = w + 1
      rw [encodeLane_length w (x / 256)]

@[simp] theorem decodeLane_nil : decodeLane [] = 0 := rfl

theorem pow_lane_succ (w : Nat) : 2 ^ (8 * (w + 1)) = 256 * 2 ^ (8 * w) := by
  have h8 : (2 : Nat) ^ 8 = 256 := rfl
  rw [Nat.mul_succ, Nat.pow_add, h8, Nat.mul_comm]

/-- A `w`-byte lane holds a number below `2 ^ (8 * w)`. -/
theorem decodeLane_lt : ∀ (bs : List UInt8) (w : Nat), bs.length = w →
    decodeLane bs < 2 ^ (8 * w)
  | [], w, h => by
      subst h
      simp [decodeLane]
  | b :: bs, w, h => by
      cases w with
      | zero => exact absurd h (by simp)
      | succ w =>
        have hlen : bs.length = w := by simpa using h
        have ih := decodeLane_lt bs w hlen
        have hb : b.toNat < 256 := by
          have := b.toNat_lt
          simp at this
          exact this
        rw [pow_lane_succ]
        show b.toNat + 256 * decodeLane bs < 256 * 2 ^ (8 * w)
        omega

/-- Decoding the encoding of an in-range number returns that number. -/
theorem decodeLane_encodeLane : ∀ (w x : Nat), x < 2 ^ (8 * w) →
    decodeLane (encodeLane w x) = x
  | 0, x, h => by
      have : x = 0 := by simpa using h
      simp [this, encodeLane]
  | w + 1, x, h => by
      rw [pow_lane_succ] at h
      have hdiv : x / 256 < 2 ^ (8 * w) := by omega
      have ih := decodeLane_encodeLane w (x / 256) hdiv
      show (UInt8.ofNat (x % 256)).toNat + 256 * decodeLane (encodeLane w (x / 256)) = x
      rw [ih]
      have hb : (UInt8.ofNat (x % 256)).toNat = x % 256 := by
        rw [UInt8.toNat_ofNat']
        simp
      rw [hb]
      omega

/-- Encoding the decoding of a `w`-byte list returns that list. -/
theorem encodeLane_decodeLane : ∀ (bs : List UInt8) (w : Nat), bs.length = w →
    encodeLane w (decodeLane bs) = bs
  | [], w, h => by
      subst h
      rfl
  | b :: bs, w, h => by
      cases w with
      | zero => exact absurd h (by simp)
      | succ w =>
        have hlen : bs.length = w := by simpa using h
        have ih := encodeLane_decodeLane bs w hlen
        have hb : b.toNat < 256 := by
          have := b.toNat_lt
          simpa using this
        have hmod : (b.toNat + 256 * decodeLane bs) % 256 = b.toNat := by omega
        have hdiv : (b.toNat + 256 * decodeLane bs) / 256 = decodeLane bs := by omega
        show UInt8.ofNat ((b.toNat + 256 * decodeLane bs) % 256) ::
            encodeLane w ((b.toNat + 256 * decodeLane bs) / 256) = b :: bs
        rw [hmod, hdiv, ih]
        simp

/-! ## Lane shapes

The six standard shapes of the enabled fixed-width SIMD family.  `VecShape`
itself is owned by `Wasm/Syntax.lean`; this section adds its metrics. -/

namespace VecShape

/-- The number of lanes of a shape. -/
def lanes : VecShape → Nat
  | .i8x16 => 16
  | .i16x8 => 8
  | .i32x4 => 4
  | .i64x2 => 2
  | .f32x4 => 4
  | .f64x2 => 2

/-- The width of one lane, in bytes. -/
def laneWidth : VecShape → Nat
  | .i8x16 => 1
  | .i16x8 => 2
  | .i32x4 => 4
  | .i64x2 => 8
  | .f32x4 => 4
  | .f64x2 => 8

/-- The width of one lane, in bits. -/
def laneBits (s : VecShape) : Nat := 8 * s.laneWidth

/-- Whether the lanes of a shape are floating point. -/
def isFloat : VecShape → Bool
  | .f32x4 => true
  | .f64x2 => true
  | _ => false

/-- **The 128-bit law.**  For every shape, lane count times lane width is
exactly the sixteen bytes of the carrier. -/
theorem lanes_mul_laneWidth (s : VecShape) : s.lanes * s.laneWidth = 16 := by
  cases s <;> rfl

/-- The same law in bits: every shape partitions exactly 128 bits. -/
theorem lanes_mul_laneBits (s : VecShape) : s.lanes * s.laneBits = 128 := by
  cases s <;> rfl

theorem lanes_pos (s : VecShape) : 0 < s.lanes := by cases s <;> decide

theorem laneWidth_pos (s : VecShape) : 0 < s.laneWidth := by cases s <;> decide

theorem laneBits_pos (s : VecShape) : 0 < s.laneBits := by cases s <;> decide

theorem lanes_le (s : VecShape) : s.lanes ≤ 16 := by cases s <;> decide

theorem laneWidth_le (s : VecShape) : s.laneWidth ≤ 16 := by cases s <;> decide

/-- The complete enumeration of the standard shapes. -/
def all : List VecShape := [.i8x16, .i16x8, .i32x4, .i64x2, .f32x4, .f64x2]

theorem mem_all (s : VecShape) : s ∈ all := by cases s <;> simp [all]

theorem all_nodup : all.Nodup := by decide

theorem all_length : all.length = 6 := rfl

instance instFintype : Fintype VecShape where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

/-- Every standard shape belongs to the enabled fixed-width SIMD family. -/
def requiredFeature (_ : VecShape) : FeatureFamily := .fixedWidthSimd128

/-- SPEC section 7.2: standard fixed 128-bit SIMD is enabled. -/
theorem requiredFeature_enabled (s : VecShape) : Enabled (requiredFeature s) := rfl

/-- SPEC section 7.2: relaxed SIMD is rejected, and no shape of this file
belongs to that family --- nothing here is a relaxed operator. -/
theorem requiredFeature_ne_relaxed (s : VecShape) :
    requiredFeature s ≠ FeatureFamily.relaxedSimd := by
  cases s <;> exact (by decide)

theorem relaxedSimd_rejected : Rejected FeatureFamily.relaxedSimd := rfl

end VecShape

/-! ## The 128-bit vector value

A `V128` is exactly sixteen bytes.  The length is a field of the structure, so
no value of the type can be the wrong width. -/

/-- The standard fixed-width 128-bit vector value: sixteen bytes. -/
structure V128 where
  /-- The sixteen bytes, in little-endian lane order. -/
  bytes : List UInt8
  /-- The carrier is exactly 128 bits wide. -/
  length_eq : bytes.length = 16

namespace V128

theorem ext {a b : V128} (h : a.bytes = b.bytes) : a = b := by
  cases a; cases b; cases h; rfl

theorem ext_iff {a b : V128} : a = b ↔ a.bytes = b.bytes :=
  ⟨fun h => by rw [h], ext⟩

instance instDecidableEq : DecidableEq V128 := fun a b =>
  if h : a.bytes = b.bytes then isTrue (ext h)
  else isFalse (fun hab => h (congrArg V128.bytes hab))

/-- The all-zero vector. -/
def zero : V128 := ⟨List.replicate 16 0, by simp⟩

instance instInhabited : Inhabited V128 := ⟨zero⟩

@[simp] theorem zero_bytes : zero.bytes = List.replicate 16 0 := rfl

/-! ### Lane views -/

/-- The bytes of lane `i` under shape `s`. -/
def laneBytes (v : V128) (s : VecShape) (i : Nat) : List UInt8 :=
  readBytes v.bytes (i * s.laneWidth) s.laneWidth

/-- The value held by lane `i` under shape `s`. -/
def extractLane (v : V128) (s : VecShape) (i : Nat) : Nat :=
  decodeLane (v.laneBytes s i)

/-- The vector obtained by writing `x` into lane `i` under shape `s`. -/
def replaceLane (v : V128) (s : VecShape) (i x : Nat) : V128 :=
  ⟨setBytes v.bytes (i * s.laneWidth) (encodeLane s.laneWidth x),
   by rw [setBytes_length]; exact v.length_eq⟩

@[simp] theorem laneBytes_length (v : V128) (s : VecShape) (i : Nat) :
    (v.laneBytes s i).length = s.laneWidth :=
  readBytes_length _ _ _

@[simp] theorem replaceLane_bytes (v : V128) (s : VecShape) (i x : Nat) :
    (v.replaceLane s i x).bytes =
      setBytes v.bytes (i * s.laneWidth) (encodeLane s.laneWidth x) := rfl

/-- A lane of shape `s` occupies bytes `[i * laneWidth, i * laneWidth +
laneWidth)` and that range is inside the sixteen bytes. -/
theorem lane_bound {s : VecShape} {i : Nat} (h : i < s.lanes) :
    i * s.laneWidth + s.laneWidth ≤ 16 := by
  cases s <;> simp only [VecShape.lanes, VecShape.laneWidth] at h ⊢ <;> omega

/-- Distinct lanes occupy disjoint byte ranges. -/
theorem lane_disjoint {s : VecShape} {i j : Nat} (h : j < i) :
    j * s.laneWidth + s.laneWidth ≤ i * s.laneWidth := by
  cases s <;> simp only [VecShape.laneWidth] <;> omega

/-- **Every extracted lane respects its width.** -/
theorem extractLane_lt (v : V128) (s : VecShape) (i : Nat) :
    v.extractLane s i < 2 ^ s.laneBits :=
  decodeLane_lt _ _ (laneBytes_length v s i)

/-- Writing back what a read produced is the identity on the byte store. -/
theorem setBytes_readBytes_self (d : List UInt8) :
    ∀ (n a : Nat), a + n ≤ d.length → setBytes d a (readBytes d a n) = d := by
  intro n
  induction n with
  | zero => intro a _; rfl
  | succ n ih =>
    intro a h
    have ha : a < d.length := by omega
    rw [readBytes_succ, setBytes_cons]
    have hset : d.set a (d[a]?.getD 0) = d := by
      rw [List.getElem?_eq_getElem ha]
      simp
    rw [hset]
    exact ih (a + 1) (by omega)

/-- **Lane round trip, read after write.**  Reading lane `i` of a vector whose
lane `i` was just set to `x` returns exactly `x`. -/
theorem extractLane_replaceLane_self {v : V128} {s : VecShape} {i x : Nat}
    (hi : i < s.lanes) (hx : x < 2 ^ s.laneBits) :
    (v.replaceLane s i x).extractLane s i = x := by
  have hb : i * s.laneWidth + (encodeLane s.laneWidth x).length ≤ v.bytes.length := by
    rw [encodeLane_length, v.length_eq]
    exact lane_bound hi
  have hread := readBytes_setBytes_self (encodeLane s.laneWidth x) v.bytes
    (i * s.laneWidth) hb
  rw [encodeLane_length] at hread
  show decodeLane (readBytes (setBytes v.bytes (i * s.laneWidth)
    (encodeLane s.laneWidth x)) (i * s.laneWidth) s.laneWidth) = x
  rw [hread]
  exact decodeLane_encodeLane _ _ hx

/-- **Lane round trip, write after read.**  Writing a lane back with the value
just read from it is the identity. -/
theorem replaceLane_extractLane_self {v : V128} {s : VecShape} {i : Nat}
    (hi : i < s.lanes) : v.replaceLane s i (v.extractLane s i) = v := by
  apply ext
  show setBytes v.bytes (i * s.laneWidth)
    (encodeLane s.laneWidth (decodeLane (readBytes v.bytes (i * s.laneWidth)
      s.laneWidth))) = v.bytes
  rw [encodeLane_decodeLane _ _ (readBytes_length v.bytes (i * s.laneWidth)
    s.laneWidth)]
  exact setBytes_readBytes_self v.bytes s.laneWidth (i * s.laneWidth)
    (by rw [v.length_eq]; exact lane_bound hi)

/-- **Lanes are independent.**  Writing lane `i` leaves every other lane of the
same shape unchanged. -/
theorem extractLane_replaceLane_ne {v : V128} {s : VecShape} {i j x : Nat}
    (hij : j ≠ i) :
    (v.replaceLane s i x).extractLane s j = v.extractLane s j := by
  show decodeLane (readBytes (setBytes v.bytes (i * s.laneWidth)
      (encodeLane s.laneWidth x)) (j * s.laneWidth) s.laneWidth) =
    decodeLane (readBytes v.bytes (j * s.laneWidth) s.laneWidth)
  rcases Nat.lt_or_ge j i with h | h
  · rw [readBytes_setBytes_lt (encodeLane s.laneWidth x) v.bytes
      (i * s.laneWidth) (j * s.laneWidth) s.laneWidth (lane_disjoint h)]
  · have hji : i < j := by omega
    rw [readBytes_setBytes_ge (encodeLane s.laneWidth x) v.bytes
      (i * s.laneWidth) (j * s.laneWidth) s.laneWidth
      (by rw [encodeLane_length]; exact lane_disjoint hji)]

/-- Two vectors agreeing on every lane of a shape agree on every byte, and are
therefore equal.  (Stated through the byte carrier: the lane view of a shape is
a partition of the carrier.) -/
theorem eq_of_bytes_eq {a b : V128} (h : a.bytes = b.bytes) : a = b := ext h

/-! ### Lanewise combinators

`splat`, `mapLanes` and `zipLanes` are the standard lane-parallel shapes of the
enabled SIMD family.  Each is defined by iterating `replaceLane` over the lane
indices, so its extraction law follows from the round-trip lemmas above rather
than from a fresh byte-level argument. -/

/-- Auxiliary: set lanes `0, …, n-1` to `x`. -/
def splatAux (s : VecShape) (x : Nat) : Nat → V128 → V128
  | 0, v => v
  | n + 1, v => (splatAux s x n v).replaceLane s n x

/-- `t.splat`: every lane holds `x`. -/
def splat (s : VecShape) (x : Nat) : V128 := splatAux s x s.lanes zero

theorem extractLane_splatAux {s : VecShape} {x : Nat} (hx : x < 2 ^ s.laneBits) :
    ∀ (n : Nat) (v : V128) (i : Nat), i < n → n ≤ s.lanes →
      (splatAux s x n v).extractLane s i = x := by
  intro n
  induction n with
  | zero => intro _ i hi _; exact absurd hi (by omega)
  | succ n ih =>
    intro v i hi hn
    rcases Nat.lt_or_ge i n with h | h
    · show ((splatAux s x n v).replaceLane s n x).extractLane s i = x
      rw [extractLane_replaceLane_ne (by omega : i ≠ n)]
      exact ih v i h (by omega)
    · have hin : i = n := by omega
      subst hin
      show ((splatAux s x i v).replaceLane s i x).extractLane s i = x
      exact extractLane_replaceLane_self (by omega) hx

/-- **`splat` law.**  Every lane of `splat s x` holds `x`. -/
theorem extractLane_splat {s : VecShape} {x i : Nat} (hx : x < 2 ^ s.laneBits)
    (hi : i < s.lanes) : (splat s x).extractLane s i = x :=
  extractLane_splatAux hx s.lanes zero i hi (Nat.le_refl _)

/-- Auxiliary: rewrite lanes `0, …, n-1` by `f` applied to the source lane. -/
def mapLanesAux (s : VecShape) (f : Nat → Nat) (src : V128) : Nat → V128 → V128
  | 0, v => v
  | n + 1, v =>
      (mapLanesAux s f src n v).replaceLane s n
        (f (src.extractLane s n) % 2 ^ s.laneBits)

/-- The lanewise unary operation `f`, truncated to the lane width. -/
def mapLanes (s : VecShape) (f : Nat → Nat) (src : V128) : V128 :=
  mapLanesAux s f src s.lanes src

theorem two_pow_pos (n : Nat) : 0 < 2 ^ n := Nat.two_pow_pos n

theorem extractLane_mapLanesAux {s : VecShape} {f : Nat → Nat} {src : V128} :
    ∀ (n : Nat) (v : V128) (i : Nat), i < n → n ≤ s.lanes →
      (mapLanesAux s f src n v).extractLane s i =
        f (src.extractLane s i) % 2 ^ s.laneBits := by
  intro n
  induction n with
  | zero => intro _ i hi _; exact absurd hi (by omega)
  | succ n ih =>
    intro v i hi hn
    rcases Nat.lt_or_ge i n with h | h
    · show ((mapLanesAux s f src n v).replaceLane s n _).extractLane s i = _
      rw [extractLane_replaceLane_ne (by omega : i ≠ n)]
      exact ih v i h (by omega)
    · have hin : i = n := by omega
      subst hin
      show ((mapLanesAux s f src i v).replaceLane s i _).extractLane s i = _
      exact extractLane_replaceLane_self (by omega)
        (Nat.mod_lt _ (two_pow_pos _))

/-- **Lanewise unary law.** -/
theorem extractLane_mapLanes {s : VecShape} {f : Nat → Nat} {src : V128} {i : Nat}
    (hi : i < s.lanes) :
    (mapLanes s f src).extractLane s i =
      f (src.extractLane s i) % 2 ^ s.laneBits :=
  extractLane_mapLanesAux s.lanes src i hi (Nat.le_refl _)

/-- Auxiliary: rewrite lanes `0, …, n-1` by `g` applied to both source lanes. -/
def zipLanesAux (s : VecShape) (g : Nat → Nat → Nat) (a b : V128) :
    Nat → V128 → V128
  | 0, v => v
  | n + 1, v =>
      (zipLanesAux s g a b n v).replaceLane s n
        (g (a.extractLane s n) (b.extractLane s n) % 2 ^ s.laneBits)

/-- The lanewise binary operation `g`, truncated to the lane width. -/
def zipLanes (s : VecShape) (g : Nat → Nat → Nat) (a b : V128) : V128 :=
  zipLanesAux s g a b s.lanes a

theorem extractLane_zipLanesAux {s : VecShape} {g : Nat → Nat → Nat} {a b : V128} :
    ∀ (n : Nat) (v : V128) (i : Nat), i < n → n ≤ s.lanes →
      (zipLanesAux s g a b n v).extractLane s i =
        g (a.extractLane s i) (b.extractLane s i) % 2 ^ s.laneBits := by
  intro n
  induction n with
  | zero => intro _ i hi _; exact absurd hi (by omega)
  | succ n ih =>
    intro v i hi hn
    rcases Nat.lt_or_ge i n with h | h
    · show ((zipLanesAux s g a b n v).replaceLane s n _).extractLane s i = _
      rw [extractLane_replaceLane_ne (by omega : i ≠ n)]
      exact ih v i h (by omega)
    · have hin : i = n := by omega
      subst hin
      show ((zipLanesAux s g a b i v).replaceLane s i _).extractLane s i = _
      exact extractLane_replaceLane_self (by omega)
        (Nat.mod_lt _ (two_pow_pos _))

/-- **Lanewise binary law.** -/
theorem extractLane_zipLanes {s : VecShape} {g : Nat → Nat → Nat} {a b : V128}
    {i : Nat} (hi : i < s.lanes) :
    (zipLanes s g a b).extractLane s i =
      g (a.extractLane s i) (b.extractLane s i) % 2 ^ s.laneBits :=
  extractLane_zipLanesAux s.lanes a i hi (Nat.le_refl _)

/-! ### The standard integer lane operations

Each is the corresponding wrapping operation on lane values; the truncation is
supplied once and for all by `zipLanes`. -/

/-- `t.add`. -/
def addLanes (s : VecShape) (a b : V128) : V128 := zipLanes s (· + ·) a b
/-- `t.mul`. -/
def mulLanes (s : VecShape) (a b : V128) : V128 := zipLanes s (· * ·) a b
/-- `v128.and`, read lanewise. -/
def andLanes (s : VecShape) (a b : V128) : V128 := zipLanes s (· &&& ·) a b
/-- `v128.or`, read lanewise. -/
def orLanes (s : VecShape) (a b : V128) : V128 := zipLanes s (· ||| ·) a b
/-- `v128.xor`, read lanewise. -/
def xorLanes (s : VecShape) (a b : V128) : V128 := zipLanes s (· ^^^ ·) a b

theorem extractLane_addLanes {s : VecShape} {a b : V128} {i : Nat}
    (hi : i < s.lanes) :
    (addLanes s a b).extractLane s i =
      (a.extractLane s i + b.extractLane s i) % 2 ^ s.laneBits :=
  extractLane_zipLanes hi

theorem extractLane_mulLanes {s : VecShape} {a b : V128} {i : Nat}
    (hi : i < s.lanes) :
    (mulLanes s a b).extractLane s i =
      (a.extractLane s i * b.extractLane s i) % 2 ^ s.laneBits :=
  extractLane_zipLanes hi

theorem extractLane_andLanes {s : VecShape} {a b : V128} {i : Nat}
    (hi : i < s.lanes) :
    (andLanes s a b).extractLane s i =
      (a.extractLane s i &&& b.extractLane s i) % 2 ^ s.laneBits :=
  extractLane_zipLanes hi

theorem extractLane_orLanes {s : VecShape} {a b : V128} {i : Nat}
    (hi : i < s.lanes) :
    (orLanes s a b).extractLane s i =
      (a.extractLane s i ||| b.extractLane s i) % 2 ^ s.laneBits :=
  extractLane_zipLanes hi

theorem extractLane_xorLanes {s : VecShape} {a b : V128} {i : Nat}
    (hi : i < s.lanes) :
    (xorLanes s a b).extractLane s i =
      (a.extractLane s i ^^^ b.extractLane s i) % 2 ^ s.laneBits :=
  extractLane_zipLanes hi

/-- Lanewise addition is commutative on every lane. -/
theorem extractLane_addLanes_comm {s : VecShape} {a b : V128} {i : Nat}
    (hi : i < s.lanes) :
    (addLanes s a b).extractLane s i = (addLanes s b a).extractLane s i := by
  rw [extractLane_addLanes hi, extractLane_addLanes hi, Nat.add_comm]

end V128

/-! ## Set-valued vector results (SPEC section 7.1)

`Numeric` and `Vector` jointly own the set-valued numeric relations.  The
integer lane operations of the enabled family are single valued, and that is
proved rather than assumed.  The floating-point NaN case is genuinely set
valued: the pinned semantics permits any NaN payload in the result lane, and
that relation is proved **not** deterministic, so no evaluator choice can be
substituted for it. -/

namespace Vec

open WasmGemmGnaf.Wasm.V128

/-- The permitted results of an integer lanewise binary operation: exactly one
vector. -/
def laneBinResults (s : VecShape) (g : Nat → Nat → Nat) (a b : V128) :
    Num.Results V128 :=
  Num.det (zipLanes s g a b)

theorem laneBinResults_deterministic (s : VecShape) (g : Nat → Nat → Nat)
    (a b : V128) : Num.Deterministic (laneBinResults s g a b) :=
  Num.det_deterministic _

theorem laneBinResults_permitted (s : VecShape) (g : Nat → Nat → Nat)
    (a b : V128) : Num.Permitted (laneBinResults s g a b) :=
  Num.det_permitted _

theorem mem_laneBinResults_iff (s : VecShape) (g : Nat → Nat → Nat)
    (a b w : V128) : laneBinResults s g a b w ↔ w = zipLanes s g a b := Iff.rfl

/-- The permitted results of an `f32x4` operation whose lane `i` is
mathematically NaN: every 32-bit NaN payload, exactly as in
`Num.nanResults32`. -/
def f32x4NanLaneResults (v : V128) (i : Nat) : Num.Results V128 :=
  fun w => ∃ bits : UInt32, Num.isNaN32 bits = true ∧
    w = v.replaceLane .f32x4 i bits.toNat

theorem f32x4NanLaneResults_permitted (v : V128) (i : Nat) :
    Num.Permitted (f32x4NanLaneResults v i) :=
  ⟨v.replaceLane .f32x4 i Num.canonicalNaN32.toNat,
   ⟨Num.canonicalNaN32, Num.canonicalNaN32_isNaN, rfl⟩⟩

/-- Every permitted result really does carry a NaN payload in the named lane. -/
theorem f32x4NanLaneResults_isNaN {v w : V128} {i : Nat}
    (hi : i < VecShape.f32x4.lanes) (h : f32x4NanLaneResults v i w) :
    Num.isNaN32 (UInt32.ofNat (w.extractLane .f32x4 i)) = true := by
  obtain ⟨bits, hbits, rfl⟩ := h
  rw [extractLane_replaceLane_self hi (by
    have := bits.toNat_lt
    simpa [VecShape.laneBits, VecShape.laneWidth] using this)]
  rwa [UInt32.ofNat_toNat]

/-- **The NaN lane relation is genuinely set valued.**  Two distinct canonical
NaN payloads are both permitted, so a single evaluator result cannot stand in
for the relation. -/
theorem f32x4NanLaneResults_not_deterministic (v : V128) {i : Nat}
    (hi : i < VecShape.f32x4.lanes) :
    ¬ Num.Deterministic (f32x4NanLaneResults v i) := by
  intro hdet
  have h₁ : f32x4NanLaneResults v i
      (v.replaceLane .f32x4 i Num.canonicalNaN32.toNat) :=
    ⟨Num.canonicalNaN32, Num.canonicalNaN32_isNaN, rfl⟩
  have h₂ : f32x4NanLaneResults v i
      (v.replaceLane .f32x4 i Num.negCanonicalNaN32.toNat) :=
    ⟨Num.negCanonicalNaN32, Num.negCanonicalNaN32_isNaN, rfl⟩
  have heq := Num.eq_of_deterministic hdet h₁ h₂
  have hlane : (v.replaceLane .f32x4 i Num.canonicalNaN32.toNat).extractLane .f32x4 i
      = (v.replaceLane .f32x4 i Num.negCanonicalNaN32.toNat).extractLane .f32x4 i := by
    rw [heq]
  rw [extractLane_replaceLane_self hi (by
        have := Num.canonicalNaN32.toNat_lt
        simpa [VecShape.laneBits, VecShape.laneWidth] using this),
      extractLane_replaceLane_self hi (by
        have := Num.negCanonicalNaN32.toNat_lt
        simpa [VecShape.laneBits, VecShape.laneWidth] using this)] at hlane
  exact absurd (UInt32.toNat_inj.mp hlane) (by decide)

/-! ## The cost tie-in (SPEC section 7.5)

`Cost/Vector.lean` charges a vector operation on the `vectorLaneOps`
coordinate.  The number of lanes charged for a shape is `VecShape.lanes`, and
the whole-vector shuffle unit of every lawful profile is exactly the sixteen
`i8x16` lanes. -/

theorem wholeVectorShuffleLanes_eq_i8x16_lanes (profile : Profile) :
    profile.costTableBody.wholeVectorShuffleLanes = VecShape.i8x16.lanes := by
  rw [Profile.wholeVectorShuffleLanes_eq]
  rfl

/-- Charging a shape by its lane count never exceeds the whole-vector unit. -/
theorem lanes_le_wholeVectorShuffleLanes (profile : Profile) (s : VecShape) :
    s.lanes ≤ profile.costTableBody.wholeVectorShuffleLanes := by
  rw [Profile.wholeVectorShuffleLanes_eq]
  exact VecShape.lanes_le s

end Vec

end WasmGemmGnaf.Wasm

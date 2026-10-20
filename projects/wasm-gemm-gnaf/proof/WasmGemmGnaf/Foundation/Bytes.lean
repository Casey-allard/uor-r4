set_option autoImplicit false

/-!
# Foundation: canonical byte encodings (SPEC §6.2)

SPEC §6.2 requires that canonical encoders be *prefix-free*, *versioned*, and
*injective*, and that the injectivity actually be proved rather than assumed.

The whole development happens on `List UInt8` (where structural induction is
available) and is transported to `ByteArray` through the proved bijection
`ByteArray.toList` / `Bytes.pack`.

The central notion is `Bytes.PrefixFree`: an encoder is prefix-free when the
encoded value *and* the trailing bytes are simultaneously determined by the
concatenation.  This is exactly the property that makes concatenation-based
composition (products, sums, lists) injective, and it is strictly stronger than
injectivity (`PrefixFree.injective`).
-/

namespace WasmGemmGnaf.Foundation
namespace Bytes

/-! ## `ByteArray` and `List UInt8` -/

/-- Auxiliary unfolding of the core `ByteArray.toList` accumulator loop.

`n` is a fuel bound on `b.size - i`; the statement is proved by induction on the
fuel because the loop index *increases*. -/
theorem toList_loop_eq (b : ByteArray) :
    ∀ (n i : Nat) (r : List UInt8), b.size - i ≤ n →
      ByteArray.toList.loop b i r = r.reverse ++ b.data.toList.drop i := by
  intro n
  induction n with
  | zero =>
    intro i r h
    rw [ByteArray.toList.loop]
    have hi : ¬ (i < b.size) := by omega
    simp only [hi, if_false]
    rw [List.drop_eq_nil_of_le (by simp; omega)]
    simp
  | succ n ih =>
    intro i r h
    rw [ByteArray.toList.loop]
    by_cases hi : i < b.size
    · rw [if_pos hi, ih (i + 1) _ (by omega), List.reverse_cons, List.append_assoc]
      congr 1
      have hl : i < b.data.toList.length := by simpa using hi
      rw [List.drop_eq_getElem_cons hl]
      have h' : i < b.data.size := by simpa using hi
      simp only [ByteArray.get!, List.singleton_append, List.cons.injEq, and_true]
      exact getElem!_pos b.data i h'
    · rw [if_neg hi]
      rw [List.drop_eq_nil_of_le (by simp; omega)]
      simp

/-- `ByteArray.toList` is the underlying array's list of bytes. -/
theorem byteArray_toList (b : ByteArray) : b.toList = b.data.toList := by
  have h := toList_loop_eq b b.size 0 [] (by omega)
  simpa [ByteArray.toList] using h

/-- Canonical packing of a byte list into a `ByteArray`. -/
def pack (l : List UInt8) : ByteArray := l.toByteArray

@[simp] theorem data_toList_pack (l : List UInt8) : (pack l).data.toList = l :=
  List.toList_data_toByteArray

@[simp] theorem toList_pack (l : List UInt8) : (pack l).toList = l := by
  rw [byteArray_toList]; exact data_toList_pack l

@[simp] theorem pack_data_toList (b : ByteArray) : pack b.data.toList = b := by
  apply ByteArray.ext
  apply Array.toList_inj.mp
  exact data_toList_pack _

@[simp] theorem pack_toList (b : ByteArray) : pack b.toList = b := by
  rw [byteArray_toList]; exact pack_data_toList b

theorem toList_injective : Function.Injective ByteArray.toList := by
  intro a b h
  have h' := congrArg pack h
  simpa using h'

theorem pack_injective : Function.Injective pack := by
  intro a b h
  have h' := congrArg ByteArray.toList h
  simpa using h'

@[simp] theorem size_pack (l : List UInt8) : (pack l).size = l.length := by
  simp [pack, ← ByteArray.size_data, ← Array.length_toList]

@[simp] theorem length_toList (b : ByteArray) : b.toList.length = b.size := by
  simp [byteArray_toList, ← ByteArray.size_data]

@[simp] theorem toList_append (a b : ByteArray) :
    (a ++ b).toList = a.toList ++ b.toList := by
  simp [byteArray_toList]

theorem pack_append (a b : List UInt8) : pack (a ++ b) = pack a ++ pack b := by
  apply toList_injective
  simp

instance : DecidableEq ByteArray := fun _ _ => inferInstance

/-! ## Prefix-free encoders -/

/-- An encoder is **prefix-free** (self-delimiting) when a concatenation
`f a ++ r` determines both `a` and the remainder `r`.

This is the property SPEC §6.2 demands of canonical encoders: it makes byte
concatenation a uniquely decodable composition. -/
def PrefixFree {α : Type} (f : α → List UInt8) : Prop :=
  ∀ (a b : α) (r s : List UInt8), f a ++ r = f b ++ s → a = b ∧ r = s

theorem PrefixFree.injective {α : Type} {f : α → List UInt8} (h : PrefixFree f) :
    Function.Injective f := by
  intro a b hab
  exact (h a b [] [] (by simp [hab])).1

/-- Precomposition with an injection preserves prefix-freeness. -/
theorem PrefixFree.comp {α β : Type} {f : β → List UInt8} {g : α → β}
    (hf : PrefixFree f) (hg : Function.Injective g) :
    PrefixFree (fun a => f (g a)) := by
  intro a b r s h
  obtain ⟨h1, h2⟩ := hf (g a) (g b) r s h
  exact ⟨hg h1, h2⟩

/-- A fixed-width injective encoder is prefix-free. -/
theorem prefixFree_of_constLength {α : Type} (f : α → List UInt8) (k : Nat)
    (hlen : ∀ a, (f a).length = k) (hinj : Function.Injective f) :
    PrefixFree f := by
  intro a b r s h
  obtain ⟨h1, h2⟩ := List.append_inj h (by rw [hlen, hlen])
  exact ⟨hinj h1, h2⟩

/-! ## Bytes of a natural number (base-128, self-delimiting) -/

/-- Canonical self-delimiting little-endian base-128 encoding of a natural
number.  The high bit of every byte but the last is set, so the encoding of one
number is never a prefix of the encoding of another. -/
def natBytes (n : Nat) : List UInt8 :=
  if _h : n < 128 then [UInt8.ofNat n]
  else UInt8.ofNat (n % 128 + 128) :: natBytes (n / 128)
termination_by n
decreasing_by omega

theorem natBytes_lt {n : Nat} (h : n < 128) : natBytes n = [UInt8.ofNat n] := by
  rw [natBytes]; simp [h]

theorem natBytes_ge {n : Nat} (h : ¬ n < 128) :
    natBytes n = UInt8.ofNat (n % 128 + 128) :: natBytes (n / 128) := by
  rw [natBytes]; simp [h]

/-- `UInt8.ofNat` identifies exactly the naturals congruent mod 256. -/
theorem uint8_ofNat_eq_iff {p q : Nat} :
    UInt8.ofNat p = UInt8.ofNat q ↔ p % 256 = q % 256 := by
  constructor
  · intro h
    have h' := congrArg UInt8.toNat h
    simpa [UInt8.toNat_ofNat'] using h'
  · intro h
    apply UInt8.toNat_inj.mp
    simpa [UInt8.toNat_ofNat'] using h

theorem natBytes_prefixFree : PrefixFree natBytes := by
  intro a
  induction a using Nat.strongRecOn with
  | _ a ih =>
    intro b r s h
    by_cases ha : a < 128 <;> by_cases hb : b < 128
    · rw [natBytes_lt ha, natBytes_lt hb] at h
      simp only [List.singleton_append, List.cons.injEq] at h
      obtain ⟨h1, h2⟩ := h
      rw [uint8_ofNat_eq_iff] at h1
      exact ⟨by omega, h2⟩
    · rw [natBytes_lt ha, natBytes_ge hb] at h
      simp only [List.cons_append, List.cons.injEq] at h
      obtain ⟨h1, _⟩ := h
      rw [uint8_ofNat_eq_iff] at h1
      omega
    · rw [natBytes_ge ha, natBytes_lt hb] at h
      simp only [List.cons_append, List.cons.injEq] at h
      obtain ⟨h1, _⟩ := h
      rw [uint8_ofNat_eq_iff] at h1
      omega
    · rw [natBytes_ge ha, natBytes_ge hb] at h
      simp only [List.cons_append, List.cons.injEq] at h
      obtain ⟨h1, h2⟩ := h
      rw [uint8_ofNat_eq_iff] at h1
      obtain ⟨h3, h4⟩ := ih (a / 128) (by omega) (b / 128) r s h2
      exact ⟨by omega, h4⟩

theorem natBytes_injective : Function.Injective natBytes :=
  natBytes_prefixFree.injective

/-! ## Fixed-width big-endian unsigned words -/

/-- Canonical one-byte encoding. -/
def u8Bytes (x : UInt8) : List UInt8 := [x]

theorem u8Bytes_injective : Function.Injective u8Bytes := by
  intro a b h
  simpa [u8Bytes] using h

theorem u8Bytes_prefixFree : PrefixFree u8Bytes :=
  prefixFree_of_constLength u8Bytes 1 (fun _ => rfl) u8Bytes_injective

/-- Canonical two-byte big-endian encoding. -/
def u16Bytes (x : UInt16) : List UInt8 :=
  [UInt8.ofNat (x.toNat / 256), UInt8.ofNat (x.toNat % 256)]

theorem u16Bytes_injective : Function.Injective u16Bytes := by
  intro a b h
  simp only [u16Bytes, List.cons.injEq, and_true] at h
  obtain ⟨h1, h2⟩ := h
  rw [uint8_ofNat_eq_iff] at h1
  rw [uint8_ofNat_eq_iff] at h2
  apply UInt16.toNat_inj.mp
  have ha : a.toNat < 65536 := by simpa using UInt16.toNat_lt a
  have hb : b.toNat < 65536 := by simpa using UInt16.toNat_lt b
  omega

theorem u16Bytes_prefixFree : PrefixFree u16Bytes :=
  prefixFree_of_constLength u16Bytes 2 (fun _ => rfl) u16Bytes_injective

/-- Canonical four-byte big-endian encoding. -/
def u32Bytes (x : UInt32) : List UInt8 :=
  [ UInt8.ofNat (x.toNat / 16777216 % 256)
  , UInt8.ofNat (x.toNat / 65536 % 256)
  , UInt8.ofNat (x.toNat / 256 % 256)
  , UInt8.ofNat (x.toNat % 256) ]

theorem u32Bytes_injective : Function.Injective u32Bytes := by
  intro a b h
  simp only [u32Bytes, List.cons.injEq, and_true] at h
  obtain ⟨h1, h2, h3, h4⟩ := h
  rw [uint8_ofNat_eq_iff] at h1 h2 h3 h4
  apply UInt32.toNat_inj.mp
  have ha : a.toNat < 4294967296 := by simpa using UInt32.toNat_lt a
  have hb : b.toNat < 4294967296 := by simpa using UInt32.toNat_lt b
  omega

theorem u32Bytes_prefixFree : PrefixFree u32Bytes :=
  prefixFree_of_constLength u32Bytes 4 (fun _ => rfl) u32Bytes_injective

/-- Canonical eight-byte big-endian encoding. -/
def u64Bytes (x : UInt64) : List UInt8 :=
  [ UInt8.ofNat (x.toNat / 72057594037927936 % 256)
  , UInt8.ofNat (x.toNat / 281474976710656 % 256)
  , UInt8.ofNat (x.toNat / 1099511627776 % 256)
  , UInt8.ofNat (x.toNat / 4294967296 % 256)
  , UInt8.ofNat (x.toNat / 16777216 % 256)
  , UInt8.ofNat (x.toNat / 65536 % 256)
  , UInt8.ofNat (x.toNat / 256 % 256)
  , UInt8.ofNat (x.toNat % 256) ]

set_option maxHeartbeats 2000000 in
theorem u64Bytes_injective : Function.Injective u64Bytes := by
  intro a b h
  simp only [u64Bytes, List.cons.injEq, and_true] at h
  obtain ⟨h1, h2, h3, h4, h5, h6, h7, h8⟩ := h
  rw [uint8_ofNat_eq_iff] at h1 h2 h3 h4 h5 h6 h7 h8
  apply UInt64.toNat_inj.mp
  have ha : a.toNat < 18446744073709551616 := by simpa using UInt64.toNat_lt a
  have hb : b.toNat < 18446744073709551616 := by simpa using UInt64.toNat_lt b
  omega

theorem u64Bytes_prefixFree : PrefixFree u64Bytes :=
  prefixFree_of_constLength u64Bytes 8 (fun _ => rfl) u64Bytes_injective

/-! ## Length-prefixed byte strings -/

/-- Length-prefixed (hence self-delimiting) encoding of a raw byte string. -/
def stringBytes (l : List UInt8) : List UInt8 := natBytes l.length ++ l

theorem stringBytes_prefixFree : PrefixFree stringBytes := by
  intro a b r s h
  simp only [stringBytes, List.append_assoc] at h
  obtain ⟨hlen, htail⟩ := natBytes_prefixFree _ _ _ _ h
  exact List.append_inj htail hlen

theorem stringBytes_injective : Function.Injective stringBytes :=
  stringBytes_prefixFree.injective

/-- Length-prefixed encoding of a `ByteArray`. -/
def byteArrayBytes (b : ByteArray) : List UInt8 := stringBytes b.toList

theorem byteArrayBytes_prefixFree : PrefixFree byteArrayBytes :=
  stringBytes_prefixFree.comp toList_injective

theorem byteArrayBytes_injective : Function.Injective byteArrayBytes :=
  byteArrayBytes_prefixFree.injective

/-! ## Structural combinators -/

/-- Concatenated encoding of a pair. -/
def pairBytes {α β : Type} (f : α → List UInt8) (g : β → List UInt8)
    (p : α × β) : List UInt8 :=
  f p.1 ++ g p.2

theorem pairBytes_prefixFree {α β : Type} {f : α → List UInt8} {g : β → List UInt8}
    (hf : PrefixFree f) (hg : PrefixFree g) : PrefixFree (pairBytes f g) := by
  intro a b r s h
  simp only [pairBytes, List.append_assoc] at h
  obtain ⟨h1, h2⟩ := hf _ _ _ _ h
  obtain ⟨h3, h4⟩ := hg _ _ _ _ h2
  refine ⟨?_, h4⟩
  cases a; cases b
  simp_all

theorem pairBytes_injective {α β : Type} {f : α → List UInt8} {g : β → List UInt8}
    (hf : PrefixFree f) (hg : PrefixFree g) :
    Function.Injective (pairBytes f g) :=
  (pairBytes_prefixFree hf hg).injective

/-- Tagged encoding of a sum. -/
def sumBytes {α β : Type} (f : α → List UInt8) (g : β → List UInt8) :
    α ⊕ β → List UInt8
  | .inl a => 0 :: f a
  | .inr b => 1 :: g b

theorem sumBytes_prefixFree {α β : Type} {f : α → List UInt8} {g : β → List UInt8}
    (hf : PrefixFree f) (hg : PrefixFree g) : PrefixFree (sumBytes f g) := by
  intro a b r s h
  cases a with
  | inl a =>
    cases b with
    | inl b =>
      simp only [sumBytes, List.cons_append, List.cons.injEq, true_and] at h
      obtain ⟨h1, h2⟩ := hf _ _ _ _ h
      exact ⟨by rw [h1], h2⟩
    | inr b =>
      simp only [sumBytes, List.cons_append, List.cons.injEq] at h
      exact absurd h.1 (by decide)
  | inr a =>
    cases b with
    | inl b =>
      simp only [sumBytes, List.cons_append, List.cons.injEq] at h
      exact absurd h.1 (by decide)
    | inr b =>
      simp only [sumBytes, List.cons_append, List.cons.injEq, true_and] at h
      obtain ⟨h1, h2⟩ := hg _ _ _ _ h
      exact ⟨by rw [h1], h2⟩

theorem sumBytes_injective {α β : Type} {f : α → List UInt8} {g : β → List UInt8}
    (hf : PrefixFree f) (hg : PrefixFree g) :
    Function.Injective (sumBytes f g) :=
  (sumBytes_prefixFree hf hg).injective

/-- Concatenation of the element encodings of a list. -/
def concatBytes {α : Type} (f : α → List UInt8) : List α → List UInt8
  | [] => []
  | x :: xs => f x ++ concatBytes f xs

theorem concatBytes_eq_of_length {α : Type} {f : α → List UInt8}
    (hf : PrefixFree f) :
    ∀ (xs ys : List α) (r s : List UInt8), xs.length = ys.length →
      concatBytes f xs ++ r = concatBytes f ys ++ s → xs = ys ∧ r = s := by
  intro xs
  induction xs with
  | nil =>
    intro ys r s hlen h
    cases ys with
    | nil => simpa [concatBytes] using h
    | cons y ys => simp at hlen
  | cons x xs ih =>
    intro ys r s hlen h
    cases ys with
    | nil => simp at hlen
    | cons y ys =>
      simp only [concatBytes, List.append_assoc] at h
      obtain ⟨hxy, h'⟩ := hf _ _ _ _ h
      obtain ⟨h1, h2⟩ := ih ys r s (by simpa using hlen) h'
      exact ⟨by rw [hxy, h1], h2⟩

/-- Length-prefixed encoding of a list. -/
def listBytes {α : Type} (f : α → List UInt8) (xs : List α) : List UInt8 :=
  natBytes xs.length ++ concatBytes f xs

theorem listBytes_prefixFree {α : Type} {f : α → List UInt8} (hf : PrefixFree f) :
    PrefixFree (listBytes f) := by
  intro a b r s h
  simp only [listBytes, List.append_assoc] at h
  obtain ⟨hlen, h'⟩ := natBytes_prefixFree _ _ _ _ h
  exact concatBytes_eq_of_length hf a b r s hlen h'

theorem listBytes_injective {α : Type} {f : α → List UInt8} (hf : PrefixFree f) :
    Function.Injective (listBytes f) :=
  (listBytes_prefixFree hf).injective

/-- Optional value, tagged. -/
def optionBytes {α : Type} (f : α → List UInt8) : Option α → List UInt8
  | none => [0]
  | some a => 1 :: f a

theorem optionBytes_prefixFree {α : Type} {f : α → List UInt8} (hf : PrefixFree f) :
    PrefixFree (optionBytes f) := by
  intro a b r s h
  cases a with
  | none =>
    cases b with
    | none => simpa [optionBytes] using h
    | some b =>
      simp only [optionBytes, List.cons_append, List.cons.injEq] at h
      exact absurd h.1 (by decide)
  | some a =>
    cases b with
    | none =>
      simp only [optionBytes, List.cons_append, List.cons.injEq] at h
      exact absurd h.1 (by decide)
    | some b =>
      simp only [optionBytes, List.cons_append, List.cons.injEq, true_and] at h
      obtain ⟨h1, h2⟩ := hf _ _ _ _ h
      exact ⟨by rw [h1], h2⟩

theorem optionBytes_injective {α : Type} {f : α → List UInt8} (hf : PrefixFree f) :
    Function.Injective (optionBytes f) :=
  (optionBytes_prefixFree hf).injective

/-- Boolean, one byte. -/
def boolBytes (b : Bool) : List UInt8 := if b then [1] else [0]

theorem boolBytes_injective : Function.Injective boolBytes := by
  intro a b h
  cases a <;> cases b <;> simp_all [boolBytes]

theorem boolBytes_prefixFree : PrefixFree boolBytes :=
  prefixFree_of_constLength boolBytes 1 (fun b => by cases b <;> rfl)
    boolBytes_injective

/-! ## Versioning

SPEC §6.2 requires canonical encoders to be versioned.  A version is a
self-delimiting natural-number prefix; prepending it preserves prefix-freeness
and injectivity. -/

/-- Prefix an encoder with its schema version. -/
def versioned {α : Type} (version : Nat) (f : α → List UInt8) (a : α) : List UInt8 :=
  natBytes version ++ f a

theorem versioned_prefixFree {α : Type} (version : Nat) {f : α → List UInt8}
    (hf : PrefixFree f) : PrefixFree (versioned version f) := by
  intro a b r s h
  simp only [versioned, List.append_assoc] at h
  obtain ⟨_, h2⟩ := natBytes_prefixFree _ _ _ _ h
  exact hf _ _ _ _ h2

theorem versioned_injective {α : Type} (version : Nat) {f : α → List UInt8}
    (hf : PrefixFree f) : Function.Injective (versioned version f) :=
  (versioned_prefixFree version hf).injective

/-! ## `ByteArray`-valued encoders

These are the encoders installed into a `CanonicalSchema`. -/

/-- Pack a prefix-free list encoder into a `ByteArray`-valued encoder. -/
def packed {α : Type} (f : α → List UInt8) (a : α) : ByteArray := pack (f a)

theorem packed_injective {α : Type} {f : α → List UInt8}
    (hf : Function.Injective f) : Function.Injective (packed f) := by
  intro a b h
  exact hf (pack_injective h)

theorem packed_injective_of_prefixFree {α : Type} {f : α → List UInt8}
    (hf : PrefixFree f) : Function.Injective (packed f) :=
  packed_injective hf.injective

/-- Canonical `ByteArray` encoding of a natural number. -/
def encodeNat (n : Nat) : ByteArray := packed natBytes n

theorem encodeNat_injective : Function.Injective encodeNat :=
  packed_injective natBytes_injective

/-- Canonical `ByteArray` encoding of a byte string. -/
def encodeByteArray (b : ByteArray) : ByteArray := packed byteArrayBytes b

theorem encodeByteArray_injective : Function.Injective encodeByteArray :=
  packed_injective byteArrayBytes_injective

/-- Canonical `ByteArray` encoding of a `UInt8`. -/
def encodeUInt8 (x : UInt8) : ByteArray := packed u8Bytes x

theorem encodeUInt8_injective : Function.Injective encodeUInt8 :=
  packed_injective u8Bytes_injective

/-- Canonical `ByteArray` encoding of a `UInt16`. -/
def encodeUInt16 (x : UInt16) : ByteArray := packed u16Bytes x

theorem encodeUInt16_injective : Function.Injective encodeUInt16 :=
  packed_injective u16Bytes_injective

/-- Canonical `ByteArray` encoding of a `UInt32`. -/
def encodeUInt32 (x : UInt32) : ByteArray := packed u32Bytes x

theorem encodeUInt32_injective : Function.Injective encodeUInt32 :=
  packed_injective u32Bytes_injective

/-- Canonical `ByteArray` encoding of a `UInt64`. -/
def encodeUInt64 (x : UInt64) : ByteArray := packed u64Bytes x

theorem encodeUInt64_injective : Function.Injective encodeUInt64 :=
  packed_injective u64Bytes_injective

/-- Reinterpret a `ByteArray`-valued encoder as a prefix-free list encoder by
length-prefixing its output. -/
def framed {α : Type} (e : α → ByteArray) (a : α) : List UInt8 :=
  byteArrayBytes (e a)

theorem framed_prefixFree {α : Type} {e : α → ByteArray}
    (he : Function.Injective e) : PrefixFree (framed e) :=
  byteArrayBytes_prefixFree.comp he

theorem framed_injective {α : Type} {e : α → ByteArray}
    (he : Function.Injective e) : Function.Injective (framed e) :=
  (framed_prefixFree he).injective

end Bytes
end WasmGemmGnaf.Foundation

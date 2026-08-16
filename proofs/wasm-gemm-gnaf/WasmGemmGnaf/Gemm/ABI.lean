import WasmGemmGnaf.Gemm.Descriptor
import WasmGemmGnaf.Foundation.Bytes
set_option autoImplicit false
set_option maxRecDepth 4000

/-!
# Gemm: the released ABI (SPEC §8.3)

> The release ABI header is exactly 256 bytes at `ptr`, all multibyte fields are
> little-endian, and all offsets are relative to `ptr`:
>
> | Bytes | Field |
> |---:|---|
> | `0..3` | ASCII magic `WGNG` |
> | `4..5` | ABI version `1` as `u16` |
> | `6..7` | header size `256` as `u16` |
> | `8,9,10,11` | A, B, C, and accumulator kind tags |
> | `12` | arithmetic-mode tag |
> | `13` | transpose bits: bit 0 A, bit 1 B; other bits zero |
> | `14` | aliasing tag |
> | `15` | zero |
> | `16..23,24..31,32..39,40..47` | `m`, `n`, `k`, batch as `u64` |
> | `48..87` | A view: offset, byte length, row stride, column stride, batch stride |
> | `88..127` | B view in the same form |
> | `128..167` | C view in the same form |
> | `168..183` | alpha bits, value-width prefix followed by zero padding |
> | `184..199` | beta bits, value-width prefix followed by zero padding |
> | `200..207,208..215` | scratch offset and byte length |
> | `216..223,224..231` | status-detail offset and byte length |
> | `232..255` | zero |

Every offset in that table is *derived* here from the field widths — see
`header_cumulative_offsets`, which is `rfl` — so a width edit cannot silently
move a field.  `abi_roundtrip` proves that decoding the encoding of any header
returns that header.
-/

namespace WasmGemmGnaf.Gemm

/-! ## Little-endian byte codecs -/

/-- `w` little-endian bytes of `n`. -/
def natToBytesLE : Nat → Nat → List UInt8
  | 0, _ => []
  | w + 1, n => UInt8.ofNat (n % 256) :: natToBytesLE w (n / 256)

/-- Value of a little-endian byte list. -/
def natOfBytesLE : List UInt8 → Nat
  | [] => 0
  | b :: bs => b.toNat + 256 * natOfBytesLE bs

@[simp] theorem natToBytesLE_length (w n : Nat) : (natToBytesLE w n).length = w := by
  induction w generalizing n with
  | zero => rfl
  | succ w ih => simp [natToBytesLE, ih]

theorem natOfBytesLE_natToBytesLE (w n : Nat) (h : n < 256 ^ w) :
    natOfBytesLE (natToBytesLE w n) = n := by
  induction w generalizing n with
  | zero =>
    simp only [Nat.pow_zero] at h
    simp only [natToBytesLE, natOfBytesLE]
    omega
  | succ w ih =>
    have h1 : n / 256 < 256 ^ w := by
      rw [Nat.pow_succ] at h
      omega
    have h2 : (2 : Nat) ^ 8 = 256 := by decide
    simp only [natToBytesLE, natOfBytesLE, UInt8.toNat_ofNat', h2, ih _ h1]
    omega

theorem natOfBytesLE_lt (l : List UInt8) : natOfBytesLE l < 256 ^ l.length := by
  induction l with
  | nil => decide
  | cons b bs ih =>
    have hb : b.toNat < 256 := by have := b.toNat_lt; omega
    simp only [natOfBytesLE, List.length_cons, Nat.pow_succ]
    have := Nat.two_pow_pos bs.length
    omega

/-! ## Fixed-width field tables -/

/-- Concatenated little-endian encoding of a `(width, value)` table. -/
def encodeFields : List (Nat × Nat) → List UInt8
  | [] => []
  | (w, v) :: rest => natToBytesLE w v ++ encodeFields rest

/-- Byte offset of field `i` in the table: the sum of the preceding widths. -/
def cumWidth (fs : List (Nat × Nat)) (i : Nat) : Nat :=
  ((fs.take i).map Prod.fst).sum

@[simp] theorem cumWidth_zero (fs : List (Nat × Nat)) : cumWidth fs 0 = 0 := rfl

theorem cumWidth_cons (p : Nat × Nat) (rest : List (Nat × Nat)) (i : Nat) :
    cumWidth (p :: rest) (i + 1) = p.1 + cumWidth rest i := by
  simp [cumWidth]

/-- The little-endian value stored at `[off, off + w)`. -/
def fieldValue (l : List UInt8) (off w : Nat) : Nat :=
  natOfBytesLE ((l.drop off).take w)

theorem encodeFields_length (fs : List (Nat × Nat)) :
    (encodeFields fs).length = (fs.map Prod.fst).sum := by
  induction fs with
  | nil => rfl
  | cons p rest ih =>
    obtain ⟨w, v⟩ := p
    simp [encodeFields, ih]

/-- Reading field `i` back out of the encoding returns its value. -/
theorem fieldValue_encodeFields (fs : List (Nat × Nat))
    (hb : ∀ p ∈ fs, p.2 < 256 ^ p.1) (i w v : Nat) (hi : fs[i]? = some (w, v)) :
    fieldValue (encodeFields fs) (cumWidth fs i) w = v := by
  induction fs generalizing i with
  | nil => simp at hi
  | cons p rest ih =>
    cases i with
    | zero =>
      simp only [List.getElem?_cons_zero, Option.some.injEq] at hi
      subst hi
      simp only [cumWidth_zero, encodeFields, fieldValue, List.drop_zero]
      rw [List.take_left' (natToBytesLE_length w v)]
      exact natOfBytesLE_natToBytesLE w v (hb (w, v) List.mem_cons_self)
    | succ i =>
      simp only [List.getElem?_cons_succ] at hi
      have hrest : ∀ q ∈ rest, q.2 < 256 ^ q.1 :=
        fun q hq => hb q (List.mem_cons_of_mem _ hq)
      have hres := ih hrest i hi
      obtain ⟨pw, pv⟩ := p
      simp only [cumWidth_cons, encodeFields, fieldValue] at *
      rw [← List.drop_drop, List.drop_left' (natToBytesLE_length pw pv)]
      exact hres

theorem cumWidth_eq (fs : List (Nat × Nat)) (i : Nat) :
    cumWidth fs i = (((fs.map Prod.fst).take i).sum) := by
  simp [cumWidth, List.map_take]

/-! ## Width bounds -/

theorem u8_lt (x : UInt8) : x.toNat < 256 ^ 1 := by have := x.toNat_lt; omega
theorem u16_lt (x : UInt16) : x.toNat < 256 ^ 2 := by have := x.toNat_lt; omega
theorem u32_lt (x : UInt32) : x.toNat < 256 ^ 4 := by have := x.toNat_lt; omega
theorem u64_lt (x : UInt64) : x.toNat < 256 ^ 8 := by have := x.toNat_lt; omega

/-! ## Header constants (SPEC §8.3) -/

/-- ASCII magic `WGNG`. -/
def magicBytes : List UInt8 := [0x57, 0x47, 0x4e, 0x47]

/-- The magic read as a little-endian `u32`. -/
def magicValue : Nat := 0x474e4757

theorem magicValue_bytes : natToBytesLE 4 magicValue = magicBytes := by decide

/-- ABI version `1`. -/
def abiVersion : Nat := 1

/-- Header size `256`. -/
def abiHeaderSize : Nat := 256

theorem abiHeaderSize_eq_headerBytes : abiHeaderSize = headerBytes := rfl

/-! ## The raw header -/

/-- The 256-byte ABI header, one field per table row.  Strides are the raw
two's-complement `i64` bit patterns; `View` reinterprets them over `Int`. -/
structure RawHeader where
  /-- bytes `0 .. 3` -/
  magic : UInt32
  /-- bytes `4 .. 5` -/
  version : UInt16
  /-- bytes `6 .. 7` -/
  headerSize : UInt16
  /-- bytes `8 .. 8` -/
  aTag : UInt8
  /-- bytes `9 .. 9` -/
  bTag : UInt8
  /-- bytes `10 .. 10` -/
  cTag : UInt8
  /-- bytes `11 .. 11` -/
  accTag : UInt8
  /-- bytes `12 .. 12` -/
  modeTag : UInt8
  /-- bytes `13 .. 13` -/
  transposeBits : UInt8
  /-- bytes `14 .. 14` -/
  aliasTag : UInt8
  /-- bytes `15 .. 15` -/
  reserved15 : UInt8
  /-- bytes `16 .. 23` -/
  m : UInt64
  /-- bytes `24 .. 31` -/
  n : UInt64
  /-- bytes `32 .. 39` -/
  k : UInt64
  /-- bytes `40 .. 47` -/
  batch : UInt64
  /-- bytes `48 .. 55` -/
  aOffset : UInt64
  /-- bytes `56 .. 63` -/
  aByteLength : UInt64
  /-- bytes `64 .. 71` -/
  aRowStride : UInt64
  /-- bytes `72 .. 79` -/
  aColStride : UInt64
  /-- bytes `80 .. 87` -/
  aBatchStride : UInt64
  /-- bytes `88 .. 95` -/
  bOffset : UInt64
  /-- bytes `96 .. 103` -/
  bByteLength : UInt64
  /-- bytes `104 .. 111` -/
  bRowStride : UInt64
  /-- bytes `112 .. 119` -/
  bColStride : UInt64
  /-- bytes `120 .. 127` -/
  bBatchStride : UInt64
  /-- bytes `128 .. 135` -/
  cOffset : UInt64
  /-- bytes `136 .. 143` -/
  cByteLength : UInt64
  /-- bytes `144 .. 151` -/
  cRowStride : UInt64
  /-- bytes `152 .. 159` -/
  cColStride : UInt64
  /-- bytes `160 .. 167` -/
  cBatchStride : UInt64
  /-- bytes `168 .. 175` -/
  alphaBits : UInt64
  /-- bytes `176 .. 183` -/
  alphaPad : UInt64
  /-- bytes `184 .. 191` -/
  betaBits : UInt64
  /-- bytes `192 .. 199` -/
  betaPad : UInt64
  /-- bytes `200 .. 207` -/
  scratchOffset : UInt64
  /-- bytes `208 .. 215` -/
  scratchLength : UInt64
  /-- bytes `216 .. 223` -/
  statusOffset : UInt64
  /-- bytes `224 .. 231` -/
  statusLength : UInt64
  /-- bytes `232 .. 239` -/
  reserved232 : UInt64
  /-- bytes `240 .. 247` -/
  reserved240 : UInt64
  /-- bytes `248 .. 255` -/
  reserved248 : UInt64
  deriving DecidableEq, Repr, Inhabited

namespace RawHeader

/-- The `(width, value)` table of the header, in byte order. -/
def fields (h : RawHeader) : List (Nat × Nat) :=
  [(4, h.magic.toNat),
   (2, h.version.toNat),
   (2, h.headerSize.toNat),
   (1, h.aTag.toNat),
   (1, h.bTag.toNat),
   (1, h.cTag.toNat),
   (1, h.accTag.toNat),
   (1, h.modeTag.toNat),
   (1, h.transposeBits.toNat),
   (1, h.aliasTag.toNat),
   (1, h.reserved15.toNat),
   (8, h.m.toNat),
   (8, h.n.toNat),
   (8, h.k.toNat),
   (8, h.batch.toNat),
   (8, h.aOffset.toNat),
   (8, h.aByteLength.toNat),
   (8, h.aRowStride.toNat),
   (8, h.aColStride.toNat),
   (8, h.aBatchStride.toNat),
   (8, h.bOffset.toNat),
   (8, h.bByteLength.toNat),
   (8, h.bRowStride.toNat),
   (8, h.bColStride.toNat),
   (8, h.bBatchStride.toNat),
   (8, h.cOffset.toNat),
   (8, h.cByteLength.toNat),
   (8, h.cRowStride.toNat),
   (8, h.cColStride.toNat),
   (8, h.cBatchStride.toNat),
   (8, h.alphaBits.toNat),
   (8, h.alphaPad.toNat),
   (8, h.betaBits.toNat),
   (8, h.betaPad.toNat),
   (8, h.scratchOffset.toNat),
   (8, h.scratchLength.toNat),
   (8, h.statusOffset.toNat),
   (8, h.statusLength.toNat),
   (8, h.reserved232.toNat),
   (8, h.reserved240.toNat),
   (8, h.reserved248.toNat)]

/-- The table has one row per header field. -/
theorem fields_length (h : RawHeader) : (h.fields).length = 41 := rfl

/-- The field widths, in byte order. -/
def widths : List Nat :=
  [4, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
   8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8]

theorem fields_widths (h : RawHeader) : h.fields.map Prod.fst = widths := rfl

theorem fields_bounded (h : RawHeader) : ∀ p ∈ h.fields, p.2 < 256 ^ p.1 := by
  intro p hp
  simp only [fields, List.mem_cons, List.not_mem_nil, or_false] at hp
  rcases hp with rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl <;>
    first
      | exact u8_lt _
      | exact u16_lt _
      | exact u32_lt _
      | exact u64_lt _

/-- Every offset in SPEC §8.3's table is the cumulative width of the fields
before it, and the header is exactly 256 bytes. -/
theorem header_cumulative_offsets :
    (List.range 42).map (fun i => ((widths.take i).sum)) =
      [0, 4, 6, 8, 9, 10, 11, 12, 13, 14, 15, 16, 24, 32, 40, 48, 56, 64, 72, 80, 88,
       96, 104, 112, 120, 128, 136, 144, 152, 160, 168, 176, 184, 192, 200, 208, 216,
       224, 232, 240, 248, 256] := by decide

/-- The literal offsets of SPEC §8.3's table really are the field offsets. -/
theorem header_field_offsets (h : RawHeader) :
    (List.range 42).map (cumWidth h.fields) =
      [0, 4, 6, 8, 9, 10, 11, 12, 13, 14, 15, 16, 24, 32, 40, 48, 56, 64, 72, 80, 88,
       96, 104, 112, 120, 128, 136, 144, 152, 160, 168, 176, 184, 192, 200, 208, 216,
       224, 232, 240, 248, 256] := by
  rw [← header_cumulative_offsets]
  apply List.map_congr_left
  intro i _
  rw [cumWidth_eq, fields_widths]

theorem cumWidth_fields (h : RawHeader) (i : Nat) :
    cumWidth h.fields i = (widths.take i).sum := by
  rw [cumWidth_eq, fields_widths]

end RawHeader

/-- Reading field `i` of a header back out of its encoding, at the literal byte
offset SPEC §8.3 assigns to it. -/
theorem fieldValue_header (h : RawHeader) (i w v off : Nat)
    (hi : h.fields[i]? = some (w, v)) (hoff : (RawHeader.widths.take i).sum = off) :
    fieldValue (encodeFields h.fields) off w = v := by
  rw [← hoff, ← RawHeader.cumWidth_fields h i]
  exact fieldValue_encodeFields h.fields (RawHeader.fields_bounded h) i w v hi

/-- Encode a header to its exact 256 bytes. -/
def encodeHeaderList (h : RawHeader) : List UInt8 := encodeFields h.fields

/-- Encode a header to its exact 256 bytes. -/
def encodeHeader (h : RawHeader) : ByteArray :=
  Foundation.Bytes.pack (encodeHeaderList h)

theorem encodeHeaderList_length (h : RawHeader) : (encodeHeaderList h).length = 256 := by
  rw [encodeHeaderList, encodeFields_length, RawHeader.fields_widths]
  decide

@[simp] theorem encodeHeader_size (h : RawHeader) : (encodeHeader h).size = 256 := by
  simp [encodeHeader, encodeHeaderList_length]

/-- Decode a header from a byte list, at the fixed offsets of SPEC §8.3. -/
def decodeHeaderList (l : List UInt8) : Option RawHeader :=
  if 256 ≤ l.length then
    some {
      magic := UInt32.ofNat (fieldValue l 0 4),
      version := UInt16.ofNat (fieldValue l 4 2),
      headerSize := UInt16.ofNat (fieldValue l 6 2),
      aTag := UInt8.ofNat (fieldValue l 8 1),
      bTag := UInt8.ofNat (fieldValue l 9 1),
      cTag := UInt8.ofNat (fieldValue l 10 1),
      accTag := UInt8.ofNat (fieldValue l 11 1),
      modeTag := UInt8.ofNat (fieldValue l 12 1),
      transposeBits := UInt8.ofNat (fieldValue l 13 1),
      aliasTag := UInt8.ofNat (fieldValue l 14 1),
      reserved15 := UInt8.ofNat (fieldValue l 15 1),
      m := UInt64.ofNat (fieldValue l 16 8),
      n := UInt64.ofNat (fieldValue l 24 8),
      k := UInt64.ofNat (fieldValue l 32 8),
      batch := UInt64.ofNat (fieldValue l 40 8),
      aOffset := UInt64.ofNat (fieldValue l 48 8),
      aByteLength := UInt64.ofNat (fieldValue l 56 8),
      aRowStride := UInt64.ofNat (fieldValue l 64 8),
      aColStride := UInt64.ofNat (fieldValue l 72 8),
      aBatchStride := UInt64.ofNat (fieldValue l 80 8),
      bOffset := UInt64.ofNat (fieldValue l 88 8),
      bByteLength := UInt64.ofNat (fieldValue l 96 8),
      bRowStride := UInt64.ofNat (fieldValue l 104 8),
      bColStride := UInt64.ofNat (fieldValue l 112 8),
      bBatchStride := UInt64.ofNat (fieldValue l 120 8),
      cOffset := UInt64.ofNat (fieldValue l 128 8),
      cByteLength := UInt64.ofNat (fieldValue l 136 8),
      cRowStride := UInt64.ofNat (fieldValue l 144 8),
      cColStride := UInt64.ofNat (fieldValue l 152 8),
      cBatchStride := UInt64.ofNat (fieldValue l 160 8),
      alphaBits := UInt64.ofNat (fieldValue l 168 8),
      alphaPad := UInt64.ofNat (fieldValue l 176 8),
      betaBits := UInt64.ofNat (fieldValue l 184 8),
      betaPad := UInt64.ofNat (fieldValue l 192 8),
      scratchOffset := UInt64.ofNat (fieldValue l 200 8),
      scratchLength := UInt64.ofNat (fieldValue l 208 8),
      statusOffset := UInt64.ofNat (fieldValue l 216 8),
      statusLength := UInt64.ofNat (fieldValue l 224 8),
      reserved232 := UInt64.ofNat (fieldValue l 232 8),
      reserved240 := UInt64.ofNat (fieldValue l 240 8),
      reserved248 := UInt64.ofNat (fieldValue l 248 8)
    }
  else none

/-- Decode a header from the invocation bytes. -/
def decodeHeader (b : ByteArray) : Option RawHeader := decodeHeaderList b.toList

/-- **SPEC §8.3 round trip.**  Encoding a header and decoding it again returns
exactly that header; in particular this holds for every well-formed header. -/
theorem abi_roundtrip (h : RawHeader) : decodeHeader (encodeHeader h) = some h := by
  have e0 : fieldValue (encodeFields h.fields) 0 4 = h.magic.toNat :=
    fieldValue_header h 0 4 _ 0 rfl (by decide)
  have e1 : fieldValue (encodeFields h.fields) 4 2 = h.version.toNat :=
    fieldValue_header h 1 2 _ 4 rfl (by decide)
  have e2 : fieldValue (encodeFields h.fields) 6 2 = h.headerSize.toNat :=
    fieldValue_header h 2 2 _ 6 rfl (by decide)
  have e3 : fieldValue (encodeFields h.fields) 8 1 = h.aTag.toNat :=
    fieldValue_header h 3 1 _ 8 rfl (by decide)
  have e4 : fieldValue (encodeFields h.fields) 9 1 = h.bTag.toNat :=
    fieldValue_header h 4 1 _ 9 rfl (by decide)
  have e5 : fieldValue (encodeFields h.fields) 10 1 = h.cTag.toNat :=
    fieldValue_header h 5 1 _ 10 rfl (by decide)
  have e6 : fieldValue (encodeFields h.fields) 11 1 = h.accTag.toNat :=
    fieldValue_header h 6 1 _ 11 rfl (by decide)
  have e7 : fieldValue (encodeFields h.fields) 12 1 = h.modeTag.toNat :=
    fieldValue_header h 7 1 _ 12 rfl (by decide)
  have e8 : fieldValue (encodeFields h.fields) 13 1 = h.transposeBits.toNat :=
    fieldValue_header h 8 1 _ 13 rfl (by decide)
  have e9 : fieldValue (encodeFields h.fields) 14 1 = h.aliasTag.toNat :=
    fieldValue_header h 9 1 _ 14 rfl (by decide)
  have e10 : fieldValue (encodeFields h.fields) 15 1 = h.reserved15.toNat :=
    fieldValue_header h 10 1 _ 15 rfl (by decide)
  have e11 : fieldValue (encodeFields h.fields) 16 8 = h.m.toNat :=
    fieldValue_header h 11 8 _ 16 rfl (by decide)
  have e12 : fieldValue (encodeFields h.fields) 24 8 = h.n.toNat :=
    fieldValue_header h 12 8 _ 24 rfl (by decide)
  have e13 : fieldValue (encodeFields h.fields) 32 8 = h.k.toNat :=
    fieldValue_header h 13 8 _ 32 rfl (by decide)
  have e14 : fieldValue (encodeFields h.fields) 40 8 = h.batch.toNat :=
    fieldValue_header h 14 8 _ 40 rfl (by decide)
  have e15 : fieldValue (encodeFields h.fields) 48 8 = h.aOffset.toNat :=
    fieldValue_header h 15 8 _ 48 rfl (by decide)
  have e16 : fieldValue (encodeFields h.fields) 56 8 = h.aByteLength.toNat :=
    fieldValue_header h 16 8 _ 56 rfl (by decide)
  have e17 : fieldValue (encodeFields h.fields) 64 8 = h.aRowStride.toNat :=
    fieldValue_header h 17 8 _ 64 rfl (by decide)
  have e18 : fieldValue (encodeFields h.fields) 72 8 = h.aColStride.toNat :=
    fieldValue_header h 18 8 _ 72 rfl (by decide)
  have e19 : fieldValue (encodeFields h.fields) 80 8 = h.aBatchStride.toNat :=
    fieldValue_header h 19 8 _ 80 rfl (by decide)
  have e20 : fieldValue (encodeFields h.fields) 88 8 = h.bOffset.toNat :=
    fieldValue_header h 20 8 _ 88 rfl (by decide)
  have e21 : fieldValue (encodeFields h.fields) 96 8 = h.bByteLength.toNat :=
    fieldValue_header h 21 8 _ 96 rfl (by decide)
  have e22 : fieldValue (encodeFields h.fields) 104 8 = h.bRowStride.toNat :=
    fieldValue_header h 22 8 _ 104 rfl (by decide)
  have e23 : fieldValue (encodeFields h.fields) 112 8 = h.bColStride.toNat :=
    fieldValue_header h 23 8 _ 112 rfl (by decide)
  have e24 : fieldValue (encodeFields h.fields) 120 8 = h.bBatchStride.toNat :=
    fieldValue_header h 24 8 _ 120 rfl (by decide)
  have e25 : fieldValue (encodeFields h.fields) 128 8 = h.cOffset.toNat :=
    fieldValue_header h 25 8 _ 128 rfl (by decide)
  have e26 : fieldValue (encodeFields h.fields) 136 8 = h.cByteLength.toNat :=
    fieldValue_header h 26 8 _ 136 rfl (by decide)
  have e27 : fieldValue (encodeFields h.fields) 144 8 = h.cRowStride.toNat :=
    fieldValue_header h 27 8 _ 144 rfl (by decide)
  have e28 : fieldValue (encodeFields h.fields) 152 8 = h.cColStride.toNat :=
    fieldValue_header h 28 8 _ 152 rfl (by decide)
  have e29 : fieldValue (encodeFields h.fields) 160 8 = h.cBatchStride.toNat :=
    fieldValue_header h 29 8 _ 160 rfl (by decide)
  have e30 : fieldValue (encodeFields h.fields) 168 8 = h.alphaBits.toNat :=
    fieldValue_header h 30 8 _ 168 rfl (by decide)
  have e31 : fieldValue (encodeFields h.fields) 176 8 = h.alphaPad.toNat :=
    fieldValue_header h 31 8 _ 176 rfl (by decide)
  have e32 : fieldValue (encodeFields h.fields) 184 8 = h.betaBits.toNat :=
    fieldValue_header h 32 8 _ 184 rfl (by decide)
  have e33 : fieldValue (encodeFields h.fields) 192 8 = h.betaPad.toNat :=
    fieldValue_header h 33 8 _ 192 rfl (by decide)
  have e34 : fieldValue (encodeFields h.fields) 200 8 = h.scratchOffset.toNat :=
    fieldValue_header h 34 8 _ 200 rfl (by decide)
  have e35 : fieldValue (encodeFields h.fields) 208 8 = h.scratchLength.toNat :=
    fieldValue_header h 35 8 _ 208 rfl (by decide)
  have e36 : fieldValue (encodeFields h.fields) 216 8 = h.statusOffset.toNat :=
    fieldValue_header h 36 8 _ 216 rfl (by decide)
  have e37 : fieldValue (encodeFields h.fields) 224 8 = h.statusLength.toNat :=
    fieldValue_header h 37 8 _ 224 rfl (by decide)
  have e38 : fieldValue (encodeFields h.fields) 232 8 = h.reserved232.toNat :=
    fieldValue_header h 38 8 _ 232 rfl (by decide)
  have e39 : fieldValue (encodeFields h.fields) 240 8 = h.reserved240.toNat :=
    fieldValue_header h 39 8 _ 240 rfl (by decide)
  have e40 : fieldValue (encodeFields h.fields) 248 8 = h.reserved248.toNat :=
    fieldValue_header h 40 8 _ 248 rfl (by decide)
  have hlen : (encodeFields h.fields).length = 256 := by
    rw [encodeFields_length, RawHeader.fields_widths]
    decide
  show decodeHeaderList (Foundation.Bytes.pack (encodeHeaderList h)).toList = some h
  rw [encodeHeaderList]
  rw [Foundation.Bytes.toList_pack, decodeHeaderList,
    if_pos (by omega : 256 ≤ (encodeFields h.fields).length)]
  simp only [e0, e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11, e12, e13, e14, e15, e16, e17, e18, e19, e20, e21, e22, e23, e24, e25, e26, e27, e28, e29, e30, e31, e32, e33, e34, e35, e36, e37, e38, e39, e40,
    UInt8.ofNat_toNat, UInt16.ofNat_toNat, UInt32.ofNat_toNat, UInt64.ofNat_toNat]

/-- The list-level round trip. -/
theorem abi_roundtrip_list (h : RawHeader) :
    decodeHeaderList (encodeHeaderList h) = some h := by
  have := abi_roundtrip h
  simpa [decodeHeader, encodeHeader] using this

/-! ## Status codes (SPEC §8.3)

"Function results are `0=success`, `1=invalid`, `2=unsupported`,
`3=resource-exhausted`, `4=checked-overflow`, and `5=arithmetic-exception`; no
other return is sanctioned." -/

/-- The six sanctioned function results. -/
inductive StatusCode
  | success | invalid | unsupported | resourceExhausted | checkedOverflow
  | arithmeticException
  deriving DecidableEq, Repr, Inhabited

namespace StatusCode

def code : StatusCode → Nat
  | success => 0
  | invalid => 1
  | unsupported => 2
  | resourceExhausted => 3
  | checkedOverflow => 4
  | arithmeticException => 5

def ofCode : Nat → Option StatusCode
  | 0 => some success
  | 1 => some invalid
  | 2 => some unsupported
  | 3 => some resourceExhausted
  | 4 => some checkedOverflow
  | 5 => some arithmeticException
  | _ + 6 => none

def all : List StatusCode :=
  [success, invalid, unsupported, resourceExhausted, checkedOverflow,
   arithmeticException]

theorem mem_all (c : StatusCode) : c ∈ all := by cases c <;> decide
theorem all_nodup : all.Nodup := by decide
theorem all_map_code : all.map code = List.range 6 := by decide
theorem code_lt (c : StatusCode) : c.code < 6 := by cases c <;> decide

theorem code_injective : Function.Injective code := by
  intro a b h; cases a <;> cases b <;> simp_all [code]

@[simp] theorem ofCode_code (c : StatusCode) : ofCode c.code = some c := by cases c <;> rfl

theorem code_ofCode {n : Nat} {c : StatusCode} (h : ofCode n = some c) : c.code = n := by
  match n with
  | 0 | 1 | 2 | 3 | 4 | 5 => simp only [ofCode, Option.some.injEq] at h; subst h; rfl
  | _ + 6 => exact absurd h (by simp [ofCode])

theorem ofCode_eq_none_iff (n : Nat) : ofCode n = none ↔ 6 ≤ n := by
  match n with
  | 0 | 1 | 2 | 3 | 4 | 5 => simp [ofCode]
  | _ + 6 => simp [ofCode]

end StatusCode

/-! ## Status-detail field codes (SPEC §8.3)

"Field codes are `0=none`, `1=header`, `2=version`, `3=kind`,
`4=arithmetic-mode`, `5=dimension`, `6=view`, `7=alias`, `8=resource`,
`9=overflow`, and `10=arithmetic`." -/

/-- The eleven status-detail field codes. -/
inductive FieldCode
  | none | header | version | kind | arithmeticMode | dimension | view | alias
  | resource | overflow | arithmetic
  deriving DecidableEq, Repr, Inhabited

namespace FieldCode

def code : FieldCode → Nat
  | none => 0
  | header => 1
  | version => 2
  | kind => 3
  | arithmeticMode => 4
  | dimension => 5
  | view => 6
  | alias => 7
  | resource => 8
  | overflow => 9
  | arithmetic => 10

def ofCode : Nat → Option FieldCode
  | 0 => some none
  | 1 => some header
  | 2 => some version
  | 3 => some kind
  | 4 => some arithmeticMode
  | 5 => some dimension
  | 6 => some view
  | 7 => some alias
  | 8 => some resource
  | 9 => some overflow
  | 10 => some arithmetic
  | _ + 11 => Option.none

def all : List FieldCode :=
  [none, header, version, kind, arithmeticMode, dimension, view, alias, resource,
   overflow, arithmetic]

theorem mem_all (c : FieldCode) : c ∈ all := by cases c <;> decide
theorem all_nodup : all.Nodup := by decide
theorem all_map_code : all.map code = List.range 11 := by decide
theorem code_lt (c : FieldCode) : c.code < 11 := by cases c <;> decide

theorem code_injective : Function.Injective code := by
  intro a b h; cases a <;> cases b <;> simp_all [code]

@[simp] theorem ofCode_code (c : FieldCode) : ofCode c.code = some c := by cases c <;> rfl

theorem code_ofCode {n : Nat} {c : FieldCode} (h : ofCode n = some c) : c.code = n := by
  match n with
  | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 =>
      simp only [ofCode, Option.some.injEq] at h; subst h; rfl
  | _ + 11 => exact absurd h (by simp [ofCode])

theorem ofCode_eq_none_iff (n : Nat) : ofCode n = Option.none ↔ 11 ≤ n := by
  match n with
  | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 => simp [ofCode]
  | _ + 11 => simp [ofCode]

end FieldCode

/-! ## The 32-byte status-detail record (SPEC §8.3)

"Its little-endian record is: status `u32` at `0..3`, field code `u32` at
`4..7`, offending byte offset or logical index `u64` at `8..15`, required
quantity `u64` at `16..23`, and available quantity `u64` at `24..31`." -/

/-- The 32-byte status-detail record. -/
structure StatusDetail where
  /-- bytes `0 .. 3` -/
  status : UInt32
  /-- bytes `4 .. 7` -/
  fieldCode : UInt32
  /-- bytes `8 .. 15` -/
  offendingOffset : UInt64
  /-- bytes `16 .. 23` -/
  required : UInt64
  /-- bytes `24 .. 31` -/
  available : UInt64
  deriving DecidableEq, Repr, Inhabited

namespace StatusDetail

/-- "Success writes an all-zero record." -/
def zero : StatusDetail := ⟨0, 0, 0, 0, 0⟩

/-- The `(width, value)` table of the record, in byte order. -/
def fields (r : StatusDetail) : List (Nat × Nat) :=
  [(4, r.status.toNat), (4, r.fieldCode.toNat), (8, r.offendingOffset.toNat),
   (8, r.required.toNat), (8, r.available.toNat)]

/-- The record's field widths. -/
def widths : List Nat := [4, 4, 8, 8, 8]

theorem fields_widths (r : StatusDetail) : r.fields.map Prod.fst = widths := rfl

/-- The record is exactly 32 bytes, and its fields sit at `0, 4, 8, 16, 24`. -/
theorem record_offsets :
    (List.range 6).map (fun i => (widths.take i).sum) = [0, 4, 8, 16, 24, 32] := by
  decide

theorem fields_bounded (r : StatusDetail) : ∀ p ∈ r.fields, p.2 < 256 ^ p.1 := by
  intro p hp
  simp only [fields, List.mem_cons, List.not_mem_nil, or_false] at hp
  rcases hp with rfl | rfl | rfl | rfl | rfl <;>
    first
      | exact u32_lt _
      | exact u64_lt _

theorem cumWidth_fields (r : StatusDetail) (i : Nat) :
    cumWidth r.fields i = (widths.take i).sum := by
  rw [cumWidth_eq, fields_widths]

end StatusDetail

/-- Reading field `i` of a status-detail record back out of its encoding. -/
theorem fieldValue_statusDetail (r : StatusDetail) (i w v off : Nat)
    (hi : r.fields[i]? = some (w, v)) (hoff : (StatusDetail.widths.take i).sum = off) :
    fieldValue (encodeFields r.fields) off w = v := by
  rw [← hoff, ← StatusDetail.cumWidth_fields r i]
  exact fieldValue_encodeFields r.fields (StatusDetail.fields_bounded r) i w v hi

/-- Encode the status-detail record to its exact 32 bytes. -/
def encodeStatusDetailList (r : StatusDetail) : List UInt8 := encodeFields r.fields

/-- Encode the status-detail record to its exact 32 bytes. -/
def encodeStatusDetail (r : StatusDetail) : ByteArray :=
  Foundation.Bytes.pack (encodeStatusDetailList r)

theorem encodeStatusDetailList_length (r : StatusDetail) :
    (encodeStatusDetailList r).length = 32 := by
  rw [encodeStatusDetailList, encodeFields_length, StatusDetail.fields_widths]
  decide

@[simp] theorem encodeStatusDetail_size (r : StatusDetail) :
    (encodeStatusDetail r).size = 32 := by
  simp [encodeStatusDetail, encodeStatusDetailList_length]

/-- Decode a status-detail record at the fixed offsets of SPEC §8.3. -/
def decodeStatusDetailList (l : List UInt8) : Option StatusDetail :=
  if 32 ≤ l.length then
    some {
      status := UInt32.ofNat (fieldValue l 0 4),
      fieldCode := UInt32.ofNat (fieldValue l 4 4),
      offendingOffset := UInt64.ofNat (fieldValue l 8 8),
      required := UInt64.ofNat (fieldValue l 16 8),
      available := UInt64.ofNat (fieldValue l 24 8)
    }
  else Option.none

def decodeStatusDetail (b : ByteArray) : Option StatusDetail :=
  decodeStatusDetailList b.toList

/-- Round trip for the status-detail record. -/
theorem status_detail_roundtrip (r : StatusDetail) :
    decodeStatusDetail (encodeStatusDetail r) = some r := by
  have e0 : fieldValue (encodeFields r.fields) 0 4 = r.status.toNat :=
    fieldValue_statusDetail r 0 4 _ 0 rfl (by decide)
  have e1 : fieldValue (encodeFields r.fields) 4 4 = r.fieldCode.toNat :=
    fieldValue_statusDetail r 1 4 _ 4 rfl (by decide)
  have e2 : fieldValue (encodeFields r.fields) 8 8 = r.offendingOffset.toNat :=
    fieldValue_statusDetail r 2 8 _ 8 rfl (by decide)
  have e3 : fieldValue (encodeFields r.fields) 16 8 = r.required.toNat :=
    fieldValue_statusDetail r 3 8 _ 16 rfl (by decide)
  have e4 : fieldValue (encodeFields r.fields) 24 8 = r.available.toNat :=
    fieldValue_statusDetail r 4 8 _ 24 rfl (by decide)
  have hlen : (encodeFields r.fields).length = 32 := by
    rw [encodeFields_length, StatusDetail.fields_widths]
    decide
  show decodeStatusDetailList (Foundation.Bytes.pack (encodeStatusDetailList r)).toList
    = some r
  rw [encodeStatusDetailList, Foundation.Bytes.toList_pack, decodeStatusDetailList,
    if_pos (by omega : 32 ≤ (encodeFields r.fields).length)]
  simp only [e0, e1, e2, e3, e4, UInt32.ofNat_toNat, UInt64.ofNat_toNat]

/-! ## The numeric constants of SPEC §8.3

"The allowed-kind bitset is `0x0fff` for A, B, or C and `0x1fff` for the
accumulator; the allowed-mode bitset is `0x0f`. … The packed observed tuple is
`aTag | (bTag << 8) | (cTag << 16) | (accTag << 24) | (modeTag << 32)`. All
shifts are on `u64`." -/

/-- Allowed kind tags for A, B and C. -/
def allowedKindBitset : Nat := 0x0fff

/-- Allowed kind tags for the accumulator (`exactDyadic` included). -/
def allowedAccumulatorKindBitset : Nat := 0x1fff

/-- Allowed arithmetic-mode tags. -/
def allowedModeBitset : Nat := 0x0f

/-- `0x0fff` really is the set of stored kinds. -/
theorem allowedKindBitset_eq :
    (ScalarKind.all.filter (fun k => k.isStored)).foldl
      (fun acc k => acc + 2 ^ k.tag) 0 = allowedKindBitset := by decide

/-- `0x1fff` really is the set of all thirteen kinds. -/
theorem allowedAccumulatorKindBitset_eq :
    ScalarKind.all.foldl (fun acc k => acc + 2 ^ k.tag) 0
      = allowedAccumulatorKindBitset := by decide

/-- `0x0f` really is the set of all four modes. -/
theorem allowedModeBitset_eq :
    ArithmeticMode.all.foldl (fun acc m => acc + 2 ^ m.tag) 0
      = allowedModeBitset := by decide

/-- A stored-kind tag is admissible for A, B or C exactly when its bit is set in
`allowedKindBitset`. -/
theorem kind_allowed_iff (t : Nat) :
    (allowedKindBitset / 2 ^ t % 2 = 1) ↔ t < 12 := by
  match t with
  | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 => simp [allowedKindBitset]
  | m + 12 =>
    have h : (2 : Nat) ^ 12 ≤ 2 ^ (m + 12) :=
      Nat.pow_le_pow_right (by decide) (by omega)
    simp only [allowedKindBitset]
    rw [Nat.div_eq_of_lt (by omega)]
    omega

/-- An accumulator tag is admissible exactly when its bit is set in
`allowedAccumulatorKindBitset`. -/
theorem accumulator_allowed_iff (t : Nat) :
    (allowedAccumulatorKindBitset / 2 ^ t % 2 = 1) ↔ t < 13 := by
  match t with
  | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 =>
      simp [allowedAccumulatorKindBitset]
  | m + 13 =>
    have h : (2 : Nat) ^ 13 ≤ 2 ^ (m + 13) :=
      Nat.pow_le_pow_right (by decide) (by omega)
    simp only [allowedAccumulatorKindBitset]
    rw [Nat.div_eq_of_lt (by omega)]
    omega

/-- A mode tag is admissible exactly when its bit is set in `allowedModeBitset`. -/
theorem mode_allowed_iff (t : Nat) :
    (allowedModeBitset / 2 ^ t % 2 = 1) ↔ t < 4 := by
  match t with
  | 0 | 1 | 2 | 3 => simp [allowedModeBitset]
  | m + 4 =>
    have h : (2 : Nat) ^ 4 ≤ 2 ^ (m + 4) := Nat.pow_le_pow_right (by decide) (by omega)
    simp only [allowedModeBitset]
    rw [Nat.div_eq_of_lt (by omega)]
    omega

/-- SPEC §8.3's packed observed tuple
`aTag | (bTag << 8) | (cTag << 16) | (accTag << 24) | (modeTag << 32)`. -/
def packedKindModeTuple (aTag bTag cTag accTag modeTag : Nat) : Nat :=
  aTag + bTag * 2 ^ 8 + cTag * 2 ^ 16 + accTag * 2 ^ 24 + modeTag * 2 ^ 32

/-- Because every tag is a single byte, the packed tuple is a `u64` value. -/
theorem packedKindModeTuple_lt (a b c acc m : Nat)
    (ha : a < 256) (hb : b < 256) (hc : c < 256) (hacc : acc < 256) (hm : m < 256) :
    packedKindModeTuple a b c acc m < 2 ^ 64 := by
  simp only [packedKindModeTuple]
  have h1 : b * 2 ^ 8 ≤ 255 * 2 ^ 8 := Nat.mul_le_mul_right _ (by omega)
  have h2 : c * 2 ^ 16 ≤ 255 * 2 ^ 16 := Nat.mul_le_mul_right _ (by omega)
  have h3 : acc * 2 ^ 24 ≤ 255 * 2 ^ 24 := Nat.mul_le_mul_right _ (by omega)
  have h4 : m * 2 ^ 32 ≤ 255 * 2 ^ 32 := Nat.mul_le_mul_right _ (by omega)
  omega

/-- The packed tuple determines the five observed tags: no host endianness or
enumeration order participates. -/
theorem packedKindModeTuple_injective {a b c acc m a' b' c' acc' m' : Nat}
    (ha : a < 256) (hb : b < 256) (hc : c < 256) (hacc : acc < 256)
    (ha' : a' < 256) (hb' : b' < 256) (hc' : c' < 256) (hacc' : acc' < 256)
    (h : packedKindModeTuple a b c acc m = packedKindModeTuple a' b' c' acc' m') :
    a = a' ∧ b = b' ∧ c = c' ∧ acc = acc' ∧ m = m' := by
  simp only [packedKindModeTuple] at h
  refine ⟨?_, ?_, ?_, ?_, ?_⟩ <;> omega

end WasmGemmGnaf.Gemm

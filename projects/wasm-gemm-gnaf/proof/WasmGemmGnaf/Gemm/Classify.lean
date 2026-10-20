import WasmGemmGnaf.Gemm.ABI
import WasmGemmGnaf.Foundation.Result
set_option autoImplicit false
set_option maxRecDepth 4000

/-!
# Gemm: the total raw classifier (SPEC §8.3)

> Classification precedence is total and fixed: malformed magic/version/header
> size/reserved bits or truncated bytes produce `invalid`; otherwise unknown or
> profile-disabled kind/mode tags produce `unsupported`; otherwise incompatible
> tags, arithmetic overflow in descriptor calculations, illegal views, or alias
> violations produce `invalid`; otherwise failure of the fixed resource
> predicate produces `resource-exhausted`; all remaining descriptors are
> `valid`. … The first matching class determines both return and status-detail
> record.

The classifier below *is* that cascade, written as four decidable stages so
that `classifier_exact_domain` — "every byte-level well-formed descriptor whose
views fit memory and whose tag combination appears above SHALL classify as valid
unless it violates the explicitly enumerated alias or resource predicate" — is
a direct consequence of the definition rather than a separate claim.

`StatusDetailOf` re-runs the *same* cascade as a total canonical function of the
raw bytes, so "every classified raw input determines one record byte sequence".
-/

namespace WasmGemmGnaf.Gemm

/-! ## The raw invocation carrier (SPEC §8.3) -/

/-- SPEC §8.3's raw-input carrier.  All byte strings of every representable
length participate, including malformed headers. -/
structure RawInvocationBody (P : Wasm.Profile) where
  ptr : UInt32
  len : UInt32
  bytes : ByteArray
  deriving DecidableEq

/-- SPEC §8.3: `RawInvocationLawful` is *exactly*
`bytes.size = len.toNat ∧ ptr.toNat + len.toNat ≤ 2^P.addressBits ∧
pagesFor (ptr.toNat + len.toNat) ≤ P.maxPages`. -/
def RawInvocationLawful (P : Wasm.Profile) (body : RawInvocationBody P) : Prop :=
  body.bytes.size = body.len.toNat ∧
    body.ptr.toNat + body.len.toNat ≤ 2 ^ P.addressBits ∧
    Wasm.pagesFor (body.ptr.toNat + body.len.toNat) ≤ P.maxPages

instance (P : Wasm.Profile) (body : RawInvocationBody P) :
    Decidable (RawInvocationLawful P body) :=
  inferInstanceAs (Decidable (_ ∧ _ ∧ _))

/-- A lawful raw invocation.  Its proof is not part of the identity preimage. -/
structure RawInvocation (P : Wasm.Profile) where
  body : RawInvocationBody P
  lawful : RawInvocationLawful P body

namespace RawInvocationBody

variable {P : Wasm.Profile}

/-- Forget the GEMM-specific wrapper. -/
def toWasm (b : RawInvocationBody P) : Wasm.InvocationBody P :=
  { ptr := b.ptr, len := b.len, bytes := b.bytes }

end RawInvocationBody

/-- SPEC §8.3's bridge: GEMM lawfulness *is* Wasm invocation lawfulness. -/
theorem RawInvocationLawful.toWasmInvocationLawful {P : Wasm.Profile}
    {body : RawInvocationBody P} (h : RawInvocationLawful P body) :
    Wasm.InvocationLawful P body.toWasm := h

/-! ### Canonical encoding of a raw invocation -/

/-- The canonical proof-free encoding: `ptr`, `len`, then the bytes. -/
def encodeRawInvocation {P : Wasm.Profile} (raw : RawInvocation P) : ByteArray :=
  Foundation.Bytes.pack
    (natToBytesLE 4 raw.body.ptr.toNat ++ natToBytesLE 4 raw.body.len.toNat ++
      raw.body.bytes.toList)

/-- Failure modes of raw-invocation decoding. -/
inductive RawDecodeError
  | truncated
  | unlawful
  deriving DecidableEq, Repr, Inhabited

/-- Parse the proof-free carrier back out of the canonical encoding. -/
def parseRawInvocationBody (P : Wasm.Profile) (b : ByteArray) :
    Option (RawInvocationBody P) :=
  let l := b.toList
  if 8 ≤ l.length then
    some { ptr := UInt32.ofNat (fieldValue l 0 4),
           len := UInt32.ofNat (fieldValue l 4 4),
           bytes := Foundation.Bytes.pack (l.drop 8) }
  else none

/-- Decode a raw invocation, re-checking lawfulness. -/
def decodeRawInvocation (P : Wasm.Profile) (b : ByteArray) :
    Except RawDecodeError (RawInvocation P) :=
  match parseRawInvocationBody P b with
  | none => .error .truncated
  | some body =>
      if h : RawInvocationLawful P body then .ok ⟨body, h⟩ else .error .unlawful

theorem parse_encodeRawInvocation {P : Wasm.Profile} (raw : RawInvocation P) :
    parseRawInvocationBody P (encodeRawInvocation raw) = some raw.body := by
  have hp : (natToBytesLE 4 raw.body.ptr.toNat).length = 4 := natToBytesLE_length _ _
  have hl : (natToBytesLE 4 raw.body.len.toNat).length = 4 := natToBytesLE_length _ _
  simp only [parseRawInvocationBody, encodeRawInvocation, Foundation.Bytes.toList_pack]
  rw [if_pos (by simp [hp, hl]; omega : 8 ≤ (natToBytesLE 4 raw.body.ptr.toNat ++
        natToBytesLE 4 raw.body.len.toNat ++ raw.body.bytes.toList).length)]
  have hptr : fieldValue (natToBytesLE 4 raw.body.ptr.toNat ++
      (natToBytesLE 4 raw.body.len.toNat ++ raw.body.bytes.toList)) 0 4
      = raw.body.ptr.toNat := by
    simp only [fieldValue, List.drop_zero]
    rw [List.take_left' hp]
    exact natOfBytesLE_natToBytesLE 4 _ (u32_lt raw.body.ptr)
  have hlen : fieldValue (natToBytesLE 4 raw.body.ptr.toNat ++
      (natToBytesLE 4 raw.body.len.toNat ++ raw.body.bytes.toList)) 4 4
      = raw.body.len.toNat := by
    simp only [fieldValue]
    rw [List.drop_left' hp, List.take_left' hl]
    exact natOfBytesLE_natToBytesLE 4 _ (u32_lt raw.body.len)
  have hbytes : (natToBytesLE 4 raw.body.ptr.toNat ++
      (natToBytesLE 4 raw.body.len.toNat ++ raw.body.bytes.toList)).drop 8
      = raw.body.bytes.toList := by
    have h8 : (8 : Nat) = 4 + 4 := rfl
    rw [h8, ← List.drop_drop, List.drop_left' hp, List.drop_left' hl]
  simp only [List.append_assoc, hptr, hlen, hbytes, UInt32.ofNat_toNat,
    Foundation.Bytes.pack_toList]

/-- **SPEC §8.3**: decoding the canonical encoding of a raw invocation returns
that invocation. -/
theorem raw_invocation_roundtrip {P : Wasm.Profile} (raw : RawInvocation P) :
    decodeRawInvocation P (encodeRawInvocation raw) = .ok raw := by
  simp only [decodeRawInvocation, parse_encodeRawInvocation]
  rw [dif_pos raw.lawful]

/-- **SPEC §8.3**: every lawful `(ptr, len, bytes)` triple really is a raw
invocation; the carrier is not narrowed by the problem implementation. -/
theorem raw_invocation_surjective {P : Wasm.Profile}
    (ptr len : UInt32) (bytes : ByteArray)
    (hlen : bytes.size = len.toNat)
    (hrange : ptr.toNat + len.toNat ≤ 2 ^ P.addressBits)
    (hpages : Wasm.pagesFor (ptr.toNat + len.toNat) ≤ P.maxPages) :
    ∃ raw : RawInvocation P,
      raw.body.ptr = ptr ∧ raw.body.len = len ∧ raw.body.bytes = bytes :=
  ⟨⟨⟨ptr, len, bytes⟩, ⟨hlen, hrange, hpages⟩⟩, rfl, rfl, rfl⟩

/-! ## Saturating conversions (SPEC §8.3)

"A logical index too large for `u64` is saturated at `u64::MAX`." -/

/-- Saturating conversion to `u64`. -/
def sat64 (n : Nat) : UInt64 := UInt64.ofNat (Nat.min n (2 ^ 64 - 1))

/-- Saturating conversion to `u32`. -/
def sat32 (n : Nat) : UInt32 := UInt32.ofNat (Nat.min n (2 ^ 32 - 1))

theorem sat64_of_lt {n : Nat} (h : n < 2 ^ 64) : (sat64 n).toNat = n := by
  have hmin : Nat.min n (2 ^ 64 - 1) = n := Nat.min_eq_left (by omega)
  simp only [sat64, hmin]
  exact UInt64.toNat_ofNat_of_lt' (by omega)

theorem sat64_max : (sat64 (2 ^ 64)).toNat = 2 ^ 64 - 1 := by decide

/-- Saturating conversion of a possibly negative mathematical address. -/
def satAddr (z : Int) : UInt64 := if z ≤ 0 then 0 else sat64 z.toNat

/-! ## Reports -/

/-- Canonical identity of a named GEMM resource or feature. -/
def gemmObjectId (tag : List UInt8) (name : List UInt8) :
    Foundation.CanonicalObjectId :=
  { schemaVersion := 1
    domain := .gemmProblem
    typeTag := Foundation.TypeTag.leaf tag
    canonicalBodyBytes := Foundation.Bytes.pack name }

/-- The invocation-extent resource. -/
def invocationExtentResource : Foundation.CanonicalObjectId :=
  gemmObjectId [0x52] [0x6c, 0x65, 0x6e]

/-- The memory-page resource. -/
def memoryPageResource : Foundation.CanonicalObjectId :=
  gemmObjectId [0x52] [0x70, 0x61, 0x67, 0x65]

/-- The scalar-kind feature. -/
def scalarKindFeature : Foundation.FeatureId :=
  gemmObjectId [0x46] [0x6b, 0x69, 0x6e, 0x64]

/-- The arithmetic-mode feature. -/
def arithmeticModeFeature : Foundation.FeatureId :=
  gemmObjectId [0x46] [0x6d, 0x6f, 0x64, 0x65]

/-- Why a raw input is invalid.  The exact record is computed separately by the
total canonical function `StatusDetailOf`. -/
inductive InvalidReason
  | truncated | badMagic | badVersion | badHeaderSize | reservedNonZero
  | incompatibleTags | dimensionOverflow | illegalView | aliasViolation
  deriving DecidableEq, Repr, Inhabited

/-- A rejected raw input. -/
structure InvalidInputReport where
  reason : InvalidReason
  offendingByteOffset : Nat
  deriving DecidableEq, Repr, Inhabited

/-- An unsupported tag. -/
structure UnsupportedReport where
  feature : Foundation.FeatureId
  offendingByteOffset : Nat
  observedTag : Nat
  deriving DecidableEq

/-! ## Header-level checks -/

/-- SPEC §8.3's first precedence class, in ascending header-byte order.
Returns the reason and the *lowest* offending header byte offset. -/
def headerMalformed (h : RawHeader) : Option (InvalidReason × Nat) :=
  if h.magic.toNat ≠ magicValue then some (.badMagic, 0)
  else if h.version.toNat ≠ abiVersion then some (.badVersion, 4)
  else if h.headerSize.toNat ≠ abiHeaderSize then some (.badHeaderSize, 6)
  else if 4 ≤ h.transposeBits.toNat then some (.reservedNonZero, 13)
  else if h.reserved15 ≠ 0 then some (.reservedNonZero, 15)
  else if h.alphaPad ≠ 0 then some (.reservedNonZero, 176)
  else if h.betaPad ≠ 0 then some (.reservedNonZero, 192)
  else if h.reserved232 ≠ 0 then some (.reservedNonZero, 232)
  else if h.reserved240 ≠ 0 then some (.reservedNonZero, 240)
  else if h.reserved248 ≠ 0 then some (.reservedNonZero, 248)
  else none

/-- SPEC §8.3's second precedence class: unknown or profile-disabled kind and
mode tags, in ascending header-byte order.  Returns the offending byte offset,
the observed tag, and the allowed-tag bitset. -/
def tagUnsupported (h : RawHeader) : Option (Nat × Nat × Nat) :=
  if 12 ≤ h.aTag.toNat then some (8, h.aTag.toNat, allowedKindBitset)
  else if 12 ≤ h.bTag.toNat then some (9, h.bTag.toNat, allowedKindBitset)
  else if 12 ≤ h.cTag.toNat then some (10, h.cTag.toNat, allowedKindBitset)
  else if 13 ≤ h.accTag.toNat then
    some (11, h.accTag.toNat, allowedAccumulatorKindBitset)
  else if 4 ≤ h.modeTag.toNat then some (12, h.modeTag.toNat, allowedModeBitset)
  else none

/-- Tag decoding, total by construction; only ever applied after
`tagUnsupported` has returned `none`. -/
def kindOfTag (t : Nat) : ScalarKind := (ScalarKind.ofTag t).getD .i8

/-- Mode decoding, total by construction. -/
def modeOfTag (t : Nat) : ArithmeticMode := (ArithmeticMode.ofTag t).getD .modular

/-- Aliasing-tag decoding, total by construction. -/
def aliasOfTag (t : Nat) : AliasingTag := (AliasingTag.ofTag t).getD .disjoint

theorem kindOfTag_tag (k : ScalarKind) : kindOfTag k.tag = k := by
  simp [kindOfTag]

theorem modeOfTag_tag (m : ArithmeticMode) : modeOfTag m.tag = m := by
  simp [modeOfTag]

theorem aliasOfTag_tag (a : AliasingTag) : aliasOfTag a.tag = a := by
  simp [aliasOfTag]

/-! ## From header bytes to a descriptor body -/

/-- Bytes actually backed by the profile's page limit. -/
def profileMemoryBytes (P : Wasm.Profile) : Nat := P.maxPages * Wasm.pageSize

/-- The memory window the invocation is read against. -/
def windowOf (P : Wasm.Profile) (raw : RawInvocationBody P) : MemoryWindow where
  ptr := raw.ptr.toNat
  len := raw.len.toNat
  memoryBytes := profileMemoryBytes P

/-- The descriptor body a header denotes.  Strides are reinterpreted as
two's-complement `i64` over the mathematical integers (SPEC §8.3). -/
def descriptorOf (h : RawHeader) : DescriptorBody where
  m := h.m.toNat
  n := h.n.toNat
  k := h.k.toNat
  batch := h.batch.toNat
  aKind := kindOfTag h.aTag.toNat
  bKind := kindOfTag h.bTag.toNat
  cKind := kindOfTag h.cTag.toNat
  accumulatorKind := kindOfTag h.accTag.toNat
  aView := ⟨h.aOffset.toNat, h.aByteLength.toNat, toSigned 64 h.aRowStride.toNat,
            toSigned 64 h.aColStride.toNat, toSigned 64 h.aBatchStride.toNat⟩
  bView := ⟨h.bOffset.toNat, h.bByteLength.toNat, toSigned 64 h.bRowStride.toNat,
            toSigned 64 h.bColStride.toNat, toSigned 64 h.bBatchStride.toNat⟩
  cView := ⟨h.cOffset.toNat, h.cByteLength.toNat, toSigned 64 h.cRowStride.toNat,
            toSigned 64 h.cColStride.toNat, toSigned 64 h.cBatchStride.toNat⟩
  transposeA := h.transposeBits.toNat % 2 = 1
  transposeB := h.transposeBits.toNat / 2 % 2 = 1
  mode := modeOfTag h.modeTag.toNat
  alphaBits := h.alphaBits.toNat
  betaBits := h.betaBits.toNat
  aliasing := aliasOfTag h.aliasTag.toNat
  scratch := ⟨h.scratchOffset.toNat, h.scratchLength.toNat⟩
  statusDetail := ⟨h.statusOffset.toNat, h.statusLength.toNat⟩

/-- SPEC §8.3's "arithmetic overflow in descriptor calculations": every product
of logical extents and every declared interval endpoint must be representable
without wraparound. -/
def dimensionsRepresentable (h : RawHeader) : Prop :=
  h.batch.toNat * h.m.toNat * h.k.toNat < 2 ^ 64 ∧
    h.batch.toNat * h.k.toNat * h.n.toNat < 2 ^ 64 ∧
    h.batch.toNat * h.m.toNat * h.n.toNat < 2 ^ 64 ∧
    h.aOffset.toNat + h.aByteLength.toNat < 2 ^ 64 ∧
    h.bOffset.toNat + h.bByteLength.toNat < 2 ^ 64 ∧
    h.cOffset.toNat + h.cByteLength.toNat < 2 ^ 64 ∧
    h.scratchOffset.toNat + h.scratchLength.toNat < 2 ^ 64 ∧
    h.statusOffset.toNat + h.statusLength.toNat < 2 ^ 64

instance (h : RawHeader) : Decidable (dimensionsRepresentable h) :=
  inferInstanceAs (Decidable (_ ∧ _))

/-- The compatibility premise of the descriptor a header denotes. -/
def headerCompatible (h : RawHeader) : Prop :=
  ((descriptorOf h).arithmetic).Compatible

instance (h : RawHeader) : Decidable (headerCompatible h) :=
  inferInstanceAs (Decidable ((descriptorOf h).arithmetic).Compatible)

/-! ## The resource predicate (SPEC §8.2) -/

/-- SPEC §8.2's fixed resource predicate.  It contains only profile limits: it
"cannot be used to reject a memory-fitting GEMM merely because it is large". -/
def ResourceOk (P : Wasm.Profile) (raw : RawInvocationBody P) : Prop :=
  raw.len.toNat ≤ P.maxInvocationBytes ∧
    Wasm.pagesFor (raw.ptr.toNat + raw.len.toNat) ≤ P.maxPages

instance (P : Wasm.Profile) (raw : RawInvocationBody P) : Decidable (ResourceOk P raw) :=
  inferInstanceAs (Decidable (_ ∧ _))

/-! ## Classification -/

/-- A raw input that survived every precedence class. -/
structure ValidInvocation (P : Wasm.Profile) where
  raw : RawInvocationBody P
  header : RawHeader
  descriptor : Descriptor P
  descriptorMatches : descriptor.body = descriptorOf header
  windowMatches : descriptor.window = windowOf P raw

/-- SPEC §8.3's classification result. -/
inductive Classification (P : Wasm.Profile)
  | valid (invocation : ValidInvocation P)
  | invalid (report : InvalidInputReport)
  | unsupported (report : UnsupportedReport)
  | resourceExhausted (report : Foundation.ResourceReport)

/-- Precedence classes three, four and five. -/
def classifyDescriptor (P : Wasm.Profile) (raw : RawInvocationBody P)
    (h : RawHeader) : Classification P :=
  if hcomp : headerCompatible h then
    if _ : dimensionsRepresentable h then
      if hwf : DescriptorWellFormed (descriptorOf h) (windowOf P raw) then
        if _hres : ResourceOk P raw then
          .valid { raw := raw, header := h
                 , descriptor := ⟨descriptorOf h, windowOf P raw, hcomp, hwf⟩
                 , descriptorMatches := rfl
                 , windowMatches := rfl }
        else
          .resourceExhausted
            (if raw.len.toNat ≤ P.maxInvocationBytes then
               { resource := memoryPageResource, limit := P.maxPages
               , demanded := Wasm.pagesFor (raw.ptr.toNat + raw.len.toNat) }
             else
               { resource := invocationExtentResource
               , limit := P.maxInvocationBytes, demanded := raw.len.toNat })
      else
        .invalid
          (if Regions.AliasOk (descriptorOf h).aliasing (descriptorOf h).regions then
             { reason := .illegalView, offendingByteOffset := 48 }
           else
             { reason := .aliasViolation, offendingByteOffset := 14 })
    else
      .invalid { reason := .dimensionOverflow, offendingByteOffset := 16 }
  else
    .invalid { reason := .incompatibleTags, offendingByteOffset := 8 }

/-- Precedence class two, then the rest. -/
def classifyTags (P : Wasm.Profile) (raw : RawInvocationBody P)
    (h : RawHeader) : Classification P :=
  match tagUnsupported h with
  | some (off, t, _) =>
      .unsupported
        { feature := if off = 12 then arithmeticModeFeature else scalarKindFeature
        , offendingByteOffset := off
        , observedTag := t }
  | none => classifyDescriptor P raw h

/-- Precedence class one (header malformation), then the rest. -/
def classifyHeader (P : Wasm.Profile) (raw : RawInvocationBody P)
    (h : RawHeader) : Classification P :=
  match headerMalformed h with
  | some (r, off) => .invalid { reason := r, offendingByteOffset := off }
  | none => classifyTags P raw h

/-- **SPEC §8.3's total classifier.** -/
def classify (P : Wasm.Profile) (raw : RawInvocationBody P) : Classification P :=
  match decodeHeader raw.bytes with
  | none => .invalid { reason := .truncated, offendingByteOffset := raw.bytes.size }
  | some h => classifyHeader P raw h

/-- **SPEC §8.4**: the classifier is total. -/
theorem classify_total {P : Wasm.Profile} (raw : RawInvocationBody P) :
    ∃ classification, classify P raw = classification :=
  ⟨_, rfl⟩

/-- The classifier is a function, hence deterministic. -/
theorem classify_deterministic {P : Wasm.Profile} (raw : RawInvocationBody P)
    {c₁ c₂ : Classification P} (h₁ : classify P raw = c₁) (h₂ : classify P raw = c₂) :
    c₁ = c₂ := by
  rw [← h₁, ← h₂]

/-- **SPEC §8.3's exact-domain requirement.**

"Every byte-level well-formed descriptor whose views fit memory and whose tag
combination appears above SHALL classify as valid unless it violates the
explicitly enumerated alias or resource predicate."

The alias predicate is part of `DescriptorWellFormed`; the resource predicate is
`ResourceOk`. -/
theorem classifier_exact_domain {P : Wasm.Profile} (raw : RawInvocationBody P)
    (h : RawHeader)
    (hdec : decodeHeader raw.bytes = some h)
    (hmal : headerMalformed h = none)
    (htag : tagUnsupported h = none)
    (hcomp : headerCompatible h)
    (hdim : dimensionsRepresentable h)
    (hwf : DescriptorWellFormed (descriptorOf h) (windowOf P raw))
    (hres : ResourceOk P raw) :
    ∃ invocation, classify P raw = .valid invocation := by
  refine ⟨{ raw := raw, header := h
          , descriptor := ⟨descriptorOf h, windowOf P raw, hcomp, hwf⟩
          , descriptorMatches := rfl
          , windowMatches := rfl }, ?_⟩
  simp only [classify, hdec, classifyHeader, hmal, classifyTags, htag,
    classifyDescriptor, dif_pos hcomp, dif_pos hdim, dif_pos hwf, dif_pos hres]

/-- Conversely, a `valid` classification really does produce a descriptor that
satisfies every clause of the well-formedness predicate. -/
theorem valid_descriptor_wellFormed {P : Wasm.Profile} {raw : RawInvocationBody P}
    {inv : ValidInvocation P} (_h : classify P raw = .valid inv) :
    DescriptorWellFormed inv.descriptor.body inv.descriptor.window :=
  inv.descriptor.wellFormed

/-- A `valid` classification carries a compatible kind tuple. -/
theorem valid_kinds_compatible {P : Wasm.Profile} {raw : RawInvocationBody P}
    {inv : ValidInvocation P} (_h : classify P raw = .valid inv) :
    (inv.descriptor.body.arithmetic).Compatible :=
  inv.descriptor.kindsCompatible

/-! ## The canonical status-detail function (SPEC §8.3)

"`Gemm.StatusDetailOf` is a total canonical function, not a report callback.
Within the first failing precedence class it chooses the lowest header byte
offset, then the lexicographically least logical index." -/

/-- The expected literal of a header field, for the
"(expected literal, observed literal)" rule. -/
def expectedLiteral (offset : Nat) : Nat :=
  if offset = 0 then magicValue
  else if offset = 4 then abiVersion
  else if offset = 6 then abiHeaderSize
  else 0

/-- The observed literal of a header field. -/
def observedLiteral (h : RawHeader) (offset : Nat) : Nat :=
  if offset = 0 then h.magic.toNat
  else if offset = 4 then h.version.toNat
  else if offset = 6 then h.headerSize.toNat
  else if offset = 13 then h.transposeBits.toNat
  else if offset = 15 then h.reserved15.toNat
  else if offset = 176 then h.alphaPad.toNat
  else if offset = 192 then h.betaPad.toNat
  else if offset = 232 then h.reserved232.toNat
  else if offset = 240 then h.reserved240.toNat
  else h.reserved248.toNat

/-- The field code of a header-malformation offset: `version` for the version
field, `header` for everything else. -/
def headerFieldCode (offset : Nat) : FieldCode :=
  if offset = 4 then .version else .header

/-- The first required exclusive byte endpoint of a view, saturated at
`u64::MAX`, and the declared available endpoint. -/
def viewEndpoints (v : View) (s : Shape) (t : Bool) (w : Nat) : Nat × Nat :=
  ((satAddr (v.addrHi s t + (w : Int))).toNat, v.offset + v.byteLength)

/-- The first view that fails the layout predicate, with its header offset. -/
def firstFailingView (d : DescriptorBody) (M : MemoryWindow) : Nat × Nat × Nat :=
  if ¬ View.LayoutOk d.aView d.shapeA d.transposeA d.aKind.byteWidth M then
    (48, (viewEndpoints d.aView d.shapeA d.transposeA d.aKind.byteWidth).1,
         (viewEndpoints d.aView d.shapeA d.transposeA d.aKind.byteWidth).2)
  else if ¬ View.LayoutOk d.bView d.shapeB d.transposeB d.bKind.byteWidth M then
    (88, (viewEndpoints d.bView d.shapeB d.transposeB d.bKind.byteWidth).1,
         (viewEndpoints d.bView d.shapeB d.transposeB d.bKind.byteWidth).2)
  else if ¬ View.LayoutOk d.cView d.shapeC false d.cKind.byteWidth M then
    (128, (viewEndpoints d.cView d.shapeC false d.cKind.byteWidth).1,
          (viewEndpoints d.cView d.shapeC false d.cKind.byteWidth).2)
  else if ¬ DistinctDisjointBounded d.cView d.shapeC false d.cKind.byteWidth then
    (128, (viewEndpoints d.cView d.shapeC false d.cKind.byteWidth).1,
          (viewEndpoints d.cView d.shapeC false d.cKind.byteWidth).2)
  else if ¬ d.scratch.stop ≤ M.len then (200, d.scratch.stop, M.len)
  else if ¬ d.statusDetail.stop ≤ M.len then (216, d.statusDetail.stop, M.len)
  else if ¬ d.statusDetail.length = statusDetailBytes then
    (224, statusDetailBytes, d.statusDetail.length)
  else if ¬ headerBytes ≤ M.len then (0, headerBytes, M.len)
  else if ¬ d.alphaBits < 2 ^ (8 * d.cKind.byteWidth) then
    (168, 2 ^ (8 * d.cKind.byteWidth), d.alphaBits)
  else (184, 2 ^ (8 * d.cKind.byteWidth), d.betaBits)

/-- The status-detail record of a truncated raw input. -/
def truncationDetail (len : Nat) : StatusDetail where
  status := 1
  fieldCode := 1
  offendingOffset := sat64 len
  required := sat64 headerBytes
  available := sat64 len

/-- **SPEC §8.3's total canonical status-detail function.**

Its cascade is literally the classifier's cascade, so the returned record and
the returned status always agree. -/
def StatusDetailOf (P : Wasm.Profile) (raw : RawInvocationBody P) : StatusDetail :=
  match decodeHeader raw.bytes with
  | none => truncationDetail raw.bytes.size
  | some h =>
    match headerMalformed h with
    | some (_, off) =>
        { status := 1
        , fieldCode := UInt32.ofNat (headerFieldCode off).code
        , offendingOffset := sat64 off
        , required := sat64 (expectedLiteral off)
        , available := sat64 (observedLiteral h off) }
    | none =>
      match tagUnsupported h with
      | some (off, t, bitset) =>
          { status := 2
          , fieldCode := UInt32.ofNat (if off = 12 then FieldCode.arithmeticMode.code
                                       else FieldCode.kind.code)
          , offendingOffset := sat64 off
          , required := sat64 bitset
          , available := sat64 t }
      | none =>
        if ¬ headerCompatible h then
          { status := 1
          , fieldCode := UInt32.ofNat FieldCode.kind.code
          , offendingOffset := sat64 8
          , required := sat64 (compatibleRowBitset (descriptorOf h).mode
              (descriptorOf h).aKind (descriptorOf h).bKind (descriptorOf h).cKind)
          , available := sat64 (packedKindModeTuple h.aTag.toNat h.bTag.toNat
              h.cTag.toNat h.accTag.toNat h.modeTag.toNat) }
        else if ¬ dimensionsRepresentable h then
          { status := 1
          , fieldCode := UInt32.ofNat FieldCode.dimension.code
          , offendingOffset := sat64 16
          , required := sat64 (2 ^ 64)
          , available := sat64 (h.batch.toNat * h.m.toNat * h.n.toNat) }
        else if ¬ DescriptorWellFormed (descriptorOf h) (windowOf P raw) then
          if Regions.AliasOk (descriptorOf h).aliasing (descriptorOf h).regions then
            let f := firstFailingView (descriptorOf h) (windowOf P raw)
            { status := 1
            , fieldCode := UInt32.ofNat FieldCode.view.code
            , offendingOffset := sat64 f.1
            , required := sat64 f.2.1
            , available := sat64 f.2.2 }
          else
            { status := 1
            , fieldCode := UInt32.ofNat FieldCode.alias.code
            , offendingOffset := sat64 14
            , required := 0
            , available := sat64 (Regions.aliasOverlapCount (descriptorOf h).aliasing
                (descriptorOf h).regions) }
        else if _hres : ¬ ResourceOk P raw then
          if raw.len.toNat ≤ P.maxInvocationBytes then
            { status := 3
            , fieldCode := UInt32.ofNat FieldCode.resource.code
            , offendingOffset := 0
            , required := sat64 (Wasm.pagesFor (raw.ptr.toNat + raw.len.toNat))
            , available := sat64 P.maxPages }
          else
            { status := 3
            , fieldCode := UInt32.ofNat FieldCode.resource.code
            , offendingOffset := 0
            , required := sat64 raw.len.toNat
            , available := sat64 P.maxInvocationBytes }
        else StatusDetail.zero

/-- `StatusDetailOf` is a total function of the raw bytes. -/
theorem statusDetailOf_total {P : Wasm.Profile} (raw : RawInvocationBody P) :
    ∃ record, StatusDetailOf P raw = record :=
  ⟨_, rfl⟩

/-- `StatusDetailOf` is deterministic: "every classified raw input determines one
record byte sequence". -/
theorem statusDetailOf_deterministic {P : Wasm.Profile} (raw : RawInvocationBody P)
    {r₁ r₂ : StatusDetail}
    (h₁ : StatusDetailOf P raw = r₁) (h₂ : StatusDetailOf P raw = r₂) : r₁ = r₂ := by
  rw [← h₁, ← h₂]

/-- Two encoders of the record cannot disagree: the record determines its bytes. -/
theorem statusDetailOf_bytes_deterministic {P : Wasm.Profile}
    (raw : RawInvocationBody P) :
    ∀ b₁ b₂ : ByteArray,
      b₁ = encodeStatusDetail (StatusDetailOf P raw) →
      b₂ = encodeStatusDetail (StatusDetailOf P raw) → b₁ = b₂ := by
  intro b₁ b₂ h₁ h₂; rw [h₁, h₂]

/-- A valid classification writes the all-zero record: "Success writes an
all-zero record." -/
theorem statusDetailOf_valid {P : Wasm.Profile} {raw : RawInvocationBody P}
    {h : RawHeader}
    (hdec : decodeHeader raw.bytes = some h)
    (hmal : headerMalformed h = none)
    (htag : tagUnsupported h = none)
    (hcomp : headerCompatible h)
    (hdim : dimensionsRepresentable h)
    (hwf : DescriptorWellFormed (descriptorOf h) (windowOf P raw))
    (hres : ResourceOk P raw) :
    StatusDetailOf P raw = StatusDetail.zero := by
  simp only [StatusDetailOf, hdec, hmal, htag, hcomp, hdim, hwf, hres,
    not_true_eq_false, if_false, dite_false]

/-- A truncated raw input reports status `1` with the header field code and the
`(256, len)` pair SPEC §8.3 prescribes. -/
theorem statusDetailOf_truncated {P : Wasm.Profile} {raw : RawInvocationBody P}
    (hdec : decodeHeader raw.bytes = none) :
    StatusDetailOf P raw = truncationDetail raw.bytes.size := by
  simp only [StatusDetailOf, hdec]

/-- Every status code the canonical record can carry is one of the six
sanctioned results. -/
theorem statusDetailOf_status_sanctioned {P : Wasm.Profile}
    (raw : RawInvocationBody P) :
    (StatusDetailOf P raw).status.toNat = 0 ∨ (StatusDetailOf P raw).status.toNat = 1 ∨
      (StatusDetailOf P raw).status.toNat = 2 ∨ (StatusDetailOf P raw).status.toNat = 3 := by
  unfold StatusDetailOf
  split
  · exact Or.inr (Or.inl rfl)
  · split
    · exact Or.inr (Or.inl rfl)
    · split
      · exact Or.inr (Or.inr (Or.inl rfl))
      · split
        · exact Or.inr (Or.inl rfl)
        · split
          · exact Or.inr (Or.inl rfl)
          · split
            · split
              · exact Or.inr (Or.inl rfl)
              · exact Or.inr (Or.inl rfl)
            · split
              · split
                · exact Or.inr (Or.inr (Or.inr rfl))
                · exact Or.inr (Or.inr (Or.inr rfl))
              · exact Or.inl rfl

end WasmGemmGnaf.Gemm

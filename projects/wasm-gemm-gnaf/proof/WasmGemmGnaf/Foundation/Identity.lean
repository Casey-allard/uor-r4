import WasmGemmGnaf.Foundation.Canonical
set_option autoImplicit false

/-!
# Foundation: structural identity (SPEC §6.2)

`Identity` is the only way to name an object.  `Identity_eq_iff` is the load
bearing fact: two identities under one frozen schema are equal exactly when the
bodies are equal, so identity comparison *is* structural comparison and no proof
ever needs to assume collision resistance of a digest.
-/

namespace WasmGemmGnaf.Foundation

/-- The canonical identity of `body` under `schema` (SPEC §6.2). -/
def Identity
    {α : Type} (schema : CanonicalSchema α) (body : α) : ObjectId α :=
  { schemaVersion := schema.version
    domain := schema.domain
    typeTag := schema.typeTag
    canonicalBodyBytes := schema.encode body }

theorem Identity_eq_iff
    {α : Type} (schema : CanonicalSchema α) {left right : α} :
    Identity schema left = Identity schema right ↔ left = right := by
  constructor
  · intro h
    exact schema.encode_injective
      (congrArg ObjectId.canonicalBodyBytes h)
  · intro h
    cases h
    rfl

theorem Identity_injective {α : Type} (schema : CanonicalSchema α) :
    Function.Injective (Identity schema) :=
  fun _ _ h => (Identity_eq_iff schema).mp h

@[simp] theorem Identity_schemaVersion {α : Type} (schema : CanonicalSchema α)
    (body : α) : (Identity schema body).schemaVersion = schema.version := rfl

@[simp] theorem Identity_domain {α : Type} (schema : CanonicalSchema α)
    (body : α) : (Identity schema body).domain = schema.domain := rfl

@[simp] theorem Identity_typeTag {α : Type} (schema : CanonicalSchema α)
    (body : α) : (Identity schema body).typeTag = schema.typeTag := rfl

@[simp] theorem Identity_canonicalBodyBytes {α : Type} (schema : CanonicalSchema α)
    (body : α) : (Identity schema body).canonicalBodyBytes = schema.encode body := rfl

/-- The type-erased canonical object identifier (SPEC §6.2). -/
structure CanonicalObjectId where
  schemaVersion : Nat
  domain : CanonicalDomainTag
  typeTag : ByteArray
  canonicalBodyBytes : ByteArray
  deriving DecidableEq

/-- Erase a typed identifier.  Because type tags are structural and unique per
registered type (see `Foundation/SchemaRegistry.lean`), erasure cannot merge two
registered types. -/
def CanonicalObjectId.ofTyped
    {α : Type}
    (id : ObjectId α) : CanonicalObjectId :=
  { schemaVersion := id.schemaVersion
    domain := id.domain
    typeTag := id.typeTag
    canonicalBodyBytes := id.canonicalBodyBytes }

theorem CanonicalObjectId.ofTyped_injective {α : Type} :
    Function.Injective (CanonicalObjectId.ofTyped (α := α)) := by
  intro a b h
  cases a; cases b
  simp only [CanonicalObjectId.ofTyped, CanonicalObjectId.mk.injEq] at h
  simp only [ObjectId.mk.injEq]
  exact h

/-- Erasure preserves the identity equation of SPEC §6.2: within one frozen
schema, equal erased identifiers still force equal bodies. -/
theorem CanonicalObjectId.ofTyped_Identity_eq_iff
    {α : Type} (schema : CanonicalSchema α) {left right : α} :
    CanonicalObjectId.ofTyped (Identity schema left) =
      CanonicalObjectId.ofTyped (Identity schema right) ↔ left = right := by
  constructor
  · intro h
    exact schema.encode_injective
      (congrArg CanonicalObjectId.canonicalBodyBytes h)
  · intro h
    cases h
    rfl

namespace CanonicalObjectId

/-- Canonical prefix-free byte encoding of an erased identifier.  Identifiers
occur inside other canonical bodies (SPEC §6.2's `Gemm.ProblemId` carries the
profile identity), so they need an encoder of their own. -/
def bytes (id : CanonicalObjectId) : List UInt8 :=
  Bytes.natBytes id.schemaVersion ++
    CanonicalDomainTag.bytes id.domain ++
    Bytes.byteArrayBytes id.typeTag ++
    Bytes.byteArrayBytes id.canonicalBodyBytes

theorem bytes_prefixFree : Bytes.PrefixFree bytes := by
  intro a b r s h
  cases a with
  | mk av ad at_ ab =>
  cases b with
  | mk bv bd bt bb =>
    simp only [bytes, List.append_assoc] at h
    obtain ⟨hv, h1⟩ := Bytes.natBytes_prefixFree _ _ _ _ h
    obtain ⟨hd, h2⟩ := CanonicalDomainTag.bytes_prefixFree _ _ _ _ h1
    obtain ⟨ht, h3⟩ := Bytes.byteArrayBytes_prefixFree _ _ _ _ h2
    obtain ⟨hb, h4⟩ := Bytes.byteArrayBytes_prefixFree _ _ _ _ h3
    exact ⟨by rw [hv, hd, ht, hb], h4⟩

theorem bytes_injective : Function.Injective bytes :=
  bytes_prefixFree.injective

/-- Canonical byte string of an erased identifier. -/
def canonicalBytes (id : CanonicalObjectId) : ByteArray := Bytes.pack id.bytes

theorem canonicalBytes_injective : Function.Injective canonicalBytes :=
  Bytes.packed_injective bytes_injective

/-- The canonical total order on identifiers, used to canonicalise sets and
maps of identifiers. -/
def le (a b : CanonicalObjectId) : Prop :=
  UnsignedLexicographicLE a.bytes b.bytes

theorem le_refl (a : CanonicalObjectId) : le a a :=
  UnsignedLexicographicLE.refl _

theorem le_trans {a b c : CanonicalObjectId} (hab : le a b) (hbc : le b c) : le a c :=
  UnsignedLexicographicLE.trans _ _ _ hab hbc

theorem le_antisymm {a b : CanonicalObjectId} (hab : le a b) (hba : le b a) : a = b :=
  bytes_injective (UnsignedLexicographicLE.antisymm _ _ hab hba)

theorem le_total (a b : CanonicalObjectId) : le a b ∨ le b a :=
  UnsignedLexicographicLE.total _ _

instance instDecidableLe (a b : CanonicalObjectId) : Decidable (le a b) :=
  instDecidableUnsignedLexicographicLE _ _

end CanonicalObjectId

/-! ## Scope identity abbreviations (SPEC §6.2) -/

abbrev ProfileId := CanonicalObjectId
abbrev ProblemId := CanonicalObjectId
abbrev ObjectiveId := CanonicalObjectId
abbrev StateIdentity := CanonicalObjectId
abbrev FeatureId := CanonicalObjectId
abbrev DeltaId := CanonicalObjectId
abbrev RequestId := CanonicalObjectId
abbrev QueryResultId := CanonicalObjectId

end WasmGemmGnaf.Foundation

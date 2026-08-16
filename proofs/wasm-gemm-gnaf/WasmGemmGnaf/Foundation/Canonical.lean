import WasmGemmGnaf.Foundation.Bytes
import WasmGemmGnaf.Foundation.Order
set_option autoImplicit false

/-!
# Foundation: canonical schemas (SPEC §6.2)

Proof identity is structural.  A `CanonicalSchema` bundles a version, a domain
tag, a nonempty structural type tag and an encoder *together with the proof that
the encoder is injective* — the proof is a field of the schema, so no schema can
be built without discharging it.
-/

namespace WasmGemmGnaf.Foundation

/-- The closed set of canonical identity domains (SPEC §6.2). -/
inductive CanonicalDomainTag
  | wasmProfile | gemmProblem | costObjective | atlasState
  | atlasSealCheck | delta | request | queryResult
  | manifest | authority | artifact | generic
  deriving DecidableEq, Repr, Inhabited

namespace CanonicalDomainTag

/-- Structural index of a domain tag; the canonical one-byte encoding. -/
def index : CanonicalDomainTag → Nat
  | wasmProfile => 0
  | gemmProblem => 1
  | costObjective => 2
  | atlasState => 3
  | atlasSealCheck => 4
  | delta => 5
  | request => 6
  | queryResult => 7
  | manifest => 8
  | authority => 9
  | artifact => 10
  | generic => 11

theorem index_injective : Function.Injective index := by
  intro a b h
  cases a <;> cases b <;> simp_all [index]

/-- Canonical one-byte encoding of a domain tag. -/
def bytes (d : CanonicalDomainTag) : List UInt8 := [UInt8.ofNat d.index]

theorem bytes_injective : Function.Injective bytes := by
  intro a b h
  simp only [bytes, List.cons.injEq, and_true] at h
  rw [Bytes.uint8_ofNat_eq_iff] at h
  apply index_injective
  cases a <;> cases b <;> simp_all [index]

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Bytes.prefixFree_of_constLength bytes 1 (fun d => by cases d <;> rfl) bytes_injective

/-- The complete finite enumeration of domain tags. -/
def all : List CanonicalDomainTag :=
  [wasmProfile, gemmProblem, costObjective, atlasState, atlasSealCheck, delta,
   request, queryResult, manifest, authority, artifact, generic]

theorem mem_all (d : CanonicalDomainTag) : d ∈ all := by
  cases d <;> simp [all]

theorem all_nodup : all.Nodup := by decide

end CanonicalDomainTag

/-- A frozen canonical schema: version, domain, structural type tag and an
injective encoder.  SPEC §6.2 forbids treating schemas as typeclass arguments;
every identity call names one frozen schema value. -/
structure CanonicalSchema (α : Type) where
  version : Nat
  domain : CanonicalDomainTag
  typeTag : ByteArray
  encode : α → ByteArray
  encode_injective : Function.Injective encode
  typeTagNonempty : typeTag.size > 0

namespace CanonicalSchema

theorem encode_eq_iff {α : Type} (schema : CanonicalSchema α) {a b : α} :
    schema.encode a = schema.encode b ↔ a = b :=
  ⟨fun h => schema.encode_injective h, fun h => by rw [h]⟩

/-- Build a schema from a prefix-free byte-list encoder.  This is the intended
construction path: prefix-freeness is what makes structural composition (§6.2's
product and sum schemas) injective. -/
def ofPrefixFree {α : Type} (version : Nat) (domain : CanonicalDomainTag)
    (typeTag : ByteArray) (typeTagNonempty : typeTag.size > 0)
    (enc : α → List UInt8) (henc : Bytes.PrefixFree enc) : CanonicalSchema α where
  version := version
  domain := domain
  typeTag := typeTag
  encode := Bytes.packed enc
  encode_injective := Bytes.packed_injective_of_prefixFree henc
  typeTagNonempty := typeTagNonempty

@[simp] theorem ofPrefixFree_encode {α : Type} (version : Nat)
    (domain : CanonicalDomainTag) (typeTag : ByteArray) (h : typeTag.size > 0)
    (enc : α → List UInt8) (henc : Bytes.PrefixFree enc) (a : α) :
    (ofPrefixFree version domain typeTag h enc henc).encode a = Bytes.pack (enc a) :=
  rfl

@[simp] theorem ofPrefixFree_version {α : Type} (version : Nat)
    (domain : CanonicalDomainTag) (typeTag : ByteArray) (h : typeTag.size > 0)
    (enc : α → List UInt8) (henc : Bytes.PrefixFree enc) :
    (ofPrefixFree version domain typeTag h enc henc).version = version :=
  rfl

@[simp] theorem ofPrefixFree_domain {α : Type} (version : Nat)
    (domain : CanonicalDomainTag) (typeTag : ByteArray) (h : typeTag.size > 0)
    (enc : α → List UInt8) (henc : Bytes.PrefixFree enc) :
    (ofPrefixFree version domain typeTag h enc henc).domain = domain :=
  rfl

@[simp] theorem ofPrefixFree_typeTag {α : Type} (version : Nat)
    (domain : CanonicalDomainTag) (typeTag : ByteArray) (h : typeTag.size > 0)
    (enc : α → List UInt8) (henc : Bytes.PrefixFree enc) :
    (ofPrefixFree version domain typeTag h enc henc).typeTag = typeTag :=
  rfl

/-- The framed (length-prefixed) form of a schema's encoder is prefix-free, so
schema encoders compose structurally. -/
theorem framed_prefixFree {α : Type} (schema : CanonicalSchema α) :
    Bytes.PrefixFree (Bytes.framed schema.encode) :=
  Bytes.framed_prefixFree schema.encode_injective

end CanonicalSchema

/-- A typed canonical object identifier (SPEC §6.2). -/
structure ObjectId (α : Type) where
  schemaVersion : Nat
  domain : CanonicalDomainTag
  typeTag : ByteArray
  canonicalBodyBytes : ByteArray
  deriving DecidableEq

/-! ## Structural type tags

Product and sum schemas derive their type tags from their component tags, so
erasing a typed identifier to `CanonicalObjectId` cannot merge two registered
types. -/

namespace TypeTag

/-- Leaf type tag: a versioned, length-prefixed structural name. -/
def leaf (name : List UInt8) : ByteArray :=
  Bytes.pack (0 :: Bytes.stringBytes name)

/-- Product type tag, derived structurally from the component tags. -/
def product (left right : ByteArray) : ByteArray :=
  Bytes.pack (1 :: (Bytes.byteArrayBytes left ++ Bytes.byteArrayBytes right))

/-- Sum type tag, derived structurally from the component tags. -/
def sum (left right : ByteArray) : ByteArray :=
  Bytes.pack (2 :: (Bytes.byteArrayBytes left ++ Bytes.byteArrayBytes right))

/-- List type tag, derived structurally from the element tag. -/
def list (element : ByteArray) : ByteArray :=
  Bytes.pack (3 :: Bytes.byteArrayBytes element)

theorem leaf_size_pos (name : List UInt8) : (leaf name).size > 0 := by
  simp [leaf]

theorem product_size_pos (left right : ByteArray) : (product left right).size > 0 := by
  simp [product]

theorem sum_size_pos (left right : ByteArray) : (sum left right).size > 0 := by
  simp [sum]

theorem list_size_pos (element : ByteArray) : (list element).size > 0 := by
  simp [list]

theorem leaf_injective : Function.Injective leaf := by
  intro a b h
  have h' := Bytes.pack_injective h
  simp only [List.cons.injEq, true_and] at h'
  exact Bytes.stringBytes_injective h'

theorem product_injective {a b c d : ByteArray}
    (h : product a b = product c d) : a = c ∧ b = d := by
  have h' := Bytes.pack_injective h
  simp only [List.cons.injEq, true_and] at h'
  obtain ⟨h1, h2⟩ := Bytes.byteArrayBytes_prefixFree _ _ _ _ h'
  exact ⟨h1, Bytes.byteArrayBytes_injective h2⟩

theorem sum_injective {a b c d : ByteArray}
    (h : sum a b = sum c d) : a = c ∧ b = d := by
  have h' := Bytes.pack_injective h
  simp only [List.cons.injEq, true_and] at h'
  obtain ⟨h1, h2⟩ := Bytes.byteArrayBytes_prefixFree _ _ _ _ h'
  exact ⟨h1, Bytes.byteArrayBytes_injective h2⟩

theorem list_injective : Function.Injective list := by
  intro a b h
  have h' := Bytes.pack_injective h
  simp only [List.cons.injEq, true_and] at h'
  exact Bytes.byteArrayBytes_injective h'

/-- Product and sum tags are never confused with each other. -/
theorem product_ne_sum (a b c d : ByteArray) : product a b ≠ sum c d := by
  intro h
  have h' := Bytes.pack_injective h
  simp only [List.cons.injEq] at h'
  exact absurd h'.1 (by decide)

/-- Leaf tags are never confused with product tags. -/
theorem leaf_ne_product (name : List UInt8) (a b : ByteArray) :
    leaf name ≠ product a b := by
  intro h
  have h' := Bytes.pack_injective h
  simp only [List.cons.injEq] at h'
  exact absurd h'.1 (by decide)

end TypeTag

end WasmGemmGnaf.Foundation

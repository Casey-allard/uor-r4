import WasmGemmGnaf.Foundation.Identity
set_option autoImplicit false

/-!
# Foundation: the finite schema registry (SPEC §6.2)

SPEC §6.2 mandates a finite registry of every schema used by the release, and
requires a proof that equal `(schemaVersion, domain, typeTag)` triples identify
the same registered body type and encoder.

That proof is `SchemaRegistry.key_determines_entry`; it follows from the
registry's well-formedness field `distinctKeys`, which is a *checkable syntactic*
side condition on the entry list, not a stored conclusion.

The file also builds the structural product/sum/list schema combinators, whose
type tags derive from their component tags (`TypeTag.product`, `TypeTag.sum`,
`TypeTag.list`), so a composite type can never collide with one of its
components or with a differently shaped composite.
-/

namespace WasmGemmGnaf.Foundation

/-- The registry key of a schema: exactly the triple SPEC §6.2 names. -/
structure SchemaKey where
  schemaVersion : Nat
  domain : CanonicalDomainTag
  typeTag : ByteArray
  deriving DecidableEq

/-- One registered schema, together with the body type it encodes. -/
structure SchemaEntry where
  Body : Type
  schema : CanonicalSchema Body

/-- The registry key of an entry. -/
def SchemaEntry.key (e : SchemaEntry) : SchemaKey :=
  { schemaVersion := e.schema.version
    domain := e.schema.domain
    typeTag := e.schema.typeTag }

/-- The identity of `body` under a registered entry, erased. -/
def SchemaEntry.identity (e : SchemaEntry) (body : e.Body) : CanonicalObjectId :=
  CanonicalObjectId.ofTyped (Identity e.schema body)

theorem SchemaEntry.identity_eq_iff (e : SchemaEntry) {a b : e.Body} :
    e.identity a = e.identity b ↔ a = b :=
  CanonicalObjectId.ofTyped_Identity_eq_iff e.schema

/-- The finite registry of every schema used by the release.

`distinctKeys` is the well-formedness condition: no two entries share a
`(schemaVersion, domain, typeTag)` triple. -/
structure SchemaRegistry where
  entries : List SchemaEntry
  distinctKeys : entries.Pairwise (fun a b => a.key ≠ b.key)

namespace SchemaRegistry

theorem pairwise_key_unique : ∀ (l : List SchemaEntry),
    l.Pairwise (fun a b => a.key ≠ b.key) →
    ∀ (e₁ e₂ : SchemaEntry), e₁ ∈ l → e₂ ∈ l → e₁.key = e₂.key → e₁ = e₂ := by
  intro l
  induction l with
  | nil => intro _ e₁ e₂ h1 _ _; simp at h1
  | cons x xs ih =>
    intro hp e₁ e₂ h1 h2 hk
    rw [List.pairwise_cons] at hp
    obtain ⟨hx, hrest⟩ := hp
    rcases List.mem_cons.mp h1 with rfl | h1'
    · rcases List.mem_cons.mp h2 with rfl | h2'
      · rfl
      · exact absurd hk (hx e₂ h2')
    · rcases List.mem_cons.mp h2 with rfl | h2'
      · exact absurd hk.symm (hx e₁ h1')
      · exact ih hrest e₁ e₂ h1' h2' hk

/-- **SPEC §6.2**: equal `(schemaVersion, domain, typeTag)` triples identify the
same registered entry. -/
theorem key_determines_entry (R : SchemaRegistry) {e₁ e₂ : SchemaEntry}
    (h₁ : e₁ ∈ R.entries) (h₂ : e₂ ∈ R.entries) (hk : e₁.key = e₂.key) :
    e₁ = e₂ :=
  pairwise_key_unique R.entries R.distinctKeys e₁ e₂ h₁ h₂ hk

/-- ... hence the same body type. -/
theorem key_determines_body (R : SchemaRegistry) {e₁ e₂ : SchemaEntry}
    (h₁ : e₁ ∈ R.entries) (h₂ : e₂ ∈ R.entries) (hk : e₁.key = e₂.key) :
    e₁.Body = e₂.Body := by
  rw [key_determines_entry R h₁ h₂ hk]

/-- ... and the same schema, hence the same encoder. -/
theorem key_determines_schema (R : SchemaRegistry) {e₁ e₂ : SchemaEntry}
    (h₁ : e₁ ∈ R.entries) (h₂ : e₂ ∈ R.entries) (hk : e₁.key = e₂.key) :
    HEq e₁.schema e₂.schema := by
  have h := key_determines_entry R h₁ h₂ hk
  subst h
  exact HEq.rfl

theorem key_determines_encoder (R : SchemaRegistry) {e₁ e₂ : SchemaEntry}
    (h₁ : e₁ ∈ R.entries) (h₂ : e₂ ∈ R.entries) (hk : e₁.key = e₂.key) :
    HEq e₁.schema.encode e₂.schema.encode := by
  have h := key_determines_entry R h₁ h₂ hk
  subst h
  exact HEq.rfl

/-- Registry lookup by key. -/
def find? (R : SchemaRegistry) (k : SchemaKey) : Option SchemaEntry :=
  R.entries.find? (fun e => decide (e.key = k))

theorem find?_mem {R : SchemaRegistry} {k : SchemaKey} {e : SchemaEntry}
    (h : R.find? k = some e) : e ∈ R.entries :=
  List.mem_of_find?_eq_some h

theorem find?_key {R : SchemaRegistry} {k : SchemaKey} {e : SchemaEntry}
    (h : R.find? k = some e) : e.key = k := by
  have := List.find?_some h
  simpa using this

/-- Lookup is *the* registered entry: nothing else in the registry carries that
key. -/
theorem find?_unique {R : SchemaRegistry} {k : SchemaKey} {e e' : SchemaEntry}
    (h : R.find? k = some e) (he' : e' ∈ R.entries) (hk : e'.key = k) :
    e' = e :=
  key_determines_entry R he' (find?_mem h) (by rw [hk, find?_key h])

/-- A key that is not found is carried by no registry entry. -/
theorem find?_none {R : SchemaRegistry} {k : SchemaKey}
    (h : R.find? k = none) : ∀ e ∈ R.entries, e.key ≠ k := by
  intro e he hk
  have := List.find?_eq_none.mp h e he
  simp [hk] at this

end SchemaRegistry

/-! ## Structural schema combinators

A composite schema's type tag is built from its components' type tags, and its
encoder is the length-framed concatenation of the component encoders.  Both the
tag construction and the encoder are injective, proved. -/

namespace CanonicalSchema

/-- Product schema.  The type tag is `TypeTag.product` of the component tags. -/
def product {α β : Type} (version : Nat) (domain : CanonicalDomainTag)
    (left : CanonicalSchema α) (right : CanonicalSchema β) :
    CanonicalSchema (α × β) :=
  ofPrefixFree version domain
    (TypeTag.product left.typeTag right.typeTag)
    (TypeTag.product_size_pos _ _)
    (Bytes.pairBytes (Bytes.framed left.encode) (Bytes.framed right.encode))
    (Bytes.pairBytes_prefixFree
      (Bytes.framed_prefixFree left.encode_injective)
      (Bytes.framed_prefixFree right.encode_injective))

@[simp] theorem product_typeTag {α β : Type} (version : Nat)
    (domain : CanonicalDomainTag) (left : CanonicalSchema α)
    (right : CanonicalSchema β) :
    (product version domain left right).typeTag =
      TypeTag.product left.typeTag right.typeTag := rfl

theorem product_encode_injective {α β : Type} (version : Nat)
    (domain : CanonicalDomainTag) (left : CanonicalSchema α)
    (right : CanonicalSchema β) :
    Function.Injective (product version domain left right).encode :=
  (product version domain left right).encode_injective

/-- Sum schema.  The type tag is `TypeTag.sum` of the component tags. -/
def sum {α β : Type} (version : Nat) (domain : CanonicalDomainTag)
    (left : CanonicalSchema α) (right : CanonicalSchema β) :
    CanonicalSchema (α ⊕ β) :=
  ofPrefixFree version domain
    (TypeTag.sum left.typeTag right.typeTag)
    (TypeTag.sum_size_pos _ _)
    (Bytes.sumBytes (Bytes.framed left.encode) (Bytes.framed right.encode))
    (Bytes.sumBytes_prefixFree
      (Bytes.framed_prefixFree left.encode_injective)
      (Bytes.framed_prefixFree right.encode_injective))

@[simp] theorem sum_typeTag {α β : Type} (version : Nat)
    (domain : CanonicalDomainTag) (left : CanonicalSchema α)
    (right : CanonicalSchema β) :
    (sum version domain left right).typeTag =
      TypeTag.sum left.typeTag right.typeTag := rfl

theorem sum_encode_injective {α β : Type} (version : Nat)
    (domain : CanonicalDomainTag) (left : CanonicalSchema α)
    (right : CanonicalSchema β) :
    Function.Injective (sum version domain left right).encode :=
  (sum version domain left right).encode_injective

/-- List schema.  The type tag is `TypeTag.list` of the element tag. -/
def list {α : Type} (version : Nat) (domain : CanonicalDomainTag)
    (element : CanonicalSchema α) : CanonicalSchema (List α) :=
  ofPrefixFree version domain
    (TypeTag.list element.typeTag)
    (TypeTag.list_size_pos _)
    (Bytes.listBytes (Bytes.framed element.encode))
    (Bytes.listBytes_prefixFree (Bytes.framed_prefixFree element.encode_injective))

@[simp] theorem list_typeTag {α : Type} (version : Nat)
    (domain : CanonicalDomainTag) (element : CanonicalSchema α) :
    (list version domain element).typeTag = TypeTag.list element.typeTag := rfl

theorem list_encode_injective {α : Type} (version : Nat)
    (domain : CanonicalDomainTag) (element : CanonicalSchema α) :
    Function.Injective (list version domain element).encode :=
  (list version domain element).encode_injective

/-- Optional schema. -/
def option {α : Type} (version : Nat) (domain : CanonicalDomainTag)
    (element : CanonicalSchema α) : CanonicalSchema (Option α) :=
  ofPrefixFree version domain
    (TypeTag.list element.typeTag)
    (TypeTag.list_size_pos _)
    (Bytes.optionBytes (Bytes.framed element.encode))
    (Bytes.optionBytes_prefixFree (Bytes.framed_prefixFree element.encode_injective))

/-- Structural type tags of composites determine the component tags. -/
theorem product_typeTag_inj {α β γ δ : Type} {v₁ v₂ : Nat}
    {d₁ d₂ : CanonicalDomainTag}
    {a : CanonicalSchema α} {b : CanonicalSchema β}
    {c : CanonicalSchema γ} {d : CanonicalSchema δ}
    (h : (product v₁ d₁ a b).typeTag = (product v₂ d₂ c d).typeTag) :
    a.typeTag = c.typeTag ∧ b.typeTag = d.typeTag :=
  TypeTag.product_injective h

theorem sum_typeTag_inj {α β γ δ : Type} {v₁ v₂ : Nat}
    {d₁ d₂ : CanonicalDomainTag}
    {a : CanonicalSchema α} {b : CanonicalSchema β}
    {c : CanonicalSchema γ} {d : CanonicalSchema δ}
    (h : (sum v₁ d₁ a b).typeTag = (sum v₂ d₂ c d).typeTag) :
    a.typeTag = c.typeTag ∧ b.typeTag = d.typeTag :=
  TypeTag.sum_injective h

/-- A product schema's type tag is never a sum schema's type tag, so a product
and a sum are never registered under the same key. -/
theorem product_typeTag_ne_sum_typeTag {α β γ δ : Type} {v₁ v₂ : Nat}
    {d₁ d₂ : CanonicalDomainTag}
    {a : CanonicalSchema α} {b : CanonicalSchema β}
    {c : CanonicalSchema γ} {d : CanonicalSchema δ} :
    (product v₁ d₁ a b).typeTag ≠ (sum v₂ d₂ c d).typeTag :=
  TypeTag.product_ne_sum _ _ _ _

/-! ## Leaf schemas for the primitive carriers -/

/-- Leaf schema for natural numbers. -/
def natSchema (version : Nat) (domain : CanonicalDomainTag)
    (name : List UInt8) : CanonicalSchema Nat :=
  ofPrefixFree version domain (TypeTag.leaf name) (TypeTag.leaf_size_pos _)
    Bytes.natBytes Bytes.natBytes_prefixFree

/-- Leaf schema for byte strings. -/
def byteArraySchema (version : Nat) (domain : CanonicalDomainTag)
    (name : List UInt8) : CanonicalSchema ByteArray :=
  ofPrefixFree version domain (TypeTag.leaf name) (TypeTag.leaf_size_pos _)
    Bytes.byteArrayBytes Bytes.byteArrayBytes_prefixFree

/-- Leaf schema for booleans. -/
def boolSchema (version : Nat) (domain : CanonicalDomainTag)
    (name : List UInt8) : CanonicalSchema Bool :=
  ofPrefixFree version domain (TypeTag.leaf name) (TypeTag.leaf_size_pos _)
    Bytes.boolBytes Bytes.boolBytes_prefixFree

/-- Leaf schema for erased canonical identifiers. -/
def objectIdSchema (version : Nat) (domain : CanonicalDomainTag)
    (name : List UInt8) : CanonicalSchema CanonicalObjectId :=
  ofPrefixFree version domain (TypeTag.leaf name) (TypeTag.leaf_size_pos _)
    CanonicalObjectId.bytes CanonicalObjectId.bytes_prefixFree

end CanonicalSchema

end WasmGemmGnaf.Foundation

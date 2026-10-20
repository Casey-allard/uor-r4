/-
  Wasm/Revision.lean --- the pinned WebAssembly Core revision identity.

  Normative source: SPEC.md section 4 (pinned authority) and section 7.2
  (released portable profile).  The revision is a *checked literal*: the
  commit is written out here and every property of it that the release gate
  needs is proved, never assumed.

  This file also carries the small byte-encoding helpers that the rest of the
  `Wasm` layer needs on top of `Foundation/Bytes.lean` (strings and lists of
  naturals).  They live here because `Revision.lean` is the bottom of the
  `Wasm` import order.

  Every declaration in this file is proved.  Nothing is assumed.
-/
import WasmGemmGnaf.Foundation.Identity

set_option autoImplicit false

namespace WasmGemmGnaf.Wasm

open WasmGemmGnaf.Foundation

/-! ## Encoding helpers

`Foundation/Bytes.lean` supplies prefix-free encoders for `Nat`, `ByteArray`,
pairs, sums, lists and options.  The `Wasm` layer additionally needs `String`
(module, export and rule-identifier names) and `List Nat` (first-order records
whose fields are all counts). -/

namespace Enc

/-- `String` is a structure over its UTF-8 `ByteArray` together with a
proof-irrelevant validity field, so the byte array determines the string. -/
theorem toByteArray_injective : Function.Injective String.toByteArray := by
  intro s t h
  cases s
  cases t
  cases h
  rfl

/-- Prefix-free encoding of a `String`: the length-prefixed UTF-8 bytes. -/
def stringBytes (s : String) : List UInt8 :=
  Bytes.byteArrayBytes s.toByteArray

theorem stringBytes_prefixFree : Bytes.PrefixFree stringBytes :=
  Bytes.byteArrayBytes_prefixFree.comp toByteArray_injective

theorem stringBytes_injective : Function.Injective stringBytes :=
  stringBytes_prefixFree.injective

/-- Prefix-free encoding of a list of naturals: the shape used to encode a
first-order record all of whose fields are counts. -/
def natsBytes (l : List Nat) : List UInt8 :=
  Bytes.listBytes Bytes.natBytes l

theorem natsBytes_prefixFree : Bytes.PrefixFree natsBytes :=
  Bytes.listBytes_prefixFree Bytes.natBytes_prefixFree

theorem natsBytes_injective : Function.Injective natsBytes :=
  natsBytes_prefixFree.injective

/-- Prefix-free encoding of a list of strings. -/
def stringsBytes (l : List String) : List UInt8 :=
  Bytes.listBytes stringBytes l

theorem stringsBytes_prefixFree : Bytes.PrefixFree stringsBytes :=
  Bytes.listBytes_prefixFree stringBytes_prefixFree

/-- The bytes of an ASCII schema-tag name, for `TypeTag.leaf`. -/
def nameBytes (s : String) : List UInt8 :=
  s.toByteArray.toList

end Enc

/-! ## The pinned revision -/

/-- The identity of a pinned upstream source revision.  First-order: no
functions, no proofs (SPEC section 6.2). -/
structure RevisionBody where
  /-- Human-readable authority name, e.g. `WebAssembly Core`. -/
  authority : String
  /-- The pinned branch or working-group designation, e.g. `wg-3.0`. -/
  branch : String
  /-- The full 40-hex-digit commit identifier. -/
  commit : String
  deriving DecidableEq, Repr, Inhabited

namespace RevisionBody

/-- Lowercase hexadecimal digit test on a character code. -/
def isLowerHexDigit (c : Char) : Bool :=
  (48 ≤ c.toNat && c.toNat ≤ 57) || (97 ≤ c.toNat && c.toNat ≤ 102)

/-- A commit identifier is exactly forty lowercase hex digits. -/
def IsCommitDigest (s : String) : Bool :=
  s.toList.length == 40 && s.toList.all isLowerHexDigit

/-- Prefix-free canonical encoding. -/
def bytes (r : RevisionBody) : List UInt8 :=
  Enc.stringBytes r.authority ++
    (Enc.stringBytes r.branch ++ Enc.stringBytes r.commit)

theorem bytes_prefixFree : Bytes.PrefixFree bytes := by
  intro x y r s h
  simp only [bytes, List.append_assoc] at h
  obtain ⟨h1, h⟩ := Enc.stringBytes_prefixFree _ _ _ _ h
  obtain ⟨h2, h⟩ := Enc.stringBytes_prefixFree _ _ _ _ h
  obtain ⟨h3, h⟩ := Enc.stringBytes_prefixFree _ _ _ _ h
  refine ⟨?_, h⟩
  cases x
  cases y
  simp_all

theorem bytes_injective : Function.Injective bytes :=
  bytes_prefixFree.injective

/-- The frozen canonical schema of a pinned revision (SPEC section 6.2). -/
def identitySchema : CanonicalSchema RevisionBody :=
  CanonicalSchema.ofPrefixFree 1 .authority
    (TypeTag.leaf (Enc.nameBytes "wasm.revision.body/1"))
    (TypeTag.leaf_size_pos _)
    bytes bytes_prefixFree

/-- The erased canonical identity of a pinned revision. -/
def identity (r : RevisionBody) : CanonicalObjectId :=
  CanonicalObjectId.ofTyped (Identity identitySchema r)

theorem identity_eq_iff {a b : RevisionBody} :
    identity a = identity b ↔ a = b :=
  CanonicalObjectId.ofTyped_Identity_eq_iff identitySchema

end RevisionBody

/-- The pinned WebAssembly Core revision of SPEC section 4: the official
`wg-3.0` source at commit `9d36019973201a19f9c9ebb0f10828b2fe2374aa`. -/
def core3Revision : RevisionBody :=
  { authority := "WebAssembly Core"
    branch := "wg-3.0"
    commit := "9d36019973201a19f9c9ebb0f10828b2fe2374aa" }

/-- The pinned commit, as a standalone literal for the profile body. -/
def core3RevisionCommit : String :=
  "9d36019973201a19f9c9ebb0f10828b2fe2374aa"

theorem core3Revision_commit :
    core3Revision.commit = core3RevisionCommit := rfl

theorem core3Revision_authority :
    core3Revision.authority = "WebAssembly Core" := rfl

theorem core3Revision_branch :
    core3Revision.branch = "wg-3.0" := rfl

/-- The pinned commit really is a forty-digit lowercase hex identifier. -/
theorem core3Revision_commit_isDigest :
    RevisionBody.IsCommitDigest core3Revision.commit = true := by decide

theorem core3RevisionCommit_length :
    core3RevisionCommit.toList.length = 40 := by decide

theorem core3RevisionCommit_utf8_size :
    core3RevisionCommit.toByteArray.size = 40 := by decide

/-- Distinct revisions have distinct canonical identities; identical ones do
not.  This is what makes `profile_matches_pinned_revision` a real binding. -/
theorem core3Revision_identity_eq_iff (r : RevisionBody) :
    RevisionBody.identity r = RevisionBody.identity core3Revision ↔
      r = core3Revision :=
  RevisionBody.identity_eq_iff

end WasmGemmGnaf.Wasm

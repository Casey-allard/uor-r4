/-
  Coverage scope: what the seal's cover check does and does not establish.

  This file exists to HARDEN the repository, not to weaken it.  SPEC §12.1 lets
  `Atlas.SealCertificate` record the result of a deterministic cover checker, and
  SPEC §15 separately requires `Atlas.seal_implies_universal_coverage`.  Those are
  two different propositions, and conflating them is the single most attractive way
  to make this repository *appear* to prove global optimality.

  The lemma below makes the distinction machine-checked: it proves that
  `universalCoverCompleteCheck` is blind to the byte universe.  A latent gap that
  only a careful reader would notice becomes a fact the kernel has verified.
-/
import WasmGemmGnaf.Atlas.Certificate

set_option autoImplicit false

namespace WasmGemmGnaf.Atlas

/--
**Scope blindness of the cover check.**

`universalCoverCompleteCheck` is a function of exactly three recorded components:
the state's `searchPartitions`, the identities in its `candidateFacts`, and the
core's `partitionCoverRoot`.  Two states agreeing on those three agree on the
check — *whatever* byte strings exist, decode, validate, or compute GEMM.

Consequence, and the reason this lemma is here: no proposition quantified over
`ByteArray` can be derived from `universalCoverCompleteCheck … = true` alone.
The check verifies that the recorded cover is internally consistent with the
recorded candidate facts.  It does **not** verify that those partitions denote
every profile-valid byte string, and it is satisfiable by a cover that records
nothing at all.

This is exactly the failure mode SPEC §10.5 forbids when it requires
`universal_sublevel_coverage` to "have no coverage hypothesis", and that
UOR-GNAF §18 lists as an explicit non-claim:

  *that registration, discovery, index presence, or provenance alone proves
  eligibility, correctness, trust, feasibility, cost, or optimality.*

`Atlas.seal_implies_universal_coverage` (SPEC §15) is therefore **absent** from
this repository rather than derived from the seal.  Deriving it from this check
would be unsound.
-/
theorem universalCoverCompleteCheck_scope_blind
    (s₁ s₂ : UnsealedState) (core₁ core₂ : SealCore)
    (hp : s₁.body.searchPartitions = s₂.body.searchPartitions)
    (hc : s₁.body.candidateFacts.keys = s₂.body.candidateFacts.keys)
    (hr : core₁.partitionCoverRoot = core₂.partitionCoverRoot) :
    universalCoverCompleteCheck s₁ core₁ = universalCoverCompleteCheck s₂ core₂ := by
  simp only [universalCoverCompleteCheck, hp, hc, hr]

/--
The same statement in the form the release gate cares about: the cover check
cannot distinguish two cores whose recorded roots agree, so it cannot be the
witness for a claim about all competitor bytes.
-/
theorem universalCoverCompleteCheck_determined_by_record
    (s : UnsealedState) (core₁ core₂ : SealCore)
    (hr : core₁.partitionCoverRoot = core₂.partitionCoverRoot) :
    universalCoverCompleteCheck s core₁ = universalCoverCompleteCheck s core₂ :=
  universalCoverCompleteCheck_scope_blind s s core₁ core₂ rfl rfl hr

end WasmGemmGnaf.Atlas

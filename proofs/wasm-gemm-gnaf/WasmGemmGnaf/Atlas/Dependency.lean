import WasmGemmGnaf.Atlas.SemanticClosure
set_option autoImplicit false

/-!
# Atlas: dependency and invalidation (SPEC §12.3)

SPEC §12.3:

  *Every certificate SHALL list its exact object, edge, profile, problem,
  objective, and partition dependencies.  Adding or changing an edge invalidates
  the complete transitive impact cone before any new seal.*

This file supplies the three pieces.

## Exact dependency edges

`Atlas.exactDependencyEdges` derives, from the structure of a `StateBody`
alone, the dependency edge of every shape hyperedge, closure derivation, search
partition, envelope region and stored certificate.  For a certificate,
`Atlas.certificateDependencies` is the exact list SPEC §12.3 enumerates: the
subject object, the profile, problem and objective identities, the recorded
dependencies, and every partition containing the subject.
`Atlas.certificateDependenciesExact` is the decidable check that a stored
certificate's own list already contains all of them, with
`Atlas.certificateDependenciesExact_iff` proving the check equivalent to the
property (it is a real check: it fails whenever any of them is missing).

## The impact cone

The cone is the least set closed under "depends on something already in the
cone", computed with the terminating saturation of `Atlas/SemanticClosure.lean`:
`Atlas.impactCone edges changed = Closure.closureList (coneRules edges) changed`.
It is therefore a total, fuel-bounded reachability computation, not a search.

* `Atlas.impactCone_sound` — everything in the cone really reaches a changed
  object along recorded dependency edges.
* `Atlas.impactCone_complete` — everything that reaches a changed object is in
  the cone.
* `Atlas.impactCone_iff` — the two together: the computation is exactly
  reachability.
* `Atlas.invalidation_complete` — **SPEC §12.3**: every certificate that
  transitively depends on a changed object is in the cone.

## Anti-vacuity

`Atlas.impactCone_excludes_unrecorded` proves the converse gap explicitly: an
object with *no recorded dependency edge* is never in the cone, however it may
truly depend on the change.  Cone completeness is completeness **relative to the
recorded edge set**; it is not evidence that the edge set is itself complete.
That is why `Atlas.RecordsExactDependencies` (a decidable obligation on the
state, not a stored conclusion) is separated out and proved to transport
reachability: `Atlas.impactCone_recorded_covers_exact`.
-/

namespace WasmGemmGnaf.Atlas

open WasmGemmGnaf.Foundation

/-! ## Exact dependencies of a certificate (SPEC §12.3) -/

/-- The partitions that contain `x` as a member. -/
def partitionsContaining (b : StateBody) (x : CanonicalObjectId) :
    List CanonicalObjectId :=
  (b.searchPartitions.entries.filter (fun p => memId p.members x)).map (·.partitionId)

theorem mem_partitionsContaining (b : StateBody) (x p : CanonicalObjectId) :
    p ∈ partitionsContaining b x ↔
      ∃ e ∈ b.searchPartitions.entries, x ∈ e.members ∧ e.partitionId = p := by
  simp only [partitionsContaining, List.mem_map, List.mem_filter]
  constructor
  · intro ⟨e, ⟨he, hm⟩, hp⟩; exact ⟨e, he, (memId_iff e.members x).mp hm, hp⟩
  · intro ⟨e, he, hm, hp⟩; exact ⟨e, ⟨he, (memId_iff e.members x).mpr hm⟩, hp⟩

/-- **SPEC §12.3**: the exact object, edge, profile, problem, objective and
partition dependencies of a stored certificate. -/
def certificateDependencies (b : StateBody) (c : CertificateEntry) :
    List CanonicalObjectId :=
  c.subject :: b.profileId :: b.problemId :: b.objectiveId ::
    (c.dependencies ++ partitionsContaining b c.subject)

/-- The decidable obligation that a certificate's recorded dependency list is
already exact: it names its subject, the three scope identities and every
partition containing its subject. -/
def certificateDependenciesExact (b : StateBody) (c : CertificateEntry) : Bool :=
  subsetId (certificateDependencies b c) c.dependencies

theorem certificateDependenciesExact_iff (b : StateBody) (c : CertificateEntry) :
    certificateDependenciesExact b c = true ↔
      ∀ d ∈ certificateDependencies b c, d ∈ c.dependencies :=
  subsetId_iff _ _

/-- An exact certificate lists its subject. -/
theorem exact_lists_subject {b : StateBody} {c : CertificateEntry}
    (h : certificateDependenciesExact b c = true) : c.subject ∈ c.dependencies :=
  (certificateDependenciesExact_iff b c).mp h c.subject (by simp [certificateDependencies])

/-- An exact certificate lists the profile, problem and objective identities. -/
theorem exact_lists_scope {b : StateBody} {c : CertificateEntry}
    (h : certificateDependenciesExact b c = true) :
    b.profileId ∈ c.dependencies ∧ b.problemId ∈ c.dependencies ∧
      b.objectiveId ∈ c.dependencies := by
  refine ⟨?_, ?_, ?_⟩ <;>
    exact (certificateDependenciesExact_iff b c).mp h _ (by simp [certificateDependencies])

/-- An exact certificate lists every partition containing its subject. -/
theorem exact_lists_partitions {b : StateBody} {c : CertificateEntry}
    (h : certificateDependenciesExact b c = true) (p : CanonicalObjectId)
    (hp : p ∈ partitionsContaining b c.subject) : p ∈ c.dependencies :=
  (certificateDependenciesExact_iff b c).mp h p (by
    simp only [certificateDependencies, List.mem_cons, List.mem_append]
    exact Or.inr (Or.inr (Or.inr (Or.inr (Or.inr hp)))))

/-! ## The exact dependency edges of a state (SPEC §12.3) -/

/-- **SPEC §12.3**: the exact dependency edges induced by the structure of a
state body.  A shape edge depends on its endpoints, a closure conclusion on its
derivation edge and premises, a partition on its members, an envelope region on
its attained candidate, and a certificate on `certificateDependencies`. -/
def exactDependencyEdges (b : StateBody) : List DependencyEdge :=
  b.shapeEdges.edges.map (fun e => ⟨e.edgeId, e.endpoints⟩) ++
  b.semanticClosure.derivations.map (fun d => ⟨d.conclusion, d.edgeId :: d.premises⟩) ++
  b.searchPartitions.entries.map (fun p => ⟨p.partitionId, p.members⟩) ++
  b.lowerEnvelope.regions.map (fun r => ⟨r.regionId, r.attained.toList⟩) ++
  b.certificates.entries.map (fun c => ⟨c.certificateId, certificateDependencies b c⟩)

theorem mem_exactDependencyEdges_certificate (b : StateBody) (c : CertificateEntry)
    (hc : c ∈ b.certificates.entries) :
    (⟨c.certificateId, certificateDependencies b c⟩ : DependencyEdge) ∈
      exactDependencyEdges b := by
  simp only [exactDependencyEdges, List.mem_append, List.mem_map]
  exact Or.inr ⟨c, hc, rfl⟩

theorem mem_exactDependencyEdges_shape (b : StateBody) (e : HyperEdge)
    (he : e ∈ b.shapeEdges.edges) :
    (⟨e.edgeId, e.endpoints⟩ : DependencyEdge) ∈ exactDependencyEdges b := by
  simp only [exactDependencyEdges, List.mem_append, List.mem_map]
  exact Or.inl (Or.inl (Or.inl (Or.inl ⟨e, he, rfl⟩)))

theorem mem_exactDependencyEdges_derivation (b : StateBody) (d : DerivationEdge)
    (hd : d ∈ b.semanticClosure.derivations) :
    (⟨d.conclusion, d.edgeId :: d.premises⟩ : DependencyEdge) ∈ exactDependencyEdges b := by
  simp only [exactDependencyEdges, List.mem_append, List.mem_map]
  exact Or.inl (Or.inl (Or.inl (Or.inr ⟨d, hd, rfl⟩)))

/-- The recorded dependency graph covers every structural dependency edge.  A
decidable obligation on the state, not a stored conclusion. -/
def RecordsExactDependencies (b : StateBody) : Prop :=
  ∀ e ∈ exactDependencyEdges b, e ∈ b.dependencyGraph.edges

instance (b : StateBody) : Decidable (RecordsExactDependencies b) := by
  unfold RecordsExactDependencies; infer_instance

/-! ## Reachability along dependency edges -/

/-- `x` depends directly on `y`. -/
def DependsDirect (edges : List DependencyEdge) (x y : CanonicalObjectId) : Prop :=
  ∃ e ∈ edges, e.source = x ∧ y ∈ e.dependsOn

/-- Transitive-reflexive dependency: `x` reaches `y` along recorded edges. -/
inductive Reaches (edges : List DependencyEdge) : CanonicalObjectId → CanonicalObjectId → Prop
  | refl (x : CanonicalObjectId) : Reaches edges x x
  | step {x y z : CanonicalObjectId} :
      DependsDirect edges x y → Reaches edges y z → Reaches edges x z

theorem Reaches.trans {edges : List DependencyEdge} {x y z : CanonicalObjectId}
    (h₁ : Reaches edges x y) (h₂ : Reaches edges y z) : Reaches edges x z := by
  induction h₁ with
  | refl _ => exact h₂
  | step hd _ ih => exact Reaches.step hd (ih h₂)

theorem Reaches.mono {e₁ e₂ : List DependencyEdge} (h : ∀ x ∈ e₁, x ∈ e₂)
    {x y : CanonicalObjectId} (hr : Reaches e₁ x y) : Reaches e₂ x y := by
  induction hr with
  | refl x => exact Reaches.refl x
  | step hd _ ih =>
    obtain ⟨e, he, hs, hm⟩ := hd
    exact Reaches.step ⟨e, h e he, hs, hm⟩ ih

/-! ## The transitive impact cone -/

/-- The inference rules whose least closure is the impact cone: whenever `y` is
in the cone and `x` depends on `y`, `x` is in the cone. -/
def coneRules (edges : List DependencyEdge) : List (Closure.Rule CanonicalObjectId) :=
  edges.flatMap (fun e => e.dependsOn.map (fun d => ⟨[d], e.source⟩))

theorem mem_coneRules (edges : List DependencyEdge)
    (r : Closure.Rule CanonicalObjectId) :
    r ∈ coneRules edges ↔
      ∃ e ∈ edges, ∃ d ∈ e.dependsOn, r = ⟨[d], e.source⟩ := by
  simp only [coneRules, List.mem_flatMap, List.mem_map]
  constructor
  · intro ⟨e, he, d, hd, hr⟩; exact ⟨e, he, d, hd, hr.symm⟩
  · intro ⟨e, he, d, hd, hr⟩; exact ⟨e, he, d, hd, hr.symm⟩

theorem coneRule_of_dependsDirect {edges : List DependencyEdge}
    {x y : CanonicalObjectId} (h : DependsDirect edges x y) :
    (⟨[y], x⟩ : Closure.Rule CanonicalObjectId) ∈ coneRules edges := by
  obtain ⟨e, he, hs, hm⟩ := h
  exact (mem_coneRules edges _).mpr ⟨e, he, y, hm, by rw [hs]⟩

/-- **The transitive impact cone**: every object that transitively depends on a
changed object.  A terminating computation: it is the fuel-bounded least closure
of `changed` under `coneRules`. -/
def impactCone (edges : List DependencyEdge) (changed : List CanonicalObjectId) :
    List CanonicalObjectId :=
  Closure.closureList (coneRules edges) changed

theorem mem_impactCone_iff_cl (edges : List DependencyEdge)
    (changed : List CanonicalObjectId) (x : CanonicalObjectId) :
    x ∈ impactCone edges changed ↔
      Closure.Cl (coneRules edges) (Closure.ofList changed) x = true :=
  Closure.mem_closureList_iff _ _ _

/-- Changed objects are themselves in their own impact cone. -/
theorem changed_mem_impactCone (edges : List DependencyEdge)
    (changed : List CanonicalObjectId) (x : CanonicalObjectId) (h : x ∈ changed) :
    x ∈ impactCone edges changed :=
  (mem_impactCone_iff_cl edges changed x).mpr
    (Closure.Cl_extensive _ _ x ((Closure.ofList_iff changed x).mpr h))

/-- **Soundness of the cone computation.**  Everything the computation returns
really reaches a changed object along recorded dependency edges. -/
theorem impactCone_sound (edges : List DependencyEdge)
    (changed : List CanonicalObjectId) (x : CanonicalObjectId)
    (h : x ∈ impactCone edges changed) :
    ∃ t ∈ changed, Reaches edges x t := by
  have hd := Closure.Cl_sound _ _ _ ((mem_impactCone_iff_cl edges changed x).mp h)
  clear h
  induction hd with
  | @base y hy =>
    exact ⟨y, (Closure.ofList_iff changed y).mp hy, Reaches.refl y⟩
  | @rule r hr _ ih =>
    obtain ⟨e, he, d, hdm, rfl⟩ := (mem_coneRules edges r).mp hr
    obtain ⟨t, ht, hreach⟩ := ih d (by simp)
    exact ⟨t, ht, Reaches.step ⟨e, he, rfl, hdm⟩ hreach⟩

/-- **Completeness of the cone computation.**  Everything that reaches a changed
object is returned. -/
theorem impactCone_complete (edges : List DependencyEdge)
    (changed : List CanonicalObjectId) (x t : CanonicalObjectId)
    (ht : t ∈ changed) (hr : Reaches edges x t) :
    x ∈ impactCone edges changed := by
  rw [mem_impactCone_iff_cl]
  induction hr with
  | refl y =>
    exact Closure.Cl_extensive _ _ y ((Closure.ofList_iff changed y).mpr ht)
  | @step a b c hab _ ih =>
    refine Closure.Cl_closed (coneRules edges) (Closure.ofList changed)
      ⟨[b], a⟩ (coneRule_of_dependsDirect hab) ?_
    intro p hp
    simp only [List.mem_singleton] at hp
    subst hp
    exact ih ht

/-- **The cone computation is exactly reachability.** -/
theorem impactCone_iff (edges : List DependencyEdge)
    (changed : List CanonicalObjectId) (x : CanonicalObjectId) :
    x ∈ impactCone edges changed ↔ ∃ t ∈ changed, Reaches edges x t := by
  constructor
  · exact impactCone_sound edges changed x
  · intro ⟨t, ht, hr⟩; exact impactCone_complete edges changed x t ht hr

/-- The cone grows with the changed set. -/
theorem impactCone_mono_changed (edges : List DependencyEdge)
    {c₁ c₂ : List CanonicalObjectId} (h : ∀ x ∈ c₁, x ∈ c₂) (x : CanonicalObjectId)
    (hx : x ∈ impactCone edges c₁) : x ∈ impactCone edges c₂ := by
  obtain ⟨t, ht, hr⟩ := impactCone_sound edges c₁ x hx
  exact impactCone_complete edges c₂ x t (h t ht) hr

/-- The cone grows with the edge set. -/
theorem impactCone_mono_edges {e₁ e₂ : List DependencyEdge}
    (h : ∀ e ∈ e₁, e ∈ e₂) (changed : List CanonicalObjectId) (x : CanonicalObjectId)
    (hx : x ∈ impactCone e₁ changed) : x ∈ impactCone e₂ changed := by
  obtain ⟨t, ht, hr⟩ := impactCone_sound e₁ changed x hx
  exact impactCone_complete e₂ changed x t ht (Reaches.mono h hr)

/-- If the state records every structural dependency edge, the cone over the
recorded graph covers the cone over the structural edges. -/
theorem impactCone_recorded_covers_exact (b : StateBody)
    (h : RecordsExactDependencies b) (changed : List CanonicalObjectId)
    (x : CanonicalObjectId) (hx : x ∈ impactCone (exactDependencyEdges b) changed) :
    x ∈ impactCone b.dependencyGraph.edges changed :=
  impactCone_mono_edges h changed x hx

/-! ## Invalidation (SPEC §12.3) -/

/--
**SPEC §12.3 — `Atlas.invalidation_complete`.**

Every certificate that transitively depends on a changed object is in the impact
cone.  "Transitively depends" is spelled out exactly: one of the certificate's
*exact* dependencies (subject, profile, problem, objective, recorded
dependencies, containing partitions) reaches a changed object along the state's
structural dependency edges.

So invalidating the cone before resealing cannot miss an affected certificate.
-/
theorem invalidation_complete (b : StateBody) (changed : List CanonicalObjectId)
    (c : CertificateEntry) (hc : c ∈ b.certificates.entries)
    (d : CanonicalObjectId) (hd : d ∈ certificateDependencies b c)
    (t : CanonicalObjectId) (ht : t ∈ changed)
    (hreach : Reaches (exactDependencyEdges b) d t) :
    c.certificateId ∈ impactCone (exactDependencyEdges b) changed :=
  impactCone_complete _ changed c.certificateId t ht
    (Reaches.step
      ⟨⟨c.certificateId, certificateDependencies b c⟩,
        mem_exactDependencyEdges_certificate b c hc, rfl, hd⟩
      hreach)

/-- The immediate case: a certificate one of whose exact dependencies is itself
a changed object. -/
theorem invalidation_complete_direct (b : StateBody) (changed : List CanonicalObjectId)
    (c : CertificateEntry) (hc : c ∈ b.certificates.entries)
    (d : CanonicalObjectId) (hd : d ∈ certificateDependencies b c)
    (hchanged : d ∈ changed) :
    c.certificateId ∈ impactCone (exactDependencyEdges b) changed :=
  invalidation_complete b changed c hc d hd d hchanged (Reaches.refl d)

/-- **SPEC §12.3, "adding or changing an edge".**  Changing a shape hyperedge
invalidates every certificate that depends on that edge, however long the
dependency chain. -/
theorem invalidation_complete_of_edge_change (b : StateBody)
    (changed : List CanonicalObjectId) (e : HyperEdge)
    (_he : e ∈ b.shapeEdges.edges) (hchanged : e.edgeId ∈ changed)
    (c : CertificateEntry) (hc : c ∈ b.certificates.entries)
    (d : CanonicalObjectId) (hd : d ∈ certificateDependencies b c)
    (hreach : Reaches (exactDependencyEdges b) d e.edgeId) :
    c.certificateId ∈ impactCone (exactDependencyEdges b) changed :=
  invalidation_complete b changed c hc d hd e.edgeId hchanged hreach

/-- Every endpoint of a changed shape edge puts that edge in the cone. -/
theorem edge_mem_cone_of_endpoint_changed (b : StateBody)
    (changed : List CanonicalObjectId) (e : HyperEdge) (he : e ∈ b.shapeEdges.edges)
    (x : CanonicalObjectId) (hx : x ∈ e.endpoints) (hchanged : x ∈ changed) :
    e.edgeId ∈ impactCone (exactDependencyEdges b) changed :=
  impactCone_complete _ changed e.edgeId x hchanged
    (Reaches.step ⟨⟨e.edgeId, e.endpoints⟩,
      mem_exactDependencyEdges_shape b e he, rfl, hx⟩ (Reaches.refl x))

/-! ## Anti-vacuity: what the cone does *not* establish -/

/--
**Scope of cone completeness.**

`impactCone_complete` is completeness *relative to the recorded edge set*: an
object with no recorded dependency edge is never invalidated, no matter what it
truly depends on.  Making that machine-checked is the point of this lemma — a
passing invalidation pass is evidence about the recorded graph, never evidence
that the recorded graph is complete.

The separate, decidable obligation `Atlas.RecordsExactDependencies` is what ties
the recorded graph to the structural one; it must be discharged, not assumed.
-/
theorem impactCone_excludes_unrecorded (edges : List DependencyEdge)
    (changed : List CanonicalObjectId) (x : CanonicalObjectId)
    (hno : ∀ y, ¬ DependsDirect edges x y) (hx : x ∉ changed) :
    x ∉ impactCone edges changed := by
  intro hmem
  obtain ⟨t, ht, hr⟩ := impactCone_sound edges changed x hmem
  cases hr with
  | refl _ => exact hx ht
  | step hd _ => exact hno _ hd

/-- The cone is a function of the recorded edges and the changed set alone. -/
theorem impactCone_determined_by_edges (e₁ e₂ : List DependencyEdge)
    (changed : List CanonicalObjectId) (h : e₁ = e₂) :
    impactCone e₁ changed = impactCone e₂ changed := by rw [h]

end WasmGemmGnaf.Atlas

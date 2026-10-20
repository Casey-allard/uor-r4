import WasmGemmGnaf.Atlas.State
set_option autoImplicit false

/-!
# Atlas: exact cost surfaces (SPEC §12.4)

SPEC §12.4 requires the lower envelope to map exact regions "to attained
candidates or complete Pareto frontiers".  Both notions are *computed here from
the recorded cost surface*, never stored as a conclusion:

* `Atlas.CostSurfaceMap.entry?` / `score?` are exact lookups, with the
  soundness lemmas that make a successful lookup mean membership.
* `Atlas.Dominates` is the exact Pareto order on the recorded coordinate
  vectors of SPEC §9.1, with irreflexivity, asymmetry and transitivity proved.
* `Atlas.CostSurfaceMap.frontier` is the *complete* nondominated set of a
  recorded candidate list, characterised exactly by `mem_frontier_iff`.
* `Atlas.CostSurfaceMap.minScore?` computes the exact minimum of the recorded
  scores, and `Atlas.IsAttainedMinimum` is the proposition that a candidate
  attains it.  `attained_of_minScore?` is the only way this file produces an
  attainment fact, and it does so from a recomputation.

Nothing in this file asserts anything about byte strings that are *not*
recorded in the surface: a cost surface is a finite record, and every statement
below is explicitly relative to the recorded candidate list.
-/

namespace WasmGemmGnaf.Atlas

open WasmGemmGnaf.Foundation

/-! ## `List.find?` soundness, proved locally

Only Lean core and `Std` are available and the exact lemma names for `find?`
move between releases, so the two facts needed here are proved directly. -/

namespace Find

/-- A successful `find?` returns a member of the list satisfying the
predicate. -/
theorem sound {α : Type} (p : α → Bool) :
    ∀ (l : List α) (a : α), l.find? p = some a → a ∈ l ∧ p a = true := by
  intro l
  induction l with
  | nil => intro a h; exact absurd h (by simp [List.find?])
  | cons x xs ih =>
    intro a h
    simp only [List.find?] at h
    split at h
    · rename_i hp
      have : x = a := by simpa using h
      exact ⟨this ▸ List.mem_cons_self, this ▸ hp⟩
    · obtain ⟨h1, h2⟩ := ih a h
      exact ⟨List.mem_cons_of_mem _ h1, h2⟩

/-- If some member satisfies the predicate then `find?` succeeds. -/
theorem complete {α : Type} (p : α → Bool) :
    ∀ (l : List α) (a : α), a ∈ l → p a = true → ∃ b, l.find? p = some b := by
  intro l
  induction l with
  | nil => intro a ha _; simp at ha
  | cons x xs ih =>
    intro a ha hp
    simp only [List.find?]
    split
    · exact ⟨x, rfl⟩
    · rename_i hx
      rcases List.mem_cons.mp ha with rfl | ha'
      · exact absurd hp (by simp [hx])
      · exact ih a ha' hp

end Find

/-! ## Exact lookup in a recorded cost surface -/

namespace CostSurfaceMap

/-- The recorded entry of a candidate, if any. -/
def entry? (m : CostSurfaceMap) (id : CanonicalObjectId) : Option CostSurfaceEntry :=
  m.entries.find? (fun e => decide (e.candidateId = id))

@[simp] theorem score?_eq_map (m : CostSurfaceMap) (id : CanonicalObjectId) :
    m.score? id = (m.entry? id).map (·.score) := rfl

/-- A successful lookup returns a recorded entry for exactly that candidate. -/
theorem entry?_sound {m : CostSurfaceMap} {id : CanonicalObjectId}
    {e : CostSurfaceEntry} (h : m.entry? id = some e) :
    e ∈ m.entries ∧ e.candidateId = id := by
  obtain ⟨hmem, hp⟩ := Find.sound _ m.entries e h
  exact ⟨hmem, by simpa using hp⟩

/-- A recorded score is the score of a recorded entry for that candidate. -/
theorem score?_sound {m : CostSurfaceMap} {id : CanonicalObjectId} {s : Nat}
    (h : m.score? id = some s) :
    ∃ e ∈ m.entries, e.candidateId = id ∧ e.score = s := by
  rw [score?_eq_map] at h
  cases he : m.entry? id with
  | none => rw [he] at h; simp at h
  | some e =>
    rw [he] at h
    obtain ⟨hmem, hid⟩ := entry?_sound he
    exact ⟨e, hmem, hid, by simpa using h⟩

/-- A recorded candidate always has a recorded score. -/
theorem score?_isSome_of_mem {m : CostSurfaceMap} {e : CostSurfaceEntry}
    (h : e ∈ m.entries) : ∃ s, m.score? e.candidateId = some s := by
  obtain ⟨b, hb⟩ :=
    Find.complete (fun x => decide (x.candidateId = e.candidateId)) m.entries e h
      (decide_eq_true rfl)
  exact ⟨b.score, by rw [score?_eq_map, entry?, hb]; rfl⟩

/-- The recorded coordinate vector of a candidate (SPEC §9.1 coordinates). -/
def coordinates? (m : CostSurfaceMap) (id : CanonicalObjectId) : Option (List Nat) :=
  (m.entry? id).map (·.coordinates)

/-- The recorded coordinate vector, or the empty vector when unrecorded. -/
def coordinates (m : CostSurfaceMap) (id : CanonicalObjectId) : List Nat :=
  (m.coordinates? id).getD []

end CostSurfaceMap

/-! ## The exact Pareto order on recorded coordinate vectors -/

/-- Componentwise `≤` on two coordinate vectors of the same length. -/
def LeAll : List Nat → List Nat → Prop
  | [], [] => True
  | _ :: _, [] => False
  | [], _ :: _ => False
  | x :: xs, y :: ys => x ≤ y ∧ LeAll xs ys

/-- Strict improvement in at least one coordinate. -/
def LtSome : List Nat → List Nat → Prop
  | [], [] => False
  | _ :: _, [] => False
  | [], _ :: _ => False
  | x :: xs, y :: ys => x < y ∨ LtSome xs ys

instance decLeAll : ∀ (a b : List Nat), Decidable (LeAll a b)
  | [], [] => isTrue trivial
  | _ :: _, [] => isFalse (fun h => h)
  | [], _ :: _ => isFalse (fun h => h)
  | x :: xs, y :: ys =>
    have : Decidable (LeAll xs ys) := decLeAll xs ys
    inferInstanceAs (Decidable (x ≤ y ∧ LeAll xs ys))

instance decLtSome : ∀ (a b : List Nat), Decidable (LtSome a b)
  | [], [] => isFalse (fun h => h)
  | _ :: _, [] => isFalse (fun h => h)
  | [], _ :: _ => isFalse (fun h => h)
  | x :: xs, y :: ys =>
    have : Decidable (LtSome xs ys) := decLtSome xs ys
    inferInstanceAs (Decidable (x < y ∨ LtSome xs ys))

/-- **Pareto domination** on recorded coordinate vectors: `a` dominates `b`
when it is no worse in every coordinate and strictly better in one. -/
def Dominates (a b : List Nat) : Prop := LeAll a b ∧ LtSome a b

instance (a b : List Nat) : Decidable (Dominates a b) :=
  inferInstanceAs (Decidable (LeAll a b ∧ LtSome a b))

namespace LeAll

theorem refl : ∀ a : List Nat, LeAll a a
  | [] => trivial
  | _ :: xs => ⟨Nat.le_refl _, refl xs⟩

theorem trans : ∀ {a b c : List Nat}, LeAll a b → LeAll b c → LeAll a c
  | [], [], [], _, _ => trivial
  | _ :: _, [], _, h, _ => absurd h (fun h => h)
  | [], _ :: _, _, h, _ => absurd h (fun h => h)
  | _ :: _, _ :: _, [], _, h => absurd h (fun h => h)
  | _ :: _, _ :: _, _ :: _, ⟨h1, h2⟩, ⟨h3, h4⟩ =>
    ⟨Nat.le_trans h1 h3, trans h2 h4⟩

theorem length_eq : ∀ {a b : List Nat}, LeAll a b → a.length = b.length
  | [], [], _ => rfl
  | _ :: _, [], h => absurd h (fun h => h)
  | [], _ :: _, h => absurd h (fun h => h)
  | _ :: xs, _ :: ys, ⟨_, h2⟩ => by simp [length_eq h2]

end LeAll

namespace LtSome

theorem irrefl : ∀ a : List Nat, ¬ LtSome a a
  | [] => fun h => h
  | _ :: xs => fun h => by
      rcases h with h | h
      · exact absurd h (Nat.lt_irrefl _)
      · exact irrefl xs h

/-- A strict improvement somewhere is incompatible with being componentwise
no better. -/
theorem not_leAll_of_ltSome : ∀ {a b : List Nat}, LtSome a b → ¬ LeAll b a
  | [], [], h, _ => h
  | _ :: _, [], h, _ => h
  | [], _ :: _, h, _ => h
  | _ :: xs, _ :: ys, h, ⟨g1, g2⟩ => by
    rcases h with h | h
    · exact absurd h (Nat.not_lt.mpr g1)
    · exact not_leAll_of_ltSome h g2

end LtSome

namespace Dominates

/-- Domination is irreflexive: nothing dominates itself. -/
theorem irrefl (a : List Nat) : ¬ Dominates a a := fun h => LtSome.irrefl a h.2

/-- Domination is asymmetric. -/
theorem asymm {a b : List Nat} (h : Dominates a b) : ¬ Dominates b a :=
  fun g => LtSome.not_leAll_of_ltSome h.2 g.1

/-- Domination is transitive. -/
theorem trans : ∀ {a b c : List Nat}, Dominates a b → Dominates b c → Dominates a c
  | [], [], [], h, _ => absurd h.2 (fun h => h)
  | _ :: _, [], _, h, _ => absurd h.1 (fun h => h)
  | [], _ :: _, _, h, _ => absurd h.1 (fun h => h)
  | _ :: _, _ :: _, [], _, h => absurd h.1 (fun h => h)
  | x :: xs, y :: ys, z :: zs, ⟨⟨hab1, hab2⟩, hab3⟩, ⟨⟨hbc1, hbc2⟩, hbc3⟩ => by
    refine ⟨⟨Nat.le_trans hab1 hbc1, LeAll.trans hab2 hbc2⟩, ?_⟩
    rcases hab3 with h | h
    · exact Or.inl (Nat.lt_of_lt_of_le h hbc1)
    · rcases hbc3 with g | g
      · exact Or.inl (Nat.lt_of_le_of_lt hab1 g)
      · exact Or.inr (trans ⟨hab2, h⟩ ⟨hbc2, g⟩).2

end Dominates

/-! ## Complete Pareto frontiers (SPEC §12.4) -/

namespace CostSurfaceMap

/-- `c` is dominated by some candidate of `ids` under the recorded
coordinates. -/
def dominatedBy (m : CostSurfaceMap) (ids : List CanonicalObjectId)
    (c : CanonicalObjectId) : Bool :=
  ids.any (fun d => decide (Dominates (m.coordinates d) (m.coordinates c)))

theorem dominatedBy_iff (m : CostSurfaceMap) (ids : List CanonicalObjectId)
    (c : CanonicalObjectId) :
    m.dominatedBy ids c = true ↔
      ∃ d ∈ ids, Dominates (m.coordinates d) (m.coordinates c) := by
  simp [dominatedBy]

/-- `c` is not dominated by any candidate of `ids`. -/
def nondominated (m : CostSurfaceMap) (ids : List CanonicalObjectId)
    (c : CanonicalObjectId) : Bool :=
  !(m.dominatedBy ids c)

theorem nondominated_iff (m : CostSurfaceMap) (ids : List CanonicalObjectId)
    (c : CanonicalObjectId) :
    m.nondominated ids c = true ↔
      ∀ d ∈ ids, ¬ Dominates (m.coordinates d) (m.coordinates c) := by
  simp [nondominated, dominatedBy]

/-- The **complete** Pareto frontier of a recorded candidate list: exactly the
candidates of `ids` that no candidate of `ids` dominates. -/
def frontier (m : CostSurfaceMap) (ids : List CanonicalObjectId) :
    List CanonicalObjectId :=
  ids.filter (fun c => m.nondominated ids c)

/-- Exact characterisation of the frontier.  This is what makes the frontier
*complete*: membership is equivalent to nondomination, in both directions. -/
theorem mem_frontier_iff (m : CostSurfaceMap) (ids : List CanonicalObjectId)
    (c : CanonicalObjectId) :
    c ∈ m.frontier ids ↔
      c ∈ ids ∧ ∀ d ∈ ids, ¬ Dominates (m.coordinates d) (m.coordinates c) := by
  simp [frontier, List.mem_filter, nondominated_iff]

/-- Frontier members are recorded candidates. -/
theorem frontier_subset (m : CostSurfaceMap) (ids : List CanonicalObjectId)
    {c : CanonicalObjectId} (h : c ∈ m.frontier ids) : c ∈ ids :=
  ((mem_frontier_iff m ids c).mp h).1

/-- A candidate outside the frontier is dominated by a recorded candidate: the
frontier omits nothing without exact cause. -/
theorem dominated_of_not_mem_frontier (m : CostSurfaceMap)
    (ids : List CanonicalObjectId) {c : CanonicalObjectId} (hc : c ∈ ids)
    (h : c ∉ m.frontier ids) :
    ∃ d ∈ ids, Dominates (m.coordinates d) (m.coordinates c) := by
  have hb : m.nondominated ids c ≠ true := fun hb =>
    h ((mem_frontier_iff m ids c).mpr ⟨hc, (nondominated_iff m ids c).mp hb⟩)
  cases hx : m.dominatedBy ids c with
  | true => exact (dominatedBy_iff m ids c).mp hx
  | false => exact absurd (by simp [nondominated, hx]) hb

/-- No frontier member dominates another frontier member. -/
theorem frontier_pairwise_nondominated (m : CostSurfaceMap)
    (ids : List CanonicalObjectId) {c d : CanonicalObjectId}
    (hc : c ∈ m.frontier ids) (hd : d ∈ m.frontier ids) :
    ¬ Dominates (m.coordinates d) (m.coordinates c) :=
  ((mem_frontier_iff m ids c).mp hc).2 d (frontier_subset m ids hd)

end CostSurfaceMap

/-! ## Exact attained minima -/

/-- The minimum of a list of natural numbers, when the list is nonempty. -/
def minOf : List Nat → Option Nat
  | [] => none
  | x :: xs => match minOf xs with
    | none => some x
    | some m => some (if x ≤ m then x else m)

/-- A nonempty list has a minimum. -/
theorem minOf_isSome_of_mem {l : List Nat} {x : Nat} (h : x ∈ l) :
    ∃ m, minOf l = some m := by
  cases l with
  | nil => exact absurd h (by simp)
  | cons y ys =>
    simp only [minOf]
    split
    · exact ⟨y, rfl⟩
    · rename_i m' _; exact ⟨if y ≤ m' then y else m', rfl⟩

theorem minOf_mem : ∀ {l : List Nat} {m : Nat}, minOf l = some m → m ∈ l
  | [], _, h => by simp [minOf] at h
  | x :: xs, m, h => by
    simp only [minOf] at h
    split at h
    · have hx : m = x := by simpa using h.symm
      rw [hx]; exact List.mem_cons_self
    · rename_i m' hm'
      have hm : m = (if x ≤ m' then x else m') := by simpa using h.symm
      by_cases hle : x ≤ m'
      · have hmx : m = x := by rw [hm, if_pos hle]
        rw [hmx]; exact List.mem_cons_self
      · have hmx : m = m' := by rw [hm, if_neg hle]
        rw [hmx]; exact List.mem_cons_of_mem _ (minOf_mem hm')

theorem minOf_le : ∀ {l : List Nat} {m x : Nat}, minOf l = some m → x ∈ l → m ≤ x
  | [], _, _, _, hx => by simp at hx
  | y :: ys, m, x, h, hx => by
    simp only [minOf] at h
    split at h
    · rename_i hnone
      rcases List.mem_cons.mp hx with rfl | hx'
      · exact Nat.le_of_eq (by simpa using h.symm)
      · obtain ⟨m'', hm''⟩ := minOf_isSome_of_mem hx'
        rw [hnone] at hm''
        exact absurd hm'' (by simp)
    · rename_i m' hm'
      have hm : m = (if y ≤ m' then y else m') := by simpa using h.symm
      rcases List.mem_cons.mp hx with rfl | hx'
      · rw [hm]; split <;> omega
      · have hstep := minOf_le hm' hx'
        rw [hm]; split <;> omega

namespace CostSurfaceMap

/-- The recorded scores of a candidate list, in candidate order. -/
def scoresOf (m : CostSurfaceMap) (ids : List CanonicalObjectId) : List Nat :=
  ids.filterMap m.score?

theorem mem_scoresOf {m : CostSurfaceMap} {ids : List CanonicalObjectId}
    {c : CanonicalObjectId} {s : Nat} (hc : c ∈ ids) (hs : m.score? c = some s) :
    s ∈ m.scoresOf ids := by
  simp only [scoresOf, List.mem_filterMap]
  exact ⟨c, hc, hs⟩

/-- The exact minimum of the recorded scores of a candidate list. -/
def minScore? (m : CostSurfaceMap) (ids : List CanonicalObjectId) : Option Nat :=
  minOf (m.scoresOf ids)

end CostSurfaceMap

/-- **Attained minimum** over a recorded candidate list: `c` is a recorded
candidate whose recorded score `s` is `≤` every recorded score of the list.
This proposition is *relative to the recorded list* and says nothing about
candidates that were never recorded. -/
def IsAttainedMinimum (m : CostSurfaceMap) (ids : List CanonicalObjectId)
    (c : CanonicalObjectId) (s : Nat) : Prop :=
  c ∈ ids ∧ m.score? c = some s ∧ ∀ d ∈ ids, ∀ t, m.score? d = some t → s ≤ t

/-- The only route to an attainment fact in this file: recompute the minimum
and exhibit a candidate that realises it. -/
theorem attained_of_minScore? {m : CostSurfaceMap} {ids : List CanonicalObjectId}
    {c : CanonicalObjectId} {s : Nat}
    (hc : c ∈ ids) (hs : m.score? c = some s) (hmin : m.minScore? ids = some s) :
    IsAttainedMinimum m ids c s := by
  refine ⟨hc, hs, ?_⟩
  intro d hd t ht
  exact minOf_le hmin (CostSurfaceMap.mem_scoresOf hd ht)

/-- An attained minimum really is the recomputed minimum. -/
theorem minScore?_eq_of_attained {m : CostSurfaceMap} {ids : List CanonicalObjectId}
    {c : CanonicalObjectId} {s : Nat} (h : IsAttainedMinimum m ids c s) :
    m.minScore? ids = some s := by
  obtain ⟨hc, hs, hle⟩ := h
  obtain ⟨m', hm'⟩ := minOf_isSome_of_mem (CostSurfaceMap.mem_scoresOf hc hs)
  have hmem : m' ∈ m.scoresOf ids := minOf_mem hm'
  simp only [CostSurfaceMap.scoresOf, List.mem_filterMap] at hmem
  obtain ⟨d, hd, hdscore⟩ := hmem
  have h1 : s ≤ m' := hle d hd m' hdscore
  have h2 : m' ≤ s := minOf_le hm' (CostSurfaceMap.mem_scoresOf hc hs)
  rw [CostSurfaceMap.minScore?, hm']
  exact congrArg some (Nat.le_antisymm h2 h1)

/-- Attained minima are unique as values. -/
theorem attained_score_unique {m : CostSurfaceMap} {ids : List CanonicalObjectId}
    {c d : CanonicalObjectId} {s t : Nat}
    (hc : IsAttainedMinimum m ids c s) (hd : IsAttainedMinimum m ids d t) : s = t :=
  Nat.le_antisymm (hc.2.2 d hd.1 t hd.2.1) (hd.2.2 c hc.1 s hc.2.1)

end WasmGemmGnaf.Atlas

import WasmGemmGnaf.Conformance.Claim
import WasmGemmGnaf.Foundation.Termination
set_option autoImplicit false

/-!
# Conformance: the claim registry (SPEC §17.2, §17.3)

The registry is the canonical list of `ClaimRow`s of `model/claims.json`.  Its
well-formedness is *decidable* and consists of

* a nonempty row list (SPEC §20.2 condition 2);
* unique claim IDs — a duplicate row is rejected (`duplicate_id_rejected`);
* no orphan dependency — every direct dependency resolves to a row;
* an acyclic dependency graph;
* a claim-family policy: every ID belongs to one of the twelve required
  families of SPEC §17.3, and the families whose level is constrained by the
  SPEC (`RT-*` is never a proof, `ME-*` is measurement or open) obey it.

## The acyclicity checker

`Dep reg a b` is the direct dependency relation "the row named `a` lists `b`".
`depthOk reg fuel a` is the bounded depth-first check.  The checker is proved
**sound and complete** with respect to that relation:

`Acyclic reg  ↔  (∃ fuel, acyclicCheck reg fuel = true)  ↔  WellFounded (DepFlip reg)`

where `Acyclic reg` is the existence of a strictly decreasing natural ranking.
Soundness constructs the ranking from the least fuel that passes; completeness
runs the check with a fuel dominating the ranking (or extracted from
accessibility).  `acyclic_no_cycle` then rules out every dependency cycle.

The decidable field `acyclicB` fixes the fuel at `rows.length + 1`; it is proved
sound (`acyclicB_sound`).  Completeness *at that particular fuel* is a finite
pigeonhole statement that is not proved here, so nothing in this file depends
on it.
-/

namespace WasmGemmGnaf.Conformance

open WasmGemmGnaf.Foundation

/-! ## Claim families (SPEC §17.3) -/

/-- The twelve required claim families of SPEC §17.3. -/
inductive ClaimFamily
  /-- Lean/toolchain/dependency identity and axiom closure. -/
  | LF
  /-- WebAssembly binary, validation, execution, feature, cost-erasure. -/
  | WS
  /-- GEMM classifier, arithmetic, ABI, reference semantics. -/
  | GM
  /-- Emitted byte identity, decoding, validation, imports/exports, refinement. -/
  | BI
  /-- Universal byte/input/run sublevel and partition coverage. -/
  | UV
  /-- Complete accounting, objective laws, properness, aggregation. -/
  | CO
  /-- Universal lower bound and falsification mutations. -/
  | LB
  /-- GNAF/Atlas closure, attention, invalidation, update, seal. -/
  | AT
  /-- Attainment and final global theorem. -/
  | GO
  /-- Executable differential evidence, never proof promotion. -/
  | RT
  /-- Pinned engine measurements, always measurement or open. -/
  | ME
  /-- Registry, manifest, source, dependency, release-gate integrity. -/
  | CM
  deriving DecidableEq, Repr, Inhabited

namespace ClaimFamily

/-- The complete enumeration of required claim families. -/
def all : List ClaimFamily := [LF, WS, GM, BI, UV, CO, LB, AT, GO, RT, ME, CM]

theorem mem_all (f : ClaimFamily) : f ∈ all := by cases f <;> simp [all]

theorem all_nodup : all.Nodup := by decide

theorem all_length : all.length = 12 := rfl

instance : Fintype ClaimFamily where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

/-- Structural index of a family. -/
def index : ClaimFamily → Nat
  | LF => 0 | WS => 1 | GM => 2 | BI => 3 | UV => 4 | CO => 5
  | LB => 6 | AT => 7 | GO => 8 | RT => 9 | ME => 10 | CM => 11

theorem index_injective : Function.Injective index := by
  intro a b h
  cases a <;> cases b <;> simp_all [index]

/-- The claim-ID tag of a family, as it appears in `LF-*`, `WS-*`, … -/
def tag : ClaimFamily → String
  | LF => "LF" | WS => "WS" | GM => "GM" | BI => "BI" | UV => "UV" | CO => "CO"
  | LB => "LB" | AT => "AT" | GO => "GO" | RT => "RT" | ME => "ME" | CM => "CM"

theorem tag_injective : Function.Injective tag := by
  intro a b h
  cases a <;> cases b <;> first
    | rfl
    | exact absurd h (by decide)

/-- Recover the family from an ID tag. -/
def ofTag? (s : String) : Option ClaimFamily := all.find? (fun f => f.tag == s)

theorem ofTag?_tag (f : ClaimFamily) : ofTag? f.tag = some f := by
  cases f <;> decide

theorem tag_of_ofTag? {s : String} {f : ClaimFamily} (h : ofTag? s = some f) :
    f.tag = s := by
  have := List.find?_some h
  exact eq_of_beq (by simpa using this)

theorem ofTag?_eq_some_iff (s : String) (f : ClaimFamily) :
    ofTag? s = some f ↔ f.tag = s := by
  constructor
  · exact tag_of_ofTag?
  · intro h; rw [← h]; exact ofTag?_tag f

end ClaimFamily

/-! ## Family level policy (SPEC §17.3)

`RT-*` is "executable differential evidence, never proof promotion" and `ME-*`
is "pinned engine measurements, always measurement or open". -/

/-- Whether a claim of the given family may carry the given level. -/
def levelPermitted : ClaimFamily → ClaimLevel → Bool
  | .RT, .formalProof => false
  | .ME, .measurement => true
  | .ME, .«open» => true
  | .ME, _ => false
  | _, _ => true

/-- `RT-*` rows are never at `formalProof`. -/
theorem levelPermitted_RT_ne_formalProof {l : ClaimLevel}
    (h : levelPermitted .RT l = true) : l ≠ .formalProof := by
  cases l <;> simp_all [levelPermitted]

/-- `RT-*` rows may never be stated with proof language (SPEC §17.1 + §17.3). -/
theorem levelPermitted_RT_not_proofLanguage {l : ClaimLevel}
    (h : levelPermitted .RT l = true) : ¬ SupportsProofLanguage l :=
  not_supportsProofLanguage_of_ne (levelPermitted_RT_ne_formalProof h)

/-- `ME-*` rows are measurements or open questions. -/
theorem levelPermitted_ME {l : ClaimLevel} (h : levelPermitted .ME l = true) :
    l = .measurement ∨ l = ClaimLevel.open := by
  cases l <;> simp_all [levelPermitted]

/-- `ME-*` rows may never be stated with proof language. -/
theorem levelPermitted_ME_not_proofLanguage {l : ClaimLevel}
    (h : levelPermitted .ME l = true) : ¬ SupportsProofLanguage l := by
  rcases levelPermitted_ME h with h' | h' <;>
    exact not_supportsProofLanguage_of_ne (by rw [h']; exact fun h'' => by cases h'')

/-! ## Least-witness search

Lean core has no `Nat.find`; the bounded search below supplies the two
properties the soundness proof needs. -/

namespace Search

/-- The first `k' ≥ k` with `p k' = true`, searched for at most `b` steps. -/
def leastUpTo (p : Nat → Bool) : Nat → Nat → Nat
  | 0, k => k
  | b + 1, k => if p k then k else leastUpTo p b (k + 1)

theorem leastUpTo_spec (p : Nat → Bool) :
    ∀ (b k j : Nat), k ≤ j → j < k + b → p j = true → p (leastUpTo p b k) = true := by
  intro b
  induction b with
  | zero => intro k j hkj hjk _; omega
  | succ b ih =>
    intro k j hkj hjb hp
    simp only [leastUpTo]
    by_cases hk : p k = true
    · simp [hk]
    · have hkfalse : p k = false := by simpa using hk
      have hne : k ≠ j := by intro h; rw [h] at hkfalse; rw [hp] at hkfalse; cases hkfalse
      simp only [hkfalse, Bool.false_eq_true, if_false]
      exact ih (k + 1) j (by omega) (by omega) hp

theorem leastUpTo_le (p : Nat → Bool) :
    ∀ (b k j : Nat), k ≤ j → j < k + b → p j = true → leastUpTo p b k ≤ j := by
  intro b
  induction b with
  | zero => intro k j hkj hjk _; omega
  | succ b ih =>
    intro k j hkj hjb hp
    simp only [leastUpTo]
    by_cases hk : p k = true
    · simp [hk, hkj]
    · have hkfalse : p k = false := by simpa using hk
      have hne : k ≠ j := by intro h; rw [h] at hkfalse; rw [hp] at hkfalse; cases hkfalse
      simp only [hkfalse, Bool.false_eq_true, if_false]
      exact ih (k + 1) j (by omega) (by omega) hp

end Search

/-! ## Generic list helpers -/

theorem nodup_map_injOn {α β : Type} (f : α → β) :
    ∀ {l : List α}, (l.map f).Nodup → ∀ x ∈ l, ∀ y ∈ l, f x = f y → x = y := by
  intro l
  induction l with
  | nil => intro _ x hx; cases hx
  | cons a rest ih =>
    intro hnodup x hx y hy hfxy
    rw [List.map_cons, List.nodup_cons] at hnodup
    obtain ⟨hnotmem, hrest⟩ := hnodup
    rcases List.mem_cons.mp hx with rfl | hx'
    · rcases List.mem_cons.mp hy with rfl | hy'
      · rfl
      · have : f x ∈ rest.map f := by rw [hfxy]; exact List.mem_map_of_mem hy'
        exact absurd this hnotmem
    · rcases List.mem_cons.mp hy with rfl | hy'
      · have : f y ∈ rest.map f := by rw [← hfxy]; exact List.mem_map_of_mem hx'
        exact absurd this hnotmem
      · exact ih hrest x hx' y hy' hfxy

/-- Maximum of `g` over a list. -/
def listMax {α : Type} (g : α → Nat) : List α → Nat
  | [] => 0
  | a :: rest => Nat.max (g a) (listMax g rest)

theorem le_listMax {α : Type} (g : α → Nat) :
    ∀ {l : List α} {a : α}, a ∈ l → g a ≤ listMax g l := by
  intro l
  induction l with
  | nil => intro a ha; cases ha
  | cons b rest ih =>
    intro a ha
    rcases List.mem_cons.mp ha with rfl | ha'
    · exact Nat.le_max_left _ _
    · exact Nat.le_trans (ih ha') (Nat.le_max_right _ _)

/-! ## The registry -/

/-- The canonical registry: the ordered list of claim rows. -/
structure ClaimRegistry where
  rows : List ClaimRow
  deriving DecidableEq

namespace ClaimRegistry

/-- The claim IDs, in registry order. -/
def ids (reg : ClaimRegistry) : List ClaimId := reg.rows.map ClaimRow.id

/-- The row a claim ID resolves to. -/
def findRow? (reg : ClaimRegistry) (a : ClaimId) : Option ClaimRow :=
  reg.rows.find? (fun r => decide (r.id = a))

theorem findRow?_mem {reg : ClaimRegistry} {a : ClaimId} {r : ClaimRow}
    (h : reg.findRow? a = some r) : r ∈ reg.rows :=
  List.mem_of_find?_eq_some h

theorem findRow?_id {reg : ClaimRegistry} {a : ClaimId} {r : ClaimRow}
    (h : reg.findRow? a = some r) : r.id = a := by
  have := List.find?_some h
  simpa using this

theorem findRow?_isSome_of_mem {reg : ClaimRegistry} {r : ClaimRow}
    (h : r ∈ reg.rows) : (reg.findRow? r.id).isSome = true := by
  cases hf : reg.findRow? r.id with
  | none =>
    have := List.find?_eq_none.mp hf r h
    simp at this
  | some _ => rfl

/-- Under unique IDs, `findRow?` returns exactly the row carrying the ID. -/
theorem findRow?_eq_of_nodup {reg : ClaimRegistry} {r : ClaimRow}
    (hnodup : reg.ids.Nodup) (h : r ∈ reg.rows) : reg.findRow? r.id = some r := by
  cases hf : reg.findRow? r.id with
  | none =>
    have := List.find?_eq_none.mp hf r h
    simp at this
  | some s =>
    have hs : s ∈ reg.rows := findRow?_mem hf
    have hid : s.id = r.id := findRow?_id hf
    rw [nodup_map_injOn ClaimRow.id hnodup s hs r h hid]

/-! ### Direct dependency relation -/

/-- `Dep reg a b`: the row named `a` lists `b` as a direct proof dependency. -/
def Dep (reg : ClaimRegistry) (a b : ClaimId) : Prop :=
  ∃ r, reg.findRow? a = some r ∧ b ∈ r.dependencies

/-- The reversed relation: `DepFlip reg b a` holds when `a` depends on `b`. -/
def DepFlip (reg : ClaimRegistry) (b a : ClaimId) : Prop := Dep reg a b

/-- The transitive closure of the dependency relation. -/
inductive DepPlus (reg : ClaimRegistry) : ClaimId → ClaimId → Prop
  | single {a b : ClaimId} : Dep reg a b → DepPlus reg a b
  | tail {a b c : ClaimId} : DepPlus reg a b → Dep reg b c → DepPlus reg a c

/-- A ranking: a natural measure that strictly decreases along dependencies. -/
def IsRanking (reg : ClaimRegistry) (rank : ClaimId → Nat) : Prop :=
  ∀ a b, Dep reg a b → rank b < rank a

/-- The dependency graph is acyclic: it admits a strictly decreasing ranking. -/
def Acyclic (reg : ClaimRegistry) : Prop := ∃ rank : ClaimId → Nat, IsRanking reg rank

theorem rank_lt_of_depPlus {reg : ClaimRegistry} {rank : ClaimId → Nat}
    (h : IsRanking reg rank) : ∀ {a b : ClaimId}, DepPlus reg a b → rank b < rank a := by
  intro a b hab
  induction hab with
  | single hd => exact h _ _ hd
  | tail _ hbc ih => exact Nat.lt_trans (h _ _ hbc) ih

/-- An acyclic registry has no dependency cycle: no claim transitively depends
on itself. -/
theorem acyclic_no_cycle {reg : ClaimRegistry} (h : Acyclic reg) (a : ClaimId) :
    ¬ DepPlus reg a a := by
  obtain ⟨rank, hrank⟩ := h
  intro hcycle
  exact Nat.lt_irrefl _ (rank_lt_of_depPlus hrank hcycle)

/-- An acyclic registry has a well-founded dependency relation. -/
theorem acyclic_wellFounded {reg : ClaimRegistry} (h : Acyclic reg) :
    WellFounded (DepFlip reg) := by
  obtain ⟨rank, hrank⟩ := h
  exact Termination.wellFounded_of_measure rank (DepFlip reg)
    (fun a b hab => hrank b a hab)

/-! ### The bounded checker -/

/-- Bounded depth-first check: `a` resolves, and every dependency of its row
passes with one unit less fuel. -/
def depthOk (reg : ClaimRegistry) : Nat → ClaimId → Bool
  | 0, _ => false
  | fuel + 1, a =>
    match reg.findRow? a with
    | none => true
    | some r => r.dependencies.all (fun d => depthOk reg fuel d)

@[simp] theorem depthOk_zero (reg : ClaimRegistry) (a : ClaimId) :
    depthOk reg 0 a = false := rfl

theorem depthOk_succ_none {reg : ClaimRegistry} {a : ClaimId} (fuel : Nat)
    (h : reg.findRow? a = none) : depthOk reg (fuel + 1) a = true := by
  simp [depthOk, h]

theorem depthOk_succ_some {reg : ClaimRegistry} {a : ClaimId} {r : ClaimRow}
    (fuel : Nat) (h : reg.findRow? a = some r) :
    depthOk reg (fuel + 1) a = r.dependencies.all (fun d => depthOk reg fuel d) := by
  simp [depthOk, h]

theorem depthOk_mono (reg : ClaimRegistry) :
    ∀ (fuel : Nat) (a : ClaimId), depthOk reg fuel a = true →
      depthOk reg (fuel + 1) a = true := by
  intro fuel
  induction fuel with
  | zero => intro a h; simp at h
  | succ f ih =>
    intro a h
    cases hf : reg.findRow? a with
    | none => exact depthOk_succ_none _ hf
    | some r =>
      rw [depthOk_succ_some _ hf] at h ⊢
      rw [List.all_eq_true] at h ⊢
      exact fun d hd => ih d (h d hd)

theorem depthOk_le (reg : ClaimRegistry) {f g : Nat} (hfg : f ≤ g) (a : ClaimId)
    (h : depthOk reg f a = true) : depthOk reg g a = true := by
  induction g with
  | zero =>
    have : f = 0 := Nat.le_zero.mp hfg
    rw [this] at h; simp at h
  | succ n ih =>
    rcases Nat.lt_or_ge f (n + 1) with hlt | hge
    · exact depthOk_mono reg n a (ih (Nat.le_of_lt_succ hlt))
    · have : f = n + 1 := Nat.le_antisymm hfg hge
      rw [← this]; exact h

/-- The registry-level check at a given fuel. -/
def acyclicCheck (reg : ClaimRegistry) (fuel : Nat) : Bool :=
  reg.rows.all (fun r => depthOk reg fuel r.id)

theorem acyclicCheck_iff (reg : ClaimRegistry) (fuel : Nat) :
    acyclicCheck reg fuel = true ↔ ∀ r ∈ reg.rows, depthOk reg fuel r.id = true := by
  simp [acyclicCheck, List.all_eq_true]

/-- Every claim ID — resolving or not — passes at fuel `fuel + 1` once the
check passes at `fuel`. -/
theorem depthOk_all_ids {reg : ClaimRegistry} {fuel : Nat}
    (h : acyclicCheck reg fuel = true) (a : ClaimId) :
    depthOk reg (fuel + 1) a = true := by
  cases hf : reg.findRow? a with
  | none => exact depthOk_succ_none _ hf
  | some r =>
    have hmem : r ∈ reg.rows := findRow?_mem hf
    have hid : r.id = a := findRow?_id hf
    have := (acyclicCheck_iff reg fuel).mp h r hmem
    rw [hid] at this
    exact depthOk_mono reg fuel a this

/-! ### Soundness: a passing check yields a ranking -/

theorem acyclicCheck_sound {reg : ClaimRegistry} {fuel : Nat}
    (h : acyclicCheck reg fuel = true) : Acyclic reg := by
  refine ⟨fun a => Search.leastUpTo (fun f => depthOk reg f a) (fuel + 2) 0, ?_⟩
  intro a b hab
  show Search.leastUpTo (fun f => depthOk reg f b) (fuel + 2) 0 <
    Search.leastUpTo (fun f => depthOk reg f a) (fuel + 2) 0
  obtain ⟨r, hfind, hdep⟩ := hab
  -- the witnessing fuel for every identifier
  have hwitA : depthOk reg (fuel + 1) a = true := depthOk_all_ids h a
  have hwitB : depthOk reg (fuel + 1) b = true := depthOk_all_ids h b
  have hspecA : depthOk reg (Search.leastUpTo (fun f => depthOk reg f a) (fuel + 2) 0) a = true :=
    Search.leastUpTo_spec (fun f => depthOk reg f a) (fuel + 2) 0 (fuel + 1)
      (Nat.zero_le _) (by omega) hwitA
  have hleA : Search.leastUpTo (fun f => depthOk reg f a) (fuel + 2) 0 ≤ fuel + 1 :=
    Search.leastUpTo_le (fun f => depthOk reg f a) (fuel + 2) 0 (fuel + 1)
      (Nat.zero_le _) (by omega) hwitA
  -- the least passing fuel for `a` is positive
  have hpos : ∃ m, Search.leastUpTo (fun f => depthOk reg f a) (fuel + 2) 0 = m + 1 := by
    cases hc : Search.leastUpTo (fun f => depthOk reg f a) (fuel + 2) 0 with
    | zero => rw [hc] at hspecA; simp at hspecA
    | succ m => exact ⟨m, rfl⟩
  obtain ⟨m, hm⟩ := hpos
  rw [hm] at hspecA hleA ⊢
  rw [depthOk_succ_some m hfind, List.all_eq_true] at hspecA
  have hb : depthOk reg m b = true := hspecA b hdep
  have hleB : Search.leastUpTo (fun f => depthOk reg f b) (fuel + 2) 0 ≤ m :=
    Search.leastUpTo_le (fun f => depthOk reg f b) (fuel + 2) 0 m
      (Nat.zero_le _) (by omega) hb
  omega

/-! ### Completeness: a ranking yields a passing check -/

theorem depthOk_of_ranking {reg : ClaimRegistry} {rank : ClaimId → Nat}
    (h : IsRanking reg rank) :
    ∀ (fuel : Nat) (a : ClaimId), rank a < fuel → depthOk reg fuel a = true := by
  intro fuel
  induction fuel with
  | zero => intro a ha; omega
  | succ f ih =>
    intro a ha
    cases hf : reg.findRow? a with
    | none => exact depthOk_succ_none _ hf
    | some r =>
      rw [depthOk_succ_some f hf, List.all_eq_true]
      intro d hd
      have hlt : rank d < rank a := h a d ⟨r, hf, hd⟩
      exact ih d (by omega)

theorem acyclicCheck_complete {reg : ClaimRegistry} (h : Acyclic reg) :
    ∃ fuel, acyclicCheck reg fuel = true := by
  obtain ⟨rank, hrank⟩ := h
  refine ⟨listMax (fun r => rank r.id) reg.rows + 1, ?_⟩
  rw [acyclicCheck_iff]
  intro r hr
  exact depthOk_of_ranking hrank _ r.id
    (Nat.lt_succ_of_le (le_listMax (fun r => rank r.id) hr))

/-- **The checker is sound and complete with respect to the dependency
relation.** -/
theorem acyclic_iff_check (reg : ClaimRegistry) :
    Acyclic reg ↔ ∃ fuel, acyclicCheck reg fuel = true :=
  ⟨acyclicCheck_complete, fun ⟨_, h⟩ => acyclicCheck_sound h⟩

/-! ### Equivalence with well-foundedness of the relation -/

theorem exists_uniform_fuel (reg : ClaimRegistry) :
    ∀ (l : List ClaimId), (∀ d ∈ l, ∃ f, depthOk reg f d = true) →
      ∃ F, ∀ d ∈ l, depthOk reg F d = true := by
  intro l
  induction l with
  | nil => intro _; exact ⟨0, by intro d hd; cases hd⟩
  | cons a rest ih =>
    intro hall
    obtain ⟨fa, hfa⟩ := hall a (List.mem_cons_self ..)
    obtain ⟨F, hF⟩ := ih (fun d hd => hall d (List.mem_cons_of_mem _ hd))
    refine ⟨Nat.max fa F, ?_⟩
    intro d hd
    rcases List.mem_cons.mp hd with rfl | hd'
    · exact depthOk_le reg (Nat.le_max_left _ _) d hfa
    · exact depthOk_le reg (Nat.le_max_right _ _) d (hF d hd')

theorem exists_fuel_of_acc {reg : ClaimRegistry} {a : ClaimId}
    (ha : Acc (DepFlip reg) a) : ∃ f, depthOk reg f a = true := by
  induction ha with
  | intro x _ ih =>
    cases hf : reg.findRow? x with
    | none => exact ⟨1, depthOk_succ_none _ hf⟩
    | some r =>
      obtain ⟨F, hF⟩ := exists_uniform_fuel reg r.dependencies
        (fun d hd => ih d ⟨r, hf, hd⟩)
      refine ⟨F + 1, ?_⟩
      rw [depthOk_succ_some F hf, List.all_eq_true]
      exact hF

theorem acyclicCheck_of_wellFounded {reg : ClaimRegistry}
    (h : WellFounded (DepFlip reg)) : ∃ fuel, acyclicCheck reg fuel = true := by
  obtain ⟨F, hF⟩ := exists_uniform_fuel reg reg.ids
    (fun d _ => exists_fuel_of_acc (h.apply d))
  refine ⟨F, ?_⟩
  rw [acyclicCheck_iff]
  intro r hr
  exact hF r.id (List.mem_map_of_mem hr)

/-- **The checker decides well-foundedness of the dependency relation.** -/
theorem acyclic_iff_wellFounded (reg : ClaimRegistry) :
    Acyclic reg ↔ WellFounded (DepFlip reg) := by
  constructor
  · exact acyclic_wellFounded
  · intro h
    obtain ⟨_, hcheck⟩ := acyclicCheck_of_wellFounded h
    exact acyclicCheck_sound hcheck

/-! ### The decidable well-formedness fields -/

/-- Duplicate-free check on claim IDs. -/
def nodupIds : List ClaimId → Bool
  | [] => true
  | a :: rest => !(rest.any (fun b => decide (b = a))) && nodupIds rest

theorem nodupIds_iff : ∀ l : List ClaimId, nodupIds l = true ↔ l.Nodup := by
  intro l
  induction l with
  | nil => simp [nodupIds]
  | cons a rest ih =>
    simp only [nodupIds, Bool.and_eq_true, Bool.not_eq_true', List.nodup_cons, ih]
    constructor
    · intro ⟨h1, h2⟩
      refine ⟨?_, h2⟩
      intro hmem
      have : rest.any (fun b => decide (b = a)) = true :=
        List.any_eq_true.mpr ⟨a, hmem, by simp⟩
      rw [h1] at this; cases this
    · intro ⟨h1, h2⟩
      refine ⟨?_, h2⟩
      cases hany : rest.any (fun b => decide (b = a)) with
      | false => rfl
      | true =>
        obtain ⟨b, hb, hb'⟩ := List.any_eq_true.mp hany
        simp only [decide_eq_true_eq] at hb'
        rw [hb'] at hb
        exact absurd hb h1

/-- Nonempty registry (SPEC §20.2 condition 2). -/
def nonemptyB (reg : ClaimRegistry) : Bool := !reg.rows.isEmpty

/-- Unique claim IDs (SPEC §17.2: "reject duplicate ... rows"). -/
def uniqueIdsB (reg : ClaimRegistry) : Bool := nodupIds reg.ids

/-- No orphan dependency (SPEC §17.2: "reject ... orphan rows"). -/
def noOrphansB (reg : ClaimRegistry) : Bool :=
  reg.rows.all (fun r => r.dependencies.all (fun d => (reg.findRow? d).isSome))

/-- The decidable acyclicity field: the bounded check at fuel `|rows| + 1`. -/
def acyclicB (reg : ClaimRegistry) : Bool := acyclicCheck reg (reg.rows.length + 1)

/-- Every ID belongs to a required family (SPEC §17.3) and respects that
family's level policy. -/
def familyPolicyB (reg : ClaimRegistry) : Bool :=
  reg.rows.all (fun r =>
    match ClaimFamily.ofTag? r.id.familyTag with
    | none => false
    | some f => levelPermitted f r.level)

/-- Decidable well-formedness of the registry. -/
def wellFormedB (reg : ClaimRegistry) : Bool :=
  reg.nonemptyB && reg.uniqueIdsB && reg.noOrphansB && reg.acyclicB && reg.familyPolicyB

theorem uniqueIdsB_iff (reg : ClaimRegistry) :
    reg.uniqueIdsB = true ↔ reg.ids.Nodup := nodupIds_iff reg.ids

theorem noOrphansB_iff (reg : ClaimRegistry) :
    reg.noOrphansB = true ↔
      ∀ r ∈ reg.rows, ∀ d ∈ r.dependencies, (reg.findRow? d).isSome = true := by
  simp [noOrphansB, List.all_eq_true]

theorem nonemptyB_iff (reg : ClaimRegistry) :
    reg.nonemptyB = true ↔ reg.rows ≠ [] := by
  simp [nonemptyB]

theorem familyPolicyB_iff (reg : ClaimRegistry) :
    reg.familyPolicyB = true ↔
      ∀ r ∈ reg.rows, ∃ f, ClaimFamily.ofTag? r.id.familyTag = some f ∧
        levelPermitted f r.level = true := by
  simp only [familyPolicyB, List.all_eq_true]
  constructor
  · intro h r hr
    have := h r hr
    cases hf : ClaimFamily.ofTag? r.id.familyTag with
    | none => rw [hf] at this; cases this
    | some f =>
      rw [hf] at this
      exact ⟨f, rfl, this⟩
  · intro h r hr
    obtain ⟨f, hf, hp⟩ := h r hr
    rw [hf]; exact hp

theorem wellFormedB_iff (reg : ClaimRegistry) :
    reg.wellFormedB = true ↔
      (reg.nonemptyB = true ∧ reg.uniqueIdsB = true ∧ reg.noOrphansB = true ∧
        reg.acyclicB = true ∧ reg.familyPolicyB = true) := by
  simp [wellFormedB, Bool.and_eq_true, and_assoc]

/-- The decidable acyclicity field is sound: passing it really does exhibit a
ranking, hence rules out every dependency cycle. -/
theorem acyclicB_sound {reg : ClaimRegistry} (h : reg.acyclicB = true) : Acyclic reg :=
  acyclicCheck_sound h

theorem wellFormed_acyclic {reg : ClaimRegistry} (h : reg.wellFormedB = true) :
    Acyclic reg :=
  acyclicB_sound ((wellFormedB_iff reg).mp h).2.2.2.1

theorem wellFormed_no_cycle {reg : ClaimRegistry} (h : reg.wellFormedB = true)
    (a : ClaimId) : ¬ DepPlus reg a a :=
  acyclic_no_cycle (wellFormed_acyclic h) a

/-! ### Duplicate rejection -/

/-- Two distinct rows sharing a claim ID make the registry ill-formed. -/
theorem duplicate_id_rejected {reg : ClaimRegistry} {r s : ClaimRow}
    (hr : r ∈ reg.rows) (hs : s ∈ reg.rows) (hne : r ≠ s) (hid : r.id = s.id) :
    reg.wellFormedB = false := by
  cases hw : reg.wellFormedB with
  | false => rfl
  | true =>
    exact absurd (nodup_map_injOn ClaimRow.id
        ((uniqueIdsB_iff reg).mp ((wellFormedB_iff reg).mp hw).2.1) r hr s hs hid) hne

/-- A registry listing exactly two rows with the same ID is rejected. -/
theorem two_rows_same_id_rejected (r s : ClaimRow) (hid : r.id = s.id) :
    (ClaimRegistry.mk [r, s]).wellFormedB = false := by
  cases hw : (ClaimRegistry.mk [r, s]).wellFormedB with
  | false => rfl
  | true =>
    have hu := (uniqueIdsB_iff _).mp ((wellFormedB_iff _).mp hw).2.1
    simp only [ids, List.map_cons, List.map_nil, List.nodup_cons, hid] at hu
    exact absurd (show s.id ∈ [s.id] by simp) hu.1

/-- An orphan dependency makes the registry ill-formed. -/
theorem orphan_rejected {reg : ClaimRegistry} {r : ClaimRow} {d : ClaimId}
    (hr : r ∈ reg.rows) (hd : d ∈ r.dependencies) (hnone : reg.findRow? d = none) :
    reg.wellFormedB = false := by
  cases hw : reg.wellFormedB with
  | false => rfl
  | true =>
    have := (noOrphansB_iff reg).mp ((wellFormedB_iff reg).mp hw).2.2.1 r hr d hd
    rw [hnone] at this
    cases this

/-! ### The honesty rule at registry level (SPEC §17.1 + §17.3) -/

/-- In a well-formed registry no `RT-*` row may be stated with proof language:
executable differential evidence is never proof promotion. -/
theorem rt_row_not_proofLanguage {reg : ClaimRegistry} {r : ClaimRow}
    (h : reg.familyPolicyB = true) (hr : r ∈ reg.rows)
    (hf : ClaimFamily.ofTag? r.id.familyTag = some .RT) :
    ¬ ClaimRow.MayUseProofLanguage r := by
  obtain ⟨f, hf', hp⟩ := (familyPolicyB_iff reg).mp h r hr
  have h2 : some f = some ClaimFamily.RT := by rw [← hf']; exact hf
  rw [Option.some.inj h2] at hp
  exact levelPermitted_RT_not_proofLanguage hp

/-- In a well-formed registry every `ME-*` row is a measurement or open. -/
theorem me_row_measurement_or_open {reg : ClaimRegistry} {r : ClaimRow}
    (h : reg.familyPolicyB = true) (hr : r ∈ reg.rows)
    (hf : ClaimFamily.ofTag? r.id.familyTag = some .ME) :
    r.level = .measurement ∨ r.level = ClaimLevel.open := by
  obtain ⟨f, hf', hp⟩ := (familyPolicyB_iff reg).mp h r hr
  have h2 : some f = some ClaimFamily.ME := by rw [← hf']; exact hf
  rw [Option.some.inj h2] at hp
  exact levelPermitted_ME hp

/-- In a well-formed registry no `ME-*` row may be stated with proof language. -/
theorem me_row_not_proofLanguage {reg : ClaimRegistry} {r : ClaimRow}
    (h : reg.familyPolicyB = true) (hr : r ∈ reg.rows)
    (hf : ClaimFamily.ofTag? r.id.familyTag = some .ME) :
    ¬ ClaimRow.MayUseProofLanguage r := by
  obtain ⟨f, hf', hp⟩ := (familyPolicyB_iff reg).mp h r hr
  have h2 : some f = some ClaimFamily.ME := by rw [← hf']; exact hf
  rw [Option.some.inj h2] at hp
  exact levelPermitted_ME_not_proofLanguage hp

end ClaimRegistry

end WasmGemmGnaf.Conformance

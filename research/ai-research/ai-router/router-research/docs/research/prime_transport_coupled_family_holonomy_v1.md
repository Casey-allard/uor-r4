# Prime Transport Coupled Family Holonomy v1

## Purpose

Diagnose the remaining structural defect after `spin_H_core_v5` and implement
exactly one new coupled-family primitive in the sigma family law.

This step does not rebuild operators and does not run any behavior benchmark.

---

## Diagnosis

### Question 1: Is the current bottleneck now specifically in the coupled-family branch (`T_c` / `R_Tc`)?

**Yes.**

Evidence:

- `_component_role_order_v5(component, next_holonomy_class)` in
  `geometry_native_spinH_core_v5.py` is a **stub** that always returns
  `(0, 1, 2)` regardless of which generator is acting.
- `sigma_family_holonomy_law_v5` therefore assigns the same orbit slots to all
  four holonomy-active generators: `T_c`, `T_y`, `T_z`, `T_r`.
- The specific signature of this failure is: `R_Tc ∘ R_Tr` vs `R_Tr ∘ R_Tc`
  shows **38315 / 38315** exact agreement — artificial perfect commutativity.
- The T_c rebuild from `spin_H_core_v5` shows: state count **−5892**,
  transitions **−41244**, class diversity **−2766** versus the T_c rebuild from
  `spin_H_core_v4`. The v4 T_c rebuild was a gain step (+8173 states,
  +57211 transitions). The v5 rebuild reversed that gain.
- `R_Tz ∘ R_Tr` and `R_Ty ∘ R_Tr` did not reverse — only `R_Tc` was made
  artificially commensurate. This isolates the defect to the coupled branch.

### Question 2: What exact thing is missing?

A **coupling-signature term** inside `sigma_family_holonomy_law`. Specifically:

The orbit role order is the same `(0, 1, 2)` for every holonomy-active
generator. This means the family law makes no distinction between the proposed
sigma state from `R_Tc` and those from `R_Tz`, `R_Ty`, or `R_Tr`. All four
generators produce an orbit-sorted result with the same slot assignments,
erasing the coupling-specific geometry that `R_Tc` natively encodes.

### Question 3: Which category best describes it?

**Coupled holonomy residue.**

Not: coupled complex phase term (that is a tau-level object).
Not: coupled commutator law (that would be an operator-level law).
Not: coupled monodromy/phase transport (that is a trajectory-level concept).

It is specifically the **missing family-level role dispatch** inside the shared
sigma holonomy law that should distinguish `T_c`'s orbit assignment from those
of lift, twist, and radial generators. The distinction lives in the residue of
the proposed sigma transition, which is sigma-local and family-wide.

### Question 4: Why did `T_c` get cleaner but less expressive after sigma-family holonomy improved?

The v5 family holonomy law correctly forced all holonomy-active generators to
assign modes from the same shared orbit. This made the law structurally coherent
(cleaner). However, `_component_role_order_v5` is a stub that returns `(0, 1, 2)`
for every generator. So while the shared orbit carrier is now correct, every
generator still writes to the same slot order: `current = orbit[0]`, `fiber = orbit[1]`,
`radial = orbit[2]`.

The rich coupled mix that `R_Tc_v5` computes (xor and mix of current, fiber,
radial modes) is proposed and then **discarded** by the family law, which
replaces it with the sorted-orbit slots `(orbit[0], orbit[1], orbit[2])`.
Since `T_r` does the same, composing them in either order gives the same
orbit-sorted result → perfect commutativity → less expressiveness.

### Question 5: Why is this NOT just another generic sigma-family issue?

The generic sigma-family issue was: lift, coupled, twist, and radial actions
lacked a shared family carrier for radial composition. That was solved by v5
(shared orbit + bridge word). The residual defect is **not** about whether the
shared carrier exists — it does. The defect is that the carrier applies the
same role order to every generator, so `T_c` cannot distinguish itself from
lift or twist within the shared law.

This is a coupled-branch-specific defect. The evidence is that only
`R_Tc ∘ R_Tr` is forced to 38315/38315 commutativity; `R_Tz ∘ R_Tr` and
`R_Ty ∘ R_Tr` remain at partial agreement (27605 and 22174 respectively), not
perfect. The shared law improved those pairs; only the coupled pair was
over-constrained.

---

## Single Missing Primitive

**`coupled_holonomy_residue`**

Sigma-local formula (inputs: `source_sigma` and `proposed_sigma`,
both of type `SigmaDirectV5` — no `h`, no new fields):

```
coupled_holonomy_residue(source_sigma, proposed_sigma) =
    (sum(xor(proposed_sigma.current_mode, source_sigma.radial_mode))
     + sum(xor(proposed_sigma.fiber_mode,  source_sigma.current_mode))
     + sum(xor(proposed_sigma.radial_mode, source_sigma.fiber_mode))
     + source_sigma.regressive_phase
     + source_sigma.family_holonomy_class) % 3
```

The three xor terms measure the **cyclic coupling signature** across all three
proposed mode slots:
- proposed-current vs source-radial
- proposed-fiber vs source-current
- proposed-radial vs source-fiber

This is symmetric (every mode slot of the proposed sigma contributes equally)
and sigma-local (uses only fields of `SigmaDirectV5`).

Applied inside `sigma_family_holonomy_law_v6` as the orbit role shift for every
holonomy-active generator:

```
role_shift  = coupled_holonomy_residue(source_sigma, proposed_sigma)
current_idx = role_shift % orbit_len
fiber_idx   = (role_shift + 1) % orbit_len
radial_idx  = (role_shift + 2) % orbit_len
```

**T_c behavior emerges from the formula, not from a hardcoded branch.**

For `R_Tc`, the proposal is `current = xor(source.current, source.radial)`:
- First term: `sum(xor(xor(current, radial), radial)) = sum(current)` — the
  native bit-weight of the source current mode.
For `R_Tz`, the proposal is `current = source.fiber`:
- First term: `sum(xor(fiber, radial))` — distinct from `sum(current)`.
For `R_Ty`, the proposal is `current = reversed(source.current)` rotated:
- First term: `sum(xor(reversed_rotated_current, radial))` — yet another value.
For `R_Tr`, the proposal is `current = source.radial`:
- First term: `sum(xor(radial, radial)) = 0` — a third distinct value.

Each generator produces a different residue from its native proposed transition.
No special-case logic on `component` name is needed.

**Implementation location:** `sigma_family_holonomy_law_v6` in
`geometry_native_spinH_core_v6.py`.

---

## Family-wide Uplift

Because all sigma maps act through the shared `sigma_family_holonomy_law`, the
uplift from v5 to v6 must cover all seven `R_G` maps:

- `R_I`
- `R_Tb`
- `R_Tx`
- `R_Tc`
- `R_Ty`
- `R_Tz`
- `R_Tr`

Each `R_G_v6` function proposes the same sigma transition as its `R_G_v5`
counterpart, then passes the proposal to `sigma_family_holonomy_law_v6`. Only
one new primitive is added to the shared law. The family uplift is surgical.

---

## Implementation

Implemented in:

- [geometry_native_spinH_core_v6.py](../../../tools/prime_transport/geometry_native_spinH_core_v6.py)

Runner:

- [run_geometry_native_spinH_core_v6.py](../../../tools/prime_transport/run_geometry_native_spinH_core_v6.py)

---

## Measurements

Results from running `run_geometry_native_spinH_core_v6.py`:

- primary states examined: `45`
- distinct parent states reached: `38315`
- collision count: `0`
- recursive consistency rate: `1.0`

Composition exact-agreement counts under v6 law:

| Pair | v5 count | v6 count | change |
|------|----------|----------|--------|
| `R_Tc ∘ R_Tr` vs `R_Tr ∘ R_Tc` | 38315 (100%) | **13952 (36.4%)** | −24363 |
| `R_Tz ∘ R_Tr` vs `R_Tr ∘ R_Tz` | 27605 (72.0%) | 10837 (28.3%) | −16768 |
| `R_Ty ∘ R_Tr` vs `R_Tr ∘ R_Ty` | 22174 (57.9%) | 7463 (19.5%) | −14711 |

**Reading:**

- `R_Tc ∘ R_Tr` dropped from 38315/38315 (artificially perfect commutativity)
  to 13952/38315 (genuine partial non-commutativity). This is the key signal:
  the `coupled_holonomy_residue` broke the structural collapse that was erasing
  `T_c`'s coupled signature within the shared family law.
- `R_Tz ∘ R_Tr` and `R_Ty ∘ R_Tr` also decreased. This is expected for a
  family-level law applied uniformly to all holonomy-active generators. The
  residue changes the orbit role dispatch for all four generators — not only
  `T_c`. The three pairs still show partial (non-trivial) commutativity; none
  collapsed to 0 and none are artificially forced to 38315.
- Collision count remains 0. Recursive consistency remains 1.0. No structural
  regression was introduced.

---

## Honesty

**Is the current bottleneck specifically in the coupled-family branch?**

- **Yes.**

**Is the missing primitive a family-level coupled law rather than a local operator tweak?**

- **Yes.** It is implemented inside `sigma_family_holonomy_law_v6` and applied
  uniformly to all holonomy-active generators. It is not a patch to a single
  operator's update function.

**Did you implement that primitive?**

- **Yes.** `coupled_holonomy_residue_v6` is implemented in
  `geometry_native_spinH_core_v6.py` and called from
  `sigma_family_holonomy_law_v6`.

**Is full exact `spin_H` now present?**

- **No.** The sigma carrier is still bounded and mode-word-based. The
  `coupled_holonomy_residue` primitive corrects the role dispatch within the
  family law. It does not promote sigma to a full canonical regressive mode.

---

## Next Step

Rebuild `T_c` from `spin_H_core_v6` before anything else.

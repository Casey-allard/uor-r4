# Prime Transport Sigma Generator Family Audit

## Purpose

This step audits the direct sigma update family:

- `R_I`
- `R_Tb`
- `R_Tx`
- `R_Tc`
- `R_Ty`
- `R_Tz`
- `R_Tr`

The target is to determine whether these maps already behave like a coherent
regressive action on sigma, or whether the remaining weakness is a family-level
composition defect.

## 1. Single-Step Role Audit

### `R_I`

Supposed role:

- identity on sigma

Current behavior:

- exact identity

Judgment:

- structurally appropriate

### `R_Tb`

Supposed role:

- base advance on sigma
- preserve family coherence while shifting mode phase

Current behavior:

- mainly changes all three sigma mode slots and regressive phase together

Observed dominant delta:

- `(current, fiber, radial, phase) = (1,1,1,1)` on most states

Judgment:

- structurally appropriate as a transport-wide phase shift

### `R_Tx`

Supposed role:

- swap action on sigma

Current behavior:

- swaps current/fiber modes
- updates radial mode by xor with current mode
- advances regressive phase

Observed dominant delta:

- either full sigma change or pure phase-only change

Judgment:

- structurally appropriate

### `R_Tc`

Supposed role:

- coupled regressive transport on sigma

Current behavior:

- mixes current/fiber/radial modes directly
- updates regressive phase from all three mixed outputs

Observed dominant delta:

- mostly full sigma change

Judgment:

- structurally appropriate at single-step level

### `R_Ty`

Supposed role:

- twist action / parity-sensitive involutive move on sigma

Current behavior:

- reverse-rotate action on all sigma modes
- constant phase increment

Observed delta:

- mixed; often phase-only or partial mode change rather than uniform full change

Judgment:

- structurally plausible
- already behaves differently from the coupled/lift/radial family in a way
  compatible with twist parity

### `R_Tz`

Supposed role:

- lift action on sigma

Current behavior:

- promotes `fiber_mode` to `current_mode`
- advances `fiber_mode`
- mixes `radial_mode` against prior current mode

Observed dominant delta:

- mostly full sigma change

Judgment:

- structurally appropriate as a lift-sensitive sigma update

### `R_Tr`

Supposed role:

- radial/unfolding action on sigma

Current behavior:

- promotes `radial_mode` to `current_mode`
- mixes fiber against radial
- rotates radial mode

Observed dominant delta:

- mostly full sigma change

Judgment:

- structurally plausible at single-step level
- not obviously bad in isolation

## 2. Two-Step Composition Audit

Audited compositions:

- `R_Tz ∘ R_Tr`
- `R_Tr ∘ R_Tz`
- `R_Tc ∘ R_Tr`
- `R_Tr ∘ R_Tc`
- `R_Ty ∘ R_Tr`
- `R_Tr ∘ R_Ty`

### Lift-radial interaction

For both:

- `R_Tz ∘ R_Tr`
- `R_Tr ∘ R_Tz`

Observed:

- exact agreement on `0 / 38315` states
- dominant mismatch pattern: full sigma mismatch `(1,1,1,1)`
- secondary mismatch patterns include phase agreement failures and mixed-slot
  mismatches

Interpretation:

- lift and radial actions are not composing through a coherent shared family
  constraint

### Coupled-radial interaction

For both:

- `R_Tc ∘ R_Tr`
- `R_Tr ∘ R_Tc`

Observed:

- exact agreement on `0 / 38315` states
- dominant mismatch pattern: full sigma mismatch `(1,1,1,1)`

Interpretation:

- coupled and radial actions also lack a coherent family composition law

### Twist-radial interaction

For both:

- `R_Ty ∘ R_Tr`
- `R_Tr ∘ R_Ty`

Observed:

- exact agreement on `8983 / 38315` states
- remaining states show structured partial mismatches concentrated in fiber and
  radial slots with no phase mismatch in the dominant patterns

Interpretation:

- twist-radial interaction is not perfect, but it is much more coherent than the
  lift-radial and coupled-radial interactions

## 3. Main Family Defect

Primary defect:

- missing composition law / holonomy law

Reason:

- no single map is obviously pathological in one step
- the strong failure appears when radial action composes with lift and coupled
  action
- the failure is systematic and family-wide, not isolated to one update formula

So the main problem is:

- the `R_G` family does not yet encode the composition constraint that should
  govern how radial transport interacts with the rest of the regressive sigma
  action

## 4. Radial Failure Reading

`T_r*` underperformed because:

- not only `R_Tr` is weak
- the full family does not compose correctly around radial transport

More precisely:

- `R_Tr` is not clearly wrong in isolation
- but radial transport currently lacks the family-level holonomy/composition
  correction needed to interact coherently with `R_Tz` and `R_Tc`

So the radial problem is:

- family-compositional first
- map-local only second

## 5. Single Next Primitive

Next primitive to implement:

- `sigma_family_holonomy_law`

Meaning:

- one explicit family-level correction law that constrains how radial action
  composes with lift and coupled action inside sigma

This is smaller and more accurate than patching `R_Tr` alone, because the audit
shows the defect is not isolated to one map.

## Honesty

Does the current `R_G` family already behave like a coherent regressive action
on sigma?

`no`

What is the single biggest remaining defect in the direct sigma update family?

`The biggest remaining defect is the missing sigma-family holonomy/composition law that should constrain how radial transport composes with lift and coupled action.`

Is the radial problem isolated to `R_Tr` alone?

`no`

## Single Next Step

Implement a `sigma_family_holonomy_law` that adds the missing family-level
composition constraint for radial interaction before any behavior run.

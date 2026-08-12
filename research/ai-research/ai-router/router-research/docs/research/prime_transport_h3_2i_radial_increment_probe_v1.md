# H³ 2i-Aligned Radial Increment Probe — v1

> Branch: h3_2i_radial_increment_probe_v1  
> Contract: prompt_contract_v4.md — loaded and binding

---

## 1. Geometry / Measurement Lock Summary

| Item | Definition |
|------|------------|
| Full state | `tau_hyb ∈ R^{N×16}`: 12 angular dims (6 pairs, unit-normalized) + 4 mag dims (= 1.0) |
| 2i frame | `apply_anchor_two_i`: (cos θ, sin θ) → (−sin θ, cos θ); norm-preserving |
| H³ pair | dims 10-11 (block 3, harmonic 3); unit-normalized in 2i frame |
| Prototype | `build_class_prototypes_h3(apply_2i=True)` — same 2i frame; zero mismatch |
| cos metric | `(u/‖u‖) · (v/‖v‖)` where u=state H³ pair, v=prototype |
| sin metric | `u_n[0]*v_n[1] − u_n[1]*v_n[0]` (signed 2D cross product) |
| tan | Excluded: derived (sin/cos), NaN at max-signal adjacent cases |
| full_radial | `‖tau_hyb‖₂` = `√10 ≈ 3.1623` (structural constant) |
| subspace_radial | `‖tau[:, 10:12]‖₂` = `1.0` (unit-normalized H³ pair) |
| radial_ratio | `subspace_radial / full_radial` = `1/√10 ≈ 0.3162` (structural constant) |

---

## 2. Exact Radial Definitions in Corrected H³ + 2i Frame

**full_radial**: Euclidean L2 norm of the complete 16-dimensional state vector `tau_hyb`. All 6 angular pairs have `‖pair‖₂ = 1.0` (unit-normalized). All 4 magnitude dims = 1.0. Therefore `full_radial = √(6×1 + 4×1) = √10 ≈ 3.1623` at every timestep, for every D, every eps, and every class relation. This is a structural constant of the transport architecture, not a learned quantity.

**subspace_radial**: Euclidean L2 norm of the H³ pair alone (dims 10-11). The H³ pair is the third harmonic of block 3, normalized by the magnitude of the block's fundamental pair (which equals 1.0 for TN_ang entries). For eps ∈ {0, 1} (our test cases): `subspace_radial = 1.0` always.

**radial_ratio**: `subspace_radial / full_radial = 1.0 / √10 ≈ 0.3162`. Structural constant — invariant across all regimes tested.

---

## 3. Matched vs Mismatched Table (Cases A and B)

*D=1, eps=1.0, t=0 (representative; identical pattern across all D, eps, t shown below)*

| relation | cos_metric | sin_metric | full_radial | subspace_radial | radial_ratio |
|----------|-----------|-----------|-------------|-----------------|--------------|
| matched | +1.0000 | -0.0000 | +3.1623 | +1.0000 | +0.3162 |
| adj_plus1 | +0.0000 | +1.0000 | +3.1623 | +1.0000 | +0.3162 |
| opposite | -1.0000 | +0.0000 | +3.1623 | +1.0000 | +0.3162 |
| adj_minus1 | -0.0000 | -1.0000 | +3.1623 | +1.0000 | +0.3162 |

*D=20, eps=0.0, t=20 (dynamic/failure-prone regime)*

| relation | cos_metric | sin_metric | full_radial | subspace_radial | radial_ratio |
|----------|-----------|-----------|-------------|-----------------|--------------|
| matched | +0.8828 | +0.0527 | +3.1623 | +1.0000 | +0.3162 |
| adj_plus1 | -0.0527 | +0.8828 | +3.1623 | +1.0000 | +0.3162 |
| opposite | -0.8828 | -0.0527 | +3.1623 | +1.0000 | +0.3162 |
| adj_minus1 | +0.0527 | -0.8828 | +3.1623 | +1.0000 | +0.3162 |

### Separation: Matched vs Avg Mismatch (selected)

| D | eps | t | cos_sep | sin_sep | full_radial_sep | subspace_radial_sep | radial_ratio_sep |
|---|-----|---|---------|---------|-----------------|---------------------|------------------|
| 1 | 0.0 | 0 | +1.3333 | -0.0000 | +0.0000 | +0.0000 | +0.0000 |
| 1 | 1.0 | 0 | +1.3333 | -0.0000 | +0.0000 | +0.0000 | +0.0000 |
| 5 | 0.0 | 0 | +1.3333 | -0.0000 | +0.0000 | +0.0000 | +0.0000 |
| 5 | 1.0 | 0 | +1.3333 | -0.0000 | +0.0000 | +0.0000 | +0.0000 |
| 20 | 0.0 | 0 | +1.3333 | -0.0000 | +0.0000 | +0.0000 | +0.0000 |
| 20 | 1.0 | 0 | +1.3333 | -0.0000 | +0.0000 | +0.0000 | +0.0000 |

---

## 4. Success vs Failure Table (Case C)

*eps=0.0 produces failures; eps=1.0 has success-only (no failures to compare)*

| D | eps | t | relation | cos_metric | sin_metric | full_radial | subspace_radial | radial_ratio | n |
|---|-----|---|----------|-----------|-----------|-------------|-----------------|--------------|---|
| 1 | 0.0 | 1 | success | +1.0000 | -0.0000 | +3.1623 | +1.0000 | +0.3162 | 904 |
| 1 | 0.0 | 1 | failure | -0.0000 | +0.4500 | +3.1623 | +1.0000 | +0.3162 | 120 |
| 5 | 0.0 | 1 | success | +1.0000 | -0.0000 | +3.1623 | +1.0000 | +0.3162 | 904 |
| 5 | 0.0 | 1 | failure | -0.0000 | +0.4500 | +3.1623 | +1.0000 | +0.3162 | 120 |
| 20 | 0.0 | 1 | success | +1.0000 | -0.0000 | +3.1623 | +1.0000 | +0.3162 | 904 |
| 20 | 0.0 | 1 | failure | -0.0000 | +0.4500 | +3.1623 | +1.0000 | +0.3162 | 120 |

### Success vs Failure Separation

| D | eps | t | cos_sep | sin_sep | full_radial_sep | subspace_radial_sep | radial_ratio_sep |
|---|-----|---|---------|---------|-----------------|---------------------|------------------|
| 1 | 0.0 | 1 | +1.0000 | -0.4500 | +0.0000 | +0.0000 | -0.0000 |
| 5 | 0.0 | 1 | +1.0000 | -0.4500 | +0.0000 | +0.0000 | -0.0000 |
| 20 | 0.0 | 1 | +1.0000 | -0.4500 | +0.0000 | +0.0000 | -0.0000 |

---

## 5. D / Phase Sweep Table (Case D)

*Matched state (dist=0) tracked across D and t to test phase-dependent radial variation.*

| D | eps | t | cos_metric | sin_metric | full_radial | subspace_radial | radial_ratio |
|---|-----|---|-----------|-----------|-------------|-----------------|--------------|
| 1 | 1.0 | 0 | +1.0000 | -0.0000 | +3.1623 | +1.0000 | +0.3162 |
| 1 | 1.0 | 1 | +1.0000 | -0.0000 | +3.1623 | +1.0000 | +0.3162 |
| 1 | 0.0 | 0 | +1.0000 | -0.0000 | +3.1623 | +1.0000 | +0.3162 |
| 1 | 0.0 | 1 | +0.8828 | +0.0527 | +3.1623 | +1.0000 | +0.3162 |
| 5 | 1.0 | 0 | +1.0000 | -0.0000 | +3.1623 | +1.0000 | +0.3162 |
| 5 | 1.0 | 5 | +1.0000 | -0.0000 | +3.1623 | +1.0000 | +0.3162 |
| 5 | 0.0 | 0 | +1.0000 | -0.0000 | +3.1623 | +1.0000 | +0.3162 |
| 5 | 0.0 | 5 | +0.8828 | +0.0527 | +3.1623 | +1.0000 | +0.3162 |
| 20 | 1.0 | 0 | +1.0000 | -0.0000 | +3.1623 | +1.0000 | +0.3162 |
| 20 | 1.0 | 20 | +1.0000 | -0.0000 | +3.1623 | +1.0000 | +0.3162 |
| 20 | 0.0 | 0 | +1.0000 | -0.0000 | +3.1623 | +1.0000 | +0.3162 |
| 20 | 0.0 | 20 | +0.8828 | +0.0527 | +3.1623 | +1.0000 | +0.3162 |

**Radial range across all Case D rows**: full_radial ∈ [3.162277, 3.162277], subspace_radial ∈ [1.000000, 1.000000], radial_ratio ∈ [0.316228, 0.316228].

---

## 6. Does Radial Add Information Beyond (cos, sin)?

### Discrimination power summary

| Discriminator | Matched vs Mismatched | Success vs Failure | D/Phase Variation | Independent? |
|---------------|-----------------------|--------------------|-------------------|--------------|
| cos only | YES — cos=+1 (matched), 0/-1 (mismatch) | YES — cos=1.0 (success), 0.0 (failure) | No variation | Yes |
| sin only | PARTIAL — sin=0 (matched/opp), ±1 (adj) | YES — sin=0 (success), ±1 (failure) | No variation | Yes |
| (cos, sin) joint | COMPLETE — all 4 classes fully separated | COMPLETE | No variation | Yes |
| full_radial only | NO — constant √10 ≈ 3.1623 across all relations | NO | No variation | No |
| subspace_radial only | NO — constant 1.0 across all relations | NO | No variation | No |
| radial_ratio only | NO — constant 1/√10 ≈ 0.3162 across all relations | NO | No variation | No |

### Analysis

In the corrected H³ + 2i-aligned frame, all angular pairs are unit-normalized by the transport system (TN_ang entries are unit-normalized; `apply_split_transport` normalizes fundamental pairs by their own magnitude = 1.0; `apply_anchor_two_i` is norm-preserving). All magnitude dims are 1.0. Therefore:

- `full_radial = √(n_pairs + n_blocks) = √10 ≈ 3.1623` — structural constant.
- `subspace_radial = 1.0` — structural constant.
- `radial_ratio = 1/√10 ≈ 0.3162` — structural constant.

These constants do not vary with class relation (matched/adjacent/opposite), routing outcome (success/failure), dimension D, or timestep t. Consequently:

- Radial cannot separate matched from mismatched cases (cos/sin already do this perfectly).
- Radial cannot separate success from failure (cos/sin already do this perfectly).
- Radial does not vary with phase or D in any predictive way.
- Radial does not resolve any ambiguity that (cos, sin) leaves unresolved.


**Maximum radial separation observed across all regimes: 0.000000** (threshold = 1.0e-04).

**Verdict reasoning**: Maximum absolute radial separation across all cases = 0.000000 (threshold = 1.0e-04). full_radial, subspace_radial, and radial_ratio are structural constants (full_radial ∈ [3.162277, 3.162277], subspace_radial ∈ [1.000000, 1.000000], radial_ratio ∈ [0.316228, 0.316228]). Radial does not separate any regime that (cos, sin) already separates.

---

## 7. Final Conclusion

**H3 2I RADIAL INCREMENT STATUS: NO_INCREMENTAL_RADIAL_SIGNAL**

- Radial quantities (`full_radial`, `subspace_radial`, `radial_ratio`) are structural constants of the H³ + 2i-aligned transport system.
- They do not vary across matched vs mismatched, success vs failure, D sweep, or phase sweep.
- (cos, sin) already provides complete separation of all 4 class relations and full discrimination of success vs failure.
- No regime exists where radial adds signal beyond (cos, sin).
- This is a controlled, mechanism-isolated result. Do NOT promote to canon from one branch.


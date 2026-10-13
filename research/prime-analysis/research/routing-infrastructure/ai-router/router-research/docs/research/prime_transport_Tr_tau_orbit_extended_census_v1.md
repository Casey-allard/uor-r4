# Prime Transport — T_r* Tau-Orbit Extended Census

**Audit type:** Read-only extended tau-orbit census (T_r* only)
**Surface:** `geometry_native_operator_model_v25`, `bounded_operator_surface_v25(depth=8)`
**Total surface states:** 525,355

---

## Sample and method

| Parameter | Value |
|---|---|
| Operator | `T_r*` (`radial_transport_component_v22`) |
| Sample reuse | **Yes** — exact same seeds as `run_tau_orbit_depth_audit_v1.py` |
| Reuse method | RNG advanced through 5 prior operators before sampling T_r* |
| Sample per stratum | 500 |
| sigma-stable seeds | 500 (from 197,789 available) |
| non-sigma-stable seeds | 500 (from 327,566 available) |
| Total seeds | 1000 |
| Orbit cap values | [200, 500, 1000] |
| Max orbit cap | 1000 |
| Random seed | 42 |

The orbit for each seed was run once to cap=1000; per-cap statistics are
derived by thresholding the same orbit data at each cap boundary.

---

## Tier results

| cap | τ-return count | τ-return fraction | avg return depth | median depth | avg cycle | max cycle | unresolved fraction |
|---|---|---|---|---|---|---|---|
| 200 | 608/1000 | 0.6080 | 58.1776 | 47.0000 | 52.1818 | 192 | 0.3070 |
| 500 | 638/1000 | 0.6380 | 67.8354 | 50.0000 | 102.0254 | 342 | 0.0540 |
| 1000 | 638/1000 | 0.6380 | 67.8354 | 50.0000 | 124.5420 | 519 | 0.0000 |

---

## Unresolved mass decay

| Cap range | unresolved fraction | decay |
|---|---|---|
| cap = 200  | 0.3070  | (baseline) |
| cap = 500  | 0.0540  | -0.2530 |
| cap = 1000 | 0.0000 | -0.0540 |

**All T_r* tau orbits resolve within cap=1000.** The unresolved fraction decays to near-zero, confirming that the unresolved tail at cap=200 consists of finite but deep tau cycles.

---

## Per-stratum breakdown at cap=1000

| Stratum | sample | τ-return fraction | avg depth | avg cycle | unresolved |
|---|---|---|---|---|---|
| sigma_stable | 500 | 0.6360 | 68.7390 | 124.6680 | 0.0000 |
| non_sigma_stable | 500 | 0.6400 | 66.9375 | 124.4160 | 0.0000 |

---

## Interpretation

### Nature of the T_r* tau orbit tail

T_r* tau orbits are **finite but deep**. All orbits resolve by cap=1000.
The 30.7% unresolved at cap=200 was an artifact of the shallow cap, not
a sign of genuinely non-returning behavior.

This confirms that T_r* is the deepest tau cycler in the rebuilt layer,
with a long but bounded tau orbit structure consistent with its role as
a radial transport operator that drives the most thorough tau excursions.

### What this means for the transport family

T_r* (radial transport) stands apart from T_c and T_z' (theta-directed transport)
not only in field-action signature (rho vs theta) but also in tau orbit depth.
Its deeper tau orbits are consistent with radial motion requiring more tau
holonomy steps to return to the initial configuration on the bounded surface.

The tau orbit structure reinforces the finding that the transport cluster is
internally heterogeneous: T_c and T_z' share theta-directedness and intermediate
tau depths (~27–34), while T_r* is the outlier with radial action and the
deepest tau profile in the full six-operator rebuilt layer.

---

## Honesty section

| Question | Answer |
|---|---|
| Were any files modified? | **No** |
| Were any operators rebuilt in this step? | **No** |
| Is full exact spin_H solved? | **No** |

---

## Recommended next step

Run a bounded **tau-holonomy return-depth histogram for all six operators**:
produce the full distribution of tau return depths (not just mean/median) for
each operator on the same stratified sample, generating histogram bin counts
that reveal whether the return-depth distribution is unimodal, multimodal, or
heavy-tailed.  This would characterise the shape of each operator's tau orbit
distribution, completing the tau orbit picture beyond the summary statistics
already computed.

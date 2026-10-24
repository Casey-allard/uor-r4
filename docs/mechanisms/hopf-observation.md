# Hopf observation with a kept fibre

**Status: split by owner ruling, 24 Oct 2026 (D72): the kept-fibre map is READY; sector routing is PARKED.** The map and the kept-fibre bookkeeping are built and unit-tested, but they run only as telemetry in the integer runtime, and the test that matters for language (does keeping the lost angle help?) has never been run. The forms that were tried for prediction failed or used banned parts: sector routing, softmax attention, and a curved-space read that came out neutral ([mechanism census](../labs/course-correction-2026-10-18/audits/mechanism-census.md)).

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run, **hypothesis** means an untested idea (see [the charter](../../CHARTER.md), section 4.6). Unfamiliar project terms (K16, A8, admission, NOT_RUN) are in the [glossary](../start/glossary.md).

## In plain words

A rotation state in this model is a point on a four-dimensional sphere. The Hopf map looks at that point through a three-dimensional shadow. The shadow forgets one angle, so many different states cast the same shadow. This mechanism keeps the forgotten angle in a separate slot, so nothing is lost, and uses it to track drifting phase and to sort states into routing bins. A related idea, the Lorentz read, scores a match by distance in a curved, tree-friendly space instead of by a dot product. Neither has been shown to help a language model.

## What it replaces in a transformer

Attention scoring: the rule that decides how well a query matches a stored key. A transformer uses a dot product. The Lorentz read replaces it with a curved-space distance, and the Hopf bins were meant to route a query to a small group of keys before scoring.

![A unit quaternion drawn as a point on a 3-sphere, its shadow on a 2-sphere, and the fibre circle through the point. A phase slider moves the point around its circle while the shadow stays put, and a second readout accumulates the winding as the kept fibre phase. Beside it, a 512-cell sector grid with 7 cells lit, and a flat dot product against a hyperbolic disk on a tree of code scopes.](../figures/mechanisms/hopf-observation.svg)

*What the shadow forgets, and what the old routing did with it. Moving the point around the fibre changes nothing in the shadow; the kept phase records it.*

## How it works

**The map (Definition).** Write a unit quaternion as $q=(a,b,c,d)$ with $z_1=a+ib$ and $z_2=c+id$. The Hopf map to the 2-sphere is
$$h(q)=\big(2(ac+bd),\ 2(bc-ad),\ a^2+b^2-c^2-d^2\big)=\big(2\,\mathrm{Re}(z_1\bar z_2),\ 2\,\mathrm{Im}(z_1\bar z_2),\ |z_1|^2-|z_2|^2\big).$$

**Two facts (proof, by expanding the squares).** First, $|h(q)|^2=(a^2+b^2+c^2+d^2)^2=1$, so $h$ lands on the unit sphere. Second, multiplying both $z_1$ and $z_2$ by the same phase $e^{i\varphi}$ leaves $z_1\bar z_2$ and both moduli unchanged. That phase moves $q$ around a circle (the *fibre*) while $h(q)$ stays put. So the shadow alone cannot be inverted. Rebuilding $q$ needs the base point, the fibre phase and a chart convention, because there is no global identification $S^3=S^2\times S^1$.

**Kept fibre (Definition).** Holonomy tracking accumulates the wrapped change in fibre phase, plus the Fubini-Study distance, along a trajectory of states. In the integer runtime these are recorded per session and are marked telemetry only: they do not change any output.

**Sector routing (Definition).** Three Hopf coordinates are binned $8\times8\times8=512$ ways. When the inputs are non-negative at most 64 sectors are reachable, a fact of the binning (proof).

**Lorentz read (Definition).** Lift query $q$ and key $k$ to the hyperboloid, $q_0=\sqrt{1+|q|^2}$ and $k_0=\sqrt{1+|k|^2}$, and set $z=q_0k_0-q\cdot k$. The score is $\beta\,(\delta-\operatorname{arccosh} z)$, with a learned radius $\delta$ and scale $\beta$.

**Icosian shadows (from enumeration).** The 120 icosians map onto 30 points of the 2-sphere, every fibre of size 4 (checked by finite enumeration in [the geometry chapter](../geometry/hopf-observation.md)).

**Rule check.**
- *D11:* not yet. The integer helpers use multiplication and division, the core helpers decode to `f64`, and the integer runtime refuses Lorentz models until an integer arccosh table exists.
- *D23:* the map has no softmax. Every Lorentz read and attention that was actually run normalised scores with softmax, which is excluded.
- *D30:* the map is a few products, not a matrix multiplication. The float Lorentz stacks used dense query and key projections, which are barred.
- *D32/D37:* the float Hopf helpers (sine, cosine, square root) fall outside the zeta-zero exception. Under D37 a float version could be re-registered with a fresh review. The current discrete Route, Flock and Relation reads use no Hopf quantity ([the geometry chapter](../geometry/hopf-observation.md)).

## What has been tried

| Date | What was run | Result | Verdict | Record |
|---|---|---|---|---|
| 28 Jul | Ancestor router: angular state as a context carrier | Angular routing and decodability only, not generation; not reproduced under today's gates | inconclusive | [evidence note](../../archive/prime_router_geometric_context_evidence.md) |
| 31 Jul | Sector routing on held-out decisions, 1,500 decisions (#303) | 7 of 512 sectors occupied (1.4%); at most 64 reachable | negative | [occupancy](../../archive/hopf_sector_occupancy_d3.md) |
| 31 Jul | Same protocol with a fixed signed projection (#306) | 43 of 512 sectors (8.4%); retrieval quality left unmeasured | inconclusive | [remediation](../../archive/hopf_projection_remediation_306.md) |
| Aug | Hopf signed projection as a retrieval filter (#430) | Occupancy spread, no retrieval value: MRR 0.0045 against 0.0743 | negative | [#430 issue record](../../archive/history/github-archive-2026-10-15/markdown/0430-test-router-422-hopf-retrieval-quality-recorded-negative-occ.md) |
| 27–28 Aug | Recursive attention with an additive summary (#952), then equal-budget direct H4 attention (#973) | The summary erased earlier order. Full H4 got 3/12 while plain fixed tangent got 12/12, so geometry was not load-bearing. Both records say Hopf is not refuted | negative | [A1.0](../../archive/recursive_geometric_attention_a1_952.md), [V3](../../archive/direct_causal_geometric_attention_973.md) |
| undated | Paired-H4 heatmap readout (#970) | Heatmap classes aliased incompatible outcomes; gate stopped the run | negative | [record](../../archive/candidate_relative_identifiability_a1p_970.md) |
| undated | Softmax decoder through R4 frames, then learned Lorentz attention in SmolLM2 (HELM-D) | Parity passes. Lorentz scored NLL 7.71 against donor 3.668 and Euclidean 4.483 | negative | [parity](../../archive/helm_d_r4_softmax_decoder_973.md), [localization](../../archive/helm_d_score_centroid_localization_973.md) |
| 19 Sep | Zero the per-token 5-dimensional readout of the September prose model | +0.3377 bits per byte when removed (baseline 1.8055). Effects do not simply add; no pairwise factorial was run | positive | [evidence](../../archive/integration/EVIDENCE.md) |
| 26 Sep | Hyperbolic keys on real code-scope hierarchies | 77.5% of non-root queries at 16 dimensions against 0.4% for dot product; 36-bit keys kept 87% of accuracy; −0.020 bits per byte on code, −0.005 on prose | positive | [cycle 2](../../archive/integration/hyperbolic-cycle2-2026-09-26.md) |
| 26 Sep | Lorentz read in the Rust learner, 4 seeds, 2,000 updates | Ahead of dot in 3 of 4 seeds, mean 0.018 nats (0.007 bits per byte), below the 0.03 bar; the 600-step pilot overstated it | positive, small | [cycle 3](../../archive/integration/hyperbolic-cycle3-2026-09-26.md) |
| 27–28 Sep | Radial adaptation from a negative parent; Lorentz in the 7M-parameter stack | Neither radial arm beat dot. Lorentz minus dot −0.0001 | negative, neutral | [radial](../../archive/integration/radial-adaptation-result-2026-09-27.md), [toolbox](../../archive/integration/geometric-toolbox-2026-09-28.md) |
| 5–6 Oct | Generator selector arms (#1782), scored against a MiniLM teacher selector (a neural comparator; inactive evidence) | Hopf arm H 49.7%, flat arm S 50.8%, Hopf with the fibre kept (H_F) 50.8%: keeping the fibre recovered the 1.1-point gap (n=376, not powered). The lexical baseline got 55.3% and any selector was capped at 77.9% by the clause segmentation, so Phase 0 was dropped on other grounds | inconclusive | [#1782 record](../../archive/history/github-archive-2026-10-15/markdown/1782-phase-0-minilm-teacher-clause-retrieval-static-e8-tables-vs.md) |
| 12 Oct | Lorentz against its flat L2 ablation, 3 seeds | Ahead on every seed, but only one met the per-seed margin: wrong-value 49 to 32 of 120 | inconclusive | [geometric read](../../archive/labs/geometric-read-2026-10-11/README.md) |
| 11 Oct | Fixed circle phases for read binding, 2 seeds | Same bound mass as learned supervision, worse answers, 13 times slower | negative | [phase binding](../../archive/labs/phase-binding-2026-10-11/README.md) |

**Reading these honestly.** Only the occupancy rows test the Hopf map itself, and they tested one projection of one state. The attention failures were an additive summary, a heatmap readout, and softmax constructions; their records say Hopf and H4 are not refuted. The Lorentz read was neutral at best, and it was never run without softmax and dense projections. The strong hyperbolic result (77.5% against 0.4%) was about keys that index a hierarchy, not about scoring a read. The audit suggests splitting this page in two: the kept-fibre map is nearer READY than PARKED, while sector routing and the Lorentz read are the parked forms. The owner ruled the split on 24 Oct 2026 (D72). D22 applies: these close configurations, not the mechanism.

## What has never been tested

- **Kept fibre for language:** never tested in a language model (the one kept-fibre datum is the small selector comparison above). Base only against base plus fibre against explicit bank tags, at equal bytes.
- **Any Hopf quantity inside the discrete reads** (Route, Flock, Relation).
- **The S7 quaternionic Hopf relation state.** No implementation exists.
- **A pairwise factorial** separating the Hopf shadow channel from the predictor weights it interacts with (JEPA).
- **A third paired seed for Lorentz against dot**, and a Lorentz-scored pointer.
- **A Lorentz read without softmax or dense matrices.** Every executed form used both.
- **Hopf holonomy as a dialogue-state feature** that changes output. The integer tests assert nonzero winding, not usefulness.

## What would reopen it

1. **A retained-fibre test:** base only against base plus fibre against explicit bank tags, at equal retained bytes, with fibre removal or permutation, a common action, one endpoint changed against both, and exact occurrence identity kept outside the lossy summary.
2. **An integer, multiplier-free Hopf map** wired into a consumer on the discrete path.
3. **A Lorentz read re-registered as an offline D37 float sheet** with a fresh review, a read that avoids softmax and dense matrices, a third paired seed and a D11 export gap (an integer arccosh table does not exist yet).
4. **The reduced finite S7 construction**, only after a witnessed information or cost need.
5. **Hierarchy-heavy tasks** where hyperbolic keys already showed a large gap, put on an equal-capacity, softmax-free harness.

## Also known as

Hopf map $S^3\to S^2$; retained fibre; holonomy tracking; `UnitS3Q30::hopf` and `hopf_fiber_project`; Fubini-Study trajectory; Hopf sector routing (512 sectors); Lorentz or hyperbolic read; `LorentzAffine`; HELM-D R4 Lorentz attention; S7 quaternionic Hopf relation state (conditional, never built).

## Code

- [`native_geometric/hopf_metric.rs`](../../crates/uor-r4-core/src/native_geometric/hopf_metric.rs): the map, fibre point, Q30 versions and holonomy tracking.
- [`prime_route_attention.rs`](../../crates/uor-r4-core/src/prime_route_attention.rs): `UnitS3Q30::hopf`, `rotate_common_fiber`, `SpinTorsionState`.
- [`uor-r4-integer/src/math.rs`](../../crates/uor-r4-integer/src/math.rs) and [`model.rs`](../../crates/uor-r4-integer/src/model.rs): integer helpers, and the telemetry-only per-session state.
- [`uor-r4-router/tests/hopf_sector_occupancy.rs`](../../crates/uor-r4-router/tests/hopf_sector_occupancy.rs): the occupancy measurement.
- [`uor-r4-training/src/joint_model.rs`](../../crates/uor-r4-training/src/joint_model.rs): the Lorentz read score in the float stacks.
- [`helm_d_r4_attention.rs`](../../crates/uor-r4-core/src/helm_d_r4_attention.rs): the HELM-D constructions (softmax, now excluded).
- Existing static art: `docs/figures/geometry/hopf-fibration.svg`.

## Related mechanisms

[prime-zeta-codebook](./prime-zeta-codebook.md): shares the unit-quaternion helpers; the table uses points on the 3-sphere but not the Hopf map. [spin-weights](./spin-weights.md): acts with the same quaternions and icosian roots. [spiralcore-fbs](./spiralcore-fbs.md): both treat E8 and H4. [vsa](./vsa.md): another lossy sketch kept beside exact identity. [route-read](./route-read.md): hyperbolic keys were pitched as an address geometry. [flock-read](./flock-read.md): ranks by frame proximity, a discrete cousin of a neighbourhood score. [read-binding](./read-binding.md): phase binding and the Lorentz-against-L2 read both sat on it. [exact-memory](./exact-memory.md): exact occurrence identity must stay outside any Hopf summary.
